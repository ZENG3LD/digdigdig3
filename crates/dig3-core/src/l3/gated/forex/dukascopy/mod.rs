//! # Dukascopy Data Provider Connector
//!
//! Category: forex
//! Type: Data Provider (historical tick data)
//!
//! ## Features
//! - REST API: No (uses direct binary file downloads)
//! - WebSocket: No (official - uses binary downloads only)
//! - Authentication: None required (public datafeed)
//! - Free tier: Yes (unlimited historical tick data)
//!
//! ## Data Types
//! - Price data: Yes (tick-level bid/ask)
//! - Historical data: Yes (2003+ for major forex pairs)
//! - Derivatives data: No
//! - Fundamentals: No
//! - On-chain: No
//! - Macro data: No
//!
//! ## Data Access Method
//!
//! Dukascopy provides free historical tick data via **binary file downloads** (.bi5 format):
//! - URL pattern: `https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5`
//! - Format: LZMA-compressed binary (20 bytes per tick)
//! - Granularity: Tick-level (every price change)
//! - Historical depth: 2003+ for major forex pairs
//! - No authentication required
//! - Free unlimited access
//!
//! ## Limitations
//!
//! - **Trading**: Not supported (data provider only)
//! - **Real-time data**: Not available via this connector (use JForex SDK for live data)
//! - **Account operations**: Not supported
//! - **WebSocket**: Not implemented (binary downloads only)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::forex::dukascopy::DukascopyConnector;
//! use connectors_v5::core::types::*;
//!
//! let connector = DukascopyConnector::new();
//!
//! // Get historical tick data
//! let symbol = Symbol {
//!     base: "EUR".to_string(),
//!     quote: "USD".to_string(),
//! };
//!
//! // Get klines (constructed from tick data)
//! let klines = connector.get_klines(
//!     symbol,
//!     "1h",
//!     Some(24),
//!     AccountType::Spot
//! ).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{
    DukascopyEndpoint,
    DukascopyUrls,
    format_symbol,
    parse_symbol,
    get_point_value,
    build_tick_data_url,
};
pub use auth::DukascopyAuth;
pub use parser::DukascopyParser;
pub use connector::DukascopyConnector;
