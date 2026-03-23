//! # Decentralized Exchanges (DEX)
//!
//! Hybrid DEXes that provide centralized-style REST/WebSocket APIs
//! while settling on-chain. These don't require direct RPC node connections.

pub mod lighter;
// jupiter extracted to dig2swap crate
pub mod gmx;
pub mod paradex;
pub mod dydx;
