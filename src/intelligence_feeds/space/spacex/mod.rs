//! # SpaceX Data Connector
//!
//! Category: data_feeds
//! Type: Space Launch Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely public)
//! - Free tier: Yes (no rate limits documented)
//!
//! ## Data Types
//! - Launch data (past, upcoming, latest, next)
//! - Rocket information
//! - Crew members
//! - Starlink satellites
//! - Launch pads and landing pads
//! - Payloads
//!
//! ## Key Endpoints
//! - /launches/latest - Latest launch
//! - /launches/next - Next upcoming launch
//! - /launches/upcoming - All upcoming launches
//! - /launches/past - All past launches
//! - /rockets - All rockets
//! - /crew - All crew members
//! - /starlink - All Starlink satellites
//!
//! ## Rate Limits
//! - No documented rate limits
//! - Public API with free access
//!
//! ## Data Coverage
//! - Complete SpaceX launch history
//! - Future scheduled launches
//! - Real-time Starlink satellite data
//! - Rocket specifications
//!
//! ## Usage Restrictions
//! - Open source API
//! - No authentication required
//! - Free for all use

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{SpaceXEndpoint, SpaceXEndpoints};
pub use auth::SpaceXAuth;
pub use parser::{
    SpaceXParser, SpaceXLaunch, SpaceXRocket, SpaceXCrew, SpaceXStarlink,
};
pub use connector::SpaceXConnector;
