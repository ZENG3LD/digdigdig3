//! # NASA Open APIs Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (DEMO_KEY available)
//!
//! ## Data Types
//! - Near Earth Objects (NEOs): Asteroid tracking
//! - Solar Activity: Flares, CMEs, geomagnetic storms
//! - Space Weather: Solar energetic particles, interplanetary shocks
//! - Astronomy: Picture of the Day (APOD)
//! - Earth Imagery: EPIC natural color images
//!
//! ## Key Endpoints
//! - /neo/rest/v1/feed - Near Earth Objects feed
//! - /neo/rest/v1/neo/{id} - Specific asteroid lookup
//! - /DONKI/FLR - Solar Flares
//! - /DONKI/CME - Coronal Mass Ejections
//! - /DONKI/GST - Geomagnetic Storms
//! - /DONKI/SEP - Solar Energetic Particles
//! - /DONKI/IPS - Interplanetary Shocks
//! - /planetary/apod - Astronomy Picture of the Day
//! - /EPIC/api/natural - Earth imagery
//!
//! ## Rate Limits
//! - DEMO_KEY: 30 requests per hour
//! - API key: 1000 requests per hour

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{NasaEndpoint, NasaEndpoints};
pub use auth::NasaAuth;
pub use parser::{
    NasaParser, NeoObject, CloseApproach, SolarFlare, GeomagneticStorm, KpIndex,
    CoronalMassEjection, SolarEnergeticParticle, InterplanetaryShock, Apod, EarthImagery,
};
pub use connector::NasaConnector;
