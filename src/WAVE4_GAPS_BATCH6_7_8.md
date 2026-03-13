# Wave 4 — Endpoint Gap Analysis: Batches 6, 7, 8

**Batch 6:** Crypto Swap/AMM (Raydium, Uniswap)
**Batch 7:** Onchain Analytics (Bitquery, Whale Alert, Etherscan)
**Batch 8:** Prediction Markets (Polymarket, PredictIt)

---

## Batch 6 — Crypto Swap / AMM

---

### 1. Raydium

**File:** `crypto/swap/raydium/endpoints.rs`
**Docs:** https://docs.raydium.io/raydium/protocol/developers/api
**Swagger:** https://api-v3.raydium.io/docs/
**Transport needed:** REST (Solana RPC for on-chain execution)

#### Current Implementation

Enum variants: `Version`, `Rpcs`, `AutoFee`, `MintList`, `MintIds`, `MintPrice`, `PoolList`, `PoolIds`, `PoolByMint`, `PoolPositions`, `FarmList`, `FarmIds`, `IdoPoolKeys`, `SwapQuoteBaseIn`, `SwapQuoteBaseOut`, `SwapTransactionBaseIn`, `SwapTransactionBaseOut`

#### Gap Analysis

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Platform | `GET /main/version` | YES | |
| Platform | `GET /main/rpcs` | YES | |
| Platform | `GET /main/auto-fee` | YES | |
| Platform | `GET /main/chain-time` | NO | Solana network time |
| Platform | `GET /main/info` | NO | Platform stats, TVL, 24h volume |
| Mint | `GET /mint/list` | YES | |
| Mint | `GET /mint/ids` | YES | |
| Mint | `GET /mint/price` | YES | |
| Pools — AMM V4 | `GET /pools/info/list` | YES | |
| Pools — AMM V4 | `GET /pools/info/ids` | YES | |
| Pools — AMM V4 | `GET /pools/info/mint` | YES | |
| Pools — AMM V4 | `GET /pools/position/list` | YES | LP positions by owner |
| Pools — CLMM | `GET /pools/info/list?poolType=concentrated` | PARTIAL | CLMM type filter not a separate variant |
| Pools — CPMM | `GET /pools/info/list?poolType=standard` | PARTIAL | CPMM type filter not a separate variant |
| Pools — Price History | `GET /pools/line/price` | NO | OHLCV candles for a pool |
| Pools — Price History | `GET /pools/line/liquidity` | NO | Liquidity history over time |
| Pools — TVL/Volume | `GET /pools/info/stats` | NO | Aggregate TVL, 24h volume, fee stats |
| CLMM Specific | `GET /clmm/configs` | NO | CLMM pool config tiers (fee rates) |
| CLMM Specific | `GET /clmm/position/list` | NO | Open CLMM positions by owner |
| CLMM Specific | `GET /clmm/position/open-position-info` | NO | Position APR, uncollected fees |
| CPMM Specific | `GET /cpmm/configs` | NO | CPMM pool config tiers |
| Farms | `GET /farms/info/list` | YES | |
| Farms | `GET /farms/info/ids` | YES | |
| Farms | `GET /farms/info/mine` | NO | Farms by owner wallet |
| Portfolio | `GET /portfolio/position` | NO | User's CLMM + CPMM + AMM positions summary |
| Portfolio | `GET /portfolio/farm` | NO | User's active farm staking |
| IDO | `GET /ido/pool-keys` | YES | |
| Swap Quote | `GET /compute/swap-base-in` | YES | |
| Swap Quote | `GET /compute/swap-base-out` | YES | |
| Swap Tx | `POST /transaction/swap-base-in` | YES | |
| Swap Tx | `POST /transaction/swap-base-out` | YES | |
| Swap Tx | `POST /transaction/clmm-create-position` | NO | Create a new CLMM LP position |
| Swap Tx | `POST /transaction/clmm-increase-liquidity` | NO | Add liquidity to CLMM position |
| Swap Tx | `POST /transaction/clmm-decrease-liquidity` | NO | Remove liquidity from CLMM position |
| Swap Tx | `POST /transaction/clmm-collect-fee` | NO | Collect accumulated CLMM fees |
| Swap Tx | `POST /transaction/cpmm-add-liquidity` | NO | Add liquidity to CPMM pool |
| Swap Tx | `POST /transaction/cpmm-remove-liquidity` | NO | Remove liquidity from CPMM pool |
| Swap Tx | `POST /transaction/stake-farm` | NO | Stake LP tokens in a farm |
| Swap Tx | `POST /transaction/unstake-farm` | NO | Unstake LP tokens from a farm |
| Swap Tx | `POST /transaction/harvest-farm` | NO | Harvest farm rewards |

**Summary of gaps:**
- Missing all CLMM-specific position management endpoints
- Missing all CPMM-specific liquidity endpoints
- Missing portfolio summary endpoints (user-level aggregation)
- Missing price/liquidity history (OHLCV candles for pools)
- Missing farm ownership and farm transaction endpoints
- Pool type (CLMM vs CPMM vs AMM) is a query param, not a separate enum variant — functional gap for type-specific queries

---

### 2. Uniswap

**File:** `crypto/swap/uniswap/endpoints.rs`
**Docs:** https://docs.uniswap.org/api/overview
**API Docs:** https://api-docs.uniswap.org/
**Transport needed:** REST (Trading API), GraphQL (Subgraph), Ethereum JSON-RPC, Ethereum WebSocket

#### Current Implementation

Enum variants: `Quote`, `Swap`, `CheckApproval`, `OrderStatus`, `SwapStatus`, `SwappableTokens`, `PoolsQuery`, `SwapsQuery`, `TokensQuery`, `PositionsQuery`, `FactoryQuery`, `EthCall`, `EthGetBalance`, `EthGetTransactionReceipt`, `EthBlockNumber`

Note: `OrderStatus` maps to `/orders`, `SwapStatus` maps to `/swaps` — these appear to be legacy or non-standard paths.

#### Gap Analysis

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| **Trading API — Swapping** | | | |
| Swap | `POST /swapping/quote` | PARTIAL | Current `/quote` — path may differ in v2 API |
| Swap | `POST /swapping/create_protocol_swap` | NO | Submit a classic swap transaction |
| Swap | `POST /swapping/create_uniswapx_order` | NO | Submit a UniswapX Dutch auction order |
| Swap | `GET /swapping/get_protocol_swap` | NO | Check status of a submitted protocol swap |
| Swap | `GET /swapping/get_uniswapx_order` | NO | Check status of a UniswapX order |
| Swap | `POST /swapping/approval` | NO | Token approval / permit2 handling |
| Swap | `POST /swapping/create_protocol_batch_swap` | NO | Batch multiple swaps in one TX |
| Swap | `POST /swapping/create_protocol_delegated_swap` | NO | EIP-7702 delegated swap |
| **Trading API — Liquidity Provisioning** | | | |
| LP | `POST /liquidity_provisioning/approval` | NO | Approve token for LP operations |
| LP | `POST /liquidity_provisioning/create_position` | NO | Create new v3/v4 LP position |
| LP | `POST /liquidity_provisioning/increase_position` | NO | Add liquidity to existing position |
| LP | `POST /liquidity_provisioning/decrease_position` | NO | Remove liquidity from position |
| LP | `POST /liquidity_provisioning/claim_fees` | NO | Collect accumulated fees |
| LP | `POST /liquidity_provisioning/claim_rewards` | NO | Collect staking/incentive rewards |
| LP | `POST /liquidity_provisioning/migrate_position` | NO | Migrate v3 position to v4 |
| **Trading API — Reference Data** | | | |
| Reference | `GET /reference_data/bridgeable_tokens` | NO | Tokens supported for cross-chain bridge |
| **Trading API — Utility** | | | |
| Utility | `POST /utility/check_wallet_delegation` | NO | EIP-7702 wallet delegation check |
| Utility | `POST /utility/create_calldata` | NO | Create arbitrary calldata |
| Utility | `POST /utility/encode_7702_transactions` | NO | Encode EIP-7702 transactions |
| **Subgraph (GraphQL)** | | | |
| Subgraph | `{pools}` query | YES | |
| Subgraph | `{swaps}` query | YES | |
| Subgraph | `{tokens}` query | YES | |
| Subgraph | `{positions}` query | YES | |
| Subgraph | `{factory}` query | YES | |
| Subgraph | `{bundles}` query — ETH price | NO | ETH/USD price from subgraph |
| Subgraph | `{ticks}` query — tick data for CLMM | NO | Per-tick liquidity for a pool |
| Subgraph | `{transactions}` query | NO | Transaction-level subgraph data |
| Subgraph | v4 subgraph (separate endpoint) | NO | Uniswap v4 pool/hook queries |
| **Ethereum RPC** | | | |
| RPC | `eth_call` | YES | |
| RPC | `eth_getBalance` | YES | |
| RPC | `eth_getTransactionReceipt` | YES | |
| RPC | `eth_blockNumber` | YES | |
| RPC | `eth_getTransactionByHash` | NO | Lookup tx by hash |
| RPC | `eth_getLogs` | NO | Filter event logs (crucial for DEX monitoring) |
| RPC | `eth_getCode` | NO | Check if address is a contract |
| RPC | `eth_estimateGas` | NO | Gas estimation |
| RPC | `eth_sendRawTransaction` | NO | Broadcast signed transaction |
| RPC | `eth_getStorageAt` | NO | Read contract storage slot |
| **Ethereum WebSocket** | | | |
| WS | `eth_subscribe newHeads` | NO | Real-time new blocks |
| WS | `eth_subscribe logs` | NO | Real-time event logs (e.g. Swap events) |
| WS | `eth_subscribe newPendingTransactions` | NO | Mempool monitoring |

**Summary of gaps:**
- The current Trading API enum uses old/incorrect paths (`/orders`, `/swaps`, `/check_approval`, `/swappable_tokens`) — these do not match the actual documented API structure (`/swapping/...`, `/liquidity_provisioning/...`)
- Entire LP provisioning category is missing (7 endpoints)
- UniswapX order flow is missing (create + poll)
- Bridge and EIP-7702 endpoints are missing
- Ethereum RPC coverage is thin (only 4 of ~10 important methods)
- WebSocket subscriptions are entirely absent (real-time block/log streaming)

---

## Batch 7 — Onchain Analytics

---

### 3. Bitquery

**File:** `onchain/analytics/bitquery/endpoints.rs`
**Docs:** https://docs.bitquery.io/
**Transport needed:** GraphQL (HTTPS POST), GraphQL WebSocket (WSS subscriptions)

#### Current Implementation

Enum variants: `Blocks`, `Transactions`, `MempoolTransactions`, `Transfers`, `BalanceUpdates`, `DexTrades`, `NftTrades`, `Events`, `Calls`, `SolanaInstructions`, `BitcoinInputs`, `BitcoinOutputs`

Networks: 20+ EVM + non-EVM chains
Datasets: `Archive`, `Realtime`, `Combined`

#### Gap Analysis

| Category | Cube / Query Type | We Have? | Notes |
|----------|-------------------|----------|-------|
| **EVM Cubes** | | | |
| EVM | `Blocks` | YES | |
| EVM | `Transactions` | YES | |
| EVM | `MempoolTransactions` | YES | |
| EVM | `Transfers` | YES | |
| EVM | `BalanceUpdates` | YES | |
| EVM | `DEXTrades` | YES | |
| EVM | `DEXTradeByTokens` | NO | Trades grouped by token pair — different schema from DEXTrades |
| EVM | `DEXPools` | NO | Liquidity pool metadata and reserves |
| EVM | `NFTTrades` | YES | |
| EVM | `Events` | YES | |
| EVM | `Calls` | YES | |
| EVM | `MinerRewards` | NO | Block reward details (burnt fees, tx fees, total) |
| EVM | `TokenHolders` | NO | Token holder snapshots by date |
| EVM | `Uncles` | NO | Uncle/orphan blocks data |
| **Price Index API** (new 2025) | | | |
| Price | `PriceIndex.Tokens` | NO | Token price per chain (new Bitquery aggregated price cube) |
| Price | `PriceIndex.Currencies` | NO | Cross-chain aggregated price view |
| Price | `PriceIndex.Pairs` | NO | Price and volume by token pair on specific market |
| **Solana-Specific** | | | |
| Solana | `Instructions` (SolanaInstructions) | YES | |
| Solana | `Transactions` | NO | Solana transaction cube (separate from EVM.Transactions) |
| Solana | `Transfers` | NO | Solana token transfers |
| Solana | `BalanceUpdates` | NO | Solana balance changes |
| Solana | `DEXTrades` | NO | Solana DEX trades (Raydium, Orca, etc.) |
| Solana | `DEXPools` | NO | Solana liquidity pools |
| Solana | `Rewards` | NO | Solana staking rewards |
| **Bitcoin-Specific** | | | |
| Bitcoin | `Inputs` | YES | |
| Bitcoin | `Outputs` | YES | |
| Bitcoin | `Transactions` | NO | Bitcoin-specific transaction cube |
| Bitcoin | `Blocks` | NO | Bitcoin block cube |
| Bitcoin | `CoinPath` | NO | Coin flow tracing |
| **WebSocket Subscriptions** | | | |
| WS | Subscription `Blocks` | PARTIAL | Builder exists but not a typed endpoint |
| WS | Subscription `DEXTrades` | PARTIAL | Builder exists but not a typed endpoint |
| WS | Subscription `DEXTradeByTokens` | NO | Not implemented |
| WS | Subscription `Transfers` | NO | |
| WS | Subscription `BalanceUpdates` | NO | |
| WS | Subscription `Transactions` | NO | |
| WS | Subscription `MempoolTransactions` | NO | |
| WS | Subscription `Events` | NO | |
| WS | Subscription `TokenHolders` | NO | |
| **EVM Streams (Protobuf)** | | | |
| Streams | `BlockMessage` (full block protobuf) | NO | Kafka / protobuf streaming (separate from GraphQL WS) |
| Streams | `TokenBlockMessage` | NO | Token-focused block stream |
| Streams | `DexBlockMessage` | NO | DEX-focused block stream |
| Streams | `DexPoolBlockMessage` | NO | Pool liquidity stream |
| **Metadata / Auth** | | | |
| Meta | OAuth token endpoint | NO | Bitquery uses OAuth2 access tokens |

**Summary of gaps:**
- `DEXTradeByTokens` is a critically different cube from `DEXTrades` (different schema, token-pair centric) — missing
- `DEXPools`, `TokenHolders`, `MinerRewards`, `Uncles` EVM cubes are missing
- Entire Solana cube set (beyond Instructions) is absent
- New 2025 Price Index API (3 cubes) is not represented
- WebSocket subscriptions are manually built as strings only — no typed subscription enum
- Protobuf/Kafka streaming transport is not modeled at all

---

### 4. Whale Alert

**File:** `onchain/analytics/whale_alert/endpoints.rs`
**Docs:** https://developer.whale-alert.io/documentation/
**Transport needed:** REST, WebSocket

#### Current Implementation

Enum variants:
- Enterprise v2: `Status`, `BlockchainStatus`, `Transaction`, `Transactions`, `Block`, `AddressTransactions`, `AddressAttributions`
- Developer v1 (deprecated): `StatusV1`, `TransactionV1`, `TransactionsV1`

WebSocket: `wss://leviathan.whale-alert.io/ws` (in `WhaleAlertEndpoints` struct but no enum variant for WS subscriptions)

#### Gap Analysis

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| **Enterprise v2 REST** | | | |
| Status | `GET /status` | YES | |
| Status | `GET /{blockchain}/status` | YES | |
| Transaction | `GET /{blockchain}/transaction/{hash}` | YES | |
| Transactions | `GET /{blockchain}/transactions` | YES | Stream from start height |
| Block | `GET /{blockchain}/block/{height}` | YES | |
| Address | `GET /{blockchain}/address/{hash}/transactions` | YES | |
| Address | `GET /{blockchain}/address/{hash}/owner_attributions` | YES | |
| **Developer v1 REST (deprecated)** | | | |
| v1 | `GET /status` | YES | |
| v1 | `GET /transaction/{blockchain}/{hash}` | YES | |
| v1 | `GET /transactions` | YES | |
| **WebSocket — Custom Alerts** | | | |
| WS | `subscribe_alerts` topic | NO | Real-time whale transaction alerts (threshold-based) |
| WS | `subscribe_socials` topic | NO | Social media mentions of large transactions |
| **WebSocket — Priority Alerts** | | | |
| WS | Priority `subscribe_alerts` | NO | Priority queue variant (enterprise tier) |
| WS | Priority `subscribe_socials` | NO | Priority social feed |

**Summary of gaps:**
- REST coverage is complete for both v1 and v2
- WebSocket alert subscriptions are entirely absent — the WS URL is stored in the struct but there are no typed enum variants for subscription topics
- `subscribe_alerts` is the core real-time use case for Whale Alert and is missing
- `subscribe_socials` (social signal feed) is also absent

---

### 5. Etherscan

**File:** `onchain/ethereum/etherscan/endpoints.rs`
**Docs:** https://docs.etherscan.io/
**Transport needed:** REST only (no WebSocket)

#### Current Implementation

Enum variants: `Balance`, `BalanceMulti`, `TxList`, `TokenTx`, `TxListInternal`, `EthSupply`, `EthPrice`, `ChainSize`, `TokenSupply`, `EthBlockNumber`, `EthGetBlockByNumber`, `BlockReward`, `GasOracle`, `GetAbi`

Note: Uses Etherscan V1 API (`https://api.etherscan.io/api`). Etherscan V2 (multichain) was released 2025.

#### Gap Analysis

| Category | Endpoint (module, action) | We Have? | Notes |
|----------|--------------------------|----------|-------|
| **Account Module** | | | |
| Account | `account, balance` | YES | Single address ETH balance |
| Account | `account, balancemulti` | YES | Multi-address ETH balance |
| Account | `account, txlist` | YES | Normal transactions |
| Account | `account, txlistinternal` | YES | Internal transactions by address |
| Account | `account, tokentx` | YES | ERC-20 transfers |
| Account | `account, tokennfttx` | NO | ERC-721 NFT transfers |
| Account | `account, token1155tx` | NO | ERC-1155 multi-token transfers |
| Account | `account, getminedblocks` | NO | Blocks validated by address |
| Account | `account, balancehistory` | NO | Historical ETH balance at block (Pro) |
| Account | `account, addresstokenbalance` | NO | ERC-20 holdings list |
| Account | `account, addresstokennftbalance` | NO | ERC-721 holdings list |
| Account | `account, addresstokennftinventory` | NO | ERC-721 inventory by contract |
| Account | `account, txlistinternal` (by block range) | PARTIAL | By address only, not block range |
| Account | `account, txlistinternal` (by tx hash) | PARTIAL | Not a separate variant |
| Account | `account, deposittx` | NO | L1→L2 deposit transactions |
| Account | `account, withdrawaltx` | NO | L2→L1 withdrawal transactions |
| Account | `account, beaconchainwithdrawals` | NO | Beacon chain (ETH2) withdrawals |
| Account | `account, plasmadeposits` | NO | Polygon plasma deposits |
| **Contract Module** | | | |
| Contract | `contract, getabi` | YES | |
| Contract | `contract, getsourcecode` | NO | Contract source code + compiler info |
| Contract | `contract, getcontractcreation` | NO | Contract creator address + deploy tx |
| Contract | `contract, checkverifystatus` | NO | Check verification job status |
| Contract | `contract, verifysourcecode` | NO | Submit source for verification |
| Contract | `contract, verifyproxycontract` | NO | Verify proxy contract |
| **Transaction Module** | | | |
| Transaction | `transaction, getstatus` | NO | Check if tx was reverted |
| Transaction | `transaction, gettxreceiptstatus` | NO | Get tx receipt status (1=success) |
| **Block Module** | | | |
| Block | `block, getblockreward` | YES | |
| Block | `block, getblockcountdown` | NO | Estimated blocks until target block |
| Block | `block, getblocknobytime` | NO | Block number at timestamp |
| **Logs Module** | | | |
| Logs | `logs, getLogs` (by address) | NO | Event logs by contract address |
| Logs | `logs, getLogs` (by address + topics) | NO | Filtered event logs |
| Logs | `logs, getLogs` (by topics only) | NO | Topic-filtered logs |
| **Token Module** | | | |
| Token | `stats, tokensupply` | YES | (currently in Stats enum) ERC-20 supply |
| Token | `account, tokenbalance` | NO | ERC-20 balance for specific token+address |
| Token | `token, tokeninfo` | NO | Token metadata (name, symbol, decimals, website) |
| Token | `token, tokenholderlist` | NO | List of token holders |
| Token | `token, tokenholdercount` | NO | Count of token holders |
| Token | `token, toptokenholders` | NO | Top N holders by balance |
| Token | `account, historicaltokenbalance` | NO | Historical ERC-20 balance (Pro) |
| Token | `stats, tokensupplyhistory` | NO | Historical token supply (Pro) |
| **Gas Tracker Module** | | | |
| Gas | `gastracker, gasoracle` | YES | |
| Gas | `gastracker, gasestimate` | NO | Gas estimate for tx confirmation time |
| Gas | `stats, dailygaslimit` | NO | Daily gas limit history |
| Gas | `stats, dailygasused` | NO | Daily gas used |
| Gas | `stats, dailyavggasprice` | NO | Daily average gas price |
| **Stats Module** | | | |
| Stats | `stats, ethsupply` | YES | |
| Stats | `stats, ethsupply2` | NO | Total supply incl. ETH2 staking |
| Stats | `stats, ethprice` | YES | |
| Stats | `stats, chainsize` | YES | |
| Stats | `stats, nodecount` | NO | Total Ethereum nodes |
| Stats | `stats, dailynewaddress` | NO | New addresses per day |
| Stats | `stats, dailynetworkutilization` | NO | Network utilization % |
| Stats | `stats, dailyavgblocksize` | NO | Daily average block size |
| Stats | `stats, dailyblkcount` | NO | Daily block count + rewards |
| Stats | `stats, dailytx` | NO | Daily transaction count |
| Stats | `stats, dailynettxfee` | NO | Daily network transaction fees |
| Stats | `stats, dailyuncleblkcount` | NO | Daily uncle block count |
| Stats | `stats, dailyavgblocktime` | NO | Daily average block time |
| Stats | `stats, etherpricehist` | NO | ETH price history |
| Stats | `stats, dailyavghashrate` | NO | Network hash rate history |
| Stats | `stats, dailyavgnetdifficulty` | NO | Network difficulty history |
| Stats | `stats, dailymktcap` | NO | Daily market cap (Pro) |
| **Proxy (JSON-RPC) Module** | | | |
| Proxy | `proxy, eth_blockNumber` | YES | |
| Proxy | `proxy, eth_getBlockByNumber` | YES | |
| Proxy | `proxy, eth_getTransactionByHash` | NO | |
| Proxy | `proxy, eth_getTransactionByBlockNumberAndIndex` | NO | |
| Proxy | `proxy, eth_getTransactionCount` | NO | Nonce |
| Proxy | `proxy, eth_sendRawTransaction` | NO | Broadcast transaction |
| Proxy | `proxy, eth_getTransactionReceipt` | NO | |
| Proxy | `proxy, eth_call` | NO | |
| Proxy | `proxy, eth_getCode` | NO | |
| Proxy | `proxy, eth_getStorageAt` | NO | |
| Proxy | `proxy, eth_gasPrice` | NO | |
| Proxy | `proxy, eth_estimateGas` | NO | |
| Proxy | `proxy, eth_getBlockTransactionCountByNumber` | NO | |
| Proxy | `proxy, eth_getUncleByBlockNumberAndIndex` | NO | |
| **V2 API (Multichain)** | | | |
| V2 | Base URL `https://api.etherscan.io/v2/api?chainid={id}` | NO | V2 supports 60+ chains via chainid param |
| V2 | `chainlist` action | NO | List all supported chains |

**Summary of gaps:**
- Coverage is sparse: ~14 of ~70+ documented actions
- Entire `logs` module is missing (critical for DeFi event monitoring)
- Entire `token` module actions beyond supply are missing
- Contract module is only partially covered (getabi only)
- All `transaction` module actions (status checks) are missing
- Most `proxy` JSON-RPC endpoints are absent
- Most `stats` daily history endpoints are absent
- No ERC-721 or ERC-1155 transfer tracking
- Etherscan V2 multichain API not modeled at all

---

## Batch 8 — Prediction Markets

---

### 6. Polymarket

**File:** `prediction/polymarket/endpoints.rs`
**Docs:** https://docs.polymarket.com/
**Transport needed:** REST (CLOB, Gamma, Data APIs), WebSocket (CLOB + Sports + RTDS)

#### Current Implementation

Enum variants: `ClobMarkets`, `ClobMarket`, `OrderBook`, `Midpoint`, `Price`, `Spread`, `LastTradePrice`, `PricesHistory`, `Time`, `GammaEvents`, `GammaEvent`, `GammaMarkets`, `GammaMarket`, `ClobOrders`, `ClobOrder`, `DataPositions`

#### Gap Analysis

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| **CLOB API — Public Market Data** | | | |
| CLOB | `GET /markets` | YES | |
| CLOB | `GET /markets/{condition_id}` | YES | |
| CLOB | `GET /book?token_id=...` | YES | Single order book |
| CLOB | `GET /books` (batch) | NO | Multiple order books in one call |
| CLOB | `GET /midpoint?token_id=...` | YES | |
| CLOB | `GET /midpoints` (batch) | NO | Multiple midpoints |
| CLOB | `GET /price?token_id=...&side=...` | YES | |
| CLOB | `GET /prices` (batch) | NO | Multiple prices |
| CLOB | `GET /spread?token_id=...` | YES | |
| CLOB | `GET /spreads` (batch) | NO | Multiple spreads |
| CLOB | `GET /last-trade-price?token_id=...` | YES | |
| CLOB | `GET /last-trades-prices` (batch) | NO | Multiple last trade prices |
| CLOB | `GET /prices-history` | YES | OHLCV / price history |
| CLOB | `GET /time` | YES | Server time |
| CLOB | `GET /tick-size?token_id=...` | NO | Minimum price increment for market |
| CLOB | `GET /fee-rate-bps?token_id=...` | NO | Fee rate in basis points |
| CLOB | `GET /neg-risk?token_id=...` | NO | Whether market uses neg-risk mechanism |
| CLOB | `GET /simplified-markets` | NO | Lightweight market listing |
| CLOB | `GET /sampling-markets` | NO | Markets eligible for liquidity rewards |
| CLOB | `GET /sampling-simplified-markets` | NO | Simplified sampling markets |
| CLOB | `GET /market-trades-events/{condition_id}` | NO | Recent trade events for market |
| CLOB | `POST /calculate-market-price` | NO | Estimate market order execution price |
| **CLOB API — Authenticated (L1/L2) — Orders** | | | |
| CLOB Auth | `POST /order` (create single limit order) | NO | Place limit order (L1 signed) |
| CLOB Auth | `POST /orders` (batch up to 15) | NO | Batch order placement |
| CLOB Auth | `DELETE /order/{id}` | NO | Cancel single order |
| CLOB Auth | `DELETE /orders` (batch cancel) | NO | Batch cancel orders |
| CLOB Auth | `DELETE /cancel-all` | NO | Cancel all open orders |
| CLOB Auth | `DELETE /cancel-market-orders?asset_id=...` | NO | Cancel all orders for market |
| CLOB Auth | `GET /orders` | YES (listed) | List open orders |
| CLOB Auth | `GET /orders/{id}` | YES (listed) | Single order details |
| CLOB Auth | `POST /market-order` | NO | Market order (immediate fill) |
| **CLOB API — Authenticated — Account** | | | |
| CLOB Auth | `GET /trades` | NO | User trade history |
| CLOB Auth | `GET /balance-allowance` | NO | USDC balance + token allowance |
| CLOB Auth | `GET /api-keys` | NO | List API keys for account |
| CLOB Auth | `DELETE /api-key` | NO | Revoke current API key |
| CLOB Auth | `POST /auth/api-key` (derive) | NO | Derive L2 API key from L1 signature |
| CLOB Auth | `GET /notifications` | NO | Account event notifications (48h retention) |
| CLOB Auth | `DELETE /notifications` | NO | Dismiss notifications |
| **Gamma API** | | | |
| Gamma | `GET /events` | YES | |
| Gamma | `GET /events/{id}` | YES | |
| Gamma | `GET /markets` | YES | |
| Gamma | `GET /markets/{id}` | YES | |
| **Data API** | | | |
| Data | `GET /positions?user=...` | YES | User positions |
| Data | `GET /trades` | NO | All trades (public trade history) |
| Data | `GET /activity?user=...` | NO | User activity log |
| Data | `GET /value?user=...` | NO | User portfolio value |
| **WebSocket — Market Channel** | | | |
| WS | `wss://ws-subscriptions-clob.polymarket.com/ws/market` | PARTIAL | URL in struct, no typed subscription enum |
| WS | `book` message type | NO | Order book snapshots |
| WS | `price_change` message type | NO | Order book level updates |
| WS | `tick_size_change` message type | NO | Tick size adjustments |
| WS | `last_trade_price` message type | NO | Trade executions |
| WS | `best_bid_ask` message type | NO | Best bid/ask (requires feature flag) |
| WS | `new_market` message type | NO | Newly created markets |
| WS | `market_resolved` message type | NO | Market resolution events |
| **WebSocket — User Channel** | | | |
| WS | `wss://ws-subscriptions-clob.polymarket.com/ws/user` | NO | Not in struct or enum |
| WS | `trade` message type | NO | Order fill lifecycle events |
| WS | `order` message type | NO | Order state changes |
| **WebSocket — Sports Channel** | | | |
| WS | `wss://sports-api.polymarket.com/ws` | NO | Sports market live scores |
| WS | `sport_result` message type | NO | Live game scores, periods, status |
| **WebSocket — RTDS** | | | |
| WS | `wss://ws-live-data.polymarket.com` | NO | Real-time data socket |
| **Polygon On-Chain** | | | |
| Onchain | Polygon JSON-RPC (for order signing) | NO | L1 auth requires Polygon chain interaction |
| Onchain | CTF Exchange contract calls | NO | Conditional Token Framework contract |

**Summary of gaps:**
- All order placement/cancellation endpoints are missing (POST /order, DELETE /order, etc.)
- All batch variants (books, prices, midpoints, etc.) are absent
- Account management endpoints missing (API keys, notifications, balance)
- User trade history (`GET /trades`) is absent
- All WebSocket subscription types are not typed — 4 channels with ~10 message types
- Data API is mostly absent beyond positions
- Market utility endpoints (tick-size, fee-rate, neg-risk) are missing
- On-chain signing infrastructure (Polygon RPC) not modeled

---

### 7. PredictIt

**File:** `intelligence_feeds/prediction/predictit/endpoints.rs`
**Docs:** https://www.predictit.org/api/marketdata/
**Transport needed:** REST only (no WebSocket available)

#### Current Implementation

Enum variants: `AllMarkets`, `Market`
- `AllMarkets` → `/all`
- `Market` → `/markets` (incomplete — the correct path is `/markets/{id}`)

Note: The `Market` variant maps to the generic `/markets` path without an ID parameter, which is incorrect — the actual endpoint requires a numeric market ID or a ticker symbol.

#### Gap Analysis

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| **Market Data** | | | |
| Markets | `GET /marketdata/all` | YES | All markets with all contracts |
| Markets | `GET /marketdata/markets/{id}` | PARTIAL | Variant exists but path is wrong (missing `/{id}`) |
| Markets | `GET /marketdata/ticker/{ticker}` | NO | Single market by ticker symbol (e.g. `SYKES.SCOTUS.NEXTJUSTICE`) |
| Markets | `GET /marketdata/category/{category_id}` | NO | Markets filtered by category (4=World, 6=US Elections, 13=US Politics) |
| Markets | `GET /marketdata/group/{group_id}` | NO | Markets filtered by group ID (14 groups) |
| **WebSocket** | | | |
| WS | None | N/A | PredictIt does not offer a WebSocket API |

**Summary of gaps:**
- The `Market` path variant is incorrect (missing `/{id}` suffix) — functional bug
- `ticker/{ticker}` lookup is absent (useful for querying by known ticker)
- `category/{id}` filter is absent (useful for political market filtering)
- `group/{id}` filter is absent
- Data update frequency is 60 seconds — no real-time mechanism exists
- API is extremely limited: PredictIt has largely shut down (announced closure, limited operations) — the public API may be unreliable

---

## Cross-Cutting Notes

### Transport Summary by Provider

| Provider | REST | GraphQL | WebSocket | Ethereum RPC | Solana RPC | Polygon RPC |
|----------|------|---------|-----------|--------------|------------|-------------|
| Raydium | YES | — | — | — | for tx broadcast | — |
| Uniswap | YES | YES (Subgraph) | YES (ETH events) | YES | — | — |
| Bitquery | — | YES | YES (subscriptions) | — | — | — |
| Whale Alert | YES | — | YES (alerts) | — | — | — |
| Etherscan | YES | — | — | — | — | — |
| Polymarket | YES | — | YES (4 channels) | — | — | YES (for signing) |
| PredictIt | YES | — | — | — | — | — |

### Priority Gaps (Highest Impact)

1. **Polymarket order management** — Cannot trade without `POST /order`, `DELETE /order`, `GET /trades`. The connector is read-only.
2. **Uniswap LP provisioning** — 7 endpoints for creating/managing liquidity positions are missing entirely.
3. **Etherscan logs module** — `getLogs` is fundamental for DeFi event monitoring (Swap events, Transfer events) and is absent.
4. **Raydium CLMM position management** — CLMM is the primary Raydium pool type; create/increase/decrease/collect endpoints are all missing.
5. **Bitquery `DEXTradeByTokens`** — Different schema from `DEXTrades`, optimized for token pair queries — missing.
6. **Whale Alert WebSocket** — `subscribe_alerts` is the primary real-time use case for the service; absent entirely.
7. **Polymarket WebSocket** — All 4 channels with ~10 message types are untyped.
8. **Etherscan token module** — `tokenbalance`, `tokeninfo`, `tokenholderlist` are absent but fundamental for ERC-20 analytics.

---

## Sources

- [Raydium API Docs](https://docs.raydium.io/raydium/protocol/developers/api)
- [Raydium Swagger UI](https://api-v3.raydium.io/docs/)
- [Uniswap API Overview](https://docs.uniswap.org/api/overview)
- [Uniswap API Docs (api-docs.uniswap.org)](https://api-docs.uniswap.org/)
- [Uniswap Quote Reference](https://api-docs.uniswap.org/api-reference/swapping/quote)
- [Bitquery Blockchain Introduction](https://docs.bitquery.io/docs/blockchain/introduction/)
- [Bitquery EVM Cubes](https://docs.bitquery.io/docs/cubes/EVM/)
- [Bitquery EVM Subscription Reference](https://docs.bitquery.io/docs/graphql-reference/objects/evm-subscription/)
- [Bitquery DEXTradesByTokens Cube](https://docs.bitquery.io/docs/cubes/dextradesbyTokens/)
- [Whale Alert Developer Documentation](https://developer.whale-alert.io/documentation/)
- [Etherscan API Documentation](https://docs.etherscan.io/)
- [Etherscan LLMs Index](https://docs.etherscan.io/llms.txt)
- [Polymarket CLOB Introduction](https://docs.polymarket.com/developers/CLOB/introduction)
- [Polymarket Public Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-public)
- [Polymarket L2 Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-l2)
- [Polymarket WebSocket Overview](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview)
- [Polymarket Market Channel](https://docs.polymarket.com/developers/CLOB/websocket/market-channel)
- [PredictIt GitHub wrappers](https://github.com/topics/predictit-api)
