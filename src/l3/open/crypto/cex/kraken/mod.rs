//! # Kraken Exchange Connector
//!
//! Complete implementation of Kraken connector for V5 architecture.
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint enum
//! - `auth` - Request signing (HMAC-SHA512)
//! - `parser` - JSON response parsing
//! - `connector` - KrakenConnector + trait implementations
//! - `websocket` - WebSocket v2 connection
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::kraken::KrakenConnector;
//!
//! let connector = KrakenConnector::new(credentials, false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended methods (Kraken-specific)
//! let symbols = connector.get_asset_pairs().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{KrakenEndpoint, KrakenUrls};
pub use auth::KrakenAuth;
pub use parser::KrakenParser;
pub use connector::KrakenConnector;
pub use websocket::KrakenWebSocket;

#[cfg(test)]
mod _tests_websocket;
