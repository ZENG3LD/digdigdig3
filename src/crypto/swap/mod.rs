//! # On-Chain Swap Protocols
//!
//! Direct on-chain swap/AMM protocols requiring blockchain RPC connections.
//! Unlike DEXes, these interact directly with smart contracts via RPC nodes.

#[cfg(feature = "onchain-evm")]
pub mod uniswap;
pub mod raydium;
