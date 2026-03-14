# Wave 4: Blockchain Connectors Audit

**Date:** 2026-03-14
**Scope:** All DEX, swap, and onchain connectors in `src/crypto/dex/`, `src/crypto/swap/`, `src/onchain/`

---

## Summary Table

| Connector | Category | Chain(s) | Current Transport | SDK Used | Needs Chain SDK? | What For |
|-----------|----------|----------|-------------------|----------|------------------|----------|
| **dYdX v4** | `crypto/dex` | Cosmos SDK (dYdX Chain) | REST (Indexer) + gRPC (validator) | `tonic` + `prost` (grpc feature) | Yes — cosmrs / cosmos-sdk | Tx signing, MsgPlaceOrder, MsgCancelOrder via `BroadcastTx` gRPC |
| **GMX** | `crypto/dex` | Arbitrum + Avalanche | REST + The Graph subgraph | `alloy` (onchain-ethereum feature) | DONE (alloy v1) | ExchangeRouter multicall, position open/close, ERC-20 contract calls |
| **Jupiter** | `crypto/dex` | Solana | REST API (api.jup.ag) | None | Yes — solana-sdk or web3.js | Tx signing and broadcasting swap txs returned by `/swap` endpoint |
| **Lighter** | `crypto/dex` | zkEVM L2 (ZKLighter/Elliot) | REST + WebSocket | None | Yes — ECDSA via k256 | `/sendTx` requires signed tx payload; uses L1 Ethereum addresses |
| **Paradex** | `crypto/dex` | StarkNet (on Ethereum) | REST + WebSocket | `starknet-crypto` (starknet feature) | DONE (starknet-crypto 0.6) | JWT auth via StarkNet signature (`starknet_crypto::sign`) |
| **Raydium** | `crypto/swap` | Solana | REST API (api-v3.raydium.io + transaction-v1) | None | Yes — solana-sdk | `/transaction/swap-base-in` returns serialized tx needing Solana wallet signing + broadcast |
| **Uniswap** | `crypto/swap` | Ethereum (+ The Graph) | REST Trading API + GraphQL + JSON-RPC | `alloy` (onchain-ethereum feature) | DONE (alloy v1) | `exactInputSingle` swap tx build, ERC-20 balance queries, eth_call |
| **Bitquery** | `onchain/analytics` | Multi-chain (EVM + Bitcoin + Solana + Cosmos + 20 others) | GraphQL REST + WebSocket | None | No | Pure indexer/analytics API — no tx submission needed |
| **Whale Alert** | `onchain/analytics` | Multi-chain (Bitcoin, Ethereum, Solana, Polygon, Ripple, etc.) | REST | None | No | Read-only whale transaction monitoring — no tx submission needed |
| **Etherscan** | `onchain/ethereum` | Ethereum (+ Sepolia testnet) | REST | None | No | Block explorer API — read ETH balances, tx history, token transfers |

---

## Detailed Analysis Per Connector

### 1. dYdX v4 (`src/crypto/dex/dydx/`)

**Chain:** dYdX Chain (Cosmos SDK app-chain)
**REST transport:** `https://indexer.dydx.trade/v4` — fully public, no auth needed
**WS transport:** `wss://indexer.dydx.trade/v4/ws`
**Trading transport:** Cosmos `BroadcastTx` gRPC via validator node (`cosmos.tx.v1beta1.Service/BroadcastTx`)

**Current state:**
- REST + WebSocket fully implemented (market data, account, positions, order history — all read via Indexer API)
- gRPC `BroadcastTx` is implemented behind `grpc` feature using `tonic` + `prost`
- `place_order` / `cancel_order` return `UnsupportedOperation` — caller must build `TxRaw` externally with a Cosmos SDK lib

**Gap — what's missing for full trading:**
- No Cosmos SDK transaction builder — caller needs `cosmrs` crate (or equivalent) to:
  - Build `MsgPlaceOrder` / `MsgCancelOrder` protobuf messages
  - Set signer info (public key, account number, sequence)
  - Apply Cosmos ECDSA secp256k1 signature
  - Serialize to `TxRaw` bytes
- **Recommended:** Add `cosmrs = "0.14"` dependency (feature-gated as `cosmos`) — handles signing, tx building, and sequence management

---

### 2. GMX (`src/crypto/dex/gmx/`)

**Chain:** Arbitrum (primary) + Avalanche
**REST transport:** `https://arbitrum-api.gmxinfra.io` + multiple fallbacks
**Subgraph transport:** The Graph (GraphQL) for historical analytics
**On-chain transport:** `alloy` HTTP provider connecting to Arbitrum/Avalanche RPC

**Current state:**
- REST fully implemented (tickers, candles, market info, GLV data, stats)
- On-chain module (`onchain.rs`) fully implemented with `alloy` v1:
  - `GmxOnchain::arbitrum()` / `GmxOnchain::avalanche()` connect to public RPCs
  - `create_position_onchain()` builds signed multicall tx: `sendWnt + sendTokens + createOrder`
  - `close_position_onchain()` builds `sendWnt + createOrder` multicall
  - `get_block_number()` and `get_native_balance()` work
  - Manual ABI encoding for `multicall(bytes[])`, `sendWnt`, `sendTokens`, `createOrder`
- Feature-gated: `#[cfg(feature = "onchain-ethereum")]`

**Gap:**
- Tx signing is NOT done inside the connector — `TransactionRequest` is returned unsigned, caller must sign with alloy signer
- No ERC-20 approval check (caller must ensure `approve(OrderVault, amount)` before calling)
- Suggested: add `alloy::signers::LocalWallet` integration or accept `alloy::signers::Signer` in `GmxOnchain::new()`

---

### 3. Jupiter (`src/crypto/dex/jupiter/`)

**Chain:** Solana
**REST transport:** `https://api.jup.ag/swap/v1`, `/price/v3`, `/tokens/v2`, `/ultra/v1`

**Current state:**
- REST fully implemented: quotes, swap instructions, price lookups, token search, Ultra Swap API
- Uses Solana mint addresses (Base58) — `MintRegistry` maps symbols to known mints
- NO Solana SDK dependency — connector only calls Jupiter's REST API

**Gap — what's missing for actual swapping:**
- Jupiter `/swap` endpoint returns a base64-encoded, partially serialized Solana transaction
- To complete a swap, caller must:
  1. Deserialize the transaction (using `solana-sdk` or `solana-client`)
  2. Sign it with a Solana Keypair
  3. Broadcast via Solana RPC (`sendTransaction`)
- **Recommended:** Add `solana-sdk = "1.18"` + `solana-client = "1.18"` (feature-gated as `solana`)
- Alternative: use raw Base58/Base64 JSON-RPC calls to Solana RPC without the full SDK (lighter approach, already patterns exist in Raydium WS)

---

### 4. Lighter (`src/crypto/dex/lighter/`)

**Chain:** ZKLighter — custom zkEVM Layer 2 (Elliot protocol), uses L1 Ethereum addresses
**REST transport:** `https://mainnet.zklighter.elliot.ai`
**WS transport:** `wss://mainnet.zklighter.elliot.ai/stream`

**Current state:**
- Market data (Phase 1) fully implemented
- Account + trading (Phases 2-3) are stubs returning `UnsupportedOperation`
- Auth module (`auth.rs`) exists but trading is not yet wired

**Gap — what's missing for trading:**
- `/sendTx` requires a signed transaction with an ECDSA signature (secp256k1, Ethereum-style)
- Lighter uses account indices (not Ethereum addresses directly for tx submission)
- Need to sign transaction payload with private key using `k256` (already in Cargo.toml as optional `k256-signing` feature)
- **Recommended:** Enable `k256-signing` feature and wire `LighterAuth` to use `k256::ecdsa::SigningKey` for `/sendTx` payloads

---

### 5. Paradex (`src/crypto/dex/paradex/`)

**Chain:** StarkNet (Ethereum L2, uses STARK cryptography)
**REST transport:** `https://api.prod.paradex.trade/v1`
**WS transport:** `wss://ws.api.prod.paradex.trade/v1`

**Current state:**
- REST + WebSocket fully implemented: market data, trading (place/cancel/modify orders), account, positions
- Auth uses JWT tokens obtained via POST `/v1/auth` with a StarkNet signature
- `starknet` feature enables `starknet-crypto` crate — `starknet_crypto::sign()` used to sign auth timestamps
- Full trading loop: JWT refresh → sign order → REST

**Gap:**
- When `starknet` feature is OFF, auth falls back to pre-obtained JWT tokens passed as `credentials.api_key` — works but no auto-refresh
- No on-chain deposit/withdrawal (requires StarkNet L1↔L2 bridge interaction)
- **Status: COMPLETE for REST trading** — starknet-crypto is already integrated

---

### 6. Raydium (`src/crypto/swap/raydium/`)

**Chain:** Solana
**REST transport:** `https://api-v3.raydium.io` + `https://transaction-v1.raydium.io`

**Current state:**
- REST fully implemented: pools, mints, farms, swap quotes, liquidity data
- Trade API: `/compute/swap-base-in` for quotes, `/transaction/swap-base-in` POST for serialized tx
- NO Solana SDK dependency

**Gap — what's missing for actual swapping:**
- Raydium `/transaction/swap-base-in` returns a serialized, unsigned Solana transaction
- To swap, caller must: deserialize tx → sign with Solana Keypair → broadcast via `sendTransaction`
- Same gap as Jupiter — needs `solana-sdk` (feature-gated)
- WebSocket (`websocket.rs`) uses raw `tokio-tungstenite` to Solana RPC (no SDK dep needed for WS data)
- `bs58` already in Cargo.toml for pubkey parsing

---

### 7. Uniswap (`src/crypto/swap/uniswap/`)

**Chain:** Ethereum Mainnet (chain ID 1) + Sepolia testnet
**REST transport:** `https://trade-api.gateway.uniswap.org/v1` (Trading API)
**GraphQL:** The Graph subgraph for pool/swap analytics
**RPC transport:** `https://ethereum-rpc.publicnode.com` (free public RPC)

**Current state:**
- REST Trading API implemented: quote, swap, approval check, order status
- GraphQL queries implemented for pools, swaps, tokens, positions
- On-chain module (`onchain.rs`) fully implemented with `alloy` v1:
  - `UniswapOnchain::mainnet()` / `UniswapOnchain::testnet()`
  - `build_swap_tx()` — builds unsigned `exactInputSingle` transaction
  - `get_token_balance_onchain()` — ERC-20 `balanceOf` via `eth_call`
  - `get_eth_balance()` — native ETH balance
- Feature-gated: `#[cfg(feature = "onchain-ethereum")]`

**Gap:**
- Same as GMX: returns unsigned `TransactionRequest`, caller must sign externally
- No ERC-20 approval (`approve`) tx builder — caller must ensure token approval

---

### 8. Bitquery (`src/onchain/analytics/bitquery/`)

**Chains:** 20+ chains — all major EVM (Ethereum, BSC, Polygon, Arbitrum, Base, Optimism, Avalanche, Fantom...) + Solana, Bitcoin, Litecoin, Cardano, Ripple, Stellar, Algorand, Cosmos, Tron
**Transport:** GraphQL REST + WebSocket (`streaming.bitquery.io/graphql`)

**Current state:**
- GraphQL query builders implemented for: DEX trades, token transfers, balance updates, blocks, transactions, smart contract events
- WebSocket subscriptions for real-time blocks and DEX trades
- Multi-chain query builder with `BitqueryNetwork` enum covering 20 chains
- Fully read-only analytics — no transaction submission

**Gap:** None for its use case (analytics/monitoring). No chain SDK needed.

---

### 9. Whale Alert (`src/onchain/analytics/whale_alert/`)

**Chains:** Bitcoin, Ethereum, Algorand, Bitcoin Cash, Dogecoin, Litecoin, Polygon, Solana, Ripple, Cardano, Tron
**Transport:** REST (`https://leviathan.whale-alert.io`) + optional WebSocket

**Current state:**
- Enterprise API v2 + legacy v1 endpoints implemented
- Transaction lookup by hash, streaming by block range, address attribution
- Fully read-only whale monitoring — no transaction submission

**Gap:** None for its use case. No chain SDK needed.

---

### 10. Etherscan (`src/onchain/ethereum/etherscan/`)

**Chain:** Ethereum Mainnet + Sepolia testnet
**Transport:** REST (`https://api.etherscan.io/api`)

**Current state:**
- Block explorer API: ETH balances, ERC-20 transfers, transaction history, token supply, gas oracle, contract ABI
- JSON-RPC proxy: `eth_blockNumber`, `eth_getBlockByNumber`
- Fully read-only chain data explorer

**Gap:** None for its use case. No chain SDK needed.

---

## Cargo.toml Features — Current State

```toml
[features]
default = ["onchain-ethereum"]
onchain-ethereum = ["dep:alloy"]          # GMX onchain + Uniswap onchain
grpc = ["dep:tonic", "dep:prost"]         # dYdX BroadcastTx
k256-signing = ["dep:k256"]              # Available but NOT wired into Lighter yet
starknet = ["dep:starknet-crypto"]        # Paradex JWT signing — DONE

[dependencies]
alloy = { version = "1", features = ["provider-ws", "rpc-types"], optional = true }
tonic = { version = "0.12", ... optional }
prost = { version = "0.13", optional }
k256 = { version = "0.13", features = ["ecdsa-core", "ecdsa"], optional }
starknet-crypto = { version = "0.6", optional }
bs58 = "0.5"          # For Raydium pubkey parsing
```

---

## Missing Dependencies (Not Yet Added)

| Need | For | Recommended Crate | Feature Gate |
|------|-----|-------------------|--------------|
| Cosmos SDK tx building | dYdX order placement | `cosmrs = "0.14"` | `cosmos` |
| Solana tx signing + broadcast | Jupiter, Raydium swap execution | `solana-sdk = "1.18"` + `solana-client = "1.18"` | `solana` |
| Ethereum tx signing (signer integration) | GMX + Uniswap (already build tx, need to sign) | `alloy` signers already in alloy v1 — just wire `LocalWallet` | `onchain-ethereum` |

---

## Priority Recommendations

1. **dYdX trading (HIGH)** — Add `cosmrs` behind `cosmos` feature. Wire `DydxConnector::place_order()` to build + sign + broadcast Cosmos SDK transactions instead of returning `UnsupportedOperation`.

2. **Lighter trading (MEDIUM)** — `k256-signing` feature is already in Cargo.toml. Wire `LighterAuth` to use `k256::ecdsa::SigningKey` for `/sendTx` payload signing. No new dep needed.

3. **Jupiter / Raydium swap execution (MEDIUM)** — Add `solana-sdk` + `solana-client` behind `solana` feature. Add `JupiterOnchain` / `RaydiumOnchain` modules similar to `gmx/onchain.rs` and `uniswap/onchain.rs` — they would deserialize the returned transaction, sign it, and broadcast via Solana RPC.

4. **GMX / Uniswap signer integration (LOW)** — Both already build `TransactionRequest`. Add a thin `sign_and_send()` wrapper that accepts an `alloy::signers::LocalWallet` (already available inside alloy v1) so the full round-trip works without the caller managing signing.

5. **Etherscan / Bitquery / Whale Alert (COMPLETE)** — No changes needed. These are pure analytics/read connectors with no on-chain write requirements.
