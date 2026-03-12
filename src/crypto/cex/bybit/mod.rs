//! # Bybit V5 Connector
//!
//! Exchange connector for Bybit V5 API.
//!
//! ## Architecture
//!
//! - `endpoints` - URL structures and endpoint definitions
//! - `auth` - HMAC-SHA256 signature implementation
//! - `parser` - Response parsing to internal types
//! - `connector` - Trait implementations (MarketData, Trading, Account, Positions)
//! - `websocket` - WebSocket connector implementation
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::bybit::BybitConnector;
//! use connectors_v5::core::types::{Symbol, AccountType};
//! use connectors_v5::core::traits::MarketData;
//!
//! // Public API
//! let connector = BybitConnector::public(false).await?;
//! let ticker = connector.get_ticker(&Symbol::new("BTC", "USDT"), AccountType::Spot).await?;
//!
//! // Private API
//! let credentials = Credentials::new("api_key", "api_secret");
//! let connector = BybitConnector::new(Some(credentials), false).await?;
//! let balance = connector.get_balance(crate::core::types::BalanceQuery { asset: None, account_type: AccountType::Spot }).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::*;
pub use auth::*;
pub use parser::*;
pub use connector::*;
pub use websocket::*;
