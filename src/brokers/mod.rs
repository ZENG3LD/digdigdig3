//! # Brokers Module
//!
//! Multi-asset brokers with full trading capabilities.
//!
//! ## Available Brokers
//!
//! - **Interactive Brokers**: Multi-asset broker (stocks, forex, futures, options)
//!   - Gateway: 10 req/sec (individual accounts)
//!   - OAuth: 50 req/sec (enterprise accounts)
//!   - REST + WebSocket streaming
//!   - Global markets (150+ exchanges)

pub mod ib;

pub use ib::IBConnector;

#[cfg(feature = "websocket")]
pub use ib::IBWebSocket;
