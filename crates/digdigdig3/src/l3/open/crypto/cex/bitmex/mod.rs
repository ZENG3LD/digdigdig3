//! BitMEX connector — public market data.
//!
//! Primary purpose: `StreamEvent::PredictedFunding` via `instrument:{sym}` WS channel.
//! No authentication — public streams only.

pub mod endpoints;
pub mod auth;
pub mod parser;
pub mod protocol;
pub mod connector;
pub mod websocket;

pub use connector::BitmexConnector;
pub use websocket::BitmexWebSocket;
