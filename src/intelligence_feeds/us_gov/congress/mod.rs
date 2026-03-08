//! # Congress.gov API Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (5000 requests/hour)
//!
//! ## Data Types
//! - Legislative data: Yes (bills, actions, cosponsors)
//! - Member data: Yes (congress members, bioguides)
//! - Committee data: Yes
//! - Nomination data: Yes
//! - Treaty data: Yes
//! - Congress data: Yes (historical congresses)
//!
//! ## Key Endpoints
//! - /bill - List bills
//! - /bill/{congress}/{type}/{number} - Get specific bill
//! - /bill/{congress}/{type}/{number}/actions - Bill actions
//! - /bill/{congress}/{type}/{number}/cosponsors - Bill cosponsors
//! - /member - List members
//! - /member/{bioguideId} - Get specific member
//! - /committee - List committees
//! - /nomination - List nominations
//! - /treaty - List treaties
//! - /congress - List congresses
//! - /summaries - Bill summaries
//!
//! ## Rate Limits
//! - Free tier: 5000 requests per hour
//! - No paid tiers available
//!
//! ## Data Coverage
//! - Bills from 93rd Congress (1973) to present
//! - Members from all historical congresses
//! - Committees (current and historical)
//! - Nominations and treaties
//!
//! ## Usage Restrictions
//! - API key required (free registration)
//! - Rate limits enforced
//! - Attribution encouraged

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{CongressEndpoint, CongressEndpoints};
pub use auth::CongressAuth;
pub use parser::{
    CongressParser, Bill, BillAction, Member, Committee, CongressInfo,
    Nomination, BillCosponsor, BillSummary,
};
pub use connector::CongressConnector;
