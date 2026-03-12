//! # Bitstamp Connector
//!
//! Implementation of all core traits for Bitstamp V2 API.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//!
//! ## Extended Methods
//! Additional Bitstamp-specific methods as struct methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    CancelAllResponse,
    ExchangeIdentity, MarketData, Trading, Account,
    CancelAll,
};
use crate::core::types::SymbolInfo;
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{BitstampUrls, BitstampEndpoint, format_symbol, map_kline_interval};
use super::auth::BitstampAuth;
use super::parser::BitstampParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitstamp connector
pub struct BitstampConnector {
    /// HTTP client
    http: HttpClient,
    /// Reqwest client for form-encoded POSTs
    reqwest_client: reqwest::Client,
    /// Authentication (None for public methods)
    auth: Option<BitstampAuth>,
    /// Rate limiter (~167 requests per second sustained, 10000/10min)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BitstampConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let reqwest_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ExchangeError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let auth = credentials.as_ref().map(BitstampAuth::new);

        // Initialize rate limiter: ~167 req/s (~10000 per 10 minutes)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(167, Duration::from_secs(1))
        ));

        Ok(Self {
            http,
            reqwest_client,
            auth,
            rate_limiter,
        })
    }

    /// Create connector only for public methods
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            // Scope the lock to ensure it's dropped before await
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire() {
                    return; // Successfully acquired, exit early
                }
                limiter.time_until_ready()
            }; // Lock is dropped here

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: BitstampEndpoint,
        pair: Option<&str>,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit (weight 1 for GET requests)
        self.rate_limit_wait().await;

        let base_url = BitstampUrls::base_url();
        let path = if let Some(p) = pair {
            endpoint.path_with_pair(p)
        } else {
            endpoint.path().to_string()
        };

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query)
        };

        let response = self.http.get(&url, &HashMap::new()).await?;
        Ok(response)
    }

    /// POST request (authenticated)
    async fn post(
        &self,
        endpoint: BitstampEndpoint,
        pair: Option<&str>,
        body_params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit (weight 1 for POST requests)
        self.rate_limit_wait().await;

        let base_url = BitstampUrls::base_url();
        let path = if let Some(p) = pair {
            endpoint.path_with_pair(p)
        } else {
            endpoint.path().to_string()
        };

        // Build form-encoded body
        let body = if body_params.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = body_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            pairs.join("&")
        };

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("POST", &path, "", &body);

        let url = format!("{}{}", base_url, path);

        // Use reqwest directly for form-encoded POST
        let mut request = self.reqwest_client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded");

        // Add auth headers
        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }

        // Set body
        if !body.is_empty() {
            request = request.body(body.clone());
        }

        let response = request.send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        let body_text = response.text()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: body_text,
            });
        }

        let json: Value = serde_json::from_str(&body_text)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Bitstamp-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all tickers
    pub async fn get_all_tickers(&self) -> ExchangeResult<Vec<Ticker>> {
        // Bitstamp doesn't have a single endpoint for all tickers
        // Would need to fetch markets list and then ticker for each
        Ok(vec![])
    }

    /// Get markets information
    pub async fn get_markets(&self) -> ExchangeResult<Value> {
        self.get(BitstampEndpoint::Markets, None, HashMap::new()).await
    }

    /// Cancel all orders
    pub async fn cancel_all_orders(&self) -> ExchangeResult<Vec<String>> {
        let response = self.post(BitstampEndpoint::CancelAllOrders, None, HashMap::new()).await?;
        // Response is success/error, not a list of cancelled IDs
        BitstampParser::check_error(&response)?;
        Ok(vec![])
    }

    /// Get open positions (perpetual futures)
    pub async fn get_open_positions(&self) -> ExchangeResult<Value> {
        self.post(BitstampEndpoint::OpenPositions, None, HashMap::new()).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BitstampConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitstamp
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
        false // Bitstamp doesn't have testnet via this API
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::Margin,
            AccountType::FuturesCross,
            AccountType::FuturesIsolated,
        ]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for BitstampConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let pair = format_symbol(&symbol, account_type);
        let response = self.get(BitstampEndpoint::Ticker, Some(&pair), HashMap::new()).await?;
        let ticker = BitstampParser::parse_ticker(&response)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let pair = format_symbol(&symbol, account_type);
        let response = self.get(BitstampEndpoint::Orderbook, Some(&pair), HashMap::new()).await?;
        BitstampParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let pair = format_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("step".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1000).to_string());
        }

        if let Some(et) = end_time {
            params.insert("end".to_string(), (et / 1000).to_string());
        }

        let response = self.get(BitstampEndpoint::Ohlc, Some(&pair), params).await?;
        BitstampParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let pair = format_symbol(&symbol, account_type);
        let response = self.get(BitstampEndpoint::Ticker, Some(&pair), HashMap::new()).await?;
        BitstampParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Use markets endpoint as ping (always available)
        let _ = self.get(BitstampEndpoint::Markets, None, HashMap::new()).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // GET /api/v2/trading-pairs-info/ returns detailed symbol info with name, url_symbol, etc.
        self.rate_limit_wait().await;
        let url = format!("{}/api/v2/trading-pairs-info/", BitstampUrls::base_url());
        let response = self.http.get(&url, &HashMap::new()).await?;
        BitstampParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BitstampConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let pair = format_symbol(&symbol, account_type);
                
                        let mut params = HashMap::new();
                        params.insert("amount".to_string(), quantity.to_string());
                
                        let endpoint = match side {
                            OrderSide::Buy => BitstampEndpoint::BuyMarket,
                            OrderSide::Sell => BitstampEndpoint::SellMarket,
                        };
                
                        let response = self.post(endpoint, Some(&pair), params).await?;
                        BitstampParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let pair = format_symbol(&symbol, account_type);
                
                        let mut params = HashMap::new();
                        params.insert("amount".to_string(), quantity.to_string());
                        params.insert("price".to_string(), price.to_string());
                
                        let endpoint = match side {
                            OrderSide::Buy => BitstampEndpoint::BuyLimit,
                            OrderSide::Sell => BitstampEndpoint::SellLimit,
                        };
                
                        let response = self.post(endpoint, Some(&pair), params).await?;
                        BitstampParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // POST /api/v2/user_transactions/ returns trade executions
        let mut params = HashMap::new();
        params.insert("sort".to_string(), "desc".to_string());

        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        // Bitstamp user_transactions supports offset but not a direct date filter in v2
        let response = self.post(BitstampEndpoint::UserTransactions, None, params).await?;
        BitstampParser::parse_user_transactions(&response)
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let mut params = HashMap::new();
            params.insert("id".to_string(), order_id.to_string());

            let response = self.post(BitstampEndpoint::CancelOrder, None, params).await?;
            BitstampParser::parse_order(&response)
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        let mut params = HashMap::new();
        params.insert("id".to_string(), order_id.to_string());

        let response = self.post(BitstampEndpoint::OrderStatus, None, params).await?;
        BitstampParser::parse_order(&response)
    
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let _symbol_str = _symbol;
        let _symbol: Option<crate::core::Symbol> = _symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let response = self.post(BitstampEndpoint::OpenOrders, None, HashMap::new()).await?;
        BitstampParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BitstampConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.post(BitstampEndpoint::Balance, None, HashMap::new()).await?;
        BitstampParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.post(BitstampEndpoint::Balance, None, HashMap::new()).await?;
        let balances = BitstampParser::parse_balance(&response)?;

        // Bitstamp doesn't have a separate account info endpoint
        // We'll construct from balances
        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.5, // Bitstamp default maker fee
            taker_commission: 0.5, // Bitstamp default taker fee
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // POST /api/v2/fees/trading/ — returns per-pair fee info
        let response = self.post(BitstampEndpoint::TradingFees, None, HashMap::new()).await?;
        BitstampParser::parse_fee_rate(&response, symbol)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for BitstampConnector {
    async fn cancel_all_orders(
        &self,
        _scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        // Bitstamp only supports cancel-all globally (no per-symbol cancel-all via API)
        let response = self.post(BitstampEndpoint::CancelAllOrders, None, HashMap::new()).await?;
        BitstampParser::check_error(&response)?;

        Ok(CancelAllResponse {
            cancelled_count: 0, // Bitstamp returns true/false, not count
            failed_count: 0,
            details: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connector_creation() {
        let connector = BitstampConnector::public().await;
        assert!(connector.is_ok());
    }

    #[test]
    fn test_exchange_identity() {
        // Can't use tokio::test for non-async test
        let http = HttpClient::new(30_000).unwrap();
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(167, Duration::from_secs(1))
        ));

        let reqwest_client = reqwest::Client::new();

        let connector = BitstampConnector {
            http,
            reqwest_client,
            auth: None,
            rate_limiter,
        };

        assert_eq!(connector.exchange_id(), ExchangeId::Bitstamp);
        assert!(!connector.is_testnet());

        let account_types = connector.supported_account_types();
        assert!(account_types.contains(&AccountType::Spot));
        assert!(account_types.contains(&AccountType::FuturesCross));
        assert!(account_types.contains(&AccountType::FuturesIsolated));

        assert_eq!(connector.exchange_type(), ExchangeType::Cex);
    }
}
