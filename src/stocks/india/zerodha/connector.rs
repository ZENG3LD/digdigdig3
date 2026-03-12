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
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
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
    async fn market_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let symbol_key = format_symbol(&symbol);
        let parts: Vec<&str> = symbol_key.split(':').collect();

        if parts.len() != 2 {
            return Err(ExchangeError::Parse(format!("Invalid symbol format: {}", symbol_key)));
        }

        let exchange = parts[0];
        let tradingsymbol = parts[1];

        let mut body = HashMap::new();
        body.insert("exchange".to_string(), exchange.to_string());
        body.insert("tradingsymbol".to_string(), tradingsymbol.to_string());
        body.insert("transaction_type".to_string(), match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }.to_string());
        body.insert("quantity".to_string(), quantity.to_string());
        body.insert("order_type".to_string(), "MARKET".to_string());
        body.insert("product".to_string(), "CNC".to_string()); // Delivery

        let response = self.post(ZerodhaEndpoint::PlaceOrder("regular".to_string()), body).await?;
        ZerodhaParser::parse_order(&response)
    }

    async fn limit_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let symbol_key = format_symbol(&symbol);
        let parts: Vec<&str> = symbol_key.split(':').collect();

        if parts.len() != 2 {
            return Err(ExchangeError::Parse(format!("Invalid symbol format: {}", symbol_key)));
        }

        let exchange = parts[0];
        let tradingsymbol = parts[1];

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
        body.insert("product".to_string(), "CNC".to_string()); // Delivery

        let response = self.post(ZerodhaEndpoint::PlaceOrder("regular".to_string()), body).await?;
        ZerodhaParser::parse_order(&response)
    }

    async fn cancel_order(
        &self,
        _symbol: Symbol,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
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
            time_in_force: TimeInForce::GTC,
        })
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let response = self.get(
            ZerodhaEndpoint::GetOrder(order_id.to_string()),
            HashMap::new()
        ).await?;

        ZerodhaParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
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
    async fn get_balance(
        &self,
        _asset: Option<Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
        let response = self.get(ZerodhaEndpoint::GetMargins, HashMap::new()).await?;
        ZerodhaParser::parse_balance(&response)
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(ZerodhaEndpoint::UserProfile, HashMap::new()).await?;
        ZerodhaParser::parse_account_info(&response)
    }
}

#[async_trait]
impl Positions for ZerodhaConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        let response = self.get(ZerodhaEndpoint::Positions, HashMap::new()).await?;
        ZerodhaParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Zerodha is a stock broker, not futures exchange
        Err(ExchangeError::UnsupportedOperation(
            "Funding rates not supported for stock broker".to_string()
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        // Zerodha is a stock broker, leverage is product-specific (MIS/NRML)
        Err(ExchangeError::UnsupportedOperation(
            "Leverage setting not supported. Use product types (MIS/NRML) instead".to_string()
        ))
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
