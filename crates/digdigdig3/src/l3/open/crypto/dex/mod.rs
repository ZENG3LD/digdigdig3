//! # Decentralized Exchanges (DEX)
//!
//! Hybrid DEXes that provide centralized-style REST/WebSocket APIs
//! while settling on-chain. These don't require direct RPC node connections.

#[cfg(feature = "exchange-lighter")]
pub mod lighter;
#[cfg(feature = "exchange-dydx")]
pub mod dydx;
#[cfg(feature = "exchange-hyperliquid")]
pub mod hyperliquid;
