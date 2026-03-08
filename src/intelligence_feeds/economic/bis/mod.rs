//! # BIS (Bank for International Settlements) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes (SDMX 2.0)
//! - WebSocket: No
//! - Authentication: None (public API)
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Price data: No (economic/financial statistics only)
//! - Historical data: Yes (extensive historical depth)
//! - Macro data: Yes (primary use case)
//! - Banking statistics: Yes (core focus)
//! - Central bank data: Yes (policy rates, reserves)
//!
//! ## Key Dataflows
//! - WS_CBPOL - Central bank policy rates
//! - WS_XRU - US dollar exchange rates
//! - WS_EER - Effective exchange rates
//! - WS_LONG_CPI - Long consumer price indices
//! - WS_SPP - Residential property prices
//! - WS_CREDIT - Credit statistics
//! - WS_DSR - Debt service ratios
//! - WS_LBS_D_PUB - Locational banking statistics
//! - WS_CBS_PUB - Consolidated banking statistics
//!
//! ## Rate Limits
//! - No explicit rate limits (public API)
//! - Fair use policy applies
//!
//! ## Data Coverage
//! - International banking statistics
//! - Central bank policy rates
//! - Exchange rates and derivatives
//! - Credit and debt statistics
//! - Property prices
//! - Global liquidity indicators

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{BisEndpoint, BisEndpoints};
pub use auth::BisAuth;
pub use parser::{
    BisParser, SdmxObservation, SdmxDataflow, SdmxDataStructure,
    SdmxCodelist, SdmxCode, SdmxConceptScheme, SdmxAvailability,
};
pub use connector::BisConnector;
