//! # ECOS (Bank of Korea Economic Statistics System) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (path-based)
//! - Free tier: Yes (free registration required)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes (Korean economic statistics)
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes
//! - Financial markets data: Limited (exchange rates, interest rates)
//!
//! ## Key Endpoints
//! - /StatisticSearch - Get time series data (CORE endpoint)
//! - /KeyStatisticList - Get list of key statistics
//! - /StatisticTableList - Get available tables for a stat code
//! - /StatisticItemList - Get available items for a stat code
//! - /StatisticWord - Search statistics by keyword
//! - /StatMeta - Get statistical metadata
//!
//! ## Rate Limits
//! - Free tier: 100,000 requests per day
//! - No commercial restrictions
//!
//! ## Data Coverage
//! - 200+ statistical tables
//! - 10,000+ economic time series
//! - Historical depth varies (most from 1960s+)
//! - Update frequency varies by series
//!
//! ## Common Statistical Codes
//! - GDP: 200Y001 (Quarterly)
//! - CPI: 901Y009 (Monthly)
//! - Policy Rate: 722Y001 (Daily)
//! - Exchange Rates: 731Y001 (Daily)
//! - Employment: 901Y027 (Monthly)
//! - Money Supply: 101Y004 (Monthly)
//! - Trade: 403Y003 (Monthly)
//! - Industrial Production: 901Y033 (Monthly)
//!
//! ## Usage Example
//! ```ignore
//! use connectors_v5::data_feeds::ecos::{EcosConnector, STAT_GDP, CYCLE_QUARTERLY};
//!
//! let connector = EcosConnector::from_env();
//!
//! // Get quarterly GDP from 2020Q1 to 2023Q4
//! let gdp = connector.get_gdp("2020Q1", "2023Q4").await?;
//!
//! // Or use generic method with custom parameters
//! let data = connector.get_statistical_data(
//!     STAT_GDP,
//!     CYCLE_QUARTERLY,
//!     "2020Q1",
//!     "2023Q4",
//!     None,
//! ).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{
    EcosEndpoint, EcosEndpoints,
    STAT_GDP, STAT_CPI, STAT_POLICY_RATE, STAT_EXCHANGE_RATES,
    STAT_EMPLOYMENT, STAT_MONEY_SUPPLY, STAT_TRADE, STAT_INDUSTRIAL_PRODUCTION,
    CYCLE_ANNUAL, CYCLE_QUARTERLY, CYCLE_MONTHLY, CYCLE_DAILY,
};
pub use auth::EcosAuth;
pub use parser::{
    EcosParser, StatisticData, KeyStatistic, StatisticTable, StatisticItem,
    StatisticWord, StatMeta,
};
pub use connector::EcosConnector;
