//! # NASA EONET (Earth Observatory Natural Event Tracker)
//!
//! Category: data_feeds/environment
//! Type: Natural Disaster & Environmental Event Data
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required (public API)
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Natural events: Yes (wildfires, storms, volcanoes, earthquakes, floods, etc.)
//! - Event metadata: Yes (locations, magnitudes, sources, timestamps)
//! - Geographic data: Yes (coordinates, bounding boxes)
//! - Historical data: Yes (closed events)
//!
//! ## Key Endpoints
//! - /events - Query events with filtering
//! - /categories - List event categories
//! - /sources - List event sources
//!
//! ## Rate Limits
//! - Enforced with automatic 1-hour block when exceeded
//! - Headers: X-RateLimit-Limit, X-RateLimit-Remaining
//! - Optional NASA API key for intensive use (not implemented)
//!
//! ## Data Coverage
//! - 13 event categories
//! - 33+ authoritative sources (NASA, NOAA, USGS, etc.)
//! - Global coverage
//! - Real-time updates
//!
//! ## Event Categories
//! - Wildfires
//! - Severe Storms (hurricanes, cyclones, tornadoes)
//! - Volcanoes
//! - Earthquakes
//! - Floods
//! - Drought
//! - Landslides
//! - Dust and Haze
//! - Snow
//! - Temperature Extremes
//! - Sea and Lake Ice (icebergs)
//! - Water Color (algae, phytoplankton)
//! - Manmade Events
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Public data
//! - Rate limiting enforced

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{NasaEonetEndpoint, NasaEonetEndpoints};
pub use auth::NasaEonetAuth;
pub use parser::{
    NasaEonetParser, NaturalEvent, EventCategory, EventSource, EventGeometry,
};
pub use connector::NasaEonetConnector;
