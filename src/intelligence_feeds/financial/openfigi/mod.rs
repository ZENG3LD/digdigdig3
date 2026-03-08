//! # OpenFIGI (Financial Instrument Global Identifier) Connector
//!
//! Category: data_feeds
//! Type: Financial Identifier Mapping
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Optional API Key (X-OPENFIGI-APIKEY header)
//! - Free tier: Yes (5 requests/min, 100 jobs/request)
//!
//! ## Data Types
//! - Identifier mapping: Yes (ticker → FIGI, ISIN → FIGI, etc.)
//! - Search: Yes (text-based instrument search)
//! - Enum values: Yes (valid values for fields)
//! - Price data: No
//! - Trading: No
//!
//! ## Key Endpoints
//! - POST /v3/mapping - Map identifiers to FIGIs (MAIN endpoint)
//! - POST /v3/search - Search by text query
//! - GET /v3/mapping/values/{key} - Enum values for a field
//!
//! ## Rate Limits
//! - Without API key: 5 requests/min, 100 jobs/request
//! - With API key: 25 requests/min, 100 jobs/request
//!
//! ## Data Coverage
//! - Global coverage across all asset classes
//! - Stocks, bonds, futures, options, ETFs, indices
//! - Multiple identifier types: ticker, ISIN, CUSIP, SEDOL, etc.
//!
//! ## Usage Restrictions
//! - Free tier available with rate limits
//! - API key optional but recommended for higher limits
//! - No redistribution of FIGI data

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{OpenFigiEndpoint, OpenFigiEndpoints};
pub use auth::OpenFigiAuth;
pub use parser::{
    OpenFigiParser, FigiResult, FigiMappingResponse, FigiSearchResponse, FigiEnumValues,
};
pub use connector::OpenFigiConnector;
