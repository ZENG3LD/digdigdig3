//! # Zerodha Kite Connect Connector
//!
//! Category: stocks/india
//! Type: Full-service broker
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: Yes (Binary + JSON)
//! - Authentication: Custom OAuth-like with SHA-256 checksum
//! - Free tier: Limited (Personal API - no WebSocket, no historical data)
//! - Paid tier: ₹500/month (Connect API - full access)
//!
//! ## Supported Exchanges
//! - NSE - National Stock Exchange (Equities)
//! - BSE - Bombay Stock Exchange (Equities)
//! - NFO - NSE Futures & Options
//! - BFO - BSE Futures & Options
//! - MCX - Multi Commodity Exchange
//! - CDS - Currency Derivatives (NSE)
//! - BCD - BSE Currency Derivatives
//!
//! ## Data Types
//! - Market quotes: Yes (LTP, OHLC, Full with 5-level depth)
//! - Historical candles: Yes (minute to day intervals)
//! - Order management: Yes (regular, AMO, GTT, iceberg)
//! - Account data: Yes (profile, margins, holdings, positions)
//! - Trading: Yes (all order types, products)
//!
//! ## Authentication
//! Custom OAuth-like flow with SHA-256 checksum.
//! See `auth.rs` for details.
//!
//! ## Usage
//! ```ignore
//! use connectors_v5::stocks::india::zerodha::ZerodhaConnector;
//! use connectors_v5::core::traits::*;
//!
//! let connector = ZerodhaConnector::from_env();
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(feature = "websocket")]
mod websocket;

pub use connector::ZerodhaConnector;

#[cfg(feature = "websocket")]
pub use websocket::ZerodhaWebSocket;
