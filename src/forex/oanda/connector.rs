//! # OANDA v20 Connector
//!
//! Implementation of core traits for OANDA forex broker.
//!
//! ## Traits Implemented
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data operations
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Position management
//!
//! ## Important Notes
//! - OANDA is a forex BROKER, not just a data provider
//! - Uses Bearer token authentication (not HMAC)
//! - HTTP streaming for real-time data (not WebSocket)
//! - Symbol format: "EUR_USD" (underscore separator)
//! - All numeric values returned as strings

use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate, SymbolInfo,
    OrderRequest, CancelRequest, CancelScope,
    AmendRequest,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    AmendOrder,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};

use super::endpoints::{OandaUrls, OandaEndpoint, format_symbol, map_granularity};
use super::auth::OandaAuth;
use super::parser::OandaParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// OANDA v20 connector
pub struct OandaConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: OandaAuth,
    /// URLs (practice/live)
    urls: OandaUrls,
    /// Practice mode flag
    practice: bool,
    /// Account ID (cached)
    account_id: Option<String>,
}

impl OandaConnector {
    /// Create new OANDA connector
    ///
    /// # Arguments
    /// - `credentials` - Bearer token (stored in api_key field)
    /// - `practice` - If true, use practice (demo) account
    pub async fn new(credentials: Credentials, practice: bool) -> ExchangeResult<Self> {
        let urls = if practice {
            OandaUrls::PRACTICE
        } else {
            OandaUrls::LIVE
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = OandaAuth::new(&credentials)?;

        Ok(Self {
            http,
            auth,
            urls,
            practice,
            account_id: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT ID MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get account ID (fetch and cache if needed)
    async fn get_account_id(&mut self) -> ExchangeResult<String> {
        if let Some(ref id) = self.account_id {
            return Ok(id.clone());
        }

        // Fetch account ID
        let endpoint = OandaEndpoint::ListAccounts;
        let response = self.get(endpoint, HashMap::new()).await?;
        let account_id = OandaParser::parse_account_id(&response)?;

        self.account_id = Some(account_id.clone());
        Ok(account_id)
    }

    /// Get account ID reference (must be cached)
    fn require_account_id(&self) -> ExchangeResult<&str> {
        self.account_id.as_deref()
            .ok_or_else(|| ExchangeError::Auth("Account ID not initialized".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// GET request
    async fn get(
        &self,
        endpoint: OandaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url;
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

        // Add auth headers
        let headers = self.auth.sign_request();

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: OandaEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let headers = self.auth.sign_request();
        let response = self.http.post(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// PUT request
    async fn put(
        &self,
        endpoint: OandaEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let headers = self.auth.sign_request();
        let response = self.http.put(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Check response for errors
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        // OANDA returns error in "errorMessage" field
        if let Some(error_msg) = response.get("errorMessage") {
            let msg = error_msg.as_str().unwrap_or("Unknown error");

            // Check for specific error codes
            if let Some(code) = response.get("errorCode").and_then(|c| c.as_str()) {
                return Err(ExchangeError::Api {
                    code: -1,
                    message: format!("{}: {}", code, msg),
                });
            }

            return Err(ExchangeError::Api {
                code: -1,
                message: msg.to_string(),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (OANDA-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get tradeable instruments for account
    pub async fn get_instruments(&mut self) -> ExchangeResult<Vec<String>> {
        let account_id = self.get_account_id().await?;
        let endpoint = OandaEndpoint::GetInstruments(account_id);

        let response = self.get(endpoint, HashMap::new()).await?;

        let instruments = response.get("instruments")
            .and_then(|i| i.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|inst| inst.get("name").and_then(|n| n.as_str()))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(instruments)
    }

    /// Close all positions for an instrument
    pub async fn close_all_positions(&mut self, symbol: Symbol) -> ExchangeResult<()> {
        let account_id = self.get_account_id().await?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::ClosePosition {
            account_id,
            instrument,
        };

        let body = json!({
            "longUnits": "ALL",
            "shortUnits": "ALL"
        });

        self.put(endpoint, body).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for OandaConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Oanda
    }

    fn is_testnet(&self) -> bool {
        self.practice
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // OANDA is forex broker - uses Spot as default account type
        vec![AccountType::Spot]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for OandaConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::GetPricing(account_id.to_string());

        let mut params = HashMap::new();
        params.insert("instruments".to_string(), instrument);

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::GetPricing(account_id.to_string());

        let mut params = HashMap::new();
        params.insert("instruments".to_string(), instrument);

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let instrument = format_symbol(&symbol.base, &symbol.quote);
        let endpoint = OandaEndpoint::GetCandles(instrument.clone());

        let mut params = HashMap::new();
        params.insert("granularity".to_string(), map_granularity(interval).to_string());
        params.insert("count".to_string(), limit.unwrap_or(500).to_string());
        params.insert("price".to_string(), "M".to_string()); // Mid prices

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        let endpoint = OandaEndpoint::GetPricing(account_id.to_string());

        let mut params = HashMap::new();
        params.insert("instruments".to_string(), instrument);

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // List accounts is a simple ping test
        let endpoint = OandaEndpoint::ListAccounts;
        let response = self.get(endpoint, HashMap::new()).await?;
        self.check_response(&response)
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Step 1: fetch account ID (can't use cached value since we have &self, not &mut self)
        let accounts_response = self.get(OandaEndpoint::ListAccounts, HashMap::new()).await?;
        let account_id = OandaParser::parse_account_id(&accounts_response)?;

        // Step 2: fetch instruments for that account
        let endpoint = OandaEndpoint::GetInstruments(account_id);
        let response = self.get(endpoint, HashMap::new()).await?;

        let instruments = response
            .get("instruments")
            .and_then(|i| i.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'instruments' array".to_string()))?;

        let infos = instruments
            .iter()
            .filter_map(|inst| {
                let name = inst.get("name").and_then(|n| n.as_str())?; // e.g. "EUR_USD"
                let _inst_type = inst.get("type").and_then(|t| t.as_str()).unwrap_or("CURRENCY");
                let _display = inst.get("displayName").and_then(|d| d.as_str()).unwrap_or(name);

                // OANDA format: "EUR_USD" → base="EUR", quote="USD"
                let parts: Vec<&str> = name.splitn(2, '_').collect();
                let (base, quote) = if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (name.to_string(), "USD".to_string())
                };

                // OANDA provides pipLocation (negative int, e.g. -5 means 5 decimal places)
                // tick_size = 10^pipLocation (e.g. -5 → 0.00001)
                let pip_location = inst.get("pipLocation")
                    .and_then(|v| v.as_i64());
                let tick_size = pip_location.map(|pl| 10f64.powi(pl as i32));

                // price_precision: number of digits after decimal point (displayPrecision or abs(pipLocation))
                let price_precision = inst.get("displayPrecision")
                    .and_then(|v| v.as_u64())
                    .unwrap_or_else(|| pip_location.map(|pl| pl.unsigned_abs()).unwrap_or(5))
                    as u8;

                Some(SymbolInfo {
                    symbol: name.to_string(),
                    base_asset: base,
                    quote_asset: quote,
                    status: "TRADING".to_string(),
                    price_precision,
                    quantity_precision: 0,
                    min_quantity: Some(1.0),
                    max_quantity: None,
                    tick_size,
                    step_size: Some(1.0),
                    min_notional: None,
                })
            })
            .collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for OandaConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;

        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&symbol.base, &symbol.quote);

        // OANDA uses signed units: positive = buy, negative = sell
        let units = match side {
            OrderSide::Buy => quantity,
            OrderSide::Sell => -quantity,
        };
        let units_str = units.to_string();

        let endpoint = OandaEndpoint::CreateOrder(account_id.to_string());

        let (body, result_order_type, result_price, result_stop_price, result_tif) = match req.order_type {
            OrderType::Market => {
                let b = json!({
                    "order": {
                        "type": "MARKET",
                        "instrument": instrument,
                        "units": units_str,
                        "timeInForce": "FOK",
                        "positionFill": "DEFAULT"
                    }
                });
                (b, OrderType::Market, None::<f64>, None::<f64>, crate::core::TimeInForce::Fok)
            }

            OrderType::Limit { price } => {
                let b = json!({
                    "order": {
                        "type": "LIMIT",
                        "instrument": instrument,
                        "units": units_str,
                        "price": price.to_string(),
                        "timeInForce": "GTC",
                        "positionFill": "DEFAULT"
                    }
                });
                (b, OrderType::Limit { price }, Some(price), None, crate::core::TimeInForce::Gtc)
            }

            OrderType::StopMarket { stop_price } => {
                // OANDA STOP order: triggers a market order at stop_price
                let b = json!({
                    "order": {
                        "type": "STOP",
                        "instrument": instrument,
                        "units": units_str,
                        "price": stop_price.to_string(),
                        "timeInForce": "GTC",
                        "positionFill": "DEFAULT"
                    }
                });
                (b, OrderType::StopMarket { stop_price }, None, Some(stop_price), crate::core::TimeInForce::Gtc)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // OANDA STOP with priceBound acts as stop-limit
                let b = json!({
                    "order": {
                        "type": "STOP",
                        "instrument": instrument,
                        "units": units_str,
                        "price": stop_price.to_string(),
                        "priceBound": limit_price.to_string(),
                        "timeInForce": "GTC",
                        "positionFill": "DEFAULT"
                    }
                });
                (b, OrderType::StopLimit { stop_price, limit_price }, Some(limit_price), Some(stop_price), crate::core::TimeInForce::Gtc)
            }

            OrderType::TrailingStop { callback_rate, activation_price: _ } => {
                // OANDA TRAILING_STOP_LOSS uses distance in price units.
                // callback_rate is a percentage — pass directly as distance.
                let b = json!({
                    "order": {
                        "type": "TRAILING_STOP_LOSS",
                        "instrument": instrument,
                        "units": units_str,
                        "distance": callback_rate.to_string(),
                        "timeInForce": "GTC",
                        "positionFill": "DEFAULT"
                    }
                });
                (b, OrderType::TrailingStop { callback_rate, activation_price: None }, None, None, crate::core::TimeInForce::Gtc)
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                // OANDA bracket: LIMIT (or MARKET) with takeProfitOnFill + stopLossOnFill
                let order_type_str = if price.is_some() { "LIMIT" } else { "MARKET" };
                let mut order_body = serde_json::Map::new();
                order_body.insert("type".to_string(), json!(order_type_str));
                order_body.insert("instrument".to_string(), json!(instrument));
                order_body.insert("units".to_string(), json!(units_str));
                order_body.insert("timeInForce".to_string(), json!("GTC"));
                order_body.insert("positionFill".to_string(), json!("DEFAULT"));
                if let Some(p) = price {
                    order_body.insert("price".to_string(), json!(p.to_string()));
                }
                order_body.insert("takeProfitOnFill".to_string(), json!({ "price": take_profit.to_string() }));
                order_body.insert("stopLossOnFill".to_string(), json!({ "price": stop_loss.to_string() }));

                let b = json!({ "order": serde_json::Value::Object(order_body) });
                let entry_price = price;
                (b, OrderType::Bracket { price, take_profit, stop_loss }, entry_price, None, crate::core::TimeInForce::Gtc)
            }

            OrderType::Ioc { price } => {
                // IOC with optional price — if price given use LIMIT IOC, else MARKET
                let mut order_body = serde_json::Map::new();
                if let Some(p) = price {
                    order_body.insert("type".to_string(), json!("LIMIT"));
                    order_body.insert("price".to_string(), json!(p.to_string()));
                } else {
                    order_body.insert("type".to_string(), json!("MARKET"));
                }
                order_body.insert("instrument".to_string(), json!(instrument));
                order_body.insert("units".to_string(), json!(units_str));
                order_body.insert("timeInForce".to_string(), json!("IOC"));
                order_body.insert("positionFill".to_string(), json!("DEFAULT"));

                let b = json!({ "order": serde_json::Value::Object(order_body) });
                (b, OrderType::Ioc { price }, price, None, crate::core::TimeInForce::Ioc)
            }

            OrderType::Fok { price } => {
                let b = json!({
                    "order": {
                        "type": "LIMIT",
                        "instrument": instrument,
                        "units": units_str,
                        "price": price.to_string(),
                        "timeInForce": "FOK",
                        "positionFill": "DEFAULT"
                    }
                });
                (b, OrderType::Fok { price }, Some(price), None, crate::core::TimeInForce::Fok)
            }

            OrderType::Gtd { price, expire_time } => {
                // OANDA GTD requires RFC3339 timestamp in gtdTime field
                let expire_rfc3339 = {
                    use std::time::{UNIX_EPOCH, Duration};
                    let secs = (expire_time / 1000) as u64;
                    let millis = (expire_time % 1000) as u32;
                    let dt = UNIX_EPOCH + Duration::from_secs(secs) + Duration::from_millis(millis as u64);
                    // Format as RFC3339 — use chrono
                    chrono::DateTime::<chrono::Utc>::from(dt).to_rfc3339()
                };

                let b = json!({
                    "order": {
                        "type": "LIMIT",
                        "instrument": instrument,
                        "units": units_str,
                        "price": price.to_string(),
                        "timeInForce": "GTD",
                        "gtdTime": expire_rfc3339,
                        "positionFill": "DEFAULT"
                    }
                });
                (b, OrderType::Gtd { price, expire_time }, Some(price), None, crate::core::TimeInForce::Gtd)
            }

            OrderType::ReduceOnly { price } => {
                // OANDA supports reduceOnly flag on market or limit orders
                let (order_type_str, tif_str) = if price.is_some() {
                    ("LIMIT", "GTC")
                } else {
                    ("MARKET", "FOK")
                };

                let mut order_body = serde_json::Map::new();
                order_body.insert("type".to_string(), json!(order_type_str));
                order_body.insert("instrument".to_string(), json!(instrument));
                order_body.insert("units".to_string(), json!(units_str));
                order_body.insert("timeInForce".to_string(), json!(tif_str));
                order_body.insert("positionFill".to_string(), json!("REDUCE_ONLY"));
                if let Some(p) = price {
                    order_body.insert("price".to_string(), json!(p.to_string()));
                }

                let b = json!({ "order": serde_json::Value::Object(order_body) });
                let tif = if price.is_some() {
                    crate::core::TimeInForce::Gtc
                } else {
                    crate::core::TimeInForce::Fok
                };
                (b, OrderType::ReduceOnly { price }, price, None, tif)
            }

            // Unsupported order types
            OrderType::Oco { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "OANDA does not support native OCO orders".to_string()
                ));
            }
            OrderType::PostOnly { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "OANDA does not support Post-Only orders".to_string()
                ));
            }
            OrderType::Iceberg { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "OANDA does not support Iceberg orders".to_string()
                ));
            }
            OrderType::Twap { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "OANDA does not support TWAP orders".to_string()
                ));
            }

            OrderType::Oto { .. } | OrderType::ConditionalPlan { .. } | OrderType::DcaRecurring { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Oto/ConditionalPlan/DcaRecurring orders are not supported on OANDA".to_string()
                ));
            }
        };

        let response = self.post(endpoint, body).await?;
        let order_id = OandaParser::parse_order_id(&response)?;

        Ok(PlaceOrderResponse::Simple(Order {
            id: order_id,
            client_order_id: req.client_order_id,
            symbol: symbol.to_string(),
            side,
            order_type: result_order_type,
            status: crate::core::OrderStatus::New,
            price: result_price,
            stop_price: result_stop_price,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: result_tif,
        }))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let account_id = self.require_account_id()?;

                let endpoint = OandaEndpoint::CancelOrder {
                    account_id: account_id.to_string(),
                    order_id: order_id.to_string(),
                };

                // OANDA returns the cancelled order in the response
                let response = self.put(endpoint, json!({})).await?;

                // Try to parse from the cancel response, fall back to minimal stub
                if let Ok(order) = OandaParser::parse_order(&response, "") {
                    return Ok(order);
                }

                Ok(Order {
                    id: order_id.to_string(),
                    client_order_id: None,
                    symbol: req.symbol
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
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
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on OANDA — only Single is supported", req.scope)
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let account_id = self.require_account_id()?;

        let endpoint = OandaEndpoint::GetOrder {
            account_id: account_id.to_string(),
            order_id: order_id.to_string(),
        };

        let response = self.get(endpoint, HashMap::new()).await?;
        OandaParser::parse_order(&response, _symbol)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let account_id = self.require_account_id()?;
        let endpoint = OandaEndpoint::ListPendingOrders(account_id.to_string());

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            // Parse "EUR/USD" or "EUR_USD" formats
            let instrument = if s.contains('/') {
                let parts: Vec<&str> = s.splitn(2, '/').collect();
                if parts.len() == 2 {
                    format_symbol(parts[0], parts[1])
                } else {
                    s.to_string()
                }
            } else {
                s.to_string()
            };
            params.insert("instrument".to_string(), instrument);
        }

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let account_id = self.require_account_id()?;
        let endpoint = OandaEndpoint::ListOrders(account_id.to_string());

        let mut params = HashMap::new();
        // OANDA state parameter: PENDING, FILLED, TRIGGERED, CANCELLED
        params.insert("state".to_string(), "FILLED".to_string());

        if let Some(limit) = filter.limit {
            params.insert("count".to_string(), limit.to_string());
        } else {
            params.insert("count".to_string(), "50".to_string());
        }

        if let Some(sym) = filter.symbol {
            let instrument = format_symbol(&sym.base, &sym.quote);
            params.insert("instrument".to_string(), instrument);
        }

        let response = self.get(endpoint, params).await?;
        OandaParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for OandaConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let account_id = self.require_account_id()?;
        let endpoint = OandaEndpoint::GetAccountSummary(account_id.to_string());

        let response = self.get(endpoint, HashMap::new()).await?;
        OandaParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let account_id = self.require_account_id()?;
        let endpoint = OandaEndpoint::GetAccountSummary(account_id.to_string());

        let response = self.get(endpoint, HashMap::new()).await?;
        OandaParser::parse_account_info(&response)
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
impl Positions for OandaConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let _account_type = query.account_type;

        let account_id = self.require_account_id()?;

        if let Some(s) = symbol {
            let instrument = format_symbol(&s.base, &s.quote);
            let endpoint = OandaEndpoint::GetPosition {
                account_id: account_id.to_string(),
                instrument,
            };

            let response = self.get(endpoint, HashMap::new()).await?;
            OandaParser::parse_position(&response).map(|p| vec![p])
        } else {
            let endpoint = OandaEndpoint::ListOpenPositions(account_id.to_string());
            let response = self.get(endpoint, HashMap::new()).await?;
            OandaParser::parse_positions(&response)
        }
    
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let _symbol_str = _symbol;
        let _symbol = {
            let parts: Vec<&str> = _symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: _symbol_str.to_string(), quote: String::new(), raw: Some(_symbol_str.to_string()) }
            }
        };

        // OANDA doesn't have funding rates (not perpetual futures)
        Err(ExchangeError::UnsupportedOperation(
            "OANDA does not have funding rates (forex broker)".to_string()
        ))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::ClosePosition { symbol, account_type: _ } => {
                let account_id = self.require_account_id()?;
                let instrument = format_symbol(&symbol.base, &symbol.quote);

                let endpoint = OandaEndpoint::ClosePosition {
                    account_id: account_id.to_string(),
                    instrument,
                };

                // Close both long and short sides
                let body = json!({
                    "longUnits": "ALL",
                    "shortUnits": "ALL"
                });

                self.put(endpoint, body).await?;
                Ok(())
            }

            PositionModification::SetLeverage { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "OANDA leverage is set at account level, not per symbol".to_string()
                ))
            }

            PositionModification::SetMarginMode { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "OANDA does not support per-symbol margin mode (uses account-level cross margin)".to_string()
                ))
            }

            PositionModification::AddMargin { .. } | PositionModification::RemoveMargin { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "OANDA does not support manual margin adjustment (auto cross-margin)".to_string()
                ))
            }

            PositionModification::SetTpSl { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "OANDA TP/SL is set on individual trades, not positions — use place_order with Bracket".to_string()
                ))
            }

            PositionModification::SwitchPositionMode { .. } | PositionModification::MovePositions { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SwitchPositionMode/MovePositions not supported on OANDA".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

/// OANDA supports native order amendment — PUT to /v3/accounts/{id}/orders/{specifier}
/// replaces the entire order in-place (cancel + recreate on the server side, but
/// atomic and preserves the order ID).
#[async_trait]
impl AmendOrder for OandaConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        if req.fields.price.is_none()
            && req.fields.quantity.is_none()
            && req.fields.trigger_price.is_none()
        {
            return Err(ExchangeError::InvalidRequest(
                "AmendOrder: at least one field must be specified".to_string()
            ));
        }

        let account_id = self.require_account_id()?;
        let instrument = format_symbol(&req.symbol.base, &req.symbol.quote);

        // First fetch the current order so we can rebuild the full replacement body
        let current_endpoint = OandaEndpoint::GetOrder {
            account_id: account_id.to_string(),
            order_id: req.order_id.clone(),
        };
        let current_response = self.get(current_endpoint, HashMap::new()).await?;
        let current_order = OandaParser::parse_order(&current_response, &instrument)?;

        // Determine effective values after the amendment
        let new_price = req.fields.price.unwrap_or(current_order.price.unwrap_or(0.0));
        let new_quantity = req.fields.quantity.unwrap_or(current_order.quantity);
        let new_stop = req.fields.trigger_price.or(current_order.stop_price);

        let units = match current_order.side {
            OrderSide::Buy => new_quantity,
            OrderSide::Sell => -new_quantity,
        };

        // Rebuild the order body based on order type
        let order_type_str = match &current_order.order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit { .. } => "LIMIT",
            OrderType::StopMarket { .. } | OrderType::StopLimit { .. } => "STOP",
            _ => "LIMIT",
        };

        let mut order_body = serde_json::Map::new();
        order_body.insert("type".to_string(), json!(order_type_str));
        order_body.insert("instrument".to_string(), json!(instrument));
        order_body.insert("units".to_string(), json!(units.to_string()));
        order_body.insert("timeInForce".to_string(), json!("GTC"));
        order_body.insert("positionFill".to_string(), json!("DEFAULT"));

        if new_price != 0.0 {
            order_body.insert("price".to_string(), json!(new_price.to_string()));
        }
        if let Some(stop) = new_stop {
            order_body.insert("priceBound".to_string(), json!(stop.to_string()));
        }

        let body = json!({ "order": serde_json::Value::Object(order_body) });

        let endpoint = OandaEndpoint::AmendOrder {
            account_id: account_id.to_string(),
            order_id: req.order_id.clone(),
        };

        let response = self.put(endpoint, body).await?;
        OandaParser::parse_order(&response, &instrument)
    }
}
