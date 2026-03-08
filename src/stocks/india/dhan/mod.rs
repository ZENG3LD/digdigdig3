//! # Dhan Exchange Connector
//!
//! Full implementation for Dhan Indian stock broker.
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint enum
//! - `auth` - Request signing (JWT token-based)
//! - `parser` - JSON response parsing
//! - `connector` - DhanConnector + trait implementations
//! - `websocket` - WebSocket connection (binary format)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::stocks::india::dhan::DhanConnector;
//!
//! let connector = DhanConnector::new(credentials, false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended methods (Dhan-specific)
//! let holdings = connector.get_holdings().await?;
//! let positions = connector.get_positions_detail().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{DhanEndpoint, DhanUrls};
pub use auth::DhanAuth;
pub use parser::DhanParser;
pub use connector::DhanConnector;
pub use websocket::DhanWebSocket;
