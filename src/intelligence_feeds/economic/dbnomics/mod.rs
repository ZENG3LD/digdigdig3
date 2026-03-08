//! # DBnomics Connector
//!
//! Category: data_feeds
//! Type: Data Aggregator
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (completely open API)
//! - Free tier: Yes (100% free, no API key required)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes (millions of economic time series)
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes
//! - Multi-provider: Yes (IMF, World Bank, ECB, OECD, Eurostat, BIS, ILO, etc.)
//!
//! ## Key Endpoints
//! - /providers - List all data providers
//! - /datasets/{provider} - List datasets for a provider
//! - /series/{provider}/{dataset}/{series} - Get series with observations (CORE endpoint)
//! - /search/series - Search for series
//! - /last-updates - Get recently updated data
//!
//! ## Rate Limits
//! - No official rate limits published
//! - Fair use policy applies
//!
//! ## Data Coverage
//! - 600+ data providers worldwide
//! - Millions of economic time series
//! - Historical depth varies by provider and series
//! - Update frequency varies by series
//!
//! ## Major Providers
//! - **IMF** - International Monetary Fund
//! - **WB** - World Bank
//! - **ECB** - European Central Bank
//! - **OECD** - Organisation for Economic Co-operation and Development
//! - **Eurostat** - European Union statistics
//! - **BIS** - Bank for International Settlements
//! - **ILO** - International Labour Organization
//!
//! ## Usage Notes
//! - No authentication required
//! - Data is free to use
//! - Check individual provider terms of use
//! - Some series may have usage restrictions from the original provider

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{DBnomicsEndpoint, DBnomicsEndpoints, DBnomicsProvider};
pub use auth::DBnomicsAuth;
pub use parser::{
    DBnomicsParser, Provider, Dataset, Series, Observation, Dimension, LastUpdate,
};
pub use connector::DBnomicsConnector;
