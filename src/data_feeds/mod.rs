//! # Data Feeds Module
//!
//! Read-only market data aggregators and data providers.
//!
//! ## Available Data Feeds
//!
//! - **CryptoCompare**: Crypto price aggregator (5,700+ coins, 170+ exchanges)
//!   - Free tier: 50 req/sec, 1000/min, 150k/hr
//!   - Paid tiers: Higher limits, orderbook access
//!   - REST + WebSocket (WS not implemented yet)
//!   - CCCAGG proprietary index, social metrics, news
//!
//! - **Yahoo Finance**: Multi-asset data aggregator (stocks, crypto, forex, options)
//!   - Unofficial API (reverse-engineered from web app)
//!   - Rate limit: ~2000 req/hr per IP
//!   - Free (no API key required for most endpoints)
//!   - Personal use only per Yahoo's terms

pub mod cryptocompare;
pub mod yahoo;

pub use cryptocompare::{CryptoCompareConnector, CryptoCompareAuth, CryptoCompareParser, CryptoCompareWebSocket};
pub use yahoo::{YahooFinanceConnector, YahooFinanceAuth, YahooFinanceParser, YahooFinanceWebSocket};
