//! BLS (Bureau of Labor Statistics) API connector
//!
//! Provides access to US labor market and economic indicators.
//!
//! # Features
//! - Consumer Price Index (CPI)
//! - Unemployment Rate
//! - Nonfarm Payrolls
//! - Producer Price Index (PPI)
//! - Job Openings (JOLTS)
//! - Employment Cost Index
//! - Productivity metrics
//! - Import/Export price indices
//! - 10,000+ other economic series
//!
//! # Authentication
//! - Optional API key via `BLS_API_KEY` environment variable
//! - Without key: 25 queries/day, 10 years max range
//! - With key: 500 queries/day, 20 years max range
//!
//! # Example
//! ```ignore
//! use connectors_v5::data_feeds::bls::BlsConnector;
//!
//! let connector = BlsConnector::from_env();
//!
//! // Get CPI data
//! let cpi = connector.get_cpi("2020", "2024").await?;
//!
//! // Get unemployment rate
//! let unemployment = connector.get_unemployment_rate("2020", "2024").await?;
//!
//! // Get multiple series at once
//! let series = connector.get_multiple_series(
//!     &["CUSR0000SA0", "LNS14000000"],
//!     "2023",
//!     "2024"
//! ).await?;
//! ```

mod auth;
mod connector;
mod endpoints;
mod parser;

pub use auth::BlsAuth;
pub use connector::BlsConnector;
pub use endpoints::{
    BlsEndpoint, BlsEndpoints, format_series_id, parse_series_id,
    // Popular series constants
    CPI_ALL_URBAN, UNEMPLOYMENT_RATE, NONFARM_PAYROLLS, AVG_HOURLY_EARNINGS,
    PPI_FINISHED_GOODS, CPI_ENERGY, CPI_FOOD, EMPLOYMENT_COST_INDEX,
    PRODUCTIVITY, IMPORT_PRICES, EXPORT_PRICES, JOLTS_JOB_OPENINGS,
};
pub use parser::{BlsParser, BlsSeries, BlsDataPoint};
