//! # IMF PortWatch Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely free)
//! - Free tier: Yes (completely free public access)
//!
//! ## Data Types
//! - Maritime traffic data: Yes
//! - Chokepoint monitoring: Yes (28 global chokepoints)
//! - Port statistics: Yes
//! - Trade flows: Yes
//! - Disruption tracking: Yes
//!
//! ## Key Endpoints
//! - /portwatch/v1/chokepoints - List all 28 chokepoints
//! - /portwatch/v1/chokepoints/{id}/statistics - Traffic stats
//! - /portwatch/v1/ports - List major ports
//! - /portwatch/v1/disruptions - Active disruptions
//! - /portwatch/v1/trade-flows - Global trade flow data
//!
//! ## Rate Limits
//! - None (public data)
//!
//! ## Data Coverage
//! - 28 global maritime chokepoints
//! - Major international ports
//! - Real-time vessel traffic data
//! - Trade disruption alerts

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{ImfPortWatchEndpoint, ImfPortWatchEndpoints};
pub use auth::ImfPortWatchAuth;
pub use parser::{
    ImfPortWatchParser, PortWatchChokepoint, PortWatchPort, PortWatchTrafficStats,
    PortWatchDisruption,
};
pub use connector::ImfPortWatchConnector;
