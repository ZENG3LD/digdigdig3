# Phase 2: Implementation Agent Prompt - Data Providers

## Agent Type
`rust-implementer`

## Variables
- `{PROVIDER}` - Provider name in lowercase (e.g., "polygon", "oanda")
- `{CATEGORY}` - Category (aggregators, forex, stocks, data_feeds)

---

## Mission

Implement Rust connector for {PROVIDER} based on research documentation.

**Key Difference from Exchanges:**
- ❌ DON'T force-fit into all traits
- ✅ Implement what makes sense
- ✅ Return `UnsupportedOperation` for irrelevant methods
- ✅ Focus on DATA access (REST + WebSocket)

---

## Input

**Research folder:** `src/{CATEGORY}/{PROVIDER}/research/`

Required files:
- api_overview.md
- endpoints_full.md
- websocket_full.md
- authentication.md
- tiers_and_limits.md
- data_types.md
- response_formats.md
- coverage.md

**Reference implementation:** `src/exchanges/kucoin/` (crypto exchange pattern)

---

## Output

**Implementation folder:** `src/{CATEGORY}/{PROVIDER}/`

Create 5-6 files:

```
src/{CATEGORY}/{PROVIDER}/
├── mod.rs          # Module exports
├── endpoints.rs    # URLs, endpoint enum, formatters
├── auth.rs         # Authentication (usually simple API key)
├── parser.rs       # JSON → domain types
├── connector.rs    # Trait implementations
└── websocket.rs    # WebSocket connector (if WS available)
```

---

## File 1: mod.rs

```rust
//! {PROVIDER} connector
//!
//! Category: {CATEGORY}
//! Type: [Data Provider / Broker / Aggregator]
//!
//! ## Features
//! - REST API: Yes/No
//! - WebSocket: Yes/No
//! - Authentication: API Key / OAuth / None
//! - Free tier: Yes/No
//!
//! ## Data Types
//! - Price data: Yes/No
//! - Historical data: Yes/No
//! - Derivatives data: Yes/No (if applicable)
//! - Fundamentals: Yes/No (if applicable)
//! - On-chain: Yes/No (if applicable)
//! - Macro data: Yes/No (if applicable)

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(feature = "websocket")]
mod websocket;

pub use connector::{ProviderNameConnector};

#[cfg(feature = "websocket")]
pub use websocket::{ProviderNameWebSocket};
```

---

## File 2: endpoints.rs

```rust
//! {PROVIDER} API endpoints

/// Base URLs
pub struct ProviderNameEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ProviderNameEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.example.com",  // from research
            ws_base: Some("wss://ws.example.com"), // from research, or None
        }
    }
}

/// API endpoint enum
#[derive(Debug, Clone)]
pub enum ProviderNameEndpoint {
    // Standard market data
    Price,
    Ticker,
    Candles,
    Symbols,

    // Extended endpoints (from research)
    Liquidations,  // if applicable
    OpenInterest,  // if applicable
    Fundamentals,  // if applicable
    MacroData,     // if applicable

    // Add ALL endpoints from endpoints_full.md
}

impl ProviderNameEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Price => "/v1/price",
            Self::Ticker => "/v1/ticker",
            Self::Candles => "/v1/candles",
            Self::Symbols => "/v1/symbols",
            Self::Liquidations => "/v1/liquidations",
            // ... map all endpoints
        }
    }
}

/// Format symbol/ticker for API
///
/// Different providers have different formats:
/// - Stocks: "AAPL", "TSLA"
/// - Forex: "EUR_USD", "EUR/USD", or "EURUSD"
/// - Crypto: may still use Symbol{base, quote}
pub fn format_symbol(symbol: &crate::core::types::Symbol) -> String {
    // Adapt based on provider requirements from research
    match {CATEGORY} {
        "stocks" => {
            // Stock tickers usually don't have quote
            // Return just base: "AAPL"
            symbol.base.to_uppercase()
        }
        "forex" => {
            // Forex pairs: "EUR_USD" or "EURUSD" or "EUR/USD"
            format!("{}_{}", symbol.base, symbol.quote) // check research
        }
        "aggregators" | "data_feeds" => {
            // Crypto aggregators may use different formats
            format!("{}{}", symbol.base, symbol.quote) // check research
        }
        _ => {
            // Default: base-quote
            format!("{}-{}", symbol.base, symbol.quote)
        }
    }
}

/// Parse symbol from API format back to domain Symbol
pub fn parse_symbol(api_symbol: &str) -> crate::core::types::Symbol {
    // Reverse of format_symbol
    // Implementation depends on provider format
    todo!("Parse symbol based on provider format")
}
```

---

## File 3: auth.rs

```rust
//! {PROVIDER} authentication
//!
//! Authentication type: [API Key / OAuth / None]
//! (from authentication.md research)

use std::collections::HashMap;

/// Authentication credentials
#[derive(Clone)]
pub struct ProviderNameAuth {
    pub api_key: Option<String>,
    pub api_secret: Option<String>, // if needed
}

impl ProviderNameAuth {
    /// Create new auth from environment
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("PROVIDER_API_KEY").ok(),
            api_secret: std::env::var("PROVIDER_API_SECRET").ok(),
        }
    }

    /// Create auth with explicit credentials
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            api_secret: None,
        }
    }

    /// Add authentication headers to request
    ///
    /// Based on authentication.md research:
    /// - Header name: X-API-Key / Authorization / etc.
    /// - Format: "Bearer xxx" / "xxx" / etc.
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            // Check research for correct header name and format
            headers.insert("X-API-Key".to_string(), key.clone());
            // OR: headers.insert("Authorization".to_string(), format!("Bearer {}", key));
        }
    }

    /// Add authentication to query params (if provider uses this)
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("apiKey".to_string(), key.clone());
        }
    }

    /// Generate signature (if provider requires HMAC - rare for data providers)
    pub fn generate_signature(
        &self,
        _timestamp: i64,
        _method: &str,
        _path: &str,
        _query: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Usually NOT needed for data providers
        // Most use simple API key
        Err("Signature not required for this provider".into())
    }
}
```

---

## File 4: parser.rs

```rust
//! {PROVIDER} response parsers
//!
//! Parse JSON responses to domain types based on response_formats.md

use serde_json::Value;
use crate::core::types::*;
use crate::core::error::{ExchangeError, ExchangeResult};

pub struct ProviderNameParser;

impl ProviderNameParser {
    // ═══════════════════════════════════════════════════════════════════════
    // STANDARD MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse price response
    ///
    /// Example from response_formats.md:
    /// ```json
    /// {"symbol": "AAPL", "price": 150.25, "timestamp": 1234567890}
    /// ```
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        response
            .get("price")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Missing 'price' field".to_string()))
    }

    /// Parse ticker response
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price: Self::require_f64(response, "last")?,
            bid_price: Self::get_f64(response, "bid"),
            ask_price: Self::get_f64(response, "ask"),
            high_24h: Self::get_f64(response, "high_24h"),
            low_24h: Self::get_f64(response, "low_24h"),
            volume_24h: Self::get_f64(response, "volume_24h"),
            quote_volume_24h: Self::get_f64(response, "quote_volume_24h"),
            price_change_24h: Self::get_f64(response, "change_24h"),
            price_change_percent_24h: Self::get_f64(response, "change_percent_24h"),
            timestamp: Self::require_i64(response, "timestamp")?,
        })
    }

    /// Parse klines/candles response
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        array.iter().map(|candle| {
            Ok(Kline {
                open_time: Self::require_i64(candle, "timestamp")?,
                open: Self::require_f64(candle, "open")?,
                high: Self::require_f64(candle, "high")?,
                low: Self::require_f64(candle, "low")?,
                close: Self::require_f64(candle, "close")?,
                volume: Self::require_f64(candle, "volume")?,
                quote_volume: Self::get_f64(candle, "quote_volume"),
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    /// Parse orderbook response (if applicable)
    ///
    /// NOTE: Many data providers DON'T provide orderbook.
    /// If not available, connector should return UnsupportedOperation.
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let bids = Self::parse_order_levels(response.get("bids"))?;
        let asks = Self::parse_order_levels(response.get("asks"))?;

        Ok(OrderBook {
            bids,
            asks,
            timestamp: Self::require_i64(response, "timestamp")?,
            sequence: Self::get_str(response, "sequence").map(|s| s.to_string()),
        })
    }

    /// Parse symbols list
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        Ok(array.iter()
            .filter_map(|v| v.get("symbol").and_then(|s| s.as_str()))
            .map(|s| s.to_string())
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXTENDED DATA TYPES (from data_types.md research)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse liquidations (if applicable - derivatives data feeds)
    pub fn parse_liquidations(response: &Value) -> ExchangeResult<Vec<Liquidation>> {
        // Implementation based on response_formats.md
        todo!("Implement if provider offers liquidation data")
    }

    /// Parse funding rate (if applicable - crypto derivatives)
    pub fn parse_funding_rate(response: &Value, symbol: &str) -> ExchangeResult<FundingRate> {
        Ok(FundingRate {
            symbol: symbol.to_string(),
            rate: Self::require_f64(response, "rate")?,
            next_funding_time: Self::get_i64(response, "next_funding_time"),
            timestamp: Self::require_i64(response, "timestamp")?,
        })
    }

    /// Parse fundamentals (if applicable - stock data)
    pub fn parse_company_profile(response: &Value) -> ExchangeResult<CompanyProfile> {
        // Implementation based on response_formats.md
        todo!("Implement if provider offers fundamental data")
    }

    /// Parse macro economic data (if applicable - FRED, etc.)
    pub fn parse_economic_series(response: &Value) -> ExchangeResult<EconomicSeries> {
        // Implementation based on response_formats.md
        todo!("Implement if provider offers macro data")
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS (if WS available)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker update
    pub fn parse_ws_ticker(msg: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        // WebSocket format may differ from REST
        // Check websocket_full.md for exact format
        Self::parse_ticker(msg, symbol)
    }

    /// Parse WebSocket trade update
    pub fn parse_ws_trade(msg: &Value) -> ExchangeResult<Trade> {
        // Implementation based on websocket_full.md
        todo!("Implement WS trade parser")
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn parse_order_levels(value: Option<&Value>) -> ExchangeResult<Vec<(f64, f64)>> {
        let array = value
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Invalid order levels".to_string()))?;

        array.iter().map(|level| {
            let arr = level.as_array()
                .ok_or_else(|| ExchangeError::Parse("Invalid level format".to_string()))?;

            let price = arr.get(0)
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .ok_or_else(|| ExchangeError::Parse("Invalid price".to_string()))?;

            let size = arr.get(1)
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .ok_or_else(|| ExchangeError::Parse("Invalid size".to_string()))?;

            Ok((price, size))
        }).collect()
    }
}
```

---

## File 5: connector.rs

**CRITICAL:** This is where trait implementation decisions happen.

```rust
//! {PROVIDER} connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::error::{ExchangeError, ExchangeResult};
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// {PROVIDER} connector
pub struct ProviderNameConnector {
    client: Client,
    auth: ProviderNameAuth,
    endpoints: ProviderNameEndpoints,
    testnet: bool,
}

impl ProviderNameConnector {
    /// Create new connector
    pub fn new(auth: ProviderNameAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: ProviderNameEndpoints::default(),
            testnet: false,
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(ProviderNameAuth::from_env())
    }

    /// Internal: Make GET request
    async fn get(
        &self,
        endpoint: ProviderNameEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Add authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        // OR: self.auth.sign_query(&mut params);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api(format!("HTTP {}", response.status())));
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for ProviderNameConnector {
    fn exchange_name(&self) -> &'static str {
        "{PROVIDER}"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::ProviderName  // Add to ExchangeId enum
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Data providers usually only support Spot equivalent
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData (Implement what makes sense)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for ProviderNameConnector {
    /// Get current price
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));

        let response = self.get(ProviderNameEndpoint::Price, params).await?;
        let price = ProviderNameParser::parse_price(&response)?;

        Ok(price)
    }

    /// Get ticker (24h stats)
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let symbol_str = format_symbol(&symbol);
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str.clone());

        let response = self.get(ProviderNameEndpoint::Ticker, params).await?;
        ProviderNameParser::parse_ticker(&response, &symbol_str)
    }

    /// Get orderbook
    ///
    /// NOTE: Many data providers DON'T have orderbook.
    /// Return UnsupportedOperation if not available.
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Check data_types.md research:
        // If orderbook is NOT available, return:
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} does not provide orderbook data - data feed only".to_string()
        ))

        // If available:
        // let response = self.get(...).await?;
        // ProviderNameParser::parse_orderbook(&response)
    }

    /// Get klines/candles
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));
        params.insert("interval".to_string(), interval.to_string());
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(ProviderNameEndpoint::Candles, params).await?;
        ProviderNameParser::parse_klines(&response)
    }

    /// Get available symbols
    async fn get_symbols(&self, _account_type: AccountType) -> ExchangeResult<Vec<String>> {
        let response = self.get(ProviderNameEndpoint::Symbols, HashMap::new()).await?;
        ProviderNameParser::parse_symbols(&response)
    }

    /// Get all tickers (if available)
    async fn get_all_tickers(&self) -> ExchangeResult<Vec<Ticker>> {
        // Implementation depends on provider
        Err(ExchangeError::UnsupportedOperation(
            "get_all_tickers not implemented for this provider".to_string()
        ))
    }

    /// Get funding rate (if applicable - derivatives only)
    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Check if provider offers funding rate data
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not available - not a derivatives platform".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (Usually UnsupportedOperation for data providers)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for ProviderNameConnector {
    async fn place_order(&self, _order: Order) -> ExchangeResult<OrderResult> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }

    async fn cancel_order(
        &self,
        _order_id: &str,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderResult> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order(
        &self,
        _order_id: &str,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderResult> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - trading not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (Usually UnsupportedOperation unless broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for ProviderNameConnector {
    async fn get_balance(&self, _account_type: AccountType) -> ExchangeResult<Vec<Balance>> {
        // If provider is a BROKER (Alpaca, OANDA, Zerodha):
        // Implement balance checking

        // If provider is DATA ONLY:
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - account operations not supported".to_string()
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - account operations not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (Usually UnsupportedOperation unless broker)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for ProviderNameConnector {
    async fn get_positions(&self, _account_type: AccountType) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn get_position(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Position> {
        Err(ExchangeError::UnsupportedOperation(
            "{PROVIDER} is a data provider - position tracking not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Provider-specific, not from traits)
// ═══════════════════════════════════════════════════════════════════════════

impl ProviderNameConnector {
    /// Get liquidations (if provider offers this data)
    pub async fn get_liquidations(&self, symbol: Symbol) -> ExchangeResult<Vec<Liquidation>> {
        // Implementation based on endpoints_full.md
        todo!("Implement if provider offers liquidation data")
    }

    /// Get company fundamentals (if provider offers this)
    pub async fn get_company_profile(&self, ticker: &str) -> ExchangeResult<CompanyProfile> {
        // Implementation based on endpoints_full.md
        todo!("Implement if provider offers fundamental data")
    }

    /// Get economic data series (if provider offers macro data)
    pub async fn get_economic_series(&self, series_id: &str) -> ExchangeResult<EconomicSeries> {
        // Implementation based on endpoints_full.md
        todo!("Implement if provider offers macro data")
    }
}
```

---

## File 6: websocket.rs (Optional - only if WS available)

**Skip if WebSocket not available** (check api_overview.md).

```rust
//! {PROVIDER} WebSocket connector

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::core::types::*;
use crate::core::error::{WebSocketError, WebSocketResult};
use crate::core::traits::WebSocketConnector;

use super::auth::ProviderNameAuth;
use super::parser::ProviderNameParser;

/// WebSocket connector for {PROVIDER}
pub struct ProviderNameWebSocket {
    auth: ProviderNameAuth,
    ws_url: String,
    status: Arc<RwLock<ConnectionStatus>>,
    broadcast_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
}

impl ProviderNameWebSocket {
    /// Create new WebSocket connector
    pub fn new(auth: ProviderNameAuth) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            auth,
            ws_url: "wss://ws.example.com".to_string(), // from websocket_full.md
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            broadcast_tx,
        }
    }
}

#[async_trait]
impl WebSocketConnector for ProviderNameWebSocket {
    async fn connect(&mut self) -> WebSocketResult<()> {
        *self.status.write().await = ConnectionStatus::Connecting;

        // Connect to WebSocket
        let url = &self.ws_url;
        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        let (write, read) = ws_stream.split();

        // Create channels
        let (event_tx, mut event_rx) = mpsc::channel::<WebSocketResult<StreamEvent>>(100);

        // Spawn reader task
        let broadcast_tx = self.broadcast_tx.clone();
        tokio::spawn(async move {
            // Read messages and parse
            // Forward to event_tx
            // Handle ping/pong based on websocket_full.md
        });

        // Forward events to broadcast
        let broadcast_tx2 = self.broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                let _ = broadcast_tx2.send(event);
            }
        });

        *self.status.write().await = ConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.write().await = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Check connection status
        let status = self.status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(WebSocketError::NotConnected);
        }
        drop(status);

        // Send subscribe message based on websocket_full.md format
        todo!("Send subscription message")
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Send unsubscribe message
        todo!("Send unsubscribe message")
    }

    fn event_stream(&self) -> impl Stream<Item = WebSocketResult<StreamEvent>> + '_ {
        tokio_stream::wrappers::BroadcastStream::new(self.broadcast_tx.subscribe())
            .filter_map(|r| async { r.ok() })
    }

    fn is_connected(&self) -> bool {
        matches!(
            *self.status.blocking_read(),
            ConnectionStatus::Connected
        )
    }
}
```

---

## Implementation Checklist

- [ ] mod.rs created with documentation
- [ ] endpoints.rs with ALL endpoints from research
- [ ] auth.rs with simple API key authentication
- [ ] parser.rs with parsers for available data types
- [ ] connector.rs with trait implementations:
  - [ ] ExchangeIdentity - ALWAYS implement
  - [ ] MarketData - Implement what provider offers
  - [ ] Trading - UnsupportedOperation (unless broker)
  - [ ] Account - UnsupportedOperation (unless broker)
  - [ ] Positions - UnsupportedOperation (unless broker)
- [ ] websocket.rs (only if WS available)
- [ ] Extended methods for provider-specific data
- [ ] Added to src/{CATEGORY}/mod.rs
- [ ] Added ExchangeId variant to src/core/types/common.rs

---

## Key Principles

1. **Don't force-fit** - If trait method doesn't apply, return UnsupportedOperation
2. **Follow research** - All decisions based on research docs
3. **Simple auth** - Most providers use API key, not HMAC
4. **Extended methods** - Add provider-specific methods outside traits
5. **Graceful errors** - Clear error messages explaining why operation not supported

---

## Compilation Test

Before Phase 3:
```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo check --package digdigdig3
```

Must compile with 0 errors.

---

## Next Phase

After implementation compiles:
→ Phase 3: Create tests
