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
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account,
};
use crate::core::types::ConnectorStats;
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
                if limiter.try_acquire(1) {
                    return; // Successfully acquired, exit early
                }
                limiter.time_until_ready(1)
            }; // Lock is dropped here

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
        // Wait for rate limit (weight 1 for most GET requests)
        self.rate_limit_wait().await;

        // Route to correct base URL based on endpoint type
        let base_url = if endpoint.is_futures() {
            MexcUrls::futures_base_url()
        } else {
            MexcUrls::base_url()
        };
        let path = endpoint.path();

        // Build query string and URL
        let (url, headers) = if endpoint.is_private() {
            // Authenticated request
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            let (headers, signed_params) = auth.sign_request(params);

            // Build query string from signed params
            let query_parts: Vec<String> = signed_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            let query_string = query_parts.join("&");

            let url = format!("{}{}?{}", base_url, path, query_string);
            (url, headers)
        } else {
            // Public request
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
        // Wait for rate limit (weight 1 for most POST requests)
        self.rate_limit_wait().await;

        let base_url = MexcUrls::base_url();
        let path = endpoint.path();

        // Auth required for POST
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let (headers, signed_params) = auth.sign_request(params);

        // Build query string from signed params
        let query_parts: Vec<String> = signed_params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_string = query_parts.join("&");

        let url = format!("{}{}?{}", base_url, path, query_string);

        // POST with empty body (all params in query string)
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
        // Wait for rate limit (weight 1 for most DELETE requests)
        self.rate_limit_wait().await;

        let base_url = MexcUrls::base_url();
        let path = endpoint.path();

        // Auth required for DELETE
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let (headers, signed_params) = auth.sign_request(params);

        // Build query string from signed params
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
                // Get ticker which contains price info
                let ticker = self.get_ticker(symbol, account_type).await?;
                Ok(ticker.last_price)
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
                // Futures: symbol in path
                let base_url = MexcUrls::futures_base_url();
                let formatted_symbol = format_symbol(&symbol, account_type);
                let path = format!("/api/v1/contract/depth/{}", formatted_symbol);
                let url = format!("{}{}", base_url, path);

                self.rate_limit_wait().await;
                let response = self.http.get(&url, &HashMap::new()).await?;
                MexcParser::check_error(&response)?;

                // Futures orderbook is in data field
                let data = response.get("data")
                    .ok_or_else(|| ExchangeError::Parse("Missing data field in futures orderbook".into()))?;
                MexcParser::parse_orderbook_futures(data)
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

                // MEXC requires BOTH startTime + endTime together.
                // endTime alone is silently ignored.
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
                // Futures: symbol in path, different interval format
                let base_url = MexcUrls::futures_base_url();
                let formatted_symbol = format_symbol(&symbol, account_type);
                let path = format!("/api/v1/contract/kline/{}", formatted_symbol);

                // Map interval to MEXC futures format
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
                    _ => "Min60", // Default to 1 hour
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

                // Futures klines are in data field
                let klines_data = response.get("data")
                    .ok_or_else(|| ExchangeError::Parse("Missing data field in futures klines".into()))?;
                MexcParser::parse_klines_futures(klines_data)
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
                // Futures ticker returns all symbols, filter for our symbol
                let response = self.get(MexcEndpoint::FuturesTicker, HashMap::new()).await?;

                let formatted_symbol = format_symbol(&symbol, account_type);

                // Response is in data.data array
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
        }
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _ = self.get(MexcEndpoint::Ping, HashMap::new()).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get(MexcEndpoint::ExchangeInfo, HashMap::new()).await?;
        MexcParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for MexcConnector {
    async fn market_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let client_order_id = format!("cc_{}", crate::core::timestamp_millis());

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
        params.insert("side".to_string(), match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }.to_string());
        params.insert("type".to_string(), "MARKET".to_string());
        params.insert("quantity".to_string(), quantity.to_string());
        params.insert("newClientOrderId".to_string(), client_order_id.clone());

        let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

        // Extract order ID from response
        let order_id = response["orderId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
            .to_string();

        // Return minimal order info (can fetch full info with get_order)
        Ok(Order {
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
            time_in_force: crate::core::TimeInForce::GTC,
        })
    }

    async fn limit_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let client_order_id = format!("cc_{}", crate::core::timestamp_millis());

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
        params.insert("side".to_string(), match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }.to_string());
        params.insert("type".to_string(), "LIMIT".to_string());
        params.insert("quantity".to_string(), quantity.to_string());
        params.insert("price".to_string(), price.to_string());
        params.insert("newClientOrderId".to_string(), client_order_id.clone());

        let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

        let order_id = response["orderId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
            .to_string();

        Ok(Order {
            id: order_id,
            client_order_id: Some(client_order_id),
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Limit,
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
            time_in_force: crate::core::TimeInForce::GTC,
        })
    }

    async fn cancel_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.delete(MexcEndpoint::CancelOrder, params).await?;
        MexcParser::parse_order(&response)
    }

    async fn get_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(MexcEndpoint::QueryOrder, params).await?;
        MexcParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();

        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s, account_type));
        }

        let response = self.get(MexcEndpoint::OpenOrders, params).await?;
        MexcParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for MexcConnector {
    async fn get_balance(
        &self,
        _asset: Option<crate::core::Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
        let response = self.get(MexcEndpoint::Account, HashMap::new()).await?;
        MexcParser::parse_balance(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(MexcEndpoint::Account, HashMap::new()).await?;

        // Get balances
        let balances = MexcParser::parse_balance(&response)?;

        // Parse account info from response
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
            .map(|c| c as f64 / 10000.0) // Convert from basis points
            .unwrap_or(0.002); // Default 0.2%

        let taker_commission = response.get("takerCommission")
            .and_then(|v| v.as_i64())
            .map(|c| c as f64 / 10000.0)
            .unwrap_or(0.002); // Default 0.2%

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
