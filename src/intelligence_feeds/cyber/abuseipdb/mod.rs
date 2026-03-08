//! # AbuseIPDB Connector
//!
//! Category: data_feeds
//! Type: IP Reputation & Abuse Reporting
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header)
//! - Free tier: Yes (1000 daily checks)
//!
//! ## Data Types
//! - IP reputation checks: Yes
//! - IP blacklist: Yes
//! - Network block checks: Yes (CIDR)
//! - Abuse reporting: Yes
//! - Abuse categories: 23 categories
//!
//! ## Key Endpoints
//! - /check - Check IP address for abuse reports
//! - /blacklist - Get blacklist of malicious IPs
//! - /check-block - Check network block (CIDR)
//! - /report - Report IP address for abuse
//! - /categories - Get abuse category list
//!
//! ## Rate Limits
//! - Free tier: 1000 checks/day
//! - Basic: 3000 checks/day
//! - Premium: 100,000 checks/day
//!
//! ## Data Coverage
//! - Global IP reputation database
//! - Community-contributed abuse reports
//! - Confidence scoring (0-100)
//! - Historical data (30+ days)
//!
//! ## Usage Restrictions
//! - API key required (env: ABUSEIPDB_API_KEY)
//! - Rate limits based on plan
//! - Blacklist download requires premium plan

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{AbuseIpdbEndpoint, AbuseIpdbEndpoints};
pub use auth::AbuseIpdbAuth;
pub use parser::{
    AbuseIpdbParser, AbuseIpReport, BlacklistEntry, CheckBlockReport,
    BlockReportedAddress, AbuseCategory,
};
pub use connector::AbuseIpdbConnector;
