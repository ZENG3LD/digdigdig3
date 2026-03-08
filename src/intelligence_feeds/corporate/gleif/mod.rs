//! # GLEIF (Global Legal Entity Identifier Foundation) Connector
//!
//! Category: data_feeds
//! Type: Legal Entity Identifiers
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely free, public API)
//! - Rate limits: 60 requests/minute
//!
//! ## Data Types
//! - Legal entity information: Yes (LEI records, names, addresses)
//! - Corporate hierarchies: Yes (parent/subsidiary relationships)
//! - Ownership structures: Yes (direct/ultimate parent chains)
//! - Search: Yes (by name, country, category)
//!
//! ## Key Endpoints
//! - GET /lei-records/{lei} - Get entity by LEI
//! - GET /lei-records?filter[entity.legalName]={name} - Search by name
//! - GET /lei-records/{lei}/direct-parent - Direct parent relationship
//! - GET /lei-records/{lei}/ultimate-parent - Ultimate parent entity
//! - GET /lei-records/{lei}/direct-children - Direct subsidiaries
//! - GET /lei-records?filter[entity.legalAddress.country]={iso2} - Filter by country
//!
//! ## Rate Limits
//! - 60 requests per minute (no authentication required)
//!
//! ## Data Coverage
//! - 2.5M+ legal entities worldwide
//! - Corporate ownership hierarchies
//! - Regulatory identifiers
//! - All asset classes (banks, funds, corporates, etc.)
//!
//! ## Usage Restrictions
//! - Free and open data
//! - No API key required
//! - Subject to rate limits

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{GleifEndpoint, GleifEndpoints};
pub use auth::GleifAuth;
pub use parser::{
    GleifParser, GleifEntity, GleifRelationship, GleifOwnershipChain,
};
pub use connector::GleifConnector;
