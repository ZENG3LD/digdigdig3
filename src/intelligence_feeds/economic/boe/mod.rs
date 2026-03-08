//! # Bank of England (BoE) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free public data)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes (economic time series from Bank of England)
//! - Macro data: Yes (monetary policy, GDP, inflation, etc.)
//! - Economic indicators: Yes
//!
//! ## Key Endpoints
//! - /_iadb-fromshowcolumns.asp - Get time series data (CORE endpoint)
//! - /fromshowcolumns.asp - Get series info
//!
//! ## Rate Limits
//! - No explicit rate limits
//! - Max 300 series codes per request
//!
//! ## Data Coverage
//! - Bank Rate, CPI, GDP, M4, employment, exchange rates, bond yields
//! - Historical depth varies by series
//! - Some series from 1960s+, others more recent
//!
//! ## Usage Restrictions
//! - Public data, free to use
//! - No authentication required

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{BoeEndpoint, BoeEndpoints};
pub use auth::BoeAuth;
pub use parser::{BoeParser, BoeObservation, BoeSeriesInfo};
pub use connector::BoeConnector;
