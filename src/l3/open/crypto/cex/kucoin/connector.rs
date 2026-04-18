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
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    AmendRequest, CancelAllResponse, OrderResult,
    UserTrade, UserTradeFilter,
    TransferResponse, DepositAddress, WithdrawResponse, FundsRecord,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,
    FundingHistory, AccountLedger,
};
use crate::core::types::{
    TransferRequest, TransferHistoryFilter, WithdrawRequest,
    FundsHistoryFilter, FundsRecordType, SubAccountOperation, SubAccountResult,
    SubAccount, ConnectorStats,
    FundingPayment, FundingFilter, LedgerEntry, LedgerFilter,
};
use crate::core::utils::{RuntimeLimiter, RateLimitMonitor, RateLimitPressure};
use crate::core::types::{RateLimitCapabilities, LimitModel, RestLimitPool, WsLimits, OrderbookCapabilities, WsBookChannel};

use super::endpoints::{KuCoinUrls, KuCoinEndpoint, format_symbol, map_kline_interval, map_futures_granularity};
use super::auth::KuCoinAuth;
use super::parser::KuCoinParser;

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CAPABILITIES (static — embedded in binary, no allocation)
// ═══════════════════════════════════════════════════════════════════════════════

static KUCOIN_POOLS: &[RestLimitPool] = &[RestLimitPool {
    name: "default",
    max_budget: 4000,
    window_seconds: 30,
    is_weight: true,
    has_server_headers: true,
    server_header: Some("X-RateLimit-Used"),
    header_reports_used: true,
}];

static KUCOIN_RATE_CAPS: RateLimitCapabilities = RateLimitCapabilities {
    model: LimitModel::Weight,
    rest_pools: KUCOIN_POOLS,
    decaying: None,
    endpoint_weights: &[],
    ws: WsLimits {
        max_connections: Some(800),
        max_subs_per_conn: Some(300),
        max_msg_per_sec: Some(10),
        max_streams_per_conn: None,
    },
};

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
    /// Runtime rate limiter (Weight model: 4000 weight per 30 seconds)
    limiter: Arc<Mutex<RuntimeLimiter>>,
    /// Pressure monitor — gates non-essential requests at >= 90%
    monitor: Arc<Mutex<RateLimitMonitor>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: crate::core::utils::precision::PrecisionCache,
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

        let limiter = Arc::new(Mutex::new(RuntimeLimiter::from_caps(&KUCOIN_RATE_CAPS)));
        let monitor = Arc::new(Mutex::new(RateLimitMonitor::new("KuCoin")));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            limiter,
            monitor,
            precision: crate::core::utils::precision::PrecisionCache::new(),
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Sync limiter from KuCoin response headers.
    ///
    /// KuCoin reports: X-RateLimit-Used = used weight.
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        if let Some(used) = headers
            .get("X-RateLimit-Used")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            if let Ok(mut limiter) = self.limiter.lock() {
                limiter.update_from_server("default", used);
            }
        }
    }

    /// Wait for rate limit budget. Non-essential requests are dropped at >= 90% utilization.
    ///
    /// Returns `true` if acquired, `false` if dropped due to cutoff pressure.
    /// Trading endpoints should pass `essential: true` to always wait through.
    async fn rate_limit_wait(&self, weight: u32, essential: bool) -> bool {
        loop {
            let wait_time = {
                let mut limiter = self.limiter.lock()
                    .expect("rate limiter mutex poisoned");

                let pressure = self.monitor.lock()
                    .expect("rate monitor mutex poisoned")
                    .check(&mut limiter);
                if pressure >= RateLimitPressure::Cutoff && !essential {
                    return false;
                }

                if limiter.try_acquire("default", weight) {
                    return true;
                }
                limiter.time_until_ready("default", weight)
            };
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
        // Market data = non-essential: drop at >= 90% utilization to preserve budget for trading
        if !self.rate_limit_wait(weight, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }

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
        // Order placement = essential: always wait, never drop
        self.rate_limit_wait(weight, true).await;

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
        // Order management = essential: always wait, never drop
        self.rate_limit_wait(weight, true).await;

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

        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))?;

        // Spot: data = { ticker: [...], time: 123 }
        // Futures: data = [ {...}, {...} ] (direct array)
        let (arr, timestamp) = if let Some(ticker_arr) = data.get("ticker").and_then(|v| v.as_array()) {
            let ts = data.get("time").and_then(|t| t.as_i64()).unwrap_or(0);
            (ticker_arr, ts)
        } else if let Some(direct_arr) = data.as_array() {
            (direct_arr, 0i64)
        } else {
            return Err(ExchangeError::Parse("Unexpected all-tickers data format".to_string()));
        };

        let tickers = arr.iter().map(|item| {
            let get_f64 = |key: &str| -> Option<f64> {
                item.get(key)
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64()))
            };
            let get_i64 = |key: &str| -> Option<i64> {
                item.get(key).and_then(|v| v.as_i64())
            };

            let ts = get_i64("time").unwrap_or(timestamp);
            let last = get_f64("last").or_else(|| get_f64("price")).unwrap_or(0.0);
            let change_rate = get_f64("changeRate").map(|r| r * 100.0);

            Ticker {
                symbol: item.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                last_price: last,
                bid_price: get_f64("buy").or_else(|| get_f64("bestBidPrice")),
                ask_price: get_f64("sell").or_else(|| get_f64("bestAskPrice")),
                high_24h: get_f64("high"),
                low_24h: get_f64("low"),
                volume_24h: get_f64("vol"),
                quote_volume_24h: get_f64("volValue"),
                price_change_24h: get_f64("changePrice"),
                price_change_percent_24h: change_rate,
                timestamp: ts,
            }
        }).collect();

        Ok(tickers)
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

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA EXTENSIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get recent spot trades for a symbol.
    pub async fn get_spot_recent_trades(
        &self,
        symbol: &str,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        self.get(KuCoinEndpoint::SpotRecentTrades, params, AccountType::Spot).await
    }

    /// Get full order book (level 2) for a symbol.
    pub async fn get_full_orderbook(&self, symbol: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        self.get(KuCoinEndpoint::FullOrderbook, params, AccountType::Spot).await
    }

    /// Get futures recent trade history for a symbol.
    pub async fn get_futures_trade_history(
        &self,
        symbol: &str,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        self.get(KuCoinEndpoint::FuturesTradeHistory, params, AccountType::FuturesCross).await
    }

    /// Get futures funding rate history for a contract.
    pub async fn get_futures_funding_rates(
        &self,
        symbol: &str,
        from: Option<i64>,
        to: Option<i64>,
        offset: Option<i64>,
        max_count: Option<u32>,
        reverse: Option<bool>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(f) = from {
            params.insert("from".to_string(), f.to_string());
        }
        if let Some(t) = to {
            params.insert("to".to_string(), t.to_string());
        }
        if let Some(o) = offset {
            params.insert("offset".to_string(), o.to_string());
        }
        if let Some(m) = max_count {
            params.insert("maxCount".to_string(), m.to_string());
        }
        if let Some(r) = reverse {
            params.insert("reverse".to_string(), r.to_string());
        }
        self.get(KuCoinEndpoint::FuturesFundingRates, params, AccountType::FuturesCross).await
    }

    /// Get current mark price for a futures symbol.
    pub async fn get_futures_mark_price(&self, symbol: &str) -> ExchangeResult<Value> {
        self.rate_limit_wait(weights::DEFAULT, false).await;
        let base_url = self.urls.rest_url(AccountType::FuturesCross);
        let path = format!("/api/v1/mark-price/{}/current", symbol);
        let url = format!("{}{}", base_url, path);
        let (response, _) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Get current index price for a futures symbol.
    pub async fn get_futures_index_price(&self, symbol: &str) -> ExchangeResult<Value> {
        self.rate_limit_wait(weights::DEFAULT, false).await;
        let base_url = self.urls.rest_url(AccountType::FuturesCross);
        let path = format!("/api/v1/index-price/{}/current", symbol);
        let url = format!("{}{}", base_url, path);
        let (response, _) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Get current premium index for a futures symbol.
    pub async fn get_futures_premium_index(&self, symbol: &str) -> ExchangeResult<Value> {
        self.rate_limit_wait(weights::DEFAULT, false).await;
        let base_url = self.urls.rest_url(AccountType::FuturesCross);
        let path = format!("/api/v1/premium-index/{}/current", symbol);
        let url = format!("{}{}", base_url, path);
        let (response, _) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FILL / TRADE HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get spot trade fills (paginated, signed).
    ///
    /// `order_id`: filter by order ID.
    /// `trade_type`: `"TRADE"` (default) or `"MARGIN_TRADE"`.
    pub async fn get_spot_fills(
        &self,
        symbol: Option<&str>,
        order_id: Option<&str>,
        start_at: Option<i64>,
        end_at: Option<i64>,
        page_size: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }
        if let Some(oid) = order_id {
            params.insert("orderId".to_string(), oid.to_string());
        }
        if let Some(st) = start_at {
            params.insert("startAt".to_string(), st.to_string());
        }
        if let Some(et) = end_at {
            params.insert("endAt".to_string(), et.to_string());
        }
        if let Some(ps) = page_size {
            params.insert("pageSize".to_string(), ps.to_string());
        }
        self.get(KuCoinEndpoint::SpotFills, params, AccountType::Spot).await
    }

    /// Get futures trade fills (paginated, signed).
    pub async fn get_futures_fills(
        &self,
        symbol: Option<&str>,
        order_id: Option<&str>,
        start_at: Option<i64>,
        end_at: Option<i64>,
        page_size: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }
        if let Some(oid) = order_id {
            params.insert("orderId".to_string(), oid.to_string());
        }
        if let Some(st) = start_at {
            params.insert("startAt".to_string(), st.to_string());
        }
        if let Some(et) = end_at {
            params.insert("endAt".to_string(), et.to_string());
        }
        if let Some(ps) = page_size {
            params.insert("pageSize".to_string(), ps.to_string());
        }
        self.get(KuCoinEndpoint::FuturesFills, params, AccountType::FuturesCross).await
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
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.limiter.lock() {
            limiter.primary_stats()
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

    fn rate_limit_capabilities(&self) -> RateLimitCapabilities {
        KUCOIN_RATE_CAPS
    }

    fn orderbook_capabilities(&self, account_type: AccountType) -> OrderbookCapabilities {
        static SPOT_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("spotMarket/level2Depth5",  5,  100),
            WsBookChannel::snapshot("spotMarket/level2Depth50", 50, 100),
            WsBookChannel::delta("market/level2",               None, None),
        ];
        static FUTURES_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("contractMarket/level2Depth5",  5,  100),
            WsBookChannel::snapshot("contractMarket/level2Depth50", 50, 100),
            WsBookChannel::delta("contractMarket/level2",           None, None),
        ];
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => OrderbookCapabilities {
                ws_depths: &[5, 50],
                ws_default_depth: Some(50),
                rest_max_depth: Some(100),
                rest_depth_values: &[20, 100],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[],
                default_speed_ms: None,
                ws_channels: FUTURES_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
            _ => OrderbookCapabilities {
                ws_depths: &[5, 50],
                ws_default_depth: Some(50),
                rest_max_depth: None,
                rest_depth_values: &[20, 100],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[],
                default_speed_ms: None,
                ws_channels: SPOT_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
        }
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
            _ => {
                params.insert("type".to_string(), map_kline_interval(interval).to_string());
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
        let symbols = KuCoinParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&symbols);
        Ok(symbols)
    }

    fn market_data_capabilities(&self, account_type: AccountType) -> MarketDataCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            // get_spot_recent_trades exists as an inherent method but is NOT part of the
            // MarketData trait — the trait method is not overridden, so false here.
            has_recent_trades: false,
            has_ws_klines: true,
            has_ws_trades: true,
            has_ws_orderbook: true,
            has_ws_ticker: true,
            // Spot: 1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 1w 1M
            // Futures: drops 3m, 6h, 1M (granularity is integer minutes)
            supported_intervals: if is_futures {
                &["1m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "1w"]
            } else {
                &["1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "1w", "1M"]
            },
            // KuCoin uses time-window, not a numeric limit param; effective max is 1500 bars.
            max_kline_limit: Some(1500),
        }
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

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotCreateOrder,
            _ => KuCoinEndpoint::FuturesCreateOrder,
        };
        let client_oid = req.client_order_id.clone()
            .unwrap_or_else(|| format!("cc_{}", crate::core::timestamp_millis()));
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
        let side_str = match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };

        let (body, order_type_out, price_out, stop_price_out, tif_out) = match req.order_type {
            OrderType::Market => {
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "market",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                });
                (body, OrderType::Market, None, None, crate::core::TimeInForce::Gtc)
            }
            OrderType::Limit { price } => {
                let tif = match req.time_in_force {
                    crate::core::TimeInForce::PostOnly => "limit", // KuCoin uses postOnly flag
                    crate::core::TimeInForce::Ioc => "IOC",
                    crate::core::TimeInForce::Fok => "FOK",
                    _ => "GTC",
                };
                let post_only = matches!(req.time_in_force, crate::core::TimeInForce::PostOnly);
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "limit",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "price": self.precision.price(&formatted_symbol, price),
                    "timeInForce": tif,
                    "postOnly": post_only,
                });
                (body, OrderType::Limit { price }, Some(price), None, crate::core::TimeInForce::Gtc)
            }
            OrderType::PostOnly { price } => {
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "limit",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "price": self.precision.price(&formatted_symbol, price),
                    "postOnly": true,
                });
                (body, OrderType::PostOnly { price }, Some(price), None, crate::core::TimeInForce::Gtc)
            }
            OrderType::Ioc { price } => {
                let px = price.unwrap_or(0.0);
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "limit",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "price": self.precision.price(&formatted_symbol, px),
                    "timeInForce": "IOC",
                });
                (body, OrderType::Ioc { price }, price, None, crate::core::TimeInForce::Ioc)
            }
            OrderType::Fok { price } => {
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "limit",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "price": self.precision.price(&formatted_symbol, price),
                    "timeInForce": "FOK",
                });
                (body, OrderType::Fok { price }, Some(price), None, crate::core::TimeInForce::Fok)
            }
            OrderType::StopMarket { stop_price } => {
                // KuCoin stop orders: use stopPrice + stop=up/down
                let stop_dir = match side {
                    OrderSide::Buy => "up",
                    OrderSide::Sell => "down",
                };
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "market",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "stop": stop_dir,
                    "stopPrice": self.precision.price(&formatted_symbol, stop_price),
                    "stopPriceType": "TP",  // trade price
                });
                (body, OrderType::StopMarket { stop_price }, None, Some(stop_price), crate::core::TimeInForce::Gtc)
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                let stop_dir = match side {
                    OrderSide::Buy => "up",
                    OrderSide::Sell => "down",
                };
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "limit",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "price": self.precision.price(&formatted_symbol, limit_price),
                    "stop": stop_dir,
                    "stopPrice": self.precision.price(&formatted_symbol, stop_price),
                    "stopPriceType": "TP",
                });
                (body, OrderType::StopLimit { stop_price, limit_price }, Some(limit_price), Some(stop_price), crate::core::TimeInForce::Gtc)
            }
            OrderType::ReduceOnly { price } => {
                // KuCoin Futures supports reduceOnly flag
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ReduceOnly orders only supported for futures on KuCoin".to_string()
                        ));
                    }
                    _ => {}
                }
                let ord_type = if price.is_some() { "limit" } else { "market" };
                let mut body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": ord_type,
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "reduceOnly": true,
                });
                if let Some(px) = price {
                    body["price"] = json!(self.precision.price(&formatted_symbol, px));
                }
                (body, OrderType::ReduceOnly { price }, price, None, crate::core::TimeInForce::Gtc)
            }
            OrderType::Gtd { price, expire_time } => {
                // KuCoin does not support GTD natively — place as GTC limit order
                let _ = expire_time;
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "limit",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "price": self.precision.price(&formatted_symbol, price),
                    "timeInForce": "GTC",
                });
                (body, OrderType::Gtd { price, expire_time }, Some(price), None, crate::core::TimeInForce::Gtc)
            }
            OrderType::Oco { price, stop_price, stop_limit_price } => {
                // KuCoin native OCO endpoint: POST /api/v3/oco/order (spot only)
                match account_type {
                    AccountType::Spot | AccountType::Margin => {}
                    _ => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "OCO orders are only supported for Spot on KuCoin".to_string()
                        ));
                    }
                }
                let limit_price = stop_limit_price.unwrap_or(stop_price);
                let oco_body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "price": self.precision.price(&formatted_symbol, price),
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "stopPrice": self.precision.price(&formatted_symbol, stop_price),
                    "limitPrice": self.precision.price(&formatted_symbol, limit_price),
                    "tradeType": "TRADE",
                });
                let base_url = self.urls.rest_url(account_type);
                let path = KuCoinEndpoint::SpotOcoOrder.path();
                let url = format!("{}{}", base_url, path);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = oco_body.to_string();
                let headers = auth.sign_request("POST", path, &body_str);
                let (response, resp_headers) = self.http.post_with_response_headers(&url, &oco_body, &headers).await?;
                self.update_rate_from_headers(&resp_headers);
                self.check_response(&response)?;
                let order_id = KuCoinParser::parse_order_id(&response)?;
                return Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_oid),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Oco { price, stop_price, stop_limit_price },
                    status: crate::core::OrderStatus::New,
                    price: Some(price),
                    stop_price: Some(stop_price),
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }));
            }
            OrderType::Iceberg { price, display_quantity } => {
                // KuCoin Spot: iceberg flag on HF order (same endpoint, extra fields)
                let tif = match req.time_in_force {
                    crate::core::TimeInForce::Ioc => "IOC",
                    crate::core::TimeInForce::Fok => "FOK",
                    _ => "GTC",
                };
                let iceberg_body = json!({
                    "clientOid": client_oid,
                    "symbol": formatted_symbol,
                    "side": side_str,
                    "type": "limit",
                    "size": self.precision.qty(&formatted_symbol, quantity),
                    "price": self.precision.price(&formatted_symbol, price),
                    "timeInForce": tif,
                    "iceberg": true,
                    "visibleSize": self.precision.qty(&formatted_symbol, display_quantity),
                });
                // Use the standard create order endpoint — KuCoin HF supports iceberg flag
                let response = self.post(endpoint, iceberg_body, account_type).await?;
                let order_id = KuCoinParser::parse_order_id(&response)?;
                return Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_oid),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Iceberg { price, display_quantity },
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
                }));
            }
            OrderType::TrailingStop { .. } | OrderType::Bracket { .. } | OrderType::Twap { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
                ));
            }
            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Order type not supported on KuCoin".to_string()
                ));
            }
        };

        let response = self.post(endpoint, body, account_type).await?;
        let order_id = KuCoinParser::parse_order_id(&response)?;

        Ok(PlaceOrderResponse::Simple(Order {
            id: order_id,
            client_order_id: Some(client_oid),
            symbol: symbol.to_string(),
            side,
            order_type: order_type_out,
            status: crate::core::OrderStatus::New,
            price: price_out,
            stop_price: stop_price_out,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: tif_out,
        }))
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotAllOrders,
            _ => KuCoinEndpoint::FuturesAllOrders,
        };

        let mut params = HashMap::new();
        // KuCoin uses status=done for filled/cancelled orders
        params.insert("status".to_string(), "done".to_string());

        if let Some(ref symbol) = filter.symbol {
            params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        }

        if let Some(start) = filter.start_time {
            params.insert("startAt".to_string(), start.to_string());
        }

        if let Some(end) = filter.end_time {
            params.insert("endAt".to_string(), end.to_string());
        }

        if let Some(limit) = filter.limit {
            params.insert("pageSize".to_string(), limit.min(500).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        KuCoinParser::parse_orders(&response)
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

                Ok(Order {
                    id: order_id.to_string(),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy,
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
            CancelScope::All { ref symbol } => {
                let account_type = req.account_type;
                let cancelled_ids = self.cancel_all_orders(symbol.clone(), account_type).await?;
                let count = cancelled_ids.len();
                let sym_str = symbol.as_ref().map(|s| s.to_string()).unwrap_or_default();
                Ok(Order {
                    id: format!("batch_cancel_{}", crate::core::timestamp_millis()),
                    client_order_id: None,
                    symbol: sym_str,
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: count as f64,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: crate::core::TimeInForce::Gtc,
                })
            }
            CancelScope::BySymbol { ref symbol } => {
                let account_type = req.account_type;
                let cancelled_ids = self.cancel_all_orders(Some(symbol.clone()), account_type).await?;
                let count = cancelled_ids.len();
                Ok(Order {
                    id: format!("batch_cancel_{}", crate::core::timestamp_millis()),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: count as f64,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: crate::core::TimeInForce::Gtc,
                })
            }
            CancelScope::Batch { ref order_ids } => {
                // KuCoin does not have a native batch cancel endpoint — return UnsupportedOperation
                let _ = order_ids;
                Err(ExchangeError::UnsupportedOperation(
                    "KuCoin does not support batch cancel. Cancel orders individually.".to_string()
                ))
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                "This cancel scope is not supported by KuCoin".to_string()
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

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        // Spot uses SpotFills (/api/v1/fills on spot base URL)
        // Futures uses FuturesFills (/api/v1/fills on futures base URL)
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotFills,
            _ => KuCoinEndpoint::FuturesFills,
        };

        let mut params = HashMap::new();

        if let Some(ref symbol) = filter.symbol {
            // symbol filter accepts KuCoin-formatted symbol (e.g. "BTC-USDT")
            // If the caller passes "BTC/USDT" we convert it; raw KuCoin symbols are passed as-is
            let formatted = if symbol.contains('/') {
                let parts: Vec<&str> = symbol.splitn(2, '/').collect();
                format_symbol(parts[0], parts[1], account_type)
            } else {
                symbol.clone()
            };
            params.insert("symbol".to_string(), formatted);
        }

        if let Some(ref order_id) = filter.order_id {
            params.insert("orderId".to_string(), order_id.clone());
        }

        if let Some(start) = filter.start_time {
            params.insert("startAt".to_string(), start.to_string());
        }

        if let Some(end) = filter.end_time {
            params.insert("endAt".to_string(), end.to_string());
        }

        // KuCoin max pageSize = 500
        if let Some(limit) = filter.limit {
            params.insert("pageSize".to_string(), limit.min(500).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        KuCoinParser::parse_fills(&response)
    }

    fn trading_capabilities(&self, account_type: AccountType) -> TradingCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            has_stop_market: true,
            has_stop_limit: true,
            // TrailingStop / Bracket / Twap all return UnsupportedOperation in place_order.
            has_trailing_stop: false,
            has_bracket: false,
            // OCO: native endpoint POST /api/v3/oco/order exists for Spot only;
            // Futures returns UnsupportedOperation.
            has_oco: !is_futures,
            // AmendOrder (POST /api/v1/orders/{id}) is Futures-only;
            // Spot returns UnsupportedOperation.
            has_amend: is_futures,
            // BatchOrders impl exists for both; cancel_orders_batch returns
            // UnsupportedOperation (no native KuCoin batch-cancel endpoint).
            has_batch: true,
            // Spot HF Pro batch: max 5 orders (same symbol, limit only).
            // Futures batch: max 20 orders (POST /api/v1/orders/multi).
            max_batch_size: if is_futures { Some(20) } else { Some(5) },
            has_cancel_all: true,
            has_user_trades: true,
            has_order_history: true,
        }
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

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // KuCoin GET /api/v1/base-fee (account-level) or /api/v1/trade-fees?symbols=... (per-symbol)
        let account_type = AccountType::Spot;

        let mut params = HashMap::new();
        if let Some(sym) = symbol {
            let parts: Vec<&str> = sym.split('/').collect();
            let kucoin_symbol = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], account_type)
            } else {
                sym.to_string()
            };
            params.insert("symbols".to_string(), kucoin_symbol);
        }

        // Use base-fee endpoint (no symbol needed for account-wide fees)
        let base_url = self.urls.rest_url(account_type);
        let path = "/api/v1/base-fee";
        let url = format!("{}{}", base_url, path);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("GET", path, "");

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        self.check_response(&response)?;

        let data = KuCoinParser::extract_data(&response)?;

        let maker_rate = data.get("makerFeeRate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.001);
        let taker_rate = data.get("takerFeeRate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.001);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(String::from),
            tier: None,
        })
    }

    fn account_capabilities(&self, account_type: AccountType) -> AccountCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        AccountCapabilities {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            // AccountTransfers impl exists (inner-transfer + history).
            has_transfers: true,
            // SubAccounts impl exists.
            has_sub_accounts: true,
            // CustodialFunds impl covers deposit addresses and withdrawals.
            has_deposit_withdraw: true,
            // Margin borrow/repay is exposed via get_balance(Margin) and the AccountTransfers
            // impl; there is no dedicated MarginTrading trait implemented, so false.
            has_margin: false,
            // No EarnStaking trait is implemented.
            has_earn_staking: false,
            // FundingHistory (GET /api/v1/funding-history) hits the Futures domain only;
            // the Spot API has no equivalent funding payment endpoint.
            has_funding_history: is_futures,
            // AccountLedger (GET /api/v1/accounts/ledgers) is a Spot/Main account endpoint;
            // Futures account does not expose a ledger via this path.
            has_ledger: !is_futures,
            // No ConvertSwap trait is implemented.
            has_convert: false,
            // Positions (GET /api/v1/positions) are Futures-only (KuCoin Futures domain).
            // Spot has no positions concept.
            has_positions: is_futures,
        }
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
            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetMarginMode not supported for Spot/Margin on KuCoin".to_string()
                        ));
                    }
                    _ => {}
                }

                // KuCoin Futures uses per-position margin mode set via leverage endpoint
                // "autoDeposit" flag controls isolated vs cross margin behavior
                let auto_deposit = match margin_type {
                    crate::core::MarginType::Isolated => false,
                    crate::core::MarginType::Cross => true,
                };

                let body = json!({
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "autoDeposit": auto_deposit,
                });

                // KuCoin auto-deposit endpoint: POST /api/v1/position/margin/auto-deposit-status
                let base_url = self.urls.rest_url(account_type);
                let path = "/api/v1/position/margin/auto-deposit-status";
                let url = format!("{}{}", base_url, path);

                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", path, &body_str);

                let response = self.http.post(&url, &body, &headers).await?;
                self.check_response(&response)?;
                Ok(())
            }
            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "AddMargin only supported for futures on KuCoin".to_string()
                        ));
                    }
                    _ => {}
                }

                let body = json!({
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "bizNo": format!("cc_{}", crate::core::timestamp_millis()),
                    "margin": amount,
                });

                // KuCoin: POST /api/v1/position/margin/deposit-margin
                let base_url = self.urls.rest_url(account_type);
                let path = "/api/v1/position/margin/deposit-margin";
                let url = format!("{}{}", base_url, path);

                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", path, &body_str);

                let response = self.http.post(&url, &body, &headers).await?;
                self.check_response(&response)?;
                Ok(())
            }
            PositionModification::RemoveMargin { .. } => {
                // KuCoin does not support removing margin from a futures position
                Err(ExchangeError::UnsupportedOperation(
                    "KuCoin does not support RemoveMargin on futures positions".to_string()
                ))
            }
            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition only supported for futures on KuCoin".to_string()
                        ));
                    }
                    _ => {}
                }

                // KuCoin: place a market order with closeOrder=true to close entire position
                let client_oid = format!("cc_{}", crate::core::timestamp_millis());
                let body = json!({
                    "clientOid": client_oid,
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "side": "buy",   // Will be auto-determined for close
                    "type": "market",
                    "size": 0,       // 0 = entire position
                    "closeOrder": true,
                });

                let response = self.post(KuCoinEndpoint::FuturesCreateOrder, body, account_type).await?;
                self.check_response(&response)?;
                Ok(())
            }
            PositionModification::SetTpSl { ref symbol, take_profit, stop_loss, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetTpSl only supported for futures on KuCoin".to_string()
                        ));
                    }
                    _ => {}
                }

                // KuCoin: POST /api/v1/stop-order with stop type
                // For existing position, use stop-sell for TP (if long) and stop-sell for SL
                if let Some(tp) = take_profit {
                    let client_oid = format!("tp_{}", crate::core::timestamp_millis());
                    let body = json!({
                        "clientOid": client_oid,
                        "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                        "side": "sell",
                        "type": "market",
                        "size": 0,
                        "closeOrder": true,
                        "stop": "up",
                        "stopPrice": tp.to_string(),
                        "stopPriceType": "TP",
                    });
                    let response = self.post(KuCoinEndpoint::FuturesCreateOrder, body, account_type).await?;
                    self.check_response(&response)?;
                }

                if let Some(sl) = stop_loss {
                    let client_oid = format!("sl_{}", crate::core::timestamp_millis());
                    let body = json!({
                        "clientOid": client_oid,
                        "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                        "side": "sell",
                        "type": "market",
                        "size": 0,
                        "closeOrder": true,
                        "stop": "down",
                        "stopPrice": sl.to_string(),
                        "stopPriceType": "TP",
                    });
                    let response = self.post(KuCoinEndpoint::FuturesCreateOrder, body, account_type).await?;
                    self.check_response(&response)?;
                }

                Ok(())
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                "This position modification is not supported by KuCoin".to_string()
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders — optionally filtered to a symbol.
///
/// - Spot:    `DELETE /api/v1/orders?symbol=BTC-USDT`
/// - Futures: `DELETE /api/v1/orders?symbol=XBTUSDTM`
#[async_trait]
impl CancelAll for KuCoinConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let symbol = match &scope {
            CancelScope::All { symbol } => symbol.clone(),
            CancelScope::BySymbol { symbol } => Some(symbol.clone()),
            _ => {
                return Err(ExchangeError::InvalidRequest(
                    "cancel_all_orders only accepts All or BySymbol scope".to_string()
                ));
            }
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KuCoinEndpoint::SpotCancelAllOrders,
            _ => KuCoinEndpoint::FuturesCancelAllOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert(
                "symbol".to_string(),
                format_symbol(&s.base, &s.quote, account_type),
            );
        }

        // Build DELETE request with query params
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
        KuCoinParser::parse_cancel_all_response(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Native batch order placement.
///
/// - Spot HF:    `POST /api/v1/hf/orders/multi` — max 5 limit orders, same pair
/// - Futures:    `POST /api/v1/orders/multi`     — max 20 orders (futures base URL)
///
/// Batch cancel is not a discrete endpoint on KuCoin; cancel-all handles that.
#[async_trait]
impl BatchOrders for KuCoinConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let account_type = orders[0].account_type;
        let max = self.max_batch_place_size();

        if orders.len() > max {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch size {} exceeds KuCoin limit of {}", orders.len(), max)
            ));
        }

        let (endpoint, batch_json) = match account_type {
            AccountType::Spot | AccountType::Margin => {
                // Spot HF batch: all orders must be for the same symbol, limit only
                let batch: Vec<Value> = orders.iter().map(|req| {
                    let sym = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
                    let side_str = match req.side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };
                    let client_oid = req.client_order_id.clone()
                        .unwrap_or_else(|| format!("cc_{}", crate::core::timestamp_millis()));
                    let tif = match req.time_in_force {
                        crate::core::TimeInForce::Ioc => "IOC",
                        crate::core::TimeInForce::Fok => "FOK",
                        _ => "GTC",
                    };
                    let mut obj = json!({
                        "clientOid": client_oid,
                        "symbol": sym,
                        "side": side_str,
                        "type": "limit",
                        "timeInForce": tif,
                        "size": self.precision.qty(&sym, req.quantity),
                    });
                    if let OrderType::Limit { price } = req.order_type {
                        obj["price"] = json!(self.precision.price(&sym, price));
                    }
                    obj
                }).collect();
                (KuCoinEndpoint::SpotBatchOrders, json!(batch))
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                // Futures batch: supports limit, market, stop orders
                let batch: Vec<Value> = orders.iter().map(|req| {
                    let sym = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
                    let side_str = match req.side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };
                    let client_oid = req.client_order_id.clone()
                        .unwrap_or_else(|| format!("cc_{}", crate::core::timestamp_millis()));
                    let mut obj = json!({
                        "clientOid": client_oid,
                        "symbol": sym,
                        "side": side_str,
                        "size": self.precision.qty(&sym, req.quantity),
                    });
                    match req.order_type {
                        OrderType::Market => {
                            obj["type"] = json!("market");
                        }
                        OrderType::Limit { price } => {
                            obj["type"] = json!("limit");
                            obj["price"] = json!(self.precision.price(&sym, price));
                        }
                        _ => {
                            obj["type"] = json!("market");
                        }
                    }
                    if req.reduce_only {
                        obj["reduceOnly"] = json!(true);
                    }
                    obj
                }).collect();
                (KuCoinEndpoint::FuturesBatchOrders, json!(batch))
            }
            _ => return Err(ExchangeError::UnsupportedOperation(
                "This account type is not supported for batch orders on KuCoin".to_string()
            )),
        };

        let response = self.post(endpoint, batch_json, account_type).await?;
        KuCoinParser::parse_batch_orders_response(&response)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // KuCoin does not have a native batch cancel endpoint.
        let _ = order_ids;
        Err(ExchangeError::UnsupportedOperation(
            "KuCoin does not have a native batch cancel endpoint. Use CancelAll::cancel_all_orders instead.".to_string()
        ))
    }

    fn max_batch_place_size(&self) -> usize {
        // Spot HF: 5 limit orders (same pair). Futures: 20.
        // We use account_type to distinguish but trait doesn't pass it.
        // Return the more restrictive spot limit as the default.
        5
    }

    fn max_batch_cancel_size(&self) -> usize {
        0 // No native batch cancel
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Amend a live futures order in-place.
///
/// KuCoin Futures: `POST /api/v1/orders/{orderId}` with amended fields.
/// Spot does NOT support amend — returns `UnsupportedOperation`.
#[async_trait]
impl AmendOrder for KuCoinConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        match req.account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Amend order is not supported for Spot/Margin on KuCoin (futures only)".to_string()
                ));
            }
            _ => {}
        }

        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price or quantity must be provided for amend".to_string()
            ));
        }

        let account_type = req.account_type;
        let symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
        let base_url = self.urls.rest_url(account_type);
        // Substitute orderId in the path
        let path = KuCoinEndpoint::FuturesAmendOrder.path()
            .replace("{orderId}", &req.order_id);
        let url = format!("{}{}", base_url, path);

        let mut body = json!({});
        if let Some(price) = req.fields.price {
            body["price"] = json!(self.precision.price(&symbol_str, price));
        }
        if let Some(qty) = req.fields.quantity {
            body["size"] = json!(qty as i64);
        }

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", &path, &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        self.check_response(&response)?;

        let symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
        KuCoinParser::parse_amend_order(&response, &symbol_str)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal transfers between KuCoin account types.
///
/// - Transfer: `POST /api/v3/accounts/inner-transfer`
/// - History:  `GET  /api/v1/accounts/inner-transfer`
///
/// AccountType mapping:
/// - `Spot`           → `"main"`   (Main/funding account)
/// - `FuturesCross`   → `"trade"`  (Spot trade account)
/// - `Margin`         → `"margin"` (Margin account)
#[async_trait]
impl AccountTransfers for KuCoinConnector {
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        fn map_account(at: AccountType) -> &'static str {
            match at {
                AccountType::Spot => "main",
                AccountType::FuturesCross | AccountType::FuturesIsolated => "trade",
                AccountType::Margin => "margin",
                _ => "main",
            }
        }

        let client_oid = format!("cc_{}", crate::core::timestamp_millis());
        let body = json!({
            "clientOid": client_oid,
            "currency": req.asset,
            "from": map_account(req.from_account),
            "to": map_account(req.to_account),
            "amount": req.amount.to_string(),
        });

        let account_type = AccountType::Spot; // transfers use spot base URL
        let base_url = self.urls.rest_url(account_type);
        let path = KuCoinEndpoint::InnerTransfer.path();
        let url = format!("{}{}", base_url, path);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", path, &body_str);

        let response = self.http.post(&url, &body, &headers).await?;
        self.check_response(&response)?;

        let data = KuCoinParser::extract_data(&response)?;
        let transfer_id = data.get("orderId")
            .and_then(|v| v.as_str())
            .unwrap_or(&client_oid)
            .to_string();

        Ok(TransferResponse {
            transfer_id,
            status: "Successful".to_string(),
            asset: req.asset,
            amount: req.amount,
            timestamp: Some(crate::core::timestamp_millis() as i64),
        })
    }

    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);
        let path = KuCoinEndpoint::TransferHistory.path();

        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(start) = filter.start_time {
            params.insert("startAt".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endAt".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("pageSize".to_string(), limit.min(500).to_string());
        }

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
        let headers = auth.sign_request("GET", &full_path, "");

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;

        let data = KuCoinParser::extract_data(&response)?;
        let items = data.get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut records = Vec::with_capacity(items.len());
        for item in items {
            let transfer_id = item.get("orderId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let asset = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = item.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("DONE")
                .to_string();
            let timestamp = item.get("createdAt")
                .and_then(|v| v.as_i64());
            records.push(TransferResponse {
                transfer_id,
                status,
                asset,
                amount,
                timestamp,
            });
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit and withdrawal management for KuCoin.
///
/// - Deposit address: `GET  /api/v3/deposit-addresses`
/// - Withdraw:        `POST /api/v1/withdrawals`
/// - Deposit history: `GET  /api/v1/deposits`
/// - Withdrawal hist: `GET  /api/v1/withdrawals`
#[async_trait]
impl CustodialFunds for KuCoinConnector {
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);
        let path = KuCoinEndpoint::DepositAddress.path();

        let mut params = HashMap::new();
        params.insert("currency".to_string(), asset.to_string());
        if let Some(chain) = network {
            params.insert("chain".to_string(), chain.to_string());
        }

        let query: Vec<String> = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_str = query.join("&");
        let url = format!("{}{}?{}", base_url, path, query_str);
        let full_path = format!("{}?{}", path, query_str);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("GET", &full_path, "");

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;

        let data = KuCoinParser::extract_data(&response)?;
        let address = data.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing address field".to_string()))?
            .to_string();
        let tag = data.get("memo")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let network_out = data.get("chain")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(DepositAddress {
            address,
            tag,
            network: network_out,
            asset: asset.to_string(),
            created_at: None,
        })
    }

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);
        let path = KuCoinEndpoint::Withdraw.path();
        let url = format!("{}{}", base_url, path);

        let mut body = json!({
            "currency": req.asset,
            "address": req.address,
            "amount": req.amount,
        });
        if let Some(chain) = req.network {
            body["chain"] = json!(chain);
        }
        if let Some(memo) = req.tag {
            body["memo"] = json!(memo);
        }

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", path, &body_str);

        let response = self.http.post(&url, &body, &headers).await?;
        self.check_response(&response)?;

        let data = KuCoinParser::extract_data(&response)?;
        let withdraw_id = data.get("withdrawalId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);

        let endpoint = match filter.record_type {
            FundsRecordType::Deposit => KuCoinEndpoint::DepositHistory,
            FundsRecordType::Withdrawal => KuCoinEndpoint::WithdrawalHistory,
            FundsRecordType::Both => KuCoinEndpoint::DepositHistory, // fetch deposits first
        };

        let path = endpoint.path();

        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(ref asset) = filter.asset {
            params.insert("currency".to_string(), asset.clone());
        }
        if let Some(start) = filter.start_time {
            params.insert("startAt".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endAt".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("pageSize".to_string(), limit.min(500).to_string());
        }

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
        let headers = auth.sign_request("GET", &full_path, "");

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;

        let data = KuCoinParser::extract_data(&response)?;
        let items = data.get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let is_deposit = matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both);
        let mut records = Vec::with_capacity(items.len());

        for item in items {
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let asset = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = item.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string();
            let timestamp = item.get("createdAt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let tx_hash = item.get("txId")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            let network = item.get("chain")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);

            if is_deposit {
                records.push(FundsRecord::Deposit {
                    id,
                    asset,
                    amount,
                    tx_hash,
                    network,
                    status,
                    timestamp,
                });
            } else {
                let fee = item.get("fee")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let address = item.get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tag = item.get("memo")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(String::from);
                records.push(FundsRecord::Withdrawal {
                    id,
                    asset,
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

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB-ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Sub-account management for KuCoin.
///
/// - Create:   `POST /api/v2/sub/user/created`
/// - List:     `GET  /api/v2/sub/user`
/// - Transfer: `POST /api/v2/accounts/sub-transfer`
/// - Balance:  `GET  /api/v1/sub-accounts/{subUserId}`
#[async_trait]
impl SubAccounts for KuCoinConnector {
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        match op {
            SubAccountOperation::Create { label } => {
                let path = KuCoinEndpoint::SubAccountCreate.path();
                let url = format!("{}{}", base_url, path);
                let body = json!({
                    "subName": label,
                    "access": "All",
                });
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", path, &body_str);
                let response = self.http.post(&url, &body, &headers).await?;
                self.check_response(&response)?;

                let data = KuCoinParser::extract_data(&response)?;
                let uid = data.get("uid")
                    .and_then(|v| v.as_str())
                    .or_else(|| data.get("userId").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string();
                let name = data.get("subName")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&label)
                    .to_string();

                Ok(SubAccountResult {
                    id: Some(uid),
                    name: Some(name),
                    accounts: vec![],
                    transaction_id: None,
                })
            }

            SubAccountOperation::List => {
                let path = KuCoinEndpoint::SubAccountList.path();
                let url = format!("{}{}", base_url, path);
                let headers = auth.sign_request("GET", path, "");
                let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
                self.check_response(&response)?;

                let data = KuCoinParser::extract_data(&response)?;
                let items = data.as_array().cloned().unwrap_or_default();

                let accounts: Vec<SubAccount> = items.iter().map(|item| {
                    SubAccount {
                        id: item.get("userId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: item.get("subName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        status: item.get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Normal")
                            .to_string(),
                    }
                }).collect();

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts,
                    transaction_id: None,
                })
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                let path = KuCoinEndpoint::SubAccountTransfer.path();
                let url = format!("{}{}", base_url, path);
                let direction = if to_sub { "OUT" } else { "IN" };
                let client_oid = format!("cc_{}", crate::core::timestamp_millis());
                let body = json!({
                    "clientOid": client_oid,
                    "currency": asset,
                    "amount": amount.to_string(),
                    "direction": direction,
                    "subUserId": sub_account_id,
                    "accountType": "MAIN",
                });
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", path, &body_str);
                let response = self.http.post(&url, &body, &headers).await?;
                self.check_response(&response)?;

                let data = KuCoinParser::extract_data(&response)?;
                let order_id = data.get("orderId")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&client_oid)
                    .to_string();

                Ok(SubAccountResult {
                    id: Some(sub_account_id),
                    name: None,
                    accounts: vec![],
                    transaction_id: Some(order_id),
                })
            }

            SubAccountOperation::GetBalance { sub_account_id } => {
                let path = KuCoinEndpoint::SubAccountBalance.path()
                    .replace("{subUserId}", &sub_account_id);
                let url = format!("{}{}", base_url, path);
                let headers = auth.sign_request("GET", &path, "");
                let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
                self.check_response(&response)?;

                Ok(SubAccountResult {
                    id: Some(sub_account_id),
                    name: None,
                    accounts: vec![],
                    transaction_id: None,
                })
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// C3 ADDITIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl KuCoinConnector {
    /// Query the transferable quota for a specific asset and account type.
    ///
    /// `GET /api/v1/accounts/transferable`
    /// Required parameters: `currency`, `type` (MAIN, TRADE, MARGIN, etc.).
    pub async fn get_transfer_quotas(
        &self,
        currency: &str,
        kucoin_account_type: &str,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency".to_string(), currency.to_string());
        params.insert("type".to_string(), kucoin_account_type.to_string());
        self.get(KuCoinEndpoint::TransferQuotas, params, AccountType::Spot).await
    }

    /// Cancel a pending withdrawal by withdrawal ID.
    ///
    /// `DELETE /api/v1/withdrawals/{withdrawalId}`
    pub async fn cancel_withdrawal(&self, withdrawal_id: &str) -> ExchangeResult<Value> {
        self.delete(
            KuCoinEndpoint::WithdrawalCancel,
            &[("withdrawalId", withdrawal_id)],
            AccountType::Spot,
        ).await
    }

    /// Query withdrawal quota for a specific asset and network.
    ///
    /// `GET /api/v1/withdrawals/quotas`
    /// Required parameter: `currency`.
    /// Optional parameter: `chain` (network identifier, e.g. "ERC20").
    pub async fn get_withdrawal_quotas(
        &self,
        currency: &str,
        chain: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency".to_string(), currency.to_string());
        if let Some(c) = chain {
            params.insert("chain".to_string(), c.to_string());
        }
        self.get(KuCoinEndpoint::WithdrawalQuotas, params, AccountType::Spot).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl FundingHistory for KuCoinConnector {
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        let symbol = filter.symbol.ok_or_else(|| {
            ExchangeError::UnsupportedOperation(
                "KuCoin funding history requires a symbol".to_string(),
            )
        })?;

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol);
        if let Some(start) = filter.start_time {
            params.insert("startAt".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endAt".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("maxCount".to_string(), limit.min(100).to_string());
        }

        let response = self
            .get(KuCoinEndpoint::FuturesFundingHistory, params, AccountType::FuturesCross)
            .await?;
        KuCoinParser::parse_funding_payments(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT LEDGER
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountLedger for KuCoinConnector {
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let mut params = HashMap::new();
        if let Some(asset) = &filter.asset {
            params.insert("currency".to_string(), asset.clone());
        }
        if let Some(start) = filter.start_time {
            params.insert("startAt".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endAt".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("pageSize".to_string(), limit.min(500).to_string());
        }
        params.insert("currentPage".to_string(), "1".to_string());

        let response = self
            .get(KuCoinEndpoint::SpotLedger, params, AccountType::Spot)
            .await?;
        KuCoinParser::parse_ledger(&response)
    }
}
