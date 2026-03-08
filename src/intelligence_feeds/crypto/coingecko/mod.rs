//! # CoinGecko API Connector
//!
//! Category: data_feeds
//! Type: Cryptocurrency Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Optional API Key (x-cg-demo-key or x-cg-pro-key header)
//! - Free tier: Yes (10-30 calls/min)
//!
//! ## Data Types
//! - Price data: Yes (real-time crypto prices)
//! - Historical data: Yes (price/volume history)
//! - Market data: Yes (market cap, volume, rankings)
//! - Exchange data: Yes (exchange info, tickers)
//! - Trending: Yes (trending coins)
//!
//! ## Key Endpoints
//! - /simple/price - Simple price lookup
//! - /coins/list - List all coins
//! - /coins/{id} - Coin details
//! - /coins/{id}/market_chart - Historical price/volume
//! - /coins/markets - Market overview
//! - /search/trending - Trending coins
//! - /global - Global market data
//! - /exchanges - Exchange list
//!
//! ## Rate Limits
//! - Free tier: 10-30 calls/min
//! - Demo key: 30 calls/min
//! - Pro: Higher limits (varies by plan)
//!
//! ## Usage
//! Environment variable: `COINGECKO_API_KEY` (optional)

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{CoinGeckoEndpoint, CoinGeckoEndpoints};
pub use auth::CoinGeckoAuth;
pub use parser::{
    CoinGeckoParser, CoinPrice, CoinDetail, MarketData, CoinMarketChart,
    TrendingCoin, GlobalData, CoinGeckoExchange, SimpleCoin,
};
pub use connector::CoinGeckoConnector;
