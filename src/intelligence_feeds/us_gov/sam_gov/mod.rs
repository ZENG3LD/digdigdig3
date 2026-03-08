//! # SAM.gov (System for Award Management) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (free registration required)
//!
//! ## Data Types
//! - Entity information: Yes (government contractors)
//! - Contract opportunities: Yes (federal procurement)
//! - NAICS codes: Yes (industry classification)
//! - Registration data: Yes (SAM registration)
//!
//! ## Key Endpoints
//! - /entity-information/v3/entities - Search entities
//! - /opportunities/v2/search - Contract opportunities search
//!
//! ## Rate Limits
//! - Free tier: Rate limits vary by API key tier
//! - Premium tiers available
//!
//! ## Data Coverage
//! - All federal contractors registered in SAM
//! - All federal contract opportunities
//! - Historical registration data
//!
//! ## Usage Restrictions
//! - Free tier available with registration at api.data.gov
//! - Attribution recommended
//! - No unauthorized commercial use

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{SamGovEndpoint, SamGovEndpoints};
pub use auth::SamGovAuth;
pub use parser::{
    SamGovParser, SamEntity, SamOpportunity, SamAddress,
};
pub use connector::SamGovConnector;
