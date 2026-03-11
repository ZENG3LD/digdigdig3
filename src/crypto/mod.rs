//! # Crypto Connectors
//!
//! All cryptocurrency-related connectors organized by type:
//!
//! - **cex/** — Centralized exchanges (Binance, Bybit, OKX, etc.)
//! - **dex/** — Decentralized exchanges with REST/WS APIs (GMX, Jupiter, Lighter, etc.)
//! - **swap/** — On-chain swap protocols requiring RPC connections (Uniswap, Raydium)

pub mod cex;
pub mod dex;
pub mod swap;
