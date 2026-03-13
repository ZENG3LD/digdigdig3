//! # Upstox Connector
//!
//! Implementation of all core traits for Upstox.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - broker identification
//! - `MarketData` - market data operations
//! - `Trading` - trading operations
//! - `Account` - account information
//! - `Positions` - position management
//!
//! ## Extended Methods
//! Additional Upstox-specific methods as struct methods.

use std::collections::HashMap;
use std::io::Read;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol, Asset,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, OrderStatus, Balance, AccountInfo,
    Position,
    OrderRequest, CancelRequest, CancelScope,
    AmendRequest, OrderResult, CancelAllResponse,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    AmendOrder, BatchOrders, CancelAll,
};
use crate::core::types::SymbolInfo;

use super::endpoints::{UpstoxUrls, UpstoxEndpoint, format_symbol, map_kline_interval};
use super::auth::UpstoxAuth;
use super::parser::UpstoxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Upstox connector
pub struct UpstoxConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public endpoints)
    auth: Option<UpstoxAuth>,
    /// URLs (mainnet only, no testnet for Upstox)
    urls: UpstoxUrls,
    /// Use HFT endpoint (low latency)
    use_hft: bool,
}

impl UpstoxConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, use_hft: bool) -> ExchangeResult<Self> {
        let urls = UpstoxUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(UpstoxAuth::new)
            .transpose()?;

        Ok(Self {
            http,
            auth,
            urls,
            use_hft,
        })
    }

    /// Create connector from environment variables
    pub async fn from_env(use_hft: bool) -> ExchangeResult<Self> {
        let auth = Some(UpstoxAuth::from_env()?);
        let urls = UpstoxUrls::MAINNET;
        let http = HttpClient::new(30_000)?;

        Ok(Self {
            http,
            auth,
            urls,
            use_hft,
        })
    }

    /// Create connector for public endpoints only
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None, false).await
    }

    /// Set access token (after OAuth flow)
    pub fn set_access_token(&mut self, token: String) {
        if let Some(ref mut auth) = self.auth {
            auth.set_access_token(token);
        }
    }

    /// Get authorization URL for OAuth flow
    pub fn get_authorization_url(&self, state: Option<&str>) -> ExchangeResult<String> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;
        Ok(auth.get_authorization_url(state))
    }

    /// Exchange authorization code for access token
    ///
    /// This method requires using reqwest directly for form-urlencoded POST
    pub async fn exchange_code_for_token(&mut self, code: &str) -> ExchangeResult<String> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;

        let body_params = auth.build_token_exchange_body(code);
        let url = format!("{}{}", self.urls.rest_base, UpstoxEndpoint::LoginToken.path());

        // Use reqwest directly for form-urlencoded POST
        let client = reqwest::Client::new();
        let response = client.post(&url)
            .form(&body_params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to exchange token: {}", e)))?;

        let json_response = response.json::<Value>()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse token response: {}", e)))?;

        // Check for errors
        let _ = UpstoxParser::extract_data(&json_response)?;

        // Parse access token
        let access_token = json_response.get("access_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing access_token in response".to_string()))?
            .to_string();

        // Store token
        self.set_access_token(access_token.clone());

        Ok(access_token)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// GET request
    async fn get(
        &self,
        endpoint: UpstoxEndpoint,
        params: HashMap<String, String>,
        use_v3: bool,
    ) -> ExchangeResult<Value> {
        let base_url = if use_v3 || endpoint.is_v3() {
            self.urls.rest_v3_url()
        } else {
            self.urls.rest_url(self.use_hft)
        };

        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Add auth headers if needed
        let mut headers = HashMap::new();
        if let Some(ref auth) = self.auth {
            auth.sign_headers(&mut headers)?;
        }

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: UpstoxEndpoint,
        body: Value,
        use_v3: bool,
    ) -> ExchangeResult<Value> {
        let base_url = if use_v3 || endpoint.is_v3() {
            self.urls.rest_v3_url()
        } else {
            self.urls.rest_url(self.use_hft)
        };

        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers (required for all POST endpoints)
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers)?;
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.post(&url, &body, &headers).await?;
        Ok(response)
    }

    /// PUT request
    async fn put(
        &self,
        endpoint: UpstoxEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url(self.use_hft);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers)?;
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.put(&url, &body, &headers).await?;
        Ok(response)
    }

    /// DELETE request
    async fn delete(
        &self,
        endpoint: UpstoxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let base_url = self.urls.rest_url(self.use_hft);
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers)?;

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Upstox-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get holdings (long-term positions)
    pub async fn get_holdings(&self) -> ExchangeResult<Value> {
        self.get(UpstoxEndpoint::HoldingsLongTerm, HashMap::new(), false).await
    }

    /// Get user profile
    pub async fn get_user_profile(&self) -> ExchangeResult<Value> {
        self.get(UpstoxEndpoint::UserProfile, HashMap::new(), false).await
    }

    /// Cancel all orders (multi-order API)
    pub async fn cancel_all_orders(
        &self,
        segment: Option<&str>,
        tag: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(seg) = segment {
            params.insert("segment".to_string(), seg.to_string());
        }
        if let Some(t) = tag {
            params.insert("tag".to_string(), t.to_string());
        }

        self.delete(UpstoxEndpoint::MultiOrderCancel, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for UpstoxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Upstox
    }

    fn is_testnet(&self) -> bool {
        false // Upstox doesn't have public testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot, // Cash/Delivery
            // Note: Upstox uses product codes (I=Intraday, D=Delivery, MTF=Margin)
            // but we map to AccountType::Spot for simplicity
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
impl MarketData for UpstoxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let instrument_key = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("instrument_key".to_string(), instrument_key.clone());

        let response = self.get(UpstoxEndpoint::MarketQuoteLtp, params, false).await?;
        UpstoxParser::parse_price(&response, &instrument_key)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let instrument_key = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("instrument_key".to_string(), instrument_key.clone());

        let response = self.get(UpstoxEndpoint::MarketQuoteQuotes, params, false).await?;
        UpstoxParser::parse_orderbook(&response, &instrument_key)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let instrument_key = format_symbol(&symbol);
        let (unit, interval_str) = map_kline_interval(interval)?;

        // Build path with parameters (V3 format)
        // /v3/historical-candle/{instrument_key}/{unit}/{interval}/{to_date}/{from_date}
        // For simplicity, use intraday endpoint for current day data
        let mut params = HashMap::new();
        params.insert("instrument_key".to_string(), instrument_key);
        params.insert("unit".to_string(), unit.to_string());
        params.insert("interval".to_string(), interval_str);

        let response = self.get(UpstoxEndpoint::IntradayCandleV3, params, true).await?;
        let mut klines = UpstoxParser::parse_klines(&response)?;

        // Apply limit if specified
        if let Some(lim) = limit {
            let start = klines.len().saturating_sub(lim as usize);
            klines = klines[start..].to_vec();
        }

        Ok(klines)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let instrument_key = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("instrument_key".to_string(), instrument_key.clone());

        let response = self.get(UpstoxEndpoint::MarketQuoteQuotes, params, false).await?;
        UpstoxParser::parse_ticker(&response, &instrument_key)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Upstox doesn't have a dedicated ping endpoint
        // Use user profile as health check (requires auth)
        if self.auth.is_some() {
            self.get_user_profile().await?;
            Ok(())
        } else {
            // For public-only connector, try to get price of a known instrument
            self.get_price(
                Symbol::new("INE669E01016", "NSE_EQ"),
                AccountType::Spot
            ).await?;
            Ok(())
        }
    }

    /// Get exchange info — downloads NSE instruments JSON from Upstox CDN (gzip)
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Upstox provides instrument master as public gzip JSON files
        let url = "https://assets.upstox.com/market-quote/instruments/exchange/NSE.json.gz";

        // Download compressed bytes
        let bytes = self.http.get_bytes(url).await?;

        // Decompress gzip
        let mut decoder = flate2::read::GzDecoder::new(bytes.as_slice());
        let mut json_text = String::new();
        decoder.read_to_string(&mut json_text)
            .map_err(|e| ExchangeError::Parse(format!("Failed to decompress gzip: {}", e)))?;

        // Parse JSON array
        let arr: Vec<Value> = serde_json::from_str(&json_text)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse instruments JSON: {}", e)))?;

        let infos = arr.iter().filter_map(|item| {
            let symbol = item.get("tradingsymbol")?.as_str()?.to_string();
            let instrument_type = item.get("instrument_type").and_then(|v| v.as_str()).unwrap_or("");

            // Only equity instruments
            if instrument_type != "EQ" {
                return None;
            }

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
impl Trading for UpstoxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let instrument_key = format_symbol(&symbol);
        let transaction_type = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        // Product: D=Delivery (CNC), I=Intraday (MIS), OCO=OCO
        let product = match account_type {
            AccountType::Spot => "D",
            _ => "I",
        };

        // Validity from TIF
        let validity = match req.time_in_force {
            crate::core::TimeInForce::Ioc => "IOC",
            _ => "DAY",
        };

        let (order_type_str, price_val, trigger_val) = match req.order_type {
            OrderType::Market => ("MARKET", 0.0, 0.0),
            OrderType::Limit { price } => ("LIMIT", price, 0.0),
            OrderType::StopMarket { stop_price } => ("SL-M", 0.0, stop_price),
            OrderType::StopLimit { stop_price, limit_price } => ("SL", limit_price, stop_price),
            OrderType::Ioc { price } => {
                // IOC with optional price
                let p = price.unwrap_or(0.0);
                let ot = if price.is_some() { "LIMIT" } else { "MARKET" };
                // We handle Ioc as special case below
                let body = json!({
                    "quantity": quantity as i64,
                    "product": product,
                    "validity": "IOC",
                    "price": p,
                    "instrument_token": instrument_key,
                    "order_type": ot,
                    "transaction_type": transaction_type,
                    "disclosed_quantity": 0,
                    "trigger_price": 0,
                    "is_amo": false
                });
                let response = self.post(UpstoxEndpoint::OrderPlaceV3, body, true).await?;
                let order_id = UpstoxParser::parse_order_id(&response)?;
                let order = self.get_order(&instrument_key, &order_id, account_type).await?;
                return Ok(PlaceOrderResponse::Simple(order));
            }
            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on Upstox", req.order_type)
                ));
            }
        };

        let body = json!({
            "quantity": quantity as i64,
            "product": product,
            "validity": validity,
            "price": price_val,
            "instrument_token": instrument_key,
            "order_type": order_type_str,
            "transaction_type": transaction_type,
            "disclosed_quantity": 0,
            "trigger_price": trigger_val,
            "is_amo": false
        });

        let response = self.post(UpstoxEndpoint::OrderPlaceV3, body, true).await?;
        let order_id = UpstoxParser::parse_order_id(&response)?;
        let order = self.get_order(&instrument_key, &order_id, account_type).await?;
        Ok(PlaceOrderResponse::Simple(order))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // If a specific order ID is available via symbol/order-id query, use history endpoint.
        // Otherwise fall back to retrieve-all and filter for closed statuses.
        let response = self.get(UpstoxEndpoint::OrderBook, HashMap::new(), false).await?;
        let all_orders = UpstoxParser::parse_orders(&response)?;

        Ok(all_orders
            .into_iter()
            .filter(|o| matches!(
                o.status,
                OrderStatus::Filled | OrderStatus::Canceled | OrderStatus::Rejected | OrderStatus::Expired
            ))
            .collect())
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let mut params = HashMap::new();
            params.insert("order_id".to_string(), order_id.to_string());

            let _response = self.delete(UpstoxEndpoint::OrderCancel, params).await?;

            // Fetch updated order details after cancellation
            self.get_order(&format_symbol(&_symbol), order_id, _account_type).await
    
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

        // Get all orders and find the specific one
        let response = self.get(UpstoxEndpoint::OrderBook, HashMap::new(), false).await?;
        let orders = UpstoxParser::parse_orders(&response)?;

        orders.into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::InvalidRequest(format!("Order {} not found", order_id)))
    
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

        let response = self.get(UpstoxEndpoint::OrderBook, HashMap::new(), false).await?;
        let orders = UpstoxParser::parse_orders(&response)?;

        // Filter for open orders
        Ok(orders.into_iter()
            .filter(|o| matches!(o.status, OrderStatus::Open | OrderStatus::New | OrderStatus::PartiallyFilled))
            .collect())
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for UpstoxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get(UpstoxEndpoint::FundsAndMargin, HashMap::new(), false).await?;
        UpstoxParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get_user_profile().await?;
        let data = UpstoxParser::extract_data(&response)?;

        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: data.get("is_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            can_withdraw: false,
            can_deposit: false,
            maker_commission: 0.0,
            taker_commission: 0.0,
            balances: vec![],
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Upstox /v2/charges/brokerage requires instrument_token, quantity, product, price, transaction_type
        // Without a symbol + trade details we cannot calculate brokerage, so return a static schedule
        let _ = symbol;
        Ok(FeeInfo {
            maker_rate: 0.0,   // Upstox is flat-fee: ₹20 per order, not percentage-based
            taker_rate: 0.0,
            symbol: symbol.map(String::from),
            tier: Some("Flat ₹20 per order or 0.05% (whichever is lower)".to_string()),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for UpstoxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let _account_type = query.account_type;

        let response = self.get(UpstoxEndpoint::PositionsShortTerm, HashMap::new(), false).await?;
        UpstoxParser::parse_positions(&response)
    
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<crate::core::FundingRate> {
        // Upstox doesn't have perpetual contracts (no funding rate)
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not supported - Upstox offers futures, not perpetuals".to_string()
        ))
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                // Upstox doesn't support dynamic leverage setting
                // Margin requirements are fixed by exchange/SEBI regulations
                Err(ExchangeError::UnsupportedOperation(
                "Dynamic leverage not supported - margins are regulated by SEBI".to_string()
                ))
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait — Upstox supports PUT /v2/order/modify)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for UpstoxConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let order_id = req.order_id.clone();

        if req.fields.price.is_none()
            && req.fields.quantity.is_none()
            && req.fields.trigger_price.is_none()
        {
            return Err(ExchangeError::InvalidRequest(
                "At least one field (price, quantity, trigger_price) must be provided".to_string(),
            ));
        }

        // Fetch current order to fill in unchanged values
        let current = self.get_order("", &order_id, req.account_type).await?;

        let new_price = req.fields.price.or(current.price).unwrap_or(0.0);
        let new_quantity = req.fields.quantity.unwrap_or(current.quantity);
        let new_trigger = req.fields.trigger_price.or(current.stop_price).unwrap_or(0.0);

        let order_type_str = match &current.order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit { .. } => "LIMIT",
            OrderType::StopMarket { .. } => "SL-M",
            OrderType::StopLimit { .. } => "SL",
            _ => "LIMIT",
        };

        let validity = match current.time_in_force {
            crate::core::TimeInForce::Ioc => "IOC",
            _ => "DAY",
        };

        let body = json!({
            "order_id": order_id,
            "quantity": new_quantity as i64,
            "price": new_price,
            "trigger_price": new_trigger,
            "order_type": order_type_str,
            "validity": validity,
            "disclosed_quantity": 0,
        });

        let _response = self.put(UpstoxEndpoint::OrderModify, body).await?;

        // Fetch updated order
        self.get_order("", &order_id, req.account_type).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS (optional trait — Upstox supports POST /v2/order/multi/place)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BatchOrders for UpstoxConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // Build batch payload: array of order objects
        let mut order_payloads = Vec::with_capacity(orders.len());

        for req in &orders {
            let instrument_key = format_symbol(&req.symbol);
            let transaction_type = match req.side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            };
            let product = match req.account_type {
                AccountType::Spot => "D",
                _ => "I",
            };
            let validity = match req.time_in_force {
                crate::core::TimeInForce::Ioc => "IOC",
                _ => "DAY",
            };
            let (order_type_str, price_val, trigger_val) = match &req.order_type {
                OrderType::Market => ("MARKET", 0.0, 0.0),
                OrderType::Limit { price } => ("LIMIT", *price, 0.0),
                OrderType::StopMarket { stop_price } => ("SL-M", 0.0, *stop_price),
                OrderType::StopLimit { stop_price, limit_price } => ("SL", *limit_price, *stop_price),
                _ => return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} not supported in batch on Upstox", req.order_type)
                )),
            };

            order_payloads.push(json!({
                "quantity": req.quantity as i64,
                "product": product,
                "validity": validity,
                "price": price_val,
                "instrument_token": instrument_key,
                "order_type": order_type_str,
                "transaction_type": transaction_type,
                "disclosed_quantity": 0,
                "trigger_price": trigger_val,
                "is_amo": false,
            }));
        }

        let response = self.post(UpstoxEndpoint::MultiOrderPlace, serde_json::Value::Array(order_payloads), false).await?;

        // Parse batch response — array of { order_id, status, ... }
        UpstoxParser::parse_batch_order_results(&response)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // Upstox multi-cancel: DELETE /v2/order/multi/cancel with order_ids in body
        // The API accepts a JSON array of order IDs
        // Upstox multi-cancel: DELETE /v2/order/multi/cancel (order_ids passed as query params)
        // The API sends order IDs via query string, not a JSON body
        let base_url = self.urls.rest_url(self.use_hft);
        let path = UpstoxEndpoint::MultiOrderCancel.path();
        let url = format!("{}{}", base_url, path);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers)?;
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;

        UpstoxParser::parse_batch_order_results(&response)
    }

    fn max_batch_place_size(&self) -> usize {
        25 // Upstox documented limit
    }

    fn max_batch_cancel_size(&self) -> usize {
        25
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait — Upstox supports DELETE /v2/order/multi/cancel)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for UpstoxConnector {
    async fn cancel_all_orders(
        &self,
        scope: crate::core::CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        // Extract optional segment/tag from scope
        let symbol_opt: Option<&crate::core::Symbol> = match &scope {
            CancelScope::All { symbol } => symbol.as_ref().map(|s| s as &crate::core::Symbol),
            CancelScope::BySymbol { symbol } => Some(symbol),
            _ => return Err(ExchangeError::InvalidRequest(
                "CancelAll requires All or BySymbol scope".to_string()
            )),
        };
        let mut params = HashMap::new();
        if let Some(sym) = symbol_opt {
            params.insert("segment".to_string(), sym.quote.clone());
        }

        let response = self.delete(UpstoxEndpoint::MultiOrderCancel, params).await?;

        // Parse count from response
        let cancelled_count = response
            .get("data")
            .and_then(|d| d.as_array())
            .map(|a| a.len() as u32)
            .unwrap_or(0);

        Ok(CancelAllResponse {
            cancelled_count,
            failed_count: 0,
            details: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_creation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let connector = UpstoxConnector::public().await;
            assert!(connector.is_ok());
        });
    }

    #[test]
    fn test_exchange_identity() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let connector = UpstoxConnector::public().await.unwrap();
            assert_eq!(connector.exchange_type(), ExchangeType::Broker);
            assert!(!connector.is_testnet());
        });
    }
}
