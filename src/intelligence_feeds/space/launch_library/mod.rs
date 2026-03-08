//! # Launch Library 2 Connector
//!
//! Category: data_feeds
//! Type: Space Launch Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (free public API)
//! - Free tier: Yes (15 requests/hour)
//!
//! ## Data Types
//! - Launch data: Yes (upcoming & previous launches)
//! - Event data: Yes (landings, dockings, etc.)
//! - Space agency info: Yes
//! - Astronaut data: Yes
//! - Rocket/spacecraft info: Yes
//! - Space station data: Yes
//!
//! ## Key Endpoints
//! - /launch/upcoming/ - Get upcoming launches
//! - /launch/previous/ - Get previous launches
//! - /launch/{id}/ - Get launch details
//! - /event/upcoming/ - Get upcoming events
//! - /astronaut/ - Get astronaut data
//! - /agency/ - Get space agency info
//!
//! ## Rate Limits
//! - Free tier: 15 requests per hour
//! - Dev tier (Patreon): 300 requests per day
//!
//! ## Data Coverage
//! - Global space launch data
//! - Historical launch records
//! - Future launch schedules
//! - Real-time updates
//!
//! ## Usage Restrictions
//! - Free for non-commercial use
//! - Attribution required
//! - Dev tier recommended for production

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{LaunchLibraryEndpoint, LaunchLibraryEndpoints};
pub use auth::LaunchLibraryAuth;
pub use parser::{
    LaunchLibraryParser, SpaceLaunch, LaunchStatus, SpaceMission, SpaceOrbit,
    SpacePad, SpaceLocation, SpaceRocket, RocketConfig, SpaceAgency,
    SpaceEvent, SpaceAstronaut, PaginatedResponse,
};
pub use connector::LaunchLibraryConnector;
