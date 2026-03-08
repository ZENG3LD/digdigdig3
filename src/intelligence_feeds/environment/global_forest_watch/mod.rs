//! # Global Forest Watch (GFW) API Connector
//!
//! Category: data_feeds
//! Type: Environmental Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header)
//! - Free tier: Yes
//!
//! ## Data Types
//! - Forest coverage data: Yes
//! - Deforestation alerts: Yes
//! - Tree cover loss/gain: Yes
//! - Fire alerts: Yes
//! - Historical data: Yes
//!
//! ## Key Endpoints
//! - /dataset - List all datasets
//! - /dataset/{id} - Dataset details
//! - /dataset/{id}/latest - Latest version
//! - /dataset/{dataset_id}/{version}/query - Query dataset
//! - /forest-change/statistics - Forest change statistics
//! - /forest-change/loss - Tree cover loss data
//!
//! ## Rate Limits
//! - Free tier rate limits apply (check GFW documentation)
//!
//! ## Data Coverage
//! - Global forest coverage
//! - Historical deforestation data
//! - Real-time forest monitoring
//! - Fire alerts

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{GfwEndpoint, GfwEndpoints};
pub use auth::GfwAuth;
pub use parser::{
    GfwParser, GfwDataset, GfwTreeCoverLoss, GfwTreeCoverGain,
    GfwForestStats, GfwAlert, GfwCountryStats,
};
pub use connector::GfwConnector;
