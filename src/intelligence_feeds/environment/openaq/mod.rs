//! # OpenAQ (Open Air Quality) Connector
//!
//! Category: data_feeds
//! Type: Environmental Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Optional API Key (header)
//! - Free tier: Yes (generous rate limits)
//!
//! ## Data Types
//! - Air quality measurements: Yes (PM2.5, PM10, O3, NO2, SO2, CO)
//! - Historical data: Yes
//! - Real-time data: Yes (latest readings)
//! - Location data: Yes (monitoring stations worldwide)
//! - Geographic data: Yes (coordinates, cities, countries)
//!
//! ## Key Endpoints
//! - /locations - Get monitoring locations
//! - /measurements - Get air quality measurements
//! - /latest - Latest measurements from all locations
//! - /countries - List countries with data
//! - /cities - List cities with data
//! - /parameters - List measured parameters
//! - /averages - Get averaged data
//!
//! ## Rate Limits
//! - Free tier: Generous limits
//! - API key: Optional for better rate limits
//!
//! ## Data Coverage
//! - 10,000+ monitoring locations worldwide
//! - 200+ countries and territories
//! - Real-time updates from government agencies and research organizations
//! - Historical data varies by location
//!
//! ## Usage Restrictions
//! - Open data (free to use)
//! - Attribution appreciated
//! - No commercial restrictions

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OpenAqEndpoint, OpenAqEndpoints};
pub use auth::OpenAqAuth;
pub use parser::{
    OpenAqParser, OpenAqLocation, OpenAqMeasurement, DateInfo, OpenAqCountry,
    OpenAqCity, OpenAqParameter, OpenAqLatest, LatestMeasurement,
};
pub use connector::OpenAqConnector;
