# SDK Availability Research: Non-REST Transport Connectors

**Date:** 2026-03-14
**Scope:** Official Rust SDKs, community crates, proto/schema files, and official TS/Python SDKs for 9 connectors requiring non-REST transport.

---

## 1. dYdX v4

| Field | Value |
|-------|-------|
| Official Rust SDK | YES — `dydx` crate (v0.3.0, Oct 2025) |
| Community Rust | YES — `dydx-proto` (proto bindings only) |
| Proto/Schema files | https://github.com/dydxprotocol/v4-chain (Cosmos/gRPC proto definitions) |
| Official TS SDK | https://github.com/dydxprotocol/v4-clients/tree/main/v4-client-js |
| Official Python SDK | https://github.com/dydxprotocol/v4-clients/tree/main/v4-client-py |
| Effort to integrate | LOW |

### Notes
- The `dydx` crate on crates.io (v0.3.0) is the **official Rust client**, maintained by **Nethermind** in collaboration with dYdX protocol team. It is listed in the official dYdX documentation at https://docs.dydx.xyz/interaction/client/quick-start-rs
- Transports supported: **gRPC** (NodeClient via `https://dydx-ops-grpc.kingnodes.com:443`), **WebSocket** (IndexerClient via `wss://indexer.dydx.trade/v4/ws`), **REST** (IndexerClient HTTP)
- Features: `NodeClient`, `IndexerClient` + WebSockets, `FaucetClient`, `NobleClient`
- Fully async (Tokio), automatic WebSocket reconnect, telemetry, builder pattern for requests
- Cosmos SDK under the hood — gRPC is the primary transport for order placement
- `dydx-proto` crate provides the raw protobuf-generated types if lower-level access is needed
- Crates.io: https://crates.io/crates/dydx | https://crates.io/crates/dydx-proto

---

## 2. Jupiter

| Field | Value |
|-------|-------|
| Official Rust SDK | YES — `jup-ag-sdk` (v1.0.6, official DevRel repo) |
| Community Rust | YES — `jup-ag` (mvines), `jup-lend-sdk` |
| Proto/Schema files | N/A (REST + WebSocket JSON API, no proto) |
| Official TS SDK | https://github.com/jup-ag/jupiter-swap-api-client |
| Official Python SDK | None official (community wrappers exist) |
| Effort to integrate | LOW |

### Notes
- `jup-ag-sdk` (v1.0.6) is maintained by **Jupiter-DevRel** (official Jupiter development team) at https://github.com/Jupiter-DevRel/jup-rust-sdk
- Covers **all Jupiter APIs**: Ultra, Swap, Trigger, Recurring, Token, Price — strongly typed with builder pattern
- Transport: REST/HTTPS + Solana RPC WebSocket for transaction confirmation
- No protobuf — all JSON REST APIs
- Alternative older community crate: `jup-ag` at https://crates.io/crates/jup-ag (mvines, less maintained)
- Crates.io: https://crates.io/crates/jup-ag-sdk

---

## 3. Raydium

| Field | Value |
|-------|-------|
| Official Rust SDK | PARTIAL — `raydium-library` (official, CLI/instruction builder) |
| Community Rust | YES — `raydium-amm-swap`, `raydium_clmm`, `raydium_cpmm`, `raydium-sdk-V2` (partial port) |
| Proto/Schema files | N/A (Solana on-chain programs + REST API, no proto) |
| Official TS SDK | https://github.com/raydium-io/raydium-sdk-V2 (`@raydium-io/raydium-sdk-v2` on npm) |
| Official Python SDK | None official |
| Effort to integrate | MEDIUM |

### Notes
- Official Rust library at https://github.com/raydium-io/raydium-library — covers CLMM, CP Swap (CPMM), and AMM; provides instruction generation and amounts calculation, **not a high-level trading SDK**
- `raydium-amm-swap` (v0.1.21) on crates.io: MIT, aimed at executing swaps
- `raydium_clmm` (v0.1.13) and `raydium_cpmm` (v0.1.13): Raydium clients for concentrated liquidity and constant product pools
- Transport: Solana **WebSocket** (subscriptions to account changes) + **RPC** (transaction submission); no REST needed for on-chain interaction
- Integration requires `solana-client`, `anchor-client` or `solana-sdk` for transaction building and signing
- Community crate `raydium-sdk-V2` is a partial Rust adaptation of the TypeScript V2 SDK
- The TypeScript V2 SDK is the most complete reference: https://github.com/raydium-io/raydium-sdk-V2

---

## 4. Lighter

| Field | Value |
|-------|-------|
| Official Rust SDK | NO (no official Rust SDK from elliottech) |
| Community Rust | YES — `lighter-rs` at https://github.com/0xvasanth/lighter-rs |
| Proto/Schema files | N/A (JSON REST API + WebSocket, signing via Poseidon2/Schnorr) |
| Official Python SDK | https://github.com/elliottech/lighter-python |
| Official Go SDK | https://github.com/elliottech/lighter-go |
| Effort to integrate | HIGH |

### Notes
- Official SDKs from **Elliot Technologies**: Python and Go only. No official Rust SDK.
- Community crate `lighter-rs` (https://github.com/0xvasanth/lighter-rs) is **production-ready** with 41 unit tests passing against mainnet:
  - Implements: market orders, limit orders, stop loss, take profit, order modification, cancellation, position management, leverage control, fund transfers, liquidity pool operations
  - Cryptography: **Poseidon2 hashing + Schnorr signatures** (NOT secp256k1 — custom ZK scheme ported from the official lighter-go implementation)
  - Dependencies: `goldilocks-crypto`, `poseidon-hash` crates
  - Security caveat: not independently audited ("ported from official lighter-go, not audited")
- Transport: REST API at `https://mainnet.zklighter.elliot.ai` + **WebSocket** for streaming
- Signing scheme is ZK-native (Plonky2 field arithmetic), significantly more complex than ECDSA
- API docs: https://apidocs.lighter.xyz

---

## 5. Paradex

| Field | Value |
|-------|-------|
| Official Rust SDK | NO (community only) |
| Community Rust | YES — `paradex` crate v0.7.2 (https://crates.io/crates/paradex) |
| Proto/Schema files | N/A (JSON REST + WebSocket) |
| Official TS SDK | https://www.npmjs.com/package/@paradex/sdk |
| Official Python SDK | https://tradeparadex.github.io/paradex-py/ (tradeparadex/paradex-py) |
| Effort to integrate | HIGH |

### Notes
- Community Rust crate `paradex` (v0.7.2) at https://github.com/snow-avocado/paradex-rs — MIT license, ~3.3K SLoC; covers REST + WebSocket
- Authentication is **StarkNet-based** (L2), extremely complex multi-step process:
  1. Sign message with **Ethereum L1 private key** → deterministic signature
  2. Use signature to derive **StarkNet L2 private key** (Paradex private key)
  3. L2 private key → L2 public key → Account Contract deployed on StarkNet
  4. Sign JWT payload with L2 key using **EIP-712 typed data standard** adapted for StarkNet
  5. JWT returned; used as Bearer token for all subsequent authenticated requests
- The community `paradex` crate depends on `starknet-core 0.16.0`, `starknet-crypto 0.8.1`, `starknet-signers 0.14.0`
- Official code samples (Python, Go, Groovy/Java, TypeScript, Rust): https://github.com/tradeparadex/code-samples
- Rust sample in the official code-samples repo represents ~1.7% of codebase — minimal but present
- Transport: HTTPS REST + **WebSocket** for order book / trade streaming

---

## 6. GMX v2

| Field | Value |
|-------|-------|
| Official Rust SDK | NO |
| Community Rust | NO dedicated crate (use `alloy` + ABI files manually) |
| ABI files | https://github.com/gmx-io/gmx-synthetics/tree/main/abis (130+ JSON ABIs) |
| Official TS SDK | `@gmx-io/sdk` npm package (https://github.com/gmx-io/gmx-interface/tree/master/sdk) |
| Official Python SDK | None |
| Effort to integrate | HIGH |

### Notes
- GMX v2 is an **EVM on-chain DEX** (Arbitrum, Avalanche) — interaction requires ABI-based contract calls, no REST API for trading
- No Rust SDK exists; the recommended approach is to use the **`alloy` crate** (Rust's Ethereum library, successor to ethers-rs) with ABI JSON files from the gmx-synthetics repo
- `alloy` provides the `sol!` macro for compile-time ABI embedding: `sol!(include_str!("abis/ExchangeRouter.json"))`
- Key contracts in `abis/` directory: `ExchangeRouter.sol`, `Router.sol`, `Reader.sol`, `DataStore.sol`, `MarketFactory.sol`, `OrderHandler.sol`, `DepositHandler.sol`, `WithdrawalHandler.sol`
- Full ABI list (130+ contracts per chain): https://github.com/gmx-io/gmx-synthetics/tree/main/abis
- Contract addresses: https://docs.gmx.io/docs/api/contracts/overview/
- Price oracle data requires separate REST API calls to GMX oracle endpoints
- TypeScript SDK `@gmx-io/sdk` is the most complete integration reference
- Transport: **Ethereum JSON-RPC** (WebSocket or HTTPS) for contract reads/writes + GMX REST oracle API

---

## 7. Uniswap

| Field | Value |
|-------|-------|
| Official Rust SDK | NO (no official Uniswap Rust SDK) |
| Community Rust | YES — `uniswap-v3-sdk` (v6.1.0), `uniswap-sdk-core`, `uniswap-lens`, `uniswap_v3_math` |
| ABI files | Embedded in community crates / available from official TS packages |
| Official TS SDK | `@uniswap/v4-sdk` (v1.24.0) + `@uniswap/sdk-core` (npm) |
| Official Python SDK | None official (community: `uniswap-python`) |
| Effort to integrate | MEDIUM |

### Notes
- No official Rust SDK from Uniswap Labs — community crates are the only option
- **`uniswap-v3-sdk`** (v6.1.0) at https://github.com/shuhuiluo/uniswap-v3-sdk-rs — most complete community Rust SDK, mirrors the TypeScript SDK structure but in Rust:
  - `SimpleTick​DataProvider` — fetches tick data from V3 pool contracts via RPC
  - `EphemeralTickDataProvider` — fetches ticks via single `eth_call`
  - `TickMap` — direct HashMap-based tick data access
  - MSRV: Rust 1.88
- **`uniswap-sdk-core`** at https://github.com/malik672/uniswap-sdk-core-rust — core primitives
- **`uniswap-lens`** — query pool state via ephemeral lens contracts
- **`uniswap_v3_math`** — pure math (tick, price, liquidity calculations)
- Transport: **Ethereum JSON-RPC WebSocket/HTTPS** for pool state + event subscriptions via `eth_subscribe`
- Uniswap v4 is new (2024); community Rust support is V3-focused; V4 Rust support is minimal
- Crates.io: https://crates.io/crates/uniswap-v3-sdk | https://crates.io/crates/uniswap-sdk-core

---

## 8. Futu (Futu Securities)

| Field | Value |
|-------|-------|
| Official Rust SDK | NO |
| Community Rust | NO known Rust crate |
| Proto files | https://github.com/FutunnOpen/py-futu-api/tree/master/futu/common/pb (40+ .proto files) |
| Official Python SDK | https://github.com/FutunnOpen/py-futu-api (`futu-api` on PyPI) |
| Official JS SDK | Yes (JavaScript listed in official docs) |
| Effort to integrate | HIGH |

### Notes
- Futu OpenAPI uses a **custom TCP + Protobuf protocol** via a local "OpenD" gateway process — NOT a direct internet REST API
- The client connects to **OpenD** (a local daemon running on the user's machine or server) over TCP; OpenD then handles Futu server communication
- Protocol header is a **fixed 48-byte structure** (`FTAPIHeader`) followed by protobuf payload; supports optional RSA/AES encryption
- Proto files are all available in the Python SDK repo under `futu/common/pb/`: `InitConnect.proto`, `Qot_Common.proto` (market data), `Trd_Common.proto` (trading), 40+ files total
- Official SDK languages: Python, Java, C#, C++, JavaScript — **no Rust**
- No community Rust crate found on crates.io or GitHub
- Integration requires: implement custom TCP framing + prost-based protobuf codegen from `.proto` files
- Proto files can be compiled with `prost-build` in a `build.rs` script
- Key proto files to study: `InitConnect.proto`, `Qot_Sub.proto`, `Qot_GetKL.proto`, `Trd_PlaceOrder.proto`

---

## 9. Bitquery

| Field | Value |
|-------|-------|
| Official Rust SDK | NO |
| Community Rust | PARTIAL — `graphql-streaming-example-rs` (official example, not a full SDK) |
| Schema files | GraphQL introspection at `https://streaming.bitquery.io/graphql` (OAuth required) |
| Official TS SDK | None dedicated (use `graphql-ws` npm package + generated types) |
| Official Python SDK | None dedicated (community: `python-bitquery`) |
| Effort to integrate | MEDIUM |

### Notes
- Bitquery is a **GraphQL API** (not REST/gRPC) — transport is **WebSocket** (`wss://streaming.bitquery.io/graphql`) using GraphQL subscriptions for streaming, plus HTTPS for queries
- No official SDK in any language — Bitquery provides documentation and code samples, but the integration model is "bring your own GraphQL client"
- Official Rust streaming example at https://github.com/bitquery/graphql-streaming-example-rs — demonstrates WebSocket subscription for DEX trades on Solana (Pumpfun); uses `graphql-ws` protocol
- Authentication: **OAuth 2.0** required since January 2025 (Bearer token in `Authorization` header or passed as `token` field in WebSocket connection init payload)
- WebSocket protocol: `graphql-transport-ws` (server sends `pong` on connect) or `graphql-ws` (server sends `ka` keepalive)
- Schema introspection available after OAuth — use `graphql-client` crate for code generation from SDL
- Recommended Rust crates: `graphql-client` (https://github.com/graphql-rust/graphql-client) for typed query generation + `tokio-tungstenite` or `async-tungstenite` for WebSocket transport
- Reconnect required if no data or `ka` message received for ~10 seconds
- Streaming endpoint: `wss://streaming.bitquery.io/graphql`
- API docs: https://docs.bitquery.io/

---

## Summary Table

| Connector | Official Rust SDK | Community Rust | Primary Non-REST Transport | Effort |
|-----------|------------------|----------------|---------------------------|--------|
| dYdX v4 | YES (`dydx` v0.3.0) | YES (`dydx-proto`) | gRPC + WebSocket | LOW |
| Jupiter | YES (`jup-ag-sdk` v1.0.6) | YES (`jup-ag`) | Solana RPC WebSocket | LOW |
| Raydium | PARTIAL (`raydium-library`) | YES (multiple crates) | Solana RPC WebSocket | MEDIUM |
| Lighter | NO | YES (`lighter-rs`) | WebSocket + Poseidon2/Schnorr signing | HIGH |
| Paradex | NO | YES (`paradex` v0.7.2) | WebSocket + StarkNet JWT auth | HIGH |
| GMX v2 | NO | NO (use `alloy` + ABIs) | Ethereum JSON-RPC WebSocket | HIGH |
| Uniswap | NO | YES (`uniswap-v3-sdk` v6.1.0) | Ethereum JSON-RPC WebSocket | MEDIUM |
| Futu | NO | NO | Custom TCP + Protobuf (OpenD) | HIGH |
| Bitquery | NO | PARTIAL (official example) | GraphQL WebSocket subscriptions | MEDIUM |

---

## Effort Classification Notes

- **LOW**: Official Rust SDK exists on crates.io, minimal custom work needed
- **MEDIUM**: Good community crates exist OR standard protocol (GraphQL/Solana RPC) with known Rust tooling
- **HIGH**: No Rust SDK, complex custom signing/auth (StarkNet, ZK-Schnorr), or custom binary protocol (Futu TCP)

---

## Sources

- [dydx crate — crates.io](https://crates.io/crates/dydx)
- [dydx-proto crate — crates.io](https://crates.io/crates/dydx-proto)
- [dYdX Rust Quick Start — docs.dydx.xyz](https://docs.dydx.xyz/interaction/client/quick-start-rs)
- [dydxprotocol/v4-clients — GitHub](https://github.com/dydxprotocol/v4-clients)
- [jup-ag-sdk — crates.io](https://crates.io/crates/jup-ag-sdk)
- [Jupiter-DevRel/jup-rust-sdk — GitHub](https://github.com/Jupiter-DevRel/jup-rust-sdk)
- [jup-ag — crates.io](https://crates.io/crates/jup-ag)
- [raydium-io/raydium-library — GitHub](https://github.com/raydium-io/raydium-library)
- [raydium-amm-swap — crates.io](https://crates.io/crates/raydium-amm-swap)
- [raydium-sdk-V2 (TypeScript) — GitHub](https://github.com/raydium-io/raydium-sdk-V2)
- [Lighter SDK repos — apidocs.lighter.xyz](https://apidocs.lighter.xyz/docs/repos)
- [elliottech/lighter-python — GitHub](https://github.com/elliottech/lighter-python)
- [0xvasanth/lighter-rs — GitHub](https://github.com/0xvasanth/lighter-rs)
- [paradex crate — crates.io](https://crates.io/crates/paradex)
- [paradex crate docs — docs.rs](https://docs.rs/crate/paradex/latest)
- [tradeparadex/code-samples — GitHub](https://github.com/tradeparadex/code-samples)
- [Paradex API Authentication — docs.paradex.trade](https://docs.paradex.trade/trading/api-authentication)
- [Paradex Python SDK](https://tradeparadex.github.io/paradex-py/)
- [@paradex/sdk — npm](https://www.npmjs.com/package/@paradex/sdk)
- [gmx-io/gmx-synthetics — GitHub](https://github.com/gmx-io/gmx-synthetics)
- [@gmx-io/sdk — npm](https://www.npmjs.com/package/@gmx-io/sdk)
- [GMX SDK V2 Docs](https://docs.gmx.io/docs/api/sdk-v2/)
- [uniswap-v3-sdk — crates.io](https://crates.io/crates/uniswap-v3-sdk)
- [shuhuiluo/uniswap-v3-sdk-rs — GitHub](https://github.com/shuhuiluo/uniswap-v3-sdk-rs)
- [uniswap-sdk-core — crates.io](https://crates.io/crates/uniswap-sdk-core)
- [@uniswap/v4-sdk — npm](https://www.npmjs.com/package/@uniswap/v4-sdk)
- [FutunnOpen/py-futu-api — GitHub](https://github.com/FutunnOpen/py-futu-api)
- [Futu OpenAPI Protocol Docs](https://openapi.futunn.com/futu-api-doc/en/ftapi/protocol.html)
- [bitquery/graphql-streaming-example-rs — GitHub](https://github.com/bitquery/graphql-streaming-example-rs)
- [Bitquery WebSocket Docs](https://docs.bitquery.io/docs/subscriptions/websockets/)
- [graphql-client — GitHub](https://github.com/graphql-rust/graphql-client)
