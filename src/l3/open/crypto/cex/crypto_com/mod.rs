//! # Crypto.com Exchange Connector
//!
//! Complete implementation of Crypto.com Exchange API v1 connector.
//!
//! ## Structure
//!
//! - `endpoints` - URL'ы and endpoint enum
//! - `auth` - Request signing (HMAC-SHA256)
//! - `parser` - JSON response parsing
//! - `connector` - CryptoComConnector + trait implementations
//! - `websocket` - WebSocket client (stub)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::crypto_com::CryptoComConnector;
//! use connectors_v5::core::{Credentials, AccountType};
//!
//! let credentials = Credentials::new("api_key", "api_secret");
//! let connector = CryptoComConnector::new(Some(credentials), false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//! ```
//!
//! ## Key Differences from Other Exchanges
//!
//! - **All numeric values are strings** in API responses (e.g., "50000.00")
//! - **Spot symbols use underscore**: `BTC_USDT`
//! - **Perpetual symbols**: `BTCUSD-PERP` (no underscore)
//! - **Signature algorithm**: `method + id + api_key + params_string + nonce`
//! - **WebSocket**: 1-second delay after connection is REQUIRED

mod endpoints;
mod auth;
mod parser;
mod connector;
pub mod websocket;

pub use endpoints::{CryptoComEndpoint, CryptoComUrls, InstrumentType, format_symbol};
pub use auth::CryptoComAuth;
pub use parser::CryptoComParser;
pub use connector::CryptoComConnector;
pub use websocket::CryptoComWebSocket;

#[cfg(test)]
mod _tests_websocket;

// Research docs are in research/ directory (not exported)
