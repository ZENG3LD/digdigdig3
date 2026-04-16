//! # Upstox Connector
//!
//! Category: stocks/india
//! Type: Indian broker with trading and market data APIs
//!
//! ## Features
//! - REST API: Yes (standard & HFT endpoints)
//! - WebSocket: Yes (Protocol Buffers binary format)
//! - Authentication: OAuth 2.0 (Authorization Code flow)
//! - Free tier: API creation free, usage requires Rs 499/month subscription
//! - Trading: Full support (all order types, GTT orders, multi-order APIs)
//!
//! ## Supported Exchanges
//! - NSE - National Stock Exchange (Equities & F&O)
//! - BSE - Bombay Stock Exchange (Equities & F&O)
//! - MCX - Multi Commodity Exchange (Futures)
//!
//! ## Data Types
//! - Market quotes: Yes (LTP, OHLC, Full with depth)
//! - Historical candles: Yes (from year 2000 for daily, 2022 for intraday)
//! - Order management: Yes (all order types including GTT)
//! - Account data: Yes (profile, margins, holdings, positions)
//! - Trading: Yes (market, limit, SL, SL-M, GTT, AMO)
//! - Options: Full support (chain, Greeks, IV)
//!
//! ## Authentication
//! OAuth 2.0 with Bearer token.
//! Token expires at 3:30 AM IST daily (no refresh token).
//! See `auth.rs` for details.
//!
//! ## Usage
//! ```ignore
//! use connectors_v5::stocks::india::upstox::UpstoxConnector;
//! use connectors_v5::core::traits::*;
//!
//! let connector = UpstoxConnector::new(credentials, false).await?;
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use connector::UpstoxConnector;
