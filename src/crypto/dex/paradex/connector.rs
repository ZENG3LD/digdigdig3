//! # Paradex Connector
//!
//! Реализация всех core трейтов для Paradex.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Note
//! Paradex работает только с perpetual futures (no spot trading).
//! Все методы используют JWT authentication.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, OrderStatus, Balance, AccountInfo,
    Position, FundingRate, TimeInForce,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::GroupRateLimiter;
use crate::core::types::{ConnectorStats, SymbolInfo};

use super::endpoints::{ParadexUrls, ParadexEndpoint, format_symbol, map_kline_resolution};
use super::auth::ParadexAuth;
use super::parser::ParadexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Paradex коннектор
pub struct ParadexConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (JWT-based)
    auth: Arc<ParadexAuth>,
    /// URL'ы (mainnet/testnet)
    urls: ParadexUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (grouped: public=1500/60s, orders=17250/60s, private_gets=600/60s)
    rate_limiter: Arc<Mutex<GroupRateLimiter>>,
}

impl ParadexConnector {
    /// Создать новый коннектор
    ///
    /// ВАЖНО: Credentials должны содержать JWT token в api_key поле.
    /// Для полной реализации нужна интеграция со StarkNet signing.
    pub async fn new(credentials: Credentials, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            ParadexUrls::TESTNET
        } else {
            ParadexUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = Arc::new(ParadexAuth::new(&credentials)?);

        // Sync time with server
        let base_url = urls.rest_url();
        let url = format!("{}/system/time", base_url);
        if let Ok(response) = http.get(&url, &HashMap::new()).await {
            if let Some(server_time) = response.get("server_time").and_then(|t| t.as_i64()) {
                auth.sync_time(server_time).await;
            }
        }

        // Paradex rate limits grouped by endpoint category
        let mut rl = GroupRateLimiter::new();
        rl.add_group("public", 1500, Duration::from_secs(60));
        rl.add_group("orders", 17250, Duration::from_secs(60));
        rl.add_group("private_gets", 600, Duration::from_secs(60));
        let rate_limiter = Arc::new(Mutex::new(rl));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Создать коннектор только для публичных методов (без auth)
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        let credentials = Credentials::new("", ""); // Empty credentials
        Self::new(credentials, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    ///
    /// Groups: "public" (GET public endpoints), "orders" (POST/DELETE orders), "private_gets" (GET private endpoints)
    async fn rate_limit_wait(&self, group: &str, weight: u32) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(group, weight) {
                    return;
                }
                limiter.time_until_ready(group, weight)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// Update rate limiter from Paradex response headers.
    ///
    /// Paradex reports: `x-ratelimit-remaining`, `x-ratelimit-limit`, `x-ratelimit-window`.
    /// The `group` parameter identifies which limiter to update (the caller knows this from context).
    fn update_rate_from_headers(&self, headers: &HeaderMap, group: &str) {
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());
        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());
        if let (Some(remaining), Some(limit)) = (remaining, limit) {
            let used = limit.saturating_sub(remaining);
            if let Ok(mut limiter) = self.rate_limiter.lock() {
                limiter.update_from_server(group, used);
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: ParadexEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let group = if endpoint.requires_auth() { "private_gets" } else { "public" };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters (e.g., {market}, {order_id})
        for (key, value) in &params {
            if path.contains(&format!("{{{}}}", key)) {
                path = path.replace(&format!("{{{}}}", key), value);
            }
        }

        // Build query string (exclude path parameters)
        let query_params: HashMap<_, _> = params.iter()
            .filter(|(k, _)| !path.contains(&format!("{{{}}}", k)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let query = if query_params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = query_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);
        let full_path = format!("{}{}", path, query);

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            self.auth.sign_request("GET", &full_path, "").await?
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, group);
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: ParadexEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("orders", 1).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers (Paradex POST endpoints требуют JWT)
        let body_str = body.to_string();
        let headers = self.auth.sign_request("POST", path, &body_str).await?;

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers, "orders");
        self.check_response(&response)?;
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: ParadexEndpoint,
        path_params: &[(&str, &str)],
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("orders", 1).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Auth headers
        let headers = self.auth.sign_request("DELETE", &path, "").await?;

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, "orders");
        self.check_response(&response)?;
        Ok(response)
    }

    /// PUT запрос (для modify order)
    async fn _put(
        &self,
        endpoint: ParadexEndpoint,
        path_params: &[(&str, &str)],
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("orders", 1).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Auth headers
        let body_str = body.to_string();
        let headers = self.auth.sign_request("PUT", &path, &body_str).await?;

        let response = self.http.put(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Проверить response на ошибки
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        // Paradex error format: {"error": "ERROR_CODE", "message": "description"}
        if let Some(error) = response.get("error") {
            let code = error.as_str().unwrap_or("UNKNOWN");
            let message = response.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            return Err(ExchangeError::Api {
                code: -1,
                message: format!("{}: {}", code, message),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Paradex-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить все символы (markets)
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(ParadexEndpoint::Markets, HashMap::new()).await?;
        ParadexParser::parse_symbols(&response)
    }

    /// Получить все маркеты с summary data
    pub async fn get_markets_summary(&self, market: Option<String>) -> ExchangeResult<Vec<Ticker>> {
        let mut params = HashMap::new();
        if let Some(m) = market {
            params.insert("market".to_string(), m);
        } else {
            params.insert("market".to_string(), "ALL".to_string());
        }

        let response = self.get(ParadexEndpoint::MarketsSummary, params).await?;

        // Parse all tickers from results array
        let results = ParadexParser::extract_results(&response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        Ok(arr.iter()
            .filter_map(|item| {
                // Create a wrapper to use parse_ticker
                let wrapper = json!({"results": [item]});
                ParadexParser::parse_ticker(&wrapper).ok()
            })
            .collect())
    }

    /// Получить account summary
    pub async fn get_account_summary(&self) -> ExchangeResult<Value> {
        self.get(ParadexEndpoint::Account, HashMap::new()).await
    }

    /// Отменить все ордера
    pub async fn cancel_all_orders(&self, symbol: Option<Symbol>) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("market".to_string(), format_symbol(&s.base, &s.quote, AccountType::FuturesCross));
        }

        self.delete(ParadexEndpoint::CancelAllOrders, &[]).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for ParadexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Paradex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max, rate_groups) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            let (u, m) = limiter.primary_stats();
            let groups = limiter.all_stats()
                .into_iter()
                .map(|(name, cur, max)| (name.to_string(), cur, max))
                .collect();
            (u, m, groups)
        } else {
            (0, 0, Vec::new())
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
        vec![AccountType::FuturesCross] // Paradex только perpetual futures
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex // Paradex is a DEX
    }
}

#[async_trait]
impl MarketData for ParadexConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("market".to_string(), symbol_str);

        let response = self.get(ParadexEndpoint::MarketsSummary, params).await?;
        Ok(ParadexParser::parse_price(&response)?)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("market".to_string(), symbol_str.clone());
        if let Some(d) = depth {
            params.insert("depth".to_string(), d.to_string());
        }

        let response = self.get(ParadexEndpoint::Orderbook, params).await?;
        ParadexParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str);
        params.insert("resolution".to_string(), map_kline_resolution(interval).to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(ParadexEndpoint::Klines, params).await?;
        ParadexParser::parse_klines(&response)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("market".to_string(), symbol_str);

        let response = self.get(ParadexEndpoint::MarketsSummary, params).await?;
        ParadexParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(ParadexEndpoint::SystemState, HashMap::new()).await?;

        // Check if system is operational
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            if status == "operational" {
                return Ok(());
            }
        }

        Err(ExchangeError::Network("System not operational".to_string()))
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(ParadexEndpoint::Markets, HashMap::new()).await?;
        let symbols = ParadexParser::parse_symbols(&response)?;

        let infos = symbols.into_iter().map(|sym| {
            // Paradex format is "BTC-USD-PERP" - split on first "-"
            let parts: Vec<&str> = sym.splitn(2, '-').collect();
            let base = parts.first().copied().unwrap_or(&sym).to_string();
            let quote = if parts.len() > 1 {
                // Take "USD" from "USD-PERP"
                parts[1].split('-').next().unwrap_or("USD").to_string()
            } else {
                "USD".to_string()
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

#[async_trait]
impl Trading for ParadexConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                
                        let side_str = match side {
                            OrderSide::Buy => "BUY",
                            OrderSide::Sell => "SELL",
                        };
                
                        // NOTE: Paradex requires signature for each order
                        // This is a simplified version - full implementation needs StarkNet signing
                        let body = json!({
                            "market": symbol_str,
                            "side": side_str,
                            "type": "MARKET",
                            "size": quantity.to_string(),
                            "instruction": "IOC", // Market orders typically IOC
                            // MISSING: signature, signature_timestamp (requires StarkNet signing)
                        });
                
                        let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                        ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                
                        let side_str = match side {
                            OrderSide::Buy => "BUY",
                            OrderSide::Sell => "SELL",
                        };
                
                        let body = json!({
                            "market": symbol_str,
                            "side": side_str,
                            "type": "LIMIT",
                            "price": price.to_string(),
                            "size": quantity.to_string(),
                            "instruction": "GTC", // Good till cancel
                            // MISSING: signature, signature_timestamp
                        });
                
                        let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                        ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
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

            self.delete(ParadexEndpoint::CancelOrder, &[("order_id", order_id)]).await?;

            // Return a minimal cancelled order
            Ok(Order {
                id: order_id.to_string(),
                client_order_id: None,
                symbol: String::new(),
                side: OrderSide::Buy,
                order_type: OrderType::Limit { price: 0.0 },
                status: OrderStatus::Canceled,
                price: None,
                stop_price: None,
                quantity: 0.0,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: 0,
                updated_at: Some(crate::core::timestamp_millis() as i64),
                time_in_force: TimeInForce::Gtc,
            })
    
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
        params.insert("order_id".to_string(), order_id.to_string());

        let response = self.get(ParadexEndpoint::GetOrder, params).await?;
        ParadexParser::parse_order(&response)
    
    }

    async fn get_open_orders(&self, symbol: Option<&str>, account_type: AccountType) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            let parts: Vec<&str> = s.split('/').collect();
            let symbol_str = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], account_type)
            } else {
                s.to_string()
            };
            params.insert("market".to_string(), symbol_str);
        }

        let response = self.get(ParadexEndpoint::OpenOrders, params).await?;
        ParadexParser::parse_orders(&response)
    }
}

#[async_trait]
impl Account for ParadexConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get(ParadexEndpoint::Balances, HashMap::new()).await?;
        ParadexParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(ParadexEndpoint::Account, HashMap::new()).await?;
        ParadexParser::parse_account_info(&response)
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}

#[async_trait]
impl Positions for ParadexConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Paradex manages leverage automatically based on margin mode".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Paradex manages leverage automatically based on margin mode".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Paradex manages leverage automatically based on margin mode".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_public_connector() {
        let connector = ParadexConnector::public(true).await;
        assert!(connector.is_ok());
    }

    #[test]
    fn test_exchange_identity() {
        let connector = ParadexConnector::public(true);
        // This is async, so we can't test it directly here
        // But we can test the sync methods once we have instance
    }
}
