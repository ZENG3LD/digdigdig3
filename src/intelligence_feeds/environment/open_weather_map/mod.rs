//! # OpenWeatherMap API Connector
//!
//! Category: data_feeds
//! Type: Weather Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (60 calls/min, 1M calls/month)
//!
//! ## Data Types
//! - Current weather: Yes
//! - Weather forecast: Yes (5-day)
//! - Air pollution: Yes (current + historical)
//! - Coordinates: Yes (lat/lon lookup)
//!
//! ## Key Endpoints
//! - /weather - Current weather by city or coordinates
//! - /forecast - 5-day forecast
//! - /air_pollution - Current air pollution
//! - /air_pollution/history - Historical pollution data
//!
//! ## Rate Limits
//! - Free tier: 60 requests per minute, 1M calls/month
//! - No paid tier required for basic usage
//!
//! ## Usage Restrictions
//! - API key required (free registration)
//! - Attribution required for public use

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OpenWeatherMapEndpoint, OpenWeatherMapEndpoints};
pub use auth::OpenWeatherMapAuth;
pub use parser::{
    OpenWeatherMapParser, WeatherCurrent, WeatherForecast, AirPollution,
    Coord, WeatherCondition, MainWeather, Wind, WeatherSys,
    ForecastEntry, ForecastCity, AirPollutionEntry, AirQualityIndex,
    PollutionComponents,
};
pub use connector::OpenWeatherMapConnector;
