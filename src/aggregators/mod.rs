//! # Aggregators Module
//!
//! Connectors for data aggregators and multi-asset brokers.
//!
//! ## Available Aggregators
//!
//! - **CryptoCompare**: Crypto price aggregator (5,700+ coins, 170+ exchanges)
//!   - Free tier: 50 req/sec, 1000/min, 150k/hr
//!   - Paid tiers: Higher limits, orderbook access
//!   - REST + WebSocket (WS not implemented yet)
//!   - CCCAGG proprietary index, social metrics, news
//!
//! - **DefiLlama**: TVL, protocol data, token prices, yields
//!   - Free tier: 29 endpoints
//!   - Pro tier: 64 endpoints ($300/mo)
//!   - Hourly data updates
//!   - REST-only (no WebSocket)
//!
//! - **Interactive Brokers**: Multi-asset broker (stocks, forex, futures, options)
//!   - Gateway: 10 req/sec (individual accounts)
//!   - OAuth: 50 req/sec (enterprise accounts)
//!   - REST + WebSocket streaming
//!   - Global markets (150+ exchanges)
//!
//! - **Yahoo Finance**: Multi-asset data aggregator (stocks, crypto, forex, options)
//!   - Unofficial API (reverse-engineered from web app)
//!   - Rate limit: ~2000 req/hr per IP
//!   - Free (no API key required for most endpoints)
//!   - Personal use only per Yahoo's terms
//!
//! ## Key Differences from Exchanges
//!
//! 1. **Data Providers** (CryptoCompare, DefiLlama) - NO Trading
//! 2. **Brokers** (Interactive Brokers) - Full trading capabilities
//! 3. **Multi-Asset** - Support stocks, forex, futures beyond crypto
//! 4. **Aggregated data** - Cross-exchange price feeds, DeFi metrics
//!
//! ## Future Aggregators
//!
//! Potential additions:
//! - DeFi Pulse
//! - DappRadar
//! - Dune Analytics
//! - Token Terminal
//! - Alpaca (US stocks broker)
//! - Polygon.io (stocks data)

pub mod cryptocompare;
pub mod ib;
pub mod yahoo;
pub mod defillama;

pub use cryptocompare::{CryptoCompareConnector, CryptoCompareAuth, CryptoCompareParser, CryptoCompareWebSocket};
pub use ib::IBConnector;
pub use yahoo::{YahooFinanceConnector, YahooFinanceAuth, YahooFinanceParser, YahooFinanceWebSocket};
pub use defillama::{DefiLlamaConnector, DefiLlamaAuth, DefiLlamaParser};
