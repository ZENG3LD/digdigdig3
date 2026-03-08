//! CryptoCompare connector
//!
//! Category: aggregators
//! Type: Data Provider (acquired by CoinDesk/CCData)
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: Yes (not implemented yet)
//! - Authentication: API Key (optional for some endpoints, required for full rate limits)
//! - Free tier: Yes (50 req/sec, 1000/min, 150k/hr)
//!
//! ## Data Types
//! - Price data: Yes (spot only)
//! - Historical data: Yes (full history for daily/hourly, 7 days minute bars free tier)
//! - Derivatives data: No (spot crypto aggregator only)
//! - Fundamentals: No (crypto focus)
//! - On-chain: Limited (basic blockchain stats)
//! - Macro data: No
//! - Social metrics: Yes (Reddit, Twitter, Facebook, GitHub)
//! - News: Yes (aggregated from multiple sources)
//!
//! ## Coverage
//! - 5,700+ cryptocurrencies
//! - 170+ exchanges aggregated
//! - 260,000+ trading pairs
//! - CCCAGG proprietary index (volume-weighted aggregate)
//!
//! ## Notes
//! - CryptoCompare is a DATA PROVIDER - trading/account operations not supported
//! - API key recommended for better rate limits
//! - Attribution required for free tier: "Powered by CryptoCompare"
//! - Acquired by CoinDesk, now operates under CCData brand

mod endpoints;
mod auth;
mod parser;
mod connector;
pub mod websocket;

pub use endpoints::{CryptoCompareEndpoints, CryptoCompareEndpoint, format_symbol};
pub use auth::CryptoCompareAuth;
pub use parser::CryptoCompareParser;
pub use connector::CryptoCompareConnector;
pub use websocket::CryptoCompareWebSocket;
