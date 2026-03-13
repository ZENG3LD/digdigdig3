# Wave 4: Swap / DEX / Onchain Connectors — Deep Audit Report

Generated: 2026-03-13
Source tree: `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\`

---

## Scope

| Path | Connectors |
|------|-----------|
| `crypto/dex/` | dYdX, GMX, Jupiter, Lighter, Paradex |
| `crypto/swap/` | Raydium, Uniswap |
| `onchain/analytics/` | Bitquery, Whale Alert |
| `onchain/ethereum/` | Etherscan |

Total: **9 connectors**

---

## Per-Connector Detail Tables

### 1. dYdX v4

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\dydx\` |
| **Transport** | REST HTTPS (Indexer API) + WebSocket JSON (Indexer WS) |
| **Base URLs** | `https://indexer.dydx.trade/v4` (mainnet), `https://indexer.v4testnet.dydx.exchange/v4` (testnet) |
| **WS URL** | `wss://indexer.dydx.trade/v4/ws` |
| **Auth** | None for Indexer (all public). Future writes: Cosmos SDK gRPC + wallet mnemonic signing |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData`, `Trading` (read only), `Account`, `Positions`, `WebSocketConnector` |
| **Real vs Stub** | MarketData: **REAL** (price, ticker, orderbook, klines, exchange_info). Account.get_balance: **REAL**. Positions.get_positions/get_funding_rate: **REAL**. Trading.get_order/get_open_orders/get_order_history: **REAL** (read-only Indexer). Trading.place_order/cancel_order: **STUB** → `UnsupportedOperation`. Positions.modify_position: **STUB** → `UnsupportedOperation`. |
| **WS Channels** | `v4_orderbook`, `v4_trades`, `v4_markets`, `v4_candles` |
| **External Crates Needed** | None currently. For write ops: `cosmos-sdk-rs` or `dydx-v4-client` + gRPC via `tonic` |
| **Available API Surface** | All Indexer REST endpoints: perpetual markets, orderbooks, trades, candles, historical funding, sparklines, addresses/subaccounts, positions, orders, fills, trading rewards |
| **What to Wrap** | Read-only market + account data is fully wrapped. Order placement/cancellation needs Cosmos gRPC (Node API) |
| **Impossible Server-Side** | Order placement/cancellation — requires Cosmos wallet + private key signing of MsgPlaceOrder/MsgCancelOrder protobuf transactions |

---

### 2. GMX v2

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\gmx\` |
| **Transport** | REST HTTPS only (no native WS; connector uses polling) |
| **Base URLs** | `https://arbitrum-api.gmxinfra.io` (Arbitrum), `https://avalanche-api.gmxinfra.io` (Avalanche), with fallback1/fallback2 variants |
| **Auth** | None — all REST endpoints are public, no API key required |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData`, `Trading`, `Account`, `Positions` |
| **Real vs Stub** | MarketData.get_price: **REAL** (via tickers). get_ticker: **REAL**. get_klines: **REAL** (`/prices/candles`). get_exchange_info: **REAL** (`/markets`). get_orderbook: **STUB** → `NotSupported` (oracle pricing, no orderbook). Trading: all **STUB** → `UnsupportedOperation` (needs ethers-rs/alloy + smart contract). Account: all **STUB**. Positions: all **STUB**. |
| **WS** | Not implemented — no native public WS on gmxinfra.io |
| **External Crates Needed** | For trading: `ethers` or `alloy` (EVM signer), ERC-20 approve, multicall. For positions/account: The Graph subgraph (GraphQL) or contract event logs |
| **Available API Surface** | `/ping`, `/prices/tickers`, `/prices/candles`, `/signed_prices/latest`, `/tokens`, `/markets`, `/markets/info`, `/apy`, `/glvs`, `/glvs/info`, `/performance/annualized` |
| **What to Wrap** | All market data is wrapped. Order history / account data needs The Graph subgraph |
| **Impossible Server-Side** | Order placement/cancellation — requires EIP-712 signing, ERC-20 approvals, keeper network async execution. No server-side REST path exists |

---

### 3. Jupiter (Solana DEX Aggregator)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\jupiter\` |
| **Transport** | REST HTTPS only |
| **Base URLs** | `https://api.jup.ag/swap/v1`, `https://api.jup.ag/price/v3`, `https://api.jup.ag/tokens/v2` |
| **Auth** | **API key required for ALL endpoints** since October 2025 (header `x-api-key`) |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData`, `Trading`, `Account` |
| **Real vs Stub** | MarketData.get_price: **REAL** (Price API v3). get_ticker: **REAL** (Price API). ping: **REAL**. get_orderbook: **STUB** → `UnsupportedOperation` (aggregator, no orderbook). get_klines: **STUB** → `UnsupportedOperation` (no historical API). Trading: all **STUB** → `UnsupportedOperation`. Account: all **STUB** → `UnsupportedOperation`. |
| **WS** | Not implemented — no native WebSocket |
| **External Crates Needed** | For swap execution: `solana-sdk` + wallet keypair. Quote API returns unsigned tx bytes that must be signed and submitted via Solana RPC (`sendTransaction`) |
| **Available API Surface** | `/quote` (routing), `/swap` (get unsigned tx calldata), `/swap-instructions`, `/price/v3` (token prices), `/tokens/v2/search`, `/tokens/v2/tag`, `/tokens/v2/recent` |
| **What to Wrap** | Price data is wrapped. Quote/routing (swap) returns unsigned tx — server-side partial execution possible |
| **Impossible Server-Side** | Full swap execution — requires Solana wallet keypair for transaction signing |
| **Notes** | Identifies tokens by Solana mint addresses. Hardcoded MintRegistry for SOL, USDC, USDT, JUP, RAY, ORCA, BONK, WIF |

---

### 4. Lighter (zkEVM CLOB DEX)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\lighter\` |
| **Transport** | REST HTTPS + WebSocket JSON |
| **Base URLs** | `https://mainnet.zklighter.elliot.ai` (mainnet), `https://testnet.zklighter.elliot.ai` (testnet) |
| **WS URL** | `wss://mainnet.zklighter.elliot.ai/stream` |
| **Auth** | Public market data: no auth. Account/trading: ECDSA secp256k1 key signing (L2 transaction format). Account lookup by `account_index` or `l1_address` |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData`, `Trading`, `Account`, `Positions`, `WebSocketConnector` |
| **Real vs Stub** | MarketData: **REAL** (price, ticker, orderbook, klines, exchange_info). Account.get_balance: **REAL** (`/api/v1/account`). Account.get_account_info: **REAL**. Account.get_fees: **REAL** (`/api/v1/orderBooks`). Positions.get_positions: **REAL** (embedded in account). Positions.get_funding_rate: **REAL** (`/api/v1/fundings`). Trading.get_order_history: **REAL** (`/api/v1/accountInactiveOrders`). Trading.get_open_orders: **STUB** → `UnsupportedOperation` (no REST endpoint, WS only). Trading.place_order: **STUB** → `UnsupportedOperation` (needs ECDSA signing). Trading.cancel_order: **STUB** → `UnsupportedOperation`. Positions.modify_position: **STUB** → `UnsupportedOperation`. |
| **External Crates Needed** | For trading: `k256` or `secp256k1` crate for ECDSA signing of L2 transaction format |
| **Available API Surface** | Full CLOB: orderBooks, orderBookDetails, orderBookOrders, recentTrades, trades, candles, fundings, account, accountInactiveOrders, accountTxs, pnl, sendTx, sendTxBatch, nextNonce, deposit/withdraw history, blockchain data |
| **What to Wrap** | All read-only data wrapped. Trade execution needs ECDSA signing (tx_type=14 L2CreateOrder, tx_type=15 L2CancelOrder) via `POST /api/v1/sendTx` |
| **Impossible Server-Side** | Nothing impossible — Lighter IS a centralized service (zkEVM L2). Full trading possible once ECDSA signing is implemented |

---

### 5. Paradex (StarkNet Perpetuals DEX)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\paradex\` |
| **Transport** | REST HTTPS + WebSocket JSON |
| **Base URLs** | `https://api.prod.paradex.trade/v1` (mainnet), `https://api.testnet.paradex.trade/v1` (testnet) |
| **WS URL** | `wss://ws.api.prod.paradex.trade/v1` |
| **Auth** | JWT Bearer token. Token obtained via `POST /v1/auth` with StarkNet signature (ECDSA on StarkNet curve). JWT expires every 5 minutes, should refresh every 3 min. Current impl accepts pre-obtained JWT via `Credentials.api_key` |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData`, `Trading`, `Account`, `Positions`, `CancelAll`, `AmendOrder`, `BatchOrders`, `WebSocketConnector` |
| **Real vs Stub** | MarketData: **REAL** (price, ticker, orderbook, klines, exchange_info, ping). Account: **REAL** (balances, account_info, fees). Positions: **REAL** (positions, funding_rate, modify_position). Trading: **REAL** for all CRUD (place_order, cancel_order, get_order, get_open_orders, get_order_history). CancelAll: **REAL**. AmendOrder (ModifyOrder): **REAL** (PUT /orders/{id}). BatchOrders (CreateOrderBatch): **REAL**. |
| **WS** | Implemented — `WebSocketConnector` trait |
| **External Crates Needed** | For JWT generation from scratch: `starknet-rs` (StarkNet ECDSA signing). Current impl assumes pre-obtained JWT, so works without any extra crates |
| **Available API Surface** | Full perpetuals DEX: auth, system config/state/time, markets, market summary, orderbook, trades, klines, account, balances, positions, subaccounts, orders (CRUD + batch + cancel all), algo orders (TWAP), fills, funding payments, transactions, transfers, liquidations, tradebusts |
| **What to Wrap** | Everything is wrapped. Only gap: JWT auto-refresh needs `starknet-rs` |
| **Impossible Server-Side** | Nothing impossible — Paradex REST API is fully featured. Auto-generation of JWT requires StarkNet private key |
| **Notes** | Symbol format: `BTC-USD-PERP`. Most fully featured DEX connector in codebase — implements CancelAll, AmendOrder, BatchOrders |

---

### 6. Raydium (Solana AMM)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\swap\raydium\` |
| **Transport** | REST HTTPS only |
| **Base URLs** | `https://api-v3.raydium.io` (data), `https://transaction-v1.raydium.io` (trade/swap tx) |
| **Auth** | None — all public |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData`, `Trading`, `Account` |
| **Real vs Stub** | MarketData.get_price: **REAL** (`/mint/price`). get_ticker: **REAL** (`/pools/info/mint`). ping: **REAL** (`/main/version`). get_orderbook: **STUB** → `UnsupportedOperation` (pure AMM). get_klines: **STUB** → `NotSupported` (no kline API). Trading: all **STUB** → `UnsupportedOperation`. Account: all **STUB** → `UnsupportedOperation`. |
| **WS** | Not implemented — Raydium has no public WS (gRPC for real-time) |
| **External Crates Needed** | For swap execution: `solana-sdk` + wallet keypair. For real-time price: Solana gRPC (Yellowstone/Jito) |
| **Available API Surface** | `/main/version`, `/main/rpcs`, `/mint/list`, `/mint/ids`, `/mint/price`, `/pools/info/list`, `/pools/info/ids`, `/pools/info/mint`, `/pools/position/list`, `/farms/info/*`, `/compute/swap-base-in`, `/compute/swap-base-out`, `/transaction/swap-base-in`, `/transaction/swap-base-out` |
| **What to Wrap** | Pool data and prices are wrapped. Swap tx serialization (`/transaction/swap-*`) returns unsigned Solana tx — partial server-side possible |
| **Impossible Server-Side** | Full swap — requires Solana wallet keypair to sign and submit. Real-time price feed requires gRPC, not REST |
| **Notes** | Token identification via Solana mint addresses. Well-known mints registry included (SOL, USDC, USDT, RAY, SRM) |

---

### 7. Uniswap v3 (Ethereum AMM)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\swap\uniswap\` |
| **Transport** | Three transports in one connector: (1) REST HTTPS (Trading API), (2) GraphQL over HTTPS (The Graph subgraph), (3) JSON-RPC over HTTPS (Ethereum RPC) |
| **Base URLs** | Trading API: `https://trade-api.gateway.uniswap.org/v1`. Subgraph: `https://gateway.thegraph.com/api/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV`. ETH RPC: `https://ethereum-rpc.publicnode.com` (free public) |
| **Auth** | Trading API requires API key (header). Subgraph: optional API key. ETH RPC: public (no key needed) |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData`, `Trading`, `Account` |
| **Real vs Stub** | MarketData.get_price: **REAL** via ETH JSON-RPC `eth_call` → pool `slot0()` OR subgraph fallback. get_ticker: **REAL** (subgraph). get_orderbook: **REAL** (simulated from pool liquidity via subgraph). get_klines: **REAL** (from swap events via subgraph). ping: **REAL** (`eth_blockNumber`). get_exchange_info: **REAL** (top pools from subgraph). Trading: all **STUB** → `UnsupportedOperation`. Account: all **STUB** → `UnsupportedOperation`. |
| **WS** | Not implemented. ETH WS URL is stored (`wss://ethereum-rpc.publicnode.com`) but no WS connector |
| **External Crates Needed** | For swap execution: `ethers` or `alloy` + wallet private key + EIP-712 signing (Permit2). For WS events: `tokio-tungstenite` + Ethereum event subscription |
| **Available API Surface** | Trading API: `/quote`, `/swap`, `/check_approval`, `/orders`, `/swaps`, `/swappable_tokens`. Subgraph: pools, swaps, tokens, positions, factory. ETH RPC: `eth_call`, `eth_getBalance`, `eth_getTransactionReceipt`, `eth_blockNumber` |
| **What to Wrap** | All market data wrapped via 3-transport approach. Smart price fallback: RPC first (no API key), subgraph fallback |
| **Impossible Server-Side** | Swap execution — requires wallet private key + Permit2 EIP-712 signature + broadcast to Ethereum mempool |
| **Notes** | Known pool registry for WETH/USDC, WETH/USDT, WBTC/WETH. FeeTier enum: 0.01% / 0.05% / 0.30% / 1.00%. Hardcoded V3 contract addresses (Factory, Router, Quoter, PositionManager) |

---

### 8. Bitquery (Blockchain Analytics — GraphQL)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\onchain\analytics\bitquery\` |
| **Transport** | GraphQL over HTTPS + GraphQL Subscriptions over WebSocket |
| **Base URLs** | `https://streaming.bitquery.io/graphql` (HTTP+WS) |
| **Auth** | OAuth2 Bearer token (`Authorization: Bearer ory_at_...`). Free tier: 10 req/min |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData` (stubs), `Trading` (stubs), `Account` (stubs), `Positions` (stubs) |
| **Real vs Stub** | All standard traits: **STUB** → `UnsupportedOperation`. ping: **REAL** (minimal `{ __typename }` query). Custom methods: **REAL** — `get_dex_trades()`, `get_realtime_dex_trades()`, `get_token_transfers()`, `get_balance_updates()`, `get_blocks()`, `get_transactions()`, `get_smart_contract_events()` |
| **WS** | WS URL stored, GraphQL subscription queries provided (`_build_blocks_subscription`, `_build_dex_trades_subscription`) but not wired to `WebSocketConnector` trait |
| **External Crates Needed** | None for HTTP GraphQL. For streaming subscriptions: `async-graphql-client` or `graphql-ws` WS protocol |
| **Available API Surface** | EVM cubes: Blocks, Transactions, MempoolTransactions, Transfers, BalanceUpdates, DEXTrades, NFTTrades, Events, Calls. Solana: Instructions. Bitcoin: Inputs/Outputs. Networks: ETH, BSC, Polygon, Arbitrum, Base, Optimism, Avalanche, Solana, Bitcoin, etc. |
| **What to Wrap** | All read-only analytics data. WS subscriptions need `WebSocketConnector` impl |
| **Notes** | Data provider only. Not a DEX/exchange. Used for cross-chain analytics, whale tracking, MEV research |

---

### 9. Whale Alert (Blockchain Transaction Monitoring)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\onchain\analytics\whale_alert\` |
| **Transport** | REST HTTPS (Enterprise API v2 + Developer API v1 deprecated) + WebSocket (alerts) |
| **Base URLs** | Enterprise v2: `https://leviathan.whale-alert.io`. v1 (deprecated): `https://api.whale-alert.io/v1`. WS: `wss://leviathan.whale-alert.io/ws` |
| **Auth** | API key in query params (`api_key=...`) for v1. Enterprise v2 uses auth mechanism via `WhaleAlertAuth` struct |
| **Traits Implemented** | `ExchangeIdentity`, `MarketData` (stubs), `Trading` (stubs), `Account` (stubs), `Positions` (stubs) |
| **Real vs Stub** | All standard market/trading traits: **STUB** → `UnsupportedOperation`. ping: **REAL** (status endpoint). Custom methods: **REAL** — `get_status()`, `get_blockchain_status()`, `get_transaction()`, `get_transactions()` (stream by block height), `get_block()`, `get_address_transactions()`, `get_address_attributions()` |
| **WS** | WS URL stored but no `WebSocketConnector` impl. Native WS endpoint for real-time whale alerts exists |
| **External Crates Needed** | None currently. For WS: `tokio-tungstenite` |
| **Available API Surface** | Status, blockchain status, single transaction by hash, transaction stream by block height, complete block data, address transaction history (30-day), address owner attribution |
| **Supported Chains** | Bitcoin, Ethereum, Algorand, BCH, Dogecoin, Litecoin, Polygon, Solana, Ripple, Cardano, Tron |
| **What to Wrap** | All REST analytics wrapped. WS real-time alerts not yet wired |

---

### 10. Etherscan (Ethereum Block Explorer API)

| Field | Value |
|-------|-------|
| **Path** | `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\onchain\ethereum\etherscan\` |
| **Transport** | REST HTTPS only (no WS) |
| **Base URLs** | `https://api.etherscan.io/api` (mainnet), `https://api-sepolia.etherscan.io/api` (Sepolia testnet) |
| **Auth** | API key appended to query params (`&apikey=...`). Free tier available |
| **Traits Implemented** | `ExchangeIdentity` (via module), but no standard trading traits — standalone connector only |
| **Real vs Stub** | Custom methods only: **REAL** — `get_balance()`, `get_multi_balance()`, `get_transactions()`, `get_token_transfers()`, `get_internal_transactions()`, `get_eth_price()`, `get_eth_supply()`, `get_chain_size()`, `get_token_supply()`, `get_gas_oracle()`, `get_latest_block_number()`, `get_block_reward()`, `get_contract_abi()` |
| **External Crates Needed** | None |
| **Available API Surface** | Account: balance, multi-balance, tx list, internal txs, ERC20 transfers. Stats: ETH price, ETH supply, chain size, token supply. Gas: gas oracle. Block: block reward. Contract: ABI. Proxy: eth_blockNumber |
| **What to Wrap** | Everything is already wrapped via custom methods |
| **Notes** | Does not implement standard digdigdig3 traits (MarketData/Trading). Standalone blockchain explorer client. No WS support in Etherscan API |

---

## Summary: Transport Requirements Grouped by Protocol

### Group A: Plain REST HTTPS (reqwest is sufficient)

| Connector | Notes |
|-----------|-------|
| dYdX (Indexer) | All read operations via simple GET. No special headers |
| GMX | Pure GET. Multiple fallback URLs. No auth |
| Jupiter | GET + POST. API key in header. Solana mint addresses |
| Lighter | GET (public) + POST `sendTx` (signed tx body) |
| Raydium | GET (data) + POST (unsigned tx serialization) |
| Etherscan | GET with `apikey` query param. Module+action params |

**Required crates (already in use):** `reqwest`, `serde_json`, `tokio`

---

### Group B: REST HTTPS + WebSocket JSON

| Connector | WS Protocol | Notes |
|-----------|-------------|-------|
| dYdX | Custom JSON subscribe/unsubscribe messages over tokio-tungstenite | Implemented + working |
| Lighter | Custom JSON subscription protocol | Implemented (websocket.rs exists) |
| Paradex | Custom JSON channel subscription | Implemented (websocket.rs exists) |
| Whale Alert | WS endpoint exists (`wss://leviathan.whale-alert.io/ws`) | URL stored, NOT implemented in WebSocketConnector |
| Bitquery | GraphQL WS (subscriptions protocol) | URL stored, subscription queries exist, NOT wired |

**Required crates:** `tokio-tungstenite`, `futures-util`, `tokio-stream`

---

### Group C: GraphQL over HTTPS (POST with JSON body)

| Connector | Endpoint | Notes |
|-----------|----------|-------|
| Uniswap | The Graph subgraph (`gateway.thegraph.com`) | Raw string queries, no graphql-client crate |
| Bitquery | `streaming.bitquery.io/graphql` | Raw string queries, OAuth bearer token |

**Current approach:** Hand-written GraphQL query strings as Rust string literals, serialized as `{"query": "..."}` POST bodies. **No `graphql-client` or `cynic` crate needed** — raw `serde_json` suffices.

---

### Group D: JSON-RPC over HTTPS (Ethereum eth_ methods)

| Connector | RPC Endpoint | Notes |
|-----------|-------------|-------|
| Uniswap | `https://ethereum-rpc.publicnode.com` (free, no key) | `eth_call`, `eth_blockNumber`, `eth_getBalance`, `eth_getTransactionReceipt` |

**Current approach:** Manual `{"jsonrpc":"2.0","method":"eth_call","params":[...]}` POST body construction. **No `ethers` or `alloy` needed** for these read-only calls — raw reqwest + serde_json works.

---

### Group E: Blockchain Wallet Signing Required (Not Yet Implemented)

These operations are **explicitly stubbed** with `UnsupportedOperation` and require on-chain signing crates:

| Connector | Signing Type | Required Crate(s) | Operation |
|-----------|-------------|-------------------|-----------|
| dYdX | Cosmos SDK / protobuf | `cosmos-sdk-rs`, `tonic` (gRPC), `prost` | MsgPlaceOrder, MsgCancelOrder via Node API gRPC |
| GMX | EVM EIP-712 | `ethers` or `alloy` | ExchangeRouter contract calls + ERC-20 approvals |
| Jupiter | Solana transaction signing | `solana-sdk` | Sign + submit swap transaction to Solana RPC |
| Lighter | ECDSA secp256k1 L2 tx | `k256` or `secp256k1` | L2CreateOrder (tx_type=14), L2CancelOrder (tx_type=15) |
| Paradex | StarkNet ECDSA | `starknet-rs` | JWT generation via `POST /v1/auth` with StarkNet sig |
| Raydium | Solana transaction signing | `solana-sdk` | Sign + submit swap transaction |
| Uniswap | EVM EIP-712 + Permit2 | `ethers` or `alloy` | Swap calldata + sign + broadcast |

---

### Group F: GraphQL Subscriptions (WebSocket — Not Yet Implemented)

| Connector | WS URL | Protocol | Status |
|-----------|--------|----------|--------|
| Bitquery | `wss://streaming.bitquery.io/graphql` | `graphql-ws` subprotocol | URL stored, queries written, NOT wired |
| Whale Alert | `wss://leviathan.whale-alert.io/ws` | Custom JSON (unknown) | URL stored, NOT implemented |

---

## Priority Implementation Recommendations

### Tier 1 — Easy wins (no new crates, REST only)

1. **Lighter trading** (`k256` for ECDSA) — REST API is fully featured CLOB. One crate needed.
2. **Bitquery WS subscriptions** — Real-time DEX trades/blocks via existing WS infrastructure.
3. **Whale Alert WS** — Real-time whale alert stream.
4. **Paradex JWT auto-refresh** (`starknet-rs`) — Everything else is already working.

### Tier 2 — Medium effort (add ethers/alloy or solana-sdk)

5. **Uniswap swap execution** (`alloy`) — Quote + sign + broadcast pipeline.
6. **Raydium swap execution** (`solana-sdk`) — Quote from API + sign + sendTransaction.
7. **Jupiter swap execution** (`solana-sdk`) — Same pattern as Raydium.

### Tier 3 — Complex (new transport layer)

8. **dYdX write operations** — Requires Cosmos gRPC (`tonic`) + protobuf message signing. Different transport layer entirely.
9. **GMX trading** — Smart contract interaction (ethers + keeper network async execution model is very different from REST trading).

---

## Crate Dependency Summary

| New Crate | Version Guidance | Used By |
|-----------|-----------------|---------|
| `k256` | `0.13` | Lighter (secp256k1 ECDSA for L2 tx signing) |
| `starknet-rs` (or `starknet-crypto`) | `0.7` | Paradex JWT generation |
| `solana-sdk` | `1.18` | Raydium, Jupiter swap execution |
| `alloy` (preferred over ethers) | `0.1+` | Uniswap, GMX swap execution |
| `tonic` + `prost` | `0.12`, `0.12` | dYdX gRPC Node API |
| `graphql-client` (optional) | `0.14` | Bitquery (optional — current raw-string approach works) |

**Currently sufficient (no new crates needed):**
- `reqwest` + `serde_json` + `tokio` + `tokio-tungstenite` cover all read-only operations across all 9 connectors.
