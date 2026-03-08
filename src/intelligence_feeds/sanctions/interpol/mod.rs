//! # INTERPOL Red Notices Connector
//!
//! Category: data_feeds
//! Type: Law Enforcement Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely public)
//! - Free tier: Yes (unlimited)
//!
//! ## Data Types
//! - Price data: No
//! - Red notices: Yes (wanted persons)
//! - Yellow notices: Yes (missing persons)
//! - UN notices: Yes (UN Security Council)
//! - Person data: Yes
//!
//! ## Key Endpoints
//! - /red - Search red notices (wanted persons)
//! - /yellow - Search yellow notices (missing persons)
//! - /un - Search UN Security Council notices
//! - /red/{noticeID} - Get individual red notice details
//! - /red/{noticeID}/images - Get images for a red notice
//!
//! ## Rate Limits
//! - No documented rate limits
//!
//! ## Data Coverage
//! - International wanted persons (red notices)
//! - Missing persons (yellow notices)
//! - UN Security Council special notices
//! - Arrest warrants by country
//! - Biographical information
//!
//! ## Usage Restrictions
//! - Completely public API
//! - No authentication required
//! - Attribution recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{InterpolEndpoint, InterpolEndpoints};
pub use auth::InterpolAuth;
pub use parser::{
    InterpolParser, InterpolNotice, InterpolSearchResult,
    ArrestWarrant, InterpolImage,
};
pub use connector::InterpolConnector;
