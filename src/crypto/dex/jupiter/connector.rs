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

        Ok(Self { http, auth, urls, rate_limiter })
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
    /// execution. Returns an order object including the transaction to sign.
    ///
    /// `input_mint` and `output_mint` are Solana mint addresses.
    /// `amount` is the raw input amount (in lamports / smallest unit).
    /// `slippage_bps` is the maximum acceptable slippage in basis points (e.g. 50 = 0.5%).
    pub async fn create_ultra_swap_order(
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
        self.post(JupiterEndpoint::UltraSwapOrder, body).await
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
    /// Returns the transaction signature and confirmation status.
    pub async fn execute_ultra_swap(&self, signed_transaction: &str) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "signedTransaction": signed_transaction,
        });
        self.post(JupiterEndpoint::UltraSwapExecute, body).await
    }

    /// Get token balances for a wallet address
    ///
    /// Returns all SPL token balances for the given Solana wallet public key
    /// as seen by the Jupiter Ultra API.
    /// Corresponds to `GET /ultra/v1/balances`.
    pub async fn get_ultra_balances(&self, wallet: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("userPublicKey".to_string(), wallet.to_string());
        self.get(JupiterEndpoint::UltraSwapBalances, params).await
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
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Jupiter has no account system. \
             Query SPL token balances directly via Solana RPC (getTokenAccountsByOwner)."
                .to_string(),
        ))
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
