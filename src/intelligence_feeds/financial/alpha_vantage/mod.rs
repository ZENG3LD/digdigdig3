//! Alpha Vantage API connector
//!
//! Multi-asset data provider covering stocks, forex, crypto, economic indicators, and commodities.
//!
//! # Features
//! - Stock quotes and time series (intraday, daily, weekly, monthly)
//! - Forex exchange rates and time series
//! - Crypto ratings and daily data
//! - Economic indicators (GDP, CPI, unemployment, etc.)
//! - Technical indicators (SMA, EMA, RSI, MACD)
//! - Commodities (WTI, Brent, natural gas, copper)
//!
//! # Authentication
//! Set environment variable: `ALPHA_VANTAGE_API_KEY`
//!
//! Get your free API key at: https://www.alphavantage.co/support/#api-key
//!
//! # Rate Limits
//! - Free tier: 25 requests/day, 5 requests/minute
//! - Premium: 75+ requests/minute (depending on plan)
//!
//! # Example
//! ```ignore
//! use connectors_v5::data_feeds::alpha_vantage::AlphaVantageConnector;
//!
//! let connector = AlphaVantageConnector::from_env();
//!
//! // Stock data
//! let quote = connector.get_quote("IBM").await?;
//! let daily = connector.get_daily("AAPL").await?;
//!
//! // Economic data
//! let gdp = connector.get_gdp().await?;
//! let unemployment = connector.get_unemployment().await?;
//!
//! // Forex
//! let rate = connector.get_fx_rate("USD", "EUR").await?;
//!
//! // Crypto
//! let rating = connector.get_crypto_rating("BTC").await?;
//!
//! // Technical indicators
//! let sma = connector.get_sma("MSFT", "daily", 20, "close").await?;
//! ```

pub mod endpoints;
pub mod auth;
pub mod parser;
pub mod connector;

pub use connector::AlphaVantageConnector;
pub use auth::AlphaVantageAuth;
pub use endpoints::{AlphaVantageEndpoint, AlphaVantageEndpoints};
pub use parser::{
    AlphaVantageParser,
    GlobalQuote,
    TimeSeriesEntry,
    SymbolMatch,
    ForexRate,
    CryptoRating,
    EconomicDataPoint,
    TechnicalIndicatorEntry,
};
