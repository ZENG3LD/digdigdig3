//! # Fyers Connector
//!
//! Complete connector implementation for Fyers Securities API v3.
//!
//! Implements all core traits: MarketData, Trading, Account, Positions.

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::{json, Value};
use reqwest;

use crate::core::{
    timestamp_seconds, AccountInfo, AccountType, Asset, Balance, ExchangeError,
    ExchangeId, ExchangeResult, ExchangeType, FundingRate, HttpClient, Kline, Order, OrderBook,
    OrderSide, OrderType, Position, Price, Quantity, Symbol, Ticker,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{Account, ExchangeIdentity, MarketData, Positions, Trading};
use crate::core::types::SymbolInfo;

use super::auth::FyersAuth;
use super::endpoints::{format_symbol, map_kline_interval, FyersEndpoint, FyersUrls};
use super::parser::FyersParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Fyers connector
pub struct FyersConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: FyersAuth,
    /// Base URLs
    urls: FyersUrls,
}

impl FyersConnector {
    /// Create new connector with explicit auth
    pub fn new(auth: FyersAuth) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let urls = FyersUrls::PRODUCTION;

        Ok(Self { http, auth, urls })
    }

    /// Create connector from environment variables
    pub fn from_env() -> ExchangeResult<Self> {
        let auth = FyersAuth::from_env();
        Self::new(auth)
    }

    /// Create connector with access token
    pub fn with_token(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        access_token: impl Into<String>,
    ) -> ExchangeResult<Self> {
        let auth = FyersAuth::with_token(app_id, app_secret, access_token);
        Self::new(auth)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// GET request
    async fn get(
        &self,
        endpoint: FyersEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url(endpoint.is_data_endpoint());
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Add auth headers
        let mut headers = HashMap::new();
        if endpoint.requires_auth() {
            self.auth.sign_headers(&mut headers);
        }

        let response = self.http.get(&url, &headers).await?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: FyersEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url(endpoint.is_data_endpoint());
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Add auth headers
        let mut headers = HashMap::new();
        if endpoint.requires_auth() {
            self.auth.sign_headers(&mut headers);
        }
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.post(&url, &body, &headers).await?;
        Ok(response)
    }

    /// PUT request
    async fn put(
        &self,
        endpoint: FyersEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url(endpoint.is_data_endpoint());
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Add auth headers
        let mut headers = HashMap::new();
        if endpoint.requires_auth() {
            self.auth.sign_headers(&mut headers);
        }
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.put(&url, &body, &headers).await?;
        Ok(response)
    }

    /// DELETE request
    async fn delete(
        &self,
        endpoint: FyersEndpoint,
        params: HashMap<String, String>,
        _body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url(endpoint.is_data_endpoint());
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Add auth headers
        let mut headers = HashMap::new();
        if endpoint.requires_auth() {
            self.auth.sign_headers(&mut headers);
        }
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE TRAITS IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for FyersConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Fyers
    }

    fn is_testnet(&self) -> bool {
        false // Fyers has no testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // Fyers supports equity, F&O, commodities
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex // Broker treated as CEX
    }
}

#[async_trait]
impl MarketData for FyersConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("symbols".to_string(), symbol_str.clone());

        let response = self.get(FyersEndpoint::Quotes, params).await?;
        FyersParser::parse_ltp(&response, &symbol_str)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str.clone());
        params.insert("ohlcv_flag".to_string(), "1".to_string());

        let response = self.get(FyersEndpoint::Depth, params).await?;
        FyersParser::parse_orderbook(&response, &symbol_str)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
        let resolution = map_kline_interval(interval);

        // Calculate time range (default: last 100 candles)
        let now = timestamp_seconds();
        let limit = limit.unwrap_or(100) as u64;

        // Estimate seconds per candle
        let candle_seconds = match interval {
            "1m" => 60,
            "5m" => 300,
            "15m" => 900,
            "1h" => 3600,
            "1d" => 86400,
            _ => 3600,
        };

        let range_from = now - (limit * candle_seconds);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str);
        params.insert("resolution".to_string(), resolution);
        params.insert("date_format".to_string(), "0".to_string()); // Unix timestamp
        params.insert("range_from".to_string(), range_from.to_string());
        params.insert("range_to".to_string(), now.to_string());

        let response = self.get(FyersEndpoint::History, params).await?;
        FyersParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("symbols".to_string(), symbol_str.clone());

        let response = self.get(FyersEndpoint::Quotes, params).await?;
        FyersParser::parse_ticker(&response, &symbol_str)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Use market status as ping (public endpoint)
        let _response = self.get(FyersEndpoint::MarketStatus, HashMap::new()).await?;
        Ok(())
    }

    /// Get exchange info — returns NSE equity instruments from Fyers SymbolMaster
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // SymbolMaster returns a CSV file: fytoken,symbol,exchange,segment,description,lot_size,tick_size,...
        let base_url = self.urls.rest_url(true); // data endpoint
        let url = format!(
            "{}/data/symbol-master?exchange=NSE&segment=CM",
            base_url
        );

        // Add auth headers
        let mut headers = HashMap::new();
        if FyersEndpoint::SymbolMaster.requires_auth() {
            self.auth.sign_headers(&mut headers);
        }

        // Use reqwest directly to get text response
        let client = reqwest::Client::new();
        let mut req = client.get(&url);
        for (k, v) in &headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let response = req.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let csv_text = response.text().await
            .map_err(|e| ExchangeError::Network(format!("Failed to read text: {}", e)))?;

        // CSV format: fytoken,symbol,exchange,segment,description,lot_size,tick_size,isin,series,...
        let mut infos = Vec::new();
        for (i, line) in csv_text.lines().enumerate() {
            if i == 0 {
                continue; // skip header
            }
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 4 {
                continue;
            }

            let symbol = cols[1].trim().trim_matches('"').to_string();
            let segment = cols[3].trim();

            // Only Capital Market (equity) segment
            if segment != "CM" {
                continue;
            }

            // Symbol format is "NSE:SBIN-EQ", extract the ticker part
            let display_symbol = if let Some(colon_pos) = symbol.find(':') {
                symbol[colon_pos + 1..].to_string()
            } else {
                symbol.clone()
            };

            infos.push(SymbolInfo {
                symbol: display_symbol.clone(),
                base_asset: display_symbol,
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
impl Trading for FyersConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                
                        let body = json!({
                            "symbol": symbol_str,
                            "qty": quantity as i64,
                            "type": 2, // MARKET
                            "side": match side {
                                OrderSide::Buy => 1,
                                OrderSide::Sell => -1,
                            },
                            "productType": "INTRADAY",
                            "limitPrice": 0,
                            "stopPrice": 0,
                            "validity": "DAY",
                            "disclosedQty": 0,
                            "offlineOrder": false
                        });
                
                        let response = self.post(FyersEndpoint::PlaceOrder, body).await?;
                        FyersParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                
                        let body = json!({
                            "symbol": symbol_str,
                            "qty": quantity as i64,
                            "type": 1, // LIMIT
                            "side": match side {
                                OrderSide::Buy => 1,
                                OrderSide::Sell => -1,
                            },
                            "productType": "INTRADAY",
                            "limitPrice": price,
                            "stopPrice": 0,
                            "validity": "DAY",
                            "disclosedQty": 0,
                            "offlineOrder": false
                        });
                
                        let response = self.post(FyersEndpoint::PlaceOrder, body).await?;
                        FyersParser::parse_order(&response).map(PlaceOrderResponse::Simple)
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
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let body = json!({
                "id": order_id
            });

            let response = self.delete(FyersEndpoint::CancelOrder, HashMap::new(), body).await?;
            FyersParser::parse_order(&response)
    
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

        // Fyers doesn't have a single order endpoint, so we get all orders and filter
        let response = self.get(FyersEndpoint::GetOrders, HashMap::new()).await?;
        let orders = FyersParser::parse_orders(&response)?;

        orders
            .into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::NotFound(format!("Order {} not found", order_id)))
    
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

        let response = self.get(FyersEndpoint::GetOrders, HashMap::new()).await?;
        let mut orders = FyersParser::parse_orders(&response)?;

        // Filter for open orders
        orders.retain(|o| {
            matches!(
                o.status,
                crate::core::types::OrderStatus::Open | crate::core::types::OrderStatus::PartiallyFilled
            )
        });

        // Filter by symbol if provided
        if let Some(sym) = symbol {
            let symbol_str = format_symbol(&sym.base, &sym.quote, account_type);
            orders.retain(|o| o.symbol == symbol_str);
        }

        Ok(orders)
    
    }
}

#[async_trait]
impl Account for FyersConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get(FyersEndpoint::Funds, HashMap::new()).await?;
        FyersParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(FyersEndpoint::Profile, HashMap::new()).await?;
        FyersParser::parse_account_info(&response)
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}

#[async_trait]
impl Positions for FyersConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        let response = self.get(FyersEndpoint::Positions, HashMap::new()).await?;
        let mut positions = FyersParser::parse_positions(&response)?;

        // Filter by symbol if provided
        if let Some(sym) = symbol {
            let symbol_str = format_symbol(&sym.base, &sym.quote, account_type);
            positions.retain(|p| p.symbol == symbol_str);
        }

        Ok(positions)
    
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

        // Fyers is not a perpetual futures exchange
        // F&O contracts on NSE/BSE don't have funding rates
        Err(ExchangeError::UnsupportedOperation(
            "Funding rates not applicable for Indian F&O market".to_string(),
        ))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                // Leverage in Indian markets is product-specific (INTRADAY/MARGIN)
                // Not configurable per symbol
                Err(ExchangeError::UnsupportedOperation(
                "Leverage is product-specific in Indian markets. Use productType in orders.".to_string(),
                ))
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Fyers-specific)
// ═══════════════════════════════════════════════════════════════════════════════

impl FyersConnector {
    /// Get holdings (delivery portfolio)
    pub async fn get_holdings(&self) -> ExchangeResult<Value> {
        self.get(FyersEndpoint::Holdings, HashMap::new()).await
    }

    /// Get trade book (executed trades)
    pub async fn get_tradebook(&self) -> ExchangeResult<Value> {
        self.get(FyersEndpoint::Tradebook, HashMap::new()).await
    }

    /// Convert position between product types
    pub async fn convert_position(
        &self,
        symbol: &str,
        position_side: i32,
        convert_qty: f64,
        convert_from: &str,
        convert_to: &str,
    ) -> ExchangeResult<Value> {
        let body = json!({
            "symbol": symbol,
            "positionSide": position_side,
            "convertQty": convert_qty as i64,
            "convertFrom": convert_from,
            "convertTo": convert_to
        });

        self.put(FyersEndpoint::ConvertPosition, body).await
    }

    /// Modify existing order
    pub async fn modify_order(
        &self,
        order_id: &str,
        order_type: Option<i32>,
        limit_price: Option<f64>,
        quantity: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut body_map = serde_json::Map::new();
        body_map.insert("id".to_string(), json!(order_id));

        if let Some(t) = order_type {
            body_map.insert("type".to_string(), json!(t));
        }
        if let Some(p) = limit_price {
            body_map.insert("limitPrice".to_string(), json!(p));
        }
        if let Some(q) = quantity {
            body_map.insert("qty".to_string(), json!(q));
        }

        let body = Value::Object(body_map);
        self.put(FyersEndpoint::ModifyOrder, body).await
    }

    /// Exchange auth code for access token
    pub async fn exchange_auth_code(&mut self, auth_code: &str) -> ExchangeResult<String> {
        let body = json!(self.auth.prepare_token_request(auth_code));
        let response = self.post(FyersEndpoint::ValidateAuthCode, body).await?;

        let access_token = FyersParser::parse_access_token(&response)?;
        self.auth.set_access_token(access_token.clone());

        Ok(access_token)
    }

    /// Get authorization URL for OAuth flow
    pub fn get_authorization_url(&self, redirect_uri: &str, state: Option<&str>) -> String {
        self.auth.get_authorization_url(redirect_uri, state)
    }
}
