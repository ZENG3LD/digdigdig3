//! # C2IntelFeeds (Command & Control Threat Intelligence) Connector
//!
//! Category: data_feeds
//! Type: Cybersecurity Threat Intelligence
//!
//! ## Features
//! - REST API: Yes (GitHub raw files)
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free, public GitHub repo)
//!
//! ## Data Types
//! - IP indicators: Yes (Cobalt Strike, Metasploit C2 IPs)
//! - Domain indicators: Yes (C2 domains, fronting domains)
//! - Time windows: All-time, 90-day, 30-day, 7-day
//! - IOC classification: Yes (threat framework identification)
//!
//! ## Key Endpoints
//! - /IPC2s.csv - All-time IP indicators
//! - /IPC2s-30day.csv - Last 30 days IP indicators
//! - /IPC2s-7day.csv - Last 7 days IP indicators
//! - /domainC2s.csv - All-time domain indicators
//! - /domainC2s-30day.csv - Last 30 days domain indicators
//!
//! ## Rate Limits
//! - GitHub raw files: ~60 requests/hour (unauthenticated)
//! - Recommended: Poll once per hour or less
//!
//! ## Data Coverage
//! - Focus: Cobalt Strike and Metasploit C2 infrastructure
//! - Sources: Community-contributed threat intelligence
//! - Historical depth: Multi-year dataset
//! - Update frequency: Multiple times per day
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Public GitHub repository
//! - No authentication required
//! - Respect GitHub rate limits

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{C2IntelFeedsEndpoint, C2IntelFeedsEndpoints};
pub use auth::C2IntelFeedsAuth;
pub use parser::{C2IntelFeedsParser, C2Indicator, IndicatorType};
pub use connector::C2IntelFeedsConnector;
