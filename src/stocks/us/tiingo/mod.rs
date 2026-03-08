//! # Tiingo Financial Data Platform Connector
//!
//! Category: stocks/us
//! Type: Multi-asset data provider (stocks, crypto, forex)
//!
//! ## Features
//! - REST API: Yes (https://api.tiingo.com)
//! - WebSocket: Yes (IEX, Forex, Crypto)
//! - Authentication: API Token
//! - Free tier: Yes (5 req/min, 500 req/day)
//!
//! ## Data Types
//! - Price data: Yes (EOD and real-time IEX)
//! - Historical data: Yes (50+ years for stocks)
//! - OHLC aggregates: Yes (1min to 1day intervals)
//! - Fundamentals: Yes (5,500+ equities, 80+ indicators)
//! - News: Yes (curated financial news)
//! - Crypto: Yes (2,100+ tickers, 40+ exchanges)
//! - Forex: Yes (140+ pairs from tier-1 banks)
//!
//! ## Important Notes
//! - This is a **data provider**, not an exchange
//! - Trading methods return `UnsupportedOperation`
//! - Symbol format: "AAPL" for stocks, "btcusd" for crypto, "eurusd" for forex
//! - Authentication: Simple API token (not HMAC)
//! - WebSocket authentication: Token in subscribe message

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use connector::TiingoConnector;
pub use websocket::TiingoWebSocket;
