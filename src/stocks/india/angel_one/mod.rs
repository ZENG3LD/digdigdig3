//! # Angel One SmartAPI Connector
//!
//! Comprehensive trading connector for Indian markets via Angel One (Angel Broking).
//!
//! **Category**: stocks/india
//! **Type**: Full Trading Broker (Market Data + Order Execution + Portfolio Management)
//!
//! ## Features
//!
//! - **REST API**: Yes (https://apiconnect.angelone.in)
//! - **WebSocket V2**: Yes (4 modes: LTP, Quote, Snap Quote, Depth 20)
//! - **Authentication**: Client Code + PIN + TOTP (3-factor, mandatory 2FA)
//! - **Free tier**: Yes (completely FREE for all Angel One clients)
//! - **Cost**: ₹0 monthly (only standard brokerage applies)
//!
//! ## Data Types
//!
//! - **Price data**: Yes (real-time LTP, OHLC, market depth)
//! - **Historical data**: Yes (free for all segments, up to 2000 days)
//! - **Trading**: YES (full order execution for equity, F&O, commodities, currency)
//! - **Portfolio**: YES (holdings, positions, P&L, margins)
//! - **WebSocket**: YES (4 streaming modes including unique Depth 20)
//!
//! ## Markets Supported
//!
//! - **Equity**: NSE, BSE (~7,000 stocks)
//! - **Derivatives**: NFO, BFO (futures + options)
//! - **Commodities**: MCX, NCDEX
//! - **Currency**: CDS (USD/INR, EUR/INR, GBP/INR, JPY/INR)
//! - **Indices**: 120+ indices (NSE, BSE, MCX)
//!
//! ## Rate Limits
//!
//! - **Orders**: 20/sec (place/modify/cancel/GTT)
//! - **Queries**: 10/sec (individual order status, margin calc)
//! - **WebSocket**: 1000 token subscription limit
//!
//! ## Unique Features
//!
//! - **Depth 20**: 20-level order book (unique to Angel One)
//! - **Free Historical Data**: All segments, up to 8000 candles per request
//! - **GTT Orders**: Good Till Triggered (valid 1 year), OCO support
//! - **Margin Calculator**: Pre-trade margin validation
//! - **120+ Indices**: Real-time OHLC for major indices
//!
//! ## Authentication
//!
//! Requires 3 credentials:
//! - Client Code (Angel One account ID)
//! - Client PIN (account password)
//! - TOTP Secret (for 2FA code generation)
//!
//! Three token types returned:
//! - **JWT Token**: REST API authorization
//! - **Refresh Token**: Token renewal without re-login
//! - **Feed Token**: WebSocket authentication
//!
//! Sessions expire at midnight (market close).
//!
//! ## Usage Example
//!
//! ```ignore
//! use connectors_v5::stocks::india::angel_one::AngelOneConnector;
//! use connectors_v5::core::traits::*;
//!
//! // Initialize with credentials
//! let connector = AngelOneConnector::new(
//!     api_key,
//!     client_code,
//!     pin,
//!     totp_secret,
//!     false, // not testnet
//! ).await?;
//!
//! // Market data
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let ticker = connector.get_ticker(symbol, AccountType::Spot).await?;
//! let candles = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot).await?;
//!
//! // Trading
//! let order = connector.market_order(symbol, OrderSide::Buy, 1.0, AccountType::Spot).await?;
//! let positions = connector.get_positions(AccountType::Spot).await?;
//! let balance = connector.get_balance(crate::core::types::BalanceQuery { asset: None, account_type: AccountType::Spot }).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
// mod websocket; // TODO: Implement WebSocket V2 support

pub use endpoints::{AngelOneEndpoint, AngelOneUrls, format_symbol};
pub use auth::AngelOneAuth;
pub use parser::AngelOneParser;
pub use connector::AngelOneConnector;
// pub use websocket::AngelOneWebSocket; // TODO: Implement WebSocket V2 support
