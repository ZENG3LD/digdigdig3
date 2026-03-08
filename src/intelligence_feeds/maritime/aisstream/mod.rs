//! # AISStream.io Connector
//!
//! Category: data_feeds
//! Type: Maritime AIS Data Provider
//!
//! ## Features
//! - REST API: Limited (primarily WebSocket)
//! - WebSocket: Yes (primary interface)
//! - Authentication: API Key
//! - Free tier: Yes
//!
//! ## Data Types
//! - Vessel positions: Yes (real-time AIS position reports)
//! - Vessel static data: Yes (ship name, type, dimensions, etc.)
//! - Historical data: No (real-time streaming only)
//! - SAR aircraft: Yes
//! - Base station reports: Yes
//!
//! ## Key Message Types
//! - PositionReport - vessel position (MMSI, lat, lon, speed, course, heading)
//! - ShipStaticData - vessel info (MMSI, IMO, name, ship_type, dimensions)
//! - StandardSearchAndRescueAircraftReport - SAR aircraft
//! - BaseStationReport - base station info
//!
//! ## Rate Limits
//! - Free tier: Standard WebSocket connection
//! - No documented rate limits for subscription messages
//!
//! ## Data Coverage
//! - Global AIS coverage
//! - Real-time streaming data
//! - Filtering by bounding boxes, ship types, MMSIs
//!
//! ## Usage Restrictions
//! - API key required (free registration)
//! - Check AISStream.io terms of service

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{AisStreamEndpoint, AisStreamEndpoints};
pub use auth::AisStreamAuth;
pub use parser::{
    AisStreamParser, AisPosition, AisVesselStatic, AisMessage, AisMetadata,
    BoundingBox, SubscriptionMessage,
};
pub use connector::AisStreamConnector;
