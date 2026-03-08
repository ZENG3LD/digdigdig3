//! # OpenSky Network Connector
//!
//! Category: data_feeds
//! Type: Aviation Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Optional (Basic Auth) - anonymous access available
//! - Free tier: Yes (anonymous: 10 req/10s, authenticated: 4000 credits/day)
//!
//! ## Data Types
//! - Real-time aircraft positions (state vectors)
//! - Flight tracking data
//! - Historical flight data
//! - Flight arrivals/departures by airport
//! - Aircraft waypoint tracking
//!
//! ## Key Endpoints
//! - /states/all - Get all aircraft state vectors (CORE endpoint)
//! - /states/own - Get own sensors' state vectors (authenticated only)
//! - /flights/all - Get flights in time range
//! - /flights/aircraft - Get flights by specific aircraft
//! - /flights/arrival - Get arrivals at airport
//! - /flights/departure - Get departures from airport
//! - /tracks/all - Get flight track waypoints
//!
//! ## Rate Limits
//! - Anonymous: 10 requests per 10 seconds
//! - Authenticated: 4000 credits per day (credit cost varies by endpoint)
//!
//! ## Data Coverage
//! - Real-time global aircraft positions
//! - Historical flight data
//! - Coverage depends on ADS-B receiver network
//!
//! ## Usage Restrictions
//! - Research and non-commercial use
//! - Attribution required
//! - Respect rate limits

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OpenskyEndpoint, OpenskyEndpoints};
pub use auth::OpenskyAuth;
pub use parser::{
    OpenskyParser, StateVector, Flight, TrackPoint, OpenskyStates,
    OpenskyFlights, OpenskyTrack,
};
pub use connector::OpenskyConnector;
