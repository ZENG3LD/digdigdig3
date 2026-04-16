//! # JQuants Connector
//!
//! Category: stocks/japan
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes (V1 with two-token auth)
//! - WebSocket: No (REST-only data provider)
//! - Authentication: Two-token system (refresh token → ID token)
//! - Free tier: Yes (12-week delayed data, 2 years history)
//!
//! ## Data Types
//! - Price data: Yes (daily OHLC, minute bars, ticks)
//! - Historical data: Yes (2+ years on free tier)
//! - Derivatives data: Yes (futures, options) - Premium only
//! - Fundamentals: Yes (financial statements, dividends)
//! - Indices: Yes (TOPIX, etc.) - Standard+ plan
//!
//! ## Important Notes
//! - **Data-only provider**: NO trading capabilities
//! - **Japan-focused**: Tokyo Stock Exchange only
//! - **Free tier delay**: 12-week delay on all data
//! - **Two-token auth**: Refresh token (7 days) → ID token (24 hours)
//!
//! ## Provider Information
//! - Official data from Japan Exchange Group (JPX)
//! - Website: https://jpx-jquants.com/en
//! - Documentation: https://jpx.gitbook.io/j-quants-en/

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{JQuantsEndpoint, JQuantsUrls};
pub use auth::JQuantsAuth;
pub use parser::JQuantsParser;
pub use connector::JQuantsConnector;
