//! # Eurostat API Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely open)
//! - Free tier: Yes (100% free, no registration required)
//!
//! ## Data Types
//! - Price data: No (economic/social statistics only)
//! - Historical data: Yes (extensive historical coverage)
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes (GDP, CPI, unemployment, etc.)
//! - Social statistics: Yes (population, health, education, etc.)
//!
//! ## Key APIs
//! - Statistics API: Main data access (JSON-stat v2 format)
//! - SDMX API: Metadata and structural definitions
//! - Catalogue API: Table of contents
//!
//! ## Key Endpoints
//! - /data/{dataset} - Get dataset observations (CORE endpoint)
//! - /label/{dataset} - Get dataset metadata
//! - /dataflow/ESTAT/all/latest - List all dataflows
//! - /toc - Table of contents
//!
//! ## Rate Limits
//! - No explicit rate limits documented
//! - Fair use policy applies
//! - Max 50 sub-indicators per request
//!
//! ## Data Coverage
//! - European Union member states
//! - Some global statistics
//! - Economic, social, environmental data
//! - Historical depth varies by indicator
//!
//! ## Key Datasets
//! - nama_10_gdp: GDP and main components
//! - prc_hicp_midx: HICP (Harmonised Index of Consumer Prices)
//! - une_rt_m: Unemployment rate (monthly)
//! - sts_inpr_m: Industrial production (monthly)
//! - ext_lt_maineu: International trade
//! - demo_gind: Population indicators
//! - gov_10dd_edpt1: Government debt
//!
//! ## Usage Restrictions
//! - Free for all uses
//! - Attribution recommended
//! - Copyright: European Union
//! - Reuse permitted under open data policy

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{EurostatEndpoint, EurostatEndpoints, EndpointBase};
pub use auth::EurostatAuth;
pub use parser::{
    EurostatParser, EurostatDataset, EurostatDimension, EurostatLabel,
    EurostatDataflow, EurostatTocEntry,
};
pub use connector::EurostatConnector;
