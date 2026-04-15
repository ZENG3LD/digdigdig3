//! # Bitfinex Exchange Connector
//!
//! Full implementation of Bitfinex API v2 connector.
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint enum
//! - `auth` - Request signing (HMAC-SHA384)
//! - `parser` - JSON array parsing
//! - `connector` - BitfinexConnector + trait implementations
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::bitfinex::BitfinexConnector;
//!
//! let connector = BitfinexConnector::new(credentials, false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//! ```
//!
//! ## Key Differences from Other Exchanges
//!
//! 1. **Array-based responses**: All data returned as arrays, not objects
//! 2. **HMAC-SHA384**: Uses SHA384 instead of SHA256
//! 3. **Microsecond nonces**: Nonces in microseconds, not milliseconds
//! 4. **Symbol prefixes**: `t` for trading pairs, `f` for funding
//! 5. **Signed amounts**: Positive=buy, negative=sell for orders/positions

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{BitfinexEndpoint, BitfinexUrls};
pub use auth::BitfinexAuth;
pub use parser::BitfinexParser;
pub use connector::BitfinexConnector;
pub use websocket::BitfinexWebSocket;

#[cfg(test)]
mod _tests_websocket;
