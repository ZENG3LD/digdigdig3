//! # GMX Exchange Connector
//!
//! Full implementation of GMX V2 connector.
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint enum
//! - `auth` - No-op authentication (public endpoints only)
//! - `parser` - JSON response parsing with 30-decimal price handling
//! - `connector` - GmxConnector + MarketData trait implementation
//! - `websocket` - Polling-based WebSocket simulation
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::gmx::GmxConnector;
//!
//! // Create connector for Arbitrum
//! let connector = GmxConnector::arbitrum().await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(symbol, AccountType::FuturesCross).await?;
//! let klines = connector.get_klines(symbol, "1h", Some(100), AccountType::FuturesCross, None).await?;
//!
//! // Extended methods (GMX-specific)
//! let markets = connector.get_markets().await?;
//! let tickers = connector.get_all_tickers().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;
#[cfg(feature = "onchain-evm")]
pub mod onchain;

pub use endpoints::{GmxEndpoint, GmxUrls};
pub use auth::GmxAuth;
pub use parser::GmxParser;
pub use connector::GmxConnector;
pub use websocket::GmxWebSocket;

#[cfg(feature = "onchain-evm")]
pub use onchain::{
    GmxOnchain, GmxOrderType, GmxPositionSide,
    CreatePositionParams, ClosePositionParams,
};
