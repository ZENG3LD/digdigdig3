//! # arXiv (Academic Papers) Connector
//!
//! Category: data_feeds
//! Type: Academic Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Research papers: Yes
//! - Metadata: Yes (titles, authors, abstracts, categories)
//! - Full text: No (only abstracts via API)
//! - PDF links: Yes
//!
//! ## Key Endpoints
//! - /api/query - Search papers with flexible query syntax
//!
//! ## Rate Limits
//! - Recommended: 1 request per 3 seconds
//! - No hard limits but respect the API
//!
//! ## Data Coverage
//! - 2+ million research papers
//! - Fields: Physics, Mathematics, Computer Science, Quantitative Finance, Economics, Statistics
//! - Historical depth: Papers from 1991 onwards
//! - Update frequency: Real-time as papers are submitted
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Attribution requested
//! - Rate limiting recommended

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{ArxivEndpoint, ArxivEndpoints};
pub use auth::ArxivAuth;
pub use parser::{
    ArxivParser, ArxivPaper, ArxivSearchResult,
};
pub use connector::ArxivConnector;
