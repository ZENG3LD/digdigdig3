//! # OKX Exchange Connector
//!
//! Implementation of OKX API v5 following the V5 connector architecture.
//!
//! ## Modules
//! - `endpoints` - API endpoints and URL configuration
//! - `auth` - Request signing and authentication
//! - `parser` - Response parsing
//! - `connector` - Main connector implementing all traits
//! - `websocket` - WebSocket implementation
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::okx::OkxConnector;
//! use connectors_v5::core::{Credentials, AccountType, Symbol};
//! use connectors_v5::core::traits::{MarketData, Trading};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create connector
//!     let credentials = Credentials::new("api_key", "api_secret")
//!         .with_passphrase("passphrase");
//!     let okx = OkxConnector::new(Some(credentials), false).await?;
//!
//!     // Get price
//!     let symbol = Symbol::new("BTC", "USDT");
//!     let price = okx.get_price(symbol, AccountType::Spot).await?;
//!     println!("BTC-USDT: {}", price);
//!
//!     Ok(())
//! }
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

#[cfg(test)]
mod _tests_websocket;
