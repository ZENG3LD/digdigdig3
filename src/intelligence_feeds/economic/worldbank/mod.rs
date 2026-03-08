//! # World Bank API Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely free, no API key required)
//! - Free tier: Yes (unlimited, non-commercial)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes (varies by indicator)
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes (16,000+ indicators)
//! - Country coverage: 200+ countries and territories
//!
//! ## Key Endpoints
//! - /country/{country}/indicator/{indicator} - Get time series data (CORE endpoint)
//! - /indicator/{id} - Get indicator metadata
//! - /indicator?qterm={query} - Search for indicators
//! - /country/{code} - Get country metadata
//! - /topic/{id}/indicator - Browse indicators by topic
//!
//! ## Rate Limits
//! - No explicit rate limits
//! - Recommended: Keep requests reasonable for free service
//! - No paid tiers available
//!
//! ## Data Coverage
//! - 16,000+ development indicators
//! - 200+ countries and territories
//! - Historical depth: Varies (some from 1960s, many from 1990s+)
//! - Update frequency: Annual, quarterly, or monthly depending on indicator
//!
//! ## Common Indicators
//! - GDP (current US$): NY.GDP.MKTP.CD
//! - GDP growth (annual %): NY.GDP.MKTP.KD.ZG
//! - Inflation (CPI %): FP.CPI.TOTL.ZG
//! - Unemployment (%): SL.UEM.TOTL.ZS
//! - Population: SP.POP.TOTL
//! - GNI per capita: NY.GNP.PCAP.CD
//!
//! ## Usage Example
//! ```ignore
//! use connectors_v5::data_feeds::worldbank::WorldBankConnector;
//!
//! let connector = WorldBankConnector::new();
//!
//! // Get GDP data for USA from 2020-2023
//! let data = connector.get_indicator_data("US", "NY.GDP.MKTP.CD", Some("2020"), Some("2023")).await?;
//!
//! // Search for GDP indicators
//! let indicators = connector.search_indicators("GDP", None, Some(10)).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{WorldBankEndpoint, WorldBankEndpoints, indicators};
pub use auth::WorldBankAuth;
pub use parser::{
    WorldBankParser, IndicatorObservation, IndicatorMetadata, IndicatorInfo,
    Country, CountryInfo, Topic, TopicInfo, Source, SourceInfo,
    IncomeLevel, LendingType, Pagination,
};
pub use connector::WorldBankConnector;
