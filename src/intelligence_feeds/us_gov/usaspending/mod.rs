//! # USASpending.gov Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely public API)
//! - Free tier: Yes (completely free, no rate limits mentioned)
//!
//! ## Data Types
//! - Federal spending data: Yes
//! - Award data: Yes (contracts, grants, loans, etc.)
//! - Agency data: Yes
//! - State spending: Yes
//! - Recipient data: Yes
//!
//! ## Key Endpoints
//! - /spending/explorer/ - Spending explorer (POST)
//! - /search/spending_by_award/ - Award spending search (POST)
//! - /references/agency/ - Federal agencies list
//! - /bulk_download/awards/ - Bulk download awards (POST)
//! - /spending/state/ - State spending data
//! - /awards/count/federal_account/ - Federal account award counts
//!
//! ## Rate Limits
//! - None explicitly documented (reasonable use expected)
//!
//! ## Data Coverage
//! - Federal spending and award data
//! - Historical depth varies by data type
//! - Real-time updates

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UsaSpendingEndpoint, UsaSpendingEndpoints};
pub use auth::UsaSpendingAuth;
pub use parser::{
    UsaSpendingParser, UsaSpendingAward, UsaSpendingAgency, UsaSpendingState,
};
pub use connector::UsaSpendingConnector;
