//! # Jupiter Exchange Connector
//!
//! Jupiter DEX aggregator connector for Solana.
//!
//! ## Architecture
//!
//! - `endpoints` - URL'ы и endpoint enum
//! - `auth` - API key authentication
//! - `parser` - Парсинг JSON ответов
//! - `connector` - JupiterConnector + impl трейтов
//! - `websocket` - WebSocket (not supported)
//!
//! ## Features
//!
//! - ✅ Market data via Quote and Price APIs
//! - ✅ Token discovery via Tokens API
//! - ❌ Native WebSocket (not available)
//! - ❌ Trading (requires Solana wallet integration)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::jupiter::JupiterConnector;
//! use connectors_v5::{Symbol, AccountType};
//!
//! // Public access (no API key, limited features)
//! let connector = JupiterConnector::public().await?;
//!
//! // Or with API key for full access
//! let connector = JupiterConnector::new(Some("your-api-key".to_string())).await?;
//!
//! // Get price (works with or without API key)
//! let symbol = Symbol::new("SOL", "USDC");
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//!
//! // Get ticker (requires API key)
//! let ticker = connector.get_ticker(symbol, AccountType::Spot).await?;
//! ```
//!
//! ## Notes
//!
//! - Jupiter uses Solana **mint addresses** instead of symbols
//! - Common tokens (SOL, USDC, etc.) are mapped automatically
//! - For other tokens, use mint address directly in Symbol
//! - No orderbook or klines (DEX aggregator limitation)
//! - WebSocket not supported (use polling or Solana RPC)

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{JupiterUrls, JupiterEndpoint, MintRegistry, to_raw_amount, from_raw_amount};
pub use auth::JupiterAuth;
pub use parser::{JupiterParser, QuoteResponse, PriceResponse, TokenMetadata};
pub use connector::JupiterConnector;
pub use websocket::JupiterWebSocket;
