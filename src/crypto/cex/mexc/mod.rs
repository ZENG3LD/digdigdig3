//! # MEXC Connector
//!
//! Exchange connector for MEXC Spot API.
//!
//! ## Architecture
//!
//! - `endpoints` - URL structures and endpoint definitions
//! - `auth` - HMAC-SHA256 signature implementation
//! - `parser` - Response parsing to internal types
//! - `connector` - Trait implementations (MarketData, Trading, Account)
//! - `websocket` - WebSocket connector implementation
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::mexc::MexcConnector;
//! use connectors_v5::core::types::{Symbol, AccountType};
//! use connectors_v5::core::traits::MarketData;
//!
//! // Public API
//! let connector = MexcConnector::public().await?;
//! let ticker = connector.get_ticker(&Symbol::new("BTC", "USDT"), AccountType::Spot).await?;
//!
//! // Private API
//! let credentials = Credentials::new("api_key", "api_secret");
//! let connector = MexcConnector::new(Some(credentials)).await?;
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
