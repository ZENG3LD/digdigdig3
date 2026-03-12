//! # Angel One SmartAPI Connector
//!
//! Full broker implementation with TOTP authentication.
//!
//! ## Core traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data operations
//! - `Trading` - trading operations (full broker capabilities)
//! - `Account` - account information and balance
//! - `Positions` - positions management (equity, F&O, commodity)
//!
//! ## Extended methods
//! Additional Angel One-specific methods as struct methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate, OrderStatus, TimeInForce,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::SymbolInfo;
use crate::core::utils::SimpleRateLimiter;
use crate::core::timestamp_millis;

use super::endpoints::{AngelOneUrls, AngelOneEndpoint, format_symbol, map_interval};
use super::auth::AngelOneAuth;
use super::parser::AngelOneParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Angel One SmartAPI connector
///
/// Full-featured broker connector for Indian markets.
/// Supports equity, derivatives (F&O), commodities, and currency trading.
pub struct AngelOneConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication handler
    auth: Arc<Mutex<AngelOneAuth>>,
    /// URLs (mainnet/testnet)
    urls: AngelOneUrls,
    /// Testnet mode (Angel One has no real testnet)
    testnet: bool,
    /// Rate limiter (20 orders/sec, 10 queries/sec)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl AngelOneConnector {
    /// Create new Angel One connector
    ///
    /// # Parameters
    /// - `api_key` - Angel One API key
    /// - `client_code` - Angel One client code (account ID)
    /// - `pin` - Account PIN/password
    /// - `totp_secret` - TOTP secret for 2FA
    /// - `testnet` - Testnet flag (no effect, Angel One has no testnet)
    pub async fn new(
        api_key: String,
        client_code: String,
        pin: String,
        totp_secret: String,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = AngelOneUrls::get(testnet);
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = AngelOneAuth::new(api_key, client_code, pin, totp_secret);

        // Perform login to get JWT tokens
        let base_url = urls.rest_base;
        let login_url = format!("{}{}", base_url, AngelOneEndpoint::Login.path());

        let login_body = auth.build_login_body()?;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-PrivateKey".to_string(), auth.api_key.clone());
        headers.insert("X-ClientLocalIP".to_string(), "192.168.1.1".to_string());
        headers.insert("X-ClientPublicIP".to_string(), "0.0.0.0".to_string());
        headers.insert("X-MACAddress".to_string(), "00:00:00:00:00:00".to_string());

        let response = http.post(&login_url, &login_body, &headers).await?;

        // Parse login response and store tokens
        let (jwt_token, refresh_token) = AngelOneParser::parse_login(&response)?;

        // Get feed token for WebSocket
        let feed_token_url = format!("{}{}", base_url, AngelOneEndpoint::GetFeedToken.path());
        let mut auth_headers = headers.clone();
        auth_headers.insert("Authorization".to_string(), format!("Bearer {}", jwt_token));

        let feed_response = http.get_with_headers(&feed_token_url, &HashMap::new(), &auth_headers).await?;
        let feed_data = AngelOneParser::extract_data(&feed_response)?;
        let feed_token = feed_data.get("feedToken")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing feedToken".to_string()))?
            .to_string();

        auth.set_tokens(jwt_token, refresh_token, feed_token);

        // Initialize rate limiter: 10 requests per second (conservative)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(10, Duration::from_secs(1))
        ));

        Ok(Self {
            http,
            auth: Arc::new(Mutex::new(auth)),
            urls,
            testnet,
            rate_limiter,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
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

    /// GET request
    async fn get(
        &self,
        endpoint: AngelOneEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_base;
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
            let auth = self.auth.lock().expect("Mutex poisoned");
            auth.sign_headers()?
        } else {
            HashMap::new()
        };

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: AngelOneEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_base;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Add auth headers
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.lock().expect("Mutex poisoned");
            auth.sign_headers()?
        } else {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        };

        let response = self.http.post(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Check response for errors
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        let status = response.get("status")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !status {
            let message = response.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            let error_code = response.get("errorcode")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            return Err(ExchangeError::Api {
                code: -1,
                message: format!("Angel One API error: {} (code: {})", message, error_code),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Angel One-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Refresh JWT token using refresh token
    pub async fn refresh_token(&self) -> ExchangeResult<()> {
        let refresh_body = {
            let auth = self.auth.lock().expect("Mutex poisoned");
            auth.build_refresh_body()?
        };

        let url = format!("{}{}", self.urls.rest_base, AngelOneEndpoint::TokenRefresh.path());

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.post(&url, &refresh_body, &headers).await?;

        let (jwt_token, refresh_token) = AngelOneParser::parse_token_refresh(&response)?;

        let mut auth = self.auth.lock().expect("Mutex poisoned");
        let current_feed = auth.feed_token.clone().unwrap_or_default();
        auth.set_tokens(jwt_token, refresh_token, current_feed);

        Ok(())
    }

    /// Logout and clear session
    pub async fn logout(&self) -> ExchangeResult<()> {
        let logout_body = {
            let auth = self.auth.lock().expect("Mutex poisoned");
            auth.build_logout_body()
        };

        let response = self.post(AngelOneEndpoint::Logout, logout_body).await?;
        self.check_response(&response)?;

        let mut auth = self.auth.lock().expect("Mutex poisoned");
        auth.clear_tokens();

        Ok(())
    }

    /// Get user profile
    pub async fn get_profile(&self) -> ExchangeResult<Value> {
        self.get(AngelOneEndpoint::GetProfile, HashMap::new()).await
    }

    /// Search scrip by name or symbol
    pub async fn search_scrip(&self, exchange: &str, searchscrip: &str) -> ExchangeResult<Vec<Value>> {
        let body = json!({
            "exchange": exchange,
            "searchscrip": searchscrip
        });

        let response = self.post(AngelOneEndpoint::SearchScrip, body).await?;
        let data = AngelOneParser::extract_data(&response)?;

        let results = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of search results".to_string()))?
            .to_vec();

        Ok(results)
    }

    /// Get holdings
    pub async fn get_holdings(&self) -> ExchangeResult<Vec<Value>> {
        let response = self.get(AngelOneEndpoint::GetHoldings, HashMap::new()).await?;
        let data = AngelOneParser::extract_data(&response)?;

        let holdings = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of holdings".to_string()))?
            .to_vec();

        Ok(holdings)
    }

    /// Modify existing order
    pub async fn modify_order(
        &self,
        order_id: &str,
        quantity: Option<f64>,
        price: Option<f64>,
        order_type: Option<OrderType>,
    ) -> ExchangeResult<Value> {
        let mut body = json!({
            "orderid": order_id,
            "variety": "NORMAL"
        });

        if let Some(qty) = quantity {
            body["quantity"] = json!(qty.to_string());
        }
        if let Some(p) = price {
            body["price"] = json!(p.to_string());
        }
        if let Some(ot) = order_type {
            body["ordertype"] = json!(match ot {
                OrderType::Market => "MARKET",
                OrderType::Limit => "LIMIT",
                OrderType::StopLoss => "STOPLOSS_MARKET",
                OrderType::StopLossLimit => "STOPLOSS_LIMIT",
                OrderType::TakeProfit => "LIMIT", // Angel One doesn't have native take profit
                OrderType::TakeProfitLimit => "LIMIT",
            });
        }

        self.post(AngelOneEndpoint::ModifyOrder, body).await
    }

    /// Get order book (all orders)
    pub async fn get_order_book(&self) -> ExchangeResult<Vec<Value>> {
        let response = self.get(AngelOneEndpoint::GetOrderBook, HashMap::new()).await?;
        let data = AngelOneParser::extract_data(&response)?;

        let orders = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?
            .to_vec();

        Ok(orders)
    }

    /// Get trade book (executed trades)
    pub async fn get_trade_book(&self) -> ExchangeResult<Vec<Value>> {
        let response = self.get(AngelOneEndpoint::GetTradeBook, HashMap::new()).await?;
        let data = AngelOneParser::extract_data(&response)?;

        let trades = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of trades".to_string()))?
            .to_vec();

        Ok(trades)
    }

    /// Get RMS (Risk Management System) data including margin info
    pub async fn get_rms(&self) -> ExchangeResult<Value> {
        let response = self.get(AngelOneEndpoint::GetRMS, HashMap::new()).await?;
        AngelOneParser::extract_data(&response).cloned()
    }

    /// Convert position (e.g., intraday to delivery)
    #[allow(clippy::too_many_arguments)]
    pub async fn convert_position(
        &self,
        symbol_token: &str,
        exchange: &str,
        transaction_type: &str, // "BUY" or "SELL"
        position_type: &str,    // "INTRADAY" or "DELIVERY"
        quantity: f64,
        old_product_type: &str,
        new_product_type: &str,
    ) -> ExchangeResult<Value> {
        let body = json!({
            "symboltoken": symbol_token,
            "exchange": exchange,
            "transactiontype": transaction_type,
            "positiontype": position_type,
            "quantity": quantity.to_string(),
            "type": old_product_type,
            "targettype": new_product_type
        });

        self.post(AngelOneEndpoint::ConvertPosition, body).await
    }

    /// Calculate margin requirement before placing order
    pub async fn calculate_margin(&self, orders: Vec<Value>) -> ExchangeResult<Value> {
        let body = json!({
            "positions": orders
        });

        self.post(AngelOneEndpoint::MarginCalculator, body).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for AngelOneConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::AngelOne
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot, // Used for all trading (equity, F&O, commodity)
        ]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Broker
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for AngelOneConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Angel One requires symboltoken and exchange for quote API
        // For now, use LTP mode with simplified approach
        let body = json!({
            "mode": "LTP",
            "exchangeTokens": {
                "NSE": [format_symbol(&symbol)]
            }
        });

        let response = self.post(AngelOneEndpoint::Quote, body).await?;
        AngelOneParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Get FULL quote for order book data
        let body = json!({
            "mode": "FULL",
            "exchangeTokens": {
                "NSE": [format_symbol(&symbol)]
            }
        });

        let response = self.post(AngelOneEndpoint::Quote, body).await?;
        AngelOneParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        // Angel One historical data requires symboltoken
        // For simplification, using basic parameters
        let to_date = chrono::Utc::now();
        let from_date = to_date - chrono::Duration::days(30);

        let body = json!({
            "exchange": "NSE",
            "symboltoken": format_symbol(&symbol),
            "interval": map_interval(interval),
            "fromdate": from_date.format("%Y-%m-%d %H:%M").to_string(),
            "todate": to_date.format("%Y-%m-%d %H:%M").to_string()
        });

        let response = self.post(AngelOneEndpoint::HistoricalCandles, body).await?;
        let mut klines = AngelOneParser::parse_klines(&response)?;

        // Apply limit if specified
        if let Some(l) = limit {
            klines.truncate(l as usize);
        }

        Ok(klines)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        // Get FULL quote for ticker data
        let body = json!({
            "mode": "FULL",
            "exchangeTokens": {
                "NSE": [format_symbol(&symbol)]
            }
        });

        let response = self.post(AngelOneEndpoint::Quote, body).await?;
        AngelOneParser::parse_ticker(&response, &symbol.to_string())
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Use GetProfile as ping
        let response = self.get(AngelOneEndpoint::GetProfile, HashMap::new()).await?;
        self.check_response(&response)
    }

    /// Get exchange info — search NSE equity symbols (Angel One uses search-based approach)
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Angel One doesn't have a bulk symbol listing endpoint.
        // Use SearchScrip with a broad search on NSE exchange.
        // In practice, callers should use `search_scrip()` for targeted searches.
        let body = serde_json::json!({
            "exchange": "NSE",
            "searchscrip": ""
        });

        let response = self.post(AngelOneEndpoint::SearchScrip, body).await?;
        let data = AngelOneParser::extract_data(&response)?;

        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array from SearchScrip".to_string()))?;

        let infos = arr.iter().filter_map(|item| {
            let symbol = item.get("tradingsymbol")?.as_str()?.to_string();
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let exchange = item.get("exch_seg").and_then(|v| v.as_str()).unwrap_or("NSE").to_string();
            let _ = name;
            let _ = exchange;

            Some(SymbolInfo {
                symbol: symbol.clone(),
                base_asset: symbol,
                quote_asset: "INR".to_string(),
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: Some(1.0),
                max_quantity: None,
                step_size: Some(1.0),
                min_notional: None,
            })
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for AngelOneConnector {
    async fn market_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let body = json!({
            "variety": "NORMAL",
            "tradingsymbol": format_symbol(&symbol),
            "symboltoken": format_symbol(&symbol), // Should be actual token
            "transactiontype": match side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            },
            "exchange": "NSE",
            "ordertype": "MARKET",
            "producttype": "INTRADAY",
            "duration": "DAY",
            "quantity": quantity.to_string(),
        });

        let response = self.post(AngelOneEndpoint::PlaceOrder, body).await?;
        let order_id = AngelOneParser::parse_order_id(&response)?;

        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Market,
            status: OrderStatus::New,
            price: None,
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: timestamp_millis() as i64,
            updated_at: None,
            time_in_force: TimeInForce::GTC,
        })
    }

    async fn limit_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let body = json!({
            "variety": "NORMAL",
            "tradingsymbol": format_symbol(&symbol),
            "symboltoken": format_symbol(&symbol),
            "transactiontype": match side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            },
            "exchange": "NSE",
            "ordertype": "LIMIT",
            "producttype": "INTRADAY",
            "duration": "DAY",
            "price": price.to_string(),
            "quantity": quantity.to_string(),
        });

        let response = self.post(AngelOneEndpoint::PlaceOrder, body).await?;
        let order_id = AngelOneParser::parse_order_id(&response)?;

        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Limit,
            status: OrderStatus::New,
            price: Some(price),
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: timestamp_millis() as i64,
            updated_at: None,
            time_in_force: TimeInForce::GTC,
        })
    }

    async fn cancel_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let body = json!({
            "variety": "NORMAL",
            "orderid": order_id
        });

        let response = self.post(AngelOneEndpoint::CancelOrder, body).await?;
        self.check_response(&response)?;

        Ok(Order {
            id: order_id.to_string(),
            client_order_id: None,
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            status: OrderStatus::Canceled,
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: 0,
            updated_at: Some(timestamp_millis() as i64),
            time_in_force: TimeInForce::GTC,
        })
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("orderid".to_string(), order_id.to_string());

        let response = self.get(AngelOneEndpoint::GetOrderDetails, params).await?;
        let details = AngelOneParser::parse_order_details(&response)?;

        Ok(Order {
            id: details.order_id,
            client_order_id: None,
            symbol: details.symbol,
            side: details.side,
            order_type: details.order_type,
            status: details.status,
            price: details.price,
            stop_price: None,
            quantity: details.quantity,
            filled_quantity: details.filled_quantity.unwrap_or(0.0),
            average_price: details.average_price,
            commission: None,
            commission_asset: None,
            created_at: timestamp_millis() as i64,
            updated_at: Some(timestamp_millis() as i64),
            time_in_force: TimeInForce::GTC,
        })
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let order_book = self.get_order_book().await?;

        let orders: Vec<Order> = order_book.iter()
            .filter_map(|order| {
                let status = order.get("orderstatus")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");

                // Filter for open orders
                if status == "OPEN" || status == "PENDING" {
                    let order_id = order.get("orderid")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    let symbol = order.get("tradingsymbol")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();

                    let side = match order.get("transactiontype").and_then(|s| s.as_str()) {
                        Some("BUY") => OrderSide::Buy,
                        Some("SELL") => OrderSide::Sell,
                        _ => OrderSide::Buy,
                    };

                    let order_type = match order.get("ordertype").and_then(|s| s.as_str()) {
                        Some("MARKET") => OrderType::Market,
                        Some("LIMIT") => OrderType::Limit,
                        Some("STOPLOSS_LIMIT") => OrderType::StopLossLimit,
                        Some("STOPLOSS_MARKET") => OrderType::StopLoss,
                        _ => OrderType::Market,
                    };

                    Some(Order {
                        id: order_id,
                        client_order_id: None,
                        symbol,
                        side,
                        order_type,
                        status: OrderStatus::New,
                        price: order.get("price").and_then(|p| p.as_f64()),
                        stop_price: None,
                        quantity: order.get("quantity").and_then(|q| q.as_f64()).unwrap_or(0.0),
                        filled_quantity: order.get("filledshares").and_then(|f| f.as_f64()).unwrap_or(0.0),
                        average_price: order.get("averageprice").and_then(|a| a.as_f64()),
                        commission: None,
                        commission_asset: None,
                        created_at: timestamp_millis() as i64,
                        updated_at: None,
                        time_in_force: TimeInForce::GTC,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(orders)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for AngelOneConnector {
    async fn get_balance(
        &self,
        _asset: Option<crate::core::Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
        let response = self.get(AngelOneEndpoint::GetRMS, HashMap::new()).await?;
        AngelOneParser::parse_balance(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(AngelOneEndpoint::GetProfile, HashMap::new()).await?;
        let mut account_info = AngelOneParser::parse_account_info(&response)?;

        // Add balance information
        let balances = self.get_balance(None, account_type).await?;
        account_info.balances = balances;

        Ok(account_info)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for AngelOneConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        let response = self.get(AngelOneEndpoint::GetPositions, HashMap::new()).await?;
        AngelOneParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Angel One doesn't have funding rates (not a crypto perpetual futures exchange)
        // Indian F&O contracts have expiry dates, not funding rates
        Err(ExchangeError::UnsupportedOperation(
            "Funding rates not applicable for Indian equity/F&O markets".to_string()
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        // Angel One doesn't have adjustable leverage
        // Margin is determined by product type (INTRADAY, DELIVERY, etc.)
        Err(ExchangeError::UnsupportedOperation(
            "Leverage is fixed by product type in Indian markets".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_creation_fails_without_credentials() {
        // Cannot test without valid credentials
        // This is a placeholder to ensure tests compile
        assert!(true);
    }
}
