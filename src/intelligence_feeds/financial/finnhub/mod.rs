//! # Finnhub API Connector
//!
//! Category: data_feeds
//! Type: Financial Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: Yes
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (60 requests/minute)
//!
//! ## Data Types
//! - Stock quotes: Yes
//! - Historical candles: Yes (stock, forex, crypto)
//! - Company data: Yes (profile, peers, financials)
//! - Market news: Yes
//! - Economic calendar: Yes
//! - Sentiment data: Yes (social, insider, analyst)
//!
//! ## Key Endpoints
//! - /quote - Real-time stock quotes
//! - /stock/candle - Historical OHLC data
//! - /search - Symbol search
//! - /stock/profile2 - Company profiles
//! - /news - Market news
//! - /company-news - Company-specific news
//! - /calendar/earnings - Earnings calendar
//! - /stock/social-sentiment - Social sentiment
//!
//! ## Rate Limits
//! - Free tier: 60 requests per minute
//! - Paid tiers available with higher limits
//!
//! ## Data Coverage
//! - Stock markets: Global
//! - Forex: Major pairs
//! - Crypto: Major exchanges
//! - News: Real-time from multiple sources
//! - Economic indicators: Global
//!
//! ## Usage Example
//! ```ignore
//! use connectors_v5::data_feeds::finnhub::FinnhubConnector;
//!
//! let connector = FinnhubConnector::from_env();
//!
//! // Get real-time quote
//! let quote = connector.get_quote("AAPL").await?;
//! println!("Current price: {}", quote.current_price);
//!
//! // Get historical data
//! let candles = connector.get_candles("AAPL", "D", 1640000000, 1650000000).await?;
//!
//! // Search for symbols
//! let results = connector.search_symbols("Apple").await?;
//!
//! // Get company profile
//! let profile = connector.get_company_profile("AAPL").await?;
//! println!("Company: {}", profile.name);
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{FinnhubEndpoint, FinnhubEndpoints, format_symbol};
pub use auth::FinnhubAuth;
pub use parser::{
    FinnhubParser, Quote, Candle, SearchResult, CompanyProfile, NewsArticle,
    MarketStatus, Earnings, Ipo, SocialSentiment, SentimentData,
    InsiderTransaction, RecommendationTrend,
};
pub use connector::FinnhubConnector;
