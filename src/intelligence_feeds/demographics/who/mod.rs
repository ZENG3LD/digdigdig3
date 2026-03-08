//! # WHO GHO (Global Health Observatory) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free, no limits)
//!
//! ## Data Types
//! - Health indicators: Yes (1000+ indicators)
//! - Country data: Yes (194 countries)
//! - Regional data: Yes (6 WHO regions)
//! - Time series: Yes (historical health data)
//!
//! ## Key Endpoints
//! - /Indicator - List all health indicators
//! - /{IndicatorCode} - Get data for specific indicator
//! - /DIMENSION/COUNTRY - List countries
//! - /DIMENSION/REGION - List regions
//!
//! ## Rate Limits
//! - None (free tier)
//!
//! ## Data Coverage
//! - 1000+ health indicators
//! - 194 countries
//! - Historical data varies by indicator
//! - Updated quarterly
//!
//! ## Usage Restrictions
//! - Free for all uses
//! - Attribution recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{WhoEndpoint, WhoEndpoints};
pub use auth::WhoAuth;
pub use parser::{
    WhoParser, WhoIndicator, WhoDataPoint, WhoCountry, WhoRegion,
};
pub use connector::WhoConnector;
