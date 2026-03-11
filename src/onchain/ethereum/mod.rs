//! # Ethereum On-Chain
//!
//! Direct Ethereum blockchain connections via RPC nodes.
//! Uniswap requires the `onchain-ethereum` feature (for alloy dependency).

#[cfg(feature = "onchain-ethereum")]
pub mod uniswap;
pub mod etherscan;
