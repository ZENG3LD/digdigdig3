//! # UK Parliament API Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None
//! - Free tier: Yes (completely free, public access)
//!
//! ## Data Types
//! - Members: MPs and Lords information
//! - Voting records: Division voting history
//! - Bills: Parliamentary bills and legislation
//! - Constituencies: UK parliamentary constituencies
//!
//! ## Key Endpoints
//! - /Members/Search - Search members by name
//! - /Members/{id} - Get member details
//! - /Members/{id}/Voting - Get member voting record
//! - /Bills - Search bills
//! - /Bills/{id} - Get bill details
//! - /Location/Constituency/Search - Search constituencies
//!
//! ## Rate Limits
//! - No explicit rate limits documented
//! - Public API, free access
//!
//! ## Data Coverage
//! - Current and historical MPs/Lords
//! - Voting records
//! - Bill tracking and status
//! - Constituency information

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UkParliamentEndpoint, UkParliamentEndpoints};
pub use auth::UkParliamentAuth;
pub use parser::{
    UkParliamentParser, UkMember, UkBill, UkBillStage, UkVote, UkConstituency,
};
pub use connector::UkParliamentConnector;
