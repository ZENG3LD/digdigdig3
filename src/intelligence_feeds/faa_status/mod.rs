//! # FAA Airport Status (NASSTATUS) Connector
//!
//! Category: data_feeds/aviation
//! Type: Government Aviation Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Airport closures: Yes
//! - Ground stops: Yes
//! - Ground delay programs: Yes
//! - Arrival/departure delays: Yes
//! - Airspace flow programs: Yes
//! - Weather conditions: Limited (in delay reasons)
//!
//! ## Key Endpoints
//! - /api/airport-status-information - Get all current delays
//!
//! ## Rate Limits
//! - Not documented
//! - Recommended: Poll every 30-60 seconds
//! - Cache with 60s TTL
//!
//! ## Data Coverage
//! - Major US airports (IATA/ICAO codes)
//! - Large and medium hub airports
//! - Limited coverage of smaller regional airports
//! - Real-time updates (typically < 5 minutes)
//!
//! ## Usage Restrictions
//! - Public API (government-operated)
//! - No authentication required
//! - Reasonable use expected
//!
//! ## Example
//! ```ignore
//! use connectors_v5::data_feeds::faa_status::{FaaStatusConnector, DelaySeverity};
//!
//! let connector = FaaStatusConnector::new();
//!
//! // Get all delays
//! let status = connector.get_all_delays().await?;
//! println!("Total delays: {}", status.count);
//!
//! // Get severe delays only
//! let severe = connector.get_delays_by_severity(DelaySeverity::Major).await?;
//!
//! // Check specific airport
//! let ord_delays = connector.get_airport_delays("ORD").await?;
//!
//! // Get ground stops
//! let stops = connector.get_ground_stops().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{FaaStatusEndpoint, FaaStatusEndpoints};
pub use auth::FaaStatusAuth;
pub use parser::{
    FaaStatusParser, AirportDelay, AirportStatus, DelayType, DelaySeverity,
};
pub use connector::FaaStatusConnector;
