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
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::{
    OrderRequest, CancelRequest, Order, OrderHistoryFilter, PlaceOrderResponse,
    BalanceQuery, Balance, AccountInfo, FeeInfo,
    PositionQuery, Position, FundingRate, PositionModification,
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

// GMX trading requires:
// 1. Web3/Ethers library for smart contract interaction (ethers-rs or alloy)
// 2. Wallet private key for EIP-712 transaction signing
// 3. ERC20 token approvals (approve ExchangeRouter contract)
// 4. Multi-call transactions (transfer + createOrder in one tx)
// 5. Blockchain event monitoring for execution callbacks
// 6. Keeper network executes orders asynchronously (not synchronous)
//
// REST API is read-only — no trading endpoints exist.

#[async_trait]
impl Trading for GmxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "GMX trading requires blockchain wallet integration (ethers-rs/alloy). \
             REST API is read-only. Use GMX SDK or ExchangeRouter contract directly."
                .to_string(),
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "GMX order cancellation requires smart contract transaction. \
             REST API is read-only."
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
            "GMX does not provide a REST endpoint to query individual orders by ID. \
             Use blockchain indexer (e.g. The Graph) or contract event logs."
                .to_string(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "GMX open orders require account address and smart contract queries. \
             REST API does not expose per-account open orders."
                .to_string(),
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "GMX order history requires The Graph subgraph or contract event logs. \
             REST API is read-only and does not expose order history."
                .to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for GmxConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "GMX balances require ERC-20 token contract queries or The Graph. \
             REST API does not expose wallet balances."
                .to_string(),
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "GMX has no account concept in its REST API. \
             Account data requires on-chain queries."
                .to_string(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // GMX uses a protocol fee model, not maker/taker.
        // Fees: 0.05% open + 0.05% close for market orders (position fee),
        // plus price impact and funding rates.
        // Not translatable to a maker/taker FeeInfo struct.
        Err(ExchangeError::UnsupportedOperation(
            "GMX uses protocol fees (position fee, price impact, funding) not maker/taker rates. \
             See GMX docs for fee details."
                .to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for GmxConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "GMX positions require smart contract queries (PositionStore contract) \
             or The Graph subgraph. REST API does not expose per-account positions."
                .to_string(),
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "GMX uses borrowing fees (not funding rates). \
             Borrow rates are available in /markets REST endpoint as annualized rates."
                .to_string(),
        ))
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
