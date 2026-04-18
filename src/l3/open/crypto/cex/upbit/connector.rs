//! # Upbit Connector
//!
//! Реализация всех core трейтов для Upbit.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//!
//! ## Note
//! Upbit only supports Spot trading (no Futures).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    CancelAllResponse,
    ExchangeIdentity, MarketData, Trading, Account,
    CancelAll, AmendOrder, CustodialFunds,
    AmendRequest,
    DepositAddress, WithdrawResponse, FundsRecord,
    UserTrade, UserTradeFilter,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};
use crate::core::types::{WithdrawRequest, FundsHistoryFilter, FundsRecordType};
use crate::core::types::SymbolInfo;
use crate::core::types::ConnectorStats;
use crate::core::utils::{RuntimeLimiter, RateLimitMonitor, RateLimitPressure};
use crate::core::types::{RateLimitCapabilities, LimitModel, RestLimitPool, WsLimits, EndpointWeight, OrderbookCapabilities};
use crate::core::utils::PrecisionCache;

use super::endpoints::{UpbitUrls, UpbitEndpoint, format_symbol, map_kline_interval};
use super::auth::{UpbitAuth, json_to_query_string};
use super::parser::UpbitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

static UPBIT_POOLS: &[RestLimitPool] = &[
    RestLimitPool {
        name: "market",
        max_budget: 10,
        window_seconds: 1,
        is_weight: false,
        has_server_headers: true,
        server_header: Some("Remaining-Req"),
        header_reports_used: false,
    },
    RestLimitPool {
        name: "account",
        max_budget: 30,
        window_seconds: 1,
        is_weight: false,
        has_server_headers: true,
        server_header: Some("Remaining-Req"),
        header_reports_used: false,
    },
    RestLimitPool {
        name: "order",
        max_budget: 8,
        window_seconds: 1,
        is_weight: false,
        has_server_headers: true,
        server_header: Some("Remaining-Req"),
        header_reports_used: false,
    },
];

static UPBIT_RATE_CAPS: RateLimitCapabilities = RateLimitCapabilities {
    model: LimitModel::Group,
    rest_pools: UPBIT_POOLS,
    decaying: None,
    endpoint_weights: &[] as &[EndpointWeight],
    ws: WsLimits {
        max_connections: None,
        max_subs_per_conn: None,
        max_msg_per_sec: Some(5),
        max_streams_per_conn: None,
    },
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Upbit коннектор
pub struct UpbitConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<UpbitAuth>,
    /// URL'ы (регион)
    urls: UpbitUrls,
    /// Runtime rate limiter (Group model: market 10/1s + account 30/1s + order 8/1s)
    limiter: Arc<Mutex<RuntimeLimiter>>,
    /// Pressure monitor
    monitor: Arc<Mutex<RateLimitMonitor>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl UpbitConnector {
    /// Создать новый коннектор
    /// region: "kr"/"korea" (Korea, KRW markets), "sg" (Singapore), "id" (Indonesia), "th" (Thailand)
    pub async fn new(credentials: Option<Credentials>, region: &str) -> ExchangeResult<Self> {
        let urls = match region {
            "kr" | "korea" => UpbitUrls::KOREA,
            "sg" | "singapore" => UpbitUrls::SINGAPORE,
            "id" => UpbitUrls::INDONESIA,
            "th" => UpbitUrls::THAILAND,
            _ => UpbitUrls::KOREA, // Default to Korea (KRW markets)
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(UpbitAuth::new)
            .transpose()?;

        let limiter = Arc::new(Mutex::new(RuntimeLimiter::from_caps(&UPBIT_RATE_CAPS)));
        let monitor = Arc::new(Mutex::new(RateLimitMonitor::new("Upbit")));

        Ok(Self {
            http,
            auth,
            urls,
            limiter,
            monitor,
            precision: PrecisionCache::new(),
        })
    }

    /// Создать коннектор только для публичных методов (Korea region, KRW markets)
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None, "kr").await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from Upbit response headers
    ///
    /// Upbit reports: Remaining-Req = "group=market; min=99; sec=9"
    /// Parse the group name and `sec=XX` remaining-per-second value,
    /// then compute used = group_max - remaining and call update_from_server.
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        let header_val = match headers
            .get("Remaining-Req")
            .and_then(|v| v.to_str().ok())
        {
            Some(s) => s.to_string(),
            None => return,
        };

        // Parse group name from "group=market; min=99; sec=9"
        let group_name = header_val
            .split(';')
            .find(|part| part.trim().starts_with("group="))
            .and_then(|part| part.trim().strip_prefix("group="))
            .map(|s| s.trim().to_string());

        // Parse sec=XX (remaining per second)
        let remaining_sec = header_val
            .split(';')
            .find(|part| part.trim().starts_with("sec="))
            .and_then(|part| part.trim().strip_prefix("sec="))
            .and_then(|v| v.trim().parse::<u32>().ok());

        if let (Some(group), Some(remaining)) = (group_name, remaining_sec) {
            // Determine group max from known limits
            let group_max = match group.as_str() {
                "market" => 10u32,
                "account" => 30u32,
                "order" => 8u32,
                _ => return,
            };
            let used = group_max.saturating_sub(remaining);
            if let Ok(mut limiter) = self.limiter.lock() {
                limiter.update_from_server(&group, used);
            }
        }
    }

    /// Wait for rate limit if needed, routing to the appropriate group.
    ///
    /// Non-essential requests (market data via "market" group) are dropped at >= 90% utilization.
    /// Returns `true` if acquired, `false` if dropped due to cutoff pressure.
    async fn rate_limit_wait(&self, group: &str, weight: u32, essential: bool) -> bool {
        loop {
            let wait_time = {
                let mut limiter = self.limiter.lock().expect("limiter poisoned");
                let pressure = self.monitor.lock().expect("monitor poisoned").check(&mut limiter);
                if pressure >= RateLimitPressure::Cutoff && !essential {
                    return false;
                }
                if limiter.try_acquire(group, weight) {
                    return true;
                }
                limiter.time_until_ready(group, weight)
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: UpbitEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Route to appropriate rate limit group; market data is non-essential
        let group = if endpoint.requires_auth() { "account" } else { "market" };
        let essential = endpoint.requires_auth();
        if !self.rate_limit_wait(group, 1, essential).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; market data request dropped".to_string(),
            });
        }

        let base_url = self.urls.rest_url(account_type);
        let mut path = endpoint.path().to_string();

        // For CandlesMinutes, add unit to path
        if endpoint == UpbitEndpoint::CandlesMinutes {
            if let Some(unit) = params.get("unit") {
                path = format!("{}/{}", path, unit);
            }
        }

        // Build query string
        let query_string = if params.is_empty() {
            String::new()
        } else {
            let pairs: Vec<_> = params.iter()
                .filter(|(k, _)| *k != "unit") // Skip unit param for CandlesMinutes
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish()
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", &path, Some(&query_string))?
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: UpbitEndpoint,
        body: Value,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Order operations are always essential
        self.rate_limit_wait("order", 1, true).await;

        let base_url = self.urls.rest;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        // Convert JSON body to query string for signing
        let body_str = body.to_string();
        let query_string = json_to_query_string(&body_str)?;
        let headers = auth.sign_request("POST", path, Some(&query_string))?;

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: UpbitEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Order operations are always essential
        self.rate_limit_wait("order", 1, true).await;

        let base_url = self.urls.rest;
        let path = endpoint.path();

        // Build query string
        let query_string = if params.is_empty() {
            String::new()
        } else {
            let pairs: Vec<_> = params.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish()
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("DELETE", path, Some(&query_string))?;

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Upbit-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить список всех торговых пар
    pub async fn get_trading_pairs(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(UpbitEndpoint::TradingPairs, HashMap::new(), AccountType::Spot).await?;

        if let Some(arr) = response.as_array() {
            Ok(arr.iter()
                .filter_map(|v| v.get("market").and_then(|m| m.as_str()).map(String::from))
                .collect())
        } else {
            Ok(vec![])
        }
    }

    /// Closed order history — `GET /v1/orders/closed` (signed)
    ///
    /// Returns filled and cancelled orders using cursor-based pagination.
    /// Optional params: `market`, `state` (`done`/`cancel`), `start_time`, `end_time`,
    /// `limit` (1–1000, default 100), `order_by` (`asc`/`desc`), `cursor`.
    pub async fn get_closed_orders(
        &self,
        market: Option<&str>,
        state: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(m) = market {
            params.insert("market".to_string(), m.to_string());
        }
        // state: "done" (filled) or "cancel" (cancelled). Default "done".
        params.insert(
            "state".to_string(),
            state.unwrap_or("done").to_string(),
        );
        if let Some(st) = start_time {
            params.insert("start_time".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("end_time".to_string(), et.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.clamp(1, 1000).to_string());
        }
        if let Some(c) = cursor {
            params.insert("cursor".to_string(), c.to_string());
        }
        self.get(UpbitEndpoint::ClosedOrders, params, AccountType::Spot).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for UpbitConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Upbit
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max, rate_groups) = if let Ok(mut limiter) = self.limiter.lock() {
            let (used, max) = limiter.primary_stats();
            let groups = limiter.group_stats();
            (used, max, groups)
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

    fn rate_limit_capabilities(&self) -> RateLimitCapabilities {
        UPBIT_RATE_CAPS
    }

    fn is_testnet(&self) -> bool {
        false
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[1, 5, 15, 30],
            ws_default_depth: Some(30),
            rest_max_depth: Some(30),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: false,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: &[],
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &[],
        }
    }
}

#[async_trait]
impl MarketData for UpbitConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("markets".to_string(), upbit_symbol);

        let response = self.get(UpbitEndpoint::Tickers, params, account_type).await?;
        UpbitParser::parse_price(&response)
    }

    async fn get_orderbook(&self, symbol: Symbol, depth: Option<u16>, account_type: AccountType) -> ExchangeResult<OrderBook> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("markets".to_string(), upbit_symbol);
        if let Some(n) = depth {
            let count = n.clamp(1, 30);
            params.insert("count".to_string(), count.to_string());
        }

        let response = self.get(UpbitEndpoint::Orderbook, params, account_type).await?;
        UpbitParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let (endpoint, unit) = map_kline_interval(interval);

        let mut params = HashMap::new();
        params.insert("market".to_string(), upbit_symbol);
        if let Some(u) = unit {
            params.insert("unit".to_string(), u.to_string());
        }
        if let Some(l) = limit {
            params.insert("count".to_string(), l.min(200).to_string());
        }
        if let Some(et) = end_time {
            if let Some(dt) = chrono::DateTime::from_timestamp_millis(et) {
                params.insert("to".to_string(), dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
            }
        }

        let response = self.get(endpoint, params, account_type).await?;
        UpbitParser::parse_klines(&response)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("markets".to_string(), upbit_symbol);

        let response = self.get(UpbitEndpoint::Tickers, params, account_type).await?;
        UpbitParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Upbit doesn't have a dedicated ping endpoint, so we'll just call the server time endpoint
        self.get(UpbitEndpoint::TradingPairs, HashMap::new(), AccountType::Spot).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // GET /v1/market/all returns all markets
        let response = self.get(UpbitEndpoint::TradingPairs, HashMap::new(), AccountType::Spot).await?;
        let info = UpbitParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&info);
        Ok(info)
    }

    fn market_data_capabilities(&self, _account_type: AccountType) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            // RecentTrades endpoint exists in the enum but no connector method calls it
            has_recent_trades: false,
            // map_kline_interval covers: 1m 3m 5m 10m 15m 30m 1h 4h 1d 1w 1M
            supported_intervals: &["1m", "3m", "5m", "10m", "15m", "30m", "1h", "4h", "1d", "1w", "1M"],
            // get_klines caps count at .min(200)
            max_kline_limit: Some(200),
            // WebSocket: kline subscriptions supported
            has_ws_klines: true,
            // WebSocket: trade channel supported
            has_ws_trades: true,
            // WebSocket: orderbook channel supported
            has_ws_orderbook: true,
            // WebSocket: ticker channel supported
            has_ws_ticker: true,
        }
    }
}

#[async_trait]
impl Trading for UpbitConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let upbit_symbol = if let Some(raw) = symbol.raw() {
                            raw.to_string()
                        } else {
                            format_symbol(&symbol.base, &symbol.quote, account_type)
                        };
                
                        // Upbit order types: "price" (market buy with total spend), "market" (market sell)
                        let (ord_type, side_str) = match side {
                            OrderSide::Buy => ("price", "bid"),
                            OrderSide::Sell => ("market", "ask"),
                        };
                
                        let mut body = json!({
                            "market": upbit_symbol,
                            "side": side_str,
                            "ord_type": ord_type,
                        });
                
                        // Market buy: quantity is total amount to spend
                        // Market sell: quantity is volume to sell
                        match side {
                            OrderSide::Buy => {
                                body["price"] = json!(self.precision.qty(&upbit_symbol, quantity));
                            },
                            OrderSide::Sell => {
                                body["volume"] = json!(self.precision.qty(&upbit_symbol, quantity));
                            },
                        }
                
                        let response = self.post(UpbitEndpoint::CreateOrder, body, account_type).await?;
                        UpbitParser::parse_order(&response, &upbit_symbol).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let upbit_symbol = if let Some(raw) = symbol.raw() {
                            raw.to_string()
                        } else {
                            format_symbol(&symbol.base, &symbol.quote, account_type)
                        };

                        let side_str = match side {
                            OrderSide::Buy => "bid",
                            OrderSide::Sell => "ask",
                        };

                        let body = json!({
                            "market": upbit_symbol,
                            "side": side_str,
                            "ord_type": "limit",
                            "volume": self.precision.qty(&upbit_symbol, quantity),
                            "price": self.precision.price(&upbit_symbol, price),
                        });

                        let response = self.post(UpbitEndpoint::CreateOrder, body, account_type).await?;
                        UpbitParser::parse_order(&response, &upbit_symbol).map(PlaceOrderResponse::Simple)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // GET /v1/orders with state=done (filled) or state=cancel
        let mut params = HashMap::new();

        // Default to "done" (filled orders) if no status specified
        params.insert("state".to_string(), "done".to_string());

        if let Some(ref sym) = filter.symbol {
            let upbit_symbol = if let Some(raw) = sym.raw() {
                raw.to_string()
            } else {
                format_symbol(&sym.base, &sym.quote, account_type)
            };
            params.insert("market".to_string(), upbit_symbol);
        }

        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(100).to_string());
        }

        let response = self.get(UpbitEndpoint::ListOrders, params, account_type).await?;
        UpbitParser::parse_orders(&response)
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?;
                let account_type = req.account_type;

                let upbit_symbol = if let Some(raw) = symbol.raw() {
                    raw.to_string()
                } else {
                    format_symbol(&symbol.base, &symbol.quote, account_type)
                };
                let mut params = HashMap::new();
                params.insert("uuid".to_string(), order_id.to_string());

                let response = self.delete(UpbitEndpoint::CancelOrder, params, account_type).await?;
                UpbitParser::parse_order(&response, &upbit_symbol)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(&self, symbol: &str, order_id: &str, account_type: AccountType) -> ExchangeResult<Order> {
        let parts: Vec<&str> = symbol.split('/').collect();
        let sym = if parts.len() == 2 {
            crate::core::Symbol::new(parts[0], parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };
        let upbit_symbol = if let Some(raw) = sym.raw() {
            raw.to_string()
        } else {
            format_symbol(&sym.base, &sym.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("uuid".to_string(), order_id.to_string());

        let response = self.get(UpbitEndpoint::GetOrder, params, account_type).await?;
        UpbitParser::parse_order(&response, &upbit_symbol)
    }

    async fn get_open_orders(&self, symbol: Option<&str>, account_type: AccountType) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("state".to_string(), "wait".to_string());

        if let Some(s) = symbol {
            let parts: Vec<&str> = s.split('/').collect();
            let sym = if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            };
            let upbit_symbol = if let Some(raw) = sym.raw() {
                raw.to_string()
            } else {
                format_symbol(&sym.base, &sym.quote, account_type)
            };
            params.insert("market".to_string(), upbit_symbol);
        }

        let response = self.get(UpbitEndpoint::ListOrders, params, account_type).await?;
        UpbitParser::parse_orders(&response)
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        // Upbit has no bulk fills/trades endpoint.
        // The only way to retrieve fills is via a single order's detail response,
        // which embeds a `trades` array.
        let order_id = filter.order_id.as_deref().ok_or_else(|| {
            ExchangeError::UnsupportedOperation(
                "Upbit requires order_id for get_user_trades (no bulk fills endpoint)".to_string(),
            )
        })?;

        let mut params = HashMap::new();
        params.insert("uuid".to_string(), order_id.to_string());

        let response = self.get(UpbitEndpoint::GetOrder, params, account_type).await?;
        UpbitParser::parse_order_trades(&response)
    }

    fn trading_capabilities(&self, _account_type: AccountType) -> TradingCapabilities {
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            // Upbit spot supports only market and limit order types
            has_stop_market: false,
            has_stop_limit: false,
            has_trailing_stop: false,
            has_bracket: false,
            has_oco: false,
            // AmendOrder trait implemented via cancel-and-new (POST /v1/orders/cancel_and_new)
            has_amend: true,
            // No batch order placement; BatchCancelOrders is for cancel-all only
            has_batch: false,
            max_batch_size: None,
            // CancelAll trait implemented via DELETE /v1/orders
            has_cancel_all: true,
            // get_user_trades implemented, but requires order_id (no bulk fills endpoint)
            has_user_trades: true,
            has_order_history: true,
        }
    }
}

#[async_trait]
impl Account for UpbitConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let account_type = query.account_type;
        let response = self.get(UpbitEndpoint::Balances, HashMap::new(), account_type).await?;
        let balances = UpbitParser::parse_balances(&response)?;

        // Filter by asset if provided
        if let Some(asset_name) = asset {
            Ok(balances.into_iter()
                .filter(|b| b.asset.eq_ignore_ascii_case(&asset_name))
                .collect())
        } else {
            Ok(balances)
        }
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            balances,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.05, // Upbit default maker commission 0.05%
            taker_commission: 0.05, // Upbit default taker commission 0.05%
        })
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Upbit does not expose a fee endpoint via API
        Err(ExchangeError::UnsupportedOperation(
            "Upbit does not provide a fee query API endpoint".to_string()
        ))
    }

    fn account_capabilities(&self, _account_type: AccountType) -> AccountCapabilities {
        AccountCapabilities {
            has_balances: true,
            has_account_info: true,
            // Upbit has no fee query API endpoint — get_fees returns UnsupportedOperation
            has_fees: false,
            // No AccountTransfers trait impl (no spot↔futures — spot-only exchange)
            has_transfers: false,
            has_sub_accounts: false,
            // CustodialFunds trait implemented: deposit address + withdraw
            has_deposit_withdraw: true,
            // Spot-only exchange — no margin, earn, staking, or perp funding
            has_margin: false,
            has_earn_staking: false,
            has_funding_history: false,
            // No AccountLedger trait impl
            has_ledger: false,
            has_convert: false,
            // Spot-only exchange — no positions
            has_positions: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for UpbitConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        // DELETE /v1/orders — batch cancel by market or side
        let mut params = HashMap::new();

        match &scope {
            CancelScope::All { symbol } => {
                if let Some(sym) = symbol {
                    let upbit_symbol = if let Some(raw) = sym.raw() {
                        raw.to_string()
                    } else {
                        format_symbol(&sym.base, &sym.quote, account_type)
                    };
                    params.insert("market".to_string(), upbit_symbol);
                }
            }
            CancelScope::BySymbol { symbol } => {
                let upbit_symbol = if let Some(raw) = symbol.raw() {
                    raw.to_string()
                } else {
                    format_symbol(&symbol.base, &symbol.quote, account_type)
                };
                params.insert("market".to_string(), upbit_symbol);
            }
            _ => return Err(ExchangeError::InvalidRequest(
                "cancel_all_orders requires CancelScope::All or BySymbol".to_string()
            )),
        }

        let _response = self.delete(UpbitEndpoint::BatchCancelOrders, params, account_type).await?;

        Ok(CancelAllResponse {
            cancelled_count: 0, // Upbit doesn't return count
            failed_count: 0,
            details: vec![],
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for UpbitConnector {
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        // First try to get an existing address from GET /v1/deposits/coin_addresses
        let mut params = HashMap::new();
        params.insert("currency".to_string(), asset.to_uppercase());
        if let Some(net) = network {
            params.insert("net_type".to_string(), net.to_string());
        }

        let existing = self.get(UpbitEndpoint::ListDepositAddresses, params.clone(), AccountType::Spot).await;

        // If we get a valid address back, return it
        if let Ok(ref response) = existing {
            // Response may be a single object or array
            let addr_obj = if let Some(arr) = response.as_array() {
                arr.first().cloned()
            } else if response.is_object() {
                Some(response.clone())
            } else {
                None
            };

            if let Some(obj) = addr_obj {
                if let Some(addr) = obj.get("deposit_address").and_then(|v| v.as_str()) {
                    if !addr.is_empty() {
                        return Ok(DepositAddress {
                            address: addr.to_string(),
                            tag: obj.get("secondary_address")
                                .and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                            network: network.map(|n| n.to_string()),
                            asset: asset.to_uppercase(),
                            created_at: None,
                        });
                    }
                }
            }
        }

        // If no existing address found, generate one via POST /v1/deposits/generate_coin_address
        let body = if let Some(net) = network {
            serde_json::json!({
                "currency": asset.to_uppercase(),
                "net_type": net,
            })
        } else {
            serde_json::json!({
                "currency": asset.to_uppercase(),
            })
        };

        let response = self.post(UpbitEndpoint::CreateDepositAddress, body, AccountType::Spot).await?;

        // Upbit may return 202 (generating) or 200 (ready)
        let address = response.get("deposit_address")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if address.is_empty() {
            return Err(ExchangeError::InvalidRequest(
                "Deposit address is being generated — retry in a few seconds".to_string()
            ));
        }

        Ok(DepositAddress {
            address: address.to_string(),
            tag: response.get("secondary_address")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            network: network.map(|n| n.to_string()),
            asset: asset.to_uppercase(),
            created_at: None,
        })
    }

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        // POST /v1/withdraws/coin
        let mut body = serde_json::json!({
            "currency": req.asset.to_uppercase(),
            "amount": req.amount.to_string(),
            "address": req.address,
        });

        if let Some(ref net) = req.network {
            body["net_type"] = serde_json::json!(net);
        }

        // secondary_address = destination tag / memo for assets like XRP
        if let Some(ref tag) = req.tag {
            body["secondary_address"] = serde_json::json!(tag);
        }

        let response = self.post(UpbitEndpoint::InitiateWithdrawal, body, AccountType::Spot).await?;

        let withdraw_id = response.get("uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing uuid in withdrawal response".to_string()))?
            .to_string();

        let status = response.get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("submitted")
            .to_string();

        let tx_hash = response.get("txid")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status,
            tx_hash,
        })
    }

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let mut records = Vec::new();

        let fetch_deposits = matches!(
            filter.record_type,
            FundsRecordType::Deposit | FundsRecordType::Both
        );
        let fetch_withdrawals = matches!(
            filter.record_type,
            FundsRecordType::Withdrawal | FundsRecordType::Both
        );

        // Fetch deposit records
        if fetch_deposits {
            let mut params: HashMap<String, String> = HashMap::new();
            if let Some(ref asset) = filter.asset {
                params.insert("currency".to_string(), asset.to_uppercase());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.min(100u32).to_string());
            }
            params.insert("order_by".to_string(), "desc".to_string());

            let response = self.get(UpbitEndpoint::ListDeposits, params, AccountType::Spot).await?;

            if let Some(arr) = response.as_array() {
                for item in arr {
                    let id = item.get("uuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let asset_name = item.get("currency").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let amount = item.get("amount").and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| item.get("amount").and_then(|v| v.as_f64()))
                        .unwrap_or(0.0);
                    let tx_hash = item.get("txid").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let status = item.get("state").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let timestamp = item.get("created_at").and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.timestamp_millis())
                        .unwrap_or(0);
                    let network = item.get("net_type").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    records.push(FundsRecord::Deposit {
                        id,
                        asset: asset_name,
                        amount,
                        tx_hash,
                        network,
                        status,
                        timestamp,
                    });
                }
            }
        }

        // Fetch withdrawal records
        if fetch_withdrawals {
            let mut params: HashMap<String, String> = HashMap::new();
            if let Some(ref asset) = filter.asset {
                params.insert("currency".to_string(), asset.to_uppercase());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.min(100u32).to_string());
            }
            params.insert("order_by".to_string(), "desc".to_string());

            let response = self.get(UpbitEndpoint::ListWithdrawals, params, AccountType::Spot).await?;

            if let Some(arr) = response.as_array() {
                for item in arr {
                    let id = item.get("uuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let asset_name = item.get("currency").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let amount = item.get("amount").and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| item.get("amount").and_then(|v| v.as_f64()))
                        .unwrap_or(0.0);
                    let fee = item.get("fee").and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| item.get("fee").and_then(|v| v.as_f64()));
                    let address = item.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let tag = item.get("secondary_address").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let tx_hash = item.get("txid").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let status = item.get("state").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let timestamp = item.get("created_at").and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.timestamp_millis())
                        .unwrap_or(0);
                    let network = item.get("net_type").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    records.push(FundsRecord::Withdrawal {
                        id,
                        asset: asset_name,
                        amount,
                        fee,
                        address,
                        tag,
                        tx_hash,
                        network,
                        status,
                        timestamp,
                    });
                }
            }
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for UpbitConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        // Upbit implements amend as an atomic cancel-and-replace via
        // POST /v1/orders/cancel_and_new
        let symbol = &req.symbol;
        let account_type = AccountType::Spot; // Upbit is Spot-only

        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };

        let mut body = serde_json::json!({
            "cancel_uuid": req.order_id,
            "market": upbit_symbol,
        });

        // At least one of price or quantity must be provided.
        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "AmendOrder requires at least one of: price, quantity".to_string(),
            ));
        }

        if let Some(new_price) = req.fields.price {
            body["price"] = serde_json::json!(self.precision.price(&upbit_symbol, new_price));
        }
        if let Some(new_qty) = req.fields.quantity {
            body["volume"] = serde_json::json!(self.precision.qty(&upbit_symbol, new_qty));
        }

        // Upbit cancel_and_new requires ord_type; default to "limit" since amend
        // is only meaningful for resting limit orders.
        if body.get("price").is_some() {
            body["ord_type"] = serde_json::json!("limit");
        }

        let response = self.post(UpbitEndpoint::ReplaceOrder, body, account_type).await?;
        // Response contains the newly created order under the "new_order" key.
        let new_order = response.get("new_order").unwrap_or(&response);
        UpbitParser::parse_order(new_order, &upbit_symbol)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// C3 ADDITIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl UpbitConnector {
    /// Get order chance (market restrictions and trading fees) for a market.
    ///
    /// `GET /v1/orders/chance`
    /// Required parameter: `market` (e.g. `"KRW-BTC"`).
    pub async fn get_order_chance(&self, market: &str) -> ExchangeResult<Value> {
        let mut params = std::collections::HashMap::new();
        params.insert("market".to_string(), market.to_string());
        self.get(UpbitEndpoint::OrderChance, params, AccountType::Spot).await
    }

    /// List open (unfilled) orders with pagination, optionally filtered by market.
    ///
    /// `GET /v1/orders/open`
    /// This is the v2 paginated endpoint, distinct from the trait's `get_open_orders`.
    /// Optional parameters: `market`, `page`, `limit`, `order_by`.
    pub async fn list_open_orders_paginated(
        &self,
        market: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = std::collections::HashMap::new();
        if let Some(m) = market {
            params.insert("market".to_string(), m.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(UpbitEndpoint::OpenOrders, params, AccountType::Spot).await
    }

    /// Get wallet status for all assets or a specific currency.
    ///
    /// `GET /v1/status/wallet`
    /// Optional parameter: `currency` (e.g. `"BTC"`).
    pub async fn get_wallet_status(&self, currency: Option<&str>) -> ExchangeResult<Value> {
        let mut params = std::collections::HashMap::new();
        if let Some(c) = currency {
            params.insert("currency".to_string(), c.to_string());
        }
        self.get(UpbitEndpoint::WalletStatus, params, AccountType::Spot).await
    }

    /// Withdraw Korean Won (KRW) to a bank account.
    ///
    /// `POST /v1/withdraws/krw`
    /// Required parameters: `amount`, `two_factor_type`.
    pub async fn withdraw_krw(
        &self,
        amount: f64,
        two_factor_type: &str,
    ) -> ExchangeResult<Value> {
        let body = json!({
            "amount": amount.to_string(),
            "two_factor_type": two_factor_type,
        });
        self.post(UpbitEndpoint::WithdrawKrw, body, AccountType::Spot).await
    }

    /// Cancel a pending withdrawal by UUID.
    ///
    /// `DELETE /v1/withdraws/uuid`
    /// Required parameter: `uuid`.
    pub async fn cancel_withdraw(&self, uuid: &str) -> ExchangeResult<Value> {
        let mut params = std::collections::HashMap::new();
        params.insert("uuid".to_string(), uuid.to_string());
        self.delete(UpbitEndpoint::CancelWithdraw, params, AccountType::Spot).await
    }
}
