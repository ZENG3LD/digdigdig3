//! US Census Bureau API connector
//!
//! This module provides access to the US Census Bureau's extensive collection of
//! economic indicators and demographic data through their public API.
//!
//! # Key Features
//!
//! ## Economic Indicators (Most Important for Trading)
//! - **Retail Sales**: Advance Monthly Sales for Retail and Food Services
//! - **Housing Starts**: New Residential Construction
//! - **New Home Sales**: New Residential Sales
//! - **Trade Data**: U.S. International Trade in Goods and Services
//! - **Manufacturing**: Manufacturers' Shipments, Inventories, and Orders
//! - **Construction Spending**: Value of Construction Put in Place
//!
//! ## Demographic Data
//! - **Population**: American Community Survey (ACS) data
//! - **Income**: Median household income
//! - **Employment**: Labor force statistics
//!
//! # Authentication
//!
//! Requires a free API key from: https://api.census.gov/data/key_signup.html
//!
//! Set the environment variable: `CENSUS_API_KEY=your_key_here`
//!
//! # Quick Start
//!
//! ```ignore
//! use connectors_v5::data_feeds::census::CensusConnector;
//!
//! let connector = CensusConnector::from_env();
//!
//! // Get latest retail sales data
//! let retail = connector.get_retail_sales(None).await?;
//!
//! // Get housing starts for January 2024
//! let housing = connector.get_housing_starts(Some("2024-01")).await?;
//!
//! // Get population data
//! let pop = connector.get_population("2021", "*").await?;
//! ```
//!
//! # API Response Format
//!
//! Census API returns data as array of arrays:
//! ```json
//! [
//!   ["NAME", "B01001_001E", "state"],
//!   ["Alabama", "5024279", "01"],
//!   ["Alaska", "733391", "02"]
//! ]
//! ```
//!
//! The first array is the header row, and subsequent arrays are data rows.

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{
    CensusEndpoint, CensusEndpoints, format_geography, parse_census_response,
    indicators, datasets,
};
pub use auth::CensusAuth;
pub use parser::{
    CensusParser, CensusDataRow, EconomicIndicatorObservation, DatasetInfo,
};
pub use connector::CensusConnector;
