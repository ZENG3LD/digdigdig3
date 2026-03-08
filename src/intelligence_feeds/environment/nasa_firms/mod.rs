//! # NASA FIRMS (Fire Information for Resource Management System) Connector
//!
//! Category: data_feeds
//! Type: Environmental/Disaster Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (requires registration)
//!
//! ## Data Types
//! - Active fire data: Yes (near-real-time)
//! - Thermal anomalies: Yes (hotspots, wildfires)
//! - Historical fire data: Yes (up to 10 days)
//! - Global coverage: Yes
//! - Satellite sources: VIIRS, MODIS
//!
//! ## Key Endpoints
//! - /area - Fire data by geographic area (bounding box or world)
//! - /country - Fire data by country code
//! - Geographic filtering: Bounding box or proximity search
//!
//! ## Rate Limits
//! - Free tier: Generous limits with API key
//! - Registration required at firms.modaps.eosdis.nasa.gov
//!
//! ## Data Coverage
//! - VIIRS NOAA-20 NRT: Near-real-time (375m resolution)
//! - VIIRS S-NPP NRT: Near-real-time (375m resolution)
//! - MODIS NRT: Near-real-time (1km resolution)
//! - Historical data: Up to 10 days retention
//!
//! ## Usage Restrictions
//! - API key required (free registration)
//! - Attribution required
//! - Non-commercial use encouraged
//!
//! ## Quick Start
//! ```ignore
//! use connectors_v5::data_feeds::nasa_firms::NasaFirmsConnector;
//!
//! // Set NASA_FIRMS_KEY environment variable
//! let connector = NasaFirmsConnector::from_env();
//!
//! // Get global fires in last 24 hours
//! let fires = connector.get_global_fires_24h().await?;
//!
//! // Get fires in a specific area
//! let area_fires = connector.get_fires_by_area(
//!     "VIIRS_NOAA20_NRT",
//!     "-180,-90,180,90",
//!     1,
//!     None
//! ).await?;
//!
//! // Get fires near a location (lat, lon, radius_km, days)
//! let nearby_fires = connector.get_fires_near(37.7749, -122.4194, 100.0, 2).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{NasaFirmsEndpoint, NasaFirmsEndpoints};
pub use auth::NasaFirmsAuth;
pub use parser::{
    NasaFirmsParser, FireHotspot, FireSummary, CountryFireCount,
};
pub use connector::NasaFirmsConnector;
