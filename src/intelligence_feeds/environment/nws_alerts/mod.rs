//! # NWS Weather Alerts (National Weather Service) Connector
//!
//! Category: data_feeds/environment
//! Type: Weather Alert Distribution
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required (User-Agent header only)
//! - Free tier: Yes (completely free government service)
//!
//! ## Data Types
//! - Weather alerts: Yes (watches, warnings, advisories, statements)
//! - Geographic filtering: Yes (by state, zone, point)
//! - Severity levels: Extreme, Severe, Moderate, Minor
//! - Alert metadata: Onset, expires, area description, instructions
//!
//! ## Key Endpoints
//! - /alerts/active - All active alerts nationwide
//! - /alerts/active/area/{area} - Alerts by state/territory
//! - /alerts/active/zone/{zone} - Alerts by NWS forecast zone
//! - /alerts/{id} - Specific alert by ID
//!
//! ## Rate Limits
//! - Recommended: 1 request per 30 seconds
//! - Enforcement: Rate-limiting firewalls protect against abuse
//!
//! ## Data Coverage
//! - Geographic: Continental US, Alaska, Hawaii, territories
//! - Marine: Atlantic, Pacific, Gulf of Mexico regions
//! - Historical: Active alerts plus 7-day archive
//! - Update frequency: Real-time as alerts are issued
//!
//! ## Usage Restrictions
//! - Free for all use
//! - User-Agent header required for application identification
//! - Respect rate limits
//!
//! ## Alert Types
//! Common event types include:
//! - Tornado Warning/Watch
//! - Severe Thunderstorm Warning/Watch
//! - Winter Weather Advisory
//! - Flash Flood Warning
//! - Hurricane Warning
//! - Heat Advisory
//! - And 125+ other official NWS alert types
//!
//! ## Data Format
//! Returns GeoJSON FeatureCollection with CAP v1.2 compliant properties.

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{NwsAlertsEndpoint, NwsAlertsEndpoints};
pub use auth::NwsAlertsAuth;
pub use parser::{
    NwsAlertsParser, WeatherAlert, Severity, Certainty, Urgency,
};
pub use connector::NwsAlertsConnector;
