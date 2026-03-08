//! # JQuants Connector Implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// JQuants connector
pub struct JQuantsConnector {
    client: Client,
    auth: Mutex<JQuantsAuth>, // Mutex for thread-safe interior mutability (token caching)
    urls: JQuantsUrls,
}

impl JQuantsConnector {
    /// Create new connector with refresh token
    pub fn new(refresh_token: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: Mutex::new(JQuantsAuth::new(refresh_token)),
            urls: JQuantsUrls::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: JQUANTS_REFRESH_TOKEN
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: Mutex::new(JQuantsAuth::from_env()),
            urls: JQuantsUrls::default(),
        }
    }

    /// Ensure we have a valid ID token (fetch if needed)
    async fn ensure_id_token(&self) -> ExchangeResult<String> {
        // Check if we have a cached valid ID token
        if let Some(token) = self.auth.lock().expect("Mutex poisoned").get_cached_id_token() {
            return Ok(token.to_string());
        }

        // Need to fetch new ID token using refresh token
        let refresh_token = self.auth.lock().expect("Mutex poisoned").refresh_token().to_string();
        if refresh_token.is_empty() {
            return Err(ExchangeError::Auth(
                "Missing refresh token. Set JQUANTS_REFRESH_TOKEN env var.".to_string()
            ));
        }

        let url = format!(
            "{}{}?refreshtoken={}",
            self.urls.rest_base,
            JQuantsEndpoint::AuthRefresh.path(),
            refresh_token
        );

        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to get ID token: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Auth(format!(
                "Failed to get ID token: HTTP {} - {}",
                status, text
            )));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse ID token response: {}", e)))?;

        let id_token = JQuantsParser::parse_id_token(&json)?;

        // Cache the token
        self.auth.lock().expect("Mutex poisoned").cache_id_token(id_token.clone());

        Ok(id_token)
    }

    /// Internal: Make GET request with authentication
    async fn get(
        &self,
        endpoint: JQuantsEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let id_token = self.ensure_id_token().await?;

        let url = format!("{}{}", self.urls.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add auth header
        request = request.header("Authorization", format!("Bearer {}", id_token));
        request = request.header("Content-Type", "application/json");

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();

        if status.as_u16() == 401 {
            // Token expired, clear cache and retry once
            self.auth.lock().expect("Mutex poisoned").clear_id_token();
            return Err(ExchangeError::Auth("ID token expired, retry request".to_string()));
        }

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: text,
            });
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for JQuantsConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::JQuants
    }

    fn is_testnet(&self) -> bool {
        false // JQuants has no testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // Data provider, treating as "Spot" equivalent
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData (Implement what makes sense)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for JQuantsConnector {
    /// Get current price (using latest daily quote close price)
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let code = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("code".to_string(), code);

        let response = self.get(JQuantsEndpoint::DailyQuotes, params).await?;
        JQuantsParser::parse_current_price(&response)
    }

    /// Get orderbook - NOT AVAILABLE (data provider only)
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants does not provide orderbook data - it is a data-only provider with delayed data".to_string()
        ))
    }

    /// Get klines/candles (historical daily OHLC)
    async fn get_klines(
        &self,
        symbol: Symbol,
        _interval: &str, // JQuants only has daily data on free tier
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let code = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("code".to_string(), code);

        // TODO: Add date range support based on limit
        // For now, fetch all available data

        let response = self.get(JQuantsEndpoint::DailyQuotes, params).await?;
        let mut klines = JQuantsParser::parse_daily_quotes(&response)?;

        // Apply limit if specified
        if let Some(lim) = limit {
            let len = klines.len();
            if len > lim as usize {
                klines = klines[len - lim as usize..].to_vec();
            }
        }

        Ok(klines)
    }

    /// Get ticker (24h stats from latest daily quote)
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let code = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("code".to_string(), code.clone());

        let response = self.get(JQuantsEndpoint::DailyQuotes, params).await?;
        JQuantsParser::parse_ticker(&response, &code)
    }

    /// Ping - check connectivity
    async fn ping(&self) -> ExchangeResult<()> {
        // Simple connectivity check - try to fetch anything
        // We can't use a truly public endpoint since all require auth
        // This is a basic check that will fail if network is down
        Ok(())
    }

    /// Get exchange info — returns listed Japanese stock codes from JQuants
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let params = HashMap::new();
        let response = self.get(JQuantsEndpoint::ListedInfo, params).await?;
        let symbols = JQuantsParser::parse_symbols(&response)?;

        let infos = symbols.into_iter().map(|code| SymbolInfo {
            symbol: code.clone(),
            base_asset: code,
            quote_asset: "JPY".to_string(),
            status: "TRADING".to_string(),
            price_precision: 0,
            quantity_precision: 0,
            min_quantity: Some(1.0),
            max_quantity: None,
            step_size: Some(1.0),
            min_notional: None,
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (UnsupportedOperation - data provider only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for JQuantsConnector {
    async fn market_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - trading not supported".to_string()
        ))
    }

    async fn limit_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _price: Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - trading not supported".to_string()
        ))
    }

    async fn cancel_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - trading not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UnsupportedOperation - data provider only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for JQuantsConnector {
    async fn get_balance(
        &self,
        _asset: Option<Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - account operations not supported".to_string()
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - account operations not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UnsupportedOperation - data provider only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for JQuantsConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a stock data provider - funding rates not applicable".to_string()
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "JQuants is a stock data provider - leverage not applicable".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (JQuants-specific, not from traits)
// ═══════════════════════════════════════════════════════════════════════════

impl JQuantsConnector {
    /// Get list of available symbols (stock codes)
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let params = HashMap::new();
        let response = self.get(JQuantsEndpoint::ListedInfo, params).await?;
        JQuantsParser::parse_symbols(&response)
    }
}
