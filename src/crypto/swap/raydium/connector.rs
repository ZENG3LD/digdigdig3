//! # Raydium Connector Implementation
//!
//! Market data connector for Raydium DEX on Solana.
//!
//! ## Implementation Status
//!
//! - [x] ExchangeIdentity
//! - [x] MarketData (basic public data)
//! - [ ] Trading (not implemented - requires Solana wallet integration)
//! - [ ] Account (not implemented - DEX architecture)
//! - [ ] WebSocket (not available - use gRPC instead)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::{
    HttpClient, ExchangeResult, ExchangeError,
    ExchangeId, ExchangeType, AccountType, Symbol,
    Price, Ticker, OrderBook, Kline,
    ExchangeIdentity, MarketData,
};
use crate::core::traits::{Trading, Account};
use crate::core::types::{
    ConnectorStats,
    OrderRequest, CancelRequest, Order, OrderHistoryFilter, PlaceOrderResponse,
    BalanceQuery, Balance, AccountInfo, FeeInfo,
};
use crate::core::utils::SimpleRateLimiter;

#[cfg(feature = "onchain-solana")]
use solana_sdk::signature::{Keypair, Signer};
#[cfg(feature = "onchain-solana")]
use crate::core::chain::SolanaProvider;

use super::{RaydiumUrls, RaydiumAuth, RaydiumParser, RaydiumEndpoint};

/// Raydium DEX connector
pub struct RaydiumConnector {
    /// HTTP client
    http: HttpClient,
    /// API URLs
    urls: RaydiumUrls,
    /// Authentication handler (no-op for Raydium)
    _auth: RaydiumAuth,
    /// Is devnet
    is_testnet: bool,
    /// Rate limiter (10 req/s conservative)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Optional Solana chain provider for on-chain transaction submission.
    ///
    /// When present, [`submit_swap`] can deserialize, sign, and broadcast
    /// swap transactions returned by the Raydium swap API.
    #[cfg(feature = "onchain-solana")]
    solana_provider: Option<Arc<SolanaProvider>>,
}

impl RaydiumConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `is_testnet` - Use devnet (true) or mainnet (false)
    ///
    /// Note: Credentials not needed - Raydium APIs are public
    pub async fn new(is_testnet: bool) -> ExchangeResult<Self> {
        let urls = if is_testnet {
            RaydiumUrls::DEVNET
        } else {
            RaydiumUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = RaydiumAuth::new();

        // Raydium rate limit: 12 req/60s (conservative, ~1 req/5s)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(12, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            urls,
            _auth: auth,
            is_testnet,
            rate_limiter,
            #[cfg(feature = "onchain-solana")]
            solana_provider: None,
        })
    }

    /// Wait for rate limit if necessary
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

    /// Make GET request to endpoint
    async fn get_request(
        &self,
        endpoint: RaydiumEndpoint,
        params: &HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        self.rate_limit_wait().await;

        let url = endpoint.url(&self.urls);

        self.http.get(&url, params).await
    }

    /// Make POST request to endpoint
    async fn post_request(
        &self,
        endpoint: RaydiumEndpoint,
        body: &serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        self.rate_limit_wait().await;

        let url = endpoint.url(&self.urls);
        let headers = HashMap::new();

        self.http.post(&url, body, &headers).await
    }

    /// Get pool data by mint pair
    async fn get_pool_by_mints(
        &self,
        mint_a: &str,
        mint_b: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("mint1".to_string(), mint_a.to_string());
        params.insert("mint2".to_string(), mint_b.to_string());

        self.get_request(RaydiumEndpoint::PoolByMint, &params).await
    }

    /// Get current Solana cluster time
    ///
    /// Returns the on-chain Unix timestamp reported by the Solana cluster.
    /// Corresponds to `GET /main/chain-time`.
    pub async fn get_chain_time(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::ChainTime, &params).await
    }

    /// Get Raydium platform summary
    ///
    /// Returns aggregate platform stats: TVL, 24h trading volume, and fee revenue.
    /// Corresponds to `GET /main/info`.
    pub async fn get_platform_info(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::PlatformInfo, &params).await
    }

    /// Get pool token price history (OHLCV line data)
    ///
    /// Returns historical price data for a specific pool.
    /// `pool_id` is the Solana public key (Base58) of the pool.
    /// `resolution` can be `"15m"`, `"1h"`, `"4h"`, `"1d"`, etc.
    /// Corresponds to `GET /pools/line/price`.
    pub async fn get_pool_price_history(
        &self,
        pool_id: &str,
        resolution: &str,
        time_before: Option<i64>,
        time_after: Option<i64>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), pool_id.to_string());
        params.insert("type".to_string(), resolution.to_string());
        if let Some(before) = time_before {
            params.insert("before".to_string(), before.to_string());
        }
        if let Some(after) = time_after {
            params.insert("after".to_string(), after.to_string());
        }
        self.get_request(RaydiumEndpoint::PoolPriceHistory, &params).await
    }

    /// Get pool liquidity history over time
    ///
    /// Returns historical liquidity (TVL) data for a specific pool.
    /// `pool_id` is the Solana public key (Base58) of the pool.
    /// `resolution` can be `"15m"`, `"1h"`, `"4h"`, `"1d"`, etc.
    /// Corresponds to `GET /pools/line/liquidity`.
    pub async fn get_pool_liquidity_history(
        &self,
        pool_id: &str,
        resolution: &str,
        time_before: Option<i64>,
        time_after: Option<i64>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), pool_id.to_string());
        params.insert("type".to_string(), resolution.to_string());
        if let Some(before) = time_before {
            params.insert("before".to_string(), before.to_string());
        }
        if let Some(after) = time_after {
            params.insert("after".to_string(), after.to_string());
        }
        self.get_request(RaydiumEndpoint::PoolLiquidityHistory, &params).await
    }

    /// Get aggregate pool statistics (TVL and volume across all pools)
    ///
    /// Corresponds to `GET /pools/info/stats`.
    pub async fn get_pool_stats(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::PoolStats, &params).await
    }

    /// Get CLMM (concentrated liquidity) pool configuration tiers
    ///
    /// Returns available fee tiers and tick-spacing configurations for CLMM pools.
    /// Corresponds to `GET /clmm/configs`.
    pub async fn get_clmm_configs(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::ClmmConfigs, &params).await
    }

    /// Get CPMM (constant product) pool configuration tiers
    ///
    /// Returns available fee tiers for CPMM pools.
    /// Corresponds to `GET /cpmm/configs`.
    pub async fn get_cpmm_configs(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::CpmmConfigs, &params).await
    }

    /// Get farms owned or staked by a wallet address
    ///
    /// `wallet` is the Base58-encoded Solana wallet public key.
    /// Corresponds to `GET /farms/info/mine`.
    pub async fn get_farm_ownership(&self, wallet: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("owner".to_string(), wallet.to_string());
        self.get_request(RaydiumEndpoint::FarmOwnership, &params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SWAP API
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get swap quote for a token pair
    ///
    /// Returns routing and pricing data for a proposed swap.
    /// Use the returned quote response with [`build_swap_transaction`] to
    /// obtain a serialized transaction ready for signing.
    ///
    /// # Arguments
    /// * `input_mint`   - Source token mint address (Base58)
    /// * `output_mint`  - Destination token mint address (Base58)
    /// * `amount`       - Quantity in smallest units (lamports / token atomic units)
    /// * `slippage_bps` - Maximum accepted slippage in basis points (50 = 0.5%)
    /// * `base_in`      - `true` to fix the input amount (`SwapQuoteBaseIn`),
    ///                    `false` to fix the output amount (`SwapQuoteBaseOut`)
    ///
    /// Corresponds to `GET /compute/swap-base-in` or `GET /compute/swap-base-out`.
    pub async fn get_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u64,
        base_in: bool,
    ) -> ExchangeResult<serde_json::Value> {
        let endpoint = if base_in {
            RaydiumEndpoint::SwapQuoteBaseIn
        } else {
            RaydiumEndpoint::SwapQuoteBaseOut
        };

        let mut params = HashMap::new();
        params.insert("inputMint".to_string(), input_mint.to_string());
        params.insert("outputMint".to_string(), output_mint.to_string());
        params.insert("amount".to_string(), amount.to_string());
        params.insert("slippageBps".to_string(), slippage_bps.to_string());
        // txVersion is required by the Trade API
        params.insert("txVersion".to_string(), "V0".to_string());

        self.get_request(endpoint, &params).await
    }

    /// Build a serialized swap transaction from a quote
    ///
    /// POSTs the quote response (obtained from [`get_swap_quote`]) together
    /// with the user's wallet public key to the Trade API. Returns a
    /// base64-encoded, unsigned Solana transaction that the user must sign
    /// and submit to the network.
    ///
    /// # Arguments
    /// * `quote_response` - Full JSON object returned by [`get_swap_quote`]
    /// * `wallet_pubkey`  - User's Solana wallet public key (Base58)
    ///
    /// Corresponds to `POST /transaction/swap-base-in` or
    /// `POST /transaction/swap-base-out`.  The variant is inferred from
    /// the `swapType` field inside `quote_response`; if the field is absent
    /// the base-in endpoint is used as default.
    pub async fn build_swap_transaction(
        &self,
        quote_response: &serde_json::Value,
        wallet_pubkey: &str,
    ) -> ExchangeResult<String> {
        // Determine endpoint from quote type (base-in vs base-out).
        // The compute endpoints embed a `swapType` field in their response.
        let is_base_in = quote_response
            .get("swapType")
            .and_then(|v| v.as_str())
            .map(|s| s != "BaseOut")
            .unwrap_or(true);

        let endpoint = if is_base_in {
            RaydiumEndpoint::SwapTransactionBaseIn
        } else {
            RaydiumEndpoint::SwapTransactionBaseOut
        };

        let body = serde_json::json!({
            "swapResponse": quote_response,
            "wallet": wallet_pubkey,
            "txVersion": "V0",
            "wrapSol": true,
            "unwrapSol": true,
        });

        let response = self.post_request(endpoint, &body).await?;

        // The Trade API wraps the serialized transaction(s) under `data`.
        // It may return a single transaction or an array; we return the
        // first (and typically only) base64 string.
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' in swap transaction response".to_string()))?;

        // data may be an array of transaction objects or a single object
        let tx_obj = if let Some(arr) = data.as_array() {
            arr.first()
                .ok_or_else(|| ExchangeError::Parse("Empty transaction array in swap response".to_string()))?
        } else {
            data
        };

        // Each transaction object has a `transaction` key with the base64 string
        tx_obj
            .get("transaction")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse(
                "Missing 'transaction' field in swap transaction response".to_string()
            ))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MINT / TOKEN ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get Raydium's default token list
    ///
    /// Returns token metadata (mint address, symbol, decimals, logo) for all
    /// tokens recognized by Raydium. Mainnet only.
    /// Corresponds to `GET /mint/list`.
    pub async fn get_token_list(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::MintList, &params).await
    }

    /// Get detailed token info for specific mint addresses
    ///
    /// `mints` is a slice of Base58 Solana token mint addresses.
    /// Multiple mints are sent as a single comma-separated `mints` query param.
    /// Corresponds to `GET /mint/ids`.
    pub async fn get_token_info(&self, mints: &[&str]) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("mints".to_string(), mints.join(","));
        self.get_request(RaydiumEndpoint::MintIds, &params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POOL ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get paginated pool list
    ///
    /// Returns all Raydium liquidity pools with TVL, volume and APR data.
    /// Optional params accepted by the API (type, sort, order, page, pageSize)
    /// can be passed via `extra_params`.
    /// Corresponds to `GET /pools/info/list`.
    pub async fn get_pool_list(
        &self,
        extra_params: Option<HashMap<String, String>>,
    ) -> ExchangeResult<serde_json::Value> {
        let params = extra_params.unwrap_or_default();
        self.get_request(RaydiumEndpoint::PoolList, &params).await
    }

    /// Get specific pools by their IDs
    ///
    /// `ids` is a slice of Base58 Solana pool public keys.
    /// Multiple IDs are sent as a single comma-separated `ids` query param.
    /// Corresponds to `GET /pools/info/ids`.
    pub async fn get_pool_by_id(&self, ids: &[&str]) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("ids".to_string(), ids.join(","));
        self.get_request(RaydiumEndpoint::PoolIds, &params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FARM ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get paginated farm list
    ///
    /// Returns all active Raydium farms with APY and staking info.
    /// Corresponds to `GET /farms/info/list`.
    pub async fn get_farm_list(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::FarmList, &params).await
    }

    /// Get specific farms by their IDs
    ///
    /// `ids` is a slice of Base58 farm public keys.
    /// Multiple IDs are sent as a single comma-separated `ids` query param.
    /// Corresponds to `GET /farms/info/ids`.
    pub async fn get_farm_by_id(&self, ids: &[&str]) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("ids".to_string(), ids.join(","));
        self.get_request(RaydiumEndpoint::FarmIds, &params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MAIN / INFRASTRUCTURE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get recommended Solana RPC endpoints for Raydium
    ///
    /// Returns a list of Solana RPC URLs recommended by the Raydium UI.
    /// Useful for selecting low-latency RPCs for on-chain operations.
    /// Corresponds to `GET /main/rpcs`.
    pub async fn get_recommended_rpcs(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::Rpcs, &params).await
    }

    /// Get auto priority fee recommendations
    ///
    /// Returns microLamport fee tiers (very-high, high, medium) for Solana
    /// transaction priority.  Use these values in `computeUnitPriceMicroLamports`
    /// when building swap transactions.
    /// Corresponds to `GET /main/auto-fee`.
    pub async fn get_priority_fees(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get_request(RaydiumEndpoint::AutoFee, &params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ON-CHAIN INTEGRATION (onchain-solana feature)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Attach a Solana chain provider to enable on-chain swap execution.
    ///
    /// Once a provider is set, [`submit_swap`] can deserialize, sign, and
    /// broadcast transactions returned by the Raydium swap API.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use digdigdig3::core::chain::SolanaProvider;
    ///
    /// let provider = Arc::new(SolanaProvider::mainnet());
    /// let connector = RaydiumConnector::new(false).await?
    ///     .with_solana_provider(provider);
    /// ```
    #[cfg(feature = "onchain-solana")]
    pub fn with_solana_provider(mut self, provider: Arc<SolanaProvider>) -> Self {
        self.solana_provider = Some(provider);
        self
    }

    /// Deserialize, sign, and submit a Raydium swap transaction to the Solana network.
    ///
    /// The Raydium swap API returns a base64-encoded, unsigned Solana transaction.
    /// This method:
    ///
    /// 1. Decodes the base64 transaction bytes.
    /// 2. Deserializes the bincode-encoded [`Transaction`].
    /// 3. Signs it with the provided `keypair` (the user's wallet).
    /// 4. Submits it via the attached [`SolanaProvider`].
    ///
    /// Returns the base58-encoded transaction signature on success.
    ///
    /// # Errors
    ///
    /// - `ExchangeError::UnsupportedOperation` if no `SolanaProvider` is attached.
    /// - `ExchangeError::InvalidRequest` if base64 or bincode decoding fails.
    /// - `ExchangeError::Network` if RPC submission fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Obtain an unsigned swap transaction from the Raydium Swap API
    /// // (e.g. POST https://transaction-v1.raydium.io/transaction/swap)
    /// let tx_b64 = "..."; // base64-encoded unsigned tx from Raydium API
    ///
    /// let sig = connector.submit_swap(tx_b64, &keypair).await?;
    /// println!("Swap signature: {}", sig);
    /// ```
    #[cfg(feature = "onchain-solana")]
    pub async fn submit_swap(
        &self,
        transaction_b64: &str,
        keypair: &Keypair,
    ) -> ExchangeResult<String> {
        use base64::Engine as _;
        use crate::core::chain::SolanaChain;

        let provider = self.solana_provider.as_ref().ok_or_else(|| {
            ExchangeError::UnsupportedOperation(
                "No SolanaProvider attached. Call with_solana_provider() first.".to_string(),
            )
        })?;

        // Step 1: Decode from base64
        let tx_bytes = base64::engine::general_purpose::STANDARD
            .decode(transaction_b64)
            .map_err(|e| {
                ExchangeError::InvalidRequest(format!(
                    "Failed to decode swap transaction from base64: {}",
                    e
                ))
            })?;

        // Step 2: Deserialize from bincode
        let mut tx: solana_sdk::transaction::Transaction =
            bincode::deserialize(&tx_bytes).map_err(|e| {
                ExchangeError::InvalidRequest(format!(
                    "Failed to deserialize swap transaction (bincode): {}",
                    e
                ))
            })?;

        // Step 3: Get a fresh blockhash and sign
        let blockhash = provider.get_latest_blockhash().await?;
        tx.sign(&[keypair], blockhash);

        // Step 4: Submit via SolanaProvider
        let sig = provider.send_transaction(&tx).await?;

        Ok(sig.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for RaydiumConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Raydium
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_count(), lim.max_requests())
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

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }

    fn is_testnet(&self) -> bool {
        self.is_testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for RaydiumConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        // For Raydium, symbol.base and symbol.quote are mint addresses
        let mut params = HashMap::new();
        params.insert("mints".to_string(), symbol.base.clone());

        let response = self.get_request(RaydiumEndpoint::MintPrice, &params).await?;
        let price_value = RaydiumParser::parse_price(&response, &symbol.base)?;

        Ok(price_value)
    }

    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let response = self.get_pool_by_mints(&symbol.base, &symbol.quote).await?;
        let data = RaydiumParser::extract_data(&response)?;

        // Get first pool from array
        let pool = data.as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No pools found for pair".to_string()))?;

        let mut ticker = RaydiumParser::parse_ticker(pool)?;
        ticker.symbol = symbol.to_string();

        Ok(ticker)
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _limit: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Raydium is a pure AMM (Automated Market Maker) using constant product formula (x*y=k).
        // AMM pools do not have traditional orderbooks with discrete bid/ask levels.
        //
        // Historical Note: Raydium v4 originally integrated with Serum/OpenBook orderbooks,
        // but this integration has been deprecated. Modern Raydium pools are pure AMMs.
        //
        // Alternative: To simulate market depth, query pool reserves via Solana RPC
        // and calculate prices at different swap amounts using the AMM formula.
        Err(ExchangeError::UnsupportedOperation(
            "Orderbooks not supported - Raydium is a pure AMM. Use get_price() or query pool reserves.".to_string()
        ))
    }

    async fn get_klines(
        &self,
        _symbol: Symbol,
        _interval: &str,
        _limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::NotSupported(
            "Raydium API does not provide kline data".to_string()
        ))
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Ping by fetching version endpoint
        let params = HashMap::new();
        let _response = self.get_request(RaydiumEndpoint::Version, &params).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════

// Raydium swap execution requires:
// 1. Solana wallet integration (solana-sdk)
// 2. Wallet keypair for transaction signing
// 3. Swap API provides unsigned transaction; must sign locally
// 4. Submit signed transaction to Solana RPC (sendTransaction)
// 5. Confirm via Solana transaction signature
//
// REST API provides swap routing and unsigned tx data only.

#[async_trait]
impl Trading for RaydiumConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "Raydium swap execution requires Solana wallet integration (solana-sdk). \
             Use Swap API to get transaction data, then sign and broadcast via Solana RPC."
                .to_string(),
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "Raydium AMM swaps are atomic Solana transactions — they cannot be cancelled. \
             Transactions either confirm or fail."
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
            "Raydium has no order tracking. \
             Use Solana transaction signature to check swap status via getTransaction RPC call."
                .to_string(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Raydium AMM swaps are atomic — there are no open/pending orders. \
             Pure AMM model does not support limit orders."
                .to_string(),
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Raydium does not provide order history via REST API. \
             Query transaction history via Solana RPC (getSignaturesForAddress)."
                .to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for RaydiumConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Raydium has no account system. \
             Query SPL token balances via Solana RPC (getTokenAccountsByOwner)."
                .to_string(),
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Raydium is a permissionless Solana AMM — there is no account concept. \
             Use Solana wallet address to query on-chain account data."
                .to_string(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Raydium uses protocol fee tiers per pool (not maker/taker):
        // - CPMM pools: typically 0.25% (configurable)
        // - CLMM pools: 0.01%, 0.05%, 0.30%, or 1.00%
        // - Legacy AMM v4: 0.25% (0.22% to LPs, 0.03% protocol)
        // Not translatable to FeeInfo maker/taker structure.
        Err(ExchangeError::UnsupportedOperation(
            "Raydium uses pool fee tiers (0.01%–1.00%) paid to LPs, not maker/taker rates. \
             Fee is per pool — query pool data via /pools/info endpoint."
                .to_string(),
        ))
    }
}
