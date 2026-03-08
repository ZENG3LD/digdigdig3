//! # FRED (Federal Reserve Economic Data) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (completely free for non-commercial use)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes (840,000+ economic time series)
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes
//! - Vintage/revision data: Yes (ALFRED)
//!
//! ## Key Endpoints
//! - /fred/series/observations - Get time series data (CORE endpoint)
//! - /fred/series - Get series metadata
//! - /fred/series/search - Search for series
//! - /fred/category/* - Browse by category
//! - /fred/releases/* - Browse by release
//! - /fred/tags/* - Browse by tags
//!
//! ## Rate Limits
//! - Free tier: 120 requests per minute
//! - No paid tiers available
//!
//! ## Data Coverage
//! - 840,000+ economic time series
//! - 118 data sources
//! - Historical depth varies (some from 1700s, most from 1900s+)
//! - Update frequency varies by series
//!
//! ## Usage Restrictions
//! - Non-commercial use only (free tier)
//! - No AI/ML training prohibited
//! - No caching/archiving of data
//! - Attribution required

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{FredEndpoint, FredEndpoints};
pub use auth::FredAuth;
pub use parser::{
    FredParser, Observation, SeriesMetadata, Category, Release, Source, Tag,
    ReleaseDate, SeriesUpdate, VintageDate,
};
pub use connector::FredConnector;
