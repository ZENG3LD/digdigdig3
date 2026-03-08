//! # OECD (Organisation for Economic Co-operation and Development) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes (SDMX REST 2.0)
//! - WebSocket: No
//! - Authentication: None (public access)
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes
//! - Social statistics: Yes
//!
//! ## Key Endpoints
//! - /data/{dataflow}/{key} - Get time series data (CORE endpoint)
//! - /dataflow/{agency} - List available dataflows
//! - /dataflow/{agency}/{id} - Get dataflow metadata
//! - /datastructure/{agency}/{id} - Get data structure definition
//! - /codelist/{agency}/{id} - Get code lists
//! - /availableconstraint/{dataflow}/{key} - Check data availability
//!
//! ## Rate Limits
//! - No official rate limits documented
//! - Reasonable use expected
//!
//! ## Data Coverage
//! - 40+ member countries
//! - 100+ partner economies
//! - Economic, social, and environmental data
//! - Historical depth varies by indicator
//!
//! ## Common Dataflows
//! - QNA: Quarterly National Accounts (GDP)
//! - PRICES_CPI: Consumer Price Index
//! - MEI_CLI: Composite Leading Indicators (unemployment)
//! - MEI_FIN: Financial Indicators (interest rates)
//! - MEI_TRD: International Trade
//! - MEI_REAL: Real Sector (industrial production)
//!
//! ## Key Format
//! OECD uses dimension keys like: `COUNTRY.INDICATOR.FREQUENCY.MEASURE`
//! Example: `USA.GDP.Q.V` = USA GDP Quarterly Volume
//!
//! ## Usage Example
//! ```ignore
//! use connectors_v5::data_feeds::oecd::OecdConnector;
//!
//! let connector = OecdConnector::new();
//!
//! // Get USA GDP data from Quarterly National Accounts
//! let data = connector.get_data("QNA", "USA.GDP.Q.V", Some("2020-Q1"), Some("2023-Q4")).await?;
//!
//! // List all OECD dataflows
//! let dataflows = connector.list_dataflows("OECD").await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OecdEndpoint, OecdEndpoints, dataflows, agencies};
pub use auth::OecdAuth;
pub use parser::{
    OecdParser, OecdObservation, OecdDataflow, OecdDatastructure,
    OecdCodelist, OecdCode, OecdAvailability,
};
pub use connector::OecdConnector;
