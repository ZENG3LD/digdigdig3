//! # ReliefWeb API Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Optional (appname query parameter)
//! - Free tier: Yes (public API)
//!
//! ## Data Types
//! - Humanitarian reports: Yes
//! - Disaster data: Yes
//! - Country profiles: Yes
//! - Job listings: Yes
//! - Training opportunities: Yes
//! - Source information: Yes
//!
//! ## Key Endpoints
//! - /reports - Humanitarian reports and situation updates
//! - /disasters - Natural disasters and crises
//! - /countries - Country profiles
//! - /jobs - Humanitarian job listings
//! - /training - Training opportunities
//! - /sources - Organizations and sources
//!
//! ## Rate Limits
//! - No documented rate limits (public API)
//! - Recommended to use appname parameter for identification
//!
//! ## Data Coverage
//! - Global humanitarian data from UN OCHA
//! - Real-time crisis and disaster information
//! - Comprehensive reports and situation updates
//!
//! ## Usage Restrictions
//! - Public API, attribution recommended
//! - Check ReliefWeb terms of use for specific restrictions

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{ReliefWebEndpoint, ReliefWebEndpoints};
pub use auth::ReliefWebAuth;
pub use parser::{ReliefWebParser, ReliefWebReport, ReliefWebDisaster, ReliefWebCountry, ReliefWebSearchResult};
pub use connector::ReliefWebConnector;
