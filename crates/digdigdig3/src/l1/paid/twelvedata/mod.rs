//! # Twelvedata Data Provider
//!
//! Multi-asset data provider (stocks, forex, crypto, ETFs, commodities, indices).
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint definitions
//! - `auth` - API key authentication (simple header-based)
//! - `parser` - JSON response parsing
//! - `connector` - TwelvedataConnector + trait implementations
//! - `websocket` - WebSocket connection (Pro+ tier only)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::stocks::us::twelvedata::TwelvedataConnector;
//!
//! let connector = TwelvedataConnector::new(api_key);
//!
//! // Market data methods (supported)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let klines = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot).await?;
//!
//! // Trading methods (NOT supported - data provider only)
//! // Will return UnsupportedOperation error
//! ```
//!
//! ## Key Features
//!
//! - **Multi-asset**: Stocks, Forex, Crypto, ETFs, Commodities, Indices
//! - **100+ technical indicators** built-in
//! - **Fundamental data** for US stocks (Grow+ tier)
//! - **WebSocket streaming** (Pro+ tier only)
//! - **Free tier**: 8 req/min, 800 req/day
//! - **Demo API key**: `apikey=demo` for testing
//!
//! ## Important Notes
//!
//! 1. **DATA PROVIDER ONLY**: No trading/order execution capabilities
//! 2. **String numerics**: Time series values returned as strings (preserve precision)
//! 3. **Null handling**: Many fields may be null when data unavailable
//! 4. **Rate limits**: Respect X-RateLimit headers, implement exponential backoff for 429 errors
//! 5. **Credit system**: Different endpoints cost different credits

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{TwelvedataEndpoint, TwelvedataUrls};
pub use auth::TwelvedataAuth;
pub use parser::TwelvedataParser;
pub use connector::TwelvedataConnector;
pub use websocket::TwelvedataWebSocket;
