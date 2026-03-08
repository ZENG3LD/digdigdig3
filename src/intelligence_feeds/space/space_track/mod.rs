//! # Space-Track.org Connector
//!
//! Category: data_feeds
//! Type: Space Situational Awareness Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Username/Password with session cookies
//! - Free tier: Yes (requires free registration)
//!
//! ## Data Types
//! - Satellite tracking data (TLE - Two-Line Elements)
//! - Satellite catalog information
//! - Orbital decay predictions
//! - Space debris tracking
//! - Launch site information
//! - Tracking & Impact Predictions (TIP)
//!
//! ## Key Endpoints
//! - /basicspacedata/query/class/gp/NORAD_CAT_ID/{id}/format/json - TLE data for specific satellite
//! - /basicspacedata/query/class/satcat/orderby/LAUNCH desc/limit/25/format/json - Recent satellite launches
//! - /basicspacedata/query/class/decay/orderby/DECAY_EPOCH desc/limit/25/format/json - Decay predictions
//! - /basicspacedata/query/class/gp/OBJECT_TYPE/DEBRIS/orderby/LAUNCH desc/limit/50/format/json - Space debris
//! - /basicspacedata/query/class/launch_site/format/json - Launch sites
//! - /basicspacedata/query/class/tip/format/json - Tracking & Impact Predictions
//!
//! ## Rate Limits
//! - Free tier: 30 requests per minute, 300 requests per hour
//! - No paid tiers available
//!
//! ## Data Coverage
//! - 50,000+ tracked objects in orbit
//! - Historical TLE data
//! - Decay predictions for deorbiting objects
//! - Comprehensive satellite catalog
//!
//! ## Usage Restrictions
//! - Free registration required at space-track.org
//! - Data is public but requires authentication
//! - Attribution requested for published materials
//! - No commercial restrictions for most data

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{SpaceTrackEndpoint, SpaceTrackEndpoints};
pub use auth::SpaceTrackAuth;
pub use parser::{
    SpaceTrackParser, Satellite, DecayPrediction, TleData,
};
pub use connector::SpaceTrackConnector;
