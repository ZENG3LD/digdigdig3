//! # PredictIt Prediction Markets Connector
//!
//! Category: data_feeds
//! Type: Prediction Markets
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (public API)
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Price data: Yes (contract prices)
//! - Market data: Yes (prediction market contracts)
//! - Historical data: Limited (current prices only)
//! - Political prediction markets
//!
//! ## Key Endpoints
//! - /all - Get all markets with contracts (CORE endpoint)
//! - /markets/{id} - Get specific market
//!
//! ## Rate Limits
//! - Free tier: Unlimited
//! - No authentication required
//!
//! ## Data Coverage
//! - Political markets (elections, policies)
//! - Economic markets (GDP, inflation)
//! - Current contract prices
//!
//! ## Usage Restrictions
//! - Free public access
//! - No authentication needed

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{PredictItEndpoint, PredictItEndpoints};
pub use auth::PredictItAuth;
pub use parser::{
    PredictItParser, PredictItMarket, PredictItContract, PredictItResponse,
};
pub use connector::PredictItConnector;
