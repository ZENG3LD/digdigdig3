//! # Wikipedia Pageviews Connector
//!
//! Category: data_feeds
//! Type: Alternative Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (User-Agent header only)
//! - Free tier: Yes (completely free, no rate limits)
//!
//! ## Data Types
//! - Pageview data: Yes (article views, aggregate views)
//! - Top articles: Yes (most viewed articles by day)
//! - Geographic data: Yes (pageviews by country)
//! - Time series: Yes (daily/monthly granularity)
//!
//! ## Key Endpoints
//! - /per-article/* - Get pageviews for a specific article
//! - /aggregate/* - Get total pageviews for entire project
//! - /top/* - Get most viewed articles for a date
//! - /top-per-country/* - Get pageviews by country
//!
//! ## Rate Limits
//! - Free tier: No explicit rate limits
//! - Wikimedia Foundation APIs are open and free
//!
//! ## Data Coverage
//! - All Wikimedia projects (Wikipedia, Wiktionary, Wikimedia, etc.)
//! - All languages (en.wikipedia, de.wikipedia, etc.)
//! - Historical data from 2015-07-01 onwards
//! - Daily and monthly granularity
//!
//! ## Usage Restrictions
//! - Must set User-Agent header
//! - Rate limiting encouraged but not enforced
//! - Attribution appreciated

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{WikipediaEndpoint, WikipediaEndpoints};
pub use auth::WikipediaAuth;
pub use parser::{
    WikipediaParser, PageviewsEntry, TopArticle, TopCountry,
};
pub use connector::WikipediaConnector;
