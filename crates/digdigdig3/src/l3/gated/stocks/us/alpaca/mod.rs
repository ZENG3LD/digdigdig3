//! # Alpaca Connector
//!
//! Category: stocks/us
//! Type: US Stock Broker + Market Data Provider
//!
//! ## Features
//! - REST API: Yes (Trading + Market Data)
//! - WebSocket: Yes (Market Data + Trading Updates)
//! - Authentication: API Key ID + Secret (simple, no HMAC)
//! - Free tier: Yes (IEX feed, 200 API calls/min, paper trading)
//!
//! ## Data Types
//! - Price data: Yes (stocks, options, crypto)
//! - Historical data: Yes (7+ years stocks, 6+ years crypto)
//! - Derivatives data: Yes (options with greeks, IV)
//! - Fundamentals: No (use separate provider)
//! - News: Yes (Benzinga feed)
//! - Corporate actions: Yes (dividends, splits)
//!
//! ## Trading Capabilities
//! - Commission-free trading (stocks, ETFs, options, crypto)
//! - Paper trading (free, unlimited)
//! - Fractional shares (2,000+ symbols)
//! - Margin trading (up to 4X intraday)
//! - Options (up to Level 3)
//! - Crypto (24/7 spot trading)
//!
//! ## Limitations
//! - US markets only (no international stocks)
//! - Live trading US residents only (paper trading global)
//! - No Level 2 orderbook for stocks (crypto only)
//! - IEX feed on free tier (~2.5% market volume)
//! - SIP feed requires paid tier ($99/mo for all US exchanges)

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use connector::AlpacaConnector;
pub use websocket::AlpacaWebSocket;
