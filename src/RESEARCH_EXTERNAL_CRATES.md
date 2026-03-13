# Type F External Crates Research — Signing & Blockchain SDKs

**Purpose**: Comprehensive research on all external crates needed for DEX/blockchain connectors
in digdigdig3. Covers k256, starknet-crypto, solana-sdk, alloy, and cosmrs — including versions,
sizes, feature flags, dependency overlaps, and feature-gating strategy.

**Date**: 2026-03-14
**Relevant exchanges**: Lighter (L2 ECDSA), Paradex (StarkNet ECDSA + JWT), Jupiter (Solana),
Raydium (Solana AMM), Uniswap (EVM), GMX (EVM perpetuals), HyperLiquid (EIP-712), dYdX v4 (Cosmos gRPC)

---

## Table of Contents

1. [k256 — secp256k1 ECDSA](#1-k256--secp256k1-ecdsa)
2. [starknet-crypto — StarkNet ECDSA](#2-starknet-crypto--starknet-ecdsa)
3. [solana-sdk — Solana Transaction Signing](#3-solana-sdk--solana-transaction-signing)
4. [alloy — EVM/Ethereum](#4-alloy--evmethereum)
5. [cosmrs + cosmos-sdk-proto — Cosmos/dYdX gRPC](#5-cosmrs--cosmos-sdk-proto--cosmosdydx-grpc)
6. [Dependency Overlaps](#6-dependency-overlaps)
7. [Feature-Gating Strategy](#7-feature-gating-strategy)
8. [Official SDKs vs Our Approach](#8-official-sdks-vs-our-approach)
9. [Recommended Cargo.toml Additions](#9-recommended-cargotoml-additions)
10. [Summary Table](#10-summary-table)

---

## 1. k256 — secp256k1 ECDSA

### Overview

- **Crate**: `k256`
- **Latest stable**: `0.13.4`
- **Latest RC**: `0.14.0-rc.8` (released 2026-03-10, active development)
- **Maintainer**: RustCrypto project (Tony Arcieri / iqlusion)
- **Repository**: `https://github.com/RustCrypto/elliptic-curves/tree/master/k256`
- **Downloads**: ~46M total, ~3.2M/month — massive adoption
- **Used in**: 5,218 crates (478 directly), ranked #1 in the `#ethereum` category

### Size & Compile Impact

| Metric | Value |
|--------|-------|
| Crate source size | 370 KB |
| SLoC | ~7.5K |
| Total dep tree size | ~3-5 MB |
| MSRV | Rust 1.65+ |

The crate itself is lightweight. Its transitive dep tree (via `elliptic-curve`, `sha2`, `signature`,
`ecdsa`) adds ~3-5 MB total but these are shared with alloy, hmac/sha2 already in our tree.

### Feature Flags

**Default features (all enabled unless overridden):**

| Feature | What it enables |
|---------|----------------|
| `arithmetic` | Scalar/point types (projective/affine), constant-time math |
| `ecdsa` | ECDSA signing/verification/recovery via `ecdsa-core` |
| `pkcs8` | PKCS#8 key serialization |
| `precomputed-tables` | Faster pubkey derivation via lookup tables (`once_cell`) |
| `schnorr` | BIP340 Taproot Schnorr signatures |
| `std` | Standard library support |
| `sha256` | SHA-256 digest support via `sha2` |

**Optional features (off by default):**

| Feature | Purpose |
|---------|---------|
| `ecdh` | Elliptic Curve Diffie-Hellman |
| `serde` | Serialize/deserialize keys and signatures |
| `jwk` | JSON Web Key format |
| `pem` | PEM encoding/decoding |
| `bits` | Bit manipulation on field elements |
| `hash2curve` | Hash-to-curve operations (advanced) |
| `expose-field` | Internal field exposure for low-level use |
| `hex-literal` | `hex!()` macro support |
| `test-vectors` | Known-answer test vectors |
| `critical-section` | Embedded/no_std safe precomputed tables |

### Minimal Usage for Signing Only

```toml
# Minimal: just ECDSA signing + recovery, no Schnorr, no PKCS8, no precomputed tables
k256 = { version = "0.13", default-features = false, features = ["ecdsa", "arithmetic"] }
```

This gives secp256k1 signing/verification/recovery without the extra schnorr/pkcs8/precomputed
overhead. Still pulls `sha2` which we already have.

### Is k256 Already in Our Dep Tree?

**Yes, transitively.** Our existing `alloy = { version = "1", ... }` already pulls `k256 0.13.x`
as a direct dependency of `alloy-signer`. Additionally, `alloy-primitives` activates k256 features.
No new compile cost. Adding `k256` explicitly just gives us direct access to the same instance.

**Key discovery**: `alloy-signer`'s `Cargo.toml` lists:
```toml
alloy-primitives = { features = ["k256"] }
k256 = { ... }
elliptic-curve = { ... }
```

So k256 is already compiled. We just need to add it as an explicit dependency with no-default-features
to avoid duplication.

### Security Note

k256 was audited by NCC Group. Two high-severity issues were found (ECDSA and Schnorr) and both
corrected. Current 0.13.x is considered safe. All secret operations are constant-time via `subtle`.

### Usage: Lighter (L2 ECDSA) and HyperLiquid

- **Lighter**: Uses standard secp256k1 ECDSA. API keys are private keys; requests are signed
  with the key. Need `k256::ecdsa::SigningKey` + `k256::ecdsa::RecoverableSignature`.
- **HyperLiquid**: Uses EIP-712 typed data signing on secp256k1. Alloy handles this via
  `alloy-signer` (which wraps k256 internally).

---

## 2. starknet-crypto — StarkNet ECDSA

### Overview

- **Crate**: `starknet-crypto`
- **Latest version**: `0.8.1` (released 2025-08-29)
- **Maintainer**: xJonathanLEI / community (not official StarkWare)
- **Repository**: `https://github.com/xJonathanLEI/starknet-rs`
- **Related**: `starknet` crate (full SDK), `starknet-core`, `starknet-signers`

### Two Options: standalone crypto vs full SDK

| Option | Crate | What You Get | When to Use |
|--------|-------|-------------|-------------|
| Minimal | `starknet-crypto` | ECDSA sign/verify, Pedersen hash, Poseidon hash, RFC-6979 | Just need signing |
| Full SDK | `starknet` | All of above + Account management, contract interaction, RPC client | Full dApp |

**Recommendation**: Use `starknet-crypto` alone. The full `starknet` crate pulls in tokio, reqwest,
jsonwebtoken, plus all of starknet-core, starknet-providers, starknet-accounts — massive overkill
for just signing Paradex JWT requests.

### Size & Compile Impact

| Metric | Value |
|--------|-------|
| Source size | ~78 KB |
| Doc size | ~5 MB |
| Key deps | crypto-bigint, sha2, hmac, rfc6979, starknet-curve, starknet-types-core, zeroize |

Dependencies overlap heavily with what we already have (sha2, hmac are already present).
`starknet-curve` and `starknet-types-core` are small focused crates.

### Feature Flags

| Feature | Effect |
|---------|--------|
| (default) | Includes Pedersen hash lookup table for maximum speed |
| `pedersen_no_lookup` | Removes 256 KB lookup table, ~10x slower Pedersen but smaller binary |

For digdigdig3 (off-chain signing, not on-chain program), use the default (with lookup table).
Speed matters more than binary size here.

### Dependencies (Key)

```
starknet-crypto 0.8.1
├── crypto-bigint       # big integer arithmetic
├── sha2                # SHA-256 (ALREADY in our tree)
├── hmac                # HMAC (ALREADY in our tree)
├── rfc6979             # Deterministic k generation per RFC-6979
├── starknet-curve      # StarkNet-specific curve parameters
├── starknet-types-core # FieldElement, Felt252 types
└── zeroize             # Secure memory clearing
```

### Security Note

**IMPORTANT**: The README explicitly states: "This library does NOT provide constant-time guarantees
and is NOT audited or reviewed for security." For off-chain signing (generating JWT tokens for API
authentication), this is acceptable. For production financial signing at scale, consider audited
alternatives if they exist (currently none exist for StarkNet in Rust that are audited).

### Usage: Paradex JWT Authentication

Paradex uses StarkNet ECDSA for authentication:
1. User has an L1 Ethereum private key
2. The L1 key signs a message to derive the L2 StarkNet private key (subkey)
3. Every JWT request is signed with the L2 subkey using StarkNet ECDSA
4. JWTs expire every 5 minutes — re-auth is frequent

The signing flow requires:
```rust
use starknet_crypto::{sign, FieldElement};

// Sign the auth message hash with StarkNet ECDSA
let signature = sign(&private_key, &msg_hash, &k)?;
```

The community Rust Paradex crate (`paradex` on crates.io, v0.7.1) uses `starknet-crypto 0.8.1`
as its signing dependency — confirming this is the right choice.

### Full starknet Crate: What NOT to Use

The full `starknet` crate (v0.17.0) is much heavier:
- Pulls `starknet-providers` (full RPC client with reqwest)
- Pulls `starknet-accounts` (account abstraction layer)
- Pulls `starknet-contract` (contract deployment/interaction)
- Essentially a complete dApp framework — we only need the crypto

---

## 3. solana-sdk — Solana Transaction Signing

### Overview

- **Old monolith**: `solana-sdk` — **DEPRECATED / REPLACED** as of 2025
- **New architecture**: ~117 individual crates in `anza-xyz/solana-sdk` repo
- **Current version of solana-transaction**: `4.0.0` (released 2026-02-14)
- **Current version of solana-sdk (legacy)**: `4.0.1` (released 2026-02-17)

### The Big Build Time Problem (Historical)

The old monolithic `solana-sdk` was notorious:
- **Old**: 147-300 crates compiled, ~24 seconds for a simple program
- **New component crates**: 21 crates compiled, <3 seconds for same program

The old SDK pulled in `openssl`, `rocksdb`, and validator-specific code even for simple signing.

### New Modular Architecture (2025/2026)

The SDK has been split into ~117 focused crates. For transaction signing only:

```toml
# Minimal set for Solana transaction signing (no full SDK needed)
solana-keypair    = "4"   # Key management (wraps ed25519-dalek internally)
solana-signature  = "4"   # Signature creation/verification
solana-transaction = "4"  # Transaction building and signing
solana-instruction = "4"  # Instruction types
solana-message    = "4"   # Message types (contains instruction list)
solana-pubkey     = "4"   # Public key / address type
```

### solana-transaction Dependency Count

| Metric | Value |
|--------|-------|
| Version | 4.0.0 |
| Required deps | 10 |
| Optional deps | 4 |
| Dev deps | 13 |
| Average build time | ~19 seconds (this crate alone) |
| Source size | 167 KB |

This is still nontrivial but MUCH better than the old ~300-crate monster.

### The ed25519-dalek Alternative

For pure signing without any Solana-specific types, `ed25519-dalek` alone works:

```toml
ed25519-dalek = { version = "2", features = ["rand_core"] }
```

**Pros**: ~50 KB crate, fast compile, well-audited, no Solana-specific overhead
**Cons**: You must manually implement Solana's transaction binary serialization format
(bincode + compact-array encoding), which is non-trivial

The Solana SDK's `Keypair` wraps `ed25519_dalek::Keypair` internally, so the signing algorithm
is identical. The question is whether you want Solana's serialization helpers.

**For Jupiter/Raydium (swap execution)**: Use minimal Solana crates — you need proper Transaction
serialization to submit to the RPC node. `ed25519-dalek` alone is insufficient.

**For read-only price data (no tx submission)**: `ed25519-dalek` alone is sufficient.

### Feature Flags for solana-transaction

| Feature | Effect |
|---------|--------|
| `serde` | Enable serialization (usually needed) |
| `frozen-abi` | ABI compatibility checking (not needed for client use) |
| `wincode` | Windows-specific (auto-selected) |

Recommended minimal:
```toml
solana-transaction = { version = "4", features = ["serde"] }
```

### Community Helper Crates

- **`jup-ag-sdk`**: Community Rust SDK for Jupiter swap routing (wraps Solana component crates)
- **`raydium-amm-swap`**: High-level Raydium AMM swap client (swap quotes + execution)
- **`sol-trade-sdk`**: High-performance trading SDK for Raydium AMM v4/CPMM

These exist on crates.io but are community-maintained, not official. They do correctly handle
the transaction signing + submission flow.

### Size Note: Current Status

The `solana-sdk` legacy crate is `585 KB` source with `~10-14 MB` total dep tree. The new
component crates individually are much smaller. When using only 5-6 component crates for signing,
the total dep tree is estimated at 2-4 MB vs the old 10-14 MB.

---

## 4. alloy — EVM/Ethereum

### Overview

- **Crate**: `alloy` (meta-crate)
- **Latest version**: `1.7.3`
- **Status**: Stable since v1.0 (June 2024), actively maintained by Paradigm/alloy-rs
- **Supersedes**: `ethers-rs` (deprecated, do not use)
- **Repository**: `https://github.com/alloy-rs/alloy`
- **Key users**: Foundry, Reth, Revm, SP1 zkVM

### Already in Our Cargo.toml

```toml
# Current (Cargo.toml line 76):
alloy = { version = "1", features = ["provider-ws", "rpc-types"], optional = true }
```

This is gated behind `onchain-ethereum` feature and used for Uniswap WebSocket. It already pulls
in the signing machinery including k256.

### Subcrate Architecture

Alloy is a collection of independent crates. The meta-crate `alloy` assembles them via features.
Key subcrates:

| Subcrate | Purpose | Size/Deps |
|----------|---------|-----------|
| `alloy-primitives` | U256, Address, B256, bytes types | Small, used everywhere |
| `alloy-core` | ABI encoding, sol! macro | Medium |
| `alloy-signer` | Abstract signing trait | Small, requires k256 |
| `alloy-signer-local` | Private key / keystore / mnemonic signing | Medium |
| `alloy-sol-types` | Solidity type bindings, EIP-712 compile-time | Medium |
| `alloy-dyn-abi` | Runtime ABI/EIP-712 encoding | Medium |
| `alloy-consensus` | Transaction types, receipt types | Medium |
| `alloy-provider` | RPC provider (HTTP/WS/IPC) | Large |
| `alloy-transport` | Transport layer | Large |
| `alloy-contract` | Smart contract interaction | Large |

### alloy-signer Dependencies (Direct)

From the actual `Cargo.toml`:
```
alloy-primitives (with k256 feature)
k256
elliptic-curve
either
async-trait
auto_impl
thiserror
# optional:
alloy-sol-types (for eip712 feature)
alloy-dyn-abi (for eip712 feature)
```

### Signing-Only Usage (No Provider)

For EIP-712 signing without any RPC provider/transport:

```toml
# Option A: Use subcrates directly (minimal)
alloy-signer = "1"
alloy-signer-local = "1"  # For PrivateKeySigner
alloy-sol-types = { version = "1", features = ["eip712"] }

# Option B: Use meta-crate with minimal features
alloy = { version = "1", default-features = false, features = [
    "signer-local",
    "eip712",
    "sol-types",
] }
```

### Feature Flags Relevant to Signing

| Feature | What it adds |
|---------|-------------|
| `signer-local` | `PrivateKeySigner`, `MnemonicBuilder`, keystore |
| `eip712` | EIP-712 typed data signing support |
| `secp256k1` | Alternative secp256k1 backend (libsecp256k1 C binding, faster but FFI) |
| `signer-mnemonic` | BIP39 mnemonic support |
| `signer-keystore` | Keystore file (UTC/JSON) support |
| `signer-aws` | AWS KMS signing |
| `signer-ledger` | Ledger hardware wallet |

### Feature Flags to AVOID (Heavy)

| Feature | Why Heavy |
|---------|-----------|
| `full` | Includes EVERYTHING: all providers, KZG, WS, traces |
| `provider-ws` | Pulls in tokio-tungstenite + full WS stack |
| `provider-http` | Pulls reqwest (already in our tree but adds headers etc) |
| `rpc-types` | Large set of typed JSON-RPC request/response structs |
| `contract` | Smart contract deployment and interaction machinery |
| `ens` | ENS name resolution |

### HyperLiquid EIP-712 Signing

HyperLiquid uses two signing schemes:
1. **`sign_l1_action`**: Signs Hyperliquid-specific L1 trading actions (order placement, cancel, etc.)
   - Uses EIP-712 with a "phantom agent" mechanism
   - Action is msgpack-serialized then hashed, used as `connectionId` in EIP-712 struct
2. **`sign_user_signed_action`**: For user-level actions

The community Rust SDK (`hypersdk`) explicitly uses alloy for all signing — no k256 direct usage.

```toml
# HyperLiquid feature gate
alloy = { version = "1", default-features = false, features = ["signer-local", "eip712"], optional = true }
```

### GMX (Arbitrum/EVM Perpetuals)

No official Rust SDK exists. Implementation requires:
- `alloy-contract` (to call `ExchangeRouter` contract)
- `alloy-provider` (to submit transactions)
- Full alloy with HTTP provider

GMX V2 uses isolated markets on Arbitrum/Avalanche. Transaction submission requires a proper
RPC provider. The `onchain-ethereum` feature (already in our Cargo.toml) is sufficient.

### Uniswap (EVM AMM)

Community crate: `uniswap-v3-sdk` (latest: v5.3.0+, uses `alloy ^1.1`).
Also `uniswap-v4-sdk` and `uniswap-sdk-core` exist on crates.io.

---

## 5. cosmrs + cosmos-sdk-proto — Cosmos/dYdX gRPC

### Overview

Two crates work together for Cosmos/dYdX:

| Crate | Version | Purpose |
|-------|---------|---------|
| `cosmos-sdk-proto` | `0.27.0` | Generated protobuf structs for Cosmos SDK messages |
| `cosmrs` | `0.22.0` | High-level Cosmos wallet: key management, tx building, broadcast |

### cosmos-sdk-proto

- **Version**: `0.27.0`
- **MSRV**: Rust 1.75+
- **Dependencies**: `prost 0.13`, `tendermint-proto 0.40.0`
- **Optional**: `tonic 0.13` (for gRPC), `serde 1`, `pbjson 0.7`

**Feature Flags**:

| Feature | Effect |
|---------|--------|
| `std` | Standard library support |
| `grpc` | Enables tonic gRPC client stubs |
| `grpc-transport` | Enables tonic transport (default) |
| `cosmwasm` | CosmWasm contract messages |
| `serde` | JSON serialization of proto types |

Recommended:
```toml
cosmos-sdk-proto = { version = "0.27", features = ["grpc-transport", "std"] }
```

### cosmrs

- **Version**: `0.22.0`
- **Key dependency**: `k256 0.13` — cosmrs uses k256 for Cosmos secp256k1 signing
- **Also uses**: `cosmos-sdk-proto 0.27`, `tendermint 0.40.0`, `ecdsa 0.16`, `signature 2`

**Feature Flags**:

| Feature | Effect |
|---------|--------|
| `bip32` (default) | HD wallet derivation paths |
| `getrandom` (default) | Random key generation |
| `cosmwasm` | CosmWasm support |
| `grpc` | gRPC client (enables cosmos-sdk-proto/grpc-transport + grpc-core) |
| `grpc-core` | Core gRPC (enables cosmos-sdk-proto/grpc) |
| `rpc` | Tendermint RPC client (HTTP-based, separate from gRPC) |
| `dev` | Development utilities |

**Key insight**: cosmrs already depends on `k256 0.13` — same version as alloy. They will share
the same compiled instance with no conflicts.

### dYdX v4 Custom Protobuf Types

dYdX v4 is a Cosmos SDK chain but with custom message types not in `cosmos-sdk-proto`:
- `MsgPlaceOrder`, `MsgCancelOrder`, `MsgBatchCancel`
- Custom order book structures, clob types, perpetuals

**Options**:

**Option A: Generate from dYdX protos with prost-build** (Recommended)
```toml
# build-dependencies
prost-build = "0.13"
tonic-build = "0.13"
```
Clone dYdX v4-chain protos from `github.com/dydxprotocol/v4-chain/proto/` and generate in
`build.rs`. This is the correct approach — no unofficial crates needed.

**Option B: Use cosmos-sdk-proto + hand-write dYdX-specific types**
For common Cosmos actions (send, delegate), cosmos-sdk-proto covers it.
For dYdX-specific trading, hand-write the prost structs or generate from protos.

**Note**: dYdX's own gRPC stream client example uses Go/Python, no official Rust client.
The `ibc-proto-rs` crate (`github.com/cosmos/ibc-proto-rs`) provides IBC proto types that
dYdX also uses.

### gRPC Stack for Cosmos

```
tonic (0.13)        — gRPC runtime (async, HTTP/2)
prost (0.13)        — protobuf encoding/decoding
tonic-build (0.13)  — code generation (build dep)
prost-build (0.13)  — proto compilation (build dep)
cosmos-sdk-proto    — pre-generated Cosmos SDK types
cosmrs              — wallet + tx signing layer
```

---

## 6. Dependency Overlaps

### k256 Shared Between Multiple Crates

| Crate | Uses k256? | Version Required |
|-------|-----------|-----------------|
| `alloy-signer` | Yes (direct dep) | 0.13.x |
| `cosmrs` | Yes (direct dep) | 0.13.x |
| `starknet-crypto` | No (uses its own curve) | N/A |
| `solana-*` | No (uses ed25519-dalek) | N/A |

Both alloy and cosmrs require `k256 0.13.x` — they will share the same compiled instance.
No version conflict. Adding explicit `k256 = "0.13"` also resolves to the same instance.

### sha2 Shared

| Already present | Version |
|-----------------|---------|
| `sha2 = "0.10"` in Cargo.toml | 0.10.x |

Used by: our HMAC signing, `k256`, `starknet-crypto`, cosmrs (via ecdsa).
All on 0.10.x series — single compiled instance shared.

### hmac Shared

| Already present | Version |
|-----------------|---------|
| `hmac = "0.12"` in Cargo.toml | 0.12.x |

Used by: our signing, `starknet-crypto`. Same version, no conflict.

### tonic/prost for gRPC

`cosmos-sdk-proto` requires `prost 0.13` and optional `tonic 0.13`. These are NOT currently
in our dep tree. Adding Cosmos/dYdX support will add:
- `prost` (~300 KB compiled)
- `tonic` (large: adds HTTP/2 via `hyper`, `tower`, `h2` — adds ~2-3 MB dep tree)
- `hyper` (but reqwest already uses hyper, may share)

### Potential Conflict: reqwest + tonic + hyper versions

Our current `reqwest = "0.12"` uses `hyper 1.x`. Tonic 0.13 also uses `hyper 1.x` (tonic
migrated to hyper 1 in tonic 0.12+). **No conflict expected** — both will share hyper 1.x.

### Summary Dependency Overlap Matrix

```
           | k256 | sha2 | hmac | elliptic-curve | hyper | tokio |
-----------|------|------|------|----------------|-------|-------|
existing   |  ✓*  |  ✓   |  ✓   |     ✓*         |  ✓    |  ✓    |
alloy      |  ✓   |  ✓   |      |     ✓          |  ✓    |  ✓    |
cosmrs     |  ✓   |  ✓   |      |     ✓          |       |  ✓    |
starknet-c |      |  ✓   |  ✓   |                |       |       |
solana-*   |      |  ✓   |      |                |       |  ✓    |
tonic/prost|      |      |      |                |  ✓    |  ✓    |

* = transitively via alloy already present
```

Key insight: starknet-crypto and solana-* add nearly zero new transitive deps beyond what
we already have. cosmrs adds k256 (already there) and tendermint (new ~500 KB crate).
The biggest new addition is `tonic` + `prost` for gRPC.

---

## 7. Feature-Gating Strategy

### Proposed Feature Flags

```toml
[features]
default = ["onchain-ethereum"]

# --- Existing ---
websocket = []
onchain-ethereum = ["dep:alloy"]

# --- Type F: New Blockchain Signing Features ---

# secp256k1 ECDSA for Lighter L2 and explicit k256 use
k256-signing = ["dep:k256"]

# StarkNet ECDSA for Paradex JWT auto-generation
starknet = ["dep:starknet-crypto"]

# Solana transaction signing for Jupiter + Raydium execution
solana = [
    "dep:solana-keypair",
    "dep:solana-signature",
    "dep:solana-transaction",
    "dep:solana-instruction",
    "dep:solana-message",
    "dep:solana-pubkey",
]

# Cosmos/dYdX gRPC + Cosmos wallet signing
cosmos-grpc = [
    "dep:cosmrs",
    "dep:cosmos-sdk-proto",
    "dep:tonic",
    "dep:prost",
    "cosmrs/grpc",
    "cosmos-sdk-proto/grpc-transport",
]

# Convenience: all DEX execution (EVM + Solana + StarkNet)
dex-execution = ["onchain-ethereum", "k256-signing", "starknet", "solana"]

# Convenience: all onchain
onchain-full = ["dex-execution", "cosmos-grpc"]
```

### Cargo.toml Additions (Optional Deps Section)

```toml
# --- Type F: Blockchain Signing SDKs ---

# secp256k1 (already in tree via alloy, explicit for Lighter)
k256 = { version = "0.13", default-features = false, features = ["ecdsa", "arithmetic"], optional = true }

# StarkNet ECDSA for Paradex
starknet-crypto = { version = "0.8", optional = true }

# Solana component crates (new modular SDK, not legacy monolith)
solana-keypair    = { version = "4", optional = true }
solana-signature  = { version = "4", optional = true }
solana-transaction = { version = "4", features = ["serde"], optional = true }
solana-instruction = { version = "4", optional = true }
solana-message    = { version = "4", optional = true }
solana-pubkey     = { version = "4", optional = true }

# Cosmos wallet + signing (for dYdX)
cosmrs = { version = "0.22", default-features = false, features = ["grpc"], optional = true }
cosmos-sdk-proto = { version = "0.27", default-features = false, features = ["grpc-transport", "std"], optional = true }

# gRPC transport (for dYdX Cosmos)
tonic = { version = "0.13", optional = true }
prost = { version = "0.13", optional = true }

# dYdX custom types (generated from protos — see build.rs)
# No separate crate needed; generated via prost-build in build.rs
```

### Why Not Default = Everything?

The library is used by digdigdig3 which is a CONNECTOR LIBRARY, not a trading bot. Most users
only need REST/WebSocket price data. They should not be forced to compile:
- solana: +21 crates, adds ed25519-dalek, borsh, etc.
- starknet: +6 crates, adds specialized curve math
- tonic/prost: +15 crates, full gRPC stack with HTTP/2

The feature gate approach lets users opt in to exactly what they need.

### Conditional Compilation Pattern

```rust
// In lighter/auth.rs
#[cfg(feature = "k256-signing")]
pub mod signing {
    use k256::ecdsa::{SigningKey, Signature, signature::Signer};
    // ...
}

// In paradex/auth.rs
#[cfg(feature = "starknet")]
pub mod signing {
    use starknet_crypto::{sign, FieldElement};
    // ...
}

// In jupiter/connector.rs
#[cfg(feature = "solana")]
pub mod execution {
    use solana_transaction::Transaction;
    use solana_keypair::Keypair;
    // ...
}
```

---

## 8. Official SDKs vs Our Approach

### dYdX

| Aspect | Status |
|--------|--------|
| Official Rust SDK | Does not exist |
| Official SDKs | JavaScript/TypeScript, Python |
| Rust approach | Generate from protos with `prost-build` + `tonic-build` |
| dYdX proto location | `github.com/dydxprotocol/v4-chain/proto/` |
| Community Rust crates | None maintained/reliable |
| gRPC stream example | `github.com/dydxprotocol/grpc-stream-client` (Go) |
| Recommendation | Generate protos in `build.rs`, use cosmrs for tx signing |

Custom types needed:
- `dydxprotocol.clob.MsgPlaceOrder`
- `dydxprotocol.clob.MsgCancelOrder`
- `dydxprotocol.clob.Order` (with `OrderId`, `ConditionType`, etc.)
- Standard Cosmos: `cosmos.tx.v1beta1.TxBody`, `AuthInfo`, `SignDoc`

### Jupiter (Solana DEX Aggregator)

| Aspect | Status |
|--------|--------|
| Official Rust SDK | Does not exist |
| Official SDKs | TypeScript (`@jup-ag/api`) |
| Community Rust | `jup-ag-sdk` on crates.io (unofficial, wraps Solana crates) |
| Our approach | REST API for quotes + transaction deserialization + sign + submit |
| Jupiter API | Returns unsigned transaction base64; we decode, sign, submit to RPC |
| Key crates needed | solana-transaction, solana-keypair + Solana RPC HTTP |

### Raydium (Solana AMM)

| Aspect | Status |
|--------|--------|
| Official Rust SDK | Does not exist as standalone client SDK |
| Official on-chain programs | `raydium-amm`, `raydium-amm-v3` (on-chain program crates) |
| Community Rust | `raydium-amm-swap`, `sol-trade-sdk` on crates.io |
| Our approach | Similar to Jupiter — use Raydium API/program calls + solana component crates |
| Alternative | Use `raydium-amm-swap` crate (has swap quote + execution logic) |

### Lighter.xyz (L2 Order Book)

| Aspect | Status |
|--------|--------|
| Official Rust SDK | Does not exist |
| Official SDKs | Python (`lighter-sdk` on PyPI) |
| Community Rust | None found |
| Our approach | REST API + `k256` for request signing |
| Signing mechanism | Standard secp256k1 ECDSA on request hash; API key = private key |
| Key crates needed | `k256` (already in tree via alloy) |

### Paradex (StarkNet DEX)

| Aspect | Status |
|--------|--------|
| Official Rust SDK | Community crate `paradex` (v0.7.1) by snow-avocado |
| Official SDKs | Python (`paradex-py`), JavaScript (`@paradex/sdk`) |
| StarkNet signing | C++ library: `starknet-signing-cpp` (uses Rust starknet-crypto internally) |
| Our approach | REST API + `starknet-crypto` for JWT generation |
| JWT mechanism | StarkNet ECDSA sign of auth hash → JWT valid 5 min |
| Performance | Go gnark: 1430 signs/sec, Python+Rust bindings: 182 signs/sec, JS: 50 signs/sec |
| Key crates needed | `starknet-crypto 0.8.1` |

### GMX (EVM Perpetuals — Arbitrum/Avalanche)

| Aspect | Status |
|--------|--------|
| Official Rust SDK | Does not exist |
| Official SDKs | None official (JavaScript/Solidity only) |
| Community Rust | None maintained |
| Our approach | `alloy` (already in tree) + contract ABI interaction |
| V2 architecture | Isolated markets, `ExchangeRouter` contract |
| Key crates needed | Full alloy with `contract` feature + provider |

### Uniswap (EVM AMM)

| Aspect | Status |
|--------|--------|
| Official Rust SDK | Does not exist |
| Community Rust | `uniswap-v3-sdk` (v5.3.0, uses alloy ^1.1), `uniswap-v4-sdk` |
| Our approach | `uniswap-v3-sdk` crate or direct alloy contract calls |
| Key crates needed | `alloy` (already in tree) + `uniswap-v3-sdk` if needed |

---

## 9. Recommended Cargo.toml Additions

Below are the exact additions needed. Current `Cargo.toml` already has:
- `alloy = { version = "1", features = ["provider-ws", "rpc-types"], optional = true }`
- `hmac`, `sha2`, `hex` — all from RustCrypto, compatible with k256/starknet-crypto
- `bs58` — useful for Solana pubkey parsing (already present)

### Minimal Additions Needed

```toml
[features]
default = ["onchain-ethereum"]
websocket = []
onchain-ethereum = ["dep:alloy"]
# NEW:
k256-signing = ["dep:k256"]
starknet = ["dep:starknet-crypto"]
solana = ["dep:solana-keypair", "dep:solana-signature", "dep:solana-transaction",
          "dep:solana-instruction", "dep:solana-message", "dep:solana-pubkey"]
cosmos-grpc = ["dep:cosmrs", "dep:cosmos-sdk-proto", "dep:tonic", "dep:prost",
               "cosmrs/grpc", "cosmos-sdk-proto/grpc-transport"]

[dependencies]
# ... existing deps unchanged ...

# NEW: k256 explicit (0 compile cost — already in tree via alloy)
k256 = { version = "0.13", default-features = false, features = ["ecdsa", "arithmetic"], optional = true }

# NEW: StarkNet ECDSA for Paradex
starknet-crypto = { version = "0.8", optional = true }

# NEW: Solana component crates (new modular SDK)
solana-keypair    = { version = "4", optional = true }
solana-signature  = { version = "4", optional = true }
solana-transaction = { version = "4", features = ["serde"], optional = true }
solana-instruction = { version = "4", optional = true }
solana-message    = { version = "4", optional = true }
solana-pubkey     = { version = "4", optional = true }

# NEW: Cosmos for dYdX
cosmrs = { version = "0.22", default-features = false, features = ["grpc"], optional = true }
cosmos-sdk-proto = { version = "0.27", default-features = false, features = ["grpc-transport", "std"], optional = true }
tonic = { version = "0.13", optional = true }
prost = { version = "0.13", optional = true }

# NEW build deps for dYdX proto generation
[build-dependencies]
prost-build = { version = "0.13", optional = true }
tonic-build = { version = "0.13", optional = true }
```

### dYdX build.rs (Proto Generation)

```rust
// build.rs
#[cfg(feature = "cosmos-grpc")]
fn main() {
    // Compile dYdX custom proto types
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &["proto/dydxprotocol/clob/tx.proto",
              "proto/dydxprotocol/clob/order.proto"],
            &["proto/"],
        )
        .unwrap();
}

#[cfg(not(feature = "cosmos-grpc"))]
fn main() {}
```

---

## 10. Summary Table

| Crate | Latest Version | Compile Cost | Existing in Tree? | Feature Flag | Use Case |
|-------|---------------|--------------|------------------|--------------|----------|
| `k256` | 0.13.4 (stable) / 0.14.0-rc.8 | Near-zero (already compiled via alloy) | YES (transitive) | `k256-signing` | Lighter signing |
| `starknet-crypto` | 0.8.1 | Low (~78 KB, overlapping sha2/hmac) | NO | `starknet` | Paradex JWT |
| `solana-keypair` + 5 others | 4.0.x | Medium (~19s for solana-transaction alone) | NO | `solana` | Jupiter, Raydium |
| `alloy` | 1.7.3 | Already compiled (onchain-ethereum feature) | YES (optional) | `onchain-ethereum` | Uniswap, HyperLiquid, GMX |
| `cosmrs` | 0.22.0 | Medium (tendermint adds ~500 KB new) | NO | `cosmos-grpc` | dYdX tx signing |
| `cosmos-sdk-proto` | 0.27.0 | Medium (prost types) | NO | `cosmos-grpc` | dYdX proto types |
| `tonic` | 0.13.x | Large (HTTP/2 stack, but hyper shared) | NO | `cosmos-grpc` | dYdX gRPC transport |

### Version Conflict Risk Assessment

| Pair | Risk | Notes |
|------|------|-------|
| alloy k256 ↔ cosmrs k256 | None | Both use k256 0.13.x |
| alloy sha2 ↔ starknet-crypto sha2 | None | Both use sha2 0.10.x |
| reqwest hyper ↔ tonic hyper | None | Both use hyper 1.x |
| solana-* tokio ↔ our tokio | None | Both use tokio 1.x |
| alloy prost ↔ cosmos prost | None | Both use prost 0.13 |

**Zero version conflicts expected.** The RustCrypto ecosystem (k256, sha2, hmac, ecdsa) is
internally consistent at 0.10/0.12/0.13/0.16 series. Alloy, cosmrs, and starknet-crypto
are all built on the same RustCrypto foundation.

### Decision Matrix: Build Everything vs Feature-Gate

| Scenario | Default deps | k256-signing | starknet | solana | cosmos-grpc |
|----------|-------------|-------------|---------|--------|-------------|
| REST-only price data | ✓ | — | — | — | — |
| + HyperLiquid/Uniswap execution | ✓ | — | — | — | — |
| + Lighter L2 signing | ✓ | ✓ | — | — | — |
| + Paradex JWT | ✓ | — | ✓ | — | — |
| + Jupiter/Raydium swaps | ✓ | — | — | ✓ | — |
| + dYdX order placement | ✓ | — | — | — | ✓ |
| Full DEX execution | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## Sources

- [k256 on crates.io](https://crates.io/crates/k256)
- [k256 on lib.rs (metrics)](https://lib.rs/crates/k256)
- [k256 feature flags on docs.rs](https://docs.rs/crate/k256/latest/features)
- [k256 RustCrypto GitHub](https://github.com/RustCrypto/elliptic-curves/tree/master/k256)
- [starknet-crypto on crates.io](https://crates.io/crates/starknet-crypto)
- [starknet-crypto docs.rs](https://docs.rs/starknet-crypto/latest/starknet_crypto/)
- [starknet-rs GitHub (xJonathanLEI)](https://github.com/xJonathanLEI/starknet-rs)
- [Paradex API Authentication](https://docs.paradex.trade/api/general-information/authentication)
- [Paradex starknet-signing-cpp](https://github.com/tradeparadex/starknet-signing-cpp)
- [paradex crate on crates.io](https://crates.io/crates/paradex)
- [solana-sdk new repo (anza-xyz)](https://github.com/anza-xyz/solana-sdk)
- [Solana SDK Revamp 2025 (Accelerate)](https://solanacompass.com/learn/accelerate-25/scale-or-die-at-accelerate-2025-solana-sdk-is-dead-long-live-the-solana-sdk)
- [solana-transaction on docs.rs](https://docs.rs/crate/solana-transaction/latest)
- [Rust Solana SDK Docs](https://solana.com/docs/clients/official/rust)
- [alloy crate on crates.io](https://crates.io/crates/alloy)
- [alloy docs.rs](https://docs.rs/alloy/latest/alloy/)
- [alloy feature flags](https://docs.rs/crate/alloy/latest/features)
- [alloy-signer Cargo.toml](https://github.com/alloy-rs/alloy/tree/main/crates/signer)
- [Introducing Alloy v1.0 (Paradigm)](https://www.paradigm.xyz/2025/05/introducing-alloy-v1-0)
- [Alloy Getting Started](https://alloy.rs/introduction/getting-started/)
- [HyperLiquid Signing Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/signing)
- [hypersdk Rust SDK](https://github.com/infinitefield/hypersdk)
- [cosmrs on docs.rs](https://docs.rs/cosmrs/latest/cosmrs/)
- [cosmrs Cargo.toml](https://raw.githubusercontent.com/cosmos/cosmos-rust/main/cosmrs/Cargo.toml)
- [cosmos-sdk-proto Cargo.toml](https://raw.githubusercontent.com/cosmos/cosmos-rust/main/cosmos-sdk-proto/Cargo.toml)
- [cosmos/cosmos-rust GitHub](https://github.com/cosmos/cosmos-rust)
- [dYdX v4-chain GitHub](https://github.com/dydxprotocol/v4-chain)
- [dYdX gRPC stream client](https://github.com/dydxprotocol/grpc-stream-client)
- [tonic gRPC Rust](https://github.com/hyperium/tonic)
- [uniswap-v3-sdk Rust](https://crates.io/crates/uniswap-v3-sdk)
- [jup-ag-sdk Rust](https://crates.io/crates/jup-ag-sdk)
- [raydium-amm-swap Rust](https://crates.io/crates/raydium-amm-swap)
- [Lighter Docs](https://docs.lighter.xyz/)
- [Lighter API](https://apidocs.lighter.xyz/docs/get-started-for-programmers-1)
