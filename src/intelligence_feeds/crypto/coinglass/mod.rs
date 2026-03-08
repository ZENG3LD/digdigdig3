//! # Coinglass Exchange Connector
//!
//! Полная реализация коннектора для Coinglass API V4.
//!
//! ## Important: Derivatives Analytics Provider
//!
//! Coinglass is NOT a standard exchange - it's a derivatives analytics platform:
//! - NO trading operations (Trading trait returns UnsupportedOperation)
//! - NO account balances (Account trait returns UnsupportedOperation)
//! - NO standard price/OHLC data (use exchanges for that)
//!
//! ## Focus Areas
//!
//! - **Liquidations** - Real-time and historical liquidation data
//! - **Open Interest** - Aggregated OI across exchanges
//! - **Funding Rates** - Current and historical funding rates
//! - **Long/Short Ratios** - Account and position ratios
//! - **On-Chain Analytics** - Exchange flows, whale transfers
//! - **ETF Tracking** - Bitcoin/Ethereum ETF flows
//!
//! ## Структура
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - API key authentication
//! - `parser` - Парсинг JSON ответов (custom data structures)
//! - `connector` - CoinglassConnector + impl трейтов
//!
//! ## Использование
//!
//! ```ignore
//! use connectors_v5::data_feeds::coinglass::CoinglassConnector;
//! use connectors_v5::Credentials;
//!
//! // Create connector with API key (requires paid subscription)
//! let credentials = Credentials {
//!     api_key: "your_api_key".to_string(),
//!     api_secret: String::new(),
//!     passphrase: None,
//! };
//!
//! // Specify rate limit based on your tier (30, 80, 300, 1200, 6000)
//! let connector = CoinglassConnector::new(credentials, 80).await?;
//!
//! // Get liquidation data
//! let liquidations = connector.get_liquidation_history("BTC", "1h", Some(100)).await?;
//!
//! // Get Open Interest data
//! let oi_data = connector.get_open_interest_ohlc("BTC", "1h", Some(100)).await?;
//!
//! // Get funding rates
//! let funding = connector.get_funding_rate_history("BTC", Some("Binance"), Some(100)).await?;
//!
//! // Get long/short ratios
//! let ratios = connector.get_long_short_ratio("BTC", "1h", Some(100)).await?;
//! ```
//!
//! ## API Key Requirements
//!
//! Coinglass has NO free tier. API access requires a paid subscription:
//! - Hobbyist: $29/mo (30 req/min)
//! - Startup: $79/mo (80 req/min) - Recommended
//! - Standard: $299/mo (300 req/min)
//! - Professional: $699/mo (1200 req/min)
//! - Enterprise: Custom pricing (6000 req/min)

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{CoinglassEndpoint, CoinglassUrls};
pub use auth::CoinglassAuth;
pub use parser::{
    CoinglassParser,
    CoinglassResponse,
    LiquidationData,
    OpenInterestOhlc,
    FundingRateData,
    LongShortRatio,
    SupportedCoins,
};
pub use connector::CoinglassConnector;
