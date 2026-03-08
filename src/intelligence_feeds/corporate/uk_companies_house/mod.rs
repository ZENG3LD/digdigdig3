//! # UK Companies House API Connector
//!
//! Category: data_feeds
//! Type: Corporate Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: HTTP Basic Auth (API key as username, empty password)
//! - Free tier: Yes (600 requests per 5 minutes)
//!
//! ## Data Types
//! - Company profiles: Yes
//! - Officers (directors, secretaries): Yes
//! - Persons with Significant Control (beneficial owners): Yes
//! - Filing history: Yes
//! - Charges/mortgages: Yes
//! - Insolvency information: Yes
//! - Cross-company officer appointments: Yes
//!
//! ## Key Endpoints
//! - /search/companies - Search for companies by name
//! - /company/{number} - Get company profile
//! - /company/{number}/officers - Get company officers
//! - /company/{number}/persons-with-significant-control - Get PSC (beneficial owners)
//! - /company/{number}/filing-history - Get filing history
//! - /company/{number}/charges - Get charges/mortgages
//! - /company/{number}/insolvency - Get insolvency information
//! - /officers/{id}/appointments - Get all appointments for an officer
//!
//! ## Rate Limits
//! - Free tier: 600 requests per 5 minutes
//! - No paid tiers available
//!
//! ## Data Coverage
//! - UK company registry
//! - Company incorporation records
//! - Beneficial ownership data (PSC)
//! - Director and secretary information
//! - Corporate structure and control
//!
//! ## Usage Restrictions
//! - Free for non-commercial use
//! - Attribution required
//! - Rate limited to 600 requests per 5 minutes

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UkCompaniesHouseEndpoint, UkCompaniesHouseEndpoints};
pub use auth::UkCompaniesHouseAuth;
pub use parser::{
    UkCompaniesHouseParser, ChCompany, ChOfficer, ChPsc, ChFiling, ChSearchResult,
};
pub use connector::UkCompaniesHouseConnector;
