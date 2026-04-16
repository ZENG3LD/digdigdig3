//! # OANDA v20 Connector
//!
//! Full implementation of OANDA v20 REST API connector.
//!
//! ## Provider Type
//! Forex Broker (both market data AND trading capabilities)
//!
//! ## Features
//! - Bearer token authentication
//! - 120+ forex pairs, metals, commodities, indices
//! - Market data (pricing, candles, orderbook)
//! - Trading (market orders, limit orders, positions)
//! - Account management
//! - HTTP streaming (pricing and transactions)
//!
//! ## Symbol Format
//! - EUR/USD → EUR_USD
//! - GBP/JPY → GBP_JPY
//! - XAU/USD → XAU_USD (Gold)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::forex::oanda::OandaConnector;
//! use connectors_v5::{Credentials, Symbol, AccountType};
//!
//! // Practice account
//! let credentials = Credentials::new("YOUR_BEARER_TOKEN", "");
//! let mut connector = OandaConnector::new(credentials, true).await?;
//!
//! // Get EUR/USD price
//! let symbol = Symbol::new("EUR", "USD");
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//!
//! // Place market order
//! let order = connector.market_order(
//!     symbol,
//!     OrderSide::Buy,
//!     10000.0, // 10,000 units
//!     AccountType::Spot
//! ).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod streaming;

pub use endpoints::{OandaEndpoint, OandaUrls, format_symbol, parse_symbol, map_granularity};
pub use auth::OandaAuth;
pub use parser::OandaParser;
pub use connector::OandaConnector;
pub use streaming::{PricingStream, TransactionStream, StreamMessage, PriceUpdate};
