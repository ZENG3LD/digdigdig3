//! # Interactive Brokers Connector Implementation
//!
//! Core connector implementing the V5 traits for IB Client Portal Web API.

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::types::{
    AccountType, ExchangeError, ExchangeId, ExchangeResult, Kline, OrderBook, Price, Symbol,
    Ticker,
};
use crate::core::traits::{ExchangeIdentity, MarketData};

use super::auth::IBAuth;
use super::endpoints::{IBEndpoint, IBEndpoints};
use super::parser::{IBAccountSummary, IBParser, IBPosition};

/// Interactive Brokers connector
pub struct IBConnector {
    client: Client,
    auth: IBAuth,
    endpoints: IBEndpoints,
    testnet: bool,
    /// Symbol to conid cache
    symbol_cache: Arc<RwLock<HashMap<String, i64>>>,
}

impl IBConnector {
    /// Create new IB connector from Gateway (localhost)
    ///
    /// # Arguments
    /// * `base_url` - Gateway base URL (e.g., "https://localhost:5000/v1/api")
    /// * `account_id` - IB account ID (e.g., "DU12345")
    ///
    /// # Note
    /// Requires Gateway to be running and authenticated via browser.
    pub async fn from_gateway(
        base_url: impl Into<String>,
        account_id: impl Into<String>,
    ) -> ExchangeResult<Self> {
        let base_url = base_url.into();
        let auth = IBAuth::new(account_id);

        // Create HTTP client with SSL verification disabled for localhost
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // For Gateway self-signed cert
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ExchangeError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let connector = Self {
            client,
            auth,
            endpoints: IBEndpoints::custom(base_url, None::<String>),
            testnet: false,
            symbol_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Check authentication status
        connector.check_auth().await?;

        Ok(connector)
    }

    /// Create new IB connector for paper trading (IB paper account via Gateway).
    ///
    /// IB uses port-based separation:
    /// - Live trading Gateway: port 4001 (live) / 4002 (IB Gateway)
    /// - Paper trading Gateway: port 4003 (paper TWS) / 4004 (paper IB Gateway)
    ///
    /// This constructor connects to port 4004 (paper IB Gateway) by default.
    /// Pass a custom `base_url` to override (e.g., `"https://localhost:4003/v1/api"`).
    ///
    /// # Arguments
    /// * `account_id` - IB paper account ID (e.g., "DU12345")
    /// * `base_url` - Optional custom Gateway URL; defaults to `https://localhost:4004/v1/api`
    pub async fn paper(
        account_id: impl Into<String>,
        base_url: Option<impl Into<String>>,
    ) -> ExchangeResult<Self> {
        let url = base_url
            .map(|u| u.into())
            .unwrap_or_else(|| "https://localhost:4004/v1/api".to_string());

        let auth = IBAuth::new(account_id);

        let client = Client::builder()
            .danger_accept_invalid_certs(true) // For Gateway self-signed cert
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ExchangeError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let connector = Self {
            client,
            auth,
            endpoints: IBEndpoints::custom(url, None::<String>),
            testnet: true,
            symbol_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        connector.check_auth().await?;

        Ok(connector)
    }

    /// Builder method — set testnet flag after construction.
    ///
    /// Useful when you already have a connector built with `from_gateway` and
    /// want to mark it as paper/testnet without reconstructing it.
    pub fn with_testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        self
    }

    /// Create new IB connector for production OAuth
    ///
    /// # Note
    /// OAuth 2.0 not yet fully implemented. Use `from_gateway` for now.
    pub async fn from_oauth(_account_id: impl Into<String>) -> ExchangeResult<Self> {
        Err(ExchangeError::UnsupportedOperation(
            "OAuth 2.0 authentication not yet implemented. Use from_gateway() instead.".to_string(),
        ))
    }

    /// Check authentication status
    async fn check_auth(&self) -> ExchangeResult<()> {
        let url = format!("{}{}", self.endpoints.rest_base, IBEndpoint::AuthStatus.path());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Auth check failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Auth(format!(
                "Authentication check failed: HTTP {}",
                response.status()
            )));
        }

        let auth_status: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse auth status: {}", e)))?;

        let authenticated = auth_status
            .get("authenticated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !authenticated {
            return Err(ExchangeError::Auth(
                "Not authenticated. Please login via browser to Gateway.".to_string(),
            ));
        }

        Ok(())
    }

    /// Resolve symbol to conid (with caching)
    async fn resolve_symbol(&self, symbol: &Symbol) -> ExchangeResult<i64> {
        let symbol_key = symbol.to_concat();

        // Check cache first
        {
            let cache = self.symbol_cache.read().await;
            if let Some(&conid) = cache.get(&symbol_key) {
                return Ok(conid);
            }
        }

        // Search for contract
        let contracts = self.search_contract(&symbol.base, "STK").await?;

        if contracts.is_empty() {
            return Err(ExchangeError::NotFound(format!(
                "Symbol {} not found",
                symbol_key
            )));
        }

        let conid = contracts[0].0;

        // Cache the result
        {
            let mut cache = self.symbol_cache.write().await;
            cache.insert(symbol_key, conid);
        }

        Ok(conid)
    }

    /// Search for contract by symbol
    async fn search_contract(
        &self,
        symbol: &str,
        sec_type: &str,
    ) -> ExchangeResult<Vec<(i64, String, String)>> {
        let url = format!("{}{}", self.endpoints.rest_base, IBEndpoint::ContractSearch.path());

        let body = serde_json::json!({
            "symbol": symbol,
            "name": false,
            "secType": sec_type
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Contract search failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Http(format!(
                "Contract search failed: HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse search results: {}", e)))?;

        IBParser::parse_contract_search(&json)
    }

    /// Get market data snapshot for conid
    async fn get_market_data_snapshot(
        &self,
        conid: i64,
        fields: &[&str],
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!(
            "{}{}",
            self.endpoints.rest_base,
            IBEndpoint::MarketDataSnapshot.path()
        );

        let mut params = HashMap::new();
        params.insert("conids".to_string(), conid.to_string());
        params.insert("fields".to_string(), fields.join(","));

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Market data request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Http(format!(
                "Market data request failed: HTTP {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse market data: {}", e)))
    }

    /// Get historical market data for conid
    async fn get_historical_data(
        &self,
        conid: i64,
        period: &str,
        bar_size: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!(
            "{}{}",
            self.endpoints.rest_base,
            IBEndpoint::MarketDataHistory.path()
        );

        let mut params = HashMap::new();
        params.insert("conid".to_string(), conid.to_string());
        params.insert("period".to_string(), period.to_string());
        params.insert("bar".to_string(), bar_size.to_string());

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Historical data request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Http(format!(
                "Historical data request failed: HTTP {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse historical data: {}", e)))
    }

    /// Get portfolio positions
    pub async fn get_positions(&self) -> ExchangeResult<Vec<IBPosition>> {
        let url = format!(
            "{}{}",
            self.endpoints.rest_base,
            IBEndpoint::PortfolioPositions {
                account_id: self.auth.account_id().to_string(),
                page: 0
            }
            .path()
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Positions request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Http(format!(
                "Positions request failed: HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse positions: {}", e)))?;

        IBParser::parse_positions(&json)
    }

    /// Get account summary
    pub async fn get_account_summary(&self) -> ExchangeResult<IBAccountSummary> {
        let url = format!(
            "{}{}",
            self.endpoints.rest_base,
            IBEndpoint::PortfolioSummary {
                account_id: self.auth.account_id().to_string()
            }
            .path()
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Account summary request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Http(format!(
                "Account summary request failed: HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse account summary: {}", e)))?;

        IBParser::parse_account_summary(&json)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for IBConnector {
    fn exchange_name(&self) -> &'static str {
        "Interactive Brokers"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Ib
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // IB supports many types but map to Spot for now
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for IBConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        let conid = self.resolve_symbol(&symbol).await?;

        // Request market data snapshot with field 31 (last price)
        let snapshot = self.get_market_data_snapshot(conid, &["31"]).await?;

        IBParser::parse_price(&snapshot)
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // IB doesn't provide orderbook via snapshot endpoint
        Err(ExchangeError::UnsupportedOperation(
            "IB does not provide orderbook via REST API. Use TWS API for Level 2 data.".to_string(),
        ))
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let conid = self.resolve_symbol(&symbol).await?;

        // Map interval to IB format
        // interval could be "1m", "5m", "1h", "1d", etc.
        let (period, bar_size) = self.map_interval(interval, limit)?;

        let historical = self.get_historical_data(conid, &period, &bar_size).await?;

        IBParser::parse_klines(&historical)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let conid = self.resolve_symbol(&symbol).await?;

        // Request comprehensive market data fields
        let fields = &["31", "84", "86", "70", "71", "87", "7219"];
        let snapshot = self.get_market_data_snapshot(conid, fields).await?;

        IBParser::parse_ticker(&snapshot, &symbol.to_concat())
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let url = format!("{}{}", self.endpoints.rest_base, IBEndpoint::Tickle.path());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Ping failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ExchangeError::Network(format!(
                "Ping failed: HTTP {}",
                response.status()
            )))
        }
    }
}

impl IBConnector {
    /// Map interval string to IB period and bar size
    fn map_interval(&self, interval: &str, limit: Option<u16>) -> ExchangeResult<(String, String)> {
        // IB uses period (like "1d", "5d", "1w") and bar size (like "1min", "5min", "1h")
        let limit = limit.unwrap_or(100);

        let (bar_size, bar_duration_mins) = match interval {
            "1m" => ("1min", 1),
            "5m" => ("5min", 5),
            "15m" => ("15min", 15),
            "30m" => ("30min", 30),
            "1h" => ("1h", 60),
            "2h" => ("2h", 120),
            "4h" => ("4h", 240),
            "1d" => ("1d", 1440),
            "1w" => ("1w", 10080),
            _ => return Err(ExchangeError::InvalidRequest(format!("Unsupported interval: {}", interval))),
        };

        // Calculate period based on limit and bar duration
        let total_mins = limit as u64 * bar_duration_mins;
        let period = if total_mins < 1440 {
            // Less than 1 day: use hours or minutes
            format!("{}d", 1)
        } else if total_mins < 10080 {
            // Less than 1 week: use days
            let days = (total_mins / 1440).max(1);
            format!("{}d", days)
        } else if total_mins < 43200 {
            // Less than 1 month: use weeks
            let weeks = (total_mins / 10080).max(1);
            format!("{}w", weeks)
        } else {
            // Use months
            let months = (total_mins / 43200).clamp(1, 12);
            format!("{}m", months)
        };

        Ok((period, bar_size.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_interval() {
        let connector = create_test_connector();

        let (_period, bar) = connector.map_interval("1m", Some(100)).unwrap();
        assert_eq!(bar, "1min");

        let (_period, bar) = connector.map_interval("1h", Some(24)).unwrap();
        assert_eq!(bar, "1h");

        let (_period, bar) = connector.map_interval("1d", Some(30)).unwrap();
        assert_eq!(bar, "1d");
    }

    fn create_test_connector() -> IBConnector {
        IBConnector {
            client: Client::new(),
            auth: IBAuth::new("TEST"),
            endpoints: IBEndpoints::default(),
            testnet: false,
            symbol_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
