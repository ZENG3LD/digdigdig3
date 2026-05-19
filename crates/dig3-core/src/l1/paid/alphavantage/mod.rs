//! # AlphaVantage Connector
//!
//! Category: forex (also supports stocks, crypto, commodities, economic indicators)
//! Type: Data Provider ONLY - no trading capabilities
//!
//! ## Features
//! - REST API: Yes (function-based)
//! - WebSocket: NO
//! - Authentication: API key (query parameter)
//! - Free tier: Yes (25 req/day, 5 req/min)
//!
//! ## Data Types
//! - Forex data: Yes (182 physical currencies)
//! - Stock data: Yes (200,000+ global tickers)
//! - Crypto data: Yes
//! - Technical indicators: Yes (50+ indicators)
//! - Fundamental data: Yes (stocks only)
//! - Economic data: Yes (GDP, CPI, unemployment, etc.)
//!
//! ## Limitations
//! - NO WebSocket support (REST only)
//! - NO trading support (data provider only)
//! - NO account operations
//! - Very restrictive free tier (25 req/day)
//! - Intraday data requires premium tier
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::forex::alphavantage::AlphaVantageConnector;
//!
//! let connector = AlphaVantageConnector::from_env();
//!
//! // Get forex exchange rate
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//!
//! // Get daily forex candles
//! let klines = connector.get_klines(symbol, "1d", Some(100), AccountType::Spot, None).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(test)]
mod tests;

pub use connector::AlphaVantageConnector;
pub use endpoints::{AlphaVantageFunction, AlphaVantageEndpoints};
pub use auth::AlphaVantageAuth;
pub use parser::AlphaVantageParser;
