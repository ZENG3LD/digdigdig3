//! # FBI Crime Data API (Crime Data Explorer) Connector
//!
//! Category: data_feeds
//! Type: Data Provider (US Government)
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (public API)
//!
//! ## Data Types
//! - Crime statistics: Yes (national and state-level)
//! - Agency data: Yes (FBI-participating agencies)
//! - NIBRS data: Yes (offense, victim, offender counts)
//! - Participation rates: Yes (agency reporting participation)
//!
//! ## Key Endpoints
//! - /api/estimates/national - National crime estimates by year
//! - /api/estimates/states/{state_abbr} - State-level estimates
//! - /api/summarized/state/{state_abbr}/{offense} - Offense summaries
//! - /api/participation/national - National participation rates
//! - /api/agencies - List of reporting agencies
//! - /api/nibrs/{offense}/offender/states/{state}/count - NIBRS offender data
//! - /api/nibrs/{offense}/victim/states/{state}/count - NIBRS victim data
//!
//! ## Rate Limits
//! - No documented rate limits
//!
//! ## Data Coverage
//! - US crime statistics from FBI Uniform Crime Reporting (UCR) Program
//! - National Incident-Based Reporting System (NIBRS)
//! - Historical data coverage varies by jurisdiction
//!
//! ## Usage Restrictions
//! - Public domain data
//! - API key required (free from api.data.gov)

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{FbiCrimeEndpoint, FbiCrimeEndpoints};
pub use auth::FbiCrimeAuth;
pub use parser::{FbiCrimeParser, CrimeEstimate, CrimeAgency, NibrsData};
pub use connector::FbiCrimeConnector;
