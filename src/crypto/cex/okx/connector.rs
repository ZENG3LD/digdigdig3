//! # OKX Connector
//!
//! Реализация всех core трейтов для OKX API v5.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType,Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    AmendRequest, CancelAllResponse, OrderResult,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{OkxUrls, OkxEndpoint, format_symbol, map_kline_interval, get_inst_type, get_trade_mode};
use super::auth::OkxAuth;
use super::parser::OkxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// OKX коннектор
pub struct OkxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<OkxAuth>,
    /// URL'ы (mainnet/testnet)
    urls: OkxUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (10 requests per 2 seconds = 5 rps)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl OkxConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            OkxUrls::TESTNET
        } else {
            OkxUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(OkxAuth::new)
            .transpose()?;

        // Initialize rate limiter: 20 requests per 2 seconds (OKX public endpoint limit)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(20, Duration::from_secs(2))
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

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock()
                    .expect("Rate limiter mutex poisoned");
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

    /// GET запрос
    async fn get(
        &self,
        endpoint: OkxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url();
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
            if self.testnet {
                auth.sign_request_testnet("GET", &full_path, "")
            } else {
                auth.sign_request("GET", &full_path, "")
            }
        } else {
            HashMap::new()
        };

        self.http.get_with_headers(&url, &HashMap::new(), &headers).await
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: OkxEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = if self.testnet {
            auth.sign_request_testnet("POST", path, &body_str)
        } else {
            auth.sign_request("POST", path, &body_str)
        };

        self.http.post(&url, &body, &headers).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (OKX-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить все тикеры для определенного типа инструментов
    pub async fn get_all_tickers(&self, account_type: AccountType) -> ExchangeResult<Vec<Ticker>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        let response = self.get(OkxEndpoint::AllTickers, params).await?;
        // TODO: implement parse_all_tickers in parser
        let _ = response;
        Ok(vec![])
    }

    /// Получить информацию о инструментах/символах
    pub async fn get_instruments(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        let response = self.get(OkxEndpoint::Instruments, params).await?;
        OkxParser::parse_symbols(&response)
    }

    /// Получить список символов (алиас для get_instruments для совместимости с тестами)
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        self.get_instruments(account_type).await
    }

    /// Получить server time
    pub async fn get_server_time(&self) -> ExchangeResult<i64> {
        let response = self.get(OkxEndpoint::ServerTime, HashMap::new()).await?;
        let data = OkxParser::extract_first_data(&response)?;
        OkxParser::parse_i64(data.get("ts").ok_or_else(|| ExchangeError::Parse("Missing 'ts'".to_string()))?)
            .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for OkxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::OKX
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
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for OkxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::Ticker, params).await?;
        let ticker = OkxParser::parse_ticker(&response)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        if let Some(d) = depth {
            params.insert("sz".to_string(), d.to_string());
        }

        let response = self.get(OkxEndpoint::Orderbook, params).await?;
        OkxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("bar".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(300).to_string());
        }

        // OKX naming is inverted: "after" = older-than (paginate backward).
        // /market/candles has ~1440 bar depth limit on 1m.
        // /market/history-candles has full depth — use it for pagination.
        let endpoint = if end_time.is_some() {
            OkxEndpoint::HistoryKlines
        } else {
            OkxEndpoint::Klines
        };

        if let Some(et) = end_time {
            params.insert("after".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params).await?;
        OkxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::Ticker, params).await?;
        OkxParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        self.get(OkxEndpoint::ServerTime, HashMap::new()).await?;
        Ok(())
    }

    /// Получить информацию о всех торговых символах биржи
    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        self.get_instruments(account_type).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for OkxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let inst_id = format_symbol(&symbol.base, &symbol.quote, account_type);
        let td_mode = get_trade_mode(account_type);
        let side_str = match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };
        let cl_ord_id = req.client_order_id.clone()
            .unwrap_or_else(|| format!("cc_{}", crate::core::timestamp_millis()));

        let body = match req.order_type {
            OrderType::Market => {
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "market",
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::Limit { price } => {
                let tif = match req.time_in_force {
                    crate::core::TimeInForce::PostOnly => "post_only",
                    crate::core::TimeInForce::Ioc => "optimal_limit_ioc",
                    crate::core::TimeInForce::Fok => "fok",
                    _ => "limit",
                };
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": tif,
                    "px": price.to_string(),
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::PostOnly { price } => {
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "post_only",
                    "px": price.to_string(),
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::Ioc { price } => {
                let px_str = price.map(|p| p.to_string()).unwrap_or_else(|| "-1".to_string());
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "optimal_limit_ioc",
                    "px": px_str,
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::Fok { price } => {
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "fok",
                    "px": price.to_string(),
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::StopMarket { stop_price } => {
                // OKX algo order: tp/sl trigger; uses /api/v5/trade/order-algo
                // For a simpler approach, use stop-market via conditional orders
                // OKX represents these as ordType="conditional" with slTriggerPx or tpTriggerPx
                let (tp_trigger_px, sl_trigger_px) = match side {
                    OrderSide::Buy => (Some(stop_price), None::<f64>),
                    OrderSide::Sell => (None, Some(stop_price)),
                };
                let mut body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "conditional",
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                });
                if let Some(tp) = tp_trigger_px {
                    body["tpTriggerPx"] = json!(tp.to_string());
                    body["tpOrdPx"] = json!("-1"); // market price
                }
                if let Some(sl) = sl_trigger_px {
                    body["slTriggerPx"] = json!(sl.to_string());
                    body["slOrdPx"] = json!("-1"); // market price
                }
                body
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                let (tp_trigger_px, sl_trigger_px) = match side {
                    OrderSide::Buy => (Some((stop_price, limit_price)), None::<(f64, f64)>),
                    OrderSide::Sell => (None, Some((stop_price, limit_price))),
                };
                let mut body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "conditional",
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                });
                if let Some((trigger, limit)) = tp_trigger_px {
                    body["tpTriggerPx"] = json!(trigger.to_string());
                    body["tpOrdPx"] = json!(limit.to_string());
                }
                if let Some((trigger, limit)) = sl_trigger_px {
                    body["slTriggerPx"] = json!(trigger.to_string());
                    body["slOrdPx"] = json!(limit.to_string());
                }
                body
            }
            OrderType::ReduceOnly { price } => {
                let ord_type = if price.is_some() { "limit" } else { "market" };
                let mut body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": ord_type,
                    "sz": quantity.to_string(),
                    "reduceOnly": true,
                    "clOrdId": cl_ord_id,
                });
                if let Some(px) = price {
                    body["px"] = json!(px.to_string());
                }
                body
            }
            OrderType::Gtd { price, expire_time } => {
                // OKX supports GTC/IOC but not native GTD — place as limit with note
                let _ = expire_time;
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "limit",
                    "px": price.to_string(),
                    "sz": quantity.to_string(),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::TrailingStop { callback_rate, activation_price } => {
                // OKX supports trailing stop via algo orders endpoint
                // ordType="move_order_stop", callbackRatio or callbackSpread
                let mut body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "move_order_stop",
                    "sz": quantity.to_string(),
                    "callbackRatio": callback_rate.to_string(),
                    "clOrdId": cl_ord_id,
                });
                if let Some(act_px) = activation_price {
                    body["activePx"] = json!(act_px.to_string());
                }
                body
            }
            OrderType::Twap { duration_seconds, interval_seconds } => {
                // OKX has TWAP algo: ordType="twap"
                let _ = interval_seconds;
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "twap",
                    "sz": quantity.to_string(),
                    "pxVar": "0.01",  // 1% price variance
                    "szLimit": quantity.to_string(),
                    "pxLimit": "0",   // no limit
                    "timeInterval": duration_seconds.to_string(),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::Oco { .. } | OrderType::Bracket { .. } | OrderType::Iceberg { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
                ));
            }
        };

        let response = self.post(OkxEndpoint::PlaceOrder, body).await?;
        let order_id = OkxParser::parse_order_response(&response)?;

        // Get full order details
        let symbol_str = symbol.to_string();
        let order = self.get_order(&symbol_str, &order_id, account_type).await?;
        Ok(PlaceOrderResponse::Simple(order))
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        if let Some(ref symbol) = filter.symbol {
            let inst_id = format_symbol(&symbol.base, &symbol.quote, account_type);
            params.insert("instId".to_string(), inst_id);
        }

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        if let Some(start) = filter.start_time {
            params.insert("begin".to_string(), start.to_string());
        }

        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), end.to_string());
        }

        let response = self.get(OkxEndpoint::OrderHistory, params).await?;
        OkxParser::parse_orders(&response)
    }

async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "ordId": order_id,
                });

                let response = self.post(OkxEndpoint::CancelOrder, body).await?;
                OkxParser::parse_order_response(&response)?;

                // Get full order details after cancellation
                let symbol_str = symbol.to_string();
                self.get_order(&symbol_str, order_id, account_type).await
            }
            CancelScope::All { ref symbol } => {
                let account_type = req.account_type;
                let inst_type = get_inst_type(account_type).to_string();

                // OKX cancel-all requires cancelling per instrument type; no single "cancel all" REST endpoint.
                // We fetch open orders and cancel each — but per non-composition rule we must not loop.
                // OKX does NOT have an atomic cancel-all REST endpoint; return UnsupportedOperation.
                // (The batch cancel endpoint requires explicit ordId list.)
                let _ = (symbol, inst_type);
                Err(ExchangeError::UnsupportedOperation(
                    "OKX does not provide an atomic cancel-all REST endpoint. Use CancelScope::Batch with explicit order IDs.".to_string()
                ))
            }
            CancelScope::BySymbol { ref symbol } => {
                // Same limitation as All — no atomic by-symbol cancel-all
                let _ = symbol;
                Err(ExchangeError::UnsupportedOperation(
                    "OKX does not provide an atomic cancel-by-symbol REST endpoint. Use CancelScope::Batch with explicit order IDs.".to_string()
                ))
            }
            CancelScope::Batch { ref order_ids } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for batch cancel".into()))?
                    .clone();
                let account_type = req.account_type;
                let inst_id = format_symbol(&symbol.base, &symbol.quote, account_type);

                // OKX batch cancel: POST /api/v5/trade/cancel-batch-orders
                // Body is an array of {instId, ordId}
                let orders_arr: Vec<Value> = order_ids.iter()
                    .map(|oid| json!({ "instId": inst_id, "ordId": oid }))
                    .collect();

                let response = self.post(OkxEndpoint::CancelBatchOrders, json!(orders_arr)).await?;

                // Return the first successfully cancelled order or error
                let data = OkxParser::extract_data(&response)?;
                let arr = data.as_array()
                    .ok_or_else(|| ExchangeError::Parse("Expected array in batch cancel response".to_string()))?;

                if arr.is_empty() {
                    return Err(ExchangeError::Api { code: 0, message: "No orders were cancelled".to_string() });
                }

                // Return a synthetic cancelled order from the first result
                let first = &arr[0];
                let order_id_str = OkxParser::get_str(first, "ordId").unwrap_or("").to_string();
                self.get_order(&symbol.to_string(), &order_id_str, account_type).await
            }
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("ordId".to_string(), order_id.to_string());

        let response = self.get(OkxEndpoint::GetOrder, params).await?;
        OkxParser::parse_order(&response)
    
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

        let mut params = HashMap::new();

        if let Some(s) = symbol {
            params.insert("instId".to_string(), format_symbol(&s.base, &s.quote, account_type));
        } else {
            params.insert("instType".to_string(), get_inst_type(account_type).to_string());
        }

        let response = self.get(OkxEndpoint::OpenOrders, params).await?;
        OkxParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for OkxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let _account_type = query.account_type;
        let mut params = HashMap::new();
        if let Some(a) = asset {
            params.insert("ccy".to_string(), a);
        }

        let response = self.get(OkxEndpoint::Balance, params).await?;
        OkxParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Get balances
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true, // OKX doesn't provide this field
            can_withdraw: false, // Would need to check permissions
            can_deposit: false,
            maker_commission: 0.08, // Default OKX fees
            taker_commission: 0.1,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // GET /api/v5/account/trade-fee
        let mut params = HashMap::new();
        params.insert("instType".to_string(), "SPOT".to_string());

        if let Some(sym) = symbol {
            let parts: Vec<&str> = sym.split('/').collect();
            let inst_id = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], AccountType::Spot)
            } else {
                sym.to_string()
            };
            params.insert("instId".to_string(), inst_id.clone());
        }

        let response = self.get(OkxEndpoint::AccountConfig, params).await?;
        // OKX returns fee info in account config: makerFeeRate, takerFeeRate
        let data = OkxParser::extract_first_data(&response)?;

        let maker_rate = OkxParser::get_str(data, "makerFeeRate")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.08 / 100.0);
        let taker_rate = OkxParser::get_str(data, "takerFeeRate")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.1 / 100.0);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(String::from),
            tier: OkxParser::get_str(data, "level").map(String::from),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for OkxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        let mut params = HashMap::new();

        if let Some(s) = symbol {
            params.insert("instId".to_string(), format_symbol(&s.base, &s.quote, account_type));
        } else {
            params.insert("instType".to_string(), get_inst_type(account_type).to_string());
        }

        let response = self.get(OkxEndpoint::Positions, params).await?;
        OkxParser::parse_positions(&response)
    
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

        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::FundingRate, params).await?;
        OkxParser::parse_funding_rate(&response)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();

                let margin_mode = match account_type {
                    AccountType::FuturesCross => "cross",
                    AccountType::FuturesIsolated => "isolated",
                    _ => return Err(ExchangeError::InvalidRequest("Leverage only supported for futures".to_string())),
                };

                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "lever": leverage.to_string(),
                    "mgnMode": margin_mode,
                });

                let response = self.post(OkxEndpoint::SetLeverage, body).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetMarginMode not supported for Spot".to_string()
                        ));
                    }
                    _ => {}
                }

                let mgn_mode = match margin_type {
                    crate::core::MarginType::Cross => "cross",
                    crate::core::MarginType::Isolated => "isolated",
                };

                // OKX switches margin mode via set-position-mode or set-leverage
                // For per-instrument margin mode: use set-leverage with appropriate mgnMode
                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "lever": "10",  // Required field, use current leverage
                    "mgnMode": mgn_mode,
                });

                let response = self.post(OkxEndpoint::SetLeverage, body).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "AddMargin only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: POST /api/v5/account/position/margin-balance
                // type=add for adding margin
                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "posSide": "net",
                    "type": "add",
                    "amt": amount.to_string(),
                });

                // OKX doesn't have a specific endpoint in our enum for this; use AccountConfig as fallback
                // We need to call the raw endpoint
                self.rate_limit_wait().await;
                let base_url = self.urls.rest_url();
                let path = "/api/v5/account/position/margin-balance";
                let url = format!("{}{}", base_url, path);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = if self.testnet {
                    auth.sign_request_testnet("POST", path, &body_str)
                } else {
                    auth.sign_request("POST", path, &body_str)
                };
                let response = self.http.post(&url, &body, &headers).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::RemoveMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "RemoveMargin only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: type=reduce for removing margin
                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "posSide": "net",
                    "type": "reduce",
                    "amt": amount.to_string(),
                });

                self.rate_limit_wait().await;
                let base_url = self.urls.rest_url();
                let path = "/api/v5/account/position/margin-balance";
                let url = format!("{}{}", base_url, path);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = if self.testnet {
                    auth.sign_request_testnet("POST", path, &body_str)
                } else {
                    auth.sign_request("POST", path, &body_str)
                };
                let response = self.http.post(&url, &body, &headers).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: POST /api/v5/trade/close-position
                let mgn_mode = match account_type {
                    AccountType::FuturesCross => "cross",
                    _ => "isolated",
                };

                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "mgnMode": mgn_mode,
                });

                self.rate_limit_wait().await;
                let base_url = self.urls.rest_url();
                let path = "/api/v5/trade/close-position";
                let url = format!("{}{}", base_url, path);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = if self.testnet {
                    auth.sign_request_testnet("POST", path, &body_str)
                } else {
                    auth.sign_request("POST", path, &body_str)
                };
                let response = self.http.post(&url, &body, &headers).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::SetTpSl { ref symbol, take_profit, stop_loss, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetTpSl only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: place algo order with ordType=conditional
                // For TP/SL on existing position, use ordType=oco (one-cancel-other)
                let mut body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "tdMode": "cross",
                    "side": "sell",  // For long position: TP/SL sell side
                    "ordType": "oco",
                    "sz": "0",  // Entire position (0 means position quantity)
                });

                if let Some(tp) = take_profit {
                    body["tpTriggerPx"] = json!(tp.to_string());
                    body["tpOrdPx"] = json!("-1"); // market price
                }
                if let Some(sl) = stop_loss {
                    body["slTriggerPx"] = json!(sl.to_string());
                    body["slOrdPx"] = json!("-1"); // market price
                }

                let response = self.post(OkxEndpoint::PlaceOrder, body).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders via OKX Dead Man's Switch endpoint.
///
/// OKX: `POST /api/v5/trade/cancel-all-after`
///
/// Sending `timeOut = "0"` immediately cancels all open orders and disables
/// the DMS timer. This is the only OKX native cancel-all mechanism.
///
/// Note: The `scope` symbol filter is not supported — OKX `cancel-all-after`
/// always cancels across all instruments. `CancelScope::BySymbol` will return
/// `UnsupportedOperation`.
#[async_trait]
impl CancelAll for OkxConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match &scope {
            CancelScope::All { .. } => {
                // Proceed — cancel-all-after cancels across all instruments
            }
            CancelScope::BySymbol { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "OKX cancel-all-after does not support per-symbol scope. \
                     Use CancelScope::All to cancel all open orders.".to_string()
                ));
            }
            _ => {
                return Err(ExchangeError::InvalidRequest(
                    "cancel_all_orders only accepts All or BySymbol scope".to_string()
                ));
            }
        }

        let body = json!({
            "timeOut": "0",
        });

        let response = self.post(OkxEndpoint::CancelAllAfter, body).await?;
        OkxParser::parse_cancel_all_response(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Modify a live order in-place via OKX native amend endpoint.
///
/// OKX: `POST /api/v5/trade/amend-order`
/// At least one of `newPx`, `newSz`, or `newStopPx` must be provided.
#[async_trait]
impl AmendOrder for OkxConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        if req.fields.price.is_none() && req.fields.quantity.is_none() && req.fields.trigger_price.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price, quantity, or trigger_price must be provided for amend".to_string()
            ));
        }

        let account_type = req.account_type;
        let mut body = json!({
            "instId": format_symbol(&req.symbol.base, &req.symbol.quote, account_type),
            "ordId": req.order_id,
        });

        if let Some(price) = req.fields.price {
            body["newPx"] = json!(price.to_string());
        }
        if let Some(qty) = req.fields.quantity {
            body["newSz"] = json!(qty.to_string());
        }
        if let Some(trigger_price) = req.fields.trigger_price {
            body["newStopPx"] = json!(trigger_price.to_string());
        }

        let response = self.post(OkxEndpoint::AmendOrder, body).await?;
        OkxParser::parse_amend_order_response(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Native batch order placement and cancellation via OKX batch endpoints.
///
/// OKX: `POST /api/v5/trade/batch-orders` (max 20), `POST /api/v5/trade/cancel-batch-orders` (max 20)
#[async_trait]
impl BatchOrders for OkxConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        if orders.len() > self.max_batch_place_size() {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch size {} exceeds OKX limit of {}", orders.len(), self.max_batch_place_size())
            ));
        }

        let order_list: Vec<serde_json::Value> = orders.iter().map(|req| {
            let account_type = req.account_type;
            let mut obj = serde_json::Map::new();
            obj.insert("instId".to_string(), json!(format_symbol(&req.symbol.base, &req.symbol.quote, account_type)));
            obj.insert("tdMode".to_string(), json!(get_trade_mode(account_type)));
            obj.insert("side".to_string(), json!(match req.side {
                OrderSide::Buy => "buy",
                OrderSide::Sell => "sell",
            }));

            match &req.order_type {
                OrderType::Market => {
                    obj.insert("ordType".to_string(), json!("market"));
                    obj.insert("sz".to_string(), json!(req.quantity.to_string()));
                }
                OrderType::Limit { price } => {
                    obj.insert("ordType".to_string(), json!("limit"));
                    obj.insert("sz".to_string(), json!(req.quantity.to_string()));
                    obj.insert("px".to_string(), json!(price.to_string()));
                }
                _ => {
                    obj.insert("ordType".to_string(), json!("market"));
                    obj.insert("sz".to_string(), json!(req.quantity.to_string()));
                }
            }

            if req.reduce_only {
                obj.insert("reduceOnly".to_string(), json!(true));
            }
            if let Some(ref cid) = req.client_order_id {
                obj.insert("clOrdId".to_string(), json!(cid));
            }

            serde_json::Value::Object(obj)
        }).collect();

        let response = self.post(OkxEndpoint::PlaceBatchOrders, serde_json::Value::Array(order_list)).await?;
        OkxParser::parse_batch_orders_response(&response)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }

        if order_ids.len() > self.max_batch_cancel_size() {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch cancel size {} exceeds OKX limit of {}", order_ids.len(), self.max_batch_cancel_size())
            ));
        }

        let sym = symbol.ok_or_else(|| ExchangeError::InvalidRequest(
            "instId (symbol) is required for batch cancel on OKX".to_string()
        ))?;

        // OKX requires instId per item — re-use the raw symbol string as-is
        let cancel_list: Vec<serde_json::Value> = order_ids.iter().map(|id| {
            json!({
                "instId": sym,
                "ordId": id,
            })
        }).collect();

        let response = self.post(OkxEndpoint::CancelBatchOrders, serde_json::Value::Array(cancel_list)).await?;
        OkxParser::parse_batch_orders_response(&response)
    }

    fn max_batch_place_size(&self) -> usize {
        20 // OKX limit
    }

    fn max_batch_cancel_size(&self) -> usize {
        20 // OKX limit
    }
}
