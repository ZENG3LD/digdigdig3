# Non-EVM Blockchain Ecosystems: Research Report

**Date:** 2026-03-14
**Scope:** DeFi/trading ecosystems, Rust SDK maturity, compile-time impact, integration recommendations

---

## Summary Table

| Chain | TVL (Mar 2026) | Daily Volume | Rust SDK | SDK Maturity | Compile Impact | Recommendation |
|-------|----------------|--------------|----------|--------------|----------------|----------------|
| Bitcoin | ~$1.5B (L2s) | $300-600M BRC-20 | `bitcoin` 0.32.8 | 4/5 | Light | MUST HAVE (monitoring) |
| TON | ~$150-300M | ~$100M | `tonlib-rs` (C FFI) | 2/5 | Heavy (C build) | NICE TO HAVE |
| Sui | ~$640M-2.6B | ~$200M | `sui-sdk` | 3/5 | Medium | MUST HAVE |
| Aptos | ~$500M-1B | ~$1B weekly | `aptos-sdk` | 2/5 | Heavy (tokio_unstable) | NICE TO HAVE |
| Cosmos/Osmosis | ~$50-200M | ~$8M/day | `cosmrs` | 4/5 | Medium | MUST HAVE |
| Injective | ~$10-55M | significant | `cosmrs` (reuse) | 4/5 | Medium (reuse) | MUST HAVE (via CosmosProvider) |
| NEAR | ~$148M | ~$17M/day | `near-api-rs` | 3/5 | Medium | NICE TO HAVE |
| Tezos | ~$45M | minimal | `tezos-rust-sdk` | 2/5 | Medium | SKIP |
| Polkadot | ~$180M total | minimal | `subxt` | 3/5 | Heavy (codegen) | SKIP |
| Cardano | ~$350-680M | ~$1.1B monthly | `cardano-serialization-lib` | 2/5 | Heavy | SKIP |
| Algorand | ~$103-188M | minimal | `algonaut` (community) | 1/5 | Light | SKIP |

---

## 1. Bitcoin

### DeFi Ecosystem

Bitcoin itself has no native smart contracts, but several layers enable DeFi-adjacent activity:

**Ordinals / Inscriptions**
- Arbitrary data inscribed onto satoshis via the Ordinals protocol (Casey Rodarmor, 2023)
- NFT-equivalent assets stored entirely on-chain in witness data
- Marketplaces: Magic Eden, OKX Ordinals

**BRC-20**
- Fungible token standard built on Ordinals (JSON inscriptions)
- Still dominant: ~$633M in on-chain volume over 6 months as of mid-2025
- EVM functionality being added via BRC2.0 upgrade (EVM smart contracts on Bitcoin, no bridges needed)
- Not native tokens — indexer-tracked state

**Runes**
- Launched at Bitcoin halving (April 2024), Casey Rodarmor's replacement for BRC-20
- UTXO-native fungible tokens (cleaner than BRC-20's indexer-based approach)
- Lower volume than BRC-20 currently (roughly half)

**Lightning Network**
- Payment channel network, NOT DeFi
- TVL: ~$335.9M (largest Bitcoin L2 by TVL)
- Useful for real-time payment monitoring, not trading terminal data

**Stacks (STX) — Bitcoin L2**
- EVM-compatible smart contracts anchored to Bitcoin via PoX (Proof of Transfer)
- TVL: ~$208M (as of 2025), sBTC growing toward $545M
- Active DeFi: lending, DEX, etc.
- Named "#1 Bitcoin growth network in 2026" by Tenero

**Total Bitcoin DeFi TVL:** Bitcoin ecosystem DeFi grew $1.2B YoY to reach approx. $1.5B+ across all L2s/overlays.

### What Data Can We Get via RPC?

Via `bitcoincore-rpc` against a full node:
- Mempool state (unconfirmed txs, fee rates)
- UTXO set queries
- Block data, tx data
- **Inscriptions/Ordinals/Runes**: Requires a specialized indexer (ord daemon) — not in core RPC

Useful for:
- On-chain fee monitoring (mempool congestion)
- BTC price proxy data
- Stacks: separate JSON-RPC API

### Rust SDK

**`bitcoin` crate (rust-bitcoin)**
- Version: 0.32.8 (released 2025-12-06)
- Actively maintained by the rust-bitcoin community
- Dependencies: `bech32`, `bitcoin_hashes`, `secp256k1`, `bitcoin-internals`, `hex-conservative`, `bitcoin-units`, `bitcoin-io`
- Optional: `serde`, `base64`, `bitcoinconsensus`
- `no-std` support available
- Compile impact: **Light** — pure Rust, small dep tree, no C FFI (secp256k1 has optional C backend via feature flag)
- Feature-gating: Yes — `bitcoin = { features = ["serde"] }` selectively

**`bitcoincore-rpc`**
- RPC client for Bitcoin Core daemon
- Lightweight, serde-based
- Good for raw mempool/block data

**For Ordinals/Runes:**
- No dedicated Rust crate; requires running `ord` indexer and querying its REST API
- Could be wrapped as a simple `reqwest`-based client

### Recommendation: MUST HAVE (monitoring)

Bitcoin is the largest crypto by market cap. For a trading terminal:
- BTC price via exchange connectors (already covered)
- Mempool fee monitoring (on-chain) — `bitcoincore-rpc`
- BRC-20/Runes tracking — ord indexer REST API (custom reqwest wrapper)
- No full DeFi integration needed at first

---

## 2. Tezos (XTZ)

### DeFi Ecosystem

- **Total TVL:** ~$45.1M (Q3 2025, +13.1% QoQ)
- **Youves:** Largest protocol at $31.4M TVL (synthetic assets, stablecoins)
- **QuipuSwap:** AMM DEX, TVL ~$1.2M, ~306 weekly users
- **Plenty:** DEX with stable and volatile pools, bridging from ETH/Polygon
- Recent surge: +$30M liquidity injection caused a 164% trading volume spike (temporary)

**Assessment:** Very small ecosystem. ~$45M TVL puts it below 40th place globally. Weekly active users in the hundreds. XTZ market cap ~$1B but minimal DeFi relative to chain size.

### Rust SDK

**`tezos-rust-sdk` (airgap-it)**
- Multi-crate project: crypto, RPC client, transaction building
- Not on crates.io prominently; GitHub-only distribution
- Status: Maintained but niche; very small community
- Maturity: 2/5

**`tezos_crypto_rs`**
- Cryptographic primitives for Tezos hashing/signing
- Available on crates.io

**`tezedge-client`**
- Tezos client/wallet in Rust (TezEdge project)
- TezEdge node development has **ceased** (see GitHub note)

**Note:** No officially supported Rust SDK from the Tezos Foundation. The ecosystem prefers OCaml (the Octez client) and JavaScript/TypeScript.

### Recommendation: SKIP

TVL too small ($45M), user activity minimal, no first-party Rust SDK, ecosystem stagnant. Not worth the maintenance burden.

---

## 3. TON (Telegram Open Network)

### DeFi Ecosystem

- **TVL:** ~$150-300M (estimates vary; DeDust peaked at $379M in late 2024, more recent ~$150M)
- **STON.fi:** Dominant DEX — 80% of all TON traders, >$6B cumulative volume, 27M+ transactions. Raised $9.5M Series A (Ribbit Capital + CoinFund).
- **DeDust:** Second largest AMM DEX
- **Growing catalysts (2026):** Telegram designated TON as exclusive blockchain for Mini Apps; self-custodial wallet launched for US users (Jan 2026); MoonPay cross-chain deposits enabled (Feb 2026); TON Storage (Q1 2026); Bitcoin bridge (mid-2026)
- **User base:** Access to 100M+ Telegram users is the key differentiator

### Rust SDK

**`tonlib-rs` (ston-fi)**
- GitHub: `ston-fi/tonlib-rs` — Rust SDK for TON
- Architecture: Wraps `tonlib-sys` which builds `tonlibjson_static` via CMake
- **C FFI issues:** Requires compiling the C++ TON library. Since 2024.6.1, depends on a forked version of TON. Complex build setup.
- Cross-compilation is painful; Windows builds are notoriously difficult
- Compile impact: **Heavy** — C++ build, CMake dependency, linker complexity
- Alternative: Pure HTTP API via `ton-api` endpoints (no SDK needed for read-only data)
- Maturity: 2/5

**Alternative approach for TON:**
TON has a public HTTP API (`toncenter.com/api`) and WebSocket support that can be accessed via plain `reqwest` without any native binding. For a read-only trading terminal (price feeds, TVL, swap events), this is more practical.

### Recommendation: NICE TO HAVE

Massive potential due to Telegram integration, but the Rust SDK is technically painful (C FFI, CMake builds). For the terminal, implement as a thin HTTP API client wrapping `toncenter.com/api` or STON.fi REST API. Feature-gate behind `feature = "ton"`.

---

## 4. Sui

### DeFi Ecosystem

- **TVL:** ~$640M-$2.6B (sources vary; Defiant reported $2.6B record; recent whale inflow surge to $643M in one session — March 2026)
- **Cetus:** ~$220M TVL, $13B+ cumulative volume, ~2M users, up to $194M daily volume
- **DeepBook:** Native order book DEX (fully on-chain limit orders), $8.7B lifetime volume, 10M+ users. 2026 roadmap: native margin trading, gasless transactions for DEEP stakers
- **Growth:** TVL grew from <$500M (early 2024) to $2B+ (Jan 2025)
- **Performance:** Move language, parallel execution, 297K TPS theoretical

### Rust SDK

**`sui-sdk` (MystenLabs)**
- Location: `crates/sui-sdk` in the main Sui monorepo
- Two variants:
  - **Legacy `sui-sdk`**: JSON-RPC based, stable, supports events/WebSocket
  - **New `sui-rust-sdk`**: Cleaner API, no JSON-RPC requirement, recommended for new projects
- Capabilities: Coin reads, event subscriptions, governance, PTBs (programmable transaction blocks), WebSocket streams
- Dependencies: substantial (lives in Sui monorepo — many internal crates)
- Compile impact: **Medium** — pure Rust but large dep tree from monorepo
- Status: Actively maintained by MystenLabs, well-documented
- Limitation (noted Jul 2025): "Rudimentary for beginners; no helper to auto-manage gas coins; fetching object fields in JSON not provided by default"
- Maturity: 3/5 (functional but API rough edges)

**Feature-gating:** Yes — can depend only on `sui-sdk-types` for data structures without the full client.

### Recommendation: MUST HAVE

$640M-$2.6B TVL, rapidly growing ecosystem, active order book (DeepBook), institutional adoption, best Move-chain for DeFi. Sui has momentum comparable to early-era Solana.

---

## 5. Aptos

### DeFi Ecosystem

- **TVL:** ~$500M-$1B
- **Stablecoins:** $1.15B+ on-chain
- **Weekly DEX volume:** ~$1B
- **Hyperion:** ~$97.9M TVL, $13.2B+ cumulative volume (as of mid-2025)
- **Liquidswap (Pontem):** Largest AMM by TVL, 25+ assets, triple audited
- **PancakeSwap:** Multi-chain deployment on Aptos
- **Daily transactions:** ~1.2M/day (47% growth over 30 days)
- **Fees:** Dropped 61% QoQ to ~$0.00052 avg — ultra-cheap

### Rust SDK

**`aptos-sdk`**
- Official but labeled "lightly supported" by Aptos Labs
- **Critical issue:** Requires `tokio_unstable` flag globally via `.cargo/config.toml`:
  ```toml
  [build]
  rustflags = ["--cfg", "tokio_unstable"]
  ```
- Also requires git patches in `Cargo.toml` for `merlin` and `x25519-dalek`
- This flag affects the **entire workspace** — not feature-gatable cleanly
- Preferred Aptos SDKs: TypeScript, Go, Python (all better maintained)
- **`aptos-rust-sdk`** (newer): A cleaner alternative in a separate repo (`aptos-labs/aptos-rust-sdk`), potentially without the `tokio_unstable` requirement — but less mature
- Maturity: 2/5 (official-but-lightly-supported is telling)
- Compile impact: **Heavy** (tokio_unstable breaks normal workspace builds)

### Recommendation: NICE TO HAVE

Strong ecosystem ($500M+ TVL, $1B weekly volume) but the SDK is a workspace contaminant (`tokio_unstable` poisons the build config). Implement via REST API (`https://fullnode.mainnet.aptoslabs.com/v1`) using plain `reqwest` — no SDK needed for read-only terminal data. Feature-gate behind `feature = "aptos"`.

---

## 6. NEAR Protocol

### DeFi Ecosystem

- **TVL:** ~$148.2M (RHEA Finance holds 95.2% after absorbing Ref Finance + Burrow)
- **RHEA Finance:** Merged protocol, $148M TVL, $16.2M daily DEX volume
- **Daily DEX volume:** ~$17M (+101% QoQ growth)
- **Ecosystem direction:** Heavy focus on AI agents ("Chain Abstraction" — NEAR AI)
- **Aurora:** EVM-compatible L2 on NEAR (handles EVM dApps)

### Rust SDK

**`near-api-rs`** (recommended as of 2025 roadmap)
- Strategic replacement for `nearcore` dependencies
- Undergoing final ergonomics testing before v1.0 stabilization
- Goal: decouple from `nearcore`'s frequent 0.x version bumps

**`near-jsonrpc-client-rs`**
- Low-level JSON-RPC client for NEAR
- Transitioning to v1.0 (formerly `near-openapi-client-rs`)

**Maturity challenges:**
- `nearcore` bumps 0.x versions frequently → cascading breaking changes
- The 2025 stabilization push is a response to this pain
- 1.x stable releases promised by end-2025 but timeline unclear
- Maturity: 3/5 (improving)

**Compile impact:** Medium — pure Rust, reasonable dep tree once decoupled from nearcore

### Recommendation: NICE TO HAVE

$148M TVL is modest but the AI agent angle (NEAR AI, intents) makes it interesting for the "terminal as agent API" vision. `near-api-rs` should be usable once it hits 1.0. Feature-gate behind `feature = "near"`.

---

## 7. Polkadot / Substrate

### DeFi Ecosystem

**Polkadot parachains TVL (late 2025):**
- Acala (DeFi hub): ~$69M
- Bifrost (liquid staking): ~$44M
- Hydration (formerly HydraDX): ~$41M
- Astar: ~$30M
- Total Polkadot ecosystem: ~$180M

**Key context:**
- Moonbeam (largest parachain by DeFi activity) is **EVM-compatible** — already covered by EVM connectors
- Acala has stablecoin (aUSD), DEX, liquid staking
- Hydration: specialized AMM with omnipool design
- DOT itself: governance token, not DeFi asset
- Parachain model is declining in favor of the new "coretime" model (Polkadot 2.0)

### Rust SDK

**`subxt`** (paritytech)
- "Submit Extrinsics" — type-safe Substrate client in Rust
- Active development, well-maintained by Parity
- **Compile-time metadata codegen:** Downloads chain metadata at build time, generates typed Rust structs via `#[subxt::subxt]` macro
- Supports `no-std` via `subxt-core`
- Can connect to any Substrate-based chain (Polkadot, Kusama, Acala, Hydration, etc.)
- **Compile impact: Heavy** — codegen macro downloads chain metadata, generates large code; build time is significant
- Maturity: 3/5

**Feature-gating:** Possible but the compile-time metadata download is awkward for CI

### Recommendation: SKIP

Despite usable tooling, the DeFi ecosystem is fragmented across parachains, TVL is modest ($180M total), and the compile-time metadata download model adds significant complexity. Moonbeam (the most active parachain for DeFi) is already covered via EVM. The trade-off is not worth it for a trading terminal.

---

## 8. Cardano (ADA)

### DeFi Ecosystem

- **TVL:** $350-680M (sources vary; Defiant/LeveX report $300-680M range)
- **Monthly DEX volume:** ~$1.1B (Minswap + SundaeSwap + others)
- **Minswap:** Largest DEX, ~$77.4M TVL, top by transaction volume and users
- **SundaeSwap:** ~$12M TVL (+77% growth)
- **Active protocols:** 70%+ of smart contract interactions are on DEXs
- **Catalyst:** USDCx stablecoin launched on Cardano network
- **Hydra:** Layer-2 scaling solution (in development)
- **eUTXO model:** Extended UTXO, fundamentally different from account-based chains

### Rust SDK

**`cardano-serialization-lib` (EMURGO)**
- Rust library for Cardano data structures (serialization/deserialization)
- Primary use: building transactions, not chain querying
- No built-in RPC/chain interaction — just encoding/decoding
- Compile impact: Heavy (complex CBOR handling, many Cardano-specific types)

**`cardano-multiplatform-lib` (dcSpark)**
- Auto-generated from Cardano's CDDL spec via `cddl-codegen`
- Handles CBOR details correctly, more compatible
- Also Rust-first (WASM compilation target)
- Still not an RPC client

**For chain data:** Must use Blockfrost API (`blockfrost.io`) or Koios (decentralized API) via HTTP — no native Rust RPC client

**Maturity:** 2/5 — Rust is used for WASM targets but not as a chain interaction library; community maintains tooling, no Cardano Foundation official Rust SDK

### Recommendation: SKIP

Despite $350-680M TVL and $1.1B monthly DEX volume, Cardano's eUTXO model makes DeFi integration extremely complex. No first-party Rust RPC client exists. The ecosystem prefers Haskell (Plutus), JavaScript, and Python. Maintenance overhead is too high.

---

## 9. Algorand (ALGO)

### DeFi Ecosystem

- **TVL:** ~$103-188M (surge to $188M in early 2026, then correction to ~$103M)
- **Tinyman:** Largest DEX, TVL $6.2M, volume -63% YoY
- **Folks Finance:** Growing lending protocol
- **Pact:** AMM DEX
- **ALGO market cap:** ~$1.2B but DeFi activity is minimal relative to market cap

**Assessment:** Despite 170%+ TVL growth from ~$70M, actual DeFi usage is tiny. Tinyman's volume decline of 63% is a red flag. The ecosystem is small and user numbers are low.

### Rust SDK

**`algonaut`** (community, manuelmauro)
- "A rusty SDK for Algorand" — community project, NOT officially supported by Algorand Foundation
- Status: Work in progress, limited maintenance activity
- Official Algorand SDKs: JavaScript, Python, Java, Go — Rust is not on the official list
- Alternative: `algosdk` crate exists but also community-maintained
- Maturity: 1/5

**Compile impact:** Light (pure Rust, small dep tree)

### Recommendation: SKIP

Very small DeFi ecosystem (effective TVL under $100M), declining volumes, no official Rust SDK. Not worth supporting.

---

## 10. Cosmos IBC Chains

### Overview

The Cosmos ecosystem consists of sovereign app-chains connected via IBC (Inter-Blockchain Communication Protocol). As of 2026, 115+ chains connect via IBC with >$1B monthly cross-chain volume. Key chains for a trading terminal:

---

### 10a. Osmosis

- **TVL:** ~$50M (Defiant data); $40B+ lifetime volume
- **Daily volume:** ~$8.36M
- **Role:** Premier DEX and cross-chain hub for Cosmos. Connects 140+ blockchains via IBC.
- **Protocol:** Concentrated liquidity AMM, superfluid staking, custom AMM pools
- **Note:** Volume down 46% MoM — ecosystem is mature but facing competition from EVM chains

### 10b. Injective

- **TVL:** ~$10.8M on-chain (DefiLlama, down 15%) — but this understates activity
- **Cumulative trading volume:** $55B+
- **Monthly active addresses:** 561K (doubled from 291K in early 2025)
- **Architecture:** Cosmos SDK app-chain purpose-built for finance
  - Fully on-chain order book (derivatives + spot)
  - 0.65-second block time, 25,000+ TPS
  - Zero gas fees for traders
  - Cross-chain via IBC + bridges (ETH, BSC, Solana)
- **Key differentiation:** The only Cosmos chain with a native on-chain order book — closer to dYdX than Osmosis
- **2026 event:** Injective Summit (July 2026, Washington DC)

### 10c. Sei

- **TVL:** $45.9M (DefiLlama) or up to $609M (Yei Finance accounts for $381-386M)
- **Architecture:** Pivoted from pure Cosmos to EVM-compatible (Sei V2) — parallelized EVM, sub-400ms finality, 200K TPS theoretical
- **Note:** Sei V2 means it is now **EVM-compatible** — existing EVM connectors may work
- **Yei Finance:** Top protocol, $381-386M TVL (>50% of network)
- **AI integration:** MCP support for AI agents to interact directly with Sei

### 10d. Neutron

- **TVL:** ~$35M (Stride integration)
- **Role:** First Cosmos Hub "consumer chain" (ICS security)
- **Focus:** Smart contracts on Cosmos (CosmWasm), DeFi infrastructure
- **Revenue share:** 25% of tx fees + MEV to ATOM stakers/validators

### Can We Reuse CosmosProvider?

**Yes, absolutely.** Since all Cosmos chains use the same:
1. Tendermint/CometBFT consensus → same block/tx query API
2. `cosmrs` / `cosmos-sdk-proto` gRPC interface
3. IBC for cross-chain transfers

A single `CosmosProvider` with configurable `chain_id`, `rpc_url`, and `grpc_url` parameters covers all of them. The `cosmrs` crate (version 0.19.0, min Rust 1.72) supports:
- Transaction building and signing
- gRPC queries (via `cosmos-sdk-proto`)
- IBC transfers

**Injective requires special handling:**
Injective has its own protobuf types (`injective-core` repo) for its exchange module (order books, derivatives). These are separate from standard Cosmos SDK protos. Either:
- Use `injective-rs` if available
- Call Injective's REST API (`api.injective.exchange`) directly via `reqwest`
- Use the Exchange gRPC API with custom proto compilation

### Cosmos Rust SDK

**`cosmrs`**
- Version: 0.19.0 (min Rust 1.72)
- Maintainers: Informal Systems, Iqlusion, Confio, Althea
- 314 commits, 348 stars — active maintenance
- Compile impact: **Medium** — gRPC + proto generation adds build time
- Maturity: 4/5

**`cosmos-sdk-proto`**
- Auto-generated protobuf types for Cosmos SDK
- Paired with `cosmrs`
- Required for any gRPC-based chain interaction

**Feature-gating:** Yes — `cosmrs = { features = ["cosmwasm"] }` for CosmWasm support

### Recommendation: MUST HAVE (Osmosis + Injective minimum)

Cosmos chains share infrastructure (CosmosProvider is reusable). Osmosis is the Cosmos DeFi hub. Injective is unique — native on-chain order book with $55B cumulative volume. Both are high-value additions. Sei may become relevant as its EVM compatibility matures.

---

## Rust SDK Maturity Ratings (Detailed)

| SDK | Crate | Version | Stars | Rust-only? | Compile Impact | Notes |
|-----|-------|---------|-------|------------|----------------|-------|
| `bitcoin` | `rust-bitcoin` | 0.32.8 | 2000+ | Yes (optional C secp256k1) | Light | Best non-EVM SDK in Rust |
| `tonlib-rs` | `ston-fi/tonlib-rs` | 0.17.6 | ~200 | No (C++ tonlib) | Heavy | CMake + C++ FFI, painful on Windows |
| `sui-sdk` | MystenLabs monorepo | latest | 5000+ | Yes | Medium | Monorepo dep tree; well-maintained |
| `aptos-sdk` | `aptos-sdk` | 0.x | 4000+ | Yes but `tokio_unstable` | Heavy (config poison) | Workspace contaminant |
| `cosmrs` | `cosmos/cosmos-rust` | 0.19.0 | 350 | Yes | Medium | gRPC-based, clean API |
| `near-api-rs` | NEAR official | 0.x (pre-1.0) | ~300 | Yes | Medium | Stabilization in progress |
| `subxt` | paritytech | 0.42.x | 700+ | Yes | Heavy (codegen) | Build-time metadata download |
| `tezos-rust-sdk` | airgap-it | 0.x | ~100 | Yes | Medium | Small community, not on crates.io |
| `algonaut` | community | 0.x | ~200 | Yes | Light | Unmaintained community project |
| `cardano-serialization-lib` | EMURGO | 15.x | 500+ | Yes (WASM target) | Heavy | Not an RPC client |

---

## Feature-Gating Strategy

All non-EVM chain integrations should be feature-gated in `Cargo.toml`:

```toml
[features]
default = []

# Tier 1: MUST HAVE
bitcoin = ["dep:bitcoin", "dep:bitcoincore-rpc"]
cosmos = ["dep:cosmrs", "dep:cosmos-sdk-proto", "dep:tonic"]
sui = ["dep:sui-sdk"]

# Tier 2: NICE TO HAVE (HTTP-only, no heavy SDKs)
ton = []       # Pure reqwest, toncenter.com API
aptos = []     # Pure reqwest, Aptos REST API
near = ["dep:near-api-rs"]

# Tier 3: Future consideration
# polkadot = ["dep:subxt"]  -- heavy compile, skip for now
```

**Key principle:** TON and Aptos can be implemented as thin `reqwest`-based HTTP clients with zero heavy SDK dependencies. Their native Rust SDKs add too much build complexity for a monitoring terminal.

---

## Prioritized Implementation Order

### Phase 1 — High ROI (implement now)
1. **Cosmos/Osmosis** — Reuse `cosmrs`, cover entire IBC ecosystem at once. $40B+ lifetime Osmosis volume, Injective $55B.
2. **Sui** — $640M-$2.6B TVL, rapidly growing, clean Rust SDK, DeepBook order book is terminal-relevant.

### Phase 2 — Medium ROI (next quarter)
3. **Bitcoin** — Mempool monitoring + BRC-20/Runes tracking via `bitcoin` crate + `ord` REST API. Lightweight.
4. **Injective** — Reuse CosmosProvider + custom proto for exchange module. Native order book = high value for trading terminal.

### Phase 3 — Low priority (future)
5. **NEAR** — Wait for `near-api-rs` v1.0 stabilization. Then implement via HTTP API.
6. **TON** — Implement as HTTP client only (`toncenter.com` REST). Skip C FFI SDK.
7. **Aptos** — Implement via REST API only. Skip SDK due to `tokio_unstable` workspace contamination.

### Never (SKIP)
- Tezos — Too small, no Rust SDK ecosystem
- Polkadot/Substrate — Fragmented, EVM parachains already covered, heavy compile
- Cardano — eUTXO complexity, no RPC client, niche language preferences
- Algorand — Effectively dead DeFi, community-only SDK

---

## Sources

- [DeFiLlama Chain Rankings](https://defillama.com/chains)
- [DeFiLlama Sui](https://defillama.com/chain/sui)
- [DeFiLlama Injective](https://defillama.com/chain/injective)
- [DeFiLlama Osmosis](https://defillama.com/protocol/osmosis-dex)
- [Sui TVL Record $2.6B — The Defiant](https://thedefiant.io/news/blockchains/sui-tvl-hits-record-usd2-6-billion-amid-defi-growth)
- [Sui Rust SDK Documentation](https://docs.sui.io/references/rust-sdk)
- [Aptos Rust SDK Documentation](https://aptos.dev/build/sdks/rust-sdk)
- [NEAR Rust Tooling 2025](https://docs.near.org/blog/near-rust-devtools-2025)
- [cosmrs on crates.io](https://crates.io/crates/cosmrs)
- [cosmos/cosmos-rust on GitHub](https://github.com/cosmos/cosmos-rust)
- [tonlib-rs (ston-fi)](https://github.com/ston-fi/tonlib-rs)
- [STON.fi $9.5M Series A — The Block](https://www.theblock.co/press-releases/364796/ston-fi-dev-raises-9-5m-series-a-to-scale-defi-on-ton)
- [rust-bitcoin on docs.rs](https://docs.rs/crate/bitcoin/latest)
- [rust-bitcoin on GitHub](https://github.com/rust-bitcoin/rust-bitcoin)
- [Tezos Q3 2025 — Messari](https://messari.io/report/state-of-tezos-q3-2025)
- [Cardano DeFi $300M TVL — LeveX](https://levex.com/en/blog/cardano-defi-ecosystem-explodes-300m-tvl-growth)
- [Injective Statistics 2026](https://coinlaw.io/injective-statistics/)
- [subxt on Polkadot Docs](https://docs.polkadot.com/develop/toolkit/api-libraries/subxt/)
- [Algonaut (algonaut)](https://github.com/manuelmauro/algonaut)
- [DWF Labs: Bitcoin Ordinals & Runes Landscape](https://www.dwf-labs.com/research/391-ordinals-runes-landscape-summary)
- [Stacks TVL — OKX](https://www.okx.com/en-us/learn/stacks-tvl-stx-bitcoin-defi-growth)
- [Lightning Network TVL — DefiLlama](https://defillama.com/protocol/lightning-network)
- [Sei Growth — AInvest](https://www.ainvest.com/news/sei-growth-momentum-tvl-expansion-deep-dive-chain-scalability-tokenomics-alignment-2509/)
- [NEAR RHEA Finance — blog.ju.com](https://blog.ju.com/rhea-finance-near-defi-guide/)
- [Osmosis $40B Volume](https://coinmarketcap.com/currencies/osmosis/)
- [Aptos DeFi Overview](https://medium.com/@chaisomsri96/aptos-defi-an-overview-and-outlook-of-the-ecosystem-439f8dec23b1)
