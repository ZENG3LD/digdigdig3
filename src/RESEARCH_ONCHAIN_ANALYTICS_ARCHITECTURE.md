# On-Chain Analytics Architecture Research

Research compiled for the NEMO trading terminal on-chain analytics layer.
Date: 2026-03-14

---

## Table of Contents

1. [What Professional Platforms Actually Do](#1-what-professional-platforms-actually-do)
2. [Per-Chain Monitoring Capabilities](#2-per-chain-monitoring-capabilities)
3. [Architecture Options](#3-architecture-options)
4. [Actionable Trading Signals](#4-actionable-trading-signals)
5. [Implementation Priority](#5-implementation-priority)
6. [Recommended Architecture for NEMO](#6-recommended-architecture-for-nemo)
7. [Sources](#sources)

---

## 1. What Professional Platforms Actually Do

### Nansen (Smart Money / Wallet Intelligence)

**Core model**: Label 500M+ wallets into behavioral categories, then track their movements.

Categories they assign:
- Smart Money (wallets with historically profitable trades)
- VCs / Funds (known institutional wallets)
- Whales (large holders by token)
- Exchanges (hot/cold wallets)
- Liquidity Providers (DeFi)
- NFT traders
- Deployers / Developers

**What they track per wallet**:
- Portfolio composition over time
- Win rate on token positions
- Realized profits (entry/exit pairs)
- Exchange deposits/withdrawals
- DeFi protocol interactions

**What they alert on**:
- Whale accumulation / distribution clusters
- Smart money entering new tokens early
- Exchange inflow spikes (sell pressure signal)
- Cold storage outflows (accumulation signal)
- Liquidity pool entry/exit by large players

**Key insight for NEMO**: The value is the labeled database, not the raw queries. Without pre-labeled wallets, you are building raw detection heuristics. The raw signals are still useful (large transfer = alert), but signal quality is much lower without labels.

---

### Arkham Intelligence (Entity Attribution + Fund Flows)

**Core model**: AI system (ULTRA) that deanonymizes blockchain addresses into real-world entities.

Three-tier metadata:
1. **Entity**: Group of addresses belonging to one organization (e.g., "Binance")
2. **Label**: Specific wallet designation within entity (e.g., "Binance Hot Wallet #3")
3. **Tag**: Activity descriptor (e.g., "receives_institutional_deposits")

Scale: 450,000+ entity pages, 800M+ address labels.

**Data sources**:
- On-chain behavior patterns (clustering heuristics)
- Off-chain: news, social media, public records
- Community marketplace: users submit labeled addresses for ARKM token rewards

**What they produce**:
- Money flow graphs (funds moved between entities)
- Wallet portfolio snapshots
- Cross-chain address attribution (same entity across ETH, BTC, SOL)

**Key insight for NEMO**: Entity labeling at Arkham's scale requires ML + community. For a trading terminal, use their API as a data source rather than replicating internally. The raw on-chain behavior (clustering heuristics) can be partially replicated with UTXO analysis and EVM common-input heuristic.

---

### Glassnode (Bitcoin Macro On-Chain Metrics)

**Core model**: Derive market-cycle indicators from UTXO and supply distribution data.

**Key metrics they calculate**:

| Metric | What It Measures | How Calculated |
|--------|-----------------|----------------|
| SOPR | Spent Output Profit Ratio | price_at_spend / price_at_creation for each UTXO |
| NUPL | Net Unrealized P&L | (market_cap - realized_cap) / market_cap |
| Exchange Balance | BTC held on exchanges | Sum of UTXOs flowing to/from labeled exchange addresses |
| Exchange Netflow | Net BTC flow to exchanges | inflow - outflow, rolling window |
| Realized Cap | Market cap at cost basis | Sum of all UTXO values at their creation price |
| Hodler Supply | BTC unmoved >1yr | UTXOs with age > 365 days |
| Miner Outflows | Miner selling pressure | Coinbase tx outputs moving to exchanges |
| Long/Short Term Holder supply | Investor cohort analysis | UTXO age-band analysis |

**Exchange detection methodology**:
1. Verified addresses (confirmed directly with exchanges)
2. External sources (public records, labeling databases)
3. Clustering heuristics: addresses that co-spend together are clustered as same entity

**Critical caveat**: Exchange flow data has significant noise. Single large transactions should be treated as preliminary until confirmed hours later (internal transfers vs. real withdrawals). Historical data gets revised retroactively as new addresses are discovered.

**Key insight for NEMO**: SOPR, NUPL, and realized cap require a full UTXO history database. Exchange flow analysis requires a labeled address set. Both are expensive to build from scratch. For a v1, use Glassnode's API as a source; for v2, implement exchange flow detection using a maintained address list.

---

### Dune Analytics (SQL Queries on Indexed Chain Data)

**Core model**: Indexed decoded event data into SQL-queryable tables. Users write queries.

**Most popular query categories**:
- DEX volume aggregation by protocol and chain
- Whale wallet activity (custom threshold, e.g., >$100K tx)
- Token holder distribution (Gini coefficient of supply)
- Protocol revenue (fee collection events)
- NFT market trends (mint + transfer volumes)
- Governance participation rates
- Bridge flow volumes between chains
- MEV/sandwich bot activity
- Stablecoin supply changes

**What Dune does NOT handle well**:
- Real-time (queries are batch, typically 30s+ lag)
- Mempool (pre-confirmation data)
- Sub-second latency requirements

**Key insight for NEMO**: Dune's model (pre-decoded event tables + SQL) is the right abstraction for analytics dashboards. For a trading terminal, you need lower latency. Use Dune for historical analysis and backtesting signal quality; use your own pipeline for real-time.

---

### DeBank (DeFi Portfolio Tracking)

**What they track per address**:
- Token balances across 50+ EVM chains
- DeFi positions: LP shares, lending deposits, staking amounts
- NFT holdings
- Transaction history with decoded protocol labels
- Yield/APY for active positions
- Historical PnL per position

**Key insight for NEMO**: DeBank is the reference implementation for multi-chain portfolio tracking. They use RPC calls + indexed data. For NEMO, the equivalent is a per-address position tracker that resolves LP token values to underlying assets.

---

### Chainalysis (Compliance + Transaction Tracing)

**Core model**: Entity attribution for compliance/law enforcement. Same clustering as Arkham but oriented toward risk scoring.

**What they do**:
- VASP (exchange) identification
- Darknet market address labeling
- Ransomware wallet tracking
- Risk score per address (0-100)
- Transaction path tracing (follow funds n hops)
- Lightning Network transaction monitoring (unique capability)
- Cross-chain bridge tracing

**Lightning Network specifics**:
- Channel open/close transactions are on-chain and trackable
- Internal channel payments are NOT on-chain (private)
- Chainalysis monitors the on-chain channel lifecycle events only

---

## 2. Per-Chain Monitoring Capabilities

### EVM Chains (Ethereum, Arbitrum, Optimism, Base, BSC, etc.)

#### What Is Monitorable via RPC

**ERC-20 Token Transfers**
- Event: `Transfer(address indexed from, address indexed to, uint256 value)`
- Topic0 (keccak256): `0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef`
- Filter: `eth_getLogs` with topic0 filter + contract address
- Whale detection: filter `value > threshold` (value is NOT indexed, requires post-filter)

**Uniswap V2 Swaps**
- Event: `Swap(address indexed sender, address indexed to, uint256 amount0In, uint256 amount1In, uint256 amount0Out, uint256 amount1Out)`
- Topic0: `0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822`
- Data: amounts in/out per side

**Uniswap V3 Swaps**
- Event: `Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)`
- Topic0: `0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67`
- Note: sqrtPriceX96 gives exact post-swap price; tick gives range position

**Uniswap V3 Liquidity Events**
- Mint (add liquidity): `Mint(address sender, address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1)`
  - Topic0: `0x7a53080ba414158be7ec69b987b5fb7d07dee101fe85488f0853ae16239d0bde`
- Burn (remove liquidity): similar structure
  - Topic0: `0x0c396cd989a39f4459b5fa1aed6a9a8dcdbc45908acfd67e028cd568da98982c`

**ERC-721 NFT Transfers**
- Same `Transfer` topic0 as ERC-20, but `tokenId` is the third indexed param
- Distinguish from ERC-20 by: value field = tokenId (uint256), ERC-20 Transfer has no tokenId

**Mempool Monitoring**
- Method: `eth_subscribe("newPendingTransactions")` via WebSocket
- Returns tx hashes; fetch full tx with `eth_getTransactionByHash`
- Use case: front-running detection, gas price trend, large pending transfers
- Limitation: mempool is node-local; different nodes see different pending txs
- For full mempool coverage: need multiple nodes or specialized APIs (Blocknative, Bloxroute)

**Gas Price Tracking**
- `eth_gasPrice` (legacy) or `eth_feeHistory` (EIP-1559)
- `eth_feeHistory(blockCount, "latest", [25, 50, 75])` returns base fee + priority fee percentiles per block
- Useful signal: gas spike = high network activity = potential volatility

**Smart Contract State (arbitrary)**
- `eth_call` with ABI-encoded calldata for any view function
- Example: Uniswap V3 pool slot0 (current price, tick, liquidity) = `slot0()` selector `0x3850c7bd`
- Example: ERC-20 balance of address = `balanceOf(address)` selector `0x70a08231`
- Example: Aave reserve data for utilization rate

**Exchange Address Detection (heuristics)**
- Common-input-ownership heuristic: UTXOs spent together likely belong to same entity
- Address clustering: if address A always funds address B, likely same organization
- Practical approach for EVM: maintain a curated list, supplement with on-chain behavior patterns

#### EVM Event Monitoring Architecture (Practical)

```
eth_subscribe("logs", {
    "topics": [
        ["0xddf252ad...", "0xd78ad95f...", "0xc42079f9..."],  // Transfer OR UniV2Swap OR UniV3Swap
    ]
})
```

This single subscription catches all ERC-20 transfers AND both Uniswap swap variants in real time.

---

### Bitcoin

#### What Is Monitorable via RPC

**UTXO Analysis**
- All UTXOs are queryable; the full UTXO set is ~5GB
- `listunspent` returns UTXOs for loaded wallets
- For arbitrary address monitoring: needs custom UTXO index or electrum server
- SOPR calculation: requires mapping each spent output to its creation tx (cost basis)

**Large Transaction Detection**
- `getblocktemplate` / `getblock` / `getrawtransaction` to decode transactions
- Filter by output value: `vout[].value > threshold`
- Currently actionable threshold: >100 BTC for "whale" tier

**Exchange Inflow/Outflow (practical approach)**
- Maintain a labeled address set (known exchange hot/cold wallets)
- Monitor these addresses with `getreceivedbyaddress` or via block scanning
- Public datasets: Glassnode, BitcoinAbuse, community-maintained lists

**Miner Behavior**
- Coinbase transactions = first tx in each block (no vin, or vin[0].coinbase set)
- Mining pool identification: parse coinbase scriptSig for pool tags (e.g., "Foundry USA", "AntPool", "/ViaBTC/")
- Miner selling signal: coinbase output flowing to known exchange address within N blocks

**Mempool Analysis**
- `getmempoolinfo` — size, bytes, fees
- `getrawmempool(true)` — full mempool with fee rates
- `getmempoolancestors` / `getmempooldescendants` — CPFP chains
- Fee rate histogram: sort by `feerate`, build percentile distribution
- Use case: fee estimation, congestion detection, large pending tx monitoring

**Ordinals / Runes Activity**
- Ordinals: inscriptions in SegWit witness data; detect by parsing witness fields for `OP_FALSE OP_IF` envelope
- Runes: protocol messages in OP_RETURN outputs with `RUNE_TEST` / `R` prefix bytes
- Both require custom script parser on top of raw tx data
- Signal: Ordinals/Runes activity spikes = block space demand = fee pressure

**UTXO Age Distribution (Diamond Hands / Coin Days Destroyed)**
- For each block, calculate: `sum(value_i * age_i)` = Coin Days Destroyed
- High CDD = long-dormant coins moving = potential sell pressure

**Lightning Network (on-chain signals only)**
- Channel open: 2-of-2 multisig P2WSH output (detectable by script pattern)
- Channel close (cooperative): P2WSH input spending the funding output
- Channel close (force): same, but with timelock script
- Signal: rapid channel closure waves = network stress / potential BTC unlock for selling

---

### Solana

#### What Is Monitorable

**Program Interaction Monitoring (Geyser)**
- Helius Geyser Enhanced Websockets: subscribe to any program address
- Returns all confirmed transactions interacting with that program
- Filters: `transactions`, `accounts`, `slots`, `blocks`

```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "transactionSubscribe",
    "params": [{
        "accountInclude": ["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"]
    }]
}
```
(address `675kPX9...` = Raydium AMM V4 program)

**Raydium Swap Detection (log-based)**
- Subscribe to Raydium program address
- Filter transactions containing log: `"ray_log:"` prefix
- Parse base64-encoded ray_log payload for swap direction, amounts, prices

**Raydium Pool Creation**
- Filter for log: `"initialize2: InitializeInstruction2"`
- Extracts AMM ID, token mint pair, initial liquidity

**Jupiter Swap Detection**
- Jupiter Aggregator V6 program: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`
- Parse instruction data to extract: route (which DEXes used), input/output mints, amounts
- Jupiter logs structured events; decode route steps for full path

**Token Account Monitoring**
- `accountSubscribe` on a specific token account (Associated Token Account)
- Returns real-time balance updates (lamports + token amount)
- Use case: track whale wallet's specific token holdings

**Validator Performance**
- `getVoteAccounts` — active validators + stake weights
- `getLeaderSchedule` — which validator produces which slots
- `getInflationReward` — rewards per epoch per validator

**MEV / Sandwich Detection**
- Pattern: tx A (buy) → target tx → tx B (sell) within same block
- Detect by: same wallet appearing as buyer just before and seller just after target tx
- Jito bundles: many Solana MEV operations go through Jito block engine; monitor Jito tip accounts

**New Token Launch Detection**
- Pump.fun program: `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`
- Token creation events: `InitializeMint` instructions on Token Program
- Cross with: first Raydium pool creation for that mint = token launched

---

### Cosmos / Osmosis

#### What Is Monitorable

**IBC Transfer Tracking**
- Cosmos SDK events: `message.action = /ibc.core.channel.v1.MsgRecvPacket`
- Event attributes: `packet_src_channel`, `packet_dst_channel`, `fungible_token_packet.denom`, `fungible_token_packet.amount`, `packet_receiver`
- Subscribe via Tendermint WebSocket: `subscribe?query="message.action='/ibc.applications.transfer.v1.MsgTransfer'"`

**DEX Pool Liquidity Changes (Osmosis)**
- Events: `pool_joined`, `pool_exited` (Balancer/CFMM pools)
- Events: `cl_pool_created`, `add_to_position`, `withdraw_position` (Concentrated Liquidity)
- Query: `osmosis.gamm.v1beta1.EventPoolJoined`
- Monitor large LP changes: `tokens_in` or `tokens_out` > threshold

**Governance Monitoring**
- Proposals: `cosmos.gov.v1beta1.MsgSubmitProposal`
- Votes: `cosmos.gov.v1beta1.MsgVote`
- Threshold events: when voting power crosses 33% (veto risk) or 50% (pass risk)
- Subscribe to proposal state changes for parameter change proposals

**Validator Slashing Events**
- Event: `slash` in BeginBlocker events
- Attributes: `address` (validator operator), `power`, `reason` (double_sign / downtime)
- Slash packet over IBC CCV channel for consumer chains
- Significant slash = validator losing stake = delegator funds at risk

**Staking Activity**
- `cosmos.staking.v1beta1.MsgDelegate` / `MsgUndelegate`
- Large unstaking events: signal of potential upcoming sell pressure (21-day unbonding)
- Validator redelegation flows indicate validator trust changes

**Tendermint RPC WebSocket Pattern**
```
wss://rpc.osmosis.zone/websocket
SUBSCRIBE query="tm.event='Tx' AND transfer.recipient='osmo1...'"
```

---

### Sui

#### What Is Monitorable

**Move Event Monitoring**
- `suix_queryEvents(query, cursor, limit, descending_order)`
- Query types: `ByPackage`, `ByModule`, `ByEventType`, `BySender`, `ByTransaction`
- Real-time subscription: `suix_subscribeEvent` (WebSocket)

**DeepBook Order Book Events (V3)**
- Package: `0x000000000000000000000000000000000000000000000000000000000000dee9`
- Events: `OrderPlaced`, `OrderCanceled`, `OrderFilled`, `BalanceManager.TradeProof`
- Field `OrderFilled`: `base_asset_quantity_filled`, `price`, `maker_order_id`, `taker_order_id`
- Query example:
```json
{
    "MoveEventType": "0xdee9::clob_v2::OrderFilled"
}
```

**Object Ownership Changes**
- `sui_getObject` — current owner of any object
- `sui_queryTransactionBlocks` — filter by `ChangedObject` for specific object ID
- LP token transfers: track ownership of specific pool share objects

**Coin Transfer Monitoring**
- `suix_getCoins` — all coin objects owned by address
- Event: `CoinReceived` from `0x2::coin` module
- SUI native transfers: `sui_getTransactionBlock` with `TransactionKind::ProgrammableTransaction`

**Cetus DEX (largest Sui DEX)**
- Package: query Cetus CLMM package for swap events
- Events: `SwapEvent` with fields `amount_in`, `amount_out`, `pool`, `a2b` (direction)

---

### TON

#### What Is Monitorable

**Jetton Transfers**
- Jetton Transfer op-code: `0x0f8a7ea5`
- Structure: source → Jetton Wallet Contract → destination Jetton Wallet Contract
- Monitor via TON Center API or TON API (toncenter.com)
- `getTransactions(address, limit, lt, hash)` — polling-based (no native WebSocket push)

**STON.fi Swap Detection**
- STON.fi router contract: parse incoming messages with op-code `0x25938561` (swap)
- Event fields after parsing: `token_wallet`, `min_out`, `receiver`, `referral_address`
- Confirmed swap: response message from pool with `op 0xf93bb43f`

**DeDust Swap Detection**
- DeDust Protocol 2.0: factory + vault + pool architecture
- Vault receives deposits; Pool executes swaps
- Op-code for swap: `0x61ee542d` (native TON vault) / `0x7362d09c` (Jetton vault)
- Parse message body from vault to pool for amounts and direction

**Staking / Validator Activity**
- Elector contract: `Ef8zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzM0vF`
- Stake deposits: messages to elector with `op = 0x654c5074` (new_stake)
- Validator election events: monitor elector contract message history

**Practical TON Monitoring**
- TON has no native event/log system like EVM
- Everything is message-passing between contracts
- Monitor by: polling contract transaction history + parsing message bodies by op-code
- TONAPI.io provides indexed data with decoded messages

---

### Aptos

#### What Is Monitorable

**Module Events (Modern API)**
- `aptos_client.get_events_by_event_handle(address, event_handle_struct, field_name)`
- Or `aptos_client.get_events_by_creation_number(address, creation_number)`
- New format: `aptos_client.get_events(event_type_str)` for module events (framework 1.7+)

**Resource Monitoring**
- `aptos_client.get_account_resource(address, resource_type)` — current state
- `aptos_client.get_account_resources(address)` — all resources for an account
- Useful for: reading LP positions, staking state, AMM pool composition

**DEX Monitoring (PancakeSwap / Liquidswap / Thala)**
- PancakeSwap on Aptos: `0xc7efb4076dbe143cbcd98cfaaa929ecfc8f299203dfff63b95ccb6bfe19850fa::swap::Swap`
- Liquidswap: `0x190d44266241744264b964a37b8f09863167a12d3e70cda39376cfb4e3561e12::liquidity_pool::SwapEvent`
- Both emit typed Move events; query by event type

**Coin Transfer Detection**
- `aptos_client.get_transactions_by_payload_type(...)` or block scanning
- `0x1::coin::TransferEvent` for native coin movements
- FA (Fungible Asset) standard: `0x1::fungible_asset::TransferEvent` (newer tokens)

**Whale Detection on Aptos**
- Query top holders of `0x1::aptos_coin::AptosCoin` resource
- Monitor large `CoinStore<AptosCoin>` changes via event polling

---

## 3. Architecture Options

### Option A: Per-Chain Analytics Modules

```
analytics/
  evm/
    whale_tracker.rs       // ERC-20 Transfer events > threshold
    dex_monitor.rs         // UniV2/V3 swap events
    mempool_analyzer.rs    // pending tx monitoring
    gas_tracker.rs         // fee history analysis
  bitcoin/
    utxo_analyzer.rs       // UTXO set analysis
    miner_tracker.rs       // coinbase tx parsing
    mempool_monitor.rs     // fee rate histogram
    exchange_flow.rs       // labeled address monitoring
  solana/
    program_monitor.rs     // Geyser subscription
    swap_decoder.rs        // Raydium/Jupiter decode
  cosmos/
    ibc_tracker.rs         // IBC packet events
    governance_monitor.rs  // proposal/vote events
  sui/
    deepbook_monitor.rs    // order book events
  ton/
    jetton_tracker.rs      // op-code message parsing
  aptos/
    module_event_listener.rs
```

**Pros**: Clean separation, easy to add per-chain features, isolated failure domains.
**Cons**: Cross-chain analytics (bridge flows, capital rotation) requires coordination layer on top.

---

### Option B: Cross-Chain Analytics with Chain-Specific Adapters

```
analytics/
  core/
    whale_tracker.rs       // interface: WhaleMoveEvent { chain, from, to, asset, usd_value }
    dex_monitor.rs         // interface: SwapEvent { chain, dex, pair, side, size_usd }
    flow_analyzer.rs       // interface: FlowEvent { from_chain, to_chain, asset, amount }
  adapters/
    evm_adapter.rs         // EVM-specific event decoding → common types
    bitcoin_adapter.rs     // Bitcoin UTXO → common types
    solana_adapter.rs      // Solana program logs → common types
    cosmos_adapter.rs
    sui_adapter.rs
    ton_adapter.rs
    aptos_adapter.rs
```

**Pros**: Enables cross-chain metrics (e.g., "BTC leaving Ethereum bridges to native Bitcoin" as capital rotation signal). Unified alert system. Unified UI.
**Cons**: Designing a lowest-common-denominator event type loses chain-specific detail.

---

### Option C: Event-Driven Pipeline

```
monitors/
  event_stream.rs          // per-chain websocket/polling → raw events
  event_classifier.rs      // raw event → typed enum (Swap, Transfer, LiquidityChange, etc.)
  event_enricher.rs        // add: usd_value, entity_label, whale_tier
  event_aggregator.rs      // build time-series metrics from event stream
  alert_engine.rs          // condition evaluation → trigger alerts
  signal_store.rs          // persist aggregated metrics for backtesting
```

Event type enum:
```rust
enum OnChainEvent {
    Swap { chain, dex, pair, amount_in, amount_out, price_impact },
    WhaleMove { chain, from, to, asset, usd_value, from_label, to_label },
    LiquidityChange { chain, protocol, pool, direction, usd_value },
    ExchangeFlow { chain, exchange, direction, asset, amount },
    LargeUnstake { chain, protocol, validator, amount },
    BridgeTransfer { from_chain, to_chain, asset, amount },
    NewTokenLaunch { chain, mint, initial_liquidity_usd },
    GovernanceEvent { chain, protocol, event_type, proposal_id },
    MempoolAlert { chain, tx_hash, pending_usd_value },
}
```

**Pros**: This is what production analytics platforms actually build. Single pipeline handles all chains. Alert engine works on any event type. Historical replay possible. Clean separation of concerns.
**Cons**: Higher upfront complexity. Event normalization is lossy for chain-specific detail.

---

### Verdict for NEMO

**Recommended: Hybrid of B and C** — Event-driven pipeline (C) with chain-specific adapters (B).

The architecture should be:

```
providers/
  evm/         // RPC connections, log subscriptions
  bitcoin/     // RPC connections, block/mempool polling
  solana/      // Geyser websocket subscriptions
  cosmos/      // Tendermint websocket subscriptions
  sui/         // Sui event subscriptions
  ton/         // TON API polling
  aptos/       // Aptos event polling

analytics/
  decoders/    // Chain-specific raw data → OnChainEvent
    evm_decoder.rs
    bitcoin_decoder.rs
    solana_decoder.rs
    ...
  pipeline/
    event_classifier.rs    // classify event type
    event_enricher.rs      // add USD value, labels
    alert_engine.rs        // condition → alert
    aggregator.rs          // metrics accumulation
  metrics/                 // derived metrics per chain
    exchange_flow.rs
    whale_tracker.rs
    liquidity_monitor.rs
    mempool_analyzer.rs
```

This gives you:
- Clean chain boundary (providers are isolated)
- Unified event stream (all chains feed same pipeline)
- Chain-specific decoding detail preserved
- Cross-chain analytics possible at the pipeline level

---

## 4. Actionable Trading Signals

Ranked by demonstrated predictive value:

### Tier 1: High Signal, Widely Proven

| Signal | What It Indicates | Detection Method |
|--------|------------------|-----------------|
| Exchange Inflows (BTC) | Selling pressure incoming | Monitor labeled exchange addresses |
| Exchange Outflows (BTC) | Accumulation / withdrawal to self-custody | Same |
| Whale Accumulation Cluster | Coordinated buying by large players | Track labeled wallets, cluster by timing |
| Smart Money Entry | Early position in token before pump | Track wallets with historical outperformance |
| Large DEX Swap (>$1M) | Immediate price impact + signal | UniV3 Swap events, filter by USD value |
| Miner Selling Pressure | Near-term BTC supply increase | Coinbase tx outflows to exchanges |

### Tier 2: Useful, Moderate Signal Quality

| Signal | What It Indicates | Detection Method |
|--------|------------------|-----------------|
| CEX Funding Rate Divergence | Basis trade opportunity | Not on-chain; from exchange APIs |
| Large Unstaking Events | Upcoming sell pressure (delay = unbonding) | Cosmos/ETH staking events |
| Bridge Inflows to Chain | Capital rotation signal | Bridge contract events |
| DEX Liquidity Removal (large) | LP exiting = preparing for volatility | UniV3 Burn events |
| New Token Launch + Whale Wallet Entry | Pump candidate | Pump.fun + smart wallet entry |
| Governance: Critical Vote Approaching | Potential volatility around snapshot/execution | Governance events |

### Tier 3: Situational / Context-Dependent

| Signal | What It Indicates | Detection Method |
|--------|------------------|-----------------|
| Gas Price Spike | Network congestion, high activity | eth_feeHistory |
| Mempool Spike (BTC) | Upcoming fee increase / high demand | getmempoolinfo |
| MEV Sandwich Spike | High retail activity (bots follow) | Pattern detect in blocks |
| NFT Mint Waves | Attention/liquidity drain from DeFi | ERC-721 Transfer events |
| IBC Large Transfer | Cosmos ecosystem capital movement | IBC packet events |
| SOPR < 1 (Bitcoin) | Capitulation / potential bottom | UTXO cost-basis database |

### What Does NOT Work Well for Real-Time Trading

- NUPL / Realized Cap: requires full UTXO history, updates too slowly
- Hodler supply: lags reality by weeks (21-day EMA basis)
- On-chain transaction count: noisy, inflated by bots
- Single large tx alerts without entity context: 70%+ false positive rate (internal exchange transfers)

---

## 5. Implementation Priority

### Phase 1: Maximum Signal / Minimum Infrastructure (RPC only, no indexer)

All of these work with a standard node RPC connection:

**EVM (Week 1-2)**
- Subscribe `eth_subscribe("logs")` for ERC-20 Transfer + Uniswap V2/V3 Swap events
- Filter by USD value threshold (requires price oracle integration)
- Subscribe `eth_subscribe("newPendingTransactions")` for mempool monitoring
- `eth_feeHistory` polling for gas trends
- Zero external dependencies beyond RPC node

**Bitcoin (Week 2-3)**
- `getrawmempool(true)` polling (5s interval) for large pending txs
- Block subscription via ZMQ or polling `getblockcount` + `getblock`
- Coinbase tx detection in each new block (miner identification)
- Exchange flow monitoring using curated address list (start with 50-100 known addresses)
- UTXO value histogram from `gettxoutsetinfo` (no per-UTXO detail, but macro supply view)

**Solana (Week 3-4)**
- Geyser WebSocket subscription to Raydium + Jupiter program addresses
- Log-based swap detection (no instruction decoder needed for basic signal)
- New pool creation detection for Raydium

**Estimated signal coverage**: ~60% of actionable tier-1 and tier-2 signals above.

---

### Phase 2: Enriched Signals (requires address database, not full indexer)

**EVM Entity Labels**
- Integrate Etherscan labels API (free tier: 5K req/day)
- Or use Arkham API for entity data
- Dramatically reduces false positives on whale alerts

**Bitcoin Exchange Flow Database**
- Scrape/maintain a list of known exchange addresses (start with Glassnode's published lists)
- Run against each new block to track net flow
- ~200-500 labeled addresses covers 80% of exchange flows

**Cosmos/Sui/TON/Aptos (Week 5-8)**
- Add remaining chain decoders
- IBC transfer monitoring for Cosmos
- DeepBook event monitoring for Sui
- Jetton transfer + op-code monitoring for TON
- Module event polling for Aptos

**Estimated signal coverage**: ~80% of actionable signals.

---

### Phase 3: Requires Indexer / Historical Database

These cannot be built with RPC alone:

| Feature | Why Indexer Required | What to Use |
|---------|---------------------|-------------|
| SOPR / NUPL | Full UTXO history + creation price | Run Bitcoin Core + custom index or use Glassnode API |
| Wallet PnL history | All historical txs + price at each time | PostgreSQL + block indexer |
| On-chain volume aggregation | Sum across all pools/pairs historically | Dune API or The Graph |
| Long-term holder distribution | UTXO age bands | Custom UTXO index |
| Protocol revenue (all time) | Sum of fee collection events | The Graph subgraph |
| MEV pattern detection (historical) | Query N blocks back for patterns | Flashbots MEV-Inspect or custom indexer |

**Recommendation**: For Phase 3, use external APIs (Glassnode, Dune, The Graph) rather than building your own indexer. Building a production-quality indexer is a multi-month infrastructure project.

---

### Practical Implementation Order (Single Developer, 2-3 months)

```
Week 1-2:  EVM log subscription → large Transfer + Swap detection
Week 2-3:  USD value enrichment via Uniswap V3 TWAP oracle
Week 3-4:  Bitcoin block polling + mempool monitoring + miner tracking
Week 4-5:  Solana Geyser integration + Raydium/Jupiter decode
Week 5-6:  Alert engine: condition builder + notification dispatch
Week 6-7:  Exchange flow monitoring (EVM + BTC with address lists)
Week 7-8:  Cosmos IBC + governance monitoring
Week 8-9:  Sui DeepBook + Aptos module events
Week 9-10: TON jetton + DEX monitoring (most complex due to message parsing)
Week 10+:  Cross-chain aggregation + Glassnode/Dune API integration for macro metrics
```

---

## 6. Recommended Architecture for NEMO

### Data Flow

```
Chain RPC/WS
    │
    ▼
┌─────────────────────────────────────────────────────┐
│  Chain Adapters (per chain, isolated failure)        │
│  evm_adapter | bitcoin_adapter | solana_adapter ...  │
└─────────────────────────────────────────────────────┘
    │  raw ChainEvent stream (channel)
    ▼
┌─────────────────────────────────────────────────────┐
│  Decoder Layer                                       │
│  classifies raw events → typed OnChainEvent enum    │
│  + USD value enrichment (price oracle lookup)        │
│  + entity label lookup (address → label map)        │
└─────────────────────────────────────────────────────┘
    │  OnChainEvent stream
    ▼
┌─────────────────────────────────────────────────────┐
│  Analytics Pipeline (fanout)                         │
│  ├── Alert Engine: condition eval → AlertFired       │
│  ├── Aggregator: rolling metrics (5m/1h/1d windows)  │
│  └── Signal Store: persist to time-series DB         │
└─────────────────────────────────────────────────────┘
    │  Signals + Alerts
    ▼
┌─────────────────────────────────────────────────────┐
│  Terminal UI + Agent API                             │
│  chart overlays | alert panel | on-chain dashboard   │
└─────────────────────────────────────────────────────┘
```

### Core Rust Types

```rust
// Central event type — all chains produce these
pub enum OnChainEvent {
    LargeTransfer {
        chain: ChainId,
        from: Address,
        to: Address,
        asset: AssetId,
        usd_value: f64,
        from_label: Option<String>,
        to_label: Option<String>,
    },
    DexSwap {
        chain: ChainId,
        protocol: DexProtocol,
        pool: Address,
        trader: Address,
        input_asset: AssetId,
        output_asset: AssetId,
        input_amount: f64,
        output_amount: f64,
        usd_value: f64,
        price_impact_bps: i32,
    },
    LiquidityChange {
        chain: ChainId,
        protocol: DexProtocol,
        pool: Address,
        direction: LiquidityDirection, // Add | Remove
        usd_value: f64,
        actor: Address,
        actor_label: Option<String>,
    },
    ExchangeFlow {
        chain: ChainId,
        exchange: String,
        direction: FlowDirection, // Inflow | Outflow
        asset: AssetId,
        amount: f64,
        usd_value: f64,
    },
    MempoolAlert {
        chain: ChainId,
        tx_hash: TxHash,
        usd_value: f64,
        note: String,
    },
    BridgeTransfer {
        from_chain: ChainId,
        to_chain: ChainId,
        bridge: String,
        asset: AssetId,
        usd_value: f64,
    },
    NewTokenLaunch {
        chain: ChainId,
        mint: Address,
        initial_liquidity_usd: f64,
        creator: Address,
    },
    GovernanceEvent {
        chain: ChainId,
        protocol: String,
        event_type: GovEventType,
        proposal_id: u64,
        description: String,
    },
    ValidatorAlert {
        chain: ChainId,
        validator: String,
        event_type: ValidatorEventType, // Slash | LargeUnstake | Jailed
        stake_affected: f64,
    },
}

// Alert conditions (composable)
pub struct AlertCondition {
    pub event_filter: EventFilter,      // which event types to watch
    pub chain_filter: Vec<ChainId>,     // which chains
    pub threshold: Option<Threshold>,  // e.g., usd_value > 1_000_000
    pub entity_filter: Option<Vec<String>>, // only specific labels/entities
    pub cooldown_secs: u64,             // min time between same alert
}
```

### What Needs An External Service vs. Internal

| Capability | Internal (RPC only) | External Service Needed |
|------------|--------------------|-----------------------|
| Large transfer detection | Yes (EVM logs, BTC blocks) | No |
| Uniswap swap monitoring | Yes (event subscriptions) | No |
| USD value enrichment | Yes (TWAP oracle) | No |
| Entity/label lookup | Partial (maintained list) | Arkham/Nansen API for full coverage |
| Bitcoin SOPR/NUPL | No (needs UTXO cost-basis DB) | Glassnode API |
| Historical DEX volumes | No (needs indexer) | Dune API or The Graph |
| Full mempool coverage (EVM) | Partial (single node) | Blocknative / Bloxroute for full |
| Solana real-time (low latency) | Partial (public RPC) | Helius Geyser Enhanced WS |
| Cross-chain portfolio tracking | No (needs indexer per chain) | DeBank API |

---

## Sources

- [Nansen - Smart Money Tracking Guide 2025](https://www.nansen.ai/post/how-to-monitor-crypto-wallet-activity-track-smart-money)
- [Nansen - Whale Watching Tools](https://www.nansen.ai/post/whale-watching-top-tools-for-monitoring-large-crypto-wallets)
- [Arkham Intelligence - Tagging System Explained](https://info.arkm.com/research/a-guide-to-arkham-intels-industry-leading-tagging-system)
- [Arkham - On-Chain Analysis Guide](https://info.arkm.com/research/on-chain-analysis-guide)
- [Glassnode - Exchange Metrics Methodology](https://insights.glassnode.com/exchange-metrics/)
- [Glassnode Documentation](https://docs.glassnode.com/)
- [Uniswap V3 Pool Events Reference](https://docs.uniswap.org/contracts/v3/reference/core/interfaces/pool/IUniswapV3PoolEvents)
- [Helius - Solana Geyser Enhanced Websockets](https://www.helius.dev/blog/how-to-monitor-solana-transactions-using-geyser-enhanced-websockets)
- [Chainstack - Solana Geyser Raydium Real-Time Analytics](https://chainstack.com/solana-geyser-raydium-bonk/)
- [VeloDB - Dual-Pipeline On-Chain Analytics Architecture](https://www.velodb.io/blog/building-real-time-on-chain-analytics-a-dual-pipeline-architecture)
- [Sui Documentation - Using Events](https://docs.sui.io/guides/developer/sui-101/using-events)
- [Sui Documentation - DeepBookV3](https://docs.sui.io/standards/deepbook)
- [Aptos Documentation - Events](https://aptos.dev/network/blockchain/events)
- [Aptos Documentation - Resources](https://aptos.dev/network/blockchain/resources)
- [Cosmos Validator Watcher (kilnfi)](https://github.com/kilnfi/cosmos-validator-watcher)
- [TON - STON.fi and DeDust on TradingView](https://ton.org/en/stonfi-dedust-tradingview-integration)
- [Dune Analytics - DEX Metrics](https://dune.com/hagaetc/dex-metrics)
- [Dune - Whale Tracking with SQL](https://cow.fi/learn/how-to-track-whale-movements-with-dune)
- [Bitquery - UTXO Comprehensive Guide](https://bitquery.io/blog/utxo-comprehensive-guide)
- [Ethereum Event Logs Deep Dive](https://medium.com/linum-labs/everything-you-ever-wanted-to-know-about-events-and-logs-on-ethereum-fec84ea7d0a5)
- [Best Blockchain Indexers 2026](https://blog.ormilabs.com/best-blockchain-indexers-in-2025-real-time-web3-data-and-subgraph-platforms-compared/)
- [Chainalysis Lightning Network Support](https://www.chainalysis.com/blog/lightning-network-support/)
- [Glassnode Insights - The Week On-Chain](https://insights.glassnode.com/the-week-onchain-week-40-2025/)
