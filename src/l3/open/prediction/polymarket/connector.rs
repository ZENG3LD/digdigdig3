//! Polymarket Connector
//!
//! Implements V5 core traits for Polymarket prediction markets.
//!
//! ## Supported traits
//! - `ExchangeIdentity` — returns `ExchangeId::Polymarket`
//! - `MarketData` — ping, price, orderbook, klines, ticker, exchange_info
//!
//! ## Market data mapping
//! - Symbol = `condition_id` (blockchain market identifier)
//! - Price = YES outcome probability (0.0 - 1.0)
//! - Klines = price history from CLOB `/prices-history` endpoint
//! - OrderBook = YES token order book from CLOB `/book` endpoint
//!
//! ## Public vs Authenticated
//!
//! Public mode gives access to all read endpoints.
//! Authenticated mode additionally enables /orders endpoint.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use crate::core::{
    AccountType, ExchangeError, ExchangeId, ExchangeResult, ExchangeType,
    Kline, OrderBook, Price, Symbol, SymbolInfo, Ticker,
};
use crate::core::traits::{ExchangeIdentity, MarketData};

use super::auth::{PolymarketAuth, PolymarketCredentials};
use super::endpoints::{
    PolymarketEndpoints, map_interval, get_fidelity,
};
use super::parser::{
    PolymarketParser, ClobMarket, PolyMarket, PolyOrderBook, PolyMidpoint,
    PolyEvent,
    clob_market_to_symbol_info, clob_market_to_ticker,
    poly_market_to_symbol_info,
    price_history_to_klines, poly_orderbook_to_v5,
    interval_to_ms,
};

// ═══════════════════════════════════════════════════════════════════════════
// CONNECTOR STRUCT
// ═══════════════════════════════════════════════════════════════════════════

/// Polymarket prediction markets connector
///
/// Provides access to Polymarket's REST APIs for market data, order books,
/// and price history. Supports both public (no auth) and authenticated access.
///
/// # Examples
///
/// ```ignore
/// use connectors_v5::data_feeds::prediction::polymarket::PolymarketConnector;
///
/// // Public access
/// let connector = PolymarketConnector::public();
///
/// // Get active markets
/// let markets = connector.get_active_markets(Some(20)).await?;
///
/// // Get klines for a market (condition_id as symbol)
/// let symbol = Symbol::new("0xABC123", "USDC");
/// let klines = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot, None).await?;
/// ```
pub struct PolymarketConnector {
    /// Reqwest HTTP client
    client: Client,
    /// Authentication (None = public mode)
    _auth: PolymarketAuth,
    /// API base URLs
    endpoints: PolymarketEndpoints,
}

impl PolymarketConnector {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Create a public (no authentication) connector
    pub fn public() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            _auth: PolymarketAuth::new(),
            endpoints: PolymarketEndpoints::default(),
        }
    }

    /// Create an authenticated connector with L2 credentials
    pub fn authenticated(creds: PolymarketCredentials) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            _auth: PolymarketAuth::with_credentials(creds),
            endpoints: PolymarketEndpoints::default(),
        }
    }

    /// Create connector from environment variables (tries auth, falls back to public)
    pub fn from_env() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            _auth: PolymarketAuth::from_env(),
            endpoints: PolymarketEndpoints::default(),
        }
    }

    // -----------------------------------------------------------------------
    // Internal HTTP helpers
    // -----------------------------------------------------------------------

    /// GET request to any URL, returns parsed JSON
    async fn get_url(&self, url: &str) -> ExchangeResult<serde_json::Value> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, body.chars().take(200).collect::<String>()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        PolymarketParser::check_error(&json)?;
        Ok(json)
    }

    /// GET request to CLOB API with optional query params
    async fn get_clob(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> ExchangeResult<serde_json::Value> {
        let base_url = format!("{}{}", self.endpoints.clob_base, path);
        let url = if params.is_empty() {
            base_url
        } else {
            let qs: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", base_url, qs)
        };
        self.get_url(&url).await
    }

    /// GET request to Gamma API with optional query params
    async fn get_gamma(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> ExchangeResult<serde_json::Value> {
        let base_url = format!("{}{}", self.endpoints.gamma_base, path);
        let url = if params.is_empty() {
            base_url
        } else {
            let qs: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", base_url, qs)
        };
        self.get_url(&url).await
    }

    // -----------------------------------------------------------------------
    // Polymarket-specific public methods
    // -----------------------------------------------------------------------

    /// Get paginated list of markets from CLOB API
    ///
    /// Returns `(markets, next_cursor)`. Pass `next_cursor` back to paginate.
    pub async fn get_markets(
        &self,
        limit: Option<u32>,
        next_cursor: Option<&str>,
    ) -> ExchangeResult<(Vec<ClobMarket>, Option<String>)> {
        let limit_str;
        let cursor_str;

        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", &limit_str));
        }
        if let Some(c) = next_cursor {
            cursor_str = c.to_string();
            params.push(("next_cursor", &cursor_str));
        }

        let response = self.get_clob("/markets", &params).await?;
        let markets = PolymarketParser::parse_clob_markets(&response)?;
        let next = PolymarketParser::get_next_cursor(&response);
        Ok((markets, next))
    }

    /// Get a specific market by condition_id (CLOB API)
    pub async fn get_market(&self, condition_id: &str) -> ExchangeResult<ClobMarket> {
        let url = format!("{}/markets/{}", self.endpoints.clob_base, condition_id);
        let response = self.get_url(&url).await?;
        PolymarketParser::parse_clob_market(&response)
    }

    /// Get order book for a specific token (CLOB API)
    pub async fn get_order_book(&self, token_id: &str) -> ExchangeResult<PolyOrderBook> {
        let response = self.get_clob("/book", &[("token_id", token_id)]).await?;
        PolymarketParser::parse_order_book(&response)
    }

    /// Get midpoint price for a token (CLOB API)
    pub async fn get_midpoint(&self, token_id: &str) -> ExchangeResult<PolyMidpoint> {
        let response = self.get_clob("/midpoint", &[("token_id", token_id)]).await?;
        PolymarketParser::parse_midpoint(&response)
    }

    /// Get last trade price for a token (CLOB API)
    pub async fn get_last_trade_price(&self, token_id: &str) -> ExchangeResult<f64> {
        let response = self.get_clob("/last-trade-price", &[("token_id", token_id)]).await?;
        PolymarketParser::parse_price(&response)
    }

    /// Get active markets only (CLOB API)
    pub async fn get_active_markets(&self, limit: Option<u32>) -> ExchangeResult<Vec<ClobMarket>> {
        let limit_str;
        let mut params: Vec<(&str, &str)> = vec![("active", "true")];
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", &limit_str));
        }
        let response = self.get_clob("/markets", &params).await?;
        PolymarketParser::parse_clob_markets(&response)
    }

    /// Get events from Gamma API
    pub async fn get_events(&self, limit: Option<u32>) -> ExchangeResult<Vec<PolyEvent>> {
        let limit_str;
        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", &limit_str));
        }
        let response = self.get_gamma("/events", &params).await?;
        PolymarketParser::parse_events(&response)
    }

    /// Get a specific event by ID from Gamma API
    pub async fn get_event(&self, event_id: &str) -> ExchangeResult<PolyEvent> {
        let url = format!("{}/events/{}", self.endpoints.gamma_base, event_id);
        let response = self.get_url(&url).await?;
        PolymarketParser::parse_event(&response)
    }

    /// Get markets with enhanced metadata from Gamma API
    pub async fn get_gamma_markets(&self, limit: Option<u32>) -> ExchangeResult<Vec<PolyMarket>> {
        let limit_str;
        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", &limit_str));
        }
        let response = self.get_gamma("/markets", &params).await?;
        PolymarketParser::parse_gamma_markets(&response)
    }

    /// Get the primary token ID for a condition_id by fetching market details.
    ///
    /// Prefers the "Yes" outcome token. Falls back to the first available token
    /// for markets that use non-binary outcome names (e.g. team names, candidates).
    async fn get_yes_token_id(&self, condition_id: &str) -> ExchangeResult<String> {
        let market = self.get_market(condition_id).await?;
        // Prefer "Yes" outcome; fall back to first token for non-binary markets.
        let token = market
            .tokens
            .iter()
            .find(|t| t.outcome == "Yes")
            .or_else(|| market.tokens.first())
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "No tokens found for condition_id {}",
                    condition_id
                ))
            })?;
        Ok(token.token_id.clone())
    }

    /// Extract identifier from symbol, lowercased.
    ///
    /// Polymarket identifiers (condition_id, token_id) are hex strings that
    /// must be lowercase for the CLOB API. `Symbol::new` uppercases `base`,
    /// so we lowercase the result here regardless of source.
    fn symbol_id<'a>(&self, symbol: &'a Symbol) -> std::borrow::Cow<'a, str> {
        let s = symbol.raw().unwrap_or(&symbol.base);
        if s.chars().any(|c| c.is_ascii_uppercase()) {
            std::borrow::Cow::Owned(s.to_lowercase())
        } else {
            std::borrow::Cow::Borrowed(s)
        }
    }
}

impl Default for PolymarketConnector {
    fn default() -> Self {
        Self::public()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for PolymarketConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Polymarket
    }

    fn is_testnet(&self) -> bool {
        false
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::DataProvider
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for PolymarketConnector {
    /// Check connectivity by fetching server time
    async fn ping(&self) -> ExchangeResult<()> {
        self.get_clob("/time", &[]).await?;
        Ok(())
    }

    /// Get current YES probability for a market
    ///
    /// `symbol.base` should be the `condition_id` (0x...).
    /// Returns the YES outcome probability as the "price" (0.0 - 1.0).
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let condition_id = self.symbol_id(&symbol);
        let condition_id = condition_id.as_ref();

        // Get the market to find the primary token ID.
        // Prefers "Yes" outcome; falls back to first token for non-binary markets.
        let market = self.get_market(condition_id).await?;
        let yes_token = market
            .tokens
            .iter()
            .find(|t| t.outcome == "Yes")
            .or_else(|| market.tokens.first())
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "No tokens found for market {}",
                    condition_id,
                ))
            })?;

        // Get last trade price for the primary token
        match self.get_last_trade_price(&yes_token.token_id).await {
            Ok(price) => Ok(price),
            Err(_) => {
                // Fall back to midpoint if last trade is unavailable
                let midpoint = self.get_midpoint(&yes_token.token_id).await?;
                Ok(midpoint.mid)
            }
        }
    }

    /// Get order book for a market's YES token
    ///
    /// `symbol.base` should be the `condition_id` (0x...).
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let condition_id = self.symbol_id(&symbol);
        let condition_id = condition_id.as_ref();

        // Get the YES token ID
        let yes_token_id = self.get_yes_token_id(condition_id).await?;

        // Fetch and convert the order book
        let poly_book = self.get_order_book(&yes_token_id).await?;
        Ok(poly_orderbook_to_v5(&poly_book))
    }

    /// Get price history as klines
    ///
    /// `symbol.base` should be the token_id (not condition_id) for efficiency,
    /// or condition_id (the connector will look up the YES token_id).
    ///
    /// Intervals: "1m" → 1m, "1h" → 1h, "6h" → 6h, "1d" → 1d, "1w" → 1w
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let input_cow = self.symbol_id(&symbol);
        let input = input_cow.as_ref();

        // Determine if this looks like a condition_id (0x... 66 chars) or a token_id.
        // A condition_id is 0x + 64 hex chars = 66 chars total.
        // A token_id is a plain numeric string (up to 78 digits).
        let token_id = if (input.starts_with("0x") || input.starts_with("0X")) && input.len() == 66
        {
            // Looks like a condition_id — look up YES token
            self.get_yes_token_id(input).await?
        } else {
            // Assume it's already a token_id
            input.to_string()
        };

        let poly_interval = map_interval(interval);
        let fidelity = get_fidelity(limit);
        let fidelity_str = fidelity.to_string();

        let response = self
            .get_clob(
                "/prices-history",
                &[
                    ("market", token_id.as_str()),
                    ("interval", poly_interval),
                    ("fidelity", fidelity_str.as_str()),
                ],
            )
            .await?;

        let history = PolymarketParser::parse_price_history(&response)?;
        let interval_ms = interval_to_ms(poly_interval);
        let klines = price_history_to_klines(history, interval_ms);

        Ok(klines)
    }

    /// Get 24h ticker for a market
    ///
    /// `symbol.base` should be the `condition_id` (0x...).
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let condition_id = self.symbol_id(&symbol);
        let condition_id = condition_id.as_ref();
        let market = self.get_market(condition_id).await?;

        clob_market_to_ticker(&market).ok_or_else(|| {
            ExchangeError::Parse(format!(
                "No ticker data available for market {}",
                condition_id
            ))
        })
    }

    /// Get all active markets as SymbolInfo
    ///
    /// Uses the Gamma API with `active=true&closed=false` filtering to return
    /// only currently-open markets. Falls back to the CLOB API if Gamma fails.
    async fn get_exchange_info(
        &self,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<SymbolInfo>> {
        // Gamma API correctly filters active/open markets (CLOB `/markets` pagination
        // orders by creation date ascending and does not filter closed markets reliably).
        let gamma_result = self
            .get_gamma(
                "/markets",
                &[("active", "true"), ("closed", "false"), ("limit", "500")],
            )
            .await;

        match gamma_result {
            Ok(response) => {
                let markets = PolymarketParser::parse_gamma_markets(&response)?;
                let symbols = markets
                    .iter()
                    .map(|m| poly_market_to_symbol_info(m, account_type))
                    .collect();
                Ok(symbols)
            }
            Err(_) => {
                // Fall back to CLOB API
                let markets = self.get_active_markets(Some(500)).await?;
                let symbols = markets
                    .iter()
                    .map(|m| clob_market_to_symbol_info(m, account_type))
                    .collect();
                Ok(symbols)
            }
        }
    }
}
