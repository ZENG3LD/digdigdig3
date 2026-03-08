//! # NVD (National Vulnerability Database) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header-based, optional)
//! - Free tier: Yes (public API with rate limits)
//!
//! ## Data Types
//! - CVE data: Yes (Common Vulnerabilities and Exposures)
//! - CPE data: Yes (Common Platform Enumerations)
//! - CVSS scores: Yes (v2, v3, v3.1)
//! - Weakness data: Yes (CWE references)
//!
//! ## Key Endpoints
//! - /cves/2.0 - Search CVEs
//! - /cves/2.0?cveId={id} - Get specific CVE
//! - /cpes/2.0 - Search CPEs
//! - /cpematch/2.0 - CPE match strings
//!
//! ## Rate Limits
//! - Without API key: 5 requests per 30 seconds
//! - With API key: 50 requests per 30 seconds
//!
//! ## Data Coverage
//! - 200,000+ CVEs dating back to 1999
//! - Updated continuously
//! - NIST official data source
//!
//! ## Usage Restrictions
//! - Public data, no commercial restrictions
//! - API key recommended for higher rate limits
//! - Attribution to NIST encouraged

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{NvdEndpoint, NvdEndpoints};
pub use auth::NvdAuth;
pub use parser::{NvdParser, NvdCve, NvdSearchResult};
pub use connector::NvdConnector;
