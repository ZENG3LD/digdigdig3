//! # Hyperliquid DEX Connector
//!
//! Implementation of Hyperliquid perpetuals and spot DEX connector.
//!
//! ## Architecture
//!
//! - `endpoints` - API URLs and endpoint enum
//! - `auth` - EIP-712 signature implementation
//! - `parser` - JSON response parsing
//! - `connector` - HyperliquidConnector + trait implementations
//! - `websocket` - WebSocket connection with auto-reconnect
//!
//! ## Authentication
//!
//! Hyperliquid uses EIP-712 wallet signatures instead of API keys:
//! - L1 actions (trading) use phantom agent construction
//! - User-signed actions (transfers, withdrawals) use direct EIP-712
//!
//! ## Symbol Format
//!
//! - Perpetuals: Direct coin name ("BTC", "ETH") → asset ID from metadata
//! - Spot: "@{index}" format ("@0" for PURR/USDC, "@107" for HYPE/USDC)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::hyperliquid::HyperliquidConnector;
//!
//! let connector = HyperliquidConnector::new(wallet_private_key, false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price("BTC".into(), AccountType::FuturesCross).await?;
//! let orderbook = connector.get_orderbook("BTC".into(), None, AccountType::FuturesCross).await?;
//!
//! // Extended methods (Hyperliquid-specific)
//! let metadata = connector.get_metadata().await?;
//! let all_mids = connector.get_all_mids().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{HyperliquidEndpoint, HyperliquidUrls};
pub use auth::HyperliquidAuth;
pub use parser::HyperliquidParser;
pub use connector::HyperliquidConnector;
pub use websocket::HyperliquidWebSocket;
