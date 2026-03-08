//! # AviationStack Connector
//!
//! Category: data_feeds
//! Type: Aviation Data Provider
//!
//! ## Features
//! - REST API: Yes (HTTP only for free tier)
//! - WebSocket: No
//! - Authentication: API key via query parameter
//! - Free tier: Yes (100 requests/month)
//!
//! ## Data Types
//! - Real-time flight data
//! - Airport database
//! - Airline database
//! - Aircraft types
//! - Cities database
//! - Countries database
//! - Flight routes
//!
//! ## Key Endpoints
//! - /flights - Real-time flight data
//! - /airports - Airport database
//! - /airlines - Airline database
//! - /aircraft_types - Aircraft types
//! - /cities - Cities database
//! - /countries - Countries database
//! - /routes - Flight routes
//!
//! ## Rate Limits
//! - Free tier: 100 requests per month
//!
//! ## Data Coverage
//! - Real-time global flight data
//! - Comprehensive airport and airline databases
//!
//! ## Usage Restrictions
//! - Free tier is HTTP only (not HTTPS)
//! - Limited monthly requests

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{AviationStackEndpoint, AviationStackEndpoints};
pub use auth::AviationStackAuth;
pub use parser::{
    AviationStackParser, AvFlight, AvAirport, AvAirline, AvFlightInfo, AvRoute,
};
pub use connector::AviationStackConnector;
