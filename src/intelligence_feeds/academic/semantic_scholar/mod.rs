//! # Semantic Scholar (Academic Research API) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header, optional)
//! - Free tier: Yes (100 requests/5 min without key, 1 req/sec with key)
//!
//! ## Data Types
//! - Academic papers with citations, authors, and metadata
//! - Author information with h-index and paper counts
//! - Citation networks and reference graphs
//! - Full-text search over 200M+ papers
//!
//! ## Key Endpoints
//! - /paper/search - Search papers by query
//! - /paper/{id} - Get paper details
//! - /paper/{id}/citations - Paper citations
//! - /paper/{id}/references - Paper references
//! - /author/search - Search authors
//! - /author/{id} - Get author details
//! - /author/{id}/papers - Author's papers
//!
//! ## Rate Limits
//! - Free tier: 100 requests per 5 minutes (no API key)
//! - With API key: 1 request per second
//!
//! ## Usage Restrictions
//! - Free for academic and commercial use
//! - No redistribution of bulk data
//! - Attribution recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{SemanticScholarEndpoint, SemanticScholarEndpoints};
pub use auth::SemanticScholarAuth;
pub use parser::{
    SemanticScholarParser, ScholarPaper, ScholarAuthor, ScholarSearchResult, ScholarCitation,
};
pub use connector::SemanticScholarConnector;
