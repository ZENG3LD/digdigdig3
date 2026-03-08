//! # Tinkoff Invest API Connector
//!
//! Russian broker with full trading support for MOEX (Moscow Exchange).
//!
//! Category: stocks/russia
//! Type: Russian broker with full trading capabilities
//!
//! ## Features
//! - REST API: Yes (REST proxy over gRPC)
//! - WebSocket: Yes (planned for future implementation)
//! - Authentication: Bearer Token
//! - Free tier: Yes (completely free for all Tinkoff Investments clients)
//!
//! ## Data Types
//! - Price data: Yes (real-time)
//! - Historical data: Yes (up to 10 years, intervals from 5s to 1 month)
//! - Order book: Yes (L2, depth 1-50 levels)
//! - Trading: Yes (market, limit, stop orders)
//! - Portfolio: Yes (real-time positions and P&L)
//! - Account: Yes (balance, margin, multiple account types)
//!
//! ## Trading Support
//! - Russian stocks: ~1,900 shares on MOEX
//! - Bonds: ~655 Russian bonds (with coupon data)
//! - ETFs: ~105 ETFs
//! - Futures: ~284 futures contracts
//! - Options: Available (with underlying asset tracking)
//! - Currencies: ~21 currency pairs
//!
//! ## Account Types
//! - Standard brokerage account
//! - IIS (Individual Investment Account)
//! - Sandbox (testing environment)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::stocks::russia::tinkoff::TinkoffConnector;
//!
//! // Create connector from environment variable TINKOFF_TOKEN
//! let connector = TinkoffConnector::from_env();
//!
//! // Or with explicit token
//! let connector = TinkoffConnector::new("t.xxx".to_string(), false);
//!
//! // Market data
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let candles = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot).await?;
//!
//! // Trading (requires full-access token)
//! let order = connector.place_order(order).await?;
//! ```
//!
//! ## Important Notes
//!
//! ### Token Requirements
//! - All endpoints require authentication (no public access)
//! - Token format: `t.` prefix + alphanumeric string
//! - Token types: Readonly, Full-access, Account-specific, Sandbox
//! - Generate at: https://www.tinkoff.ru/invest/settings/
//!
//! ### API Characteristics
//! - Primary protocol: gRPC (this implementation uses REST proxy)
//! - REST proxy base: `https://invest-public-api.tbank.ru/rest/`
//! - Sandbox base: `sandbox-invest-public-api.tinkoff.ru`
//! - Response format: JSON (Protocol Buffers mapping)
//!
//! ### Special Data Types
//! - Prices use Quotation: `{units: int64, nano: int32}` (9 decimal places)
//! - Money uses MoneyValue: `{currency, units, nano}`
//! - Timestamps in ISO 8601 UTC format
//!
//! ### Trading Restrictions
//! - Maximum order value: 6,000,000 RUB via API
//! - Some instruments require qualified investor status
//! - Some instruments forbidden for API trading (use GetTradingStatus to check)
//!
//! ## References
//! - Documentation: https://tinkoff.github.io/investAPI/
//! - Proto contracts: https://github.com/Tinkoff/investAPI

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(test)]
mod tests;

// WebSocket support planned for future implementation
// mod websocket;

pub use endpoints::{TinkoffEndpoint, TinkoffEndpoints};
pub use auth::TinkoffAuth;
pub use parser::TinkoffParser;
pub use connector::TinkoffConnector;

// #[cfg(feature = "websocket")]
// pub use websocket::TinkoffWebSocket;
