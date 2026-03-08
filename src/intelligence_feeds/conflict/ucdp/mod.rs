//! # UCDP (Uppsala Conflict Data Program) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free, public access)
//!
//! ## Data Types
//! - Conflict events: Yes (georeferenced)
//! - Battle deaths: Yes
//! - Non-state conflicts: Yes
//! - One-sided violence: Yes
//! - State-based conflicts: Yes
//!
//! ## Key Endpoints
//! - /gedevents/24.1 - Georeferenced event data (CORE endpoint)
//! - /battledeaths/24.1 - Battle-related deaths
//! - /nonstate/24.1 - Non-state conflicts
//! - /onesided/24.1 - One-sided violence
//! - /stateconflict/24.1 - State-based conflicts
//!
//! ## Rate Limits
//! - None documented (public API)
//!
//! ## Data Coverage
//! - Global conflict data
//! - Historical depth varies by dataset
//! - Regular updates (version-based)
//!
//! ## Usage Restrictions
//! - Free, public access
//! - Attribution required (cite UCDP)

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UcdpEndpoint, UcdpEndpoints};
pub use auth::UcdpAuth;
pub use parser::{
    UcdpParser, UcdpEvent, UcdpResponse, UcdpBattleDeath,
    UcdpNonStateConflict, UcdpOneSidedViolence, UcdpStateConflict,
};
pub use connector::UcdpConnector;
