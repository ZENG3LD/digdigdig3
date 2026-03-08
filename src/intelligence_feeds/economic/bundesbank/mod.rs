//! Deutsche Bundesbank (German Federal Bank) connector
//!
//! Provides access to German economic and financial statistics via SDMX REST API.
//!
//! # Overview
//!
//! The Bundesbank provides comprehensive statistics on:
//! - Exchange rates (BBEX3)
//! - Securities and financial markets (BBSIS, BBFID)
//! - Banking statistics (BBK01)
//! - Investment funds (BBK_IVF)
//! - Monetary Financial Institutions (BBMFI)
//!
//! # Authentication
//!
//! The Bundesbank API is publicly accessible and does not require authentication.
//!
//! # API Standard
//!
//! Uses SDMX (Statistical Data and Metadata eXchange) REST API.
//!
//! # Example
//!
//! ```ignore
//! use connectors_v5::data_feeds::bundesbank::{BundesbankConnector, dataflows};
//!
//! let connector = BundesbankConnector::new();
//!
//! // Get EUR/USD exchange rates
//! let data = connector.get_data(
//!     dataflows::EXCHANGE_RATES,
//!     "D.EUR.USD.BB.AC.C04",
//!     Some("2024-01-01"),
//!     Some("2024-12-31")
//! ).await?;
//!
//! // List all dataflows
//! let dataflows = connector.list_dataflows().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{
    BundesbankEndpoints,
    BundesbankEndpoint,
    dataflows,
    format_ts_key,
    format_period,
};
pub use auth::BundesbankAuth;
pub use parser::{
    BundesbankParser,
    SdmxObservation,
    SdmxDataflow,
    SdmxDataStructure,
    SdmxCodelist,
    SdmxCode,
    SdmxConceptScheme,
};
pub use connector::BundesbankConnector;
