//! # Lighter Exchange Connector
//!
//! Complete implementation of connector for Lighter DEX.
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint enum
//! - `auth` - Request signing (transaction signing for Phase 3)
//! - `parser` - JSON response parsing
//! - `connector` - LighterConnector + trait implementations
//! - `websocket` - WebSocket connection
//!
//! ## Implementation Status
//!
//! ### Phase 1 (CURRENT): Public Market Data
//! - ✅ ExchangeIdentity trait
//! - ✅ MarketData trait (get_price, get_ticker, get_klines, get_trading_pairs)
//! - ✅ Basic WebSocket structure
//! - ✅ Endpoint definitions
//! - ✅ Response parsing
//!
//! ### Phase 2 (TODO): Account Data
//! - ⏳ Auth token generation
//! - ⏳ Account balance
//! - ⏳ Positions
//! - ⏳ Authenticated WebSocket channels
//!
//! ### Phase 3 (TODO): Trading
//! - ⏳ Transaction signing (ECDSA)
//! - ⏳ Nonce management
//! - ⏳ Order creation/cancellation
//! - ⏳ Order status queries
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::lighter::LighterConnector;
//! use connectors_v5::core::{AccountType, Symbol};
//!
//! // Create connector (public-only for Phase 1)
//! let connector = LighterConnector::public(false).await?;
//!
//! // Core methods (from traits)
//! let symbol = Symbol::new("ETH", "USDC");
//! let price = connector.get_price(&symbol, AccountType::FuturesCross).await?;
//! let ticker = connector.get_ticker(&symbol, AccountType::FuturesCross).await?;
//! let klines = connector.get_klines(&symbol, "1h", Some(100), AccountType::FuturesCross, None).await?;
//!
//! // Extended methods (Lighter-specific)
//! let trades = connector.get_recent_trades("ETH", AccountType::FuturesCross, Some(50)).await?;
//! let stats = connector.get_exchange_stats().await?;
//! let height = connector.get_current_height().await?;
//! ```
//!
//! ## Market Symbols
//!
//! - **Perpetuals**: `ETH`, `BTC`, `SOL` (quote is USDC)
//! - **Spot**: `ETH/USDC`, `BTC/USDC`
//!
//! ## Notes
//!
//! - Lighter is an orderbook DEX on zkSync
//! - All perpetuals are USDC-margined
//! - Public market data does NOT require authentication
//! - Account data and trading require ECDSA signatures
//! - WebSocket available for real-time data

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{LighterEndpoint, LighterUrls, format_symbol, normalize_symbol, map_kline_interval};
pub use auth::LighterAuth;
pub use parser::LighterParser;
pub use connector::LighterConnector;
pub use websocket::LighterWebSocket;
