//! # Shodan Internet Scanner Connector
//!
//! Category: data_feeds
//! Type: Internet Security & Threat Intelligence
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (limited queries)
//!
//! ## Data Types
//! - Host information: Yes
//! - Search results: Yes
//! - DNS resolution: Yes
//! - Network scanning data: Yes
//! - Vulnerability data: Yes
//!
//! ## Key Endpoints
//! - /shodan/host/{ip} - Host information
//! - /shodan/host/count - Count results for query
//! - /shodan/host/search - Search Shodan
//! - /dns/resolve - DNS resolve hostnames to IPs
//! - /dns/reverse - Reverse DNS lookup
//! - /tools/myip - Get your current IP
//! - /api-info - API plan info
//! - /shodan/ports - List of ports Shodan crawls
//! - /shodan/protocols - List of protocols Shodan crawls
//!
//! ## Rate Limits
//! - Free tier: Limited queries
//! - Rate limits vary by plan
//!
//! ## Data Coverage
//! - Global internet-connected devices
//! - Port scanning data
//! - Service banners
//! - SSL/TLS certificates
//! - Vulnerability information
//!
//! ## Usage Restrictions
//! - API key required (env: SHODAN_API_KEY)
//! - Rate limits based on plan

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{ShodanEndpoint, ShodanEndpoints};
pub use auth::ShodanAuth;
pub use parser::{
    ShodanParser, ShodanHost, ShodanSearchResult, ShodanService,
    ShodanApiInfo, ShodanDnsResult,
};
pub use connector::ShodanConnector;
