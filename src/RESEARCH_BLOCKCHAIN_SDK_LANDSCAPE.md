# Rust Blockchain SDK Landscape (March 2026)

**Purpose:** Canonical reference for selecting and integrating chain SDKs into trading/DeFi connectors.
**Scope:** EVM, Solana, StarkNet, Cosmos, Bitcoin, Sui, Aptos, TON — architecture patterns, dependency analysis.

---

## Table of Contents

1. [EVM Chains — alloy](#1-evm-chains--alloy)
2. [Solana](#2-solana)
3. [StarkNet](#3-starknet)
4. [Cosmos (dYdX v4)](#4-cosmos-dydx-v4)
5. [Bitcoin](#5-bitcoin)
6. [Sui](#6-sui)
7. [Aptos](#7-aptos)
8. [TON](#8-ton)
9. [Architecture: Shared Chain Provider Layer](#9-architecture-shared-chain-provider-layer)
10. [Dependency Conflict Matrix](#10-dependency-conflict-matrix)
11. [Decision Summary](#11-decision-summary)

---

## 1. EVM Chains — alloy

### THE canonical SDK: `alloy`

**Status:** Production-ready. Successor to ethers-rs (which is now deprecated).
**Current version:** 1.7.3 (as of March 2026)
**MSRV:** 1.91
**Crates.io:** https://crates.io/crates/alloy

### One SDK for ALL EVM chains

Yes — alloy is **chain-agnostic by design**. All EVM-compatible chains (Ethereum, Arbitrum, Optimism, Base, Polygon, Avalanche, BSC, etc.) are accessed by changing the **RPC URL only**. The `Network` trait is the abstraction layer.

```rust
// Ethereum mainnet
let provider = ProviderBuilder::new()
    .on_http("https://eth.rpc.example.com".parse()?);

// Arbitrum — same code, different URL
let provider = ProviderBuilder::new()
    .on_http("https://arb1.rpc.example.com".parse()?);

// Base (OP-stack chain)
use op_alloy::network::Optimism;
let provider = ProviderBuilder::new()
    .network::<Optimism>()
    .on_http("https://mainnet.base.org".parse()?);
```

For OP-stack chains (Base, Optimism), the `op-alloy` companion crate provides OP-specific RPC types. All other EVM chains work with the default `Ethereum` network type.

### Crate structure (meta-crate pattern)

The `alloy` crate is a meta-crate that re-exports 25+ sub-crates. You can depend only on the sub-crates you need:

| Sub-crate | Purpose |
|-----------|---------|
| `alloy-primitives` | `Address`, `U256`, `Bytes`, `B256` — pure types, no I/O |
| `alloy-sol-types` | `sol!` macro, Solidity ABI types |
| `alloy-core` | Combines primitives + sol-types + dyn-abi + rlp + serde |
| `alloy-provider` | `Provider` trait, `ProviderBuilder`, HTTP/WS/IPC transports |
| `alloy-contract` | Smart contract interaction layer |
| `alloy-signers` | Signer trait + local/Ledger/Trezor/AWS/GCP implementations |
| `alloy-consensus` | Ethereum consensus types (transactions, receipts, blocks) |
| `alloy-rpc-types` | RPC request/response JSON types |
| `alloy-transport` | Transport abstraction (HTTP, WebSocket, IPC) |
| `alloy-network` | `Network` trait, type-level chain parameterization |
| `alloy-eips` | EIP type implementations |

### Feature flags (alloy 1.7.3)

**Total:** 93 feature flags. **Default enabled:** 23.

Default bundle (`essentials`):
```toml
alloy = "1.7.3"
# Enables: contract, provider-http, rpc-types, signer-local, reqwest, std
```

Minimal for trading (read-only market data via HTTP):
```toml
alloy = { version = "1.7.3", default-features = false, features = [
    "std",
    "reqwest",
    "provider-http",
    "rpc-types",
    "sol-types",
    "primitives",
    "serde",
] }
```

Full trading (signing + WebSocket + contract calls):
```toml
alloy = { version = "1.7.3", features = [
    "full",           # essentials + consensus + eips + k256 + kzg + network
    "provider-ws",    # WebSocket subscriptions (eth_subscribe)
    "signer-local",   # Local keystore/mnemonic signing
] }
```

Optional signers (cloud key management):
```toml
alloy = { version = "1.7.3", features = [
    "signer-aws",     # AWS KMS
    "signer-gcp",     # GCP KMS
    "signer-ledger",  # Ledger hardware wallet
    "signer-trezor",  # Trezor hardware wallet
] }
```

### Performance vs ethers-rs

| Metric | ethers-rs | alloy 1.x |
|--------|-----------|-----------|
| U256 arithmetic (DeFi math) | baseline | **35–60% faster** |
| Static ABI encoding | baseline | **~10x faster** |
| Dynamic ABI encoding | baseline | **~20% faster** |

### Dependency footprint

- With `default-features = false` + minimal features: ~30–40 transitive crates
- With full features: ~80–100 transitive crates
- Key crypto primitives used: `k256` (pure Rust secp256k1) — consistent with most other EVM tooling
- Does NOT use `ring` crate (no conflict with most crypto backends)

### Status in our codebase

Already integrated via `onchain-ethereum` feature flag. GMX and Uniswap connectors both use alloy v1.

---

## 2. Solana

### SDK landscape

| Crate | Purpose | When to use |
|-------|---------|-------------|
| `solana-sdk` | Off-chain programs, keypairs, tx construction, data structures | Client-side tx building |
| `solana-client` | RPC client (HTTP + WebSocket to validator nodes) | Sending txs, querying accounts |
| `solana-program` | On-chain program development | Writing programs/smart contracts |
| `anchor-client` | High-level Anchor framework client | If interacting with Anchor programs |

For a **trading connector** (Jupiter, Raydium): you need `solana-sdk` + `solana-client`.

### New modular architecture (2025 revamp by Anza)

The old monolithic `solana-sdk` is being replaced by a **117-crate modular SDK**. This was announced at Accelerate 2025.

**Before (old):** 147 transitive crates, ~24s cold compile time
**After (new):** 21 crates for simple programs, ~3s compile time

The new architecture splits the SDK into component crates:
- `solana-pubkey`, `solana-keypair`, `solana-hash` — pure type crates
- `solana-transaction`, `solana-instruction`, `solana-message` — tx construction
- `solana-rpc-client`, `solana-rpc-client-api` — communication
- `solana-signer`, `solana-signature` — signing

**Current published version:** `solana-sdk` v3.x (modular) exists on GitHub but is under active development. The **stable published version** on crates.io as of early 2026 is in the 2.x line.

### Recommended approach for trading connectors

```toml
# For Jupiter/Raydium — minimal deps needed
solana-sdk = { version = "2.1", default-features = false, features = [
    "std",
] }
solana-client = "2.1"
```

Or use the new modular approach (when stable):
```toml
solana-pubkey = "2.1"
solana-keypair = "2.1"
solana-transaction = "2.1"
solana-rpc-client = "2.1"
```

### Compile time warning

Even with `default-features = false`, solana-sdk pulls in substantial dependencies due to cryptographic primitives (`ed25519-dalek`, `curve25519-dalek`, `sha2`, `blake3`). Cold compile can easily be 2–5 minutes for a workspace that includes Solana.

**Mitigation strategies:**
1. Feature-gate Solana connectors behind a `solana` feature flag in the workspace
2. Use the new modular crates when published (target: mid-2026)
3. Use `sccache` or `cargo nextest` for caching

### Key crypto primitives

- `ed25519-dalek` for signatures
- `curve25519-dalek` for key derivation
- These **can conflict** with older versions of curve25519/ed25519 used by other crates

---

## 3. StarkNet

### Crate landscape

| Crate | Purpose | When to use |
|-------|---------|-------------|
| `starknet` | Meta-crate (re-exports all below) | Entry point |
| `starknet-core` | Core types: `FieldElement`, `Call`, `Transaction` | Always |
| `starknet-providers` | `JsonRpcClient`, HTTP/WebSocket transport | Reading state, querying |
| `starknet-accounts` | Account abstraction, invoke transactions | Sending transactions |
| `starknet-signers` | Key management, STARK signature | Signing |
| `starknet-crypto` | Low-level Pedersen hash, ECDSA on STARK curve | Minimal cryptography only |
| `starknet-contract` | Contract interaction utilities | ABI + calls |

### Current version: 0.17.0 (August 2025)

Supports: StarkNet v0.14.0, JSON-RPC spec v0.9.0.

### Stability warning

**Experimental — not production audited.** Official disclaimer from repo:
> "starknet-rs is still experimental. Breaking changes will be made before the first stable release. The library is also NOT audited or reviewed for security."

The `starknet-crypto` crate does **not guarantee constant-time operations**.

### Current usage in our codebase

Paradex connector uses `starknet-crypto` 0.6 (minimal — just for JWT auth signatures). This is the correct minimal approach.

### Feature flags

```toml
# Minimal — just for signing (Paradex use case)
starknet-crypto = "0.6"

# For full provider access (if needed in future)
starknet = { version = "0.17", default-features = false, features = [
    "providers",
    "accounts",
    "signers",
] }
```

### Dependency footprint

- `starknet-crypto` alone: ~10–15 transitive crates (pure Rust, no C FFI)
- Full `starknet`: ~50–70 transitive crates
- Key primitive: custom `starknet-curve` (Stark curve, NOT secp256k1) — no conflict with EVM tooling

---

## 4. Cosmos (dYdX v4)

### The chain context

dYdX v4 runs on its own **Cosmos SDK app-chain** using CometBFT consensus. It is NOT an EVM chain. Trading requires:
1. Building a `TxBody` containing `MsgPlaceOrder` or `MsgCancelOrder` protobuf messages
2. Signing with secp256k1 (Cosmos-style, not Ethereum-style)
3. Broadcasting via gRPC (`cosmos.tx.v1beta1.Service/BroadcastTx`)

### Canonical Rust SDK: `cosmrs`

**Version:** 0.22.0 (current)
**MSRV:** 1.75+
**Crates.io:** https://crates.io/crates/cosmrs
**Repo:** https://github.com/cosmos/cosmos-rust

#### What cosmrs covers

| Feature | Supported? |
|---------|-----------|
| Transaction building (TxBody, AuthInfo) | Yes |
| Transaction signing (secp256k1) | Yes |
| BIP32 HD wallet derivation | Yes (feature: `bip32`) |
| Bank messages (send tokens) | Yes |
| Staking messages | Yes |
| CosmWasm messages | Yes (feature: `cosmwasm`) |
| Broadcasting via Tendermint RPC | Yes (feature: `rpc`) |
| Broadcasting via gRPC | Via `tonic` companion |
| gRPC queries | Via `cosmos-sdk-proto` + `tonic` |

#### Feature flags

```toml
# Minimal for tx signing + broadcasting
cosmrs = { version = "0.22", features = [
    "rpc",       # Tendermint RPC client
    "bip32",     # HD wallet key derivation
] }

# For CosmWasm contract interactions
cosmrs = { version = "0.22", features = ["rpc", "bip32", "cosmwasm"] }
```

#### The `tendermint-rpc` companion

```toml
tendermint-rpc = { version = "0.39", features = ["http-client", "websocket-client"] }
```

The `tendermint-rpc` crate provides the actual transport. All networking is feature-gated, so it's lightweight when you only need core types.

### dYdX v4 specific: Nethermind Rust client

There is an **official Rust client** for dYdX v4 maintained by Nethermind:
Repository: `github.com/dydxprotocol/v4-clients` — `v4-client-rs/` directory.

This client wraps `cosmrs` and provides dYdX-specific message types and sequence number management. For our connector, we have two options:
1. Depend on `v4-client-rs` directly (heaviest, most complete)
2. Depend on `cosmrs` and build dYdX messages from protobuf definitions (lighter, more control)

**Recommended:** Option 2 — cosmrs + dYdX proto defs. Keeps dependency footprint minimal.

### Dependency footprint

- `cosmrs` with `rpc` + `bip32`: ~50–70 transitive crates
- Crypto primitives: `k256` (same as alloy — **no conflict**), `ripemd`, `sha2`
- Does NOT use `ring` crate

### Conflict potential

`cosmrs` uses `k256` from the `RustCrypto` organization — the same version used by `alloy`. They should resolve to a single copy in a unified workspace. No known conflicts.

---

## 5. Bitcoin

### Canonical crate: `rust-bitcoin`

**Crates.io:** https://crates.io/crates/bitcoin
**Repo:** https://github.com/rust-bitcoin/rust-bitcoin
**Current version:** ~0.32.x (2025/2026)

### What it provides

| Feature | Supported? |
|---------|-----------|
| Transaction types (UTXO model) | Yes |
| PSBT (Partially Signed Bitcoin Transactions) | Yes — `bitcoin-psbt` or built-in |
| Script parsing and construction | Yes |
| Address types (Legacy, SegWit, Taproot) | Yes |
| Network constants (mainnet, testnet, regtest) | Yes |
| BIP32 HD wallet | Yes |
| RPC client (Bitcoin Core) | Separate: `bitcoincore-rpc` |
| P2P protocol | No — separate project |

### Ecosystem crates

| Crate | Purpose |
|-------|---------|
| `bitcoin` | Core primitives |
| `bitcoincore-rpc` | JSON-RPC client for Bitcoin Core node |
| `psbt-v2` | BIP-370 PSBT v2 implementation |
| `ordinals-parser` | Parsing ordinal inscriptions (lightweight, no full `ord` needed) |
| `bdk` (Bitcoin Dev Kit) | Full wallet toolkit — tx building, UTXO management, hardware wallet support |

### Use cases in trading

| Use Case | Crates Needed |
|----------|--------------|
| Reading BTC prices (not on-chain) | None — REST API connectors |
| Bitcoin node queries (balances, UTXO) | `bitcoin` + `bitcoincore-rpc` |
| Ordinals inscription parsing | `ordinals-parser` |
| BRC-20 token trading (PSBT swaps) | `bitcoin` + `psbt-v2` |
| Lightning Network | Separate ecosystem (`lightning` crate) |

### For our trading system

Bitcoin trading connectors in the current system use CEX REST APIs (Binance BTC/USDT pair) — **no on-chain Bitcoin SDK needed** for that.

If Ordinals/BRC-20 DEX support is added in the future:
```toml
bitcoin = { version = "0.32", default-features = false, features = ["std"] }
psbt-v2 = "0.1"
ordinals-parser = "0.1"
```

### Dependency footprint

- `bitcoin` core: ~20–30 transitive crates
- Key primitives: `secp256k1` (C FFI binding to libsecp256k1) OR `k256` (configurable)
- **Potential conflict:** `bitcoin` uses `secp256k1` crate (C FFI), while `alloy` uses `k256` (pure Rust). These are different implementations of the same curve. Cargo will include BOTH if both are needed. Not a compilation conflict but increases binary size.

---

## 6. Sui

### Official SDK: `sui-sdk`

**Repo:** `github.com/MystenLabs/sui` (crates/sui-sdk) and `github.com/MystenLabs/sui-rust-sdk`
**Crates.io entry:** via git dependency
**Current status:** Actively maintained, production networks live.

### Two SDK variants

| SDK | Status | Use |
|-----|--------|-----|
| `sui-sdk` (legacy) | Stable, JSON-RPC based | Production use today |
| `sui-rust-sdk` (new) | Newer, GraphQL-based | Forward-compatible, newer features |

### How to depend on it

```toml
# Legacy SDK (stable)
sui_sdk = { git = "https://github.com/mystenlabs/sui", package = "sui-sdk" }
tokio = { version = "1.2", features = ["full"] }

# New SDK (for GraphQL-based queries)
sui-graphql-client = { git = "https://github.com/MystenLabs/sui-rust-sdk" }
sui-sdk-types = { git = "https://github.com/MystenLabs/sui-rust-sdk" }
```

### Capabilities

- Connect to Sui mainnet/testnet/devnet/localnet
- Read coin balances, object states
- Build and sign Programmable Transaction Blocks (PTBs)
- Execute transactions
- Subscribe to events

### Maturity assessment

Sui mainnet is live and active. The SDK is production-ready for network interaction. However:
- `sui-sdk` pulls from git (not stable crates.io release) — reproducible builds require pinning commit SHA
- Compilation is heavy due to Sui's large workspace

### Dependency footprint

- Very heavy — Sui's workspace contains hundreds of crates
- Using via git dependency means Cargo compiles the entire relevant portion of the Sui workspace
- **NOT recommended** for connectors without isolating behind a feature flag and separate crate

---

## 7. Aptos

### Official SDK: `aptos-rust-sdk`

**Repo:** `github.com/aptos-labs/aptos-rust-sdk`
**Status:** "Lightly supported" — official but secondary to the TypeScript SDK

### Usage

```toml
aptos-rust-sdk = { git = "https://github.com/aptos-labs/aptos-rust-sdk", package = "aptos-rust-sdk" }
aptos-rust-sdk-types = { git = "https://github.com/aptos-labs/aptos-rust-sdk", package = "aptos-rust-sdk-types" }
```

**Additional requirement:** Must enable an unstable Tokio feature:
```toml
# .cargo/config.toml
[build]
rustflags = ["--cfg", "tokio_unstable"]
```

This is a **red flag** for production use — it relies on `tokio_unstable` flag, which means using non-stable Tokio internals.

### Maturity assessment

- Aptos mainnet is live
- The Rust SDK is officially "lightly supported" — TypeScript SDK is primary
- Requires `tokio_unstable` — production risk
- Not published to crates.io — git-only dependency
- Not recommended for trading use without significant caveats

### Alternative

For read-only market data from Aptos DEXes, a plain HTTP connector to Aptos REST API + optional indexer GraphQL is simpler and avoids SDK dependency entirely.

---

## 8. TON

### Rust SDK: `tonlib-rs`

**Crates.io:** https://crates.io/crates/tonlib (v0.15 as of Oct 2024)
**Repo:** `github.com/ston-fi/tonlib-rs` (maintained by STON.fi, a major TON DEX)

### Structure

| Crate | Purpose |
|-------|---------|
| `tonlib` | Meta-crate |
| `tonlib-client` | Low-level TON blockchain client (C bindings to tonlib) |
| `tonlib-core` | High-level API built on tonlib-client |

### Dependencies

Requires native system libraries:
- `build-essential`, `cmake`, `libsodium-dev` (Linux)
- Wraps the C TON library via FFI

### Maturity assessment

- TON ecosystem grew significantly in 2025 due to Telegram integration
- `tonlib-rs` is maintained by STON.fi (production DEX) — reasonable stability
- However: C FFI dependency means cross-compilation complexity (especially Windows)
- Version 0.x — breaking changes possible
- **For trading on STON.fi or DeDust (TON DEXes):** tonlib-rs is the only viable Rust path

### Alternative: Pure HTTP

TON has a REST API (`toncenter.com/api/v2`) that works without the native SDK for read-only operations. For tx signing, the native SDK is needed.

---

## 9. Architecture: Shared Chain Provider Layer

### Problem statement

With 20 on-chain connectors out of 137 total, bundling chain SDKs into each connector creates:
1. Dependency duplication (e.g., alloy initialized 5x for GMX + Uniswap + Lighter + ...)
2. Multiple RPC connection pools to the same chain
3. No rate limit coordination across connectors on same chain
4. Binary size bloat from redundant crate copies

### Recommended architecture: `ChainProvider<C>` layer

```
Connectors (GMX, Uniswap, Lighter, ...)
        |
        v
ChainProvider<Ethereum>  ChainProvider<Solana>  ChainProvider<Cosmos>
        |                       |                       |
        v                       v                       v
   alloy Provider           solana RpcClient        cosmrs + tonic
   (HTTP + WS pool)         (connection pool)       (gRPC channel)
```

### Implementation pattern

```rust
// Trait definition in shared `chain-providers` crate
pub trait ChainProvider: Send + Sync {
    type SignedTx;
    type TxHash;

    async fn broadcast_tx(&self, tx: Self::SignedTx) -> Result<Self::TxHash>;
    async fn get_nonce(&self, address: &str) -> Result<u64>;
    async fn estimate_gas(&self, call: &CallRequest) -> Result<u64>;
}

// EVM implementation (wraps alloy)
pub struct EvmProvider {
    provider: Arc<dyn Provider<Http<Client>>>,
    chain_id: u64,
}

// Solana implementation (wraps solana-rpc-client)
pub struct SolanaProvider {
    client: Arc<RpcClient>,
    commitment: CommitmentConfig,
}

// Cosmos implementation (wraps cosmrs + tonic)
pub struct CosmosProvider {
    channel: Arc<Channel>,  // tonic gRPC channel
    chain_id: String,
    denom: String,
}
```

### Connector usage pattern

```rust
// GMX connector uses injected EvmProvider
pub struct GmxConnector {
    chain: Arc<EvmProvider>,  // shared with Uniswap, etc.
    exchange_router: Address,
}

// Jupiter connector uses injected SolanaProvider
pub struct JupiterConnector {
    chain: Arc<SolanaProvider>,  // shared with Raydium
    base_url: String,
}
```

### Benefits

| Concern | Bundled per-connector | Shared provider layer |
|---------|----------------------|----------------------|
| RPC connections | N connections per chain | 1 connection per chain |
| Rate limit tracking | None (each connector blind) | Centralized per chain |
| Memory | O(N * chains) | O(chains) |
| Nonce/sequence sync | Race conditions | Serialized per address |
| Config | Duplicated in each connector | Single place |

### Feature gating strategy

```toml
# In connectors-v5/Cargo.toml
[features]
default = []

# Chain provider groups (coarse-grained)
onchain-evm = ["alloy/essentials", "dep:alloy"]
onchain-solana = ["dep:solana-sdk", "dep:solana-client"]
onchain-cosmos = ["dep:cosmrs", "dep:tonic"]
onchain-starknet = ["dep:starknet-crypto"]
onchain-bitcoin = ["dep:bitcoin"]
onchain-ton = ["dep:tonlib"]

# Connector-level features that pull in chain providers
connector-gmx = ["onchain-evm"]
connector-uniswap = ["onchain-evm"]
connector-jupiter = ["onchain-solana"]
connector-raydium = ["onchain-solana"]
connector-dydx-trading = ["onchain-cosmos"]
connector-paradex = ["onchain-starknet"]
```

---

## 10. Dependency Conflict Matrix

### Crypto primitive overlap

| Primitive | alloy | cosmrs | starknet-rs | bitcoin | solana-sdk |
|-----------|-------|--------|-------------|---------|------------|
| `k256` (pure Rust secp256k1) | Yes (primary) | Yes (primary) | No | Optional | No |
| `secp256k1` (C FFI) | No | No | No | Yes (primary) | No |
| `ed25519-dalek` | No | No | No | No | Yes (primary) |
| `sha2` | Yes | Yes | No | Yes | Yes |
| `ring` | No | No | No | No | No |
| `openssl` | No | No | No | No | No |
| STARK curve | No | No | Yes (custom) | No | No |

### Conflict risk assessment

| Pairing | Risk | Notes |
|---------|------|-------|
| alloy + cosmrs | LOW | Both use `k256` from RustCrypto — typically unify to single copy |
| alloy + solana-sdk | MEDIUM | Different crypto stacks. Solana uses ed25519+curve25519; alloy uses secp256k1. No direct conflict but increases binary size |
| alloy + bitcoin | LOW-MEDIUM | alloy uses k256, bitcoin uses secp256k1 (C FFI). Both can coexist but you get two secp256k1 implementations |
| cosmrs + solana-sdk | MEDIUM | Different crypto families. Watch for `sha2` version conflicts |
| starknet-crypto + alloy | LOW | Completely different curves (Stark vs secp256k1). Minimal overlap |
| tonlib + anything | HIGH | C FFI library. Build complexity on Windows/macOS. Potential OpenSSL conflicts |
| aptos-sdk + anything | HIGH | Requires `tokio_unstable`. Patched versions of `merlin` + `x25519-dalek` can conflict |
| Sui SDK + anything | MEDIUM-HIGH | Git-only, workspace-coupled. Can pull in conflicting versions of shared crates |

### Practical advice for Cargo.toml

```toml
# To resolve k256 version conflicts between alloy and cosmrs:
[patch.crates-io]
k256 = { version = "0.13" }  # Pin to single version if they diverge

# To prevent alloy from pulling in secp256k1 C FFI:
# Use alloy with k256 feature, not secp256k1 feature
alloy = { version = "1.7", features = ["k256"] }

# solana-sdk: pin to avoid curve25519-dalek version conflicts
curve25519-dalek = "4.1"
ed25519-dalek = "2.1"
```

---

## 11. Decision Summary

### Per-chain recommendation table

| Chain | SDK | Version | Stability | Our Use | Feature Gate |
|-------|-----|---------|-----------|---------|-------------|
| Ethereum + all EVM | `alloy` | 1.7.3 | Production | GMX, Uniswap, Lighter | `onchain-evm` |
| Solana | `solana-sdk` + `solana-client` | 2.x stable | Production (heavy compile) | Jupiter, Raydium | `onchain-solana` |
| StarkNet | `starknet-crypto` (minimal) or `starknet` (full) | 0.17.0 | Experimental | Paradex | `onchain-starknet` |
| Cosmos / dYdX | `cosmrs` | 0.22.0 | Production | dYdX v4 trading | `onchain-cosmos` |
| Bitcoin | `bitcoin` + `bitcoincore-rpc` | 0.32.x | Production | Future (Ordinals/BRC-20) | `onchain-bitcoin` |
| Sui | `sui-sdk` (via git) | 2025 mainnet | Production, heavy | Future | `onchain-sui` |
| Aptos | `aptos-rust-sdk` (via git) | 0.x | Lightly supported | Not recommended | — |
| TON | `tonlib-rs` | 0.15 | Beta (C FFI) | Future | `onchain-ton` |

### Architecture decision: YES, use shared provider layer

**Rationale:**
- 7 EVM-chain connectors benefit from single alloy `Provider` instance
- 2 Solana connectors (Jupiter, Raydium) share one `RpcClient`
- dYdX needs sequence number serialization — impossible with per-connector instances
- Feature gating becomes cleaner (one `onchain-evm` flag, not 7 separate)

**Recommended crate structure:**
```
zengeld-terminal/crates/connectors/crates/v5/
├── src/
│   ├── chain_providers/        # NEW: shared chain layer
│   │   ├── mod.rs
│   │   ├── evm.rs              # alloy-based EvmProvider
│   │   ├── solana.rs           # SolanaProvider
│   │   ├── cosmos.rs           # CosmosProvider (cosmrs + tonic)
│   │   ├── starknet.rs         # StarknetProvider
│   │   └── bitcoin.rs          # BitcoinProvider
│   └── crypto/
│       └── dex/
│           ├── gmx/            # Uses Arc<EvmProvider>
│           ├── uniswap/        # Uses Arc<EvmProvider>
│           ├── jupiter/        # Uses Arc<SolanaProvider>
│           └── dydx/           # Uses Arc<CosmosProvider>
```

### Immediate action items

1. **dYdX v4 trading** — Add `cosmrs = { version = "0.22", features = ["rpc", "bip32"] }` behind `onchain-cosmos` feature. Build `MsgPlaceOrder`/`MsgCancelOrder` using cosmrs TxBuilder.

2. **Jupiter + Raydium** — Add `solana-sdk` + `solana-client` behind `onchain-solana` feature. Tx building for swap execution.

3. **Lighter** — Add `k256` directly (it's already a transitive dep of alloy). Build ECDSA-signed transaction payloads for the `/sendTx` endpoint.

4. **GMX + Uniswap** — Already done via alloy. Review whether single shared `EvmProvider` can serve both instead of two separate instances.

5. **StarkNet full provider** — If Paradex needs read queries beyond signing, upgrade from `starknet-crypto` to `starknet-providers` with `JsonRpcClient`.

---

## Sources

- [Alloy v1.0 introduction — Paradigm](https://www.paradigm.xyz/2025/05/introducing-alloy-v1-0)
- [alloy crate — docs.rs](https://docs.rs/alloy/latest/alloy/)
- [alloy feature flags — docs.rs](https://docs.rs/crate/alloy/latest/features)
- [alloy installation guide — alloy.rs](https://alloy.rs/introduction/installation/)
- [Alloy multi-chain migration guide](https://alloy.rs/migrating-from-ethers/reference/)
- [solana-sdk new modular architecture — Solana Compass](https://solanacompass.com/learn/accelerate-25/scale-or-die-at-accelerate-2025-solana-sdk-is-dead-long-live-the-solana-sdk)
- [anza-xyz/solana-sdk — GitHub](https://github.com/anza-xyz/solana-sdk)
- [solana-sdk — docs.rs](https://docs.rs/solana-sdk/latest/solana_sdk/)
- [starknet-rs — GitHub](https://github.com/xJonathanLEI/starknet-rs)
- [starknet 0.17.0 — docs.rs](https://docs.rs/crate/starknet/latest)
- [starknet-providers — crates.io](https://crates.io/crates/starknet-providers)
- [cosmrs — docs.rs](https://docs.rs/cosmrs/latest/cosmrs/)
- [cosmrs — crates.io](https://crates.io/crates/cosmrs)
- [cosmos/cosmos-rust — GitHub](https://github.com/cosmos/cosmos-rust)
- [dydxprotocol/v4-clients — GitHub](https://github.com/dydxprotocol/v4-clients)
- [rust-bitcoin — GitHub](https://github.com/rust-bitcoin/rust-bitcoin)
- [bitcoin crate — docs.rs](https://docs.rs/bitcoin/)
- [Sui Rust SDK — docs.sui.io](https://docs.sui.io/references/rust-sdk)
- [MystenLabs/sui-rust-sdk — GitHub](https://github.com/MystenLabs/sui-rust-sdk)
- [Aptos Rust SDK — aptos.dev](https://aptos.dev/build/sdks/rust-sdk)
- [aptos-labs/aptos-rust-sdk — GitHub](https://github.com/aptos-labs/aptos-rust-sdk)
- [ston-fi/tonlib-rs — GitHub](https://github.com/ston-fi/tonlib-rs)
- [tonlib — crates.io](https://crates.io/crates/tonlib)
- [k256 crate — docs.rs](https://docs.rs/k256/latest/k256/)
- [secp256k1 (C FFI) — crates.io](https://crates.io/crates/secp256k1)
- [Pinocchio (ultra-lightweight Solana) — Helius](https://www.helius.dev/blog/pinocchio)
