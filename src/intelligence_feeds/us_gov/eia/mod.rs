//! EIA (U.S. Energy Information Administration) API connector
//!
//! Provides access to U.S. energy data including:
//! - Petroleum (crude oil prices, stocks, production, imports)
//! - Natural gas (prices, storage)
//! - Electricity (generation by fuel type, retail sales)
//! - Coal
//! - Total energy
//! - Short-Term Energy Outlook (STEO) - forecasts!
//! - Annual Energy Outlook (AEO)
//! - International energy data
//! - State Energy Data System (SEDS)
//! - CO2 emissions
//!
//! # API Documentation
//! https://www.eia.gov/opendata/
//!
//! # Authentication
//! Requires API key from https://www.eia.gov/opendata/register.php
//! Set environment variable: `EIA_API_KEY`
//!
//! # Example
//! ```ignore
//! use connectors_v5::data_feeds::eia::EiaConnector;
//!
//! let connector = EiaConnector::from_env();
//!
//! // Get crude oil prices
//! let prices = connector.get_crude_oil_prices(
//!     Some("2024-01-01"),
//!     Some("2024-12-31")
//! ).await?;
//!
//! // Get natural gas storage
//! let storage = connector.get_gas_storage(None, None).await?;
//!
//! // Get STEO forecasts
//! let forecasts = connector.get_steo_forecast(None, None).await?;
//! ```

pub mod auth;
pub mod endpoints;
pub mod parser;
pub mod connector;

pub use auth::EiaAuth;
pub use endpoints::{EiaEndpoints, EiaEndpoint, Frequency, SortOrder, routes, products};
pub use parser::{EiaParser, EiaObservation, EiaMetadata, EiaFacet};
pub use connector::EiaConnector;
