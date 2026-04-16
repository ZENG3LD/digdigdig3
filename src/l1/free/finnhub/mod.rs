//! # Finnhub Connector
//!
//! Category: stocks/us
//! Type: Stock market data provider
//!
//! ## Features
//! - REST API: Yes (https://finnhub.io/api/v1)
//! - WebSocket: Yes (wss://ws.finnhub.io)
//! - Authentication: API Key (X-Finnhub-Token header or query param)
//! - Free tier: Yes (60 req/min)
//!
//! ## Data Types
//! - Price data: Yes (real-time US stocks, delayed international)
//! - Historical data: Yes (1 year on free tier, 25+ years on paid)
//! - OHLC candles: Yes (1m to monthly intervals)
//! - Company fundamentals: Yes (profile, financials, metrics)
//! - News: Yes (company and market news)
//! - Technical indicators: Yes (server-computed)
//!
//! ## Important Notes
//! - This is a **data provider**, not an exchange
//! - Trading methods return `UnsupportedOperation`
//! - Symbol format: "AAPL" (not "BTC-USDT")
//! - Authentication: Simple API key (not HMAC)
//! - Rate limit: 60 req/min (free tier), 30 req/sec hard cap

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use connector::FinnhubConnector;
pub use websocket::FinnhubWebSocket;
