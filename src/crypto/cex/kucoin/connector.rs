//! # KuCoin Connector
//!
//! Реализация всех core трейтов для KuCoin.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Extended методы
//! Дополнительные KuCoin-специфичные методы как методы структуры.

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
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{KuCoinUrls, KuCoinEndpoint, format_symbol, map_kline_interval, map_futures_granularity};
use super::auth::KuCoinAuth;
use super::parser::KuCoinParser;

// KuCoin endpoint weights (VIP0 spot limits)
mod weights {
    pub const CANDLES: u32 = 3;
    pub const ORDERBOOK: u32 = 2;
    pub const ALL_TICKERS: u32 = 15;
    pub const _STATS: u32 = 8;
    pub const PLACE_ORDER: u32 = 2;
    pub const AMEND_ORDER: u32 = 1;
    pub const DEFAULT: u32 = 1;
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// KuCoin коннектор
pub struct KuCoinConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<KuCoinAuth>,
    /// URL'ы (mainnet/testnet)
    urls: KuCoinUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (4000 weight per 30 seconds)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl KuCoinConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            KuCoinUrls::TESTNET
        } else {
            KuCoinUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials
            .as_ref()
            .map(KuCoinAuth::new)
            .transpose()?;

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = urls.rest_url(AccountType::Spot);
            let url = format!("{}/api/v1/timestamp", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(data) = response.get("data").and_then(|d| d.as_i64()) {
                    if let Some(ref mut a) = auth {
                        a.sync_time(data);
                    }
                }
            }
        }

        // Initialize rate limiter: 4000 weight per 30 seconds (KuCoin spot VIP0)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(4000, Duration::from_secs(30))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from KuCoin response headers
    ///
    /// KuCoin reports: gw-ratelimit-remaining = remaining, gw-ratelimit-limit = total limit
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        let remaining = headers
            .get("gw-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let limit = headers
            .get("gw-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        if let (Some(remaining), Some(limit)) = (remaining, limit) {
            let used = limit.saturating_sub(remaining);
            if let Ok(mut limiter) = self.rate_limiter.lock() {
                limiter.update_from_server(used);
            }
        }
    }

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            // Scope the lock to ensure it's dropped before await
            let wait_time = {
                let mut limiter = self.rate_limiter.lock()
                    .expect("Rate limiter mutex poisoned");
                if limiter.try_acquire(weight) {
                    return; // Successfully acquired, exit early
                }
                limiter.time_until_ready(weight)
            }; // Lock is dropped here

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: KuCoinEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Per-endpoint weights
        let weight = match endpoint {
            KuCoinEndpoint::SpotKlines | KuCoinEndpoint::FuturesKlines => weights::CANDLES,
            KuCoinEndpoint::SpotOrderbook | KuCoinEndpoint::FuturesOrderbook => weights::ORDERBOOK,
            KuCoinEndpoint::SpotAllTickers | KuCoinEndpoint::FuturesAllTickers => weights::ALL_TICKERS,
            _ => weights::DEFAULT,
        };
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url(account_type);
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
        let full_path = format!("{}{}", path, query);

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", &full_path, "")
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: KuCoinEndpoint,
        body: Value,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Per-endpoint weights for POST
        let weight = match endpoint {
            KuCoinEndpoint::SpotCreateOrder | KuCoinEndpoint::FuturesCreateOrder => weights::PLACE_ORDER,
            _ => weights::DEFAULT,
        };
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", path, &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        self.check_response(&response)?;
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: KuCoinEndpoint,
        path_params: &[(&str, &str)],
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Cancel order = amend weight
        let weight = match endpoint {
            KuCoinEndpoint::SpotCancelOrder | KuCoinEndpoint::FuturesCancelOrder => weights::AMEND_ORDER,
            _ => weights::DEFAULT,
        };
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url(account_type);
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("DELETE", &path, "");

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        self.check_response(&response)?;
        Ok(response)
    }

    /// Проверить response на ошибки
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("200000");

        if code != "200000" {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code.parse().unwrap_or(-1),
                message: msg.to_string(),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (KuCoin-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить все тикеры
    pub async fn get_all_tickers(&self, account_type: AccountType) -> ExchangeResult<Vec<Ticker>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotAllTickers,
            _ => KuCoinEndpoint::FuturesAllTickers,
        };

        let response = self.get(endpoint, HashMap::new(), account_type).await?;
        // TODO: parse all tickers
        let _ = response;
        Ok(vec![])
    }

    /// Получить информацию о символах
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Value> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotSymbols,
            _ => KuCoinEndpoint::FuturesContracts,
        };

        self.get(endpoint, HashMap::new(), account_type).await
    }

    /// Отменить все ордера
    pub async fn cancel_all_orders(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<String>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotCancelAllOrders,
            _ => KuCoinEndpoint::FuturesCancelAllOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        // DELETE with query params
        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);
        let full_path = format!("{}{}", path, query);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("DELETE", &full_path, "");

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;

        // Parse cancelled order IDs
        let data = KuCoinParser::extract_data(&response)?;
        let ids = data.get("cancelledOrderIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ids)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for KuCoinConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::KuCoin
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            (limiter.current_weight(), limiter.max_weight())
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
        self.testnet
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
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn interval_to_secs(interval: &str) -> u64 {
    match interval {
        "1m" => 60,
        "3m" => 180,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "2h" => 7200,
        "4h" => 14400,
        "6h" => 21600,
        "8h" => 28800,
        "12h" => 43200,
        "1d" | "1D" => 86400,
        "1w" | "1W" => 604800,
        _ => 60, // default to 1 minute
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for KuCoinConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotPrice,
            _ => KuCoinEndpoint::FuturesPrice,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        KuCoinParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotOrderbook,
            _ => KuCoinEndpoint::FuturesOrderbook,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        KuCoinParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotKlines,
            _ => KuCoinEndpoint::FuturesKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // Spot uses `type` parameter with string values like "1min", "1hour"
        // Futures uses `granularity` parameter with integer minutes like 1, 60
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                params.insert("type".to_string(), map_kline_interval(interval).to_string());
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                params.insert("granularity".to_string(), map_futures_granularity(interval).to_string());
            }
        }

        // KuCoin has no limit param — control batch size via startAt/endAt time window (max 1500 bars)
        if let Some(et) = end_time {
            let end_secs = et / 1000;
            params.insert("endAt".to_string(), end_secs.to_string());
            let count = limit.unwrap_or(1500).min(1500) as i64;
            let interval_secs = interval_to_secs(interval) as i64;
            let start_secs = end_secs - count * interval_secs;
            params.insert("startAt".to_string(), start_secs.to_string());
        } else {
            // First page — request a large window to get up to 1500 bars
            let count = limit.unwrap_or(1500).min(1500) as i64;
            let interval_secs = interval_to_secs(interval) as i64;
            let end_secs = chrono::Utc::now().timestamp();
            let start_secs = end_secs - count * interval_secs;
            params.insert("startAt".to_string(), start_secs.to_string());
            params.insert("endAt".to_string(), end_secs.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        KuCoinParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotTicker,
            _ => KuCoinEndpoint::FuturesTicker,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        KuCoinParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(KuCoinEndpoint::Timestamp, HashMap::new(), AccountType::Spot).await?;
        self.check_response(&response)
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols(account_type).await?;
        KuCoinParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for KuCoinConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let endpoint = match account_type {
                            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotCreateOrder,
                            _ => KuCoinEndpoint::FuturesCreateOrder,
                        };
                
                        let client_oid = format!("cc_{}", crate::core::timestamp_millis());
                
                        let body = json!({
                            "clientOid": client_oid,
                            "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                            "side": match side {
                                OrderSide::Buy => "buy",
                                OrderSide::Sell => "sell",
                            },
                            "type": "market",
                            "size": quantity.to_string(),
                        });
                
                        let response = self.post(endpoint, body, account_type).await?;
                        let order_id = KuCoinParser::parse_order_id(&response)?;
                
                        // Return minimal order info (can fetch full info with get_order)
                        Ok(PlaceOrderResponse::Simple(Order {
                            id: order_id,
                            client_order_id: Some(client_oid),
                            symbol: symbol.to_string(),
                            side,
                            order_type: OrderType::Market,
                            status: crate::core::OrderStatus::New,
                            price: None,
                            stop_price: None,
                            quantity,
                            filled_quantity: 0.0,
                            average_price: None,
                            commission: None,
                            commission_asset: None,
                            created_at: crate::core::timestamp_millis() as i64,
                            updated_at: None,
                            time_in_force: crate::core::TimeInForce::Gtc,
                        }))
            }
            OrderType::Limit { price } => {
                let endpoint = match account_type {
                            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotCreateOrder,
                            _ => KuCoinEndpoint::FuturesCreateOrder,
                        };
                
                        let client_oid = format!("cc_{}", crate::core::timestamp_millis());
                
                        let body = json!({
                            "clientOid": client_oid,
                            "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                            "side": match side {
                                OrderSide::Buy => "buy",
                                OrderSide::Sell => "sell",
                            },
                            "type": "limit",
                            "size": quantity,
                            "price": price,
                        });
                
                        let response = self.post(endpoint, body, account_type).await?;
                        let order_id = KuCoinParser::parse_order_id(&response)?;
                
                        Ok(PlaceOrderResponse::Simple(Order {
                            id: order_id,
                            client_order_id: Some(client_oid),
                            symbol: symbol.to_string(),
                            side,
                            order_type: OrderType::Limit { price: 0.0 },
                            status: crate::core::OrderStatus::New,
                            price: Some(price),
                            stop_price: None,
                            quantity,
                            filled_quantity: 0.0,
                            average_price: None,
                            commission: None,
                            commission_asset: None,
                            created_at: crate::core::timestamp_millis() as i64,
                            updated_at: None,
                            time_in_force: crate::core::TimeInForce::Gtc,
                        }))
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
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

            let endpoint = match account_type {
                AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotCancelOrder,
                _ => KuCoinEndpoint::FuturesCancelOrder,
            };

            let response = self.delete(endpoint, &[("orderId", order_id)], account_type).await?;
            self.check_response(&response)?;

            // Return cancelled order (minimal info)
            Ok(Order {
                id: order_id.to_string(),
                client_order_id: None,
                symbol: symbol.to_string(),
                side: OrderSide::Buy, // Unknown
                order_type: OrderType::Limit { price: 0.0 },
                status: crate::core::OrderStatus::Canceled,
                price: None,
                stop_price: None,
                quantity: 0.0,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: 0,
                updated_at: Some(crate::core::timestamp_millis() as i64),
                time_in_force: crate::core::TimeInForce::Gtc,
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
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotGetOrder,
            _ => KuCoinEndpoint::FuturesGetOrder,
        };

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path().replace("{orderId}", order_id);
        let url = format!("{}{}", base_url, path);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("GET", &path, "");

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;

        KuCoinParser::parse_order(&response, "")
    
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let symbol_str = symbol;
        let symbol: Option<crate::core::Symbol> = symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotOpenOrders,
            _ => KuCoinEndpoint::FuturesOpenOrders,
        };

        let mut params = HashMap::new();
        params.insert("status".to_string(), "active".to_string());
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(endpoint, params, account_type).await?;
        KuCoinParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for KuCoinConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let account_type = query.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotAccounts,
            _ => KuCoinEndpoint::FuturesAccount,
        };

        let mut params = HashMap::new();
        if let Some(a) = asset {
            params.insert("currency".to_string(), a.to_string());
        }
        match account_type {
            AccountType::Spot => params.insert("type".to_string(), "trade".to_string()),
            AccountType::Margin => params.insert("type".to_string(), "margin".to_string()),
            _ => None,
        };

        let response = self.get(endpoint, params, account_type).await?;

        match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinParser::parse_balances(&response),
            _ => KuCoinParser::parse_futures_account(&response),
        }
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1, // Default, should be fetched from API
            taker_commission: 0.1,
            balances,
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
impl Positions for KuCoinConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let response = if let Some(ref s) = symbol {
            let mut params = HashMap::new();
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
            self.get(KuCoinEndpoint::FuturesPosition, params, account_type).await?
        } else {
            self.get(KuCoinEndpoint::FuturesPositions, HashMap::new(), account_type).await?
        };

        if symbol.is_some() {
            KuCoinParser::parse_position(&response).map(|p| vec![p])
        } else {
            KuCoinParser::parse_positions(&response)
        }
    
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
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

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
        let base_url = self.urls.rest_url(account_type);
        let path = KuCoinEndpoint::FundingRate.path().replace("{symbol}", &formatted);
        let url = format!("{}{}", base_url, path);

        let response = self.http.get(&url, &HashMap::new()).await?;
        self.check_response(&response)?;

        KuCoinParser::parse_funding_rate(&response)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                "Leverage not supported for Spot/Margin".to_string()
                ));
                }
                _ => {}
                }

                let body = json!({
                "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                "level": leverage,
                });

                let response = self.post(KuCoinEndpoint::FuturesSetLeverage, body, account_type).await?;
                self.check_response(&response)?;

                Ok(())
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}
