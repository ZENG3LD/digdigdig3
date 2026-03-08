//! # Censys Search API v2 Connector
//!
//! Category: data_feeds
//! Type: Internet Security & Threat Intelligence
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: HTTP Basic Auth (API ID + API Secret)
//! - Free tier: Yes (250 queries/month)
//!
//! ## Data Types
//! - Host information: Yes
//! - Host search: Yes
//! - Host aggregation: Yes
//! - Host diff/snapshots: Yes
//! - Certificate search: Yes
//!
//! ## Key Endpoints
//! - /hosts/search - Search hosts (POST with JSON body)
//! - /hosts/{ip} - View host details
//! - /hosts/aggregate - Aggregate host data
//! - /hosts/diff - Compare host snapshots
//! - /certificates/search - Search certificates (POST)
//!
//! ## Rate Limits
//! - Free tier: 250 queries/month
//! - Rate limits vary by plan
//!
//! ## Data Coverage
//! - Global internet-connected hosts
//! - Service banners
//! - SSL/TLS certificates
//! - Host attributes
//! - Network services
//!
//! ## Usage Restrictions
//! - API credentials required (env: CENSYS_API_ID, CENSYS_API_SECRET)
//! - Rate limits based on plan

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{CensysEndpoint, CensysEndpoints};
pub use auth::CensysAuth;
pub use parser::{
    CensysParser, CensysHost, CensysSearchResult, CensysService,
    CensysLocation,
};
pub use connector::CensysConnector;
