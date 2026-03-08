//! # Cloudflare Radar API Connector
//!
//! Category: data_feeds
//! Type: Internet Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Bearer Token
//! - Free tier: Yes
//!
//! ## Data Types
//! - Internet traffic data: Yes
//! - HTTP traffic statistics: Yes
//! - DDoS attack data: Yes
//! - DNS query data: Yes
//! - Top domains ranking: Yes
//!
//! ## Key Endpoints
//! - /http/top/locations - Top locations by HTTP requests
//! - /http/top/ases - Top ASes by traffic
//! - /http/summary/* - Traffic summaries (bot class, device type, OS, browser, protocols)
//! - /http/timeseries - HTTP traffic time series
//! - /attacks/layer3/* - Layer 3 DDoS attacks
//! - /attacks/layer7/* - Layer 7 DDoS attacks
//! - /dns/top/locations - DNS query top locations
//! - /ranking/top - Top domains ranking
//!
//! ## Rate Limits
//! - Free tier: Available
//! - Rate limits vary by endpoint
//!
//! ## Data Coverage
//! - Global internet traffic data
//! - Real-time and historical data
//! - Multiple time ranges (1d, 7d, 14d, 28d)
//!
//! ## Usage Restrictions
//! - API token required
//! - Free tier available

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{CloudflareRadarEndpoint, CloudflareRadarEndpoints};
pub use auth::CloudflareRadarAuth;
pub use parser::{
    CloudflareRadarParser, RadarTimeSeries, RadarTopLocation, RadarTopAs,
    RadarBotSummary, RadarDeviceSummary, RadarProtocolSummary, RadarOsSummary,
    RadarBrowserSummary, RadarAttackSummary, RadarTopDomain,
};
pub use connector::CloudflareRadarConnector;
