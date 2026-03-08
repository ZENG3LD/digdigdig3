//! # Raydium DEX Connector
//!
//! Connector for Raydium AMM DEX on Solana.
//!
//! ## Architecture
//!
//! - `endpoints` - API URLs and endpoint enum
//! - `auth` - Authentication (no-op - public APIs)
//! - `parser` - JSON response parsing
//! - `connector` - RaydiumConnector + trait implementations
//! - `websocket` - WebSocket stub (not available - use gRPC)
//!
//! ## Important Notes
//!
//! Raydium is fundamentally different from CEX exchanges:
//! - **DEX Architecture**: On-chain AMM, not centralized order book
//! - **No Authentication**: All REST APIs are public
//! - **Symbol Format**: Uses Solana mint addresses (Base58), not ticker symbols
//! - **No WebSocket**: Use Solana Geyser gRPC for real-time data
//! - **AMM Pools**: No traditional orderbook, constant product formula
//!
//! ## Symbol Format
//!
//! Symbols use Solana token mint addresses:
//! - Base: SOL mint (`So11111111111111111111111111111111111111112`)
//! - Quote: USDC mint (`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`)
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::raydium::RaydiumConnector;
//! use connectors_v5::core::{Symbol, AccountType};
//!
//! let connector = RaydiumConnector::new(false).await?; // mainnet
//!
//! // Create symbol with mint addresses
//! let sol_usdc = Symbol::new(
//!     "So11111111111111111111111111111111111111112", // SOL
//!     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
//! );
//!
//! // Get price
//! let price = connector.get_price(sol_usdc.clone(), AccountType::Spot).await?;
//! let ticker = connector.get_ticker(sol_usdc, AccountType::Spot).await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{RaydiumEndpoint, RaydiumUrls, well_known_mints, validate_solana_address};
pub use auth::RaydiumAuth;
pub use parser::RaydiumParser;
pub use connector::RaydiumConnector;
pub use websocket::RaydiumWebSocket;
