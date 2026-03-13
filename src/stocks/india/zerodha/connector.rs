//! Zerodha Kite Connect Connector
//!
//! Implements all core traits for Zerodha broker API.

use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient,
    ExchangeId, AccountType, Symbol, ExchangeType,
    ExchangeError, ExchangeResult,
    Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, OrderStatus, TimeInForce, Price, Quantity,
    Balance, AccountInfo, Position, FundingRate, Asset,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    AmendRequest,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions, AmendOrder,
};
use crate::core::types::SymbolInfo;

use super::endpoints::{ZerodhaEndpoints, ZerodhaEndpoint, format_symbol};
use super::auth::ZerodhaAuth;
use super::parser::ZerodhaParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Zerodha Kite Connect connector
pub struct ZerodhaConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: ZerodhaAuth,
    /// Base URLs
    endpoints: ZerodhaEndpoints,
}

impl ZerodhaConnector {
    /// Create new connector with explicit auth
    pub fn new(auth: ZerodhaAuth) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let endpoints = ZerodhaEndpoints::default();

        Ok(Self {
            http,
            auth,
            endpoints,
        })
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        let auth = ZerodhaAuth::from_env();
        Self::new(auth.clone()).unwrap_or_else(|_| {
            // Fallback if HTTP client creation fails
            Self {
                http: HttpClient::new(30_000).expect("HTTP client creation should not fail with valid timeout"),
                auth,
                endpoints: ZerodhaEndpoints::default(),
            }
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// GET request
    async fn get(
        &self,
        endpoint: ZerodhaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let base_url = self.endpoints.rest_base;
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
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let response = self.http.get(&url, &headers).await?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: ZerodhaEndpoint,
        body: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let base_url = self.endpoints.rest_base;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Add auth headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());

        // Convert HashMap to JSON Value (the post method will handle it)
        let json_body = json!(body);
        let response = self.http.post(&url, &json_body, &headers).await?;
        Ok(response)
    }

    /// PUT request
    async fn put(
        &self,
        endpoint: ZerodhaEndpoint,
        body: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let base_url = self.endpoints.rest_base;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Add auth headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());

        // Convert HashMap to JSON Value (the put method will handle it)
        let json_body = json!(body);
        let response = self.http.put(&url, &json_body, &headers).await?;
        Ok(response)
    }

    /// DELETE request
    async fn delete(
        &self,
        endpoint: ZerodhaEndpoint,
    ) -> ExchangeResult<Value> {
        let base_url = self.endpoints.rest_base;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Add auth headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — GTT (Good Till Triggered)
    // ═══════════════════════════════════════════════════════════════════════════

    /// List all GTT triggers — `GET /gtt/triggers`
    pub async fn gtt_list(&self) -> ExchangeResult<Value> {
        self.get(ZerodhaEndpoint::GetGtts, HashMap::new()).await
    }

    /// Delete a GTT trigger — `DELETE /gtt/triggers/{trigger_id}`
    pub async fn gtt_delete(&self, trigger_id: u64) -> ExchangeResult<Value> {
        self.delete(ZerodhaEndpoint::DeleteGtt(trigger_id)).await
    }

    /// Modify a GTT trigger — `PUT /gtt/triggers/{trigger_id}`
    ///
    /// `body` — form parameters for the updated GTT trigger.
    pub async fn gtt_modify(
        &self,
        trigger_id: u64,
        body: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.put(ZerodhaEndpoint::ModifyGtt(trigger_id), body).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Instruments Master
    // ═══════════════════════════════════════════════════════════════════════════

    /// Download instruments master CSV — `GET /instruments`
    ///
    /// Returns all tradeable instruments as a raw CSV string.
    /// Optionally filter by exchange (e.g. `"NSE"`, `"BSE"`, `"NFO"`).
    pub async fn get_instruments_master(&self, exchange: Option<&str>) -> ExchangeResult<Value> {
        let endpoint = match exchange {
            Some(ex) => ZerodhaEndpoint::InstrumentsExchange(ex.to_uppercase()),
            None => ZerodhaEndpoint::Instruments,
        };
        self.get(endpoint, HashMap::new()).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Basket Orders
    // ═══════════════════════════════════════════════════════════════════════════

    /// Place basket orders — `POST /orders/baskets`
    ///
    /// `orders` — list of order parameter maps (same fields as individual order placement).
    /// All orders in the basket are validated together; none execute if any fail validation.
    pub async fn place_basket_orders(
        &self,
        orders: Vec<HashMap<String, String>>,
    ) -> ExchangeResult<Value> {
        // Kite Connect basket API accepts JSON array
        let base_url = self.endpoints.rest_base;
        let path = ZerodhaEndpoint::BasketOrders.path();
        let url = format!("{}{}", base_url, path);

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = serde_json::Value::Array(
            orders.into_iter().map(|o| json!(o)).collect()
        );
        let response = self.http.post(&url, &body, &headers).await?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for ZerodhaConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Zerodha
    }

    fn is_testnet(&self) -> bool {
        false // Zerodha has no testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

#[async_trait]
impl MarketData for ZerodhaConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        let symbol_key = format_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("i".to_string(), symbol_key.clone());

        let response = self.get(ZerodhaEndpoint::QuoteLtp, params).await?;
        ZerodhaParser::parse_ltp(&response, &symbol_key)
    }

    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let symbol_key = format_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("i".to_string(), symbol_key.clone());

        let response = self.get(ZerodhaEndpoint::Quote, params).await?;
        ZerodhaParser::parse_quote(&response, &symbol_key)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let symbol_key = format_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("i".to_string(), symbol_key.clone());

        let response = self.get(ZerodhaEndpoint::Quote, params).await?;
        ZerodhaParser::parse_orderbook(&response, &symbol_key)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        _limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        // Note: Zerodha historical endpoint requires instrument_token
        // For now, return error indicating this needs instrument token lookup
        let _ = (symbol, interval);
        Err(ExchangeError::UnsupportedOperation(
            "Historical data requires instrument_token. Use get_instruments() first to map symbols to tokens.".to_string()
        ))
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Zerodha doesn't have a dedicated ping endpoint, so we use the user profile endpoint
        let _response = self.get(ZerodhaEndpoint::UserProfile, HashMap::new()).await?;
        Ok(())
    }

    /// Get exchange info — returns NSE equity instruments from Zerodha
    ///
    /// The /instruments endpoint returns CSV data. We parse it here.
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let url = format!("{}{}", self.endpoints.rest_base, ZerodhaEndpoint::InstrumentsExchange("NSE".to_string()).path());

        // Add auth headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let bytes = self.http.get_bytes(&url).await?;
        let csv_text = String::from_utf8(bytes)
            .map_err(|e| ExchangeError::Parse(format!("Invalid UTF-8 in instruments CSV: {}", e)))?;

        // CSV format: instrument_token,exchange_token,tradingsymbol,name,last_price,expiry,strike,tick_size,lot_size,instrument_type,segment,exchange
        let mut infos = Vec::new();
        for (i, line) in csv_text.lines().enumerate() {
            if i == 0 {
                continue; // skip header
            }
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 12 {
                continue;
            }
            let symbol = cols[2].trim().to_string();
            let instrument_type = cols[9].trim();
            let exchange = cols[11].trim();

            // Only EQ (equity) instruments from NSE
            if instrument_type != "EQ" || exchange != "NSE" {
                continue;
            }

            infos.push(SymbolInfo {
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
            });
        }

        Ok(infos)
    }
}

#[async_trait]
impl Trading for ZerodhaConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let time_in_force = req.time_in_force;

        let symbol_key = format_symbol(&symbol);
        let parts: Vec<&str> = symbol_key.split(':').collect();
        if parts.len() != 2 {
            return Err(ExchangeError::Parse(format!("Invalid symbol format: {}", symbol_key)));
        }
        let exchange = parts[0];
        let tradingsymbol = parts[1];

        // IOC validity string
        let validity = match time_in_force {
            TimeInForce::Ioc => "IOC",
            _ => "DAY",
        };

        match req.order_type {
            OrderType::Fok { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "FOK orders not supported by Zerodha Kite Connect".to_string(),
                ));
            }

            OrderType::Market => {
                let mut body = HashMap::new();
                body.insert("exchange".to_string(), exchange.to_string());
                body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
                body.insert("transaction_type".to_string(), match side {
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }.to_string());
                body.insert("quantity".to_string(), quantity.to_string());
                body.insert("order_type".to_string(), "MARKET".to_string());
                body.insert("product".to_string(), "CNC".to_string());
                body.insert("validity".to_string(), validity.to_string());

                let response = self.post(ZerodhaEndpoint::PlaceOrder("regular".to_string()), body).await?;
                ZerodhaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Limit { price } => {
                let mut body = HashMap::new();
                body.insert("exchange".to_string(), exchange.to_string());
                body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
                body.insert("transaction_type".to_string(), match side {
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }.to_string());
                body.insert("quantity".to_string(), quantity.to_string());
                body.insert("order_type".to_string(), "LIMIT".to_string());
                body.insert("price".to_string(), price.to_string());
                body.insert("product".to_string(), "CNC".to_string());
                body.insert("validity".to_string(), validity.to_string());

                let response = self.post(ZerodhaEndpoint::PlaceOrder("regular".to_string()), body).await?;
                ZerodhaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Ioc { price } => {
                // IOC: variety=regular, validity=IOC, order_type=MARKET or LIMIT
                let mut body = HashMap::new();
                body.insert("exchange".to_string(), exchange.to_string());
                body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
                body.insert("transaction_type".to_string(), match side {
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }.to_string());
                body.insert("quantity".to_string(), quantity.to_string());
                body.insert("product".to_string(), "CNC".to_string());
                body.insert("validity".to_string(), "IOC".to_string());

                if let Some(p) = price {
                    body.insert("order_type".to_string(), "LIMIT".to_string());
                    body.insert("price".to_string(), p.to_string());
                } else {
                    body.insert("order_type".to_string(), "MARKET".to_string());
                }

                let response = self.post(ZerodhaEndpoint::PlaceOrder("regular".to_string()), body).await?;
                ZerodhaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopMarket { stop_price } => {
                // SL-M order: variety=regular, order_type=SL-M, trigger_price
                let mut body = HashMap::new();
                body.insert("exchange".to_string(), exchange.to_string());
                body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
                body.insert("transaction_type".to_string(), match side {
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }.to_string());
                body.insert("quantity".to_string(), quantity.to_string());
                body.insert("order_type".to_string(), "SL-M".to_string());
                body.insert("trigger_price".to_string(), stop_price.to_string());
                body.insert("product".to_string(), "CNC".to_string());
                body.insert("validity".to_string(), validity.to_string());

                let response = self.post(ZerodhaEndpoint::PlaceOrder("regular".to_string()), body).await?;
                ZerodhaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // SL order: variety=regular, order_type=SL, price + trigger_price
                let mut body = HashMap::new();
                body.insert("exchange".to_string(), exchange.to_string());
                body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
                body.insert("transaction_type".to_string(), match side {
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }.to_string());
                body.insert("quantity".to_string(), quantity.to_string());
                body.insert("order_type".to_string(), "SL".to_string());
                body.insert("price".to_string(), limit_price.to_string());
                body.insert("trigger_price".to_string(), stop_price.to_string());
                body.insert("product".to_string(), "CNC".to_string());
                body.insert("validity".to_string(), validity.to_string());

                let response = self.post(ZerodhaEndpoint::PlaceOrder("regular".to_string()), body).await?;
                ZerodhaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Bracket { price, stop_loss, .. } => {
                // Cover Order (CO): variety=co, entry price + SL trigger_price
                // Note: Kite's CO is entry + SL only — no TP leg. take_profit is ignored.
                let entry_price = price.ok_or_else(|| {
                    ExchangeError::InvalidRequest(
                        "Bracket orders on Zerodha (Cover Order) require an entry price".to_string(),
                    )
                })?;

                let mut body = HashMap::new();
                body.insert("exchange".to_string(), exchange.to_string());
                body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
                body.insert("transaction_type".to_string(), match side {
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }.to_string());
                body.insert("quantity".to_string(), quantity.to_string());
                body.insert("order_type".to_string(), "LIMIT".to_string());
                body.insert("price".to_string(), entry_price.to_string());
                body.insert("trigger_price".to_string(), stop_loss.to_string());
                body.insert("product".to_string(), "MIS".to_string()); // CO requires MIS
                body.insert("validity".to_string(), "DAY".to_string());

                let response = self.post(ZerodhaEndpoint::PlaceOrder("co".to_string()), body).await?;
                ZerodhaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Iceberg { price, display_quantity } => {
                // Iceberg order: variety=iceberg, iceberg_legs + iceberg_quantity
                let total_qty = quantity as u32;
                let display_qty = display_quantity as u32;
                // Zerodha requires iceberg_legs = ceil(total / display)
                let legs = (total_qty + display_qty - 1) / display_qty;
                let legs = legs.max(2).min(10); // Zerodha: 2–10 legs

                let mut body = HashMap::new();
                body.insert("exchange".to_string(), exchange.to_string());
                body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
                body.insert("transaction_type".to_string(), match side {
                    OrderSide::Buy => "BUY",
                    OrderSide::Sell => "SELL",
                }.to_string());
                body.insert("quantity".to_string(), quantity.to_string());
                body.insert("order_type".to_string(), "LIMIT".to_string());
                body.insert("price".to_string(), price.to_string());
                body.insert("product".to_string(), "CNC".to_string());
                body.insert("validity".to_string(), "DAY".to_string());
                body.insert("iceberg_legs".to_string(), legs.to_string());
                body.insert("iceberg_quantity".to_string(), display_quantity.to_string());

                let response = self.post(ZerodhaEndpoint::PlaceOrder("iceberg".to_string()), body).await?;
                ZerodhaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            other => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on Zerodha Kite Connect", other)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Zerodha returns today's full order log via GET /orders
        let response = self.get(ZerodhaEndpoint::GetOrders, HashMap::new()).await?;
        let all_orders = ZerodhaParser::parse_orders(&response)?;

        // Apply client-side filtering (Zerodha returns all of today's orders)
        let filtered: Vec<Order> = all_orders
            .into_iter()
            .filter(|o| {
                // Exclude open orders from history
                !matches!(o.status, OrderStatus::Open | OrderStatus::New | OrderStatus::PartiallyFilled)
            })
            .filter(|o| {
                // Apply status filter if provided
                if let Some(status) = &filter.status {
                    &o.status == status
                } else {
                    true
                }
            })
            .filter(|o| {
                // Apply symbol filter if provided
                if let Some(sym) = &filter.symbol {
                    let sym_str = format_symbol(sym);
                    let parts: Vec<&str> = sym_str.split(':').collect();
                    let trading_sym = if parts.len() == 2 { parts[1] } else { sym_str.as_str() };
                    o.symbol == trading_sym || o.symbol == sym_str
                } else {
                    true
                }
            })
            .take(filter.limit.unwrap_or(500) as usize)
            .collect();

        Ok(filtered)
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let _response = self.delete(
                ZerodhaEndpoint::CancelOrder("regular".to_string(), order_id.to_string())
            ).await?;

            // Return a basic success result
            Ok(Order {
                id: order_id.to_string(),
                client_order_id: None,
                symbol: String::new(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
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

        let response = self.get(
            ZerodhaEndpoint::GetOrder(order_id.to_string()),
            HashMap::new()
        ).await?;

        ZerodhaParser::parse_order(&response)
    
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let _symbol_str = _symbol;
        let _symbol: Option<crate::core::Symbol> = _symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let response = self.get(ZerodhaEndpoint::GetOrders, HashMap::new()).await?;
        let all_orders = ZerodhaParser::parse_orders(&response)?;

        // Filter for open orders
        Ok(all_orders.into_iter()
            .filter(|o| matches!(o.status, OrderStatus::Open))
            .collect())
    
    }
}

#[async_trait]
impl Account for ZerodhaConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get(ZerodhaEndpoint::GetMargins, HashMap::new()).await?;
        ZerodhaParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(ZerodhaEndpoint::UserProfile, HashMap::new()).await?;
        ZerodhaParser::parse_account_info(&response)
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}

#[async_trait]
impl Positions for ZerodhaConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let _account_type = query.account_type;

        let response = self.get(ZerodhaEndpoint::Positions, HashMap::new()).await?;
        ZerodhaParser::parse_positions(&response)
    
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

        // Zerodha is a stock broker, not futures exchange
        Err(ExchangeError::UnsupportedOperation(
            "Funding rates not supported for stock broker".to_string()
        ))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                // Zerodha is a stock broker, leverage is product-specific (MIS/NRML)
                Err(ExchangeError::UnsupportedOperation(
                "Leverage setting not supported. Use product types (MIS/NRML) instead".to_string()
                ))
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (Zerodha supports PUT /orders/{variety}/{order_id})
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for ZerodhaConnector {
    /// Modify a live order via PUT /orders/{variety}/{order_id}.
    ///
    /// Zerodha's modify endpoint accepts variety (regular/amo/co/iceberg)
    /// and optional price, quantity, trigger_price fields.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        if req.fields.price.is_none()
            && req.fields.quantity.is_none()
            && req.fields.trigger_price.is_none()
        {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price, quantity, or trigger_price must be provided".to_string(),
            ));
        }

        let mut body = HashMap::new();

        if let Some(price) = req.fields.price {
            body.insert("price".to_string(), price.to_string());
        }
        if let Some(qty) = req.fields.quantity {
            body.insert("quantity".to_string(), qty.to_string());
        }
        if let Some(trigger) = req.fields.trigger_price {
            body.insert("trigger_price".to_string(), trigger.to_string());
        }

        // Default variety to "regular"; callers can pass variety in order_id prefix
        // convention "regular:{order_id}" for CO/iceberg if needed.
        let (variety, order_id) = if let Some(sep) = req.order_id.find(':') {
            let v = req.order_id[..sep].to_string();
            let id = req.order_id[sep + 1..].to_string();
            (v, id)
        } else {
            ("regular".to_string(), req.order_id.clone())
        };

        let response = self
            .put(ZerodhaEndpoint::ModifyOrder(variety, order_id.clone()), body)
            .await?;

        // Zerodha's modify response just returns the order_id confirmation
        // Fetch full order details to return a complete Order
        let updated = self
            .get(ZerodhaEndpoint::GetOrder(order_id), HashMap::new())
            .await?;
        ZerodhaParser::parse_order(&updated)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Zerodha-specific)
// ═══════════════════════════════════════════════════════════════════════════════

impl ZerodhaConnector {
    /// Get holdings (long-term delivery portfolio)
    pub async fn get_holdings(&self) -> ExchangeResult<Value> {
        self.get(ZerodhaEndpoint::Holdings, HashMap::new()).await
    }

    /// Convert position between products (CNC/MIS/NRML)
    pub async fn convert_position(
        &self,
        exchange: &str,
        tradingsymbol: &str,
        transaction_type: &str,
        quantity: f64,
        old_product: &str,
        new_product: &str,
    ) -> ExchangeResult<Value> {
        let mut body = HashMap::new();
        body.insert("exchange".to_string(), exchange.to_string());
        body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
        body.insert("transaction_type".to_string(), transaction_type.to_string());
        body.insert("quantity".to_string(), quantity.to_string());
        body.insert("old_product".to_string(), old_product.to_string());
        body.insert("new_product".to_string(), new_product.to_string());

        self.put(ZerodhaEndpoint::ConvertPosition, body).await
    }
}
