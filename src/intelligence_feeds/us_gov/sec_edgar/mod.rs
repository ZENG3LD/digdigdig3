//! # SEC EDGAR Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None (User-Agent header required)
//! - Free tier: Yes (completely free, public domain)
//!
//! ## Data Types
//! - Company filings: Yes (10-K, 10-Q, 8-K, etc.)
//! - Financial data: Yes (XBRL structured financial data)
//! - Insider trading: Yes (Form 4)
//! - Institutional holdings: Yes (13F)
//! - Full-text search: Yes
//!
//! ## Key Endpoints
//! - /submissions/CIK*.json - Company filings
//! - /api/xbrl/companyfacts/CIK*.json - XBRL financial data
//! - /api/xbrl/companyconcept/CIK*/{taxonomy}/{tag}.json - Specific financial concept
//! - /api/xbrl/frames/{taxonomy}/{tag}/{unit}/CY*.json - Aggregated data across all filers
//! - /files/company_tickers.json - All companies + CIKs
//! - /search-index - Full-text search (EFTS)
//!
//! ## Rate Limits
//! - 10 requests per second
//! - No authentication required
//!
//! ## Data Coverage
//! - All SEC registered companies
//! - Historical data back to mid-1990s
//! - Real-time filings (within minutes of submission)
//!
//! ## Usage Restrictions
//! - MUST set User-Agent header with company name and email
//! - Rate limit enforcement via User-Agent tracking
//! - Public domain data (no usage restrictions)

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{SecEdgarEndpoint, SecEdgarEndpoints};
pub use auth::SecEdgarAuth;
pub use parser::{
    SecEdgarParser, CompanyFiling, CompanyFacts, CompanyConcept, XbrlFrame,
    CompanyTicker, FilingMetadata, FinancialFact, SearchResult,
};
pub use connector::SecEdgarConnector;
