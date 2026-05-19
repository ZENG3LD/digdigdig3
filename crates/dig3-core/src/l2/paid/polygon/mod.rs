//! # Polygon.io (Massive.com) Connector
//!
//! Category: stocks/us
//! Type: Stock market data provider
//!
//! ## Features
//! - REST API: Yes (https://api.massive.com)
//! - WebSocket: Yes (real-time and 15-min delayed)
//! - Authentication: API Key
//! - Free tier: Yes (5 req/min, EOD data)
//!
//! ## Data Types
//! - Price data: Yes (real-time on Advanced+, delayed on lower tiers)
//! - Historical data: Yes (20+ years on Advanced, 2-10 years on lower tiers)
//! - OHLC aggregates: Yes (1s to 1y intervals)
//! - Trades: Yes (tick-level, Advanced+)
//! - Quotes: Yes (NBBO, Advanced+)
//! - Fundamentals: Yes (financials, dividends, ratios)
//! - Technical indicators: Yes (server-computed SMA/EMA/MACD/RSI)
//!
//! ## Important Notes
//! - This is a **data provider**, not an exchange
//! - Trading methods return `UnsupportedOperation`
//! - Symbol format: "AAPL" (not "BTC-USDT")
//! - Authentication: Simple API key (not HMAC)

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use connector::PolygonConnector;
pub use websocket::PolygonWebSocket;
