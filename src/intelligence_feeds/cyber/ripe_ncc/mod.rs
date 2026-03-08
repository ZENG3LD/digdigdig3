//! # RIPE NCC (RIPEstat) Connector
//!
//! Category: data_feeds
//! Type: Internet Infrastructure Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely public)
//! - Free tier: Yes (completely free, generous rate limits)
//!
//! ## Data Types
//! - BGP routing data: Yes
//! - AS (Autonomous System) information: Yes
//! - IP allocation data: Yes
//! - Network infrastructure: Yes
//! - Country internet resources: Yes
//!
//! ## Key Endpoints
//! - /country-resource-stats/data.json - Country internet resources
//! - /as-overview/data.json - ASN overview
//! - /routing-status/data.json - BGP routing status
//! - /bgp-state/data.json - BGP state
//! - /announced-prefixes/data.json - Announced prefixes
//! - /asn-neighbours/data.json - ASN neighbors
//! - /network-info/data.json - Network info for IP
//! - /rir-stats-country/data.json - RIR allocation by country
//! - /country-resource-list/data.json - Country's resources
//! - /abuse-contact-finder/data.json - Abuse contacts
//!
//! ## Rate Limits
//! - Free tier: Very generous, no strict limits documented
//! - No authentication required
//!
//! ## Data Coverage
//! - Global internet routing and allocation data
//! - Real-time BGP routing information
//! - Historical data available for some endpoints
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Attribution appreciated but not required

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{RipeNccEndpoint, RipeNccEndpoints};
pub use auth::RipeNccAuth;
pub use parser::{
    RipeNccParser, RipeCountryStats, RipeAsOverview, RipeRoutingStatus,
    RipeBgpState, RipeAnnouncedPrefix, RipeTimeline, RipeAsnNeighbour,
    RipeNetworkInfo, RipeRirStats, RipeRirStatEntry, RipeCountryResource,
    RipeAbuseContact,
};
pub use connector::RipeNccConnector;
