//! # ACLED (Armed Conflict Location & Event Data Project) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key + Email (query parameters)
//! - Free tier: Yes (with registration)
//!
//! ## Data Types
//! - Conflict events: Yes (primary data)
//! - Geopolitical events: Yes
//! - Actor data: Yes
//! - Fatality data: Yes
//! - Geographic data: Yes
//!
//! ## Key Endpoints
//! - /acled/read - Get events (core endpoint with query filters)
//!
//! ## Rate Limits
//! - Free tier: Rate limits apply based on account type
//!
//! ## Data Coverage
//! - Global conflict and event data
//! - Real-time and historical data
//! - Multiple event types: Battles, Explosions, Violence, Protests, Riots, etc.
//!
//! ## Usage Restrictions
//! - Terms must be accepted for each request
//! - Attribution required
//! - Check ACLED terms of use for specific restrictions

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{AcledEndpoint, AcledEndpoints};
pub use auth::AcledAuth;
pub use parser::{AcledParser, AcledEvent, AcledResponse};
pub use connector::AcledConnector;
