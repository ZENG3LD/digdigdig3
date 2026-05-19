//! KRX (Korea Exchange) connector
//!
//! Category: stocks/korea
//! Type: Official stock exchange data provider (DATA ONLY - NO TRADING)
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (requires approval)
//! - Free tier: Yes (with registration)
//!
//! ## Data Types
//! - Price data: Yes (OHLCV)
//! - Historical data: Yes (from listing date)
//! - Stock information: Yes (company info, market cap)
//! - Investor trading: Yes (by investor type)
//! - Market indices: Yes (KOSPI, KOSDAQ, KONEX)
//! - Short selling: Yes
//!
//! ## Important Notes
//! - All data delayed by 1 business day minimum
//! - Updates at 1:00 PM KST
//! - Numeric values returned as comma-formatted strings
//! - Most field names and data in Korean
//! - Requires service-specific API approval
//!
//! ## API Documentation
//! - Open API Portal: https://openapi.krx.co.kr/
//! - Data Marketplace: https://data.krx.co.kr/
//! - Public Data Portal: https://www.data.go.kr/

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use connector::KrxConnector;
