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
use crate::core::traits::{ExchangeIdentity, MarketData};
use crate::core::types::ConnectorStats;
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

// Note: Trading and Account traits are not implemented for Jupiter connector
// as this is Phase 2 (market data only). Trading would require Solana wallet
// integration and transaction signing, which is beyond scope.
