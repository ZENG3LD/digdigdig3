# Audit: On-Chain Access Candidates — Monitors, Analytics, Data Feeds

**Date:** 2026-03-14
**Scope:** All connectors that interact with blockchains — DEX/swap, analytics monitors, and
data feed providers. Focus: which ones could benefit from `ChainProvider` direct RPC access.
**Reference design:** `DESIGN_CHAIN_PROVIDERS.md`

---

## 1. Complete Inventory Table

| Connector | Path | Category | Current Transport | Chain(s) | ChainProvider Adds Value? | What Specifically | Priority |
|-----------|------|----------|-------------------|-----------|--------------------------|-------------------|----------|
| **dYdX v4** | `crypto/dex/dydx/` | DEX | REST + gRPC (grpc feature) | Cosmos (dYdX chain) | **YES — CRITICAL** | Tx signing, MsgPlaceOrder/MsgCancelOrder via CosmosProvider | P0 |
| **GMX v2** | `crypto/dex/gmx/` | DEX | REST + The Graph GraphQL + alloy RPC (onchain-evm feature) | Arbitrum + Avalanche (EVM) | **YES — PARTIAL** | Signer integration: already builds unsigned tx, needs LocalWallet wired | P1 |
| **Jupiter** | `crypto/dex/jupiter/` | DEX Aggregator | REST | Solana | **YES — CRITICAL** | SolanaProvider: sign + broadcast swap tx returned by `/swap` endpoint | P0 |
| **Lighter** | `crypto/dex/lighter/` | zkEVM CLOB DEX | REST + WebSocket | zkEVM L2 (Ethereum addresses) | **YES — READY** | ECDSA k256 signing for `/sendTx` — k256 feature already in Cargo.toml | P0 |
| **Paradex** | `crypto/dex/paradex/` | StarkNet DEX | REST + WebSocket | StarkNet (Ethereum L2) | **YES — PARTIAL** | StarkNetProvider: auto-refresh JWT by signing timestamp. Currently needs pre-obtained JWT | P1 |
| **Raydium** | `crypto/swap/raydium/` | Solana AMM | REST | Solana | **YES — CRITICAL** | SolanaProvider: sign + broadcast swap tx returned by `/transaction/swap-base-in` | P0 |
| **Uniswap v3** | `crypto/swap/uniswap/` | Ethereum AMM | REST + GraphQL + JSON-RPC + alloy (onchain-evm feature) | Ethereum (+ Arbitrum, Base via V3 pools) | **YES — PARTIAL** | Signer integration: already builds unsigned tx, needs Permit2 EIP-712 signer | P1 |
| **Bitquery** | `onchain/analytics/bitquery/` | Blockchain Analytics | GraphQL REST + WS | Multi-chain: 20+ (EVM, Solana, Bitcoin, Cosmos, etc.) | **NO** | Pure indexer — read-only analytics, no tx submission ever needed | — |
| **Whale Alert** | `onchain/analytics/whale_alert/` | Whale Monitor | REST + WS (URL stored, not wired) | BTC, ETH, SOL, POLY, XRP, ADA, TRX, etc. | **NO** | Read-only whale transaction tracking API. Direct RPC would duplicate Whale Alert's own indexer | — |
| **Etherscan** | `onchain/ethereum/etherscan/` | Block Explorer | REST only | Ethereum (+ Sepolia testnet) | **MAYBE — LOW** | Direct `eth_call` via EvmProvider could replace Etherscan proxy for gas oracle / block number when no API key | P3 |
| **Hyperliquid** | `crypto/cex/hyperliquid/` | Perps L1 (hybrid) | REST | Hyperliquid L1 (EVM-compatible) | **YES — PARTIAL** | Uses EIP-712 + alloy signing internally (already done via `alloy::signers::local::PrivateKeySigner`). No external provider needed | — |
| **DefiLlama** | `aggregators/defillama/` | DeFi Aggregator | REST | Multi-chain (tracks TVL across all chains) | **NO** | TVL/yield aggregator — all data served from DefiLlama REST API, no direct chain access needed | — |
| **Coinglass** | `intelligence_feeds/crypto/coinglass/` | Derivatives Analytics | REST | Multi-exchange (off-chain aggregation) | **NO** | Aggregates liquidation/OI/funding from exchanges via their own pipelines. No chain interaction | — |
| **CoinGecko** | `intelligence_feeds/crypto/coingecko/` | Price Aggregator | REST | Multi-chain (aggregated) | **NO** | Price aggregator — no direct chain reads needed. Pulls from CoinGecko's own index | — |
| **Polymarket** | `prediction/polymarket/` | Prediction Markets | REST (CLOB API) | Polygon (PoS) | **MAYBE — MEDIUM** | Direct on-chain event resolution check via EvmProvider (eth_call to condition contract). Also on-chain order placement via CLOB contracts | P2 |
| **Vertex** | `crypto/cex/vertex/` | Perps DEX | REST | Arbitrum (EVM) | **DEFUNCT** | Service shut down August 14, 2025. Kept for reference only | — |

---

## 2. Detailed Analysis by ChainProvider Type Needed

### 2.1 Solana — SolanaProvider

**Connectors needing it:** Jupiter, Raydium

**Current gap:** Both connectors fully implement quote/routing via REST. The swap endpoint returns a
partially-serialized, unsigned Solana transaction (base64-encoded). To complete a swap:
1. Deserialize the raw bytes into a `solana_sdk::transaction::Transaction`
2. Inject a recent blockhash (`getRecentBlockhash` RPC call)
3. Sign with a `solana_sdk::signature::Keypair`
4. Broadcast via `sendTransaction` RPC

**What ChainProvider adds (concrete):**
- `SolanaProvider::get_recent_blockhash()` — inject fresh blockhash before signing
- `SolanaProvider::broadcast_tx(&[u8])` — send signed bytes via Solana RPC
- `SolanaProvider::spl_token_balance(&Pubkey)` — check token balance before swap
- `SolanaProvider::simulate_transaction(&[u8])` — pre-flight before spending lamports

**Feature gate:** `onchain-solana` → `solana-sdk = "1.18"` + `solana-client = "1.18"`

**Connector fields already prepared:**
```rust
// In jupiter/connector.rs:
#[cfg(feature = "onchain-solana")]
solana_provider: Option<Arc<SolanaProvider>>,

// In raydium/connector.rs:
#[cfg(feature = "onchain-solana")]
solana_provider: Option<Arc<SolanaProvider>>,
```
Both connectors have the field declared and feature-gated. The wire-up is the only missing piece.

---

### 2.2 EVM — EvmProvider (Arbitrum, Ethereum, Avalanche)

**Connectors needing it:** GMX (Arbitrum + Avalanche), Uniswap (Ethereum), Polymarket (Polygon)

**GMX:**
- Already uses `alloy` via `onchain-evm` feature
- `GmxOnchain` struct builds `TransactionRequest` (unsigned)
- Gap: no `LocalWallet` signer injected — caller must sign externally
- ChainProvider would enable: sign multicall tx, submit to Arbitrum RPC, poll for confirmation
- Also needed: ERC-20 `approve()` tx before order submission (token approval check)

**Uniswap:**
- Same pattern as GMX: `UniswapOnchain` builds unsigned `exactInputSingle` tx
- Gap: EIP-1559 fee estimation + Permit2 EIP-712 signing + broadcast
- `erc20_balance()` on `EvmProvider` maps directly to `UniswapOnchain::get_token_balance_onchain()`

**Polymarket:**
- Deployed on Polygon (EVM-compatible)
- Orders in the CLOB API have corresponding on-chain fills
- `eth_call` to condition contract → check if market resolved
- `EvmProvider::eth_call()` enables: read market resolution state without Polymarket REST intermediary
- Low-latency event detection: subscribe to `Transfer(bytes32 conditionId, ...)` events

**What ChainProvider adds for all EVM connectors:**
- Shared `EvmProvider` instance — one HTTP connection pool to Arbitrum/Ethereum/Polygon RPC
- `eth_call()` for contract reads without separate `alloy::Provider` per connector
- `estimate_gas()` for pre-submission fee estimation
- `broadcast_tx()` for unified submission path
- Nonce coordination across multiple connectors on same chain (prevents sequence conflicts)

---

### 2.3 Cosmos — CosmosProvider

**Connectors needing it:** dYdX v4

**Current state:** REST Indexer fully implemented. gRPC `BroadcastTx` wired (behind `grpc` feature).
Missing: Cosmos SDK transaction builder — caller must externally construct `TxRaw` protobuf.

**What ChainProvider adds (concrete):**
- `CosmosProvider::get_account_info(address)` → returns `(account_number, sequence)` for `SignerInfo`
- `CosmosProvider::simulate_tx(&[u8])` → gas estimation for `MsgPlaceOrder` before sending
- `CosmosProvider::broadcast_tx(&[u8])` → send serialized `TxRaw` to validator node
- Sequence coordination: single `CosmosProvider` instance prevents concurrent tx sequence conflicts
  (critical bug without it: two order placements → one rejected with "sequence mismatch")

**Required new dep:** `cosmrs = "0.14"` (feature gate: `onchain-cosmos`)

**Note on sequence conflict severity:** This is a **correctness issue**, not just efficiency.
Without centralized sequence tracking, any two simultaneous calls to `place_order()` from different
tasks will both query the current sequence N, both build tx with sequence N, and one will fail
on-chain. The `CosmosProvider` must atomically increment an in-memory sequence counter.

---

### 2.4 StarkNet — StarkNetProvider

**Connectors needing it:** Paradex

**Current state:** Full REST trading implemented. JWT obtained via pre-provided token in
`Credentials.api_key`. JWT expires every 5 minutes and requires manual refresh.

**What ChainProvider adds:**
- `StarkNetProvider` would hold the StarkNet private key and auto-sign new JWT timestamps
- JWT refresh: `POST /v1/auth` with current timestamp signed via `starknet_crypto::sign()`
- Direct on-chain nonce read via StarkNet RPC (for future on-chain deposit/withdrawal)

**Current dependency:** `starknet-crypto = "0.6"` already in Cargo.toml (behind `starknet` feature)
The connector already has the signing code. `StarkNetProvider` would just manage key storage and
auto-refresh the JWT when it nears expiry.

**Connector field declared:**
```rust
// In paradex/connector.rs:
#[cfg(feature = "onchain-starknet")]
starknet_provider: Option<Arc<StarkNetProvider>>,
```

---

### 2.5 L2 ECDSA — k256 Signing (Not a Full ChainProvider)

**Connectors needing it:** Lighter

**Chain:** ZKLighter zkEVM L2. Uses Ethereum-style ECDSA (secp256k1) but does NOT interact with
Ethereum L1 directly for trading. Orders are submitted via `POST /api/v1/sendTx` with a signed
L2 transaction payload.

**This is NOT a `ChainProvider` use-case** — it is pure signing without chain RPC interaction.
The `k256-signing` Cargo feature already exists. Gap is only wiring `LighterAuth` to use
`k256::ecdsa::SigningKey` for building the L2 transaction signature.

**What needs to happen:**
- Enable `k256-signing` feature in `LighterAuth`
- Build L2 tx bytes (tx_type=14 CreateOrder, tx_type=15 CancelOrder per Lighter spec)
- Sign with `k256::ecdsa::SigningKey::from_bytes(private_key_bytes)`
- POST signed payload to `/api/v1/sendTx`

No `ChainProvider` infrastructure needed — simpler than EVM/Cosmos providers.

---

## 3. Connectors with No ChainProvider Need

| Connector | Reason |
|-----------|--------|
| **Bitquery** | Pure analytics indexer. Covers 20+ chains but only reads aggregated data from Bitquery's own indexing infrastructure. Direct RPC would not add anything — Bitquery already has richer cross-chain queries than any raw RPC |
| **Whale Alert** | Read-only whale monitoring. Whale Alert tracks transactions from their own multi-chain node infrastructure. Direct RPC would duplicate their work without the attribution/labeling layer that makes Whale Alert valuable |
| **Etherscan** | Ethereum block explorer REST API. Already proxies JSON-RPC methods. For gas oracle and block number, the Etherscan API is sufficient; direct RPC only adds value if Etherscan's free tier rate limits become a bottleneck (5 req/s) |
| **DefiLlama** | TVL/yield aggregator. Data is computed by DefiLlama from multiple chains and served hourly via REST. The entire value proposition is the aggregation layer — direct chain reads would give raw pool balances, not the TVL metric that matters |
| **Coinglass** | Derivatives analytics (liquidations, OI, funding). Aggregates from CEX APIs, not from on-chain data. No blockchain interaction in the data pipeline |
| **CoinGecko** | Price/market cap aggregator. Aggregated from hundreds of exchanges. No direct chain reads needed |
| **Hyperliquid** | Already uses EIP-712 + alloy signing internally. Order submission is REST-based (signed JSON, not raw chain tx). No additional ChainProvider needed |

---

## 4. Priority Recommendations

### Priority 0 — Critical (trading blocked without this)

| Task | Connector | Blocker |
|------|-----------|---------|
| Wire `SolanaProvider` into Jupiter `submit_swap()` | Jupiter | Swap execution completely blocked |
| Wire `SolanaProvider` into Raydium `submit_swap()` | Raydium | Swap execution completely blocked |
| Add `cosmrs` + `CosmosProvider`, wire `place_order()` | dYdX v4 | Order placement completely blocked; sequence conflict risk |
| Wire `k256-signing` into `LighterAuth` for `sendTx` | Lighter | All trading blocked; k256 already in Cargo.toml |

### Priority 1 — High (sign-and-send pipeline)

| Task | Connector | Blocker |
|------|-----------|---------|
| Add `LocalWallet` signer to `GmxOnchain`, wire ERC-20 approve check | GMX | Full trading round-trip blocked (tx built, not signed) |
| Add `LocalWallet` + Permit2 signer to `UniswapOnchain` | Uniswap | Swap execution blocked (tx built, not signed) |
| Auto-refresh JWT via `StarkNetProvider` in Paradex | Paradex | Trading fails after 5 min without manual JWT rotation |

### Priority 2 — Medium (new capability)

| Task | Connector | Value |
|------|-----------|-------|
| Add `EvmProvider` (Polygon) to Polymarket for on-chain condition resolution | Polymarket | Real-time market resolution detection without polling REST |
| Wire Bitquery WS subscriptions to `WebSocketConnector` trait | Bitquery | Real-time multi-chain DEX trades and block events |
| Wire Whale Alert WS to `WebSocketConnector` trait | Whale Alert | Real-time whale alert stream (WS URL already stored) |

### Priority 3 — Low (nice to have)

| Task | Connector | Value |
|------|-----------|-------|
| Replace Etherscan proxy endpoints with direct `EvmProvider` fallback | Etherscan | Bypass Etherscan rate limits for `eth_blockNumber` and gas oracle |

---

## 5. ChainProvider Shared Instance Map

This table shows which connectors share the same chain and could share a single provider instance
(avoiding duplicate RPC connection pools):

| Chain | Chain ID | Connectors | Shared Provider |
|-------|----------|-----------|-----------------|
| Ethereum | 1 | Uniswap, (future: Etherscan fallback) | `EvmProvider("ethereum", 1)` |
| Arbitrum | 42161 | GMX (primary), (future: Vertex replacement) | `EvmProvider("arbitrum", 42161)` |
| Avalanche | 43114 | GMX (secondary) | `EvmProvider("avalanche", 43114)` |
| Polygon | 137 | Polymarket | `EvmProvider("polygon", 137)` |
| Solana | — | Jupiter, Raydium | `SolanaProvider("mainnet-beta")` |
| dYdX Chain | "dydx-mainnet-1" | dYdX v4 | `CosmosProvider("dydx-mainnet-1")` |
| StarkNet | — | Paradex | `StarkNetProvider` |
| ZKLighter L2 | — | Lighter | No provider — k256 signing only |

**Key saving:** Jupiter and Raydium both on Solana → one `SolanaProvider` instance instead of two
separate HTTP connection pools to `mainnet-beta.solana.com`.

---

## 6. Feature Gates Required

From `DESIGN_CHAIN_PROVIDERS.md`, the planned feature gates are:

```toml
[features]
# Currently in Cargo.toml:
onchain-ethereum = ["dep:alloy"]           # GMX + Uniswap — DONE
grpc = ["dep:tonic", "dep:prost"]          # dYdX BroadcastTx — PARTIAL
k256-signing = ["dep:k256"]               # Lighter — WIRED TO DEP, NOT TO TRADING
starknet = ["dep:starknet-crypto"]         # Paradex JWT signing — DONE

# To be added (per DESIGN_CHAIN_PROVIDERS.md):
onchain-evm = ["dep:alloy"]               # Alias for onchain-ethereum (all EVM chains)
onchain-solana = ["dep:solana-sdk", "dep:solana-client"]  # Jupiter + Raydium
onchain-cosmos = ["dep:cosmrs"]           # dYdX trading
onchain-starknet = ["dep:starknet-crypto"] # Paradex JWT auto-refresh (alias for starknet)
```

---

## 7. Existing `onchain` Module Structure

```
src/onchain/
├── analytics/
│   ├── bitquery/       — GraphQL blockchain data provider (20+ chains, REST/WS)
│   └── whale_alert/    — Whale transaction monitor (11 chains, REST/WS)
└── ethereum/
    └── etherscan/      — Ethereum block explorer API (ETH + Sepolia, REST only)
```

**Notable absence:** No `src/onchain/solana/`, `src/onchain/cosmos/`, or `src/onchain/evm/` folders.
The `DESIGN_CHAIN_PROVIDERS.md` plans `src/core/chain/` as the home for provider implementations,
not a new `src/onchain/` subfolder. This is intentional — providers are infrastructure, not connectors.

---

## 8. Summary: Decision Matrix

| Connector | Needs ChainProvider? | Which Type | Status | New Dep Needed? |
|-----------|---------------------|------------|--------|-----------------|
| dYdX v4 | **YES** | `CosmosProvider` | Not implemented | `cosmrs = "0.14"` |
| GMX | **YES** | `EvmProvider` (Arbitrum/Avalanche) | Partially done (alloy dep, unsigned tx) | None — alloy already in |
| Jupiter | **YES** | `SolanaProvider` | Field declared, not wired | `solana-sdk`, `solana-client` |
| Lighter | NO (signing only) | k256 ECDSA | Feature in Cargo.toml, not wired | None — k256 already in |
| Paradex | **YES** | `StarkNetProvider` (for JWT refresh) | Field declared, starknet-crypto in | None — dep already in |
| Raydium | **YES** | `SolanaProvider` | Field declared, not wired | `solana-sdk`, `solana-client` |
| Uniswap | **YES** | `EvmProvider` (Ethereum) | Partially done (alloy dep, unsigned tx) | None — alloy already in |
| Bitquery | **NO** | — | Complete for use case | — |
| Whale Alert | **NO** | — | Complete for use case | — |
| Etherscan | **NO** (optional fallback) | `EvmProvider` (Ethereum) | Complete for use case | None |
| Hyperliquid | **NO** | — | Uses alloy internally, no provider needed | — |
| DefiLlama | **NO** | — | Complete for use case | — |
| Coinglass | **NO** | — | Complete for use case | — |
| CoinGecko | **NO** | — | Complete for use case | — |
| Polymarket | **MAYBE** | `EvmProvider` (Polygon) | Not implemented | None — alloy can be reused |
| Vertex | N/A (defunct) | — | Service shut down Aug 2025 | — |
