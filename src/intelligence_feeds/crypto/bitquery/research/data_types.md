# Bitquery - Data Types Catalog

**Provider Type**: Blockchain/On-chain Data Only (NO trading, NO traditional markets)

---

## Standard Market Data

**NOT APPLICABLE** - Bitquery provides blockchain data, not traditional market data.

For crypto price data, see On-chain Data section below.

---

## Historical Data

- [x] **Historical blockchain data** (depth: From blockchain genesis)
  - Ethereum: From July 2015 (genesis block)
  - Bitcoin: From January 2009 (genesis block)
  - Other chains: From their respective launch dates
- [x] **Block-level data** (available: Yes, all blocks since genesis)
- [x] **Transaction data** (available: Yes, all transactions)
- [x] **Minute-level aggregations** (available: Yes, via time-based grouping)
- [x] **Daily aggregations** (available: Yes, via GraphQL grouping)
- [ ] **Tick data** (N/A - blockchain data is block/transaction-based)
- [x] **Adjusted prices** (N/A - but DEX prices available)

**Data Depth by Blockchain**:
- **EVM chains** (Ethereum, BSC, Polygon, etc.): Full history from genesis
- **Bitcoin/UTXO chains**: Full history from genesis
- **Solana**: Full history from mainnet launch (March 2020)
- **Other chains**: Complete data from launch date

---

## Derivatives Data (Crypto/Futures)

**NOT APPLICABLE** - Bitquery doesn't provide derivatives exchange data.

Bitquery focuses on on-chain data (DEX trades, transfers), not centralized exchange derivatives.

---

## Options Data

**NOT APPLICABLE** - No options data available.

---

## Fundamental Data (Stocks)

**NOT APPLICABLE** - Bitquery is blockchain-focused, not stock markets.

---

## On-chain Data (Crypto) - **PRIMARY FOCUS**

Bitquery specializes in blockchain/on-chain data across 40+ chains.

### Blockchain Infrastructure Data

- [x] **Block Data**
  - Block number, hash, timestamp
  - Miner/validator address (coinbase)
  - Gas limit, gas used, base fee (EIP-1559)
  - Block size, difficulty, uncle count
  - Transaction count per block
  - Parent block hash, uncle blocks

- [x] **Transaction Data**
  - Transaction hash, index
  - From/to addresses
  - Value (native token amount)
  - Gas limit, gas price, gas used
  - Transaction type (legacy, EIP-1559)
  - Nonce, input data
  - Receipt status (success/failure)
  - Effective gas price
  - Transaction cost

- [x] **Mempool Data (Pending Transactions)**
  - Real-time pending transactions
  - Gas prices (current mempool)
  - Priority fees (EIP-1559)
  - Transaction signers
  - MEV opportunity detection (via analysis)

### Token & Transfer Data

- [x] **Token Transfers**
  - ERC-20 transfers (Ethereum)
  - BEP-20 transfers (BSC)
  - SPL transfers (Solana)
  - All token standards (ERC-721, ERC-1155, etc.)
  - Transfer amount, sender, receiver
  - Token symbol, name, decimals, contract address
  - Transfer type (transfer, mint, burn)
  - Native token transfers (ETH, BNB, SOL, etc.)

- [x] **Token Metadata**
  - Token name, symbol
  - Total supply
  - Contract address
  - Decimals
  - Token type (fungible/non-fungible)
  - Creation date/block

- [x] **Token Holders**
  - Current holder addresses
  - Balance per holder
  - Holder count (total unique holders)
  - Top holders by balance
  - Holder distribution metrics
  - Historical holder snapshots

### DEX Trading Data

- [x] **DEX Trades**
  - **Supported DEXs** (40+ protocols):
    - **Ethereum**: Uniswap (V2/V3/V4), SushiSwap, Balancer, Curve, 1inch, 0x, Kyber, IDEX, etc.
    - **BSC**: PancakeSwap, BakerySwap, BiSwap, etc.
    - **Polygon**: QuickSwap, SushiSwap, etc.
    - **Arbitrum**: Uniswap, SushiSwap, Camelot, etc.
    - **Solana**: Raydium, Orca, Serum, Jupiter, etc.
    - **Base**: Uniswap, Aerodrome, etc.
  - **Trade Data**:
    - Buy/sell amounts
    - Buy/sell token symbols and addresses
    - Price (in quote currency)
    - Price in USD (calculated)
    - Buyer/seller addresses
    - DEX protocol name and version
    - Pair/pool address
    - Trade index (for multi-hop swaps)
    - Transaction hash
    - Block timestamp
  - **Multi-hop Swaps**: Tracked and identified
  - **Liquidity Pool Data**:
    - Pool reserves
    - Liquidity changes
    - Pool creation events
    - Slippage calculations
  - **OHLCV Data**:
    - Open, High, Low, Close, Volume
    - Customizable intervals (1s, 1m, 5m, 15m, 1h, 4h, 1d, etc.)
    - Real-time and historical

- [x] **DEX Volume Analytics**
  - Total volume by DEX
  - Volume by token pair
  - Volume by time period
  - 24h/7d/30d volume aggregations

- [x] **Liquidity Analytics**
  - Pool liquidity (TVL)
  - Liquidity provider positions
  - LP token transfers
  - Impermanent loss tracking (via calculation)

### NFT Data

- [x] **NFT Trades**
  - **Supported Marketplaces**:
    - **Ethereum**: OpenSea, Blur, LooksRare, Rarible, X2Y2, Foundation, SuperRare
    - **Solana**: Magic Eden (Metaplex protocol)
    - **Polygon**: OpenSea (Polygon)
    - **BSC**: NFT marketplaces on BSC
  - **Trade Data**:
    - NFT token ID
    - Collection (contract address)
    - Sale price (in ETH/SOL/WETH/etc.)
    - Sale price in USD
    - Buyer/seller addresses
    - Marketplace protocol (OpenSea Seaport, Blur, etc.)
    - Transaction hash
    - Order ID
    - Token URI
    - Royalty amounts

- [x] **NFT Transfers**
  - ERC-721 transfers
  - ERC-1155 transfers (multi-token standard)
  - Solana NFT transfers (Metaplex)
  - Sender/receiver addresses
  - Token ID
  - Quantity (for ERC-1155)
  - Transfer type (mint, transfer, burn)

- [x] **NFT Ownership**
  - Current owner by token ID
  - Ownership history
  - Collection holders (all owners of a collection)
  - Holder count per collection
  - Balance per holder (for ERC-1155)

- [x] **NFT Metadata**
  - Token URI
  - Collection name
  - Contract address
  - Token standard (ERC-721, ERC-1155)
  - Creation date/block

- [x] **NFT Analytics**
  - Floor price (via DEXTrades filtering)
  - Total sales volume
  - Unique buyers/sellers
  - Top collections by volume
  - Trending collections

### Smart Contract Data

- [x] **Smart Contract Events (Logs)**
  - Event signature (hash)
  - Event name (decoded)
  - Contract address
  - Event arguments (indexed and non-indexed)
  - Argument names, types, values
  - Transaction hash
  - Block timestamp
  - Log index

- [x] **Smart Contract Calls**
  - Function signature (hash)
  - Function name (decoded)
  - Caller address (from)
  - Contract address (to)
  - Function arguments
  - ETH value sent
  - Gas used
  - Call success/failure status
  - Return data
  - Internal calls (call tree)

- [x] **Smart Contract Traces**
  - Full call stack
  - Internal transactions
  - Contract creation
  - Self-destructs
  - Delegate calls

- [x] **Contract Deployment**
  - Deployer address
  - Contract address
  - Deployment transaction
  - Deployment block/timestamp
  - Contract bytecode (if needed, via full node)

### Balance & Wallet Data

- [x] **Balance Updates**
  - Real-time balance changes
  - Historical balances at any block
  - Address balance snapshots
  - Token balances (all ERC-20/BEP-20/SPL tokens)
  - Native token balances (ETH, BNB, SOL, etc.)
  - NFT balances (ERC-721, ERC-1155)

- [x] **Wallet Activity**
  - All transactions sent/received
  - All token transfers
  - All DEX trades
  - All NFT trades
  - All smart contract interactions
  - First/last activity timestamp

- [x] **Portfolio Tracking**
  - Current token holdings
  - Historical portfolio value (calculable)
  - Token acquisition cost (via transfer tracking)
  - Realized/unrealized gains (calculable)

### Staking & Rewards

- [x] **Validator/Miner Rewards**
  - Block rewards
  - Transaction fees collected
  - MEV rewards (if traceable)
  - Uncle rewards (Ethereum pre-merge)

- [x] **Staking Rewards (on-chain)**
  - Staking deposits
  - Staking withdrawals
  - Reward distributions (via balance updates)
  - Validator performance (via block production)

### Gas & Fees

- [x] **Gas Data**
  - Gas prices (current and historical)
  - Gas used per transaction
  - Gas limit per transaction
  - Base fee (EIP-1559)
  - Priority fee (EIP-1559)
  - Max fee per gas
  - Effective gas price
  - Total gas spent per address

- [x] **Transaction Fees**
  - Fee per transaction (in ETH/BNB/SOL/etc.)
  - Fee in USD
  - Fee statistics (avg, median, max, min)
  - Fee trends over time

### Cross-chain Data

- [x] **Bridge Transactions**
  - Lock events (source chain)
  - Mint events (destination chain)
  - Bridge contract interactions
  - Cross-chain transfer amounts
  - Supported bridges (via event filtering)

### Aggregated Analytics

- [x] **Token Analytics**
  - Total supply
  - Circulating supply (calculable)
  - Market cap (price × supply)
  - Holder count
  - Transfer count
  - Unique senders/receivers
  - Top holders
  - Gini coefficient (distribution)
  - Nakamoto coefficient (decentralization)

- [x] **DEX Analytics**
  - Trading volume (24h, 7d, 30d, all-time)
  - Unique traders
  - Trade count
  - Average trade size
  - Largest trades
  - Most traded pairs

- [x] **Network Analytics**
  - Active addresses
  - Daily transactions
  - Gas usage trends
  - Block production rate
  - Average block time
  - Network congestion metrics

---

## Macro/Economic Data

**NOT APPLICABLE** - Bitquery is blockchain-only, no macroeconomic data.

---

## Forex Specific

**NOT APPLICABLE** - No forex data.

---

## Metadata & Reference

- [x] **Blockchain Networks**
  - Network name (Ethereum, BSC, Polygon, etc.)
  - Network ID/Chain ID
  - Native token (ETH, BNB, MATIC, etc.)
  - Consensus mechanism (PoW, PoS, etc.)
  - Block time (average)

- [x] **Token Lists**
  - All tokens traded on DEXs
  - Token symbols, names, addresses
  - Token standards (ERC-20, ERC-721, etc.)
  - Token creation dates

- [x] **DEX Protocol Lists**
  - All supported DEX protocols
  - Protocol versions (Uniswap V2 vs V3)
  - Protocol families

- [x] **Smart Contract Lists**
  - All deployed contracts
  - Contract types (token, DEX, NFT marketplace, etc.)
  - Contract deployers

- [x] **Trading Calendars**
  - Not applicable (blockchains operate 24/7/365)

- [x] **Timezone Info**
  - All timestamps in UTC
  - Block timestamps (miner-reported)

---

## News & Sentiment

**NOT APPLICABLE** - Bitquery provides raw blockchain data, not news or sentiment.

For sentiment analysis, would need to integrate with separate service.

---

## Unique/Custom Data

### What makes Bitquery special:

1. **Multi-chain Coverage (40+ blockchains)**
   - Unified GraphQL API across all chains
   - Consistent schema for EVM chains
   - Chain-specific schemas for non-EVM (Solana, Bitcoin, etc.)

2. **Real-time Streaming**
   - WebSocket subscriptions for live data
   - Sub-second latency for realtime dataset
   - Mempool monitoring (pending transactions)

3. **Complete Historical Data**
   - Full blockchain history from genesis
   - No data gaps or missing blocks
   - Archive dataset for deep historical queries

4. **DEX-specific Analytics**
   - Multi-hop swap detection
   - Liquidity pool tracking
   - Price impact calculations
   - Slippage monitoring

5. **GraphQL Flexibility**
   - Query exactly what you need (no over-fetching)
   - Complex filtering and aggregations
   - Combine multiple data types in single query
   - Dimensions and metrics for analytics

6. **On-chain Social Graph**
   - Wallet interaction networks (via transactions)
   - Token holder overlap analysis
   - NFT collection holder communities
   - Smart contract usage patterns

7. **MEV Detection**
   - Identify sandwich attacks (via transaction ordering)
   - Front-running detection
   - Arbitrage opportunity tracking
   - Miner extractable value (MEV) analysis

8. **Token Holder Analytics**
   - Distribution metrics (Gini, Nakamoto coefficients)
   - Whale tracking (top holders)
   - Holder accumulation/distribution patterns
   - Unique holder growth over time

9. **NFT Floor Price Tracking**
   - Real-time floor price via DEXTrades
   - Historical floor price trends
   - Marketplace comparison (OpenSea vs Blur prices)

10. **Smart Contract Event Monitoring**
    - Decode any event with ABI
    - Filter by specific event types
    - Track contract interactions
    - Event argument extraction

11. **Cross-chain Aggregation**
    - Query multiple chains in single request
    - Compare token prices across chains
    - Track bridged assets
    - Multi-chain portfolio tracking

12. **Custom Data Exports**
    - SQL access to blockchain data
    - Kafka streaming for real-time pipelines
    - Cloud data warehouse integration (Snowflake, BigQuery)
    - Protocol Buffers for low-latency streaming

---

## Data NOT Available

### Traditional Finance
- [ ] Stock prices, fundamentals, earnings
- [ ] Forex rates (except crypto stablecoin pairs)
- [ ] Commodities (except tokenized versions)
- [ ] Bonds, treasuries
- [ ] Economic indicators (GDP, CPI, etc.)

### Centralized Exchange Data
- [ ] CEX orderbooks (Binance, Coinbase, Kraken)
- [ ] CEX trades (only on-chain/DEX trades)
- [ ] CEX futures, perpetuals, options
- [ ] CEX liquidations (only DEX liquidations if on-chain)
- [ ] Funding rates (CEX-specific)

### Off-chain Data
- [ ] Social media sentiment
- [ ] News articles
- [ ] Whale alerts (unless on-chain)
- [ ] Exchange inflows/outflows (unless on-chain tracked)

### Privacy Chains
- Limited data for privacy-focused chains (Zcash shielded transactions, Monero)
- Only public transaction data available

---

## Supported Blockchains (Full List)

### EVM-Compatible Chains
1. Ethereum (ETH)
2. Binance Smart Chain (BSC)
3. Polygon (MATIC)
4. Arbitrum One
5. Arbitrum Nova
6. Optimism
7. Base
8. Avalanche C-Chain
9. Fantom
10. Cronos
11. Celo
12. Moonbeam
13. Moonriver
14. Klaytn
15. Gnosis Chain (xDai)
16. zkSync Era
17. Polygon zkEVM
18. Linea
19. Scroll
20. Mantle

### Non-EVM Chains
21. Solana
22. Bitcoin
23. Bitcoin Cash
24. Litecoin
25. Bitcoin SV
26. Dogecoin
27. Zcash
28. Dash
29. Cardano
30. Ripple (XRP)
31. Stellar (XLM)
32. Algorand
33. Cosmos
34. Tron (TRX)
35. EOS
36. Flow
37. Hedera (HBAR)
38. Filecoin
39. TON (The Open Network)
40. Aptos

**Total**: 40+ chains (growing)

---

## Data Granularity

### Time-based Granularity
- **Block-level**: Native granularity (every block)
- **Second-level**: Achievable via timestamp grouping
- **Minute-level**: Via GraphQL time grouping
- **Hourly**: Via time aggregation
- **Daily**: Via time aggregation
- **Custom intervals**: Any interval via `Block.Time` grouping

### Transaction-level
- Individual transactions (finest granularity)
- Mempool (pre-block, pending transactions)

### Address-level
- Per-address data (all transactions, transfers, balances)

---

## Data Freshness

### Real-time Data (dataset: realtime)
- **Latency**: Sub-second (typically <1s after block inclusion)
- **Mempool**: Real-time pending transactions
- **Subscriptions**: Live streaming via WebSocket

### Archive Data (dataset: archive)
- **Latency**: Near real-time (~1-5 seconds behind chain tip)
- **Indexing delay**: Minimal (blocks indexed as they arrive)
- **Backfilling**: Complete historical data available immediately

---

## Use Cases for Bitquery Data

1. **DeFi Analytics Platforms** - Track DEX volumes, liquidity, yields
2. **NFT Marketplaces** - Real-time floor prices, sales tracking
3. **Portfolio Trackers** - Wallet balance tracking across chains
4. **Trading Bots** - DEX price feeds, arbitrage detection
5. **Blockchain Explorers** - Transaction search, address lookup
6. **Token Analytics** - Holder distribution, supply tracking
7. **MEV Bots** - Mempool monitoring, sandwich attack detection
8. **Research & Analytics** - On-chain behavior analysis
9. **Compliance/AML** - Transaction tracing, address monitoring
10. **Smart Contract Monitoring** - Event tracking, call analysis
