//! # Chain event decoders
//!
//! Raw-RPC-response-to-typed-event decoding layer for all supported chains.
//!
//! No external ABI decoder or chain SDK dependencies — all decoding is
//! implemented as manual JSON/hex parsing using `serde_json::Value`.
//!
//! ## Feature gates
//!
//! | Feature | Decoder |
//! |---------|---------|
//! | `onchain-evm` | [`EvmEventDecoder`] |
//! | `onchain-bitcoin` | [`BitcoinEventDecoder`] |
//! | `onchain-solana` | [`SolanaEventDecoder`] |
//! | `onchain-cosmos` | [`CosmosEventDecoder`] |

#[cfg(feature = "onchain-evm")]
mod evm_decoder;
#[cfg(feature = "onchain-evm")]
pub use evm_decoder::{
    decode_address, decode_address_from_data, decode_uint256, topics, EvmEventDecoder,
};

#[cfg(feature = "onchain-bitcoin")]
pub mod bitcoin_decoder;
#[cfg(feature = "onchain-bitcoin")]
pub use bitcoin_decoder::BitcoinEventDecoder;

#[cfg(feature = "onchain-solana")]
pub mod solana_decoder;
#[cfg(feature = "onchain-solana")]
pub use solana_decoder::SolanaEventDecoder;

#[cfg(feature = "onchain-cosmos")]
pub mod cosmos_decoder;
#[cfg(feature = "onchain-cosmos")]
pub use cosmos_decoder::CosmosEventDecoder;
