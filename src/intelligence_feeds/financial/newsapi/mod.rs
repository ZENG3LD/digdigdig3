//! NewsAPI.org connector
//!
//! Provides access to news articles from 150,000+ sources worldwide.
//!
//! # Features
//! - Top headlines by country/category
//! - Full-text search across all articles
//! - News sources directory
//! - Multiple language support
//! - Convenience methods for financial, crypto, market, and economic news
//!
//! # Authentication
//! Set the `NEWSAPI_KEY` environment variable with your API key from newsapi.org
//!
//! # Rate Limits
//! - Free tier: 100 requests/day
//! - Developer tier: 250 requests/day
//! - Business tier: Up to 250,000 requests/day
//!
//! # Example
//! ```ignore
//! use connectors_v5::data_feeds::newsapi::NewsApiConnector;
//!
//! let connector = NewsApiConnector::from_env();
//!
//! // Get top business headlines from US
//! let headlines = connector.get_business_news(Some("us")).await?;
//!
//! // Get crypto news
//! let crypto = connector.get_crypto_news().await?;
//!
//! // Search for specific topics
//! let articles = connector.get_everything(
//!     Some("bitcoin"),
//!     None,
//!     None,
//!     Some("2024-01-01"),
//!     None,
//!     None,
//!     None,
//!     Some(50),
//!     None,
//! ).await?;
//! ```

pub mod auth;
pub mod connector;
pub mod endpoints;
pub mod parser;

pub use auth::NewsApiAuth;
pub use connector::NewsApiConnector;
pub use endpoints::{NewsApiEndpoint, NewsApiEndpoints, NewsCategory, NewsLanguage, NewsSortBy};
pub use parser::{NewsApiParser, NewsArticle, NewsSource, NewsSourceMetadata};
