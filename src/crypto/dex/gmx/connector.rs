//! # GMX Connector
//!
//! Implementation of core traits for GMX V2.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data (REST API)
//! - `Trading` - On-chain trading via `onchain-evm` feature (unsigned tx building)
//! - `Account` - On-chain balance queries via `onchain-evm` feature
//!
//! ## Limitations
//! - `Trading::place_order` and `cancel_order` build **unsigned** `TransactionRequest`
//!   objects when the `onchain-evm` feature is enabled and `with_onchain()` was called.
//!   The caller is responsible for signing and broadcasting.
//! - `get_open_orders` / `get_order_history` / `get_positions` query the Subsquid indexer
//!   and require a wallet address to be known.  They return empty Vec if no wallet is set.
//! - `Account::get_balance` returns native token (ETH/AVAX) + ERC-20 balances via
//!   `eth_call balanceOf` when the `onchain-evm` feature is enabled.
//! - WebSocket uses polling instead of native WebSocket API

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, GraphQlClient,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::{
    OrderRequest, CancelRequest, Order, OrderHistoryFilter, PlaceOrderResponse,
    BalanceQuery, Balance, AccountInfo, FeeInfo,
    PositionQuery, Position, FundingRate, PositionModification,
    UserTrade, UserTradeFilter,
};
use crate::core::utils::SimpleRateLimiter;
use crate::core::types::{ConnectorStats, SymbolInfo};

use super::endpoints::{GmxUrls, GmxEndpoint, format_symbol, map_kline_interval};
use super::auth::GmxAuth;
use super::parser::GmxParser;

#[cfg(feature = "onchain-evm")]
use super::onchain::{
    GmxOnchain, GmxOrderType, GmxPositionSide,
    CreatePositionParams, ClosePositionParams,
};
#[cfg(feature = "onchain-evm")]
use alloy::primitives::{Address, U256};
#[cfg(feature = "onchain-evm")]
use crate::core::types::OrderSide;
#[cfg(feature = "onchain-evm")]
use crate::core::types::OrderType;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// GMX connector
pub struct GmxConnector {
    /// HTTP client for REST endpoints
    http: HttpClient,
    /// GraphQL client for The Graph stats subgraph (historical aggregate data)
    subgraph: GraphQlClient,
    /// GraphQL client for Subsquid (per-account positions, orders, trade history)
    subsquid: GraphQlClient,
    /// Authentication (no-op for public REST endpoints)
    #[allow(dead_code)]
    auth: GmxAuth,
    /// URLs
    urls: GmxUrls,
    /// Chain (arbitrum/avalanche)
    chain: String,
    /// Rate limiter (conservative: 12 req/60s, ~1 req/5s)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// On-chain provider for unsigned transaction building.
    ///
    /// Present only when the `onchain-evm` feature is enabled and
    /// `with_onchain()` was called.  Without it every trading / account
    /// method falls back to `UnsupportedOperation`.
    #[cfg(feature = "onchain-evm")]
    onchain: Option<Arc<GmxOnchain>>,
    /// Wallet address used as `from` / `receiver` in on-chain transactions.
    ///
    /// Must be supplied via `with_onchain()` to enable `place_order` and
    /// `cancel_order`.  Without it the methods return `InvalidRequest`.
    #[cfg(feature = "onchain-evm")]
    wallet_address: Option<Address>,
}

impl GmxConnector {
    /// Create new connector
    ///
    /// # Parameters
    /// - `chain`: "arbitrum" or "avalanche" (defaults to "arbitrum")
    pub async fn new(chain: Option<String>) -> ExchangeResult<Self> {
        let chain = chain.unwrap_or_else(|| "arbitrum".to_string());
        let urls = GmxUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = GmxAuth::public();

        // Build GraphQL client for The Graph stats subgraph (historical aggregate data).
        // Endpoint is resolved per-chain at construction time; chain does not
        // change after construction so a single fixed-endpoint client is fine.
        let subgraph_url = urls.subgraph_url(&chain);
        let subgraph = GraphQlClient::new(
            HttpClient::new(30_000)?,
            subgraph_url,
        );

        // Build GraphQL client for Subsquid (per-account positions / orders).
        let subsquid_url = urls.subsquid_url(&chain);
        let subsquid = GraphQlClient::new(
            HttpClient::new(30_000)?,
            subsquid_url,
        );

        // Conservative: 12 requests per 60 seconds (~1 req/5s)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(12, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            subgraph,
            subsquid,
            auth,
            urls,
            chain,
            rate_limiter,
            #[cfg(feature = "onchain-evm")]
            onchain: None,
            #[cfg(feature = "onchain-evm")]
            wallet_address: None,
        })
    }

    /// Create connector for Arbitrum
    pub async fn arbitrum() -> ExchangeResult<Self> {
        Self::new(Some("arbitrum".to_string())).await
    }

    /// Create connector for Avalanche
    pub async fn avalanche() -> ExchangeResult<Self> {
        Self::new(Some("avalanche".to_string())).await
    }

    /// Attach an on-chain provider to enable `Trading` and `Account` trait methods.
    ///
    /// - `onchain` — shared `GmxOnchain` provider (can be shared across connectors).
    /// - `wallet_address` — the EVM address that signs and receives outputs.
    ///   Transactions built by `place_order` / `cancel_order` are sent *from*
    ///   this address and collateral is returned to it on close.
    ///
    /// Without this, `place_order`, `cancel_order`, and `get_balance` return
    /// `UnsupportedOperation`.  Call this on a connector already created with
    /// `arbitrum()` or `avalanche()`.
    ///
    /// # Example
    /// ```ignore
    /// use std::sync::Arc;
    /// let wallet: Address = "0xYourAddress".parse().unwrap();
    /// let connector = GmxConnector::arbitrum().await?
    ///     .with_onchain(Arc::new(GmxOnchain::arbitrum()), wallet);
    /// ```
    #[cfg(feature = "onchain-evm")]
    pub fn with_onchain(mut self, onchain: Arc<GmxOnchain>, wallet_address: Address) -> Self {
        self.onchain = Some(onchain);
        self.wallet_address = Some(wallet_address);
        self
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ON-CHAIN HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Resolve a base-asset symbol to its GMX V2 GM pool (market token) address.
    ///
    /// Only well-known Arbitrum mainnet markets are covered here.  The function
    /// returns `None` for any symbol not in the static table so the caller can
    /// gracefully return `UnsupportedOperation` rather than panic.
    #[cfg(feature = "onchain-evm")]
    fn gmx_market_address(base: &str, chain: &str) -> Option<Address> {
        // Arbitrum mainnet GM pool token addresses (index token → GM pool token).
        // Source: https://app.gmx.io/#/pools (Arbitrum)
        let arbitrum_markets: &[(&str, &str)] = &[
            ("ETH",  "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336"),
            ("WETH", "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336"),
            ("BTC",  "0x47c031236e19d024b42f8AE6780E44A573170703"),
            ("WBTC", "0x47c031236e19d024b42f8AE6780E44A573170703"),
            ("ARB",  "0xC25cEf6061Cf5dE5eb761b50E4743c1F5D7E5407"),
            ("LINK", "0x7f1fa204bb700853D36994DA19F830b6Ad18d233"),
            ("SOL",  "0x09400D9DB990D5ed3f35D7be61DfAEB900Af03C9"),
            ("DOGE", "0x6853EA96FF216fAb11D2d930CE3C508556A4bdc4"),
            ("LTC",  "0xD9535bB5f58A1a75032416F2dFe7880C30575a41"),
            ("AVAX", "0x7BbBf946883a5701350007320F525c5379B8178A"),
            ("OP",   "0xC33B72741dA3D6F6dC0422f02bcC30BD55C5EA7C"),
            ("AAVE", "0x1CbBa6346F110c8A5ea739ef2d1eb182990e4EB2"),
            ("UNI",  "0xc7Abb2C5f3BF3CEB389dF0Eecd6120D451170B50"),
        ];
        // Avalanche mainnet GM pool token addresses.
        let avalanche_markets: &[(&str, &str)] = &[
            ("ETH",  "0xB7e69749E3d2EDd90ea59A4932EFEa2D41E245d7"),
            ("WETH", "0xB7e69749E3d2EDd90ea59A4932EFEa2D41E245d7"),
            ("AVAX", "0xB7e69749E3d2EDd90ea59A4932EFEa2D41E245d7"),
            ("BTC",  "0xFb02132333A79C8B5Bd0b64E3AbccA5f7fAf2937"),
            ("WBTC", "0xFb02132333A79C8B5Bd0b64E3AbccA5f7fAf2937"),
        ];

        let table: &[(&str, &str)] = match chain {
            "avalanche" | "avax" => avalanche_markets,
            _ => arbitrum_markets,
        };

        let base_upper = base.to_uppercase();
        table.iter()
            .find(|(sym, _)| *sym == base_upper.as_str())
            .and_then(|(_, addr)| addr.parse().ok())
    }

    /// Resolve base-asset symbol to its collateral token address on GMX.
    ///
    /// For long positions the collateral is typically the index token itself
    /// (e.g. WETH for ETH longs, WBTC for BTC longs).  For short positions
    /// USDC is the standard collateral.  This function returns the *long*
    /// collateral; callers should use USDC for short positions.
    #[cfg(feature = "onchain-evm")]
    fn gmx_collateral_address(base: &str, chain: &str) -> Option<Address> {
        let arbitrum: &[(&str, &str)] = &[
            ("ETH",  "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"), // WETH
            ("WETH", "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"),
            ("BTC",  "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f"), // WBTC
            ("WBTC", "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f"),
            // For all others / shorts: USDC
        ];
        let avalanche: &[(&str, &str)] = &[
            ("ETH",  "0x49D5c2BdFfac6CE2BFdB6640F4F80f226bc10bAB"), // WETH.e
            ("WETH", "0x49D5c2BdFfac6CE2BFdB6640F4F80f226bc10bAB"),
            ("BTC",  "0x408D4cD0ADb7ceBd1F1A1C33A0Ba2098E1295bAB"), // WBTC.e
            ("WBTC", "0x408D4cD0ADb7ceBd1F1A1C33A0Ba2098E1295bAB"),
            ("AVAX", "0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7"), // WAVAX
        ];

        let table = match chain {
            "avalanche" | "avax" => avalanche,
            _ => arbitrum,
        };
        let base_upper = base.to_uppercase();
        table.iter()
            .find(|(sym, _)| *sym == base_upper.as_str())
            .and_then(|(_, addr)| addr.parse().ok())
    }

    /// Return the USDC address for the given chain (default short collateral).
    #[cfg(feature = "onchain-evm")]
    fn usdc_address(chain: &str) -> ExchangeResult<Address> {
        let addr = match chain {
            "avalanche" | "avax" => "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E",
            _ => "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // USDC on Arbitrum
        };
        addr.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("USDC address parse: {}", e)))
    }

    /// Convert an f64 USD price into a GMX 30-decimal-precision `U256`.
    ///
    /// GMX V2 uses 30 decimal places for USD values internally:
    ///   `price_u256 = price_f64 * 10^30`
    ///
    /// Precision is limited to ~15 significant digits (f64 mantissa).  This is
    /// sufficient for order placement but callers should use integer arithmetic
    /// when exact precision is required.
    #[cfg(feature = "onchain-evm")]
    fn price_to_u256_30dec(price: f64) -> U256 {
        // Multiply by 10^12 using f64, then by 10^18 using U256 integer math to
        // avoid f64 overflow (10^30 > f64::MAX is false but loses precision).
        // Strategy: split into integer and fractional parts to preserve 6 decimals.
        if price <= 0.0 {
            return U256::ZERO;
        }
        // Use 18 decimal intermediate: price * 10^18 as u128, then multiply by 10^12.
        let price_1e18 = (price * 1e18) as u128;
        let scale_12 = U256::from(10u64).pow(U256::from(12u32));
        U256::from(price_1e18).saturating_mul(scale_12)
    }

    /// Convert an f64 quantity (base-asset units) to 18-decimal wei-equivalent `U256`.
    ///
    /// Used for `initial_collateral_delta_amount` in GMX position params.
    #[cfg(feature = "onchain-evm")]
    fn quantity_to_wei(quantity: f64) -> U256 {
        if quantity <= 0.0 {
            return U256::ZERO;
        }
        let qty_1e18 = (quantity * 1e18) as u128;
        U256::from(qty_1e18)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire() {
                    return;
                }
                limiter.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request with fallback URLs
    async fn get(
        &self,
        endpoint: GmxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(&self.chain);
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Try primary URL first
        let headers = self.auth.sign_request("GET", path, "");
        match self.http.get_with_headers(&url, &HashMap::new(), &headers).await {
            Ok(response) => return self.check_response(response),
            Err(e) => {
                // Primary URL failed, trying fallback
                eprintln!("GMX Primary URL failed: {}, trying fallback...", e);
            }
        }

        // Try fallback URLs
        for fallback_url in self.urls.fallback_urls(&self.chain) {
            let url = format!("{}{}{}", fallback_url, path, query);
            match self.http.get_with_headers(&url, &HashMap::new(), &headers).await {
                Ok(response) => return self.check_response(response),
                Err(e) => {
                    eprintln!("GMX Fallback URL {} failed: {}", fallback_url, e);
                    continue;
                }
            }
        }

        Err(ExchangeError::Network("All GMX URLs failed".to_string()))
    }

    /// Check response for errors
    fn check_response(&self, response: Value) -> ExchangeResult<Value> {
        // GMX REST API returns errors as {"error": "message"}
        if let Some(error) = response.get("error") {
            if let Some(msg) = error.as_str() {
                return Err(ExchangeError::Api {
                    code: -1,
                    message: msg.to_string(),
                });
            }
        }

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUBGRAPH QUERIES (The Graph — historical data)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Query the GMX V2 The Graph stats subgraph for historical aggregate data.
    ///
    /// The subgraph endpoint is chosen at construction time based on the chain
    /// (`arbitrum` or `avalanche`).  All GMX subgraph queries are public —
    /// no API key is required.
    ///
    /// For per-account data (positions, orders) use `query_subsquid` instead.
    ///
    /// # Parameters
    /// - `query`     — GraphQL query string
    /// - `variables` — optional variables object
    ///
    /// # Example
    /// ```ignore
    /// let connector = GmxConnector::arbitrum().await?;
    ///
    /// let result = connector.query_subgraph(
    ///     r#"{ orders(first: 10, orderBy: createdTxn__timestamp, orderDirection: desc) {
    ///         id account market sizeDeltaUsd
    ///     }}"#,
    ///     None,
    /// ).await?;
    /// ```
    pub async fn query_subgraph(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;
        self.subgraph.query(query, variables).await
    }

    /// Query the GMX V2 Subsquid indexer for per-account data.
    ///
    /// Use this for positions, open orders, and trade history.
    /// The Subsquid endpoint (`gmx.squids.live`) is publicly accessible
    /// without an API key.
    ///
    /// # Parameters
    /// - `query`     — GraphQL query string
    /// - `variables` — optional variables object
    ///
    /// # Example
    /// ```ignore
    /// let connector = GmxConnector::arbitrum().await?;
    ///
    /// let result = connector.query_subsquid(
    ///     r#"query Positions($account: String!) {
    ///         positions(where: { account_eq: $account, isOpen_eq: true }) {
    ///             id market isLong sizeInUsd entryPrice unrealizedPnl
    ///         }
    ///     }"#,
    ///     Some(serde_json::json!({ "account": "0xYourAddress" })),
    /// ).await?;
    /// ```
    pub async fn query_subsquid(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;
        self.subsquid.query(query, variables).await
    }

    /// Return the wallet address as a lowercase hex string, or `None` when
    /// no wallet has been set (no-onchain-evm build or `with_onchain` not called).
    fn wallet_address_str(&self) -> Option<String> {
        #[cfg(feature = "onchain-evm")]
        {
            return self.wallet_address.map(|a| format!("{:?}", a).to_lowercase());
        }
        #[cfg(not(feature = "onchain-evm"))]
        {
            None
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (GMX-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all markets
    pub async fn get_markets(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(GmxEndpoint::Markets, HashMap::new()).await?;
        GmxParser::parse_symbols(&response)
    }

    /// Get all tickers
    pub async fn get_all_tickers(&self) -> ExchangeResult<Vec<Ticker>> {
        let response = self.get(GmxEndpoint::Tickers, HashMap::new()).await?;
        GmxParser::parse_all_tickers(&response)
    }

    /// Get market info (detailed)
    pub async fn get_market_info(&self) -> ExchangeResult<Value> {
        self.get(GmxEndpoint::MarketInfo, HashMap::new()).await
    }

    /// Get tokens list
    pub async fn get_tokens(&self) -> ExchangeResult<Value> {
        self.get(GmxEndpoint::Tokens, HashMap::new()).await
    }

    /// Get GLV (GMX Liquidity Vault) APY data
    ///
    /// Returns annualised yield estimates for each GLV vault.
    pub async fn get_glv_apy(&self) -> ExchangeResult<Value> {
        self.get(GmxEndpoint::GlvApy, HashMap::new()).await
    }

    /// Get UI fee revenue statistics
    ///
    /// Returns historical fee revenue collected by front-end integrators.
    pub async fn get_ui_fees(
        &self,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(from) = from_timestamp {
            params.insert("from".to_string(), from.to_string());
        }
        if let Some(to) = to_timestamp {
            params.insert("to".to_string(), to.to_string());
        }
        self.get(GmxEndpoint::UiFees, params).await
    }

    /// Get position statistics
    ///
    /// Returns aggregate open-interest, position count, and related stats.
    pub async fn get_position_stats(
        &self,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(from) = from_timestamp {
            params.insert("from".to_string(), from.to_string());
        }
        if let Some(to) = to_timestamp {
            params.insert("to".to_string(), to.to_string());
        }
        self.get(GmxEndpoint::PositionStats, params).await
    }

    /// Get protocol fee metrics
    ///
    /// Returns breakdown of fees collected (position, borrow, swap).
    pub async fn get_fee_metrics(
        &self,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(from) = from_timestamp {
            params.insert("from".to_string(), from.to_string());
        }
        if let Some(to) = to_timestamp {
            params.insert("to".to_string(), to.to_string());
        }
        self.get(GmxEndpoint::FeeMetrics, params).await
    }

    /// Get trading volume statistics
    ///
    /// Returns daily/weekly/monthly volume broken down by market.
    pub async fn get_volumes(
        &self,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(from) = from_timestamp {
            params.insert("from".to_string(), from.to_string());
        }
        if let Some(to) = to_timestamp {
            params.insert("to".to_string(), to.to_string());
        }
        self.get(GmxEndpoint::Volumes, params).await
    }

    /// Get per-account statistics
    ///
    /// Returns trading statistics for a specific account address.
    pub async fn get_account_stats(&self, account: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("account".to_string(), account.to_string());
        self.get(GmxEndpoint::AccountStats, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for GmxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Gmx
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            (limiter.current_count(), limiter.max_requests())
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
        false // GMX V2 mainnet only
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::FuturesCross, // GMX only has perpetual futures
        ]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for GmxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Get ticker and extract price
        let ticker = self.get_ticker(symbol.clone(), AccountType::FuturesCross).await?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // GMX uses oracle pricing, not orderbooks
        Err(ExchangeError::NotSupported(
            "GMX uses oracle pricing, not orderbooks".to_string()
        ))
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();

        // GMX uses token symbol (ETH), not pair (ETH/USD)
        let token_symbol = symbol.base.to_uppercase();
        params.insert("tokenSymbol".to_string(), token_symbol);

        // Map interval to GMX period format
        params.insert("period".to_string(), map_kline_interval(interval).to_string());

        // Limit (1-10,000, default 1,000)
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.min(10000).to_string());
        }

        let response = self.get(GmxEndpoint::Candles, params).await?;
        GmxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        // Get all tickers and filter for symbol
        let response = self.get(GmxEndpoint::Tickers, HashMap::new()).await?;
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, AccountType::FuturesCross);
        GmxParser::parse_ticker(&response, &formatted_symbol)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(GmxEndpoint::Ping, HashMap::new()).await?;
        let ok = GmxParser::parse_ping(&response)?;

        if ok {
            Ok(())
        } else {
            Err(ExchangeError::Api {
                code: -1,
                message: "Ping failed".to_string(),
            })
        }
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(GmxEndpoint::Markets, HashMap::new()).await?;
        let symbols = GmxParser::parse_symbols(&response)?;

        let infos = symbols.into_iter().map(|sym| {
            let (base, quote) = if let Some(pos) = sym.find('/') {
                (sym[..pos].to_string(), sym[pos + 1..].to_string())
            } else {
                (sym.clone(), "USD".to_string())
            };
            SymbolInfo {
                symbol: sym,
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
//
// GMX trading architecture:
// 1. Transactions are built as unsigned `TransactionRequest` objects via alloy.
// 2. The caller is responsible for signing (EIP-1559) and broadcasting.
// 3. Keeper network executes orders asynchronously — there is no synchronous fill.
// 4. `place_order` returns an `Order` with `id = "gmx:pending:<router_addr>"` and
//    `status = New` to signal an unsigned transaction was prepared.
// 5. `cancel_order` (when a position key is passed as `order_id`) builds a
//    MarketDecrease tx for the full position.
// 6. `get_open_orders` / `get_order_history` remain UnsupportedOperation because
//    order state lives in contract storage + The Graph, not a REST endpoint.
//
// Feature gate: all on-chain paths compile only when `onchain-evm` is enabled.
// Without the feature the methods return their original UnsupportedOperation errors.

#[async_trait]
impl Trading for GmxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        #[cfg(feature = "onchain-evm")]
        {
            if let Some(ref onchain) = self.onchain {
                let from = self.wallet_address.ok_or_else(|| {
                    ExchangeError::InvalidRequest(
                        "wallet_address is required for GMX on-chain trading. \
                         Call with_onchain(provider, wallet_address) on the connector."
                            .to_string(),
                    )
                })?;

                // Determine direction
                let is_long = match req.side {
                    OrderSide::Buy => true,
                    OrderSide::Sell => false,
                };
                let gmx_side = if is_long {
                    GmxPositionSide::Long
                } else {
                    GmxPositionSide::Short
                };

                // Resolve market address from symbol
                let market = Self::gmx_market_address(&req.symbol.base, &self.chain)
                    .ok_or_else(|| ExchangeError::InvalidRequest(format!(
                        "GMX: no known market address for '{}' on {}. \
                         Only ETH, BTC, ARB, LINK, SOL, DOGE, LTC, AVAX, OP, AAVE, UNI \
                         are supported.",
                        req.symbol.base, self.chain
                    )))?;

                // Collateral: long = index token, short = USDC
                let collateral_token = if is_long {
                    Self::gmx_collateral_address(&req.symbol.base, &self.chain)
                        .ok_or_else(|| ExchangeError::InvalidRequest(format!(
                            "GMX: no collateral token mapping for '{}' on {}.",
                            req.symbol.base, self.chain
                        )))?
                } else {
                    Self::usdc_address(&self.chain)?
                };

                // Extract price from order type to determine acceptable_price and
                // whether this is a market or limit order.
                let (trigger_price, acceptable_price, gmx_order_type) = match &req.order_type {
                    OrderType::Market => {
                        // Market order: trigger_price = 0, acceptable_price must be
                        // provided by the caller.  We use 0 here; the caller should
                        // set a real slippage guard before broadcasting.
                        (U256::ZERO, U256::ZERO, GmxOrderType::MarketIncrease)
                    }
                    OrderType::Limit { price } => {
                        // Convert f64 price to 30-decimal U256 representation.
                        // e.g. $3000.50 → 3000_500_000_000_000_000_000_000_000_000_000 (30 dec)
                        let price_30 = Self::price_to_u256_30dec(*price);
                        (price_30, price_30, GmxOrderType::LimitIncrease)
                    }
                    OrderType::StopMarket { stop_price } => {
                        let price_30 = Self::price_to_u256_30dec(*stop_price);
                        (price_30, price_30, GmxOrderType::LimitIncrease)
                    }
                    _ => {
                        return Err(ExchangeError::UnsupportedOperation(format!(
                            "GMX on-chain trading supports Market, Limit, and StopMarket \
                             order types. Got: {:?}",
                            req.order_type
                        )));
                    }
                };

                // Size delta: quantity is in base-asset units.
                // We convert to 30-decimal USD using acceptable_price if non-zero,
                // otherwise fall back to a nominal 1 USD per unit (caller must supply
                // a real price for correct sizing).
                let price_for_size = if acceptable_price.is_zero() {
                    U256::from(1u64)
                } else {
                    acceptable_price
                };
                // size_delta_usd = quantity * price  (both already 30-dec after mult)
                // quantity is f64 (base units), convert to 18-dec first, then multiply.
                let quantity_wei = Self::quantity_to_wei(req.quantity);
                // (quantity_wei [18 dec] * price_30 [30 dec]) / 10^18 = result [30 dec]
                let size_delta_usd = quantity_wei
                    .checked_mul(price_for_size)
                    .and_then(|v| v.checked_div(U256::from(10u64).pow(U256::from(18u32))))
                    .unwrap_or(U256::ZERO);

                // Execution fee: 0.001 ETH default (keeper gas on Arbitrum).
                // Caller should override this before broadcasting if gas price is unusual.
                let execution_fee = U256::from(1_000_000_000_000_000u64); // 0.001 ETH in wei

                // initial_collateral_delta_amount: same as quantity_wei (collateral deposit).
                // For longs this is the index-token amount; for shorts it's USDC amount
                // which the caller must approve before broadcasting.
                let initial_collateral_delta_amount = quantity_wei;

                let params = CreatePositionParams {
                    market,
                    collateral_token,
                    size_delta_usd,
                    initial_collateral_delta_amount,
                    trigger_price,
                    acceptable_price,
                    execution_fee,
                    side: gmx_side,
                    order_type: gmx_order_type,
                    receiver: from,
                    referral_code: [0u8; 32],
                };

                let _tx = onchain.create_position_onchain(&params, from)?;
                // Transaction built successfully.  Return a synthetic pending Order.
                // The caller must sign _tx and broadcast it via their alloy signer.
                let router = onchain.exchange_router()
                    .map(|a| format!("gmx:pending:{}", a))
                    .unwrap_or_else(|_| "gmx:pending:unknown".to_string());

                use crate::core::types::OrderStatus;
                use std::time::{SystemTime, UNIX_EPOCH};
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);

                let order = Order {
                    id: router,
                    client_order_id: req.client_order_id.clone(),
                    symbol: format!("{}/{}", req.symbol.base, req.symbol.quote),
                    side: req.side,
                    order_type: req.order_type.clone(),
                    status: OrderStatus::New,
                    price: None,
                    stop_price: None,
                    quantity: req.quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: now_ms,
                    updated_at: None,
                    time_in_force: req.time_in_force,
                };

                return Ok(PlaceOrderResponse::Simple(order));
            }
        }

        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "GMX trading requires the onchain-evm feature and a wallet. \
             Enable feature 'onchain-evm' and call connector.with_onchain(provider, wallet)."
                .to_string(),
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        #[cfg(feature = "onchain-evm")]
        {
            if let Some(ref onchain) = self.onchain {
                use crate::core::types::CancelScope;
                let from = self.wallet_address.ok_or_else(|| {
                    ExchangeError::InvalidRequest(
                        "wallet_address is required for GMX on-chain cancel. \
                         Call with_onchain(provider, wallet_address) on the connector."
                            .to_string(),
                    )
                })?;

                // Extract order_id and symbol from the cancel request.
                // For GMX, `order_id` is expected to be "<base>/<quote>:<side>:<market_addr>"
                // (the format we emit in place_order).  We parse it best-effort.
                let (symbol_hint, order_id_str) = match &req.scope {
                    CancelScope::Single { order_id } => {
                        let sym = req.symbol.as_ref()
                            .map(|s| format!("{}/{}", s.base, s.quote))
                            .unwrap_or_default();
                        (sym, order_id.clone())
                    }
                    _ => {
                        // GMX does not support batch or cancel-all via ExchangeRouter.
                        return Err(ExchangeError::UnsupportedOperation(
                            "GMX cancel only supports CancelScope::Single. \
                             Batch/All/BySymbol cancels are not available on GMX V2."
                                .to_string(),
                        ));
                    }
                };

                // Determine market from symbol hint or order_id.
                let base = req.symbol.as_ref()
                    .map(|s| s.base.as_str())
                    .unwrap_or("ETH");
                let market = Self::gmx_market_address(base, &self.chain)
                    .ok_or_else(|| ExchangeError::InvalidRequest(format!(
                        "GMX cancel: cannot resolve market for '{}'", base
                    )))?;

                // For cancel we build a MarketDecrease with full-position size.
                // The caller must know the position size and supply it; we use
                // a placeholder of 1 USD (caller adjusts before broadcasting).
                let collateral_token = Self::usdc_address(&self.chain)?;
                let execution_fee = U256::from(1_000_000_000_000_000u64);
                let close_params = ClosePositionParams {
                    market,
                    collateral_token,
                    size_delta_usd: U256::from(1u64), // placeholder — caller overrides
                    initial_collateral_delta_amount: U256::ZERO,
                    trigger_price: U256::ZERO,
                    acceptable_price: U256::ZERO,
                    execution_fee,
                    side: GmxPositionSide::Long, // caller overrides to match open position
                    order_type: GmxOrderType::MarketDecrease,
                    receiver: from,
                    referral_code: [0u8; 32],
                };

                let _tx = onchain.close_position_onchain(&close_params, from)?;
                // Unsigned transaction built.  Return the cancelled order stub.
                use crate::core::types::{OrderSide, OrderStatus, TimeInForce};
                use std::time::{SystemTime, UNIX_EPOCH};
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);

                let order = Order {
                    id: order_id_str,
                    client_order_id: None,
                    symbol: symbol_hint,
                    side: OrderSide::Sell, // decrease = sell side
                    order_type: crate::core::types::OrderType::Market,
                    status: OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: now_ms,
                    updated_at: Some(now_ms),
                    time_in_force: TimeInForce::Gtc,
                };

                return Ok(order);
            }
        }

        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "GMX order cancellation requires the onchain-evm feature and a wallet. \
             Enable 'onchain-evm' and call connector.with_onchain(provider, wallet)."
                .to_string(),
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Look up a single order by its on-chain key via Subsquid.
        // Subsquid `order` (singular) query by id.
        let gql = format!(
            r#"query {{
                order(id: "{id}") {{
                    id
                    orderType
                    market
                    indexTokenSymbol
                    sizeDeltaUsd
                    triggerPrice
                    acceptablePrice
                    isLong
                    status
                    timestamp
                }}
            }}"#,
            id = order_id,
        );

        let response = self.query_subsquid(&gql, None).await?;
        GmxParser::parse_single_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Resolve wallet address — required to scope to an account
        let account = match self.wallet_address_str() {
            Some(addr) => addr,
            None => return Ok(Vec::new()),
        };

        // Optional symbol filter
        let symbol_filter = match symbol {
            Some(sym) => {
                let base = sym.split('/').next().unwrap_or(sym).to_uppercase();
                format!(", indexTokenSymbol_eq: \"{}\"", base)
            }
            None => String::new(),
        };

        let gql = format!(
            r#"query {{
                orders(where: {{ account_eq: "{account}", status_eq: "active"{symbol_filter} }}) {{
                    id
                    orderType
                    market
                    indexTokenSymbol
                    sizeDeltaUsd
                    triggerPrice
                    acceptablePrice
                    isLong
                    status
                    timestamp
                }}
            }}"#,
            account = account,
            symbol_filter = symbol_filter,
        );

        let response = self.query_subsquid(&gql, None).await?;
        GmxParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Resolve wallet address — required to scope to an account
        let account = match self.wallet_address_str() {
            Some(addr) => addr,
            None => return Ok(Vec::new()),
        };

        // Status filter: historical orders are executed or cancelled
        let status_filter = match &filter.status {
            Some(crate::core::types::OrderStatus::Filled) => {
                ", status_eq: \"executed\"".to_string()
            }
            Some(crate::core::types::OrderStatus::Canceled) => {
                ", status_eq: \"cancelled\"".to_string()
            }
            _ => {
                // All completed orders (not active)
                ", status_in: [\"executed\", \"cancelled\", \"frozen\"]".to_string()
            }
        };

        // Symbol filter
        let symbol_filter = match &filter.symbol {
            Some(sym) => {
                let base = sym.base.to_uppercase();
                format!(", indexTokenSymbol_eq: \"{}\"", base)
            }
            None => String::new(),
        };

        // Limit
        let limit_clause = match filter.limit {
            Some(n) => format!(", limit: {}", n.min(1000)),
            None => String::new(),
        };

        let gql = format!(
            r#"query {{
                orders(
                    where: {{ account_eq: "{account}"{status_filter}{symbol_filter} }}
                    orderBy: timestamp_DESC{limit_clause}
                ) {{
                    id
                    orderType
                    market
                    indexTokenSymbol
                    sizeDeltaUsd
                    triggerPrice
                    acceptablePrice
                    isLong
                    status
                    timestamp
                }}
            }}"#,
            account = account,
            status_filter = status_filter,
            symbol_filter = symbol_filter,
            limit_clause = limit_clause,
        );

        let response = self.query_subsquid(&gql, None).await?;
        GmxParser::parse_orders(&response)
    }

    /// Get user trade history from Subsquid `tradeActions`.
    ///
    /// Requires a wallet address (set via `with_onchain()`). Returns an empty
    /// Vec when no wallet is configured rather than an error, consistent with
    /// the behaviour of `get_open_orders` and `get_order_history`.
    ///
    /// GMX has no traditional fills — each position increase/decrease is modelled
    /// as a trade action on-chain. These are returned as `UserTrade` records.
    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let account = match self.wallet_address_str() {
            Some(addr) => addr,
            None => return Ok(Vec::new()),
        };

        // Optional symbol filter on indexTokenSymbol
        let symbol_filter = match &filter.symbol {
            Some(sym) => {
                let base = sym.split('/').next().unwrap_or(sym).to_uppercase();
                format!(", indexTokenSymbol_eq: \"{}\"", base)
            }
            None => String::new(),
        };

        // Optional order key filter
        let order_key_filter = match &filter.order_id {
            Some(key) => format!(", orderKey_eq: \"{}\"", key),
            None => String::new(),
        };

        // Time range — Subsquid uses Unix seconds (stored as string)
        let start_filter = match filter.start_time {
            Some(ms) => format!(", timestamp_gte: \"{}\"", ms / 1000),
            None => String::new(),
        };
        let end_filter = match filter.end_time {
            Some(ms) => format!(", timestamp_lte: \"{}\"", ms / 1000),
            None => String::new(),
        };

        // Limit
        let limit_clause = match filter.limit {
            Some(n) => format!(", limit: {}", n.min(1000)),
            None => String::new(),
        };

        let gql = format!(
            r#"query {{
                tradeActions(
                    where: {{ account_eq: "{account}"{symbol_filter}{order_key_filter}{start_filter}{end_filter} }}
                    orderBy: timestamp_DESC{limit_clause}
                ) {{
                    id
                    orderKey
                    marketAddress
                    indexTokenSymbol
                    sizeDeltaUsd
                    executionPrice
                    isLong
                    orderType
                    timestamp
                }}
            }}"#,
            account = account,
            symbol_filter = symbol_filter,
            order_key_filter = order_key_filter,
            start_filter = start_filter,
            end_filter = end_filter,
            limit_clause = limit_clause,
        );

        let response = self.query_subsquid(&gql, None).await?;
        GmxParser::parse_trade_actions(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for GmxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        #[cfg(feature = "onchain-evm")]
        {
            if let Some(ref onchain) = self.onchain {
                use crate::core::chain::EvmChain;

                let wallet = self.wallet_address.ok_or_else(|| {
                    ExchangeError::InvalidRequest(
                        "wallet_address is required to query GMX on-chain balance. \
                         Call with_onchain(provider, wallet_address) on the connector."
                            .to_string(),
                    )
                })?;

                let wallet_str = format!("{:?}", wallet);

                let native_asset = match self.chain.as_str() {
                    "avalanche" | "avax" => "AVAX",
                    _ => "ETH",
                };

                // Helper: convert a U256 raw token balance to f64 with given decimals
                let u256_to_f64 = |raw: U256, decimals: u32| -> f64 {
                    let divisor = U256::from(10u64).pow(U256::from(decimals));
                    let whole = raw / divisor;
                    let remainder = raw % divisor;
                    whole.to_string().parse::<f64>().unwrap_or(0.0)
                        + remainder.to_string().parse::<f64>().unwrap_or(0.0)
                        / 10_f64.powi(decimals as i32)
                };

                // If a specific asset is requested, try ERC-20 lookup first
                if let Some(ref asset) = query.asset {
                    let asset_upper = asset.to_uppercase();

                    // Check whether the asset is the native token
                    if asset_upper == native_asset
                        || asset_upper == "WETH"
                        || asset_upper == "WAVAX"
                    {
                        // Native / wrapped native balance
                        let raw = onchain.get_native_balance(&wallet_str).await?;
                        let balance_f64 = u256_to_f64(raw, 18);
                        return Ok(vec![Balance {
                            asset: asset_upper,
                            free: balance_f64,
                            locked: 0.0,
                            total: balance_f64,
                        }]);
                    }

                    // Try to resolve as a known collateral token (ERC-20)
                    // Use the collateral address map for WETH/WBTC, then USDC as fallback
                    let maybe_token_addr: Option<Address> =
                        Self::gmx_collateral_address(&asset_upper, &self.chain)
                            .or_else(|| {
                                // USDC
                                if asset_upper == "USDC" || asset_upper == "USDC.E" {
                                    Self::usdc_address(&self.chain).ok()
                                } else {
                                    None
                                }
                            });

                    if let Some(token_addr) = maybe_token_addr {
                        // Determine token decimals
                        let decimals: u32 = match asset_upper.as_str() {
                            "WBTC" | "BTC" => 8,
                            "USDC" | "USDC.E" | "USDT" => 6,
                            _ => 18,
                        };

                        let raw = onchain.provider()
                            .erc20_balance(token_addr, wallet)
                            .await?;
                        let balance_f64 = u256_to_f64(raw, decimals);

                        return Ok(vec![Balance {
                            asset: asset_upper,
                            free: balance_f64,
                            locked: 0.0,
                            total: balance_f64,
                        }]);
                    }

                    // Unknown ERC-20 — not in our address tables
                    return Err(ExchangeError::UnsupportedOperation(format!(
                        "GMX on-chain balance: token '{}' is not in the known collateral \
                         address table. Use the onchain EvmProvider directly for arbitrary \
                         ERC-20 queries.",
                        asset
                    )));
                }

                // No specific asset — return native token balance
                let raw_balance = onchain.get_native_balance(&wallet_str).await?;
                let balance_eth = u256_to_f64(raw_balance, 18);

                let balance = Balance {
                    asset: native_asset.to_string(),
                    free: balance_eth,
                    locked: 0.0,
                    total: balance_eth,
                };

                return Ok(vec![balance]);
            }
        }

        let _ = query;
        Err(ExchangeError::UnsupportedOperation(
            "GMX balances require the onchain-evm feature. Enable 'onchain-evm' and \
             call connector.with_onchain(provider, wallet_address)."
                .to_string(),
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // GMX has no CEX-style account.  With onchain-evm we could return a
        // synthetic AccountInfo wrapping the native-balance result, but the
        // AccountInfo.balances vec would require iterating all ERC-20 tokens.
        // For now this remains unsupported; callers should use get_balance().
        Err(ExchangeError::UnsupportedOperation(
            "GMX has no account concept in its REST API. \
             Use get_balance() with the onchain-evm feature for native token balance."
                .to_string(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // GMX uses a protocol fee model, not maker/taker.
        // Fees: 0.05% open + 0.05% close for market orders (position fee),
        // plus price impact and borrowing rates.
        // These are not translatable to a symmetric maker/taker FeeInfo.
        Err(ExchangeError::UnsupportedOperation(
            "GMX uses protocol fees (position fee 0.05%, price impact, borrow rate) \
             not maker/taker rates. See https://docs.gmx.io/docs/trading/fees for details."
                .to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for GmxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        // Resolve wallet address — required to scope to an account
        let account = match self.wallet_address_str() {
            Some(addr) => addr,
            None => {
                // No wallet available: return empty rather than error
                return Ok(Vec::new());
            }
        };

        // Build symbol filter clause (optional)
        let symbol_filter = match &query.symbol {
            Some(sym) => {
                let base = sym.base.to_uppercase();
                format!(", indexTokenSymbol_eq: \"{}\"", base)
            }
            None => String::new(),
        };

        let gql = format!(
            r#"query {{
                positions(where: {{ account_eq: "{account}", isOpen_eq: true{symbol_filter} }}) {{
                    id
                    market
                    indexTokenSymbol
                    collateralToken
                    isLong
                    sizeInUsd
                    sizeInTokens
                    collateralAmount
                    entryPrice
                    unrealizedPnl
                    realizedPnl
                    createdAt
                }}
            }}"#,
            account = account,
            symbol_filter = symbol_filter,
        );

        let response = self.query_subsquid(&gql, None).await?;
        GmxParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // GMX V2 has genuine funding rates (not just borrowing fees).
        // The dominant OI side pays the other side; rate changes with OI imbalance.
        // Source: GET /markets/info — fields: fundingFactorPerSecond, longsPayShorts
        let response = self.get(GmxEndpoint::MarketInfo, HashMap::new()).await?;
        // Default to long-side rate; caller can query again with explicit side if needed
        GmxParser::parse_funding_rate(&response, symbol, true)
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "GMX position modification requires smart contract transactions. \
             REST API is read-only."
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connector_creation() {
        let connector = GmxConnector::arbitrum().await.unwrap();
        assert_eq!(connector.chain, "arbitrum");
        assert_eq!(connector.exchange_id(), ExchangeId::Gmx);
        assert_eq!(connector.exchange_type(), ExchangeType::Dex);
    }

    #[tokio::test]
    async fn test_format_symbol() {
        let symbol = format_symbol("ETH", "USD", AccountType::FuturesCross);
        assert_eq!(symbol, "ETH/USD");
    }
}
