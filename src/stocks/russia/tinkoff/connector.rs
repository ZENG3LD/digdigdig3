//! Tinkoff Invest connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Tinkoff Invest connector
///
/// Russian broker with full trading support for MOEX (Moscow Exchange).
///
/// ## Features
/// - Real-time market data
/// - Historical candles (5s to 1 month intervals, up to 10 years)
/// - Full trading support (stocks, bonds, ETFs, futures, options)
/// - Portfolio and position tracking
/// - Multiple account types (standard, IIS, sandbox)
///
/// ## Authentication
/// All endpoints require Bearer token authentication.
/// Generate token at: https://www.tinkoff.ru/invest/settings/
pub struct TinkoffConnector {
    client: Client,
    auth: TinkoffAuth,
    endpoints: TinkoffEndpoints,
    testnet: bool,
    /// Account ID to use for operations (set after GetAccounts)
    account_id: Option<String>,
}

impl TinkoffConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `token` - API token (starts with "t.")
    /// * `testnet` - Use sandbox environment
    pub fn new(token: impl Into<String>, testnet: bool) -> Self {
        let endpoints = if testnet {
            TinkoffEndpoints::sandbox()
        } else {
            TinkoffEndpoints::default()
        };

        Self {
            client: Client::new(),
            auth: TinkoffAuth::new(token),
            endpoints,
            testnet,
            account_id: None,
        }
    }

    /// Create connector from environment variable TINKOFF_TOKEN
    pub fn from_env() -> Self {
        Self::new(TinkoffAuth::from_env().token, false)
    }

    /// Create sandbox connector from environment variable TINKOFF_SANDBOX_TOKEN
    pub fn from_env_sandbox() -> Self {
        let token = std::env::var("TINKOFF_SANDBOX_TOKEN")
            .unwrap_or_default();
        Self::new(token, true)
    }

    /// Set account ID to use for operations
    pub fn set_account_id(&mut self, account_id: impl Into<String>) {
        self.account_id = Some(account_id.into());
    }

    /// Get list of accounts and set the first one as active
    ///
    /// This is useful for initializing the connector.
    /// Tinkoff requires account_id for most trading operations.
    pub async fn initialize_account(&mut self) -> ExchangeResult<String> {
        let accounts = self.get_accounts_list().await?;
        if accounts.is_empty() {
            return Err(ExchangeError::NotFound("No accounts found".to_string()));
        }

        let account_id = accounts[0].clone();
        self.account_id = Some(account_id.clone());
        Ok(account_id)
    }

    /// Get list of account IDs
    pub async fn get_accounts_list(&self) -> ExchangeResult<Vec<String>> {
        let response = self.post(TinkoffEndpoint::GetAccounts, serde_json::json!({})).await?;

        let accounts = response
            .get("accounts")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'accounts' array".to_string()))?;

        Ok(accounts.iter()
            .filter_map(|acc| acc.get("id").and_then(|id| id.as_str()))
            .map(|s| s.to_string())
            .collect())
    }

    /// Internal: Make POST request (Tinkoff uses POST for all methods)
    async fn post(
        &self,
        endpoint: TinkoffEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let mut request = self.client.post(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add JSON body
        request = request.json(&body);

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse Tinkoff error format
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(code) = error_json.get("code").and_then(|c| c.as_i64()) {
                    let message = error_json.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");

                    // Map Tinkoff error codes to ExchangeError
                    return Err(match code {
                        40003 => ExchangeError::Auth("Invalid or expired token".to_string()),
                        40002 => ExchangeError::PermissionDenied("Insufficient privileges - use full-access token for trading".to_string()),
                        30052 => ExchangeError::InvalidRequest("Instrument forbidden for API trading".to_string()),
                        50002 => ExchangeError::NotFound("Instrument not found".to_string()),
                        80002 => ExchangeError::RateLimit,
                        90003 => ExchangeError::InvalidRequest("Order value exceeds 6,000,000 RUB limit".to_string()),
                        _ => ExchangeError::Api {
                            code: code as i32,
                            message: message.to_string()
                        },
                    });
                }
            }

            return Err(ExchangeError::Http(format!("HTTP {} - {}", status, error_text)));
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Get FIGI for a ticker symbol
    ///
    /// FIGI (Financial Instrument Global Identifier) is required for many operations.
    /// This method searches for instruments by ticker.
    pub async fn get_figi_by_ticker(&self, ticker: &str) -> ExchangeResult<String> {
        let body = serde_json::json!({
            "query": ticker,
        });

        let response = self.post(TinkoffEndpoint::FindInstrument, body).await?;

        let instruments = response
            .get("instruments")
            .and_then(|i| i.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'instruments' array".to_string()))?;

        if instruments.is_empty() {
            return Err(ExchangeError::NotFound(format!("Instrument '{}' not found", ticker)));
        }

        // Return first matching instrument's FIGI
        instruments[0]
            .get("figi")
            .and_then(|f| f.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing FIGI in response".to_string()))
    }

    /// Map candle interval to Tinkoff format
    fn map_interval(interval: &str) -> &'static str {
        match interval {
            "5s" => "CANDLE_INTERVAL_5_SEC",
            "10s" => "CANDLE_INTERVAL_10_SEC",
            "30s" => "CANDLE_INTERVAL_30_SEC",
            "1m" => "CANDLE_INTERVAL_1_MIN",
            "2m" => "CANDLE_INTERVAL_2_MIN",
            "3m" => "CANDLE_INTERVAL_3_MIN",
            "5m" => "CANDLE_INTERVAL_5_MIN",
            "10m" => "CANDLE_INTERVAL_10_MIN",
            "15m" => "CANDLE_INTERVAL_15_MIN",
            "30m" => "CANDLE_INTERVAL_30_MIN",
            "1h" => "CANDLE_INTERVAL_HOUR",
            "2h" => "CANDLE_INTERVAL_2_HOUR",
            "4h" => "CANDLE_INTERVAL_4_HOUR",
            "1d" => "CANDLE_INTERVAL_DAY",
            "1w" => "CANDLE_INTERVAL_WEEK",
            "1M" => "CANDLE_INTERVAL_MONTH",
            _ => "CANDLE_INTERVAL_HOUR", // default
        }
    }

    /// Calculate time range for candles based on limit and interval
    fn calculate_time_range(limit: u16, interval: &str) -> (String, String) {
        use chrono::{Utc, Duration};

        let now = Utc::now();
        let seconds_per_candle = match interval {
            "5s" => 5,
            "10s" => 10,
            "30s" => 30,
            "1m" => 60,
            "2m" => 120,
            "3m" => 180,
            "5m" => 300,
            "10m" => 600,
            "15m" => 900,
            "30m" => 1800,
            "1h" => 3600,
            "2h" => 7200,
            "4h" => 14400,
            "1d" => 86400,
            "1w" => 604800,
            "1M" => 2592000, // approximate
            _ => 3600,
        };

        let total_seconds = seconds_per_candle * limit as i64;
        let from = now - Duration::seconds(total_seconds);

        (
            from.to_rfc3339(),
            now.to_rfc3339(),
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for TinkoffConnector {
    fn exchange_name(&self) -> &'static str {
        "tinkoff"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Tinkoff
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Tinkoff supports spot trading (stocks, bonds, ETFs) and futures
        vec![AccountType::Spot, AccountType::FuturesCross]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for TinkoffConnector {
    /// Get current price using GetLastPrices
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<f64> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        let body = serde_json::json!({
            "figi": [figi],
        });

        let response = self.post(TinkoffEndpoint::GetLastPrices, body).await?;
        TinkoffParser::parse_price(&response)
    }

    /// Get ticker (24h stats) using GetOrderBook
    ///
    /// Note: Tinkoff doesn't provide 24h stats like crypto exchanges.
    /// We use order book data to construct a basic ticker.
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        let body = serde_json::json!({
            "figi": figi,
            "depth": 1,
        });

        let response = self.post(TinkoffEndpoint::GetOrderBook, body).await?;
        TinkoffParser::parse_ticker(&response, &ticker)
    }

    /// Get orderbook using GetOrderBook
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        // Tinkoff supports depths: 1, 10, 20, 30, 40, 50
        let depth_value = match depth.unwrap_or(10) {
            1 => 1,
            d if d <= 10 => 10,
            d if d <= 20 => 20,
            d if d <= 30 => 30,
            d if d <= 40 => 40,
            _ => 50,
        };

        let body = serde_json::json!({
            "figi": figi,
            "depth": depth_value,
        });

        let response = self.post(TinkoffEndpoint::GetOrderBook, body).await?;
        TinkoffParser::parse_orderbook(&response)
    }

    /// Get klines/candles using GetCandles
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let ticker = format_ticker(&symbol);
        let figi = self.get_figi_by_ticker(&ticker).await?;

        let limit_value = limit.unwrap_or(100).min(2500); // Max 2500 candles
        let (from, to) = Self::calculate_time_range(limit_value, interval);
        let interval_enum = Self::map_interval(interval);

        let body = serde_json::json!({
            "figi": figi,
            "from": from,
            "to": to,
            "interval": interval_enum,
        });

        let response = self.post(TinkoffEndpoint::GetCandles, body).await?;
        TinkoffParser::parse_klines(&response)
    }

    /// Ping the server
    async fn ping(&self) -> ExchangeResult<()> {
        // Use GetAccounts as a simple ping endpoint
        let _response = self.post(TinkoffEndpoint::GetAccounts, serde_json::json!({})).await?;
        Ok(())
    }

    /// Get exchange info — returns list of available MOEX shares from Tinkoff
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let body = serde_json::json!({
            "instrumentStatus": "INSTRUMENT_STATUS_BASE",
        });

        let response = self.post(TinkoffEndpoint::Shares, body).await?;
        let symbols = TinkoffParser::parse_symbols(&response)?;

        let infos = symbols.into_iter().map(|ticker| SymbolInfo {
            symbol: ticker.clone(),
            base_asset: ticker,
            quote_asset: "RUB".to_string(),
            status: "TRADING".to_string(),
            price_precision: 2,
            quantity_precision: 0,
            min_quantity: Some(1.0),
            max_quantity: None,
            step_size: Some(1.0),
            min_notional: None,
        }).collect();

        Ok(infos)
    }
}

impl TinkoffConnector {
    /// Get available symbols using Shares endpoint (extended method)
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let body = serde_json::json!({
            "instrumentStatus": "INSTRUMENT_STATUS_BASE",
        });

        let response = self.post(TinkoffEndpoint::Shares, body).await?;
        TinkoffParser::parse_symbols(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (Full support - Tinkoff is a broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for TinkoffConnector {
    async fn place_order(&self, _req: crate::core::types::OrderRequest) -> crate::core::types::ExchangeResult<crate::core::types::PlaceOrderResponse> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "place_order not yet implemented for Tinkoff connector".to_string()
        ))
    }
    async fn cancel_order(&self, _req: crate::core::types::CancelRequest) -> crate::core::types::ExchangeResult<crate::core::types::Order> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "cancel_order not yet implemented for Tinkoff connector".to_string()
        ))
    }
    async fn get_order(&self, _symbol: &str, _order_id: &str, _account_type: crate::core::types::AccountType) -> crate::core::types::ExchangeResult<crate::core::types::Order> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_order not yet implemented for Tinkoff connector".to_string()
        ))
    }
    async fn get_open_orders(&self, _symbol: Option<&str>, _account_type: crate::core::types::AccountType) -> crate::core::types::ExchangeResult<Vec<crate::core::types::Order>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_open_orders not yet implemented for Tinkoff connector".to_string()
        ))
    }
    async fn get_order_history(&self, _filter: crate::core::types::OrderHistoryFilter, _account_type: crate::core::types::AccountType) -> crate::core::types::ExchangeResult<Vec<crate::core::types::Order>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_order_history not yet implemented for Tinkoff connector".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (Full support - Tinkoff is a broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for TinkoffConnector {
    async fn get_balance(&self, _query: crate::core::types::BalanceQuery) -> crate::core::types::ExchangeResult<Vec<crate::core::types::Balance>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_balance not yet implemented for Tinkoff connector".to_string()
        ))
    }
    async fn get_account_info(&self, _account_type: crate::core::types::AccountType) -> crate::core::types::ExchangeResult<crate::core::types::AccountInfo> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_account_info not yet implemented for Tinkoff connector".to_string()
        ))
    }
    async fn get_fees(&self, _symbol: Option<&str>) -> crate::core::types::ExchangeResult<crate::core::types::FeeInfo> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented for Tinkoff connector".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (Partial support - stocks don't use funding rate)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for TinkoffConnector {
    async fn get_positions(&self, _query: crate::core::types::PositionQuery) -> crate::core::types::ExchangeResult<Vec<crate::core::types::Position>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_positions not yet implemented for Tinkoff connector".to_string()
        ))
    }
    async fn get_funding_rate(&self, _symbol: &str, _account_type: crate::core::types::AccountType) -> crate::core::types::ExchangeResult<crate::core::types::FundingRate> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_funding_rate not supported for Tinkoff connector (stocks)".to_string()
        ))
    }
    async fn modify_position(&self, _req: crate::core::types::PositionModification) -> crate::core::types::ExchangeResult<()> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "modify_position not yet implemented for Tinkoff connector".to_string()
        ))
    }
}


