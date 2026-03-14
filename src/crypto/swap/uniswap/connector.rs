//! # Uniswap Connector
//!
//! Implementation of core traits for Uniswap DEX.
//!
//! ## Supported Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data (prices, orderbook, klines, tickers)
//!
//! ## Architecture Notes
//! - Uses Trading API for quotes/swaps (requires API key)
//! - Uses The Graph Subgraph for historical data (public or with API key)
//! - Uses Ethereum RPC for on-chain data (public or provider API key)
//! - No traditional orderbook - AMM-based pricing
//! - Token addresses used instead of symbols

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, GraphQlClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
};
use crate::core::traits::{ExchangeIdentity, MarketData, Trading, Account};
use crate::core::utils::WeightRateLimiter;
use crate::core::types::{
    ConnectorStats, SymbolInfo,
    OrderRequest, CancelRequest, Order, OrderHistoryFilter, PlaceOrderResponse,
    BalanceQuery, Balance, AccountInfo, FeeInfo,
};

use super::endpoints::{UniswapUrls, UniswapEndpoint, format_token_address, find_pool_metadata, PoolMetadata};
use super::auth::UniswapAuth;
use super::parser::UniswapParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Uniswap DEX connector
pub struct UniswapConnector {
    /// HTTP client (REST + JSON-RPC)
    http: HttpClient,
    /// GraphQL client for The Graph subgraph queries
    graphql: GraphQlClient,
    /// Authentication (optional for public endpoints)
    auth: UniswapAuth,
    /// URLs (mainnet/testnet)
    urls: UniswapUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (12 requests per second for Trading API)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl UniswapConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            UniswapUrls::TESTNET
        } else {
            UniswapUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = if let Some(creds) = credentials {
            UniswapAuth::new(&creds)?
        } else {
            UniswapAuth::public()
        };

        // Build GraphQL client for The Graph subgraph.
        // The base subgraph URL is the public gateway; when an API key is set,
        // `auth.subgraph_url()` returns a key-injected URL used at query time.
        // We use the public base URL here so the GraphQlClient can be constructed
        // once; per-query URL override goes via `query_subgraph()`.
        let graphql = GraphQlClient::new(
            HttpClient::new(30_000)?,
            urls.subgraph_v3,
        );

        // Initialize rate limiter: 720 requests per 60 seconds
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(720, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            graphql,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Create connector for public endpoints only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(weight) {
                    return;
                }
                limiter.time_until_ready(weight)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// POST request to Trading API
    async fn post_trading_api(
        &self,
        endpoint: UniswapEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        let base_url = self.urls.api_url(endpoint);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let headers = if endpoint.requires_auth() {
            self.auth.trading_api_headers()?
        } else {
            UniswapAuth::public_headers()
        };

        let response = self.http.post(&url, &body, &headers).await?;
        UniswapParser::check_response(&response)?;

        Ok(response)
    }

    /// POST GraphQL query to The Graph Subgraph via `GraphQlClient`.
    ///
    /// Uses the pre-built `self.graphql` client (pointing at the public base URL).
    /// When a The Graph API key is configured in `self.auth`, use
    /// `query_subgraph()` instead — it resolves the key-injected URL per call.
    async fn post_subgraph_query(&self, query: &str) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        let headers = UniswapAuth::public_headers();
        let response = self.graphql.query_with_headers(query, None, headers).await?;
        UniswapParser::check_response(&response)?;

        Ok(response)
    }

    /// Query The Graph subgraph with optional GraphQL variables.
    ///
    /// Resolves the subgraph URL at call time so that a configured The Graph
    /// API key is injected into the URL path.  Falls back to the public
    /// gateway when no key is set.
    ///
    /// Prefer this over `post_subgraph_query` when using GraphQL variables.
    pub async fn query_subgraph(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        // Resolve endpoint with optional API key injected into the URL
        let endpoint = self.auth.subgraph_url(self.urls.subgraph_v3)?;
        let headers = UniswapAuth::public_headers();

        // Temporarily delegate through HttpClient to hit the key-injected URL
        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(serde_json::json!({}))
        });
        let response = self.http.post(&endpoint, &body, &headers).await?;
        UniswapParser::check_response(&response)?;

        Ok(response)
    }

    /// POST JSON-RPC request to Ethereum node
    async fn post_eth_rpc(&self, method: &str, params: Vec<Value>) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        let rpc_url = self.auth.rpc_url(self.urls.eth_rpc);

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let headers = UniswapAuth::public_headers();

        let response = self.http.post(&rpc_url, &body, &headers).await?;
        UniswapParser::check_response(&response)?;

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POOL QUERIES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get pool address for a token pair
    fn get_pool_address(&self, base: &str, quote: &str) -> ExchangeResult<String> {
        if let Some(pool_meta) = find_pool_metadata(base, quote) {
            Ok(pool_meta.address.to_string())
        } else {
            Err(ExchangeError::NotSupported(format!(
                "Pool not found for {}/{}. Supported pairs: WETH/USDC, WETH/USDT, WBTC/WETH",
                base, quote
            )))
        }
    }

    /// Query pool data from subgraph
    async fn query_pool(&self, pool_address: &str) -> ExchangeResult<Value> {
        let query = format!(
            r#"{{
                pool(id: "{}") {{
                    id
                    token0 {{
                        id
                        symbol
                        decimals
                    }}
                    token1 {{
                        id
                        symbol
                        decimals
                    }}
                    feeTier
                    liquidity
                    sqrtPrice
                    tick
                    volumeUSD
                    totalValueLockedUSD
                }}
            }}"#,
            pool_address.to_lowercase()
        );

        self.post_subgraph_query(&query).await
    }

    /// Query swaps from subgraph
    async fn query_swaps(&self, pool_address: &str, limit: u16) -> ExchangeResult<Value> {
        let query = format!(
            r#"{{
                swaps(
                    first: {}
                    orderBy: timestamp
                    orderDirection: desc
                    where: {{ pool: "{}" }}
                ) {{
                    id
                    timestamp
                    amount0
                    amount1
                    amountUSD
                    sqrtPriceX96
                    tick
                }}
            }}"#,
            limit,
            pool_address.to_lowercase()
        );

        self.post_subgraph_query(&query).await
    }

    /// Query all pools from subgraph
    async fn query_all_pools(&self, limit: u16) -> ExchangeResult<Value> {
        let query = format!(
            r#"{{
                pools(
                    first: {}
                    orderBy: totalValueLockedUSD
                    orderDirection: desc
                    where: {{ volumeUSD_gt: "1000000" }}
                ) {{
                    id
                    token0 {{
                        symbol
                    }}
                    token1 {{
                        symbol
                    }}
                    feeTier
                    liquidity
                    volumeUSD
                }}
            }}"#,
            limit
        );

        self.post_subgraph_query(&query).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RPC-BASED PRICE FETCHING (NO API KEY REQUIRED)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get price via direct RPC call to pool contract's slot0()
    ///
    /// This method works WITHOUT any API keys by calling the pool contract directly.
    /// It uses the pool metadata registry to know token decimals.
    async fn get_price_via_rpc(&self, pool_meta: &PoolMetadata, base: &str, _quote: &str) -> ExchangeResult<Price> {
        // Call slot0() on the pool contract
        // Function selector: 0x3850c7bd
        let params = vec![
            json!({
                "to": pool_meta.address,
                "data": "0x3850c7bd"
            }),
            json!("latest"),
        ];

        let response = self.post_eth_rpc("eth_call", params).await?;

        // Extract result
        let result = response
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing slot0 result".to_string()))?;

        // Parse sqrtPriceX96 from first 32 bytes (64 hex chars after "0x")
        if result.len() < 66 {
            return Err(ExchangeError::Parse(format!(
                "slot0 result too short: {}",
                result
            )));
        }

        let sqrt_price_hex = &result[2..66]; // Skip "0x", take 64 hex chars
        let sqrt_price_x96 = u128::from_str_radix(sqrt_price_hex, 16)
            .map_err(|e| ExchangeError::Parse(format!("Invalid sqrtPriceX96 hex: {}", e)))?;

        // Determine if we need to invert the price
        // pool returns token1/token0 price
        // We need to check which token the user wants as base
        let base_norm = if base == "ETH" { "WETH" } else { base };

        // Determine which token price the user wants
        // If base matches token1, user wants token1 price
        // If base matches token0, user wants token0 price
        let want_token1_price = base_norm == pool_meta.token1_symbol;

        // Calculate human-readable price
        let price = UniswapParser::sqrt_price_x96_to_human_price(
            sqrt_price_x96,
            pool_meta.token0_decimals,
            pool_meta.token1_decimals,
            want_token1_price,
        )?;

        Ok(price)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Uniswap-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get quote for swap (requires Trading API key)
    pub async fn get_quote(
        &self,
        token_in: &str,
        token_out: &str,
        amount: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        let token_in_addr = format_token_address(token_in, account_type);
        let token_out_addr = format_token_address(token_out, account_type);

        let body = json!({
            "type": "EXACT_INPUT",
            "amount": amount,
            "tokenInChainId": self.urls.chain_id,
            "tokenOutChainId": self.urls.chain_id,
            "tokenIn": token_in_addr,
            "tokenOut": token_out_addr,
            "slippageTolerance": 0.5,
            "routingPreference": "BEST_PRICE"
        });

        self.post_trading_api(UniswapEndpoint::Quote, body).await
    }

    /// Get token balance (via Ethereum RPC)
    pub async fn get_token_balance(
        &self,
        token_address: &str,
        wallet_address: &str,
    ) -> ExchangeResult<f64> {
        // Call ERC-20 balanceOf method
        let data = format!(
            "0x70a08231000000000000000000000000{}",
            wallet_address.trim_start_matches("0x")
        );

        let params = vec![
            json!({
                "to": token_address,
                "data": data
            }),
            json!("latest"),
        ];

        let response = self.post_eth_rpc("eth_call", params).await?;

        // Parse balance from response
        let result = response
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;

        let hex_str = result.trim_start_matches("0x");
        let balance_wei = u128::from_str_radix(hex_str, 16)
            .map_err(|e| ExchangeError::Parse(format!("Invalid hex: {}", e)))?;

        // Assume 18 decimals (should query token.decimals() in production)
        let balance = balance_wei as f64 / 1e18;

        Ok(balance)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for UniswapConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Uniswap
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            (limiter.current_weight(), limiter.max_weight())
        } else {
            (0, 0)
        };
        ConnectorStats {
            http_requests,
            http_errors,
            last_latency_ms,
            rate_used,
            rate_max,
            rate_groups: Vec::new(),
            ws_ping_rtt_ms: 0,
        }
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // Uniswap is spot-only
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for UniswapConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Try to find pool metadata in our known pools registry
        if let Some(pool_meta) = find_pool_metadata(&symbol.base, &symbol.quote) {
            // Try RPC first (no API key needed, always available)
            match self.get_price_via_rpc(pool_meta, &symbol.base, &symbol.quote).await {
                Ok(price) => return Ok(price),
                Err(e) => {
                    tracing::warn!("RPC price fetch failed, falling back to subgraph: {}", e);
                }
            }
        }

        // Fallback to subgraph (requires API key or uses public endpoint)
        let pool_address = self.get_pool_address(&symbol.base, &symbol.quote)?;
        let response = self.query_pool(&pool_address).await?;
        UniswapParser::parse_pool_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Get pool address
        let pool_address = self.get_pool_address(&symbol.base, &symbol.quote)?;

        // Query pool data
        let response = self.query_pool(&pool_address).await?;

        // Simulate orderbook from pool liquidity
        UniswapParser::parse_orderbook_from_pool(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        _interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        // Get pool address
        let pool_address = self.get_pool_address(&symbol.base, &symbol.quote)?;

        // Query recent swaps
        let limit = limit.unwrap_or(100);
        let response = self.query_swaps(&pool_address, limit).await?;

        // Convert swaps to klines
        UniswapParser::parse_klines_from_swaps(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        // Get pool address
        let pool_address = self.get_pool_address(&symbol.base, &symbol.quote)?;

        // Query pool data
        let response = self.query_pool(&pool_address).await?;

        // Parse ticker
        UniswapParser::parse_ticker(&response, &symbol.to_string())
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Ping Ethereum RPC
        let response = self.post_eth_rpc("eth_blockNumber", vec![]).await?;

        if response.get("result").is_some() {
            Ok(())
        } else {
            Err(ExchangeError::Network("Ping failed".to_string()))
        }
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Query top pools from the subgraph
        let response = self.query_all_pools(100).await?;
        let pairs = UniswapParser::parse_trading_pairs(&response)?;

        let infos = pairs.into_iter().map(|(base, quote)| {
            let symbol = format!("{}/{}", base, quote);
            SymbolInfo {
                symbol,
                base_asset: base,
                quote_asset: quote,
                status: "TRADING".to_string(),
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: None,
                max_quantity: None,
                tick_size: None,
                step_size: None,
                min_notional: None,
            }
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

// Uniswap swap execution requires:
// 1. Ethereum wallet / signer (ethers-rs or alloy)
// 2. Wallet private key for EIP-712 typed-data signing
// 3. ERC-20 token approvals (Permit2 or direct approve)
// 4. Submit signed transaction to Ethereum mempool
// 5. Wait for on-chain confirmation
//
// The Trading API provides unsigned transaction calldata only.
// Signing and broadcasting are out of scope for a REST connector.

#[async_trait]
impl Trading for UniswapConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap swap execution requires Ethereum wallet integration (ethers-rs/alloy). \
             Use get_quote() to obtain calldata, then sign and broadcast via Ethereum RPC."
                .to_string(),
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap swaps are atomic on-chain transactions — they cannot be cancelled. \
             Transactions either succeed or revert."
                .to_string(),
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap has no order tracking. \
             Use Ethereum transaction hash to check swap status via eth_getTransactionReceipt."
                .to_string(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap AMM swaps are atomic — there are no open/pending orders. \
             Limit orders require separate protocol integration (e.g., Uniswap X)."
                .to_string(),
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap does not provide order history via REST API. \
             Query swap history via The Graph subgraph with wallet address."
                .to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for UniswapConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap has no account system. \
             Query ERC-20 token balances via eth_call (balanceOf) or ETH balance via eth_getBalance."
                .to_string(),
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap is a permissionless AMM — there is no account concept or registration. \
             Use wallet address to query on-chain data."
                .to_string(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Uniswap uses a fixed pool fee tier model (not maker/taker):
        // - V3 pools: 0.01%, 0.05%, 0.30%, or 1.00% per swap
        // - Fee goes 100% to LPs; protocol fee is 0% (can be enabled by governance)
        // Not translatable to FeeInfo maker/taker structure.
        Err(ExchangeError::UnsupportedOperation(
            "Uniswap uses pool fee tiers (0.01%/0.05%/0.30%/1.00%) paid to LPs, not maker/taker rates. \
             Fee tier is per pool — query pool's feeTier via The Graph subgraph."
                .to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (not in trait)
// ═══════════════════════════════════════════════════════════════════════════════

impl UniswapConnector {
    /// Get all available trading pairs
    pub async fn get_trading_pairs(&self) -> ExchangeResult<Vec<Symbol>> {
        // Query top pools from subgraph
        let response = self.query_all_pools(100).await?;

        // Parse pairs
        let pairs = UniswapParser::parse_trading_pairs(&response)?;

        // Convert to Symbol
        let symbols = pairs
            .into_iter()
            .map(|(base, quote)| Symbol::new(base, quote))
            .collect();

        Ok(symbols)
    }
}
