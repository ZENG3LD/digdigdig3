//! # Deribit Exchange Connector
//!
//! Full implementation of Deribit derivatives exchange connector.
//!
//! ## Overview
//!
//! Deribit is a cryptocurrency derivatives exchange specializing in:
//! - Bitcoin (BTC) and Ethereum (ETH) options and futures
//! - USDC-settled linear instruments (SOL, XRP, etc.)
//! - Perpetual contracts (inverse and linear)
//! - European-style options
//!
//! ## Key Characteristics
//!
//! - **Protocol**: JSON-RPC 2.0 (over HTTP and WebSocket)
//! - **Authentication**: OAuth 2.0 (access token + refresh token)
//! - **Rate Limits**: Credit-based system
//! - **Settlement**: Cash settlement only (no physical delivery)
//! - **WebSocket**: Preferred transport for real-time data
//!
//! ## Module Structure
//!
//! - `endpoints` - JSON-RPC methods, URLs, symbol formatting
//! - `auth` - OAuth 2.0 authentication (client credentials, client signature)
//! - `parser` - JSON-RPC response parsing (REST and WebSocket)
//! - `connector` - DeribitConnector + trait implementations
//! - `websocket` - WebSocket client (subscriptions, notifications)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::deribit::DeribitConnector;
//! use connectors_v5::core::{Credentials, AccountType, Symbol};
//! use connectors_v5::core::traits::MarketData;
//!
//! // Create connector
//! let credentials = Credentials::new("client_id", "client_secret");
//! let connector = DeribitConnector::new(Some(credentials), false).await?;
//!
//! // Get market data
//! let symbol = Symbol::new("BTC", "USD");
//! let price = connector.get_price(&symbol, AccountType::FuturesCross).await?;
//!
//! // Place order
//! use connectors_v5::core::traits::Trading;
//! use connectors_v5::core::types::OrderSide;
//! let order = connector.market_order(&symbol, OrderSide::Buy, 100.0, AccountType::FuturesCross).await?;
//! ```
//!
//! ## Instrument Name Formats
//!
//! - **Perpetuals**: `BTC-PERPETUAL`, `ETH-PERPETUAL`
//! - **Linear Perpetuals**: `SOL_USDC-PERPETUAL`, `XRP_USDC-PERPETUAL`
//! - **Futures**: `BTC-29MAR24`, `ETH-27DEC24`
//! - **Options**: `BTC-27DEC24-50000-C`, `ETH-29MAR24-3000-P`
//!
//! ## Authentication
//!
//! Deribit uses OAuth 2.0:
//! 1. Call `public/auth` with client credentials or client signature
//! 2. Receive access token (15 min expiry) and refresh token
//! 3. Use `Authorization: Bearer {token}` header for private requests
//! 4. Refresh token proactively before expiration
//!
//! ## JSON-RPC Format
//!
//! All requests use JSON-RPC 2.0:
//!
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "id": 1,
//!   "method": "public/get_instruments",
//!   "params": {
//!     "currency": "BTC",
//!     "kind": "future"
//!   }
//! }
//! ```
//!
//! Response:
//!
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "id": 1,
//!   "result": [ /* data */ ],
//!   "testnet": false,
//!   "usIn": 1234567890,
//!   "usOut": 1234567892,
//!   "usDiff": 2
//! }
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{DeribitMethod, DeribitUrls, format_symbol, parse_currency, parse_instrument_kind};
pub use auth::DeribitAuth;
pub use parser::DeribitParser;
pub use connector::DeribitConnector;
pub use websocket::DeribitWebSocket;
