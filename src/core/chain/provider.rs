//! # ChainProvider — base trait for all chain providers
//!
//! Object-safe, no SDK-specific types in signatures.
//! All addresses and hashes are plain `String` / `&str`.

use async_trait::async_trait;

use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// CHAIN FAMILY
// ═══════════════════════════════════════════════════════════════════════════════

/// Identifies which chain family a provider connects to.
///
/// Used by connectors to assert that the provider they receive at runtime
/// matches the chain they were built for. For example, `GmxConnector` will
/// panic (or return an error) if handed a `ChainFamily::Solana` provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChainFamily {
    /// EVM-compatible chain (Ethereum, Arbitrum, Optimism, Base, Polygon, etc.).
    ///
    /// `chain_id` uniquely identifies the specific network:
    /// - 1   = Ethereum Mainnet
    /// - 42161 = Arbitrum One
    /// - 10  = Optimism
    /// - 8453 = Base
    /// - 137 = Polygon PoS
    Evm { chain_id: u64 },

    /// Solana mainnet or devnet.
    Solana,

    /// Cosmos SDK chain (dYdX, Osmosis, Cosmos Hub, etc.).
    ///
    /// `chain_id` is the bech32 chain identifier string, e.g. `"dydx-mainnet-1"`.
    Cosmos { chain_id: String },

    /// StarkNet L2.
    StarkNet,
}

impl ChainFamily {
    /// Human-readable name for logging and error messages.
    pub fn name(&self) -> String {
        match self {
            Self::Evm { chain_id } => format!("evm:{chain_id}"),
            Self::Solana => "solana".to_string(),
            Self::Cosmos { chain_id } => format!("cosmos:{chain_id}"),
            Self::StarkNet => "starknet".to_string(),
        }
    }

    /// Returns `true` if this is an EVM chain with the given `chain_id`.
    pub fn is_evm(&self, chain_id: u64) -> bool {
        matches!(self, Self::Evm { chain_id: id } if *id == chain_id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TX STATUS
// ═══════════════════════════════════════════════════════════════════════════════

/// Transaction status returned by [`ChainProvider::get_tx_status`].
#[derive(Debug, Clone)]
pub enum TxStatus {
    /// Transaction is in the mempool but not yet included in a block.
    Pending,

    /// Transaction was included in a block and has at least one confirmation.
    Confirmed {
        /// Block height at which the transaction was included.
        block: u64,
    },

    /// Transaction was included in a block but execution reverted.
    Failed {
        /// Revert reason or error message from the chain.
        reason: String,
    },

    /// No transaction with this hash is known to the node.
    NotFound,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CHAIN PROVIDER TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Base trait for all chain providers.
///
/// Object-safe: no generics in method signatures, no associated types.
/// All chain-specific SDK types stay behind the concrete implementation.
///
/// ## Addresses
///
/// Address format is chain-specific:
/// - EVM: hex string with `0x` prefix, e.g. `"0xabc...def"`
/// - Solana: base58 pubkey, e.g. `"So11111111111111111111111111111111111111112"`
/// - Cosmos: bech32, e.g. `"dydx1abc...xyz"`
/// - StarkNet: hex felt, e.g. `"0x04abc..."`
///
/// ## Transaction bytes
///
/// `broadcast_tx` accepts raw signed transaction bytes.
/// How those bytes are produced (signing, encoding) is the connector's
/// responsibility, not the provider's.
///
/// ## Balance units
///
/// `get_native_balance` returns the balance in the chain's **smallest unit**
/// as a decimal string (no floating-point loss):
/// - EVM: Wei (18 decimals)
/// - Solana: Lamports (9 decimals)
/// - Cosmos: uATOM / udydx / etc. (6 decimals)
#[async_trait]
pub trait ChainProvider: Send + Sync {
    /// Which chain family (and chain ID) this provider connects to.
    fn chain_family(&self) -> ChainFamily;

    /// Broadcast a pre-signed transaction.
    ///
    /// `tx_bytes` — ABI-encoded EVM tx, serialized Solana transaction,
    /// Cosmos proto-encoded `TxRaw`, or StarkNet invoke bytes.
    ///
    /// Returns the transaction hash as a hex string (with `0x` for EVM,
    /// base58 for Solana, etc.).
    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError>;

    /// Current best block / slot / sequence height.
    async fn get_height(&self) -> Result<u64, ExchangeError>;

    /// Nonce / sequence number for the given address.
    ///
    /// For EVM this is `eth_getTransactionCount` (pending nonce).
    /// For Solana this returns the current slot (callers use `get_latest_blockhash`
    /// separately; `get_nonce` here is a best-effort slot approximation).
    /// For Cosmos this is the account sequence number.
    async fn get_nonce(&self, address: &str) -> Result<u64, ExchangeError>;

    /// Native token balance in the chain's smallest unit, returned as a decimal string.
    ///
    /// See trait-level docs for unit conventions.
    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError>;

    /// Transaction status by hash.
    ///
    /// Hash format is chain-specific (see address conventions in trait docs).
    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError>;
}
