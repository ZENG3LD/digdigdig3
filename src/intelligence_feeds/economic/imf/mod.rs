//! # IMF (International Monetary Fund) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes (extensive economic time series)
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes
//! - International statistics: Yes
//!
//! ## Key Endpoints
//! - /Dataflow - List available datasets
//! - /CompactData/{db}/{dims} - Get time series data (CORE endpoint)
//! - /DataStructure/{db} - Get dimension definitions
//! - /CodeList/{code}_{db} - Get available codes for dimensions
//! - /GenericData/{db}/{dims} - Alternative data format
//!
//! ## Rate Limits
//! - No official rate limits published
//! - Reasonable use expected
//!
//! ## Key Databases
//! - IFS: International Financial Statistics (interest rates, exchange rates, prices, trade, GDP)
//! - BOP: Balance of Payments
//! - DOT: Direction of Trade Statistics
//! - GFS: Government Finance Statistics
//! - GFSR: Government Finance Statistics Revenue
//! - WEO: World Economic Outlook
//! - PCPS: Primary Commodity Prices
//! - APDREO: Asia-Pacific Regional Economic Outlook
//!
//! ## Common Indicators (IFS Database)
//! - NGDP_RPCH: Real GDP growth
//! - PCPI_IX: Consumer Price Index
//! - FPOLM_PA: Policy interest rate
//! - ENDA_XDC_USD_RATE: Exchange rate to USD
//! - TXG_FOB_USD: Exports (FOB)
//! - TMG_CIF_USD: Imports (CIF)
//! - RAFA_USD: International reserves
//!
//! ## Dimension Format
//! Dot-separated: `{freq}.{country}.{indicator}`
//! - Frequency: A (Annual), Q (Quarterly), M (Monthly)
//! - Country: ISO 2-letter codes (US, GB, DE, JP, CN, etc.)
//! - Indicator: Specific economic indicator code
//!
//! Example: `A.US.NGDP_RPCH` = Annual US Real GDP Growth
//!
//! ## Data Coverage
//! - Extensive international coverage
//! - Historical depth varies by series
//! - Updated regularly (frequency varies by indicator)
//!
//! ## Usage Restrictions
//! - Free for all uses
//! - Attribution to IMF recommended
//! - No API key required

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{ImfEndpoint, ImfEndpoints, format_dimensions, parse_dimensions};
pub use auth::ImfAuth;
pub use parser::{
    ImfParser, Dataflow, ImfSeries, ImfObservation, DataStructure,
    Dimension, CodeList, Code,
};
pub use connector::ImfConnector;
