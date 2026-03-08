//! # OpenSanctions Connector
//!
//! Category: data_feeds
//! Type: Sanctions & PEP Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header, optional)
//! - Free tier: Yes (rate limited)
//!
//! ## Data Types
//! - Price data: No
//! - Sanctions data: Yes (primary use case)
//! - PEP data: Yes (Politically Exposed Persons)
//! - Entity data: Yes (Person, Company, Organization)
//!
//! ## Key Endpoints
//! - /search/default - Search entities
//! - /entities/{id} - Get entity details
//! - /match/default - Match entity (POST)
//! - /datasets - List datasets
//! - /collections - List collections
//!
//! ## Rate Limits
//! - Free tier: Rate limited
//! - Authentication recommended for better limits
//!
//! ## Data Coverage
//! - Sanctions lists from multiple sources
//! - PEP databases
//! - Watchlist entities
//! - Entity relationships
//!
//! ## Usage Restrictions
//! - API key recommended
//! - Follow FollowTheMoney entity format
//! - Attribution required

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OpenSanctionsEndpoint, OpenSanctionsEndpoints};
pub use auth::OpenSanctionsAuth;
pub use parser::{
    OpenSanctionsParser, SanctionEntity, SanctionSearchResult,
    SanctionDataset, SanctionCollection,
};
pub use connector::OpenSanctionsConnector;
