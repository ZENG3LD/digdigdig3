//! # GMX Connector
//!
//! Implementation of core traits for GMX V2.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data (REST API)
//!
//! ## Limitations
//! - Trading requires blockchain wallet integration (not implemented)
//! - Account/Positions require smart contract queries (not implemented)
//! - WebSocket uses polling instead of native WebSocket API

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
use crate::core::traits::{
    ExchangeIdentity, MarketData,
};
use crate::core::utils::SimpleRateLimiter;
use crate::core::types::{ConnectorStats, SymbolInfo};

use super::endpoints::{GmxUrls, GmxEndpoint, format_symbol, map_kline_interval};
use super::auth::GmxAuth;
use super::parser::GmxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// GMX connector
pub struct GmxConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (no-op for public REST endpoints)
    #[allow(dead_code)]
    auth: GmxAuth,
    /// URLs
    urls: GmxUrls,
    /// Chain (arbitrum/avalanche)
    chain: String,
    /// Rate limiter (conservative: 12 req/60s, ~1 req/5s)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
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

        // Conservative: 12 requests per 60 seconds (~1 req/5s)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(12, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            chain,
            rate_limiter,
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
                step_size: None,
                min_notional: None,
            }
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING (NOT IMPLEMENTED - Requires blockchain integration)
// ═══════════════════════════════════════════════════════════════════════════════

// GMX trading requires:
// 1. Web3/Ethers library for smart contract interaction
// 2. Wallet private key for transaction signing
// 3. ERC20 token approvals
// 4. Multi-call transactions (transfer + createOrder)
// 5. Blockchain event monitoring
//
// This is beyond the scope of the basic V5 connector.
// Future implementation should use ethers-rs or alloy.

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
