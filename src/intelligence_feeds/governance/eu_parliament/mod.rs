//! # EU Parliament (European Parliament) Open Data Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (public API)
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - MEP data: Yes (Members of European Parliament)
//! - Plenary documents: Yes
//! - Meetings: Yes
//! - Committees: Yes
//! - Legislative activity: Yes
//!
//! ## Key Endpoints
//! - /meps - List Members of European Parliament
//! - /meps/{id} - MEP details
//! - /plenary-documents - List plenary documents
//! - /plenary-documents/{id} - Document details
//! - /meetings - List meetings
//! - /committees - List committees
//!
//! ## Rate Limits
//! - Free tier: No explicit limits (public API)
//!
//! ## Data Coverage
//! - Current and historical MEPs
//! - Plenary documents and legislative texts
//! - Meeting schedules and records
//! - Committee information
//!
//! ## Usage Restrictions
//! - Public, open data
//! - Attribution recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{EuParliamentEndpoint, EuParliamentEndpoints};
pub use auth::EuParliamentAuth;
pub use parser::{
    EuParliamentParser, EuMep, EuDocument, EuMeeting, EuCommittee,
};
pub use connector::EuParliamentConnector;
