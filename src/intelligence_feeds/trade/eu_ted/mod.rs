//! # EU TED (Tenders Electronic Daily) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (public API, optional API key for higher limits)
//! - Free tier: Yes (fully public)
//!
//! ## Data Types
//! - Procurement notices: Yes (EU public procurement)
//! - Business entities: Yes (contracting authorities, economic operators)
//! - Codelists: Yes (CPV codes, country codes, etc.)
//!
//! ## Key Endpoints
//! - /notices/search - Search procurement notices (POST)
//! - /notices/{notice-id} - Get specific notice
//! - /business-entities/search - Search entities (POST)
//! - /business-entities/{entity-id} - Get specific entity
//! - /codelists/{codelist-id} - Get codelist values
//!
//! ## Rate Limits
//! - Public access: Standard rate limits
//! - EU Login API key: Higher limits available
//!
//! ## Data Coverage
//! - All EU public procurement notices
//! - European Economic Area (EEA) tenders
//! - Historical procurement data
//!
//! ## Usage Restrictions
//! - Free public access
//! - EU TED terms of service apply
//! - Attribution recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{EuTedEndpoint, EuTedEndpoints};
pub use auth::EuTedAuth;
pub use parser::{
    EuTedParser, TedNotice, TedEntity, TedSearchResult,
};
pub use connector::EuTedConnector;
