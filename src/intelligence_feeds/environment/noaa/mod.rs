//! # NOAA Climate Data Online (CDO) Connector
//!
//! Category: data_feeds
//! Type: Climate Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header token)
//! - Free tier: Yes (with rate limits)
//!
//! ## Data Types
//! - Climate data: Yes (temperature, precipitation, etc.)
//! - Historical data: Yes (varies by dataset, some from 1700s)
//! - Weather stations: Yes (global coverage)
//! - Datasets: GHCND, GSOM, GSOY, Climate Normals
//!
//! ## Key Endpoints
//! - /data - Get climate observations (CORE endpoint)
//! - /datasets - Browse available datasets
//! - /datatypes - Browse data types (TMAX, TMIN, PRCP, etc.)
//! - /locations - Browse geographic locations
//! - /stations - Browse weather stations
//!
//! ## Rate Limits
//! - 5 requests per second
//! - 10,000 requests per day
//!
//! ## Data Coverage
//! - Global Historical Climatology Network - Daily (GHCND)
//! - Global Summary of Month/Year (GSOM/GSOY)
//! - Climate Normals (Daily/Monthly)
//! - Precipitation data
//! - Temperature data (max/min/avg)
//! - Snow and ice data
//!
//! ## Usage Restrictions
//! - API key required (free registration)
//! - Rate limits enforced
//! - Attribution recommended
//!
//! ## Quick Start
//! ```ignore
//! use connectors_v5::data_feeds::noaa::NoaaConnector;
//!
//! // Set NOAA_API_KEY environment variable
//! let connector = NoaaConnector::from_env();
//!
//! // Get temperature data
//! let temps = connector.get_temperature(
//!     "FIPS:37",
//!     "2024-01-01",
//!     "2024-01-31"
//! ).await?;
//!
//! // Get precipitation data
//! let precip = connector.get_precipitation(
//!     "CITY:US370007",
//!     "2024-01-01",
//!     "2024-12-31"
//! ).await?;
//!
//! // List available datasets
//! let datasets = connector.list_datasets(None, None).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{NoaaEndpoint, NoaaEndpoints};
pub use auth::NoaaAuth;
pub use parser::{
    NoaaParser, ClimateData, Dataset, Datatype, LocationCategory, Location, Station,
};
pub use connector::NoaaConnector;
