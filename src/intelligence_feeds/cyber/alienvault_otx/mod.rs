//! # AlienVault OTX (Open Threat Exchange) Connector
//!
//! Category: data_feeds
//! Type: Threat Intelligence & Security
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header)
//! - Free tier: Yes
//!
//! ## Data Types
//! - Threat intelligence pulses: Yes
//! - IP reputation: Yes
//! - Domain reputation: Yes
//! - File hash reputation: Yes
//! - URL reputation: Yes
//! - Indicators of compromise (IOCs): Yes
//!
//! ## Key Endpoints
//! - /pulses/subscribed - Get subscribed threat pulses
//! - /pulses/activity - Recent pulse activity
//! - /indicators/IPv4/{ip}/general - IP reputation
//! - /indicators/domain/{domain}/general - Domain reputation
//! - /indicators/hostname/{hostname}/general - Hostname reputation
//! - /indicators/file/{hash}/general - File hash reputation
//! - /indicators/url/{url}/general - URL reputation
//!
//! ## Rate Limits
//! - Free tier: Available with API key
//! - Rate limits vary by plan
//!
//! ## Data Coverage
//! - Global threat intelligence
//! - Community-contributed IOCs
//! - Malware analysis
//! - Network security data
//!
//! ## Usage Restrictions
//! - API key required (env: OTX_API_KEY)
//! - Rate limits based on plan

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OtxEndpoint, OtxEndpoints};
pub use auth::OtxAuth;
pub use parser::{
    OtxParser, OtxPulse, OtxIndicator, OtxIpReputation,
};
pub use connector::OtxConnector;
