//! # OpenCorporates Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Token (query parameter)
//! - Free tier: Yes (limited requests without token)
//!
//! ## Data Types
//! - Company data: Yes (primary data)
//! - Officer/Director data: Yes
//! - Corporate filings: Yes
//! - Corporate groupings: Yes
//! - Jurisdiction data: Yes
//!
//! ## Key Endpoints
//! - /companies/search - Search companies
//! - /companies/{jurisdiction}/{number} - Get specific company
//! - /officers/search - Search officers/directors
//! - /companies/{jurisdiction}/{number}/officers - Company officers
//! - /companies/{jurisdiction}/{number}/filings - Company filings
//!
//! ## Rate Limits
//! - Free tier: Limited requests per month
//! - Paid tier: Higher rate limits with API token
//!
//! ## Data Coverage
//! - 200M+ companies globally
//! - 140+ jurisdictions
//! - Real-time and historical data

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OpenCorporatesEndpoint, OpenCorporatesEndpoints};
pub use auth::OpenCorporatesAuth;
pub use parser::{OcCompany, OcOfficer, OcCompanyRef, OcFiling, OcSearchResult};
pub use connector::OpenCorporatesConnector;
