//! # GDACS (Global Disaster Alert and Coordination System) Connector
//!
//! Category: data_feeds/environment
//! Type: Disaster Monitoring Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Disaster events: Yes (earthquakes, cyclones, floods, volcanoes, wildfires, droughts, tsunamis)
//! - Alert levels: Yes (Green, Orange, Red)
//! - Impact data: Yes (population exposure, severity metrics, casualty estimates)
//! - Geographic data: Yes (coordinates, affected countries)
//!
//! ## Key Endpoints
//! - /events/geteventlist/SEARCH - Search and filter disaster events
//! - /events/geteventdata/GetByEventId - Get specific event by ID
//!
//! ## Rate Limits
//! - No documented rate limits
//! - Recommended: Poll every 5-6 minutes (RSS feeds update every 6 minutes)
//!
//! ## Data Coverage
//! - Global coverage: All countries and regions
//! - 24/7 monitoring: Real-time for automated events (EQ, TC, TS)
//! - Historical data: Available with pagination
//! - Update frequency: Varies by disaster type (seconds to days)
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Attribution required: "Data provided by GDACS (gdacs.org)"
//! - Disclaimer: Data is indicative only, verify with alternate sources
//!
//! ## Data Sources
//! - Earthquakes: USGS NEIC
//! - Tropical Cyclones: JTWC, NHC, IMD, regional agencies
//! - Floods: GLOFAS (Global Flood Awareness System)
//! - Wildfires: GWIS (Global Wildfire Information System)
//! - Volcanoes: VAAC (DARWIN, TOKYO)
//! - Droughts: GDO (Global Drought Observatory)
//! - Tsunamis: PTWC, JMA, regional centers

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{GdacsEndpoint, GdacsEndpoints};
pub use auth::GdacsAuth;
pub use parser::{
    GdacsParser, DisasterEvent, DisasterType, AlertLevel,
};
pub use connector::GdacsConnector;
