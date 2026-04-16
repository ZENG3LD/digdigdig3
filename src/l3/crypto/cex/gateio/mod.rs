//! # Gate.io Exchange Connector
//!
//! Full implementation of Gate.io V4 API connector.
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint enum
//! - `auth` - Request signing (HMAC-SHA512)
//! - `parser` - JSON response parsing
//! - `connector` - GateioConnector + trait implementations
//! - `websocket` - WebSocket connection
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::gateio::GateioConnector;
//!
//! let connector = GateioConnector::new(credentials, false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{GateioEndpoint, GateioUrls};
pub use auth::GateioAuth;
pub use parser::GateioParser;
pub use connector::GateioConnector;
pub use websocket::GateioWebSocket;

#[cfg(test)]
mod _tests_websocket;
