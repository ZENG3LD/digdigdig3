//! # USGS Earthquake Hazards API Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (public API)
//! - Free tier: Yes (completely free, no registration required)
//!
//! ## Data Types
//! - Price data: No (earthquake data only)
//! - Historical data: Yes (earthquakes from 1900+)
//! - Real-time data: Yes (updated every minute)
//! - Geographic data: Yes (GeoJSON format)
//!
//! ## Key Endpoints
//! - /query?format=geojson - Query earthquakes with various filters
//! - /count - Count earthquakes matching criteria
//!
//! ## Rate Limits
//! - Free tier: Generous rate limits (no strict limits documented)
//! - No paid tiers available
//!
//! ## Data Coverage
//! - Global earthquake coverage
//! - Real-time updates (every minute)
//! - Historical data from 1900+
//! - All magnitudes and depths
//!
//! ## Usage Restrictions
//! - Free public use
//! - No authentication required
//! - Attribution requested

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UsgsEarthquakeEndpoint, UsgsEarthquakeEndpoints};
pub use auth::UsgsEarthquakeAuth;
pub use parser::{
    UsgsEarthquakeParser, EarthquakeResponse, EqMetadata, EarthquakeFeature,
    EqProperties, EqGeometry,
};
pub use connector::UsgsEarthquakeConnector;
