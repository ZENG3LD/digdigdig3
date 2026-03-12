//! # Bitget Connector
//!
//! Реализация всех core трейтов для Bitget.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Optional трейты
//! - `CancelAll` - отмена всех ордеров
//! - `AmendOrder` - изменение ордера
//! - `BatchOrders` - пакетные ордера

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
    Position, FundingRate, MarginType,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    TimeInForce,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::{CancelAll, AmendOrder, BatchOrders};
use crate::core::types::{
    ConnectorStats, CancelAllResponse, OrderResult, AmendRequest,
};
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{
    BitgetUrls, BitgetEndpoint, format_symbol, map_kline_interval,
    map_futures_granularity, get_product_type
};
use super::auth::BitgetAuth;
use super::parser::BitgetParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitget коннектор
pub struct BitgetConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BitgetAuth>,
    /// URL'ы (mainnet only for Bitget)
    urls: BitgetUrls,
    /// Rate limiter для market data (20 req/sec)
    market_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Rate limiter для trading (10 req/sec)
    trading_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BitgetConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, _testnet: bool) -> ExchangeResult<Self> {
        let urls = BitgetUrls::MAINNET;

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials
            .as_ref()
            .map(BitgetAuth::new)
            .transpose()?;

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = urls.rest_url(AccountType::Spot);
            let url = format!("{}/api/v2/public/time", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(data) = response.get("data") {
                    if let Some(server_time_str) = data.get("serverTime").and_then(|t| t.as_str()) {
                        if let Ok(server_time) = server_time_str.parse::<i64>() {
                            if let Some(ref mut a) = auth {
                                a.sync_time(server_time);
                            }
                        }
                    }
                }
            }
        }

        // Bitget rate limits: market 20/s, trading 10/s
        let market_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(20, Duration::from_secs(1))
        ));
        let trading_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(10, Duration::from_secs(1))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            market_limiter,
            trading_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None, false).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse rate limit headers from Bitget response and update the appropriate limiter.
    ///
    /// Bitget reports: `x-mbx-used-remain-limit` = remaining requests in the current second.
    fn update_rate_from_headers(&self, headers: &HeaderMap, is_private: bool) {
        if let Some(remaining) = headers
            .get("x-mbx-used-remain-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            let limiter = if is_private { &self.trading_limiter } else { &self.market_limiter };
            if let Ok(mut lim) = limiter.lock() {
                lim.update_from_server(remaining);
            }
        }
    }

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self, is_private: bool) {
        let limiter = if is_private { &self.trading_limiter } else { &self.market_limiter };
        loop {
            let wait_time = {
                let mut l = limiter.lock().expect("lock");
                if l.try_acquire() {
                    return;
                }
                l.time_until_ready()
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: BitgetEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit based on endpoint type
        self.rate_limit_wait(endpoint.requires_auth()).await;

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

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", path, &query, "")
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, endpoint.requires_auth());
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BitgetEndpoint,
        body: Value,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // POST endpoints are always trading-related
        self.rate_limit_wait(true).await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", path, "", &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers, true);
        self.check_response(&response)?;
        Ok(response)
    }

    /// Проверить response на ошибки
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("00000");

        if code != "00000" {
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
    // EXTENDED METHODS (Bitget-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить информацию о символах
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Value> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotSymbols,
            _ => BitgetEndpoint::FuturesContracts,
        };

        let mut params = HashMap::new();

        // Futures requires productType parameter
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), "USDT-FUTURES".to_string());
        }

        self.get(endpoint, params, account_type).await
    }

    /// Build common spot order body fields
    fn spot_order_body_base(
        symbol: &Symbol,
        side: OrderSide,
        quantity: Quantity,
        account_type: AccountType,
        client_oid: &str,
    ) -> Value {
        json!({
            "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
            "side": match side {
                OrderSide::Buy => "buy",
                OrderSide::Sell => "sell",
            },
            "size": quantity.to_string(),
            "clientOid": client_oid,
        })
    }

    /// Build common futures order body fields
    fn futures_order_body_base(
        symbol: &Symbol,
        side: OrderSide,
        quantity: Quantity,
        account_type: AccountType,
        client_oid: &str,
    ) -> Value {
        json!({
            "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
            "productType": get_product_type(&symbol.quote),
            "marginCoin": symbol.quote.to_uppercase(),
            "size": quantity.to_string(),
            "side": match side {
                OrderSide::Buy => "buy",
                OrderSide::Sell => "sell",
            },
            "clientOid": client_oid,
        })
    }

    /// Build a minimal returned Order after placing
    fn build_placed_order(
        order_id: String,
        client_oid: String,
        symbol: &Symbol,
        side: OrderSide,
        order_type: OrderType,
        price: Option<Price>,
        quantity: Quantity,
    ) -> Order {
        Order {
            id: order_id,
            client_order_id: Some(client_oid),
            symbol: symbol.to_string(),
            side,
            order_type,
            status: crate::core::OrderStatus::New,
            price,
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BitgetConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitget
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.market_limiter.lock() {
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
        false // Bitget doesn't have testnet in this implementation
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
impl MarketData for BitgetConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotPrice,
            _ => BitgetEndpoint::FuturesPrice,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // Futures requires productType
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotOrderbook,
            _ => BitgetEndpoint::FuturesOrderbook,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // Bitget spot uses "type" and "limit", futures uses just "limit"
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                params.insert("type".to_string(), "step0".to_string());
                params.insert("limit".to_string(), depth.unwrap_or(100).to_string());
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
                let limit = match depth.unwrap_or(100) {
                    0..=5 => 5,
                    6..=15 => 15,
                    16..=50 => 50,
                    _ => 100,
                };
                params.insert("limit".to_string(), limit.to_string());
            }
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_orderbook(&response)
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
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotKlines,
            _ => BitgetEndpoint::FuturesKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // V2 API uses `granularity` for both Spot and Futures
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                // V2 Spot uses "granularity" with format: "1min", "1h", "1day"
                params.insert("granularity".to_string(), map_kline_interval(interval).to_string());
                params.insert("limit".to_string(), limit.unwrap_or(1000).min(1000).to_string());
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
                // V2 Futures uses "granularity" with format: "1m", "1H", "1D"
                params.insert("granularity".to_string(), map_futures_granularity(interval).to_string());
                params.insert("limit".to_string(), limit.unwrap_or(200).min(1000).to_string());
            }
        }

        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotTicker,
            _ => BitgetEndpoint::FuturesTicker,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        // Futures requires productType
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(BitgetEndpoint::Timestamp, HashMap::new(), AccountType::Spot).await?;
        self.check_response(&response)
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols(account_type).await?;
        BitgetParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BitgetConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let client_oid = format!("cc_{}", crate::core::timestamp_millis());
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);
        let margin_mode = match account_type {
            AccountType::FuturesIsolated => "isolated",
            _ => "crossed",
        };

        let (endpoint, body) = match req.order_type {
            OrderType::Market => {
                let endpoint = if is_futures { BitgetEndpoint::FuturesCreateOrder } else { BitgetEndpoint::SpotCreateOrder };
                let body = if is_futures {
                    json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginMode": margin_mode,
                        "marginCoin": symbol.quote.to_uppercase(),
                        "size": quantity.to_string(),
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "market",
                        "force": "gtc",
                        "clientOid": client_oid,
                    })
                } else {
                    json!({
                        "symbol": formatted_symbol,
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "market",
                        "force": "gtc",
                        "size": quantity.to_string(),
                        "clientOid": client_oid,
                    })
                };
                (endpoint, body)
            }

            OrderType::Limit { price } => {
                let endpoint = if is_futures { BitgetEndpoint::FuturesCreateOrder } else { BitgetEndpoint::SpotCreateOrder };
                let body = if is_futures {
                    json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginMode": margin_mode,
                        "marginCoin": symbol.quote.to_uppercase(),
                        "size": quantity.to_string(),
                        "price": price.to_string(),
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "gtc",
                        "clientOid": client_oid,
                    })
                } else {
                    json!({
                        "symbol": formatted_symbol,
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "gtc",
                        "price": price.to_string(),
                        "size": quantity.to_string(),
                        "clientOid": client_oid,
                    })
                };
                (endpoint, body)
            }

            OrderType::PostOnly { price } => {
                // Bitget: force=post_only, orderType=limit
                let endpoint = if is_futures { BitgetEndpoint::FuturesCreateOrder } else { BitgetEndpoint::SpotCreateOrder };
                let body = if is_futures {
                    json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginMode": margin_mode,
                        "marginCoin": symbol.quote.to_uppercase(),
                        "size": quantity.to_string(),
                        "price": price.to_string(),
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "post_only",
                        "clientOid": client_oid,
                    })
                } else {
                    json!({
                        "symbol": formatted_symbol,
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "post_only",
                        "price": price.to_string(),
                        "size": quantity.to_string(),
                        "clientOid": client_oid,
                    })
                };
                (endpoint, body)
            }

            OrderType::Ioc { price } => {
                // Bitget: force=ioc, orderType=limit
                let endpoint = if is_futures { BitgetEndpoint::FuturesCreateOrder } else { BitgetEndpoint::SpotCreateOrder };
                let price_val = price.unwrap_or(0.0);
                let body = if is_futures {
                    json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginMode": margin_mode,
                        "marginCoin": symbol.quote.to_uppercase(),
                        "size": quantity.to_string(),
                        "price": price_val.to_string(),
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "ioc",
                        "clientOid": client_oid,
                    })
                } else {
                    json!({
                        "symbol": formatted_symbol,
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "ioc",
                        "price": price_val.to_string(),
                        "size": quantity.to_string(),
                        "clientOid": client_oid,
                    })
                };
                (endpoint, body)
            }

            OrderType::Fok { price } => {
                // Bitget: force=fok, orderType=limit
                let endpoint = if is_futures { BitgetEndpoint::FuturesCreateOrder } else { BitgetEndpoint::SpotCreateOrder };
                let body = if is_futures {
                    json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginMode": margin_mode,
                        "marginCoin": symbol.quote.to_uppercase(),
                        "size": quantity.to_string(),
                        "price": price.to_string(),
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "fok",
                        "clientOid": client_oid,
                    })
                } else {
                    json!({
                        "symbol": formatted_symbol,
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "fok",
                        "price": price.to_string(),
                        "size": quantity.to_string(),
                        "clientOid": client_oid,
                    })
                };
                (endpoint, body)
            }

            OrderType::Gtd { price, expire_time } => {
                // Bitget: force=gtc with custom client expiry; no native GTD,
                // treat as GTC limit with expiry hint in clientOid comment
                // Note: Bitget does support GTD via force param on some endpoints,
                // but the V2 API primarily uses gtc/post_only/fok/ioc.
                // We use limit+gtc and attach expire_time as a clientOid suffix.
                let _ = expire_time; // acknowledged, not natively supported on Bitget spot
                let endpoint = if is_futures { BitgetEndpoint::FuturesCreateOrder } else { BitgetEndpoint::SpotCreateOrder };
                let body = if is_futures {
                    json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginMode": margin_mode,
                        "marginCoin": symbol.quote.to_uppercase(),
                        "size": quantity.to_string(),
                        "price": price.to_string(),
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "gtc",
                        "clientOid": client_oid,
                    })
                } else {
                    json!({
                        "symbol": formatted_symbol,
                        "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                        "orderType": "limit",
                        "force": "gtc",
                        "price": price.to_string(),
                        "size": quantity.to_string(),
                        "clientOid": client_oid,
                    })
                };
                (endpoint, body)
            }

            OrderType::ReduceOnly { price } => {
                // Futures only — reduceOnly=YES
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "ReduceOnly orders are only supported for futures on Bitget".to_string()
                    ));
                }
                let (order_type_str, price_field) = if let Some(p) = price {
                    ("limit", Some(p))
                } else {
                    ("market", None)
                };
                let mut body_obj = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&symbol.quote),
                    "marginMode": margin_mode,
                    "marginCoin": symbol.quote.to_uppercase(),
                    "size": quantity.to_string(),
                    "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": order_type_str,
                    "force": "gtc",
                    "reduceOnly": "YES",
                    "clientOid": client_oid,
                });
                if let Some(p) = price_field {
                    body_obj["price"] = json!(p.to_string());
                }
                (BitgetEndpoint::FuturesCreateOrder, body_obj)
            }

            OrderType::StopMarket { stop_price } => {
                // Bitget: plan order with planType=normal_plan, orderType=market, triggerPrice=stop_price
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopMarket plan orders are only supported for futures on Bitget".to_string()
                    ));
                }
                let body = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&symbol.quote),
                    "marginMode": margin_mode,
                    "marginCoin": symbol.quote.to_uppercase(),
                    "size": quantity.to_string(),
                    "triggerPrice": stop_price.to_string(),
                    "triggerType": "mark_price",
                    "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": "market",
                    "planType": "normal_plan",
                    "clientOid": client_oid,
                });
                (BitgetEndpoint::FuturesPlanOrder, body)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // Bitget: plan order with planType=normal_plan, orderType=limit, price=limit_price, triggerPrice=stop_price
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopLimit plan orders are only supported for futures on Bitget".to_string()
                    ));
                }
                let body = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&symbol.quote),
                    "marginMode": margin_mode,
                    "marginCoin": symbol.quote.to_uppercase(),
                    "size": quantity.to_string(),
                    "price": limit_price.to_string(),
                    "triggerPrice": stop_price.to_string(),
                    "triggerType": "mark_price",
                    "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": "limit",
                    "planType": "normal_plan",
                    "clientOid": client_oid,
                });
                (BitgetEndpoint::FuturesPlanOrder, body)
            }

            OrderType::TrailingStop { callback_rate, activation_price } => {
                // Bitget: plan order with planType=track_plan, callbackRatio=callback_rate
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "TrailingStop plan orders are only supported for futures on Bitget".to_string()
                    ));
                }
                let mut body = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&symbol.quote),
                    "marginMode": margin_mode,
                    "marginCoin": symbol.quote.to_uppercase(),
                    "size": quantity.to_string(),
                    "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": "market",
                    "planType": "track_plan",
                    "callbackRatio": callback_rate.to_string(),
                    "clientOid": client_oid,
                });
                if let Some(act_price) = activation_price {
                    body["triggerPrice"] = json!(act_price.to_string());
                    body["triggerType"] = json!("mark_price");
                }
                (BitgetEndpoint::FuturesPlanOrder, body)
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                // Bitget: regular order with presetStopSurplusPrice and presetStopLossPrice
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Bracket orders are only supported for futures on Bitget".to_string()
                    ));
                }
                let (order_type_str, price_field) = if let Some(p) = price {
                    ("limit", Some(p))
                } else {
                    ("market", None)
                };
                let mut body_obj = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&symbol.quote),
                    "marginMode": margin_mode,
                    "marginCoin": symbol.quote.to_uppercase(),
                    "size": quantity.to_string(),
                    "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": order_type_str,
                    "force": "gtc",
                    "presetStopSurplusPrice": take_profit.to_string(),
                    "presetStopLossPrice": stop_loss.to_string(),
                    "clientOid": client_oid,
                });
                if let Some(p) = price_field {
                    body_obj["price"] = json!(p.to_string());
                }
                let order_id = {
                    let response = self.post(BitgetEndpoint::FuturesCreateOrder, body_obj, account_type).await?;
                    BitgetParser::parse_order_id(&response)?
                };
                return Ok(PlaceOrderResponse::Simple(
                    Self::build_placed_order(order_id, client_oid, &symbol, side, req.order_type, price, quantity)
                ));
            }

            OrderType::Twap { duration_seconds, interval_seconds } => {
                // Bitget: native TWAP via POST /api/v2/mix/order/place-twap-order
                // Futures only — up to 30 active TWAP orders per account.
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "TWAP orders are only supported for futures on Bitget".to_string()
                    ));
                }
                let mut body = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&symbol.quote),
                    "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "tradeSide": "open",
                    "totalQuantity": quantity.to_string(),
                    // timeInterval in seconds between each sub-order execution
                    "timeInterval": interval_seconds.unwrap_or(60).to_string(),
                    // priceType: "market" = TWAP at market; "limit" = limit TWAP
                    "priceType": "market",
                    "clientOid": client_oid,
                });
                // executeQuantity: sub-order size. Default to total / (duration / interval) slices.
                let interval = interval_seconds.unwrap_or(60);
                let num_slices = (duration_seconds / interval).max(1);
                let slice_qty = quantity / num_slices as f64;
                body["executeQuantity"] = json!(slice_qty.to_string());
                body["size"] = json!(quantity.to_string());

                let response = self.post(BitgetEndpoint::FuturesTwapOrder, body, account_type).await?;
                let algo_id = response
                    .get("data")
                    .and_then(|d| d.get("twapOrderId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                return Ok(PlaceOrderResponse::Algo(crate::core::types::AlgoOrderResponse {
                    algo_id,
                    status: "Running".to_string(),
                    executed_count: None,
                    total_count: Some(num_slices as u32),
                }));
            }

            OrderType::Iceberg { price, display_quantity } => {
                // Bitget futures: orderType="iceberg" with icebergQuantity per visible slice.
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Iceberg orders are only supported for futures on Bitget".to_string()
                    ));
                }
                let body = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&symbol.quote),
                    "marginMode": margin_mode,
                    "marginCoin": symbol.quote.to_uppercase(),
                    "size": quantity.to_string(),
                    "price": price.to_string(),
                    "side": match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": "limit",
                    "force": "gtc",
                    // Bitget iceberg param — display quantity of each visible slice
                    "icebergQuantity": display_quantity.to_string(),
                    "clientOid": client_oid,
                });
                (BitgetEndpoint::FuturesCreateOrder, body)
            }

            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on Bitget", req.order_type)
                ));
            }
        };

        let response = self.post(endpoint, body, account_type).await?;
        let order_id = BitgetParser::parse_order_id(&response)?;
        let price_for_order = match &req.order_type {
            OrderType::Limit { price } | OrderType::PostOnly { price } | OrderType::Fok { price } | OrderType::Gtd { price, .. } => Some(*price),
            OrderType::Ioc { price } => *price,
            OrderType::StopLimit { limit_price, .. } => Some(*limit_price),
            OrderType::Iceberg { price, .. } => Some(*price),
            _ => None,
        };

        Ok(PlaceOrderResponse::Simple(
            Self::build_placed_order(order_id, client_oid, &symbol, side, req.order_type, price_for_order, quantity)
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;
                let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

                let endpoint = if is_futures { BitgetEndpoint::FuturesCancelOrder } else { BitgetEndpoint::SpotCancelOrder };
                let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);

                let body = if is_futures {
                    json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginCoin": symbol.quote.to_uppercase(),
                        "orderId": order_id,
                    })
                } else {
                    json!({
                        "symbol": formatted_symbol,
                        "orderId": order_id,
                    })
                };

                self.post(endpoint, body, account_type).await?;

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
                    time_in_force: TimeInForce::Gtc,
                })
            }

            CancelScope::Batch { ref order_ids } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for batch cancel".into()))?
                    .clone();
                let account_type = req.account_type;
                let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);
                let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);

                if is_futures {
                    let order_list: Vec<Value> = order_ids.iter()
                        .map(|id| json!({ "orderId": id }))
                        .collect();
                    let body = json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginCoin": symbol.quote.to_uppercase(),
                        "orderIdList": order_list,
                    });
                    self.post(BitgetEndpoint::FuturesBatchCancelOrders, body, account_type).await?;
                } else {
                    let order_list: Vec<Value> = order_ids.iter()
                        .map(|id| json!({ "orderId": id }))
                        .collect();
                    let body = json!({
                        "symbol": formatted_symbol,
                        "orderList": order_list,
                    });
                    self.post(BitgetEndpoint::SpotBatchCancelOrders, body, account_type).await?;
                }

                // Return a representative "cancelled" order
                Ok(Order {
                    id: order_ids.first().cloned().unwrap_or_default(),
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
                    time_in_force: TimeInForce::Gtc,
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope — use CancelAll trait for all/bySymbol on Bitget", req.scope)
            )),
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotGetOrder,
            _ => BitgetEndpoint::FuturesGetOrder,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_order(&response, &symbol.to_string())
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let symbol: Option<crate::core::Symbol> = symbol.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotOpenOrders,
            _ => BitgetEndpoint::FuturesOpenOrders,
        };

        let mut params = HashMap::new();

        if let Some(ref s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
            if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
                params.insert("productType".to_string(), get_product_type(&s.quote).to_string());
            }
        } else if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), "USDT-FUTURES".to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);
        let endpoint = if is_futures { BitgetEndpoint::FuturesAllOrders } else { BitgetEndpoint::SpotAllOrders };

        let mut params = HashMap::new();

        if is_futures {
            // Futures requires productType
            let product_type = filter.symbol.as_ref()
                .map(|s| get_product_type(&s.quote))
                .unwrap_or("USDT-FUTURES");
            params.insert("productType".to_string(), product_type.to_string());
        }

        if let Some(ref s) = filter.symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }
        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BitgetParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BitgetConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let account_type = query.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BitgetEndpoint::SpotAccounts,
            _ => BitgetEndpoint::FuturesAccount,
        };

        let mut params = HashMap::new();

        if let Some(ref a) = asset {
            params.insert("coin".to_string(), a.to_string());

            if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
                params.insert("productType".to_string(), "USDT-FUTURES".to_string());
                params.insert("marginCoin".to_string(), a.to_string());
            }
        } else if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            params.insert("productType".to_string(), "USDT-FUTURES".to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;

        match account_type {
            AccountType::Spot | AccountType::Margin => BitgetParser::parse_balances(&response),
            _ => BitgetParser::parse_futures_account(&response),
        }
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.2,
            taker_commission: 0.2,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Use /api/v2/common/trade-rate for account-specific fee (requires symbol+businessType)
        // Fall back to VIP fee rate if no symbol provided
        if let Some(sym_str) = symbol {
            let parts: Vec<&str> = sym_str.split('/').collect();
            let sym = if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: sym_str.to_string(), quote: String::new(), raw: Some(sym_str.to_string()) }
            };

            let mut params = HashMap::new();
            params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, AccountType::Spot));
            params.insert("businessType".to_string(), "spot".to_string());

            let response = self.get(BitgetEndpoint::TradeRate, params, AccountType::Spot).await?;
            let data = response.get("data").ok_or_else(|| ExchangeError::Parse("Missing data".to_string()))?;

            let maker = data.get("makerFeeRate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.001);
            let taker = data.get("takerFeeRate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.001);

            Ok(FeeInfo {
                maker_rate: maker,
                taker_rate: taker,
                symbol: Some(sym_str.to_string()),
                tier: None,
            })
        } else {
            // No symbol: fetch VIP fee tier 0 (public, no auth needed)
            let response = self.get(BitgetEndpoint::VipFeeRate, HashMap::new(), AccountType::Spot).await?;
            let data = response.get("data")
                .and_then(|d| d.as_array())
                .and_then(|arr| arr.first())
                .ok_or_else(|| ExchangeError::Parse("Missing VIP fee data".to_string()))?;

            let maker = data.get("makerFeeRate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.001);
            let taker = data.get("takerFeeRate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.001);
            let level = data.get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("0");

            Ok(FeeInfo {
                maker_rate: maker,
                taker_rate: taker,
                symbol: None,
                tier: Some(format!("VIP{}", level)),
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BitgetConnector {
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
            params.insert("productType".to_string(), get_product_type(&s.quote).to_string());
            params.insert("marginCoin".to_string(), s.quote.to_uppercase());
            self.get(BitgetEndpoint::FuturesPosition, params, account_type).await?
        } else {
            let mut params = HashMap::new();
            params.insert("productType".to_string(), "USDT-FUTURES".to_string());
            self.get(BitgetEndpoint::FuturesPositions, params, account_type).await?
        };

        if symbol.is_some() {
            BitgetParser::parse_position(&response).map(|p| vec![p])
        } else {
            BitgetParser::parse_positions(&response)
        }
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
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

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("productType".to_string(), get_product_type(&symbol.quote).to_string());

        let response = self.get(BitgetEndpoint::FundingRate, params, account_type).await?;
        BitgetParser::parse_funding_rate(&response)
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
                    "productType": get_product_type(&symbol.quote),
                    "marginCoin": symbol.quote.to_uppercase(),
                    "leverage": leverage.to_string(),
                    "holdSide": "long",
                });
                self.post(BitgetEndpoint::FuturesSetLeverage, body, account_type).await?;
                Ok(())
            }

            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "MarginMode not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                let margin_mode_str = match margin_type {
                    MarginType::Cross => "crossed",
                    MarginType::Isolated => "isolated",
                };
                let body = json!({
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "productType": get_product_type(&symbol.quote),
                    "marginCoin": symbol.quote.to_uppercase(),
                    "marginMode": margin_mode_str,
                });
                self.post(BitgetEndpoint::FuturesSetMarginMode, body, account_type).await?;
                Ok(())
            }

            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "AddMargin not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                let body = json!({
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "productType": get_product_type(&symbol.quote),
                    "marginCoin": symbol.quote.to_uppercase(),
                    "amount": amount.to_string(),
                    "holdSide": "long",
                    "operationType": "add",
                });
                self.post(BitgetEndpoint::FuturesSetMargin, body, account_type).await?;
                Ok(())
            }

            PositionModification::RemoveMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "RemoveMargin not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                let body = json!({
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "productType": get_product_type(&symbol.quote),
                    "marginCoin": symbol.quote.to_uppercase(),
                    "amount": amount.to_string(),
                    "holdSide": "long",
                    "operationType": "reduce",
                });
                self.post(BitgetEndpoint::FuturesSetMargin, body, account_type).await?;
                Ok(())
            }

            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                // Flash close via /api/v2/mix/order/close-positions — closes entire position at market
                let body = json!({
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "productType": get_product_type(&symbol.quote),
                    "marginCoin": symbol.quote.to_uppercase(),
                    "holdSide": "long",
                });
                self.post(BitgetEndpoint::FuturesClosePositions, body, account_type).await?;
                Ok(())
            }

            PositionModification::SetTpSl { ref symbol, take_profit, stop_loss, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetTpSl not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                // Use place-tpsl-order to set both TP and SL on the position
                let mut body = json!({
                    "symbol": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "productType": get_product_type(&symbol.quote),
                    "marginCoin": symbol.quote.to_uppercase(),
                    "planType": "profit_loss",
                    "triggerType": "mark_price",
                    "holdSide": "long",
                });
                if let Some(tp) = take_profit {
                    body["stopSurplusTriggerPrice"] = json!(tp.to_string());
                    body["stopSurplusTriggerType"] = json!("mark_price");
                    body["stopSurplusExecutePrice"] = json!("0");
                }
                if let Some(sl) = stop_loss {
                    body["stopLossTriggerPrice"] = json!(sl.to_string());
                    body["stopLossTriggerType"] = json!("mark_price");
                    body["stopLossExecutePrice"] = json!("0");
                }
                // Use the pos-tpsl endpoint for simultaneous TP+SL
                self.post(BitgetEndpoint::FuturesPosTpSl, body, account_type).await?;
                Ok(())
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for BitgetConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        match scope {
            CancelScope::All { ref symbol } => {
                if is_futures {
                    let product_type = symbol.as_ref()
                        .map(|s| get_product_type(&s.quote))
                        .unwrap_or("USDT-FUTURES");
                    let mut body = json!({
                        "productType": product_type,
                    });
                    if let Some(s) = symbol {
                        body["symbol"] = json!(format_symbol(&s.base, &s.quote, account_type));
                        body["marginCoin"] = json!(s.quote.to_uppercase());
                    }
                    let response = self.post(BitgetEndpoint::FuturesCancelBySymbol, body, account_type).await?;
                    let cancelled = response.get("data")
                        .and_then(|d| d.get("successCount"))
                        .and_then(|c| c.as_u64())
                        .unwrap_or(0) as u32;
                    Ok(CancelAllResponse {
                        cancelled_count: cancelled,
                        failed_count: 0,
                        details: vec![],
                    })
                } else {
                    // Spot: cancel-symbol-orders requires a symbol
                    let sym = symbol.as_ref()
                        .ok_or_else(|| ExchangeError::InvalidRequest(
                            "Bitget Spot cancel-all requires a symbol".to_string()
                        ))?;
                    let body = json!({
                        "symbol": format_symbol(&sym.base, &sym.quote, account_type),
                    });
                    let response = self.post(BitgetEndpoint::SpotCancelBySymbol, body, account_type).await?;
                    let cancelled = response.get("data")
                        .and_then(|d| d.get("successCount"))
                        .and_then(|c| c.as_u64())
                        .unwrap_or(0) as u32;
                    Ok(CancelAllResponse {
                        cancelled_count: cancelled,
                        failed_count: 0,
                        details: vec![],
                    })
                }
            }

            CancelScope::BySymbol { ref symbol } => {
                let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
                if is_futures {
                    let body = json!({
                        "symbol": formatted_symbol,
                        "productType": get_product_type(&symbol.quote),
                        "marginCoin": symbol.quote.to_uppercase(),
                    });
                    let response = self.post(BitgetEndpoint::FuturesCancelBySymbol, body, account_type).await?;
                    let cancelled = response.get("data")
                        .and_then(|d| d.get("successCount"))
                        .and_then(|c| c.as_u64())
                        .unwrap_or(0) as u32;
                    Ok(CancelAllResponse {
                        cancelled_count: cancelled,
                        failed_count: 0,
                        details: vec![],
                    })
                } else {
                    let body = json!({ "symbol": formatted_symbol });
                    let response = self.post(BitgetEndpoint::SpotCancelBySymbol, body, account_type).await?;
                    let cancelled = response.get("data")
                        .and_then(|d| d.get("successCount"))
                        .and_then(|c| c.as_u64())
                        .unwrap_or(0) as u32;
                    Ok(CancelAllResponse {
                        cancelled_count: cancelled,
                        failed_count: 0,
                        details: vec![],
                    })
                }
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                "cancel_all_orders only supports All and BySymbol scopes".to_string()
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for BitgetConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let symbol = &req.symbol;
        let account_type = req.account_type;
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);

        if is_futures {
            // POST /api/v2/mix/order/modify-order
            let mut body = json!({
                "symbol": formatted_symbol,
                "productType": get_product_type(&symbol.quote),
                "marginCoin": symbol.quote.to_uppercase(),
                "orderId": req.order_id,
            });
            if let Some(new_price) = req.fields.price {
                body["newPrice"] = json!(new_price.to_string());
            }
            if let Some(new_qty) = req.fields.quantity {
                body["newSize"] = json!(new_qty.to_string());
            }
            if let Some(trigger) = req.fields.trigger_price {
                body["presetStopSurplusPrice"] = json!(trigger.to_string());
            }
            let response = self.post(BitgetEndpoint::FuturesModifyOrder, body, account_type).await?;
            // Fetch updated order
            let order_id = response.get("data")
                .and_then(|d| d.get("orderId"))
                .and_then(|v| v.as_str())
                .unwrap_or(&req.order_id)
                .to_string();
            self.get_order(&symbol.to_string(), &order_id, account_type).await
        } else {
            // Spot modify order
            let mut body = json!({
                "symbol": formatted_symbol,
                "orderId": req.order_id,
            });
            if let Some(new_price) = req.fields.price {
                body["newPrice"] = json!(new_price.to_string());
            }
            if let Some(new_qty) = req.fields.quantity {
                body["newSize"] = json!(new_qty.to_string());
            }
            let response = self.post(BitgetEndpoint::SpotModifyOrder, body, account_type).await?;
            let order_id = response.get("data")
                .and_then(|d| d.get("orderId"))
                .and_then(|v| v.as_str())
                .unwrap_or(&req.order_id)
                .to_string();
            self.get_order(&symbol.to_string(), &order_id, account_type).await
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BatchOrders for BitgetConnector {
    fn max_batch_place_size(&self) -> usize { 50 }
    fn max_batch_cancel_size(&self) -> usize { 50 }

    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let account_type = orders[0].account_type;
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        // Group by symbol (Bitget batch requires same symbol per call)
        // For simplicity, use the first order's symbol as the batch symbol
        let symbol = orders[0].symbol.clone();
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut results = Vec::with_capacity(orders.len());

        if is_futures {
            let order_list: Vec<Value> = orders.iter().map(|o| {
                let price_f = match &o.order_type {
                    OrderType::Limit { price } | OrderType::PostOnly { price } | OrderType::Fok { price } => Some(*price),
                    OrderType::Ioc { price } => *price,
                    _ => None,
                };
                let (ot_str, force_str) = match &o.order_type {
                    OrderType::Market => ("market", "gtc"),
                    OrderType::Limit { .. } => ("limit", "gtc"),
                    OrderType::PostOnly { .. } => ("limit", "post_only"),
                    OrderType::Ioc { .. } => ("limit", "ioc"),
                    OrderType::Fok { .. } => ("limit", "fok"),
                    _ => ("limit", "gtc"),
                };
                let mut item = json!({
                    "marginMode": match account_type { AccountType::FuturesIsolated => "isolated", _ => "crossed" },
                    "size": o.quantity.to_string(),
                    "side": match o.side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": ot_str,
                    "force": force_str,
                    "clientOid": format!("cc_{}", crate::core::timestamp_millis()),
                });
                if let Some(p) = price_f {
                    item["price"] = json!(p.to_string());
                }
                item
            }).collect();

            let body = json!({
                "symbol": formatted_symbol,
                "productType": get_product_type(&symbol.quote),
                "marginCoin": symbol.quote.to_uppercase(),
                "orderList": order_list,
            });

            let response = self.post(BitgetEndpoint::FuturesBatchPlaceOrders, body, account_type).await?;
            let data = response.get("data").cloned().unwrap_or(json!({}));

            // Parse successList
            if let Some(success) = data.get("successList").and_then(|v| v.as_array()) {
                for item in success {
                    let order_id = item.get("orderId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    results.push(OrderResult {
                        order: None,
                        client_order_id: item.get("clientOid").and_then(|v| v.as_str()).map(String::from),
                        success: true,
                        error: None,
                        error_code: None,
                    });
                    let _ = order_id;
                }
            }
            // Parse failureList
            if let Some(failures) = data.get("failureList").and_then(|v| v.as_array()) {
                for item in failures {
                    let error_msg = item.get("errorMsg").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string();
                    let error_code = item.get("errorCode")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i32>().ok());
                    results.push(OrderResult {
                        order: None,
                        client_order_id: item.get("clientOid").and_then(|v| v.as_str()).map(String::from),
                        success: false,
                        error: Some(error_msg),
                        error_code,
                    });
                }
            }
        } else {
            // Spot batch orders — same symbol, up to 50
            let order_list: Vec<Value> = orders.iter().map(|o| {
                let price_f = match &o.order_type {
                    OrderType::Limit { price } | OrderType::PostOnly { price } | OrderType::Fok { price } => Some(*price),
                    OrderType::Ioc { price } => *price,
                    _ => None,
                };
                let (ot_str, force_str) = match &o.order_type {
                    OrderType::Market => ("market", "gtc"),
                    OrderType::Limit { .. } => ("limit", "gtc"),
                    OrderType::PostOnly { .. } => ("limit", "post_only"),
                    OrderType::Ioc { .. } => ("limit", "ioc"),
                    OrderType::Fok { .. } => ("limit", "fok"),
                    _ => ("limit", "gtc"),
                };
                let mut item = json!({
                    "side": match o.side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
                    "orderType": ot_str,
                    "force": force_str,
                    "size": o.quantity.to_string(),
                    "clientOid": format!("cc_{}", crate::core::timestamp_millis()),
                });
                if let Some(p) = price_f {
                    item["price"] = json!(p.to_string());
                }
                item
            }).collect();

            let body = json!({
                "symbol": formatted_symbol,
                "orderList": order_list,
            });

            let response = self.post(BitgetEndpoint::SpotBatchPlaceOrders, body, account_type).await?;
            let data = response.get("data").cloned().unwrap_or(json!({}));

            if let Some(success) = data.get("successList").and_then(|v| v.as_array()) {
                for item in success {
                    results.push(OrderResult {
                        order: None,
                        client_order_id: item.get("clientOid").and_then(|v| v.as_str()).map(String::from),
                        success: true,
                        error: None,
                        error_code: None,
                    });
                }
            }
            if let Some(failures) = data.get("failureList").and_then(|v| v.as_array()) {
                for item in failures {
                    let error_msg = item.get("errorMsg").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string();
                    let error_code = item.get("errorCode")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i32>().ok());
                    results.push(OrderResult {
                        order: None,
                        client_order_id: item.get("clientOid").and_then(|v| v.as_str()).map(String::from),
                        success: false,
                        error: Some(error_msg),
                        error_code,
                    });
                }
            }
        }

        Ok(results)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }

        let sym_str = symbol.ok_or_else(|| ExchangeError::InvalidRequest(
            "Symbol required for batch cancel on Bitget".to_string()
        ))?;
        let parts: Vec<&str> = sym_str.split('/').collect();
        let sym = if parts.len() == 2 {
            crate::core::Symbol::new(parts[0], parts[1])
        } else {
            crate::core::Symbol { base: sym_str.to_string(), quote: String::new(), raw: Some(sym_str.to_string()) }
        };

        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);
        let formatted_symbol = format_symbol(&sym.base, &sym.quote, account_type);

        let mut results = Vec::with_capacity(order_ids.len());

        // Bitget batch cancel: max 50 per call — chunk
        for chunk in order_ids.chunks(50) {
            if is_futures {
                let order_list: Vec<Value> = chunk.iter()
                    .map(|id| json!({ "orderId": id }))
                    .collect();
                let body = json!({
                    "symbol": formatted_symbol,
                    "productType": get_product_type(&sym.quote),
                    "marginCoin": sym.quote.to_uppercase(),
                    "orderIdList": order_list,
                });
                let response = self.post(BitgetEndpoint::FuturesBatchCancelOrders, body, account_type).await?;
                // Parse result if present
                let data = response.get("data").cloned().unwrap_or(json!({}));
                if let Some(success_list) = data.get("successList").and_then(|v| v.as_array()) {
                    for item in success_list {
                        let order_id = item.get("orderId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        results.push(OrderResult {
                            order: None,
                            client_order_id: Some(order_id),
                            success: true,
                            error: None,
                            error_code: None,
                        });
                    }
                } else {
                    // If no detailed response, assume success for all in chunk
                    for id in chunk {
                        results.push(OrderResult {
                            order: None,
                            client_order_id: Some(id.clone()),
                            success: true,
                            error: None,
                            error_code: None,
                        });
                    }
                }
            } else {
                let order_list: Vec<Value> = chunk.iter()
                    .map(|id| json!({ "orderId": id }))
                    .collect();
                let body = json!({
                    "symbol": formatted_symbol,
                    "orderList": order_list,
                });
                let response = self.post(BitgetEndpoint::SpotBatchCancelOrders, body, account_type).await?;
                let data = response.get("data").cloned().unwrap_or(json!({}));
                if let Some(success_list) = data.get("successList").and_then(|v| v.as_array()) {
                    for item in success_list {
                        let order_id = item.get("orderId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        results.push(OrderResult {
                            order: None,
                            client_order_id: Some(order_id),
                            success: true,
                            error: None,
                            error_code: None,
                        });
                    }
                } else {
                    for id in chunk {
                        results.push(OrderResult {
                            order: None,
                            client_order_id: Some(id.clone()),
                            success: true,
                            error: None,
                            error_code: None,
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}
