//! # AIS (Automatic Identification System) Connector
//!
//! Category: data_feeds
//! Type: Vessel Tracking Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (100 API credits/month)
//!
//! ## Data Types
//! - Vessel tracking: Yes
//! - Port information: Yes
//! - Fleet positions: Yes
//! - Historical vessel tracks: Yes
//!
//! ## Key Endpoints
//! - /vessel_find - Search vessels by name, MMSI, IMO, callsign
//! - /vessel_info - Get vessel details by UUID
//! - /vessel_history - Get vessel position history
//! - /port_find - Search ports
//! - /port_info - Get port details
//! - /fleet_live_map - Get live fleet positions in area
//!
//! ## Rate Limits
//! - Free tier: 100 API credits per month
//!
//! ## Data Coverage
//! - Global vessel coverage
//! - Real-time AIS data
//! - Historical position data
//!
//! ## Usage Restrictions
//! - Subject to Datalastic API terms
//! - Free tier for non-commercial use

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{AisEndpoint, AisEndpoints};
pub use auth::AisAuth;
pub use parser::{
    AisParser, AisVessel, AisPort, AisPosition,
};
pub use connector::AisConnector;
