//! # AlphaVantage Connector Implementation
//!
//! AlphaVantage is a DATA PROVIDER ONLY - no trading, account, or position management.

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// AlphaVantage connector
pub struct AlphaVantageConnector {
    client: Client,
    auth: AlphaVantageAuth,
    endpoints: AlphaVantageEndpoints,
}

impl AlphaVantageConnector {
    /// Create new connector with authentication
    pub fn new(auth: AlphaVantageAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: AlphaVantageEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Looks for `ALPHAVANTAGE_API_KEY` environment variable.
    pub fn from_env() -> Self {
        Self::new(AlphaVantageAuth::from_env())
    }

    /// Create connector with demo API key
    ///
    /// Demo key only works with IBM stock, not forex.
    pub fn demo() -> Self {
        Self::new(AlphaVantageAuth::demo())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // INTERNAL HTTP METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Make GET request to AlphaVantage API
    async fn get(
        &self,
        function: AlphaVantageFunction,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add function parameter
        params.insert("function".to_string(), function.as_str().to_string());

        // Add API key
        self.auth.add_to_params(&mut params);

        // Make request
        let response = self
            .client
            .get(self.endpoints.rest_base)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(ExchangeError::Http(format!(
                "HTTP {}",
                response.status()
            )));
        }

        // Parse JSON
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors in response body
        AlphaVantageParser::check_error(&json)?;

        Ok(json)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for AlphaVantageConnector {
    fn exchange_name(&self) -> &'static str {
        "alphavantage"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::AlphaVantage
    }

    fn is_testnet(&self) -> bool {
        false // No testnet for AlphaVantage
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Data provider - treat as Spot equivalent
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for AlphaVantageConnector {
    /// Get current exchange rate for forex pair
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<f64> {
        let (from, to) = format_fx_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("from_currency".to_string(), from);
        params.insert("to_currency".to_string(), to);

        let response = self
            .get(AlphaVantageFunction::CurrencyExchangeRate, params)
            .await?;

        AlphaVantageParser::parse_exchange_rate(&response)
    }

    /// Get ticker - NOT SUPPORTED for forex
    ///
    /// AlphaVantage doesn't provide 24h ticker statistics for forex pairs.
    async fn get_ticker(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage FX doesn't provide 24h ticker stats - use get_price() instead"
                .to_string(),
        ))
    }

    /// Get orderbook - NOT SUPPORTED
    ///
    /// AlphaVantage is a data provider, not an exchange - no orderbook.
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - no orderbook data".to_string(),
        ))
    }

    /// Get klines/candles for forex pair
    ///
    /// Supports daily, weekly, and monthly intervals on free tier.
    /// Intraday intervals (1m, 5m, 15m, 30m, 60m) require premium tier.
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let (from, to) = format_fx_symbol(&symbol);

        let mut params = HashMap::new();
        params.insert("from_symbol".to_string(), from);
        params.insert("to_symbol".to_string(), to);

        // Determine function based on interval
        let (_function, response_klines) = if interval.contains('m') || interval.contains('h') {
            // Intraday - requires premium tier
            let av_interval = map_interval(interval);
            params.insert("interval".to_string(), av_interval.to_string());

            let response = self
                .get(AlphaVantageFunction::FxIntraday, params)
                .await?;

            let klines = AlphaVantageParser::parse_fx_intraday(&response, av_interval)?;
            (AlphaVantageFunction::FxIntraday, klines)
        } else if interval.contains('w') || interval == "1w" {
            // Weekly
            let response = self.get(AlphaVantageFunction::FxWeekly, params).await?;
            let klines = AlphaVantageParser::parse_fx_weekly(&response)?;
            (AlphaVantageFunction::FxWeekly, klines)
        } else if interval.contains('M') || interval == "1M" {
            // Monthly
            let response = self.get(AlphaVantageFunction::FxMonthly, params).await?;
            let klines = AlphaVantageParser::parse_fx_monthly(&response)?;
            (AlphaVantageFunction::FxMonthly, klines)
        } else {
            // Daily (default)
            let response = self.get(AlphaVantageFunction::FxDaily, params).await?;
            let klines = AlphaVantageParser::parse_fx_daily(&response)?;
            (AlphaVantageFunction::FxDaily, klines)
        };

        // Apply limit (AlphaVantage returns all data, we truncate)
        let mut klines = response_klines;
        if let Some(lim) = limit {
            // Take most recent candles
            if klines.len() > lim as usize {
                klines = klines.split_off(klines.len() - lim as usize);
            }
        }

        Ok(klines)
    }

    /// Ping - check API connectivity
    async fn ping(&self) -> ExchangeResult<()> {
        // AlphaVantage doesn't have a dedicated ping endpoint
        // We can use a lightweight request like MARKET_STATUS
        let params = HashMap::new();
        let _ = self.get(AlphaVantageFunction::MarketStatus, params).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading - ALL UNSUPPORTED
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for AlphaVantageConnector {
    async fn market_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - trading not supported".to_string(),
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
            "AlphaVantage is a data provider - trading not supported".to_string(),
        ))
    }

    async fn cancel_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - trading not supported".to_string(),
        ))
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - trading not supported".to_string(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - trading not supported".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account - ALL UNSUPPORTED
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for AlphaVantageConnector {
    async fn get_balance(
        &self,
        _asset: Option<Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - account operations not supported".to_string(),
        ))
    }

    async fn get_account_info(
        &self,
        _account_type: AccountType,
    ) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - account operations not supported".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions - ALL UNSUPPORTED
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for AlphaVantageConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - position tracking not supported".to_string(),
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Forex doesn't have funding rates - not a derivatives market".to_string(),
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "AlphaVantage is a data provider - leverage operations not supported".to_string(),
        ))
    }
}
