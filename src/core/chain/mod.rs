//! # Chain module
//!
//! Shared chain interaction layer for all on-chain DEX connectors.
//!
//! ## Overview
//!
//! Rather than each on-chain connector creating its own SDK instance,
//! this module defines a `ChainProvider` abstraction that is shared.
//! One `EvmProvider` per RPC endpoint is enough — multiple connectors
//! pointing at Arbitrum all share a single HTTP connection pool.
//!
//! ## Feature flags
//!
//! | Feature | What it unlocks |
//! |---------|----------------|
//! | `onchain-evm` | `EvmProvider` — alloy-backed EVM chain provider |
//! | `onchain-ethereum` | Alias for `onchain-evm` (backward compat) |
//! | `onchain-solana` | `SolanaProvider` — solana-client-backed Solana chain provider |
//! | `onchain-starknet` | `StarkNetProvider` — raw JSON-RPC StarkNet chain provider |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{ChainProvider, ChainFamily};
//!
//! async fn show_height(provider: &dyn ChainProvider) {
//!     let height = provider.get_height().await.unwrap();
//!     println!("height: {height}");
//! }
//! ```

mod provider;
pub use provider::*;

#[cfg(feature = "onchain-evm")]
mod evm;
#[cfg(feature = "onchain-evm")]
pub use evm::*;

#[cfg(feature = "onchain-solana")]
mod solana;
#[cfg(feature = "onchain-solana")]
pub use solana::*;

#[cfg(feature = "onchain-cosmos")]
pub mod cosmos;
#[cfg(feature = "onchain-cosmos")]
pub use cosmos::{CosmosChain, CosmosProvider};

#[cfg(feature = "onchain-starknet")]
mod starknet_chain;
#[cfg(feature = "onchain-starknet")]
pub use starknet_chain::{StarkNetChain, StarkNetProvider};
