//! # MOEX ISS Connector Implementation
//!
//! Implementation of core traits for MOEX Moscow Exchange ISS API.

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{
    AccountType, Asset, Balance, ExchangeError, ExchangeId, ExchangeResult, Kline, Order, OrderBook,
    Position, Price, Symbol, Ticker, AccountInfo, OrderSide, Quantity, FundingRate, SymbolInfo,
    OrderRequest, CancelRequest, OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    BalanceQuery, PositionQuery, PositionModification,
};
use crate::core::traits::{ExchangeIdentity, MarketData, Trading, Account, Positions};

use super::endpoints::{MoexEndpoint, MoexEndpoints, format_symbol, map_interval, default_stock_params};
use super::auth::MoexAuth;
use super::parser::MoexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// MOEX CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// MOEX ISS connector
///
/// This connector provides access to Moscow Exchange market data via the ISS API.
/// It is a **data-only provider** and does not support trading operations.
pub struct MoexConnector {
    client: Client,
    auth: MoexAuth,
    endpoints: MoexEndpoints,
}

impl MoexConnector {
    /// Create new MOEX connector with authentication
    pub fn new(auth: MoexAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: MoexEndpoints::default(),
        }
    }

    /// Create public MOEX connector (no authentication, 15-min delay)
    pub fn new_public() -> Self {
        Self::new(MoexAuth::public())
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(MoexAuth::from_env())
    }

    /// Internal: Make GET request
    async fn get(
        &self,
        endpoint: MoexEndpoint,
        path_params: &[(&str, &str)],
        query_params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let path = endpoint.build_path(path_params);
        let url = format!("{}{}", self.endpoints.rest_base, path);

        // Add authentication headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        // Build request
        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query params
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}", status),
            });
        }

        // Parse JSON
        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for MoexConnector {
    fn exchange_name(&self) -> &'static str {
        "MOEX"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Moex
    }

    fn is_testnet(&self) -> bool {
        false
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // MOEX ISS is data-only, but conceptually supports these markets
        vec![AccountType::Spot] // Can be extended for futures data
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for MoexConnector {
    /// Get current price
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let (engine, market, board) = default_stock_params();
        let security = format_symbol(&symbol);

        let path_params = &[
            ("engine", engine),
            ("market", market),
            ("board", board),
            ("security", &security),
        ];

        let response = self
            .get(MoexEndpoint::BoardSecurityData, path_params, HashMap::new())
            .await?;

        MoexParser::parse_price(&response)
            .map_err(ExchangeError::Parse)
    }

    /// Get orderbook
    ///
    /// Note: Requires paid subscription for real-time orderbook data
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let (engine, market, _board) = default_stock_params();
        let security = format_symbol(&symbol);

        let path_params = &[
            ("engine", engine),
            ("market", market),
            ("security", &security),
        ];

        let response = self
            .get(MoexEndpoint::SecurityOrderbook, path_params, HashMap::new())
            .await?;

        MoexParser::parse_orderbook(&response)
            .map_err(ExchangeError::Parse)
    }

    /// Get klines/candles
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let (engine, market, board) = default_stock_params();
        let security = format_symbol(&symbol);
        let moex_interval = map_interval(interval);

        let path_params = &[
            ("engine", engine),
            ("market", market),
            ("board", board),
            ("security", &security),
        ];

        let mut query_params = HashMap::new();
        query_params.insert("interval".to_string(), moex_interval.to_string());

        // MOEX requires 'from' parameter for candles
        // Default to last 7 days if not specified
        let from_date = chrono::Utc::now() - chrono::Duration::days(7);
        query_params.insert("from".to_string(), from_date.format("%Y-%m-%d").to_string());

        if let Some(lim) = limit {
            // MOEX doesn't have explicit limit, but we can use 'till' to control range
            // For simplicity, just note the limitation
            query_params.insert("limit".to_string(), lim.to_string());
        }

        let response = self
            .get(MoexEndpoint::BoardCandles, path_params, query_params)
            .await?;

        MoexParser::parse_klines(&response)
            .map_err(ExchangeError::Parse)
    }

    /// Get 24h ticker
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let (engine, market, board) = default_stock_params();
        let security = format_symbol(&symbol);

        let path_params = &[
            ("engine", engine),
            ("market", market),
            ("board", board),
            ("security", &security),
        ];

        let response = self
            .get(MoexEndpoint::BoardSecurityData, path_params, HashMap::new())
            .await?;

        MoexParser::parse_ticker(&response, &security)
            .map_err(ExchangeError::Parse)
    }

    /// Ping server
    async fn ping(&self) -> ExchangeResult<()> {
        // MOEX doesn't have a dedicated ping endpoint
        // Use lightweight engines endpoint instead
        self.get(MoexEndpoint::Engines, &[], HashMap::new())
            .await
            .map(|_| ())
    }

    /// Get exchange info — returns listed securities from MOEX
    ///
    /// Delegates to the same MarketSecurities endpoint used by `get_symbols()`
    /// which reliably returns all actively-trading instruments on the stock/shares market.
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let (engine, market, _) = default_stock_params();

        let path_params = &[
            ("engine", engine),
            ("market", market),
        ];

        let response = self
            .get(MoexEndpoint::MarketSecurities, path_params, HashMap::new())
            .await?;

        let symbols = MoexParser::parse_symbols(&response)
            .map_err(ExchangeError::Parse)?;

        let infos = symbols.into_iter().map(|sec_id| SymbolInfo {
            symbol: sec_id.clone(),
            base_asset: sec_id,
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

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (UnsupportedOperation - MOEX ISS is data-only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for MoexConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - trading not supported. Use MOEX WebAPI for trading.".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - trading not supported. Use MOEX WebAPI for trading.".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - trading not supported. Use MOEX WebAPI for trading.".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - trading not supported. Use MOEX WebAPI for trading.".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - trading not supported. Use MOEX WebAPI for trading.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UnsupportedOperation - MOEX ISS is data-only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for MoexConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - account operations not supported. Use MOEX WebAPI or broker API.".to_string()
        ))
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - account operations not supported. Use MOEX WebAPI or broker API.".to_string()
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - account operations not supported. Use MOEX WebAPI or broker API.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UnsupportedOperation - MOEX ISS is data-only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for MoexConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - position tracking not supported. Use MOEX WebAPI or broker API.".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - position tracking not supported. Use MOEX WebAPI or broker API.".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "MOEX ISS is a data provider - position tracking not supported. Use MOEX WebAPI or broker API.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (MOEX-specific, not from traits)
// ═══════════════════════════════════════════════════════════════════════════════

impl MoexConnector {
    /// Get list of all symbols/securities
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let (engine, market, _) = default_stock_params();

        let path_params = &[
            ("engine", engine),
            ("market", market),
        ];

        let response = self
            .get(MoexEndpoint::MarketSecurities, path_params, HashMap::new())
            .await?;

        MoexParser::parse_symbols(&response)
            .map_err(ExchangeError::Parse)
    }

    /// Get list of all trading engines
    pub async fn get_engines(&self) -> ExchangeResult<serde_json::Value> {
        self.get(MoexEndpoint::Engines, &[], HashMap::new())
            .await
    }

    /// Get markets for a specific engine
    pub async fn get_markets(&self, engine: &str) -> ExchangeResult<serde_json::Value> {
        let path_params = &[("engine", engine)];
        self.get(MoexEndpoint::EngineMarkets, path_params, HashMap::new())
            .await
    }

    /// Get security information
    pub async fn get_security_info(&self, security: &str) -> ExchangeResult<serde_json::Value> {
        let path_params = &[("security", security)];
        self.get(MoexEndpoint::SecurityInfo, path_params, HashMap::new())
            .await
    }

    /// Get market turnovers
    pub async fn get_turnovers(&self) -> ExchangeResult<serde_json::Value> {
        self.get(MoexEndpoint::Turnovers, &[], HashMap::new())
            .await
    }

    /// Check if connector has real-time access
    pub fn has_realtime_access(&self) -> bool {
        self.auth.is_authenticated()
    }
}
