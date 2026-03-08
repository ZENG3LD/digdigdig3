//! # UNHCR Population Statistics Connector
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
//! - Price data: No (refugee and humanitarian data only)
//! - Historical data: Yes (refugee population statistics)
//! - Macro data: Yes (demographic indicators)
//! - Economic indicators: No
//!
//! ## Key Endpoints
//! - /population/ - Refugee population statistics
//! - /demographics/ - Demographic breakdowns
//! - /solutions/ - Durable solutions (resettlement, returns)
//! - /asylum-decisions/ - Asylum decisions data
//! - /countries/ - Country list
//!
//! ## Rate Limits
//! - No documented rate limits
//!
//! ## Data Coverage
//! - 200+ countries and territories
//! - Historical data from 1951
//! - Refugee, asylum seeker, IDP, and stateless populations
//!
//! ## Usage Restrictions
//! - Free for all uses
//! - Attribution recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UnhcrEndpoint, UnhcrEndpoints};
pub use auth::UnhcrAuth;
pub use parser::{
    UnhcrParser, UnhcrPopulationData, UnhcrCountry,
};
pub use connector::UnhcrConnector;
