//! Yahoo Finance connector
//!
//! Category: aggregators
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: Yes (Protobuf-based, not implemented yet)
//! - Authentication: Cookie + Crumb (for historical download), None for most endpoints
//! - Free tier: Yes (rate-limited ~2000 req/hr)
//!
//! ## Data Types
//! - Price data: Yes (real-time quotes)
//! - Historical data: Yes (OHLCV candles via chart endpoint)
//! - Options data: Yes (full options chain with Greeks)
//! - Fundamentals: Yes (income statements, balance sheets, cash flows, earnings)
//! - Ownership data: Yes (institutional holders, insider transactions)
//! - Market analysis: Yes (analyst ratings, recommendations, trending symbols)
//!
//! ## Important Notes
//! - **UNOFFICIAL API**: Yahoo Finance shut down official API in 2017
//! - All endpoints are reverse-engineered from web app
//! - Risk of breaking changes without notice
//! - Aggressive rate limiting (~2000 req/hr per IP)
//! - Personal use only per Yahoo's terms
//!
//! ## Symbol Formats
//! - US Stocks: `AAPL`, `MSFT`, `GOOGL`
//! - Crypto: `BTC-USD`, `ETH-USD`
//! - Forex: `EURUSD=X`, `GBPUSD=X`
//! - Commodities: `GC=F` (Gold), `CL=F` (Oil)
//! - Indices: `^GSPC` (S&P 500), `^DJI` (Dow)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::aggregators::yahoo::YahooFinanceConnector;
//! use connectors_v5::core::types::{Symbol, AccountType};
//! use connectors_v5::core::traits::MarketData;
//!
//! // Most endpoints don't require authentication
//! let connector = YahooFinanceConnector::new();
//!
//! // Get real-time quote
//! let symbol = Symbol::new("AAPL", "USD");
//! let ticker = connector.get_ticker(symbol, AccountType::Spot).await?;
//!
//! // Get historical candles
//! let klines = connector.get_klines(symbol, "1d", Some(30), AccountType::Spot, None).await?;
//!
//! // Extended methods (Yahoo-specific)
//! let options_chain = connector.get_options_chain("AAPL").await?;
//! let company_profile = connector.get_asset_profile("AAPL").await?;
//! let earnings = connector.get_earnings("AAPL").await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
pub mod websocket;

pub use endpoints::{YahooFinanceEndpoint, YahooFinanceUrls};
pub use auth::YahooFinanceAuth;
pub use parser::YahooFinanceParser;
pub use connector::YahooFinanceConnector;
pub use websocket::YahooFinanceWebSocket;
