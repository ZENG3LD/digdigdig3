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
    Order, OrderSide, OrderStatus, Balance, AccountInfo,
    Position,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
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
    async fn _put(
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
    async fn market_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let instrument_key = format_symbol(&symbol);
        let transaction_type = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        let body = json!({
            "quantity": quantity as i64,
            "product": "I", // Intraday
            "validity": "DAY",
            "price": 0,
            "instrument_token": instrument_key,
            "order_type": "MARKET",
            "transaction_type": transaction_type,
            "disclosed_quantity": 0,
            "trigger_price": 0,
            "is_amo": false
        });

        let response = self.post(UpstoxEndpoint::OrderPlaceV3, body, true).await?;
        let order_id = UpstoxParser::parse_order_id(&response)?;

        // Fetch order details
        self.get_order(symbol, &order_id, _account_type).await
    }

    async fn limit_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let instrument_key = format_symbol(&symbol);
        let transaction_type = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        let body = json!({
            "quantity": quantity as i64,
            "product": "I", // Intraday
            "validity": "DAY",
            "price": price,
            "instrument_token": instrument_key,
            "order_type": "LIMIT",
            "transaction_type": transaction_type,
            "disclosed_quantity": 0,
            "trigger_price": 0,
            "is_amo": false
        });

        let response = self.post(UpstoxEndpoint::OrderPlaceV3, body, true).await?;
        let order_id = UpstoxParser::parse_order_id(&response)?;

        // Fetch order details
        self.get_order(symbol, &order_id, _account_type).await
    }

    async fn cancel_order(
        &self,
        _symbol: Symbol,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("order_id".to_string(), order_id.to_string());

        let _response = self.delete(UpstoxEndpoint::OrderCancel, params).await?;

        // Fetch updated order details after cancellation
        self.get_order(_symbol, order_id, _account_type).await
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Get all orders and find the specific one
        let response = self.get(UpstoxEndpoint::OrderBook, HashMap::new(), false).await?;
        let orders = UpstoxParser::parse_orders(&response)?;

        orders.into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::InvalidRequest(format!("Order {} not found", order_id)))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
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
    async fn get_balance(
        &self,
        _asset: Option<Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
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
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for UpstoxConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        let response = self.get(UpstoxEndpoint::PositionsShortTerm, HashMap::new(), false).await?;
        UpstoxParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<crate::core::FundingRate> {
        // Upstox doesn't have perpetual contracts (no funding rate)
        Err(ExchangeError::NotSupported(
            "Funding rate not supported - Upstox offers futures, not perpetuals".to_string()
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        // Upstox doesn't support dynamic leverage setting
        // Margin requirements are fixed by exchange/SEBI regulations
        Err(ExchangeError::NotSupported(
            "Dynamic leverage not supported - margins are regulated by SEBI".to_string()
        ))
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
