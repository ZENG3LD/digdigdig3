# Design: ChainProvider Architecture

**Date:** 2026-03-14
**Status:** Design only — no implementation yet.
**Scope:** Shared chain interaction layer for all on-chain DEX connectors.

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Design Decisions](#2-design-decisions)
3. [Trait Hierarchy](#3-trait-hierarchy)
4. [Type Definitions](#4-type-definitions)
5. [Concrete Provider Structs](#5-concrete-provider-structs)
6. [Module Structure](#6-module-structure)
7. [Feature Gating Plan](#7-feature-gating-plan)
8. [How Connectors Use Providers](#8-how-connectors-use-providers)
9. [Migration Plan](#9-migration-plan)
10. [Implementation Order](#10-implementation-order)

---

## 1. Problem Statement

Today, each on-chain connector creates its own chain SDK instance independently:

- `GmxOnchain` builds its own `DynProvider<Ethereum>` via alloy's `ProviderBuilder`
- `UniswapOnchain` does the same, creating a second HTTP connection pool to Ethereum RPC
- Jupiter and Raydium have no SDK at all — both need `SolanaProvider`
- dYdX has `tonic::Channel` (gRPC) but no tx-building layer
- Lighter needs ECDSA signing but has no signing integration yet

**Concrete problems this causes:**

1. **Multiple RPC connections to the same chain.** GMX on Arbitrum and any future Arbitrum connector each open their own HTTP connection pool to the RPC endpoint. One pool is enough.

2. **No nonce/sequence coordination.** On Cosmos (dYdX), the account sequence number must be monotonically increasing across all transactions. With per-connector instances, two concurrent order placements from two connector instances would produce a sequence conflict — one transaction will be rejected. This is a correctness issue, not just efficiency.

3. **Rate limit blindness.** Public EVM RPCs enforce rate limits (e.g., Infura: 100k req/day free tier, Alchemy: 300M compute units/month). Each connector is blind to what the other connectors are spending. A shared provider can throttle all consumers against a single counter.

4. **Config duplication.** The Arbitrum RPC URL, chain ID, and connection options are repeated in every EVM connector. Changing the RPC provider means patching N connectors.

5. **Compile time bloat.** Without feature gating at the provider layer, every binary that includes any on-chain connector pulls in all chain SDKs.

---

## 2. Design Decisions

### 2.1 Trait hierarchy: base + chain-specific extensions

**Decision:** Use a two-level trait hierarchy.

- `ChainProvider` — common denominator for all chains. Covers the handful of operations that are genuinely universal: broadcast a signed transaction, get the current nonce/sequence for an address, estimate fees, query the current block/slot/height.

- `EvmProvider`, `SolanaProvider`, `CosmosProvider`, `StarkNetProvider` — chain-specific extension traits. These carry associated types and methods that are meaningful only for that chain family (e.g., EVM `eth_call` for contract reads; Solana account data queries; Cosmos module queries).

**Why not a single monolithic trait with all methods defaulting to `UnsupportedOperation`?**

The `operations.rs` pattern (see `CopyTrading`, `EarnStaking`, etc.) works well for optional *exchange capabilities* because the capabilities are additive and every connector can opt into any of them. Chain families are not additive — an EVM chain cannot execute a Solana program account query any more than a Solana account can receive an EVM `eth_call`. A single trait with 40 methods, 38 of which return `UnsupportedOperation` for any given chain, is noise that obscures the real interface.

**Why not pure generics with no trait objects?**

`Arc<dyn ChainProvider>` (dynamic dispatch) is the right choice here over `Arc<impl ChainProvider>` for two reasons:

1. Connectors store their provider as a field. With a generic `struct GmxConnector<P: EvmProvider>`, the connector's type becomes infectious — you can't put `Box<dyn CoreConnector>` into a `Vec` without also erasing the `P`. Dynamic dispatch at the provider boundary lets the connector itself remain non-generic and implement `dyn CoreConnector` cleanly.

2. Providers are created at startup from configuration and injected. The construction site knows the concrete type; the usage sites need not.

The chain-specific extension traits (`EvmChain`, `SolanaChain`, etc.) are used as concrete types in connector fields, not as trait objects, so they can also be used with static dispatch when the connector is not itself boxed.

### 2.2 Signing is outside the provider

The provider's role is **transport**: submit signed bytes, read chain state. Signing is a per-connector concern because each DEX has its own transaction format and signing domain:

- GMX signs an alloy `TypedTransaction` using EIP-1559 with a local private key
- Paradex signs a JSON payload with StarkNet ECDSA for JWT auth (not an on-chain tx)
- dYdX signs a Cosmos `TxRaw` protobuf with secp256k1
- Jupiter receives a partially-built Solana transaction from the REST API and signs it with an `ed25519` keypair

These are not interchangeable. Injecting a signer into the provider would require the provider to know each DEX's tx format, which inverts the dependency. Signing stays in the connector or in a separate `Signer` struct next to the connector.

The one exception is a `sign_and_send()` convenience method on the provider for callers who *do* want a single-call interface. This is opt-in on the concrete types (not on the trait) and accepts the chain's native signer type.

### 2.3 Where it lives: `src/core/chain/`

The provider layer sits in `src/core/chain/` — a sibling of `traits/`, `types/`, and `utils/`. This makes it a first-class part of the connector infrastructure, not an exchange-specific module. Connectors import it from `crate::core::chain::EvmChain` etc.

### 2.4 Feature gating: one feature per chain family

```
onchain-evm      → alloy
onchain-solana   → solana-sdk + solana-client
onchain-cosmos   → cosmrs + tendermint-rpc
onchain-starknet → starknet-crypto (minimal) or starknet (full)
```

These are the only new feature flags added. The existing `onchain-ethereum` flag is aliased to `onchain-evm` for backward compatibility.

---

## 3. Trait Hierarchy

### 3.1 `ChainProvider` — base trait

```rust
use async_trait::async_trait;
use crate::core::types::{ExchangeError, ExchangeResult};

/// Common denominator for all blockchain interaction providers.
///
/// This trait covers the minimal set of operations that are
/// meaningful across all supported chain families:
/// - Submitting a signed raw transaction
/// - Querying the current block/slot/height
/// - Querying the nonce or sequence number for an address
///
/// Chain-specific capabilities (contract reads, account data queries,
/// gRPC module queries, etc.) are exposed on the concrete provider
/// types or their family-specific extension traits.
///
/// # Object safety
///
/// `ChainProvider` is object-safe. Connectors can store `Arc<dyn ChainProvider>`
/// when they only need the base interface, or `Arc<EvmChain>` (concrete type)
/// when they need EVM-specific methods.
#[async_trait]
pub trait ChainProvider: Send + Sync + 'static {
    /// Human-readable chain name for logging and error messages.
    ///
    /// Examples: `"ethereum"`, `"arbitrum"`, `"solana"`, `"dydx"`.
    fn chain_name(&self) -> &str;

    /// Numeric chain identifier where applicable.
    ///
    /// For EVM chains: EIP-155 chain ID (1 = Ethereum, 42161 = Arbitrum, etc.)
    /// For Solana: `None` (Solana does not use numeric chain IDs)
    /// For Cosmos: `None` (Cosmos uses a string chain ID via `cosmos_chain_id()`)
    fn chain_id(&self) -> Option<u64>;

    /// Broadcast a signed, serialized transaction to the network.
    ///
    /// `raw_tx` is the chain-native serialized bytes:
    /// - EVM: RLP-encoded signed `TypedTransaction`
    /// - Solana: bincode-serialized `Transaction` struct
    /// - Cosmos: protobuf-encoded `TxRaw`
    ///
    /// Returns a transaction hash as an opaque hex string. The format
    /// is chain-specific (0x-prefixed for EVM, base58 for Solana,
    /// uppercase hex for Cosmos).
    async fn broadcast_tx(&self, raw_tx: &[u8]) -> ExchangeResult<String>;

    /// Get the current block number / slot / height.
    ///
    /// Returns a `u64` regardless of chain (Solana slot, EVM block number,
    /// Cosmos block height — all fit in u64 for any foreseeable future).
    async fn get_height(&self) -> ExchangeResult<u64>;

    /// Get the next transaction nonce or sequence number for `address`.
    ///
    /// - EVM: pending nonce from `eth_getTransactionCount`
    /// - Solana: not applicable — Solana uses recent blockhashes, not nonces.
    ///   Returns `Err(ExchangeError::UnsupportedOperation("nonce"))`.
    /// - Cosmos: account sequence number from `auth` module query
    ///
    /// `address` is the string representation native to the chain
    /// (0x-prefixed for EVM, base58 for Solana, bech32 for Cosmos).
    async fn get_nonce(&self, address: &str) -> ExchangeResult<u64>;

    /// Get the native token balance for `address`.
    ///
    /// Returns the balance in the chain's smallest unit (wei for EVM,
    /// lamports for Solana, uatom/adydx for Cosmos).
    async fn get_native_balance(&self, address: &str) -> ExchangeResult<u128>;

    /// Get the transaction status for a given transaction hash.
    ///
    /// Returns `TxStatus::Pending`, `TxStatus::Confirmed { block }`,
    /// or `TxStatus::Failed { reason }`.
    async fn get_tx_status(&self, tx_hash: &str) -> ExchangeResult<TxStatus>;
}

/// Status of a broadcast transaction.
#[derive(Debug, Clone)]
pub enum TxStatus {
    /// In the mempool, not yet included in a block.
    Pending,
    /// Included in a block. `block` is the block number/slot/height.
    Confirmed { block: u64 },
    /// Included in a block but execution failed (EVM revert, Cosmos error code, etc.)
    Failed { reason: String },
    /// Transaction hash is unknown to the node queried.
    NotFound,
}
```

### 3.2 `EvmChain` — EVM extension trait

```rust
#[cfg(feature = "onchain-evm")]
use alloy::primitives::{Address, Bytes, U256};
#[cfg(feature = "onchain-evm")]
use alloy::rpc::types::eth::{TransactionRequest, TransactionReceipt};

/// EVM-specific chain operations.
///
/// Implemented by `EvmProvider`. Provides the full EVM JSON-RPC surface
/// needed by connectors: `eth_call`, `eth_estimateGas`, `eth_gasPrice`,
/// and full receipt retrieval.
///
/// All EVM chains — Ethereum, Arbitrum, Optimism, Base, Polygon, BSC,
/// Avalanche — use the same implementation. The chain is selected by
/// the RPC URL and chain ID passed to `EvmProvider::new()`.
///
/// This trait is NOT object-safe due to the use of `alloy` types in
/// signatures. Use the concrete `EvmProvider` type in connector fields.
#[cfg(feature = "onchain-evm")]
#[async_trait]
pub trait EvmChain: ChainProvider {
    /// Execute a read-only contract call (`eth_call`).
    ///
    /// `call` is a `TransactionRequest` with `to` and `input` populated.
    /// `from` and `value` are optional.
    ///
    /// Returns the raw return bytes. The caller decodes them according
    /// to the ABI of the function called.
    async fn eth_call(&self, call: TransactionRequest) -> ExchangeResult<Bytes>;

    /// Estimate gas for a transaction (`eth_estimateGas`).
    async fn estimate_gas(&self, call: TransactionRequest) -> ExchangeResult<u64>;

    /// Get the current base fee per gas (`eth_gasPrice` or EIP-1559 baseFee).
    ///
    /// Returns the base fee in wei as `U256`.
    async fn gas_price(&self) -> ExchangeResult<U256>;

    /// Get the EIP-1559 max priority fee per gas suggestion.
    ///
    /// Returns the suggested priority fee (tip) in wei.
    async fn max_priority_fee(&self) -> ExchangeResult<U256>;

    /// Get the full transaction receipt once confirmed.
    ///
    /// Returns `None` if the tx is pending or unknown.
    async fn get_receipt(&self, tx_hash: &str) -> ExchangeResult<Option<TransactionReceipt>>;

    /// Read the ERC-20 `balanceOf(address)` for `token` contract.
    ///
    /// Convenience wrapper over `eth_call` for the most common
    /// on-chain read operation in DeFi connectors.
    async fn erc20_balance(&self, token: Address, account: Address) -> ExchangeResult<U256>;
}
```

### 3.3 `SolanaChain` — Solana extension trait

```rust
#[cfg(feature = "onchain-solana")]
use solana_sdk::pubkey::Pubkey;
#[cfg(feature = "onchain-solana")]
use serde_json::Value;

/// Solana-specific chain operations.
///
/// Implemented by `SolanaProvider`. Provides the Solana RPC surface
/// needed by Jupiter and Raydium connectors: account data queries,
/// recent blockhash retrieval, and simulation.
///
/// This trait is NOT object-safe due to Solana SDK types. Use the
/// concrete `SolanaProvider` type in connector fields.
#[cfg(feature = "onchain-solana")]
#[async_trait]
pub trait SolanaChain: ChainProvider {
    /// Get the most recent blockhash, needed to finalize a transaction
    /// before signing.
    ///
    /// Solana transactions embed a recent blockhash and expire if not
    /// confirmed within ~150 slots (~60 seconds). Always fetch a fresh
    /// blockhash immediately before signing.
    async fn get_recent_blockhash(&self) -> ExchangeResult<solana_sdk::hash::Hash>;

    /// Get raw account data for a public key.
    ///
    /// Returns the raw bytes stored in the account. Returns `None` if
    /// the account does not exist or has zero lamports.
    async fn get_account_data(&self, pubkey: &Pubkey) -> ExchangeResult<Option<Vec<u8>>>;

    /// Simulate a transaction and return logs without broadcasting.
    ///
    /// Useful for pre-flight checks before spending lamports on a
    /// failed transaction. Returns simulation logs as a JSON value
    /// to avoid coupling callers to a specific Solana SDK version.
    async fn simulate_transaction(&self, raw_tx: &[u8]) -> ExchangeResult<Value>;

    /// Get the SPL token balance for a token account address.
    ///
    /// `token_account` is the associated token account address (ATA),
    /// not the wallet address. Returns the balance as a `u64` in the
    /// token's smallest unit.
    async fn spl_token_balance(&self, token_account: &Pubkey) -> ExchangeResult<u64>;
}
```

### 3.4 `CosmosChain` — Cosmos extension trait

```rust
/// Cosmos-specific chain operations.
///
/// Implemented by `CosmosProvider`. Provides Cosmos SDK services
/// needed by the dYdX connector: account queries (for sequence numbers),
/// balance queries, and transaction simulation via gRPC.
///
/// The provider maintains a single `tonic::Channel` (the gRPC connection)
/// shared across all callers. Cosmos sequence numbers MUST be coordinated
/// centrally to avoid "sequence mismatch" errors — this provider's
/// `get_nonce()` is authoritative and should be called once per tx attempt.
///
/// This trait is NOT object-safe due to `async_trait` restrictions with
/// `prost` message types. Use the concrete `CosmosProvider` type.
#[cfg(feature = "onchain-cosmos")]
#[async_trait]
pub trait CosmosChain: ChainProvider {
    /// The bech32 human-readable part for this chain (e.g., "dydx", "cosmos").
    fn hrp(&self) -> &str;

    /// The string chain ID (e.g., "dydx-mainnet-1").
    fn cosmos_chain_id(&self) -> &str;

    /// The fee denom for this chain (e.g., "adydx", "uatom").
    fn fee_denom(&self) -> &str;

    /// Simulate a `TxRaw` transaction and return estimated gas.
    ///
    /// Calls `cosmos.tx.v1beta1.Service/SimulateTx`. The returned gas
    /// estimate should be multiplied by a safety factor (1.3–1.5) before
    /// setting the `fee.gas_limit` in the real transaction.
    async fn simulate_tx(&self, tx_raw_bytes: &[u8]) -> ExchangeResult<u64>;

    /// Query the account number and current sequence for a bech32 address.
    ///
    /// Returns `(account_number, sequence)`. Both are required when
    /// constructing `SignerInfo` for a Cosmos transaction.
    ///
    /// This is distinct from the base `get_nonce()` (which returns sequence
    /// only) and should be used for full tx building where account_number
    /// is also needed.
    async fn get_account_info(&self, address: &str) -> ExchangeResult<(u64, u64)>;
}
```

### 3.5 `StarkNetChain` — StarkNet extension trait

```rust
/// StarkNet-specific chain operations.
///
/// Implemented by `StarkNetProvider`. For the current codebase, Paradex
/// is the only StarkNet connector and it only needs signing (not on-chain
/// reads). This trait covers potential future needs: reading StarkNet
/// contract storage, sending `invoke` transactions, nonce queries.
///
/// Currently `starknet-crypto` is used only for JWT auth signatures —
/// that does NOT require a provider instance and should stay in the
/// connector's auth module. This trait is for if/when an actual
/// StarkNet JSON-RPC provider is needed.
#[cfg(feature = "onchain-starknet")]
#[async_trait]
pub trait StarkNetChain: ChainProvider {
    /// Get the nonce for a StarkNet account contract address.
    ///
    /// StarkNet account nonces are stored in the account contract's
    /// storage, so this is a contract state read.
    async fn get_starknet_nonce(&self, contract_address: &str) -> ExchangeResult<u64>;

    /// Call a StarkNet contract function (read-only, no fee).
    ///
    /// `calldata` is a `Vec<String>` of felt252 values in hex.
    /// Returns a `Vec<String>` of return felts.
    async fn call_contract(
        &self,
        contract_address: &str,
        entry_point_selector: &str,
        calldata: Vec<String>,
    ) -> ExchangeResult<Vec<String>>;
}
```

---

## 4. Type Definitions

These types live in `src/core/chain/types.rs` and are unconditionally compiled (no feature gate needed — they are pure data structs with no SDK dependencies).

```rust
// src/core/chain/types.rs

/// Configuration for connecting to a single chain node.
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// HTTP(S) URL for REST/JSON-RPC endpoint.
    /// For Cosmos: used for tendermint-rpc HTTP client.
    /// For Solana: used for `solana-client` RpcClient.
    /// For EVM: used for alloy HTTP provider.
    pub rpc_url: String,

    /// Optional WebSocket URL for subscription-based connections.
    /// For EVM: used for `eth_subscribe` (new blocks, logs).
    /// For Solana: used for account/slot subscriptions.
    pub ws_url: Option<String>,

    /// Optional gRPC endpoint URL (Cosmos only).
    /// Example: "https://dydx-grpc.lavenderfive.com:443"
    pub grpc_url: Option<String>,

    /// Numeric chain ID (EVM only; ignored for Solana/Cosmos).
    pub chain_id: Option<u64>,

    /// String chain ID (Cosmos only; ignored for EVM/Solana).
    /// Example: "dydx-mainnet-1"
    pub cosmos_chain_id: Option<String>,

    /// Request timeout in milliseconds. Defaults to 30_000.
    pub timeout_ms: u64,

    /// Maximum concurrent requests to the RPC endpoint.
    /// Prevents overwhelming public RPC endpoints.
    pub max_concurrent_requests: usize,
}

impl ChainConfig {
    /// Create a minimal config from an RPC URL with sensible defaults.
    pub fn from_rpc(rpc_url: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            ws_url: None,
            grpc_url: None,
            chain_id: None,
            cosmos_chain_id: None,
            timeout_ms: 30_000,
            max_concurrent_requests: 50,
        }
    }
}

/// Well-known EVM chain IDs and their canonical RPC URLs.
///
/// These are public RPCs for convenience. For production, use a
/// dedicated RPC provider (Alchemy, Infura, QuickNode, etc.).
pub struct EvmChainPreset;

impl EvmChainPreset {
    pub fn ethereum_mainnet() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://ethereum-rpc.publicnode.com".to_string(),
            chain_id: Some(1),
            ..ChainConfig::from_rpc("")
        }
    }

    pub fn arbitrum_one() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            chain_id: Some(42161),
            ..ChainConfig::from_rpc("")
        }
    }

    pub fn avalanche_c_chain() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://api.avax.network/ext/bc/C/rpc".to_string(),
            chain_id: Some(43114),
            ..ChainConfig::from_rpc("")
        }
    }

    pub fn base_mainnet() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://mainnet.base.org".to_string(),
            chain_id: Some(8453),
            ..ChainConfig::from_rpc("")
        }
    }

    pub fn polygon_mainnet() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://polygon-rpc.com".to_string(),
            chain_id: Some(137),
            ..ChainConfig::from_rpc("")
        }
    }
}

/// Well-known Solana cluster configurations.
pub struct SolanaClusterPreset;

impl SolanaClusterPreset {
    pub fn mainnet_beta() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: Some("wss://api.mainnet-beta.solana.com".to_string()),
            ..ChainConfig::from_rpc("")
        }
    }

    pub fn devnet() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: Some("wss://api.devnet.solana.com".to_string()),
            ..ChainConfig::from_rpc("")
        }
    }
}

/// Well-known Cosmos chain configurations.
pub struct CosmosChainPreset;

impl CosmosChainPreset {
    pub fn dydx_mainnet() -> ChainConfig {
        ChainConfig {
            rpc_url: "https://dydx-rpc.lavenderfive.com:443".to_string(),
            grpc_url: Some("https://dydx-grpc.lavenderfive.com:443".to_string()),
            cosmos_chain_id: Some("dydx-mainnet-1".to_string()),
            ..ChainConfig::from_rpc("")
        }
    }
}
```

---

## 5. Concrete Provider Structs

### 5.1 `EvmProvider`

```rust
// src/core/chain/evm.rs
#![cfg(feature = "onchain-evm")]

use std::sync::Arc;
use alloy::network::Ethereum;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::rpc::types::eth::{TransactionRequest, TransactionReceipt};
use async_trait::async_trait;

use crate::core::types::{ExchangeError, ExchangeResult};
use super::types::ChainConfig;
use super::traits::{ChainProvider, EvmChain, TxStatus};

/// Shared EVM chain provider wrapping an alloy HTTP provider.
///
/// One instance of `EvmProvider` is typically created per chain (one for
/// Arbitrum, one for Ethereum mainnet, etc.) and shared across all
/// connectors on that chain via `Arc<EvmProvider>`.
///
/// The internal alloy provider manages its own HTTP connection pool.
/// `EvmProvider` adds:
/// - Chain identity (`chain_name`, `chain_id`)
/// - `erc20_balance` convenience wrapper
/// - `sign_and_send()` optional method (not on the trait) for callers
///   who manage key material locally
///
/// # Thread safety
///
/// `EvmProvider` is `Send + Sync`. The alloy `DynProvider` is `Clone`
/// (cheap Arc clone). Multiple tokio tasks can call into the same
/// `EvmProvider` concurrently without coordination.
pub struct EvmProvider {
    provider: DynProvider<Ethereum>,
    config: ChainConfig,
    name: String,
}

impl EvmProvider {
    /// Create a new provider from a `ChainConfig`.
    ///
    /// Panics if `config.rpc_url` is not a valid URL. Callers should
    /// validate the URL before calling this.
    pub fn new(name: impl Into<String>, config: ChainConfig) -> ExchangeResult<Self> {
        let url: reqwest::Url = config.rpc_url.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid RPC URL '{}': {}", config.rpc_url, e))
        })?;

        let inner = ProviderBuilder::new().connect_http(url);
        Ok(Self {
            provider: DynProvider::new(inner),
            config,
            name: name.into(),
        })
    }

    /// Convenience constructor for Ethereum mainnet with the public node RPC.
    pub fn ethereum_mainnet() -> ExchangeResult<Arc<Self>> {
        Ok(Arc::new(Self::new("ethereum", super::types::EvmChainPreset::ethereum_mainnet())?))
    }

    /// Convenience constructor for Arbitrum One.
    pub fn arbitrum() -> ExchangeResult<Arc<Self>> {
        Ok(Arc::new(Self::new("arbitrum", super::types::EvmChainPreset::arbitrum_one())?))
    }

    /// Convenience constructor for Avalanche C-Chain.
    pub fn avalanche() -> ExchangeResult<Arc<Self>> {
        Ok(Arc::new(Self::new("avalanche", super::types::EvmChainPreset::avalanche_c_chain())?))
    }

    /// Access the inner alloy `DynProvider` directly.
    ///
    /// Connectors that need alloy-specific operations not covered by
    /// the `EvmChain` trait can downcast through this accessor.
    /// This escape hatch is intentional — not every possible alloy
    /// operation needs to be wrapped by the trait.
    pub fn inner(&self) -> &DynProvider<Ethereum> {
        &self.provider
    }

    /// Build, sign, and broadcast a transaction using a local private key.
    ///
    /// This convenience method is NOT on the `ChainProvider` trait because
    /// key material handling is per-connector. It is provided on the
    /// concrete struct as an opt-in shortcut.
    ///
    /// `signer` — an alloy `LocalWallet` (or any `alloy::signers::Signer`).
    /// `tx`     — an unsigned `TransactionRequest` built by the connector.
    ///
    /// Returns the transaction hash.
    #[cfg(feature = "onchain-evm")]
    pub async fn sign_and_send(
        &self,
        // The concrete alloy signer types are not listed here to keep
        // the design document SDK-version-agnostic. In implementation,
        // this will accept `impl alloy::signers::Signer`.
        _signer: &dyn std::any::Any,
        _tx: TransactionRequest,
    ) -> ExchangeResult<String> {
        // Implementation: set nonce, gas price, chain_id, sign, RLP-encode, broadcast.
        // See migration notes for gmx/onchain.rs for the alloy v1 signing API.
        todo!("implemented when signing is wired in Phase 1 EVM")
    }
}

#[async_trait]
impl ChainProvider for EvmProvider {
    fn chain_name(&self) -> &str { &self.name }

    fn chain_id(&self) -> Option<u64> { self.config.chain_id }

    async fn broadcast_tx(&self, raw_tx: &[u8]) -> ExchangeResult<String> {
        let pending = self.provider
            .send_raw_transaction(raw_tx)
            .await
            .map_err(|e| ExchangeError::Network(format!("send_raw_transaction: {}", e)))?;
        Ok(format!("{:#x}", pending.tx_hash()))
    }

    async fn get_height(&self) -> ExchangeResult<u64> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_blockNumber: {}", e)))
    }

    async fn get_nonce(&self, address: &str) -> ExchangeResult<u64> {
        let addr: Address = address.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid EVM address '{}': {}", address, e))
        })?;
        let nonce = self.provider
            .get_transaction_count(addr)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getTransactionCount: {}", e)))?;
        Ok(nonce)
    }

    async fn get_native_balance(&self, address: &str) -> ExchangeResult<u128> {
        let addr: Address = address.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid EVM address '{}': {}", address, e))
        })?;
        let balance = self.provider
            .get_balance(addr)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getBalance: {}", e)))?;
        Ok(balance.to::<u128>())
    }

    async fn get_tx_status(&self, tx_hash: &str) -> ExchangeResult<TxStatus> {
        let hash = tx_hash.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid tx hash '{}': {}", tx_hash, e))
        })?;
        let receipt = self.provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getTransactionReceipt: {}", e)))?;
        match receipt {
            None => Ok(TxStatus::Pending),
            Some(r) => {
                let block = r.block_number.unwrap_or(0);
                if r.status() { Ok(TxStatus::Confirmed { block }) }
                else { Ok(TxStatus::Failed { reason: "reverted".to_string() }) }
            }
        }
    }
}

#[async_trait]
impl EvmChain for EvmProvider {
    async fn eth_call(&self, call: TransactionRequest) -> ExchangeResult<Bytes> {
        self.provider
            .call(call)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_call: {}", e)))
    }

    async fn estimate_gas(&self, call: TransactionRequest) -> ExchangeResult<u64> {
        self.provider
            .estimate_gas(call)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_estimateGas: {}", e)))
    }

    async fn gas_price(&self) -> ExchangeResult<U256> {
        self.provider
            .get_gas_price()
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_gasPrice: {}", e)))
    }

    async fn max_priority_fee(&self) -> ExchangeResult<U256> {
        self.provider
            .get_max_priority_fee_per_gas()
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_maxPriorityFeePerGas: {}", e)))
    }

    async fn get_receipt(&self, tx_hash: &str) -> ExchangeResult<Option<TransactionReceipt>> {
        let hash = tx_hash.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid tx hash '{}': {}", tx_hash, e))
        })?;
        self.provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getTransactionReceipt: {}", e)))
    }

    async fn erc20_balance(&self, token: Address, account: Address) -> ExchangeResult<U256> {
        // ERC-20 balanceOf(address) selector: 0x70a08231
        const BALANCE_OF: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];
        let mut calldata = Vec::with_capacity(36);
        calldata.extend_from_slice(&BALANCE_OF);
        let mut padded = [0u8; 32];
        padded[12..].copy_from_slice(account.as_slice());
        calldata.extend_from_slice(&padded);

        let tx = TransactionRequest::default()
            .to(token)
            .input(Bytes::from(calldata).into());

        let result = self.eth_call(tx).await?;
        if result.len() < 32 {
            return Err(ExchangeError::Parse(format!(
                "erc20_balance: returned {} bytes, expected 32",
                result.len()
            )));
        }
        Ok(U256::from_be_slice(&result[..32]))
    }
}
```

### 5.2 `SolanaProvider` (sketch)

```rust
// src/core/chain/solana.rs  (cfg(feature = "onchain-solana"))

/// Shared Solana RPC client.
///
/// Wraps `solana_client::rpc_client::RpcClient` with the `ChainProvider`
/// and `SolanaChain` interfaces. One instance covers both mainnet-beta
/// and devnet — the cluster is set by the URL in `ChainConfig`.
///
/// # Sequence numbers (blockhashes)
///
/// Solana does not use nonces. `get_nonce()` returns `UnsupportedOperation`.
/// Instead, call `get_recent_blockhash()` immediately before signing.
/// Blockhashes expire after ~150 slots (~60 seconds).
pub struct SolanaProvider {
    // solana_client::rpc_client::RpcClient (blocking) or
    // solana_client::nonblocking::rpc_client::RpcClient (async)
    // Use the nonblocking variant to avoid blocking tokio.
    client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    config: ChainConfig,
}
```

### 5.3 `CosmosProvider` (sketch)

```rust
// src/core/chain/cosmos.rs  (cfg(feature = "onchain-cosmos"))

/// Shared Cosmos gRPC channel + sequence number state.
///
/// Wraps a `tonic::Channel` that is already used by the dYdX connector's
/// `GrpcClient`. The existing `crate::core::grpc::GrpcClient` does raw
/// channel management. `CosmosProvider` sits on top of it and adds:
/// - `CosmosChain` trait implementation
/// - Sequence number caching per bech32 address (avoids repeated RPC calls)
/// - `MsgPlaceOrder` / `MsgCancelOrder` builder helpers (not on the trait
///   — these live in `src/crypto/dex/dydx/tx_builder.rs`)
///
/// # Sequence synchronization
///
/// Cosmos rejects txs with wrong sequence numbers. The provider serializes
/// sequence increments with a `Mutex<HashMap<String, u64>>` keyed by
/// bech32 address. On each tx submission:
/// 1. Lock the mutex, read current sequence
/// 2. Build and broadcast tx with that sequence
/// 3. On success: increment and release lock
/// 4. On `sequence mismatch` error: refresh from chain, retry
pub struct CosmosProvider {
    channel: tonic::transport::Channel,
    config: ChainConfig,
    /// Cached sequence numbers per address. Mutex not RwLock — writes happen
    /// on every successful tx, so lock contention is expected.
    sequences: Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
}
```

### 5.4 `StarkNetProvider` (sketch)

```rust
// src/core/chain/starknet.rs  (cfg(feature = "onchain-starknet"))

/// StarkNet JSON-RPC provider.
///
/// Currently a stub. Paradex does NOT need this — it uses
/// `starknet-crypto` for signing only (in `paradex/auth.rs`),
/// with no on-chain reads.
///
/// This provider is defined for future use if:
/// 1. Paradex needs to read on-chain positions/balances
/// 2. A second StarkNet DEX connector is added
///
/// Implementation would wrap `starknet::providers::JsonRpcClient`
/// from the `starknet-providers` sub-crate (requires upgrading from
/// `starknet-crypto` only to the full `starknet` meta-crate behind
/// the `onchain-starknet-full` feature).
pub struct StarkNetProvider {
    config: ChainConfig,
    // Placeholder: starknet::providers::JsonRpcClient<HttpTransport>
}
```

---

## 6. Module Structure

```
src/core/
├── mod.rs                  # Existing; add `pub mod chain;`
├── traits/
├── types/
├── utils/
├── http.rs
├── websocket.rs
├── grpc.rs                 # Existing; dYdX gRPC channel
└── chain/                  # NEW: shared chain provider layer
    ├── mod.rs              # pub use EvmProvider, SolanaProvider, etc.; re-export TxStatus
    ├── traits.rs           # ChainProvider, EvmChain, SolanaChain, CosmosChain, StarkNetChain
    ├── types.rs            # ChainConfig, TxStatus, EvmChainPreset, SolanaClusterPreset, etc.
    ├── evm.rs              # EvmProvider  [cfg(onchain-evm)]
    ├── solana.rs           # SolanaProvider  [cfg(onchain-solana)]
    ├── cosmos.rs           # CosmosProvider  [cfg(onchain-cosmos)]
    └── starknet.rs         # StarkNetProvider  [cfg(onchain-starknet)]
```

`src/core/mod.rs` additions:

```rust
pub mod chain;

// Re-export the types most connectors need
pub use chain::types::{ChainConfig, TxStatus};
#[cfg(feature = "onchain-evm")]
pub use chain::evm::EvmProvider;
#[cfg(feature = "onchain-solana")]
pub use chain::solana::SolanaProvider;
#[cfg(feature = "onchain-cosmos")]
pub use chain::cosmos::CosmosProvider;
```

---

## 7. Feature Gating Plan

### 7.1 Cargo.toml changes

```toml
[features]
default = ["onchain-evm"]

# ── Chain provider features (coarse, one per chain family) ──────────────────

# EVM: Ethereum, Arbitrum, Avalanche, Base, Polygon, Optimism, BSC, ...
# Replaces existing "onchain-ethereum" flag. "onchain-ethereum" kept as alias.
onchain-evm = ["dep:alloy"]
onchain-ethereum = ["onchain-evm"]      # backward-compat alias

# Solana: Jupiter, Raydium
onchain-solana = [
    "dep:solana-sdk",
    "dep:solana-client",
]

# Cosmos SDK: dYdX v4 trading
onchain-cosmos = [
    "dep:cosmrs",
    "dep:tendermint-rpc",
    # Note: tonic/prost are already pulled in by the existing "grpc" feature.
    # "grpc" is required alongside "onchain-cosmos" for dYdX.
]

# StarkNet: Paradex (signing only today, full provider in future)
onchain-starknet = ["dep:starknet-crypto"]
onchain-starknet-full = ["onchain-starknet", "dep:starknet"]  # full JSON-RPC

# ── Connector-level features (optional convenience bundles) ─────────────────
# These allow building ONLY the connectors needed.
connector-gmx       = ["onchain-evm"]
connector-uniswap   = ["onchain-evm"]
connector-lighter   = ["dep:k256"]          # k256 signing only, no provider
connector-jupiter   = ["onchain-solana"]
connector-raydium   = ["onchain-solana"]
connector-dydx-full = ["grpc", "onchain-cosmos"]  # full trading (REST + gRPC + tx)
connector-paradex   = ["onchain-starknet"]

[dependencies]
alloy = { version = "1", features = ["provider-ws", "rpc-types"], optional = true }

# Solana (new)
solana-sdk    = { version = "2.1", default-features = false, features = ["std"], optional = true }
solana-client = { version = "2.1", optional = true }

# Cosmos (new)
cosmrs        = { version = "0.22", features = ["rpc", "bip32"], optional = true }
tendermint-rpc = { version = "0.39", features = ["http-client"], optional = true }

# Existing
tonic         = { version = "0.12", features = ["tls"], optional = true }
prost         = { version = "0.13", optional = true }
k256          = { version = "0.13", features = ["ecdsa-core", "ecdsa"], optional = true }
starknet-crypto = { version = "0.6", optional = true }
starknet      = { version = "0.17", default-features = false, features = ["providers"], optional = true }
```

### 7.2 Feature flag principles

1. **The `onchain-*` flags gate the shared provider, not individual connectors.** A connector that imports `EvmProvider` will get a compile error if `onchain-evm` is not enabled. The connector's module uses `#[cfg(feature = "onchain-evm")]` to gate itself entirely.

2. **`connector-*` flags are optional shortcuts** for downstream consumers who want coarse-grained "give me only GMX" control. They are additive and do not replace the `onchain-*` flags.

3. **`default = ["onchain-evm"]`** preserves the current behavior. Any build that uses `default-features = true` (the default) compiles GMX and Uniswap as today.

4. **Solana compile time is the main concern.** `onchain-solana` is NOT in default features. Adding Jupiter or Raydium to a build that needs fast compile cycles requires an explicit opt-in.

5. **`onchain-ethereum` remains valid** as an alias. Existing Cargo.toml files, CI scripts, and documentation using `features = ["onchain-ethereum"]` continue to work.

---

## 8. How Connectors Use Providers

### 8.1 GMX — migrated to `Arc<EvmProvider>`

```rust
// src/crypto/dex/gmx/connector.rs  (after migration)

#[cfg(feature = "onchain-evm")]
use std::sync::Arc;
#[cfg(feature = "onchain-evm")]
use crate::core::chain::EvmProvider;

pub struct GmxConnector {
    http: HttpClient,
    credentials: Option<ExchangeCredentials>,

    /// Shared EVM provider for on-chain operations.
    ///
    /// `None` when built without credentials or when the `onchain-evm`
    /// feature is disabled. On-chain methods return `UnsupportedOperation`
    /// in that case.
    #[cfg(feature = "onchain-evm")]
    pub chain: Option<Arc<EvmProvider>>,
}

impl GmxConnector {
    /// Create a GMX connector with a shared Arbitrum provider.
    ///
    /// If `chain` is `None`, all on-chain trading operations return
    /// `UnsupportedOperation`. REST market data still works.
    #[cfg(feature = "onchain-evm")]
    pub fn with_chain(
        credentials: Option<ExchangeCredentials>,
        chain: Arc<EvmProvider>,
    ) -> Self {
        Self {
            http: HttpClient::new(),
            credentials,
            chain: Some(chain),
        }
    }
}
```

The `GmxOnchain` struct in `onchain.rs` is NOT deleted. It becomes a thin helper that **accepts** an `Arc<EvmProvider>` instead of building its own:

```rust
// src/crypto/dex/gmx/onchain.rs  (after migration)
pub struct GmxOnchain {
    // Before: owned DynProvider<Ethereum>
    // After: borrowed reference to shared provider
    chain: Arc<EvmProvider>,
    chain_name: String,
}

impl GmxOnchain {
    pub fn new(chain: Arc<EvmProvider>) -> Self {
        let chain_name = chain.chain_name().to_string();
        Self { chain, chain_name }
    }

    // arbitrum() and avalanche() become convenience constructors that
    // call EvmProvider::arbitrum() / EvmProvider::avalanche() and wrap:
    pub fn arbitrum() -> ExchangeResult<Self> {
        Ok(Self::new(EvmProvider::arbitrum()?))
    }
}
```

`create_position_onchain()` and `close_position_onchain()` keep their exact signatures — they call `self.chain.inner()` to get the alloy provider when the lower-level alloy types are needed, or use `EvmChain::eth_call()` for reads.

### 8.2 Uniswap — same pattern

```rust
// src/crypto/swap/uniswap/onchain.rs  (after migration)
pub struct UniswapOnchain {
    chain: Arc<EvmProvider>,
    testnet: bool,
}

impl UniswapOnchain {
    pub fn new(chain: Arc<EvmProvider>, testnet: bool) -> Self {
        Self { chain, testnet }
    }

    // Replaces the old self.provider.call() calls:
    pub async fn get_token_balance_onchain(
        &self,
        token_address: &str,
        wallet_address: &str,
    ) -> ExchangeResult<U256> {
        let token = token_address.parse()...;
        let wallet = wallet_address.parse()...;
        // Now delegates to EvmChain::erc20_balance — no duplicated logic:
        self.chain.erc20_balance(token, wallet).await
    }
}
```

**Key benefit:** `erc20_balance` is now implemented once in `EvmProvider` and both GMX and Uniswap use it. The duplicated ABI encoding for `balanceOf` in `uniswap/onchain.rs` is removed.

### 8.3 Jupiter — new `JupiterOnchain`

```rust
// src/crypto/dex/jupiter/onchain.rs  (new file, cfg(onchain-solana))

use std::sync::Arc;
use solana_sdk::{
    transaction::Transaction,
    signature::Keypair,
    signer::Signer,
};
use crate::core::chain::SolanaProvider;
use crate::core::{ExchangeError, ExchangeResult};

/// Handles Solana transaction signing and broadcasting for Jupiter swaps.
///
/// Jupiter's REST API returns a base64-encoded, partially-built Solana
/// transaction. This module:
/// 1. Decodes the base64 transaction
/// 2. Inserts a recent blockhash (from `SolanaProvider`)
/// 3. Signs with the user's keypair
/// 4. Broadcasts via `SolanaProvider::broadcast_tx()`
pub struct JupiterOnchain {
    chain: Arc<SolanaProvider>,
}

impl JupiterOnchain {
    pub fn new(chain: Arc<SolanaProvider>) -> Self {
        Self { chain }
    }

    /// Finalize, sign, and broadcast a Jupiter swap transaction.
    ///
    /// `serialized_tx` — the base64-encoded transaction from Jupiter `/swap`.
    /// `keypair`       — the Solana keypair that will pay and sign.
    ///
    /// Returns the transaction signature (base58 string).
    pub async fn execute_swap(
        &self,
        serialized_tx: &str,
        keypair: &Keypair,
    ) -> ExchangeResult<String> {
        // 1. Decode base64
        let tx_bytes = base64::decode(serialized_tx)
            .map_err(|e| ExchangeError::Parse(format!("base64 decode: {}", e)))?;

        // 2. Deserialize the transaction (bincode)
        let mut tx: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| ExchangeError::Parse(format!("tx deserialize: {}", e)))?;

        // 3. Set recent blockhash
        let blockhash = self.chain.get_recent_blockhash().await?;
        tx.message.recent_blockhash = blockhash;

        // 4. Sign
        tx.sign(&[keypair], blockhash);

        // 5. Re-serialize and broadcast
        let signed_bytes = bincode::serialize(&tx)
            .map_err(|e| ExchangeError::Parse(format!("tx serialize: {}", e)))?;

        self.chain.broadcast_tx(&signed_bytes).await
    }
}
```

### 8.4 dYdX — `CosmosProvider` + sequence coordination

```rust
// src/crypto/dex/dydx/connector.rs  (snippet)

#[cfg(feature = "onchain-cosmos")]
use crate::core::chain::CosmosProvider;

pub struct DydxConnector {
    http: HttpClient,
    ws_client: Option<...>,

    #[cfg(all(feature = "grpc", feature = "onchain-cosmos"))]
    pub cosmos: Option<Arc<CosmosProvider>>,
}

// In place_order():
// BEFORE: return Err(UnsupportedOperation("place_order"))
// AFTER (with onchain-cosmos):
//   let cosmos = self.cosmos.as_ref().ok_or(UnsupportedOperation("cosmos provider not set"))?;
//   let (account_num, seq) = cosmos.get_account_info(&address).await?;
//   let tx_bytes = DydxTxBuilder::place_order(req, account_num, seq, &signing_key)?;
//   let tx_hash = cosmos.broadcast_tx(&tx_bytes).await?;
```

The `DydxTxBuilder` is a new struct in `src/crypto/dex/dydx/tx_builder.rs` that depends on `cosmrs` types. It is gated behind `cfg(feature = "onchain-cosmos")`. It handles building `MsgPlaceOrder` / `MsgCancelOrder` protobuf messages, setting signer info, and serializing to `TxRaw`. This keeps all `cosmrs` coupling inside the dYdX module, not in the shared `CosmosProvider`.

---

## 9. Migration Plan

Migration is designed to be non-breaking: existing connector code compiles and behaves identically before and after the migration.

### Phase 1 — Define `ChainProvider` trait hierarchy (no breaking changes)

- Create `src/core/chain/` directory with all files
- Implement `EvmProvider` (wrapping alloy, same as current `GmxOnchain`/`UniswapOnchain` do)
- All existing connector code continues to compile unchanged — the new module is additive
- No existing `onchain.rs` files are modified yet
- Add `pub mod chain;` to `src/core/mod.rs`

**Deliverable:** `src/core/chain/` compiles. `cargo check --all-features` passes.

### Phase 2 — Migrate `GmxOnchain` to use `Arc<EvmProvider>`

- Change `GmxOnchain.provider: DynProvider<Ethereum>` to `GmxOnchain.chain: Arc<EvmProvider>`
- Update `GmxOnchain::new()` to accept `Arc<EvmProvider>`
- Keep `GmxOnchain::arbitrum()` and `GmxOnchain::avalanche()` as convenience constructors that call `EvmProvider::arbitrum()` / `EvmProvider::avalanche()` internally
- All method signatures on `GmxOnchain` are unchanged — no callers need to change
- Remove the `ProviderBuilder` call from `GmxOnchain::new()` — it moves to `EvmProvider::new()`

**Compile check:** `cargo check --features onchain-evm`

### Phase 3 — Migrate `UniswapOnchain` to use `Arc<EvmProvider>`

- Same pattern as Phase 2
- Replace `self.provider.call(...)` calls with `self.chain.eth_call(...)` from `EvmChain` trait
- Replace duplicated `balanceOf` encoding in `get_token_balance_onchain()` with `self.chain.erc20_balance(token, wallet).await`

**Compile check:** `cargo check --features onchain-evm`

### Phase 4 — Add Solana provider + `JupiterOnchain`

- Add `SolanaProvider` implementation (requires adding `solana-sdk` + `solana-client` to Cargo.toml)
- Create `src/crypto/dex/jupiter/onchain.rs` with `JupiterOnchain`
- Create `src/crypto/swap/raydium/onchain.rs` with `RaydiumOnchain`
- Gate everything behind `onchain-solana` feature
- Jupiter and Raydium REST connectors are unchanged

**Compile check:** `cargo check --features onchain-solana`

### Phase 5 — Add Cosmos provider + dYdX tx builder

- Add `CosmosProvider` implementation (requires adding `cosmrs` to Cargo.toml)
- Create `src/crypto/dex/dydx/tx_builder.rs` with `DydxTxBuilder`
- Wire `DydxConnector::place_order()` and `DydxConnector::cancel_order()` to use tx builder
- Remove `UnsupportedOperation` returns from dYdX trading methods
- Gate behind `onchain-cosmos` + `grpc` features

**Compile check:** `cargo check --features grpc,onchain-cosmos`

### Phase 6 — Wire Lighter k256 signing

- `k256-signing` feature is already in Cargo.toml, already a transitive dep of alloy
- No new dependencies needed
- Create `src/crypto/dex/lighter/tx_builder.rs` using `k256::ecdsa::SigningKey`
- Wire `LighterConnector::place_order()` to sign and POST to `/sendTx`
- This does NOT require `EvmProvider` — Lighter is a zkEVM with its own L2 API, not standard EVM JSON-RPC

**Compile check:** `cargo check --features k256-signing`

---

## 10. Implementation Order

| Order | Component | Feature Gate | Blocks | Complexity |
|-------|-----------|-------------|--------|------------|
| 1 | `src/core/chain/traits.rs` + `types.rs` | none | — | Low |
| 2 | `src/core/chain/evm.rs` (EvmProvider) | `onchain-evm` | — | Low |
| 3 | Migrate `gmx/onchain.rs` to Arc<EvmProvider> | `onchain-evm` | 2 | Low |
| 4 | Migrate `uniswap/onchain.rs` to Arc<EvmProvider> | `onchain-evm` | 2 | Low |
| 5 | Wire Lighter k256 signing | `k256-signing` | — | Medium |
| 6 | `src/core/chain/solana.rs` (SolanaProvider) | `onchain-solana` | — | Medium |
| 7 | `jupiter/onchain.rs` (JupiterOnchain) | `onchain-solana` | 6 | Medium |
| 8 | `raydium/onchain.rs` (RaydiumOnchain) | `onchain-solana` | 6 | Medium |
| 9 | `src/core/chain/cosmos.rs` (CosmosProvider) | `onchain-cosmos` | — | High |
| 10 | `dydx/tx_builder.rs` + wire trading | `onchain-cosmos` | 9 | High |
| 11 | `src/core/chain/starknet.rs` (stub) | `onchain-starknet` | — | Low (stub only) |

**Critical path:** Steps 1–4 can be done sequentially in one session. Steps 5, 6–8, 9–10 are independent of each other (different feature flags, different connectors) and can be worked in parallel or in any order.

**Solana warning:** Step 6 will significantly increase cold compile times for any build that includes `onchain-solana`. Profile the workspace compile time before and after adding `solana-sdk`. If the increase is unacceptable, consider keeping Solana connectors in a separate workspace crate entirely (`connectors-v5-solana`).

---

## Appendix: Answers to Key Design Questions

### Q1: Single trait or trait hierarchy?

**Answer: Two-level hierarchy.** `ChainProvider` (base, object-safe, no SDK types in signatures) + per-family extension traits (`EvmChain`, `SolanaChain`, etc., not object-safe, carry SDK-native types). Connectors store the concrete provider type (`Arc<EvmProvider>`) not `Arc<dyn EvmChain>`.

### Q2: How to handle chain-specific features?

**Answer: Per-family extension traits + connector-internal helpers.** EVM contracts, Solana programs, and Cosmos modules are handled at three layers:
1. The family extension trait covers the cross-DEX subset (e.g., `erc20_balance` for EVM — needed by both GMX and Uniswap).
2. Connector-specific tx builders (`gmx/tx_builder.rs`, `dydx/tx_builder.rs`) hold the DEX-specific encoding. They import `Arc<EvmProvider>` or `Arc<CosmosProvider>` and call the trait methods.
3. Escape hatch: `EvmProvider::inner()` exposes the raw alloy `DynProvider` for any operation not covered by the trait.

### Q3: Where does this live?

**Answer: `src/core/chain/`.** Sibling of `traits/`, `types/`, `utils/`. First-class infrastructure, not exchange-specific code.

### Q4: Feature gating strategy?

**Answer: One feature per chain family.** `onchain-evm`, `onchain-solana`, `onchain-cosmos`, `onchain-starknet`. The existing `onchain-ethereum` becomes an alias for `onchain-evm`. `default = ["onchain-evm"]` preserves current behavior.
