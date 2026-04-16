//! # MEXC Connector
//!
//! Implementation of all core traits for MEXC Spot API.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//!
//! ## Extended Methods
//! Additional MEXC-specific methods as struct methods.

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
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    UserTrade, UserTradeFilter,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account,
};
use crate::core::{CancelAll, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts};
use crate::core::types::{
    ConnectorStats, CancelAllResponse, OrderResult,
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord, FundsHistoryFilter, FundsRecordType,
    SubAccountOperation, SubAccountResult, SubAccount,
};
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{MexcUrls, MexcEndpoint, format_symbol, map_kline_interval};
use super::auth::MexcAuth;
use super::parser::MexcParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// MEXC connector
pub struct MexcConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<MexcAuth>,
    /// Rate limiter (500 weight per 10 seconds)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: crate::core::utils::precision::PrecisionCache,
}

impl MexcConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials.as_ref().map(MexcAuth::new);

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = MexcUrls::base_url();
            let url = format!("{}/api/v3/time", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(server_time_ms) = response.get("serverTime")
                    .and_then(|t| t.as_i64())
                {
                    if let Some(ref mut a) = auth {
                        a.sync_time(server_time_ms);
                    }
                }
            }
        }

        // Initialize rate limiter: 500 weight per 10 seconds (MEXC Spot)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(500, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            rate_limiter,
            precision: crate::core::utils::precision::PrecisionCache::new(),
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
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(1) {
                    return;
                }
                limiter.time_until_ready(1)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// Update rate limiter from MEXC response headers.
    ///
    /// MEXC reports: `X-MEXC-USED-WEIGHT-1M` = weight used in the last minute.
    fn update_weight_from_headers(&self, headers: &HeaderMap) {
        let used = headers
            .get("x-mexc-used-weight-1m")
            .or_else(|| headers.get("X-MEXC-USED-WEIGHT-1M"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());
        if let Some(used) = used {
            if let Ok(mut limiter) = self.rate_limiter.lock() {
                limiter.update_from_server(used);
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: MexcEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = if endpoint.is_futures() {
            MexcUrls::futures_base_url()
        } else {
            MexcUrls::base_url()
        };
        let path = endpoint.path();

        let (url, headers) = if endpoint.is_private() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            let (headers, signed_params) = auth.sign_request(params);

            let query_parts: Vec<String> = signed_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            let query_string = query_parts.join("&");

            let url = format!("{}{}?{}", base_url, path, query_string);
            (url, headers)
        } else {
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
            (url, HashMap::new())
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::check_error(&response)?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: MexcEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = MexcUrls::base_url();
        let path = endpoint.path();

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let (headers, signed_params) = auth.sign_request(params);

        let query_parts: Vec<String> = signed_params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_string = query_parts.join("&");

        let url = format!("{}{}?{}", base_url, path, query_string);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &json!({}), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::check_error(&response)?;
        Ok(response)
    }

    /// DELETE request
    async fn delete(
        &self,
        endpoint: MexcEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = MexcUrls::base_url();
        let path = endpoint.path();

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let (headers, signed_params) = auth.sign_request(params);

        let query_parts: Vec<String> = signed_params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_string = query_parts.join("&");

        let url = format!("{}{}?{}", base_url, path, query_string);

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::check_error(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (MEXC-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get raw exchange information as Value
    pub async fn get_exchange_info_raw(&self) -> ExchangeResult<Value> {
        self.get(MexcEndpoint::ExchangeInfo, HashMap::new()).await
    }

    /// Cancel all orders for a symbol
    pub async fn cancel_all_orders(
        &self,
        symbol: &Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol, account_type));

        let response = self.delete(MexcEndpoint::CancelAllOrders, params).await?;

        // Response is array of cancelled orders
        MexcParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for MexcConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::MEXC
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_weight(), lim.max_weight())
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
        false // MEXC doesn't have testnet for spot
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::Margin,
            AccountType::FuturesCross,
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
impl MarketData for MexcConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

                let response = self.get(MexcEndpoint::TickerPrice, params).await?;

                let price = response["price"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or_else(|| ExchangeError::Parse("Invalid price".into()))?;

                Ok(price)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let ticker = self.get_ticker(symbol, account_type).await?;
                Ok(ticker.last_price)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

                if let Some(d) = depth {
                    params.insert("limit".to_string(), d.to_string());
                }

                let response = self.get(MexcEndpoint::Orderbook, params).await?;
                MexcParser::parse_orderbook(&response)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let base_url = MexcUrls::futures_base_url();
                let formatted_symbol = format_symbol(&symbol, account_type);
                let path = format!("/api/v1/contract/depth/{}", formatted_symbol);
                let url = format!("{}{}", base_url, path);

                self.rate_limit_wait().await;
                let response = self.http.get(&url, &HashMap::new()).await?;
                MexcParser::check_error(&response)?;

                let data = response.get("data")
                    .ok_or_else(|| ExchangeError::Parse("Missing data field in futures orderbook".into()))?;
                MexcParser::parse_orderbook_futures(data)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
                params.insert("interval".to_string(), map_kline_interval(interval).to_string());

                if let Some(l) = limit {
                    params.insert("limit".to_string(), l.min(1000).to_string());
                }

                if let Some(et) = end_time {
                    let interval_ms = interval_to_ms(interval);
                    let count = limit.unwrap_or(1000) as i64;
                    let st = et - count * interval_ms;
                    params.insert("startTime".to_string(), st.to_string());
                    params.insert("endTime".to_string(), et.to_string());
                }

                let response = self.get(MexcEndpoint::Klines, params).await?;
                MexcParser::parse_klines(&response)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let base_url = MexcUrls::futures_base_url();
                let formatted_symbol = format_symbol(&symbol, account_type);
                let path = format!("/api/v1/contract/kline/{}", formatted_symbol);

                let futures_interval = match interval {
                    "1m" => "Min1",
                    "5m" => "Min5",
                    "15m" => "Min15",
                    "30m" => "Min30",
                    "1h" => "Min60",
                    "4h" => "Hour4",
                    "8h" => "Hour8",
                    "1d" => "Day1",
                    "1w" => "Week1",
                    "1M" => "Month1",
                    _ => "Min60",
                };

                let mut params = HashMap::new();
                params.insert("interval".to_string(), futures_interval.to_string());

                if let Some(et) = end_time {
                    params.insert("endTime".to_string(), et.to_string());
                }

                let query = params.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");

                let url = format!("{}{}?{}", base_url, path, query);

                self.rate_limit_wait().await;
                let response = self.http.get(&url, &HashMap::new()).await?;
                MexcParser::check_error(&response)?;

                let klines_data = response.get("data")
                    .ok_or_else(|| ExchangeError::Parse("Missing data field in futures klines".into()))?;
                MexcParser::parse_klines_futures(klines_data)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

                let response = self.get(MexcEndpoint::Ticker24hr, params).await?;
                MexcParser::parse_ticker(&response)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let response = self.get(MexcEndpoint::FuturesTicker, HashMap::new()).await?;

                let formatted_symbol = format_symbol(&symbol, account_type);

                let data_array = response.get("data")
                    .or_else(|| response.as_array().map(|_| &response))
                    .ok_or_else(|| ExchangeError::Parse("Invalid futures ticker response".into()))?;

                let ticker_data = if let Some(arr) = data_array.as_array() {
                    arr.iter()
                        .find(|t| t["symbol"].as_str() == Some(&formatted_symbol))
                        .ok_or_else(|| ExchangeError::Parse(format!("Symbol {} not found", formatted_symbol)))?
                } else {
                    data_array
                };

                MexcParser::parse_ticker_futures(ticker_data)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _ = self.get(MexcEndpoint::Ping, HashMap::new()).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get(MexcEndpoint::ExchangeInfo, HashMap::new()).await?;
        let symbols = MexcParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&symbols);
        Ok(symbols)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for MexcConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let client_order_id = format!("cc_{}", crate::core::timestamp_millis());
        let symbol_str = format_symbol(&symbol, account_type);
        let qty_str = self.precision.qty(&symbol_str, quantity);

        let side_str = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        match req.order_type {
            OrderType::Market => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "MARKET".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
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
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
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

            OrderType::PostOnly { price } => {
                // MEXC: LIMIT_MAKER (post-only limit order)
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT_MAKER".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::PostOnly { price },
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

            OrderType::Ioc { price } => {
                // MEXC: LIMIT with timeInForce=IOC
                let price_val = price.unwrap_or(0.0);
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("timeInForce".to_string(), "IOC".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price_val));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Ioc { price },
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
                    time_in_force: crate::core::TimeInForce::Ioc,
                }))
            }

            OrderType::Fok { price } => {
                // MEXC: LIMIT with timeInForce=FOK
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("timeInForce".to_string(), "FOK".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Fok { price },
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
                    time_in_force: crate::core::TimeInForce::Fok,
                }))
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
                params.insert("orderId".to_string(), order_id.to_string());

                let response = self.delete(MexcEndpoint::CancelOrder, params).await?;
                MexcParser::parse_order(&response)
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported — use CancelAll trait", req.scope)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // MEXC: GET /api/v3/allOrders — requires symbol
        let symbol = filter.symbol
            .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for order history on MEXC".to_string()))?;

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let response = self.get(MexcEndpoint::AllOrders, params).await?;
        MexcParser::parse_orders(&response)
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

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(MexcEndpoint::QueryOrder, params).await?;
        MexcParser::parse_order(&response)
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

        let mut params = HashMap::new();

        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s, account_type));
        }

        let response = self.get(MexcEndpoint::OpenOrders, params).await?;
        MexcParser::parse_orders(&response)
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        // MEXC GET /api/v3/myTrades — symbol is required
        let symbol_str = filter.symbol
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Symbol required for get_user_trades on MEXC".to_string()
            ))?;

        // Parse raw symbol string into Symbol struct for format_symbol
        let sym = {
            let parts: Vec<&str> = symbol_str.splitn(2, '/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: symbol_str.clone(), quote: String::new(), raw: Some(symbol_str.clone()) }
            }
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&sym, crate::core::AccountType::Spot));

        if let Some(oid) = filter.order_id {
            params.insert("orderId".to_string(), oid);
        }
        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let response = self.get(MexcEndpoint::MyTrades, params).await?;
        MexcParser::parse_user_trades(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for MexcConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get(MexcEndpoint::Account, HashMap::new()).await?;
        MexcParser::parse_balance(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(MexcEndpoint::Account, HashMap::new()).await?;

        let balances = MexcParser::parse_balance(&response)?;

        let can_trade = response.get("canTrade")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let can_withdraw = response.get("canWithdraw")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let can_deposit = response.get("canDeposit")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let maker_commission = response.get("makerCommission")
            .and_then(|v| v.as_i64())
            .map(|c| c as f64 / 10000.0)
            .unwrap_or(0.002);

        let taker_commission = response.get("takerCommission")
            .and_then(|v| v.as_i64())
            .map(|c| c as f64 / 10000.0)
            .unwrap_or(0.002);

        Ok(AccountInfo {
            account_type,
            can_trade,
            can_withdraw,
            can_deposit,
            maker_commission,
            taker_commission,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // MEXC: GET /api/v3/tradeFee?symbol=BTCUSDT
        let mut params = HashMap::new();

        if let Some(sym) = symbol {
            let symbol_parts: Vec<&str> = sym.split('/').collect();
            let mexc_symbol = if symbol_parts.len() == 2 {
                let s = crate::core::Symbol::new(symbol_parts[0], symbol_parts[1]);
                format_symbol(&s, AccountType::Spot)
            } else {
                sym.to_uppercase().replace('/', "")
            };
            params.insert("symbol".to_string(), mexc_symbol);
        }

        let response = self.get(MexcEndpoint::TradeFee, params).await?;

        // Response: [{"symbol": "BTCUSDT", "makerCommission": "0.002", "takerCommission": "0.002"}]
        let fee_data = if let Some(arr) = response.as_array() {
            arr.first().cloned()
        } else {
            Some(response.clone())
        };

        let fee_data = fee_data
            .ok_or_else(|| ExchangeError::Parse("No fee data".to_string()))?;

        let maker_rate = fee_data.get("makerCommission")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.002);

        let taker_rate = fee_data.get("takerCommission")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.002);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for MexcConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match scope {
            CancelScope::All { symbol: Some(sym) } | CancelScope::BySymbol { symbol: sym } => {
                // MEXC requires symbol for cancel all
                let formatted_symbol = format_symbol(&sym, account_type);
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted_symbol);

                let response = self.delete(MexcEndpoint::CancelAllOrders, params).await?;

                // Response is array of cancelled orders
                let cancelled = if let Some(arr) = response.as_array() {
                    arr.len() as u32
                } else {
                    0
                };

                Ok(CancelAllResponse {
                    cancelled_count: cancelled,
                    failed_count: 0,
                    details: vec![],
                })
            }

            CancelScope::All { symbol: None } => {
                Err(ExchangeError::InvalidRequest(
                    "MEXC requires a symbol to cancel all orders — use BySymbol scope".to_string()
                ))
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported in cancel_all_orders", scope)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BatchOrders for MexcConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // MEXC: POST /api/v3/batchOrders — max 20 orders
        // Build batch order array
        let batch_orders: Vec<Value> = orders.iter().map(|req| {
            let o_sym = format_symbol(&req.symbol, req.account_type);
            let side_str = match req.side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            };
            let (order_type, price) = match &req.order_type {
                OrderType::Market => ("MARKET".to_string(), None),
                OrderType::Limit { price } => ("LIMIT".to_string(), Some(*price)),
                OrderType::PostOnly { price } => ("LIMIT_MAKER".to_string(), Some(*price)),
                _ => ("LIMIT".to_string(), None),
            };

            let mut order_obj = json!({
                "symbol": o_sym,
                "side": side_str,
                "type": order_type,
                "quantity": self.precision.qty(&o_sym, req.quantity),
            });

            if let Some(p) = price {
                order_obj["price"] = json!(self.precision.price(&o_sym, p));
            }

            order_obj
        }).collect();

        // MEXC batch orders use JSON body
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let params = HashMap::new();
        let (headers, _) = auth.sign_request(params);

        let base_url = MexcUrls::base_url();
        let path = MexcEndpoint::BatchOrders.path();
        let url = format!("{}{}", base_url, path);

        self.rate_limit_wait().await;
        let body = json!({ "batchOrders": batch_orders });
        let (response, _) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        MexcParser::check_error(&response)?;

        // Parse response — array of order results
        let results = if let Some(arr) = response.as_array() {
            arr.iter().map(|item| {
                let success = item.get("orderId").is_some();
                let order_id = item.get("orderId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                OrderResult {
                    order: order_id.map(|id| Order {
                        id,
                        client_order_id: None,
                        symbol: String::new(),
                        side: OrderSide::Buy,
                        order_type: OrderType::Market,
                        status: crate::core::OrderStatus::New,
                        price: None,
                        stop_price: None,
                        quantity: 0.0,
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: 0,
                        updated_at: None,
                        time_in_force: crate::core::TimeInForce::Gtc,
                    }),
                    client_order_id: None,
                    success,
                    error: if success { None } else {
                        item.get("msg").and_then(|v| v.as_str()).map(|s| s.to_string())
                    },
                    error_code: None,
                }
            }).collect()
        } else {
            vec![]
        };

        Ok(results)
    }

    async fn cancel_orders_batch(
        &self,
        _order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // MEXC doesn't have a true batch cancel — cancel one by one
        Err(ExchangeError::UnsupportedOperation(
            "MEXC does not support native batch cancel — use CancelAll for symbol-level cancel".to_string()
        ))
    }

    fn max_batch_place_size(&self) -> usize {
        20
    }

    fn max_batch_cancel_size(&self) -> usize {
        0 // Not supported
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountTransfers for MexcConnector {
    /// Transfer between Spot, Margin, and Futures accounts.
    ///
    /// POST /api/v3/capital/transfer
    /// Params: asset, amount, fromAccountType (SPOT/FUTURES/MARGIN), toAccountType
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        let from_type = account_type_to_mexc_str(req.from_account);
        let to_type = account_type_to_mexc_str(req.to_account);

        let mut params = HashMap::new();
        params.insert("asset".to_string(), req.asset.clone());
        params.insert("amount".to_string(), req.amount.to_string());
        params.insert("fromAccountType".to_string(), from_type.to_string());
        params.insert("toAccountType".to_string(), to_type.to_string());

        let response = self.post(MexcEndpoint::Transfer, params).await?;

        let tran_id = response["tranId"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| response["tranId"].as_i64().map(|n| n.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(TransferResponse {
            transfer_id: tran_id,
            status: "Successful".to_string(),
            asset: req.asset,
            amount: req.amount,
            timestamp: Some(crate::core::timestamp_millis() as i64),
        })
    }

    /// Get internal transfer history.
    ///
    /// GET /api/v3/capital/transfer
    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        let mut params = HashMap::new();

        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(MexcEndpoint::TransferHistory, params).await?;

        let rows = response.get("rows")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .cloned()
            .unwrap_or_default();

        let records = rows.iter().map(|item| {
            let tran_id = item["tranId"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| item["tranId"].as_i64().map(|n| n.to_string()))
                .unwrap_or_else(|| "unknown".to_string());

            let asset = item["asset"].as_str().unwrap_or("").to_string();
            let amount = item["amount"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| item["amount"].as_f64())
                .unwrap_or(0.0);
            let status = item["status"].as_str().unwrap_or("Unknown").to_string();
            let timestamp = item["timestamp"].as_i64()
                .or_else(|| item["createTime"].as_i64());

            TransferResponse {
                transfer_id: tran_id,
                status,
                asset,
                amount,
                timestamp,
            }
        }).collect();

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for MexcConnector {
    /// Get deposit address for an asset.
    ///
    /// GET /api/v3/capital/deposit/address
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let mut params = HashMap::new();
        params.insert("coin".to_string(), asset.to_uppercase());

        if let Some(net) = network {
            params.insert("network".to_string(), net.to_string());
        }

        let response = self.get(MexcEndpoint::DepositAddress, params).await?;

        let address = response["address"]
            .as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing deposit address".into()))?
            .to_string();

        let tag = response["tag"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let net = response["network"]
            .as_str()
            .or(network)
            .map(|s| s.to_string());

        Ok(DepositAddress {
            address,
            tag,
            network: net,
            asset: asset.to_uppercase(),
            created_at: None,
        })
    }

    /// Submit a withdrawal request.
    ///
    /// POST /api/v3/capital/withdraw
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let mut params = HashMap::new();
        params.insert("coin".to_string(), req.asset.clone());
        params.insert("address".to_string(), req.address.clone());
        params.insert("amount".to_string(), req.amount.to_string());

        if let Some(net) = &req.network {
            params.insert("network".to_string(), net.clone());
        }
        if let Some(memo) = &req.tag {
            params.insert("memo".to_string(), memo.clone());
        }

        let response = self.post(MexcEndpoint::Withdraw, params).await?;

        let withdraw_id = response["id"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| response["id"].as_i64().map(|n| n.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    /// Get deposit and/or withdrawal history.
    ///
    /// GET /api/v3/capital/deposit/hisrec  (deposits)
    /// GET /api/v3/capital/withdraw/history (withdrawals)
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let mut records = Vec::new();

        let mut params = HashMap::new();
        if let Some(asset) = &filter.asset {
            params.insert("coin".to_string(), asset.to_uppercase());
        }
        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        if matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both) {
            let response = self.get(MexcEndpoint::DepositHistory, params.clone()).await?;

            let items = response.as_array().cloned().unwrap_or_default();
            for item in &items {
                let id = item["id"].as_str().unwrap_or("").to_string();
                let asset = item["coin"].as_str().unwrap_or("").to_string();
                let amount = item["amount"]
                    .as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| item["amount"].as_f64())
                    .unwrap_or(0.0);
                let tx_hash = item["txId"].as_str().map(|s| s.to_string());
                let network = item["network"].as_str().map(|s| s.to_string());
                let status = item["status"].as_str().unwrap_or("Unknown").to_string();
                let timestamp = item["insertTime"].as_i64().unwrap_or(0);

                records.push(FundsRecord::Deposit {
                    id,
                    asset,
                    amount,
                    tx_hash,
                    network,
                    status,
                    timestamp,
                });
            }
        }

        if matches!(filter.record_type, FundsRecordType::Withdrawal | FundsRecordType::Both) {
            let response = self.get(MexcEndpoint::WithdrawHistory, params).await?;

            let items = response.as_array().cloned().unwrap_or_default();
            for item in &items {
                let id = item["id"].as_str().unwrap_or("").to_string();
                let asset = item["coin"].as_str().unwrap_or("").to_string();
                let amount = item["amount"]
                    .as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| item["amount"].as_f64())
                    .unwrap_or(0.0);
                let fee = item["transactionFee"]
                    .as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| item["transactionFee"].as_f64());
                let address = item["address"].as_str().unwrap_or("").to_string();
                let tag = item["addressTag"].as_str()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                let tx_hash = item["txId"].as_str().map(|s| s.to_string());
                let network = item["network"].as_str().map(|s| s.to_string());
                let status = item["status"].as_str().unwrap_or("Unknown").to_string();
                let timestamp = item["applyTime"].as_i64()
                    .or_else(|| item["insertTime"].as_i64())
                    .unwrap_or(0);

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
// SUB ACCOUNTS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl SubAccounts for MexcConnector {
    /// Perform sub-account operations: Create, List, Transfer, GetBalance.
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        match op {
            SubAccountOperation::Create { label } => {
                // POST /api/v3/sub-account/virtualSubAccount
                let mut params = HashMap::new();
                params.insert("subUserName".to_string(), label.clone());

                let response = self.post(MexcEndpoint::SubAccountCreate, params).await?;

                let id = response["subUserId"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| response["subUserId"].as_i64().map(|n| n.to_string()));

                Ok(SubAccountResult {
                    id,
                    name: Some(label),
                    accounts: vec![],
                    transaction_id: None,
                })
            }

            SubAccountOperation::List => {
                // GET /api/v3/sub-account/list
                let response = self.get(MexcEndpoint::SubAccountList, HashMap::new()).await?;

                let items = response.get("subAccounts")
                    .and_then(|v| v.as_array())
                    .or_else(|| response.as_array())
                    .cloned()
                    .unwrap_or_default();

                let accounts = items.iter().map(|item| {
                    let id = item["subUserId"]
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| item["subUserId"].as_i64().map(|n| n.to_string()))
                        .unwrap_or_default();
                    let name = item["subUserName"].as_str().unwrap_or("").to_string();
                    let status = if item["isFreeze"].as_bool().unwrap_or(false) {
                        "Frozen".to_string()
                    } else {
                        "Normal".to_string()
                    };

                    SubAccount { id, name, status }
                }).collect();

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts,
                    transaction_id: None,
                })
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                // POST /api/v3/capital/sub-account/universalTransfer
                // fromEmail / toEmail identifies the direction
                // MEXC uses email as sub-account identifier
                let mut params = HashMap::new();
                if to_sub {
                    params.insert("toEmail".to_string(), sub_account_id.clone());
                    params.insert("fromAccountType".to_string(), "SPOT".to_string());
                    params.insert("toAccountType".to_string(), "SPOT".to_string());
                } else {
                    params.insert("fromEmail".to_string(), sub_account_id.clone());
                    params.insert("fromAccountType".to_string(), "SPOT".to_string());
                    params.insert("toAccountType".to_string(), "SPOT".to_string());
                }
                params.insert("asset".to_string(), asset);
                params.insert("amount".to_string(), amount.to_string());

                let response = self.post(MexcEndpoint::SubAccountTransfer, params).await?;

                let tran_id = response["tranId"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| response["tranId"].as_i64().map(|n| n.to_string()));

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts: vec![],
                    transaction_id: tran_id,
                })
            }

            SubAccountOperation::GetBalance { sub_account_id } => {
                // GET /api/v3/sub-account/assets?email={sub_account_id}
                let mut params = HashMap::new();
                params.insert("email".to_string(), sub_account_id);

                let _response = self.get(MexcEndpoint::SubAccountAssets, params).await?;

                // Balance is available in response but SubAccountResult doesn't carry it;
                // return the sub-account id as acknowledgement that data was fetched.
                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts: vec![],
                    transaction_id: None,
                })
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (not part of core traits)
// ═══════════════════════════════════════════════════════════════════════════════

impl MexcConnector {
    /// Get recent public spot trades.
    ///
    /// `GET /api/v3/trades`
    ///
    /// # Parameters
    /// - `symbol`: Spot symbol e.g. `BTCUSDT`
    /// - `limit`: Number of trades to return (optional, default 500, max 1000)
    pub async fn get_recent_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(MexcEndpoint::RecentTrades, params).await
    }

    /// Get personal spot trade history (requires auth).
    ///
    /// `GET /api/v3/myTrades`
    ///
    /// # Parameters
    /// - `symbol`: Spot symbol e.g. `BTCUSDT`
    /// - `limit`: Max number of trades (optional, default 500, max 1000)
    /// - `start_time`: Start timestamp in ms (optional)
    /// - `end_time`: End timestamp in ms (optional)
    pub async fn get_my_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        if let Some(st) = start_time {
            params.insert("startTime".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }
        self.get(MexcEndpoint::MyTrades, params).await
    }

    /// Get futures mark price and index price for a contract.
    ///
    /// `GET /api/v1/contract/index_price/{symbol}`
    ///
    /// Returns the current mark price and index price for the given futures contract.
    pub async fn get_futures_mark_price(&self, symbol: &str) -> ExchangeResult<Value> {
        // MEXC futures endpoints use path-based symbol: /api/v1/contract/index_price/{symbol}
        let base_url = MexcUrls::futures_base_url();
        let path = format!("{}/{}", MexcEndpoint::FuturesMarkPrice.path(), symbol);
        let url = format!("{}{}", base_url, path);
        self.rate_limit_wait().await;
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.update_weight_from_headers(&resp_headers);
        Ok(response)
    }
}

/// Map internal AccountType to MEXC's transfer account type string.
fn account_type_to_mexc_str(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "SPOT",
        AccountType::Margin => "MARGIN",
        AccountType::FuturesCross | AccountType::FuturesIsolated => "FUTURES",
        AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => "SPOT",
    }
}

fn interval_to_ms(interval: &str) -> i64 {
    match interval {
        "1m" => 60_000,
        "5m" => 300_000,
        "15m" => 900_000,
        "30m" => 1_800_000,
        "1h" => 3_600_000,
        "4h" => 14_400_000,
        "12h" => 43_200_000,
        "1d" => 86_400_000,
        "1w" => 604_800_000,
        _ => 3_600_000,
    }
}
