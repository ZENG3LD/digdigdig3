//! # ADS-B Exchange (Unfiltered Flight Tracking) Connector
//!
//! Category: data_feeds
//! Type: Real-time Flight Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: RapidAPI Key (headers)
//! - Pricing: $10/month on RapidAPI
//!
//! ## Data Types
//! - Real-time aircraft positions
//! - Aircraft metadata (type, registration, callsign)
//! - Military aircraft tracking (UNFILTERED)
//! - Emergency alerts (squawk codes)
//! - LADD (Limited Aircraft Data Display)
//!
//! ## Key Features
//! - **UNFILTERED**: Includes military aircraft (mil flag)
//! - **REAL-TIME**: Live aircraft positions
//! - **COMPREHENSIVE**: All squawk codes including emergencies
//! - **GLOBAL**: Worldwide coverage
//!
//! ## Key Endpoints
//! - /v2/lat/{lat}/lon/{lon}/dist/{dist}/ - Aircraft near location
//! - /v2/hex/{icao}/ - Aircraft by ICAO hex code
//! - /v2/callsign/{callsign}/ - Aircraft by callsign
//! - /v2/registration/{reg}/ - Aircraft by registration
//! - /v2/type/{type}/ - Aircraft by type (e.g., B738, F16)
//! - /v2/mil/ - ALL military aircraft currently airborne
//! - /v2/sqk/{squawk}/ - Aircraft by squawk code
//! - /v2/ladd/ - LADD aircraft (military/sensitive)
//!
//! ## Squawk Codes
//! - 7500 - Hijack
//! - 7600 - Radio failure
//! - 7700 - Emergency
//!
//! ## Rate Limits
//! - RapidAPI limits apply (varies by plan)
//!
//! ## Authentication
//! - Requires RapidAPI key in headers
//! - X-RapidAPI-Key: your_key
//! - X-RapidAPI-Host: adsbexchange-com1.p.rapidapi.com

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{AdsbExchangeEndpoint, AdsbExchangeEndpoints};
pub use auth::AdsbExchangeAuth;
pub use parser::{AdsbExchangeParser, AdsbAircraft, AdsbResponse};
pub use connector::AdsbExchangeConnector;
