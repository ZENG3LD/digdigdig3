//! # UN Population Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (public API)
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Price data: No (demographic data only)
//! - Historical data: Yes (population statistics from 1950-2100)
//! - Macro data: Yes (demographic indicators)
//! - Economic indicators: Some (fertility rate, mortality, etc.)
//!
//! ## Key Endpoints
//! - /locations - Get countries and regions
//! - /locations/{id}/indicators/{indicatorId} - Get indicator data
//! - /indicators - List available indicators
//!
//! ## Rate Limits
//! - No documented rate limits
//!
//! ## Data Coverage
//! - 237 countries and territories
//! - Historical data: 1950-2023
//! - Projections: 2024-2100
//! - 40+ demographic indicators
//!
//! ## Usage Restrictions
//! - Free for all uses
//! - Attribution recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UnPopEndpoint, UnPopEndpoints};
pub use auth::UnPopAuth;
pub use parser::{
    UnPopParser, UnPopLocation, UnPopIndicator, UnPopDataPoint, UnPopResponse,
};
pub use connector::UnPopConnector;
