//! # Coinbase Advanced Trade Connector
//!
//! Exchange connector for Coinbase Advanced Trade API.
//!
//! ## Architecture
//!
//! - `endpoints` - URL structures and endpoint definitions
//! - `auth` - JWT (ES256) signature implementation
//! - `parser` - Response parsing to internal types
//! - `connector` - Trait implementations (MarketData, Trading, Account)
//! - `websocket` - WebSocket connector implementation
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::coinbase::CoinbaseConnector;
//! use connectors_v5::core::types::{Symbol, AccountType};
//! use connectors_v5::core::traits::MarketData;
//!
//! // Public API
//! let connector = CoinbaseConnector::public().await?;
//! let ticker = connector.get_ticker(&Symbol::new("BTC", "USD"), AccountType::Spot).await?;
//!
//! // Private API
//! let credentials = Credentials::new("organizations/.../apiKeys/...", "-----BEGIN EC PRIVATE KEY-----\n...");
//! let connector = CoinbaseConnector::new(Some(credentials)).await?;
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
