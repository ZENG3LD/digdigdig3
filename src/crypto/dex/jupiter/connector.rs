//! # Jupiter Connector
//!
//! Implementation of core traits for Jupiter DEX aggregator.
//!
//! ## Core traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data (price, ticker, orderbook simulated)
//!
//! ## Notes
//! - Jupiter uses Solana mint addresses, not traditional symbols
//! - Only public market data is implemented (no trading/account)
//! - Orderbook is simulated from quote data (no native orderbook)
//! - Klines not supported (no historical data API)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
};
use crate::core::traits::{ExchangeIdentity, MarketData, Trading, Account};
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

use super::endpoints::{self, JupiterUrls, JupiterEndpoint, MintRegistry};
use super::auth::JupiterAuth;
use super::parser::JupiterParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Jupiter DEX connector
pub struct JupiterConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (required for all endpoints since Oct 2025)
    auth: JupiterAuth,
    /// URLs (mainnet only for Jupiter)
    urls: JupiterUrls,
    /// Rate limiter (1 req/s free tier)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Optional Solana chain provider for on-chain transaction submission.
    ///
    /// When present, [`submit_swap`] can deserialize, sign, and broadcast
    /// swap transactions returned by the Jupiter REST API.
    #[cfg(feature = "onchain-solana")]
    solana_provider: Option<Arc<SolanaProvider>>,
}

impl JupiterConnector {
    /// Create new Jupiter connector
    ///
    /// # Arguments
    /// * `api_key` - API key for Jupiter API (required for all endpoints since Oct 2025)
    ///
    /// # Notes
    /// - All endpoints now require API key (changed in Jupiter API v1 Oct 2025)
    /// - Use `from_env()` to load API key from `JUPITER_API_KEY` environment variable
    pub async fn new(api_key: String) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = JupiterAuth::new(api_key);
        let urls = JupiterUrls::MAINNET;

        // Jupiter rate limit: 60 req/60s (free tier)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(60, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            rate_limiter,
            #[cfg(feature = "onchain-solana")]
            solana_provider: None,
        })
    }

    /// Create connector from environment variable
    ///
    /// Reads API key from `JUPITER_API_KEY` environment variable.
    pub async fn from_env() -> ExchangeResult<Self> {
        let api_key = std::env::var("JUPITER_API_KEY").map_err(|_| {
            ExchangeError::Auth(
                "JUPITER_API_KEY environment variable not set".to_string(),
            )
        })?;
        Self::new(api_key).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

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

    /// GET request
    async fn get(
        &self,
        endpoint: JupiterEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let url = endpoint.url(&self.urls);

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let full_url = format!("{}{}", url, query);

        // All endpoints require API key (since Oct 2025)
        let headers = self.auth.auth_headers();

        let response = self.http.get_with_headers(&full_url, &HashMap::new(), &headers).await?;
        JupiterParser::check_error(&response)?;
        Ok(response)
    }

    /// Convert Symbol to mint addresses
    ///
    /// Attempts to resolve symbols to Solana mint addresses.
    /// If symbol is already a mint address, returns as-is.
    fn symbol_to_mints(&self, symbol: &Symbol) -> ExchangeResult<(String, String)> {
        // Try to resolve base and quote symbols to mint addresses
        let base_mint = if endpoints::is_valid_mint_address(&symbol.base) {
            symbol.base.clone()
        } else {
            MintRegistry::symbol_to_mint(&symbol.base)
                .ok_or_else(|| {
                    ExchangeError::InvalidRequest(format!(
                        "Unknown token symbol: {}. Use mint address instead.",
                        symbol.base
                    ))
                })?
                .to_string()
        };

        let quote_mint = if endpoints::is_valid_mint_address(&symbol.quote) {
            symbol.quote.clone()
        } else {
            MintRegistry::symbol_to_mint(&symbol.quote)
                .ok_or_else(|| {
                    ExchangeError::InvalidRequest(format!(
                        "Unknown token symbol: {}. Use mint address instead.",
                        symbol.quote
                    ))
                })?
                .to_string()
        };

        Ok((base_mint, quote_mint))
    }

    /// Get quote for a swap
    async fn _get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("inputMint".to_string(), input_mint.to_string());
        params.insert("outputMint".to_string(), output_mint.to_string());
        params.insert("amount".to_string(), amount.to_string());
        params.insert("slippageBps".to_string(), slippage_bps.to_string());

        self.get(JupiterEndpoint::Quote, params).await
    }

    /// POST request (for Ultra Swap API and other POST endpoints)
    async fn post(
        &self,
        endpoint: JupiterEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let url = endpoint.url(&self.urls);
        let headers = self.auth.auth_headers();

        let response = self.http.post(&url, &body, &headers).await?;
        JupiterParser::check_error(&response)?;
        Ok(response)
    }

    /// Get full token list with metadata (TokensV2)
    ///
    /// Returns all tokens indexed by Jupiter, including tags, extensions, and
    /// logo URIs. Corresponds to `GET /tokens/v2`.
    pub async fn get_tokens_v2(
        &self,
        tags: Option<&[&str]>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(tag_list) = tags {
            params.insert("tags".to_string(), tag_list.join(","));
        }
        self.get(JupiterEndpoint::TokensV2, params).await
    }

    /// Create a new Ultra Swap order
    ///
    /// The Ultra Swap API provides an improved routing experience with guaranteed
    /// execution. Returns an order object including the transaction to sign and a
    /// `requestId` that must be forwarded to `execute_ultra_swap`.
    ///
    /// Uses `GET /ultra/v1/order` with query parameters (not a POST body).
    ///
    /// `input_mint` and `output_mint` are Solana mint addresses.
    /// `amount` is the raw input amount (in lamports / smallest unit).
    /// `taker` is the user's Solana wallet public key.
    pub async fn create_ultra_swap_order(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        taker: &str,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("inputMint".to_string(), input_mint.to_string());
        params.insert("outputMint".to_string(), output_mint.to_string());
        params.insert("amount".to_string(), amount.to_string());
        params.insert("taker".to_string(), taker.to_string());
        self.get(JupiterEndpoint::UltraSwapOrder, params).await
    }

    /// Get the status of an Ultra Swap by transaction ID
    ///
    /// Poll this endpoint after submitting an Ultra Swap transaction to check
    /// whether it has been confirmed on-chain.
    pub async fn get_ultra_swap_status(&self, transaction_id: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("transactionId".to_string(), transaction_id.to_string());
        self.get(JupiterEndpoint::UltraSwapStatus, params).await
    }

    /// Create an unsigned Ultra Swap transaction
    ///
    /// Returns a serialised unsigned transaction that the caller must sign
    /// before submitting via `execute_ultra_swap`.
    pub async fn create_ultra_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_public_key: &str,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "inputMint": input_mint,
            "outputMint": output_mint,
            "amount": amount.to_string(),
            "slippageBps": slippage_bps,
            "userPublicKey": user_public_key,
        });
        self.post(JupiterEndpoint::UltraSwapCreate, body).await
    }

    /// Execute (broadcast) a signed Ultra Swap transaction
    ///
    /// `signed_transaction` is the Base64-encoded signed Solana transaction.
    /// `request_id` is the UUID returned by `create_ultra_swap_order` — it is
    /// required by the `/execute` endpoint to correlate the order.
    /// Returns the transaction signature and confirmation status.
    pub async fn execute_ultra_swap(
        &self,
        signed_transaction: &str,
        request_id: &str,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "signedTransaction": signed_transaction,
            "requestId": request_id,
        });
        self.post(JupiterEndpoint::UltraSwapExecute, body).await
    }

    /// Get token holdings for a wallet address
    ///
    /// Returns native SOL balance (top-level) and all SPL token balances (under
    /// `tokens`) for the given Solana wallet public key. Uses the current
    /// `GET /ultra/v1/holdings/{address}` endpoint (replaces the deprecated
    /// `/ultra/v1/balances` endpoint).
    pub async fn get_ultra_holdings(&self, wallet: &str) -> ExchangeResult<Value> {
        let url = format!("{}/{}", JupiterEndpoint::UltraHoldings.url(&self.urls), wallet);
        self.rate_limit_wait().await;
        let headers = self.auth.auth_headers();
        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        JupiterParser::check_error(&response)?;
        Ok(response)
    }

    /// Get native SOL balance only for a wallet address
    ///
    /// Corresponds to `GET /ultra/v1/holdings/{address}/native`.
    /// Faster than `get_ultra_holdings` for wallets with many token accounts.
    pub async fn get_ultra_holdings_native(&self, wallet: &str) -> ExchangeResult<Value> {
        let url = format!("{}/{}/native", JupiterEndpoint::UltraHoldingsNative.url(&self.urls), wallet);
        self.rate_limit_wait().await;
        let headers = self.auth.auth_headers();
        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        JupiterParser::check_error(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRIGGER API (limit orders)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Create a trigger (limit) order
    ///
    /// A trigger order executes when the market price reaches the implied ratio
    /// `taking_amount / making_amount`.
    ///
    /// Returns an unsigned Solana transaction (`transaction`), the `order`
    /// account address, and a `requestId` needed for `execute_trigger_order`.
    ///
    /// All amount strings are in the token's smallest unit (pre-decimals).
    pub async fn create_trigger_order(
        &self,
        input_mint: &str,
        output_mint: &str,
        making_amount: u64,
        taking_amount: u64,
        maker: &str,
        expiry: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut body = serde_json::json!({
            "inputMint": input_mint,
            "outputMint": output_mint,
            "maker": maker,
            "payer": maker,
            "makingAmount": making_amount.to_string(),
            "takingAmount": taking_amount.to_string(),
        });

        if let Some(ts) = expiry {
            body["expiredAt"] = serde_json::json!(ts);
        }

        self.post(JupiterEndpoint::TriggerCreateOrder, body).await
    }

    /// Execute a signed trigger order transaction
    ///
    /// `signed_transaction` is the Base64-encoded signed Solana transaction from
    /// `create_trigger_order`. `request_id` is the UUID from that same response.
    pub async fn execute_trigger_order(
        &self,
        signed_transaction: &str,
        request_id: &str,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "signedTransaction": signed_transaction,
            "requestId": request_id,
        });
        self.post(JupiterEndpoint::TriggerExecute, body).await
    }

    /// Cancel a single trigger order
    ///
    /// Returns an unsigned Solana transaction and `requestId`.
    /// The caller must sign and submit via `execute_trigger_order`.
    pub async fn cancel_trigger_order(
        &self,
        order_id: &str,
        maker: &str,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "maker": maker,
            "order": order_id,
            "computeUnitPrice": "auto",
        });
        self.post(JupiterEndpoint::TriggerCancelOrder, body).await
    }

    /// Cancel multiple trigger orders in bulk
    ///
    /// `order_ids` lists the order account addresses to cancel.  If the slice
    /// is empty the API cancels ALL open orders for `maker`.
    ///
    /// Returns one or more unsigned transactions batched in groups of 5.
    pub async fn cancel_trigger_orders(
        &self,
        order_ids: &[&str],
        maker: &str,
    ) -> ExchangeResult<Value> {
        let mut body = serde_json::json!({
            "maker": maker,
            "computeUnitPrice": "auto",
        });

        if !order_ids.is_empty() {
            body["orders"] = serde_json::json!(order_ids);
        }

        self.post(JupiterEndpoint::TriggerCancelOrders, body).await
    }

    /// Get trigger orders for a wallet
    ///
    /// `status` must be `"active"` or `"history"`.
    /// Results are paginated — 10 orders per page. Pass `page` to paginate.
    pub async fn get_trigger_orders(
        &self,
        wallet: &str,
        status: &str,
        page: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("user".to_string(), wallet.to_string());
        params.insert("orderStatus".to_string(), status.to_string());
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        self.get(JupiterEndpoint::TriggerGetOrders, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RECURRING API (DCA orders)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Create a time-based recurring (DCA) order
    ///
    /// Deposits `total_amount` of `input_mint` and splits it into `num_orders`
    /// executions, each separated by `interval_secs` seconds.
    ///
    /// Returns an unsigned Solana transaction and a `requestId` needed for
    /// `execute_recurring_order`.
    pub async fn create_recurring_order(
        &self,
        input_mint: &str,
        output_mint: &str,
        total_amount: u64,
        num_orders: u32,
        interval_secs: u64,
        maker: &str,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "user": maker,
            "inputMint": input_mint,
            "outputMint": output_mint,
            "params": {
                "time": {
                    "inAmount": total_amount,
                    "numberOfOrders": num_orders,
                    "interval": interval_secs,
                    "minPrice": null,
                    "maxPrice": null,
                    "startAt": null,
                }
            }
        });
        self.post(JupiterEndpoint::RecurringCreateOrder, body).await
    }

    /// Execute a signed recurring order transaction
    ///
    /// `signed_transaction` is the Base64-encoded signed Solana transaction.
    /// `request_id` is the UUID from `create_recurring_order`.
    pub async fn execute_recurring_order(
        &self,
        signed_transaction: &str,
        request_id: &str,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "signedTransaction": signed_transaction,
            "requestId": request_id,
        });
        self.post(JupiterEndpoint::RecurringExecute, body).await
    }

    /// Cancel a recurring order
    ///
    /// `order_id` is the on-chain order account address.
    /// Returns an unsigned Solana transaction and `requestId` for signing.
    pub async fn cancel_recurring_order(
        &self,
        order_id: &str,
        maker: &str,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "order": order_id,
            "user": maker,
            "recurringType": "time",
        });
        self.post(JupiterEndpoint::RecurringCancelOrder, body).await
    }

    /// Get recurring orders for a wallet
    ///
    /// `status` must be `"active"` or `"history"`.
    /// Results are paginated — 10 orders per page.
    pub async fn get_recurring_orders(
        &self,
        wallet: &str,
        status: &str,
        page: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("user".to_string(), wallet.to_string());
        params.insert("orderStatus".to_string(), status.to_string());
        params.insert("recurringType".to_string(), "time".to_string());
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        self.get(JupiterEndpoint::RecurringGetOrders, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TOKENS V2 API
    // ═══════════════════════════════════════════════════════════════════════════

    /// Search tokens by name or symbol
    ///
    /// Corresponds to `GET /tokens/v2/search?query=...`.
    pub async fn search_tokens(&self, query: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        self.get(JupiterEndpoint::TokenSearch, params).await
    }

    /// Get tokens by tag
    ///
    /// Corresponds to `GET /tokens/v2/tag/{tag}`.
    /// Common tags: `"verified"`, `"strict"`, `"community"`, `"lst"`.
    pub async fn get_token_by_tag(&self, tag: &str) -> ExchangeResult<Value> {
        let url = format!("{}/{}", JupiterEndpoint::TokenTag.url(&self.urls), tag);
        self.rate_limit_wait().await;
        let headers = self.auth.auth_headers();
        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        JupiterParser::check_error(&response)?;
        Ok(response)
    }

    /// Get trending tokens by category and interval
    ///
    /// Corresponds to `GET /tokens/v2/{category}/{interval}`.
    /// Example categories: `"trending"`, `"new"`.
    /// Example intervals: `"1h"`, `"6h"`, `"24h"`.
    pub async fn get_trending_tokens(
        &self,
        category: &str,
        interval: &str,
    ) -> ExchangeResult<Value> {
        let url = format!(
            "{}/{}/{}",
            JupiterEndpoint::TokenCategory.url(&self.urls),
            category,
            interval,
        );
        self.rate_limit_wait().await;
        let headers = self.auth.auth_headers();
        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        JupiterParser::check_error(&response)?;
        Ok(response)
    }

    /// Get recently created tokens
    ///
    /// Corresponds to `GET /tokens/v2/recent`.
    pub async fn get_recent_tokens(&self) -> ExchangeResult<Value> {
        self.get(JupiterEndpoint::TokenRecent, HashMap::new()).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ON-CHAIN INTEGRATION (onchain-solana feature)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Attach a Solana chain provider to enable on-chain swap execution.
    ///
    /// Once a provider is set, [`submit_swap`] can deserialize, sign, and
    /// broadcast transactions returned by the Jupiter API.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use digdigdig3::core::chain::SolanaProvider;
    ///
    /// let provider = Arc::new(SolanaProvider::mainnet());
    /// let connector = JupiterConnector::new(api_key).await?
    ///     .with_solana_provider(provider);
    /// ```
    #[cfg(feature = "onchain-solana")]
    pub fn with_solana_provider(mut self, provider: Arc<SolanaProvider>) -> Self {
        self.solana_provider = Some(provider);
        self
    }

    /// Deserialize, sign, and submit a Jupiter swap transaction to the Solana network.
    ///
    /// Jupiter's swap API returns a base64-encoded, partially-built (unsigned) Solana
    /// transaction. This method:
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
    /// - `ExchangeError::InvalidRequest` if the base64 or bincode decoding fails.
    /// - `ExchangeError::Network` if the RPC submission fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Get an unsigned swap transaction from Jupiter
    /// let order = connector.create_ultra_swap(
    ///     "So11111111111111111111111111111111111111112",  // SOL
    ///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    ///     1_000_000_000, // 1 SOL in lamports
    ///     50,            // 0.5% slippage
    ///     &keypair.pubkey().to_string(),
    /// ).await?;
    ///
    /// let tx_b64 = order["transaction"].as_str().unwrap();
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

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for JupiterConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Jupiter
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

    fn is_testnet(&self) -> bool {
        false // Jupiter operates on Solana mainnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // DEX only supports spot-like swaps
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for JupiterConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let (input_mint, _output_mint) = self.symbol_to_mints(&symbol)?;

        // Use Price API (requires auth)
        // Get price of base asset (input_mint) in terms of quote asset
        let mut params = HashMap::new();
        params.insert("ids".to_string(), input_mint.clone());

        let response = self.get(JupiterEndpoint::Price, params).await?;
        JupiterParser::parse_price_from_api(&response, &input_mint)
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Jupiter is a DEX aggregator that routes trades across 20+ Solana DEXes
        // (including AMM pools and orderbooks), but does not maintain its own orderbook.
        //
        // Jupiter aggregates liquidity from: Raydium, Orca, Phoenix, OpenBook, Meteora,
        // Lifinity, GooseFX, Invariant, Cropper, Balansol, and others.
        //
        // Alternative: For orderbook data, query individual DEXes directly (e.g., Phoenix, OpenBook).
        // For aggregated depth simulation, make multiple quote requests at different amounts.
        Err(ExchangeError::UnsupportedOperation(
            "Orderbooks not supported - Jupiter is an aggregator. Use get_price() or query source DEXes.".to_string()
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
        // Jupiter doesn't provide historical kline data
        Err(ExchangeError::UnsupportedOperation(
            "Klines not supported by Jupiter".to_string(),
        ))
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let (_, output_mint) = self.symbol_to_mints(&symbol)?;

        let mut params = HashMap::new();
        params.insert("ids".to_string(), output_mint.clone());

        let response = self.get(JupiterEndpoint::Price, params).await?;
        JupiterParser::parse_ticker_from_price(&response, &output_mint)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Simple health check using Price API (requires auth)
        let mut params = HashMap::new();
        params.insert("ids".to_string(), MintRegistry::SOL.to_string());

        let _ = self.get(JupiterEndpoint::Price, params).await?;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

// Jupiter trading (swap execution) requires:
// 1. Solana wallet integration (@solana/web3.js or solana-sdk)
// 2. Wallet keypair for transaction signing
// 3. Quote API → Swap API → sign tx → submit via Solana RPC
// 4. Transaction confirmation monitoring
//
// The Jupiter REST API provides quote/routing data only.
// Actual swap execution requires signed Solana transactions.

#[async_trait]
impl Trading for JupiterConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "Jupiter swap execution requires Solana wallet integration. \
             Use Quote API to get routing, then sign and submit transaction via Solana RPC."
                .to_string(),
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "Jupiter swaps are atomic Solana transactions — they cannot be cancelled. \
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
            "Jupiter does not have order tracking. \
             Use Solana transaction signature to check swap status via Solana RPC."
                .to_string(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Jupiter swaps are atomic — there are no open/pending orders. \
             Limit orders (if using Jupiter Limit Order) require separate integration."
                .to_string(),
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Jupiter does not provide order history via REST API. \
             Query Solana transaction history via RPC for swap records."
                .to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for JupiterConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        // The wallet address is passed via `BalanceQuery::asset` when querying a
        // specific address, or we fall back to the api key owner wallet if the
        // field is empty.  Jupiter has no custody — balances live on-chain and
        // are surfaced via the Ultra holdings endpoint.
        let wallet = if let Some(ref asset) = query.asset {
            if asset.is_empty() {
                return Err(ExchangeError::InvalidRequest(
                    "Wallet address is required in BalanceQuery::asset for Jupiter".to_string(),
                ));
            }
            asset.clone()
        } else {
            return Err(ExchangeError::InvalidRequest(
                "Wallet address is required in BalanceQuery::asset for Jupiter".to_string(),
            ));
        };

        let response = self.get_ultra_holdings(&wallet).await?;

        // Parse native SOL balance from top level
        let mut balances: Vec<Balance> = Vec::new();

        if let Some(ui_amount) = response.get("uiAmount").and_then(|v| v.as_f64()) {
            balances.push(Balance {
                asset: "SOL".to_string(),
                free: ui_amount,
                locked: 0.0,
                total: ui_amount,
            });
        }

        // Parse SPL token balances from the `tokens` map
        if let Some(tokens) = response.get("tokens").and_then(|v| v.as_object()) {
            for (mint, token_data) in tokens {
                let ui_amount = token_data
                    .get("uiAmount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                balances.push(Balance {
                    asset: mint.clone(),
                    free: ui_amount,
                    locked: 0.0,
                    total: ui_amount,
                });
            }
        }

        Ok(balances)
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Jupiter is a DEX aggregator with no account concept. \
             Use Solana wallet address to query on-chain account data."
                .to_string(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Jupiter charges a platform fee on top of routing fees.
        // DEX fees vary per source pool (Raydium: 0.25%, Orca: 0.3%, etc.).
        // Jupiter platform fee: 0% (as of 2025, fees embedded in price impact).
        Err(ExchangeError::UnsupportedOperation(
            "Jupiter fees are protocol-level (0% platform fee + per-DEX pool fees). \
             Not translatable to maker/taker rates. Fee is included in swap price impact."
                .to_string(),
        ))
    }
}
