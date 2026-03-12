//! # Gemini Connector
//!
//! Реализация всех core трейтов для Gemini.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Extended методы
//! Дополнительные Gemini-специфичные методы как методы структуры.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol, Asset,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType,Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{GeminiUrls, GeminiEndpoint, format_symbol, normalize_symbol, map_kline_interval};
use super::auth::GeminiAuth;
use super::parser::GeminiParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Gemini коннектор
pub struct GeminiConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<GeminiAuth>,
    /// URL'ы (mainnet/testnet)
    urls: GeminiUrls,
    /// Testnet mode
    testnet: bool,
    /// Public rate limiter (120 req/min = 2 req/sec)
    public_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Private rate limiter (600 req/min = 10 req/sec)
    private_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl GeminiConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            GeminiUrls::TESTNET
        } else {
            GeminiUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(GeminiAuth::new)
            .transpose()?;

        // Initialize rate limiters: public 120 req/min, private 600 req/min
        let public_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(120, Duration::from_secs(60))
        ));
        let private_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(600, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            public_limiter,
            private_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, is_private: bool) {
        let limiter = if is_private {
            &self.private_limiter
        } else {
            &self.public_limiter
        };

        loop {
            let wait_time = {
                let mut lim = limiter.lock().expect("Mutex poisoned");
                if lim.try_acquire() {
                    return;
                }
                lim.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: GeminiEndpoint,
        path_params: &[(&str, &str)],
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(endpoint.requires_auth()).await;

        let base_url = self.urls.rest_url(AccountType::Spot);
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        let response = self.http.get(&url, &HashMap::new()).await?;
        GeminiParser::check_error(&response)?;
        Ok(response)
    }

    /// POST запрос (всегда требует auth)
    async fn post(
        &self,
        endpoint: GeminiEndpoint,
        params: HashMap<String, Value>,
        path_params: &[(&str, &str)],
    ) -> ExchangeResult<Value> {
        // Wait for rate limit (POST is always private)
        self.rate_limit_wait(true).await;

        let base_url = self.urls.rest_url(AccountType::Spot);
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&path, params)?;

        // Gemini POST requests have empty body, everything in headers
        let response = self.http.post(&url, &json!({}), &headers).await?;
        GeminiParser::check_error(&response)?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for GeminiConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Gemini
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.public_limiter.lock() {
            (lim.current_count(), lim.max_requests())
        } else {
            (0, 0)
        };
        let rate_groups = {
            let pub_stats = self.public_limiter.lock()
                .map(|mut lim| (lim.current_count(), lim.max_requests()))
                .unwrap_or((0, 0));
            let priv_stats = self.private_limiter.lock()
                .map(|mut lim| (lim.current_count(), lim.max_requests()))
                .unwrap_or((0, 0));
            vec![
                ("public".to_string(), pub_stats.0, pub_stats.1),
                ("private".to_string(), priv_stats.0, priv_stats.1),
            ]
        };
        ConnectorStats {
            http_requests,
            http_errors,
            last_latency_ms,
            rate_used,
            rate_max,
            rate_groups,
            ws_ping_rtt_ms: 0,
        }
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::FuturesCross,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for GeminiConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(
            GeminiEndpoint::Ticker,
            &[("symbol", &symbol_str)],
        ).await?;

        let ticker = GeminiParser::parse_ticker(&response, &symbol_str)?;
        Ok(ticker.last_price)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(
            GeminiEndpoint::TickerV2,
            &[("symbol", &symbol_str)],
        ).await?;

        GeminiParser::parse_ticker(&response, &symbol_str)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(
            GeminiEndpoint::OrderBook,
            &[("symbol", &symbol_str)],
        ).await?;

        GeminiParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        _limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));
        let time_frame = map_kline_interval(interval);

        // Use DerivativeCandles endpoint for futures
        let endpoint = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            GeminiEndpoint::DerivativeCandles
        } else {
            GeminiEndpoint::Candles
        };

        let response = self.get(
            endpoint,
            &[("symbol", &symbol_str), ("time_frame", time_frame)],
        ).await?;

        GeminiParser::parse_candles(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Gemini doesn't have a dedicated ping endpoint, use symbols as health check
        self.get(GeminiEndpoint::Symbols, &[]).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Fetch all symbols first, then get details for each
        let symbols_response = self.get(GeminiEndpoint::Symbols, &[]).await?;
        let symbols = GeminiParser::parse_symbols(&symbols_response)?;

        let mut result = Vec::with_capacity(symbols.len());

        for symbol_lower in &symbols {
            // Skip non-spot/perpetual symbols (e.g. contain digits like options)
            // Only process lowercase alpha symbols
            if !symbol_lower.chars().all(|c| c.is_alphabetic()) {
                continue;
            }

            match self.get(GeminiEndpoint::SymbolDetails, &[("symbol", symbol_lower)]).await {
                Ok(details) => {
                    if let Some(info) = GeminiParser::parse_symbol_details(&details, symbol_lower) {
                        result.push(info);
                    }
                }
                Err(_) => continue, // Skip symbols where details fetch fails
            }
        }

        Ok(result)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for GeminiConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));
                
                        let mut params = HashMap::new();
                        params.insert("symbol".to_string(), json!(symbol_str));
                        params.insert("amount".to_string(), json!(quantity.to_string()));
                        params.insert("side".to_string(), json!(match side {
                            OrderSide::Buy => "buy",
                            OrderSide::Sell => "sell",
                        }));
                        params.insert("type".to_string(), json!("exchange market"));
                
                        let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                        GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));
                
                        let mut params = HashMap::new();
                        params.insert("symbol".to_string(), json!(symbol_str));
                        params.insert("amount".to_string(), json!(quantity.to_string()));
                        params.insert("price".to_string(), json!(price.to_string()));
                        params.insert("side".to_string(), json!(match side {
                            OrderSide::Buy => "buy",
                            OrderSide::Sell => "sell",
                        }));
                        params.insert("type".to_string(), json!("exchange limit"));
                
                        let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                        GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "get_order_history not yet implemented".to_string()
        ))
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let mut params = HashMap::new();
            params.insert("order_id".to_string(), json!(order_id.parse::<i64>().unwrap_or(0)));

            let response = self.post(GeminiEndpoint::CancelOrder, params, &[]).await?;
            GeminiParser::parse_order(&response)
    
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
        params.insert("order_id".to_string(), json!(order_id.parse::<i64>().unwrap_or(0)));

        let response = self.post(GeminiEndpoint::OrderStatus, params, &[]).await?;
        GeminiParser::parse_order(&response)
    
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

        let response = self.post(GeminiEndpoint::ActiveOrders, HashMap::new(), &[]).await?;
        GeminiParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for GeminiConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;
        let response = self.post(GeminiEndpoint::Balances, HashMap::new(), &[]).await?;
        GeminiParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Gemini doesn't have a specific account info endpoint
        // Return minimal info
        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0,
            taker_commission: 0.0,
            balances: vec![],
        })
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for GeminiConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let _account_type = query.account_type;

        let response = self.post(GeminiEndpoint::Positions, HashMap::new(), &[]).await?;
        GeminiParser::parse_positions(&response)
    
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let symbol_str = symbol;
        let symbol = {
            let parts: Vec<&str> = symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: symbol_str.to_string(), quote: String::new(), raw: Some(symbol_str.to_string()) }
            }
        };

        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, AccountType::FuturesCross));

        let response = self.get(
            GeminiEndpoint::FundingAmount,
            &[("symbol", &symbol_str)],
        ).await?;

        GeminiParser::parse_funding_rate(&response)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                // Gemini doesn't have a set leverage endpoint
                // Leverage is managed through margin settings
                Err(ExchangeError::NotSupported("Set leverage not supported by Gemini".to_string()))
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Gemini-специфичные)
// ═══════════════════════════════════════════════════════════════════════════════

impl GeminiConnector {
    /// Get all available symbols
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(GeminiEndpoint::Symbols, &[]).await?;
        GeminiParser::parse_symbols(&response)
    }

    /// Cancel all active orders
    pub async fn cancel_all_orders(&self) -> ExchangeResult<()> {
        self.post(GeminiEndpoint::CancelAllOrders, HashMap::new(), &[]).await?;
        Ok(())
    }

    /// Get notional volume and fee information
    pub async fn get_notional_volume(&self) -> ExchangeResult<Value> {
        self.post(GeminiEndpoint::NotionalVolume, HashMap::new(), &[]).await
    }

    /// Get funding payment history for perpetuals
    pub async fn get_funding_payments(
        &self,
        since: Option<i64>,
        to: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();

        if let Some(s) = since {
            params.insert("since".to_string(), json!(s));
        }
        if let Some(t) = to {
            params.insert("to".to_string(), json!(t));
        }

        self.post(GeminiEndpoint::FundingPayments, params, &[]).await
    }

    /// Get margin account summary
    pub async fn get_margin_info(&self) -> ExchangeResult<Value> {
        self.post(GeminiEndpoint::MarginAccount, HashMap::new(), &[]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connector_creation() {
        let connector = GeminiConnector::public(false).await.unwrap();
        assert_eq!(connector.exchange_id(), ExchangeId::Gemini);
        assert!(!connector.is_testnet());
    }

    #[test]
    fn test_format_symbol() {
        let symbol = format_symbol("BTC", "USD", AccountType::Spot);
        assert_eq!(symbol, "btcusd");

        let symbol = format_symbol("ETH", "USD", AccountType::FuturesCross);
        assert_eq!(symbol, "ethgusdperp");
    }
}
