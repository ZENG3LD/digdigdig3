//! # VirusTotal API v3 Connector
//!
//! Category: data_feeds
//! Type: Security & Threat Intelligence
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header: x-apikey)
//! - Free tier: Yes (4 req/min, 500/day, 15.5K/month)
//!
//! ## Data Types
//! - File reports: Yes (by hash)
//! - URL scan reports: Yes
//! - Domain reports: Yes
//! - IP address reports: Yes
//! - Search functionality: Yes
//!
//! ## Key Endpoints
//! - /files/{id} - File report by hash (MD5/SHA1/SHA256)
//! - /urls/{id} - URL scan report (id = base64url of URL)
//! - /domains/{domain} - Domain report
//! - /ip_addresses/{ip} - IP address report
//! - /search - Search for files, URLs, domains, IPs
//!
//! ## Rate Limits
//! - Free tier: 4 requests/minute, 500/day, 15.5K/month
//! - Rate limits vary by plan
//!
//! ## Data Coverage
//! - Malware detection and analysis
//! - URL and domain reputation
//! - IP address reputation
//! - File analysis and threat intelligence
//!
//! ## Usage Restrictions
//! - API key required (env: VIRUSTOTAL_API_KEY)
//! - Rate limits based on plan

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{VirusTotalEndpoint, VirusTotalEndpoints};
pub use auth::VirusTotalAuth;
pub use parser::{
    VirusTotalParser, VtFileReport, VtAnalysisStats, VtDomainReport, VtIpReport,
};
pub use connector::VirusTotalConnector;
