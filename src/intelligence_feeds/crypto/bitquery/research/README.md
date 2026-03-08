# Bitquery API Research - Complete Documentation

**Research Date**: 2026-01-26
**Provider**: Bitquery
**Category**: data_feeds (Blockchain/On-chain Data Provider)
**Type**: Data Only - NO Trading Functionality
**Total Documentation**: 4,657 lines across 8 comprehensive files

---

## Overview

Bitquery is a **blockchain data platform** providing GraphQL APIs for querying on-chain data across 40+ blockchains. It is **NOT a crypto exchange** - it only provides data, not trading capabilities.

**Key Characteristics**:
- **GraphQL-first API** (not REST)
- **Multi-chain support** (40+ blockchains: Ethereum, BSC, Solana, Bitcoin, etc.)
- **Real-time WebSocket streaming** via GraphQL subscriptions
- **Complete historical data** from blockchain genesis
- **DEX trades, NFT sales, token transfers, smart contracts, and more**

---

## Research Files

### 1. [api_overview.md](./api_overview.md) (189 lines)
**High-level provider information**

- Provider details and website
- API types (GraphQL, WebSocket, gRPC)
- Base URLs and endpoints
- Documentation quality assessment
- Licensing, terms, support channels
- Use cases and key differentiators

**Key Findings**:
- V2 GraphQL endpoint: `https://streaming.bitquery.io/graphql`
- WebSocket: `wss://streaming.bitquery.io/graphql`
- Excellent documentation with interactive IDE
- OAuth 2.0 authentication required for all requests
- Free tier available (10K points/month)

---

### 2. [endpoints_full.md](./endpoints_full.md) (639 lines)
**Complete API surface documentation**

**IMPORTANT**: Bitquery uses GraphQL "cubes" (data tables), not REST endpoints.

**All Cubes Documented**:
- **Blocks** - Block-level data
- **Transactions** - Transaction data
- **Transfers** - Token transfers (ERC-20, ERC-721, ERC-1155, SPL, etc.)
- **DEXTrades** - DEX trading data (40+ protocols)
- **BalanceUpdates** - Historical balance changes
- **Events** - Smart contract events/logs
- **Calls** - Smart contract function calls
- **MempoolTransactions** - Pending transactions (realtime)
- **NFTTrades** - NFT marketplace trades
- **Solana-specific cubes** - Solana blockchain data
- **Bitcoin-specific cubes** - UTXO data (Inputs, Outputs)

**Query Parameters**:
- Filtering (`where`, operators: `is`, `not`, `in`, `gt`, `lt`, `since`, `till`)
- Pagination (`limit: {count, offset}`)
- Sorting (`orderBy`)
- Aggregations (metrics: `count`, `sum`, `average`, `max`, `min`)

**Supported Networks**: 40+ (eth, bsc, polygon, arbitrum, solana, bitcoin, etc.)

---

### 3. [websocket_full.md](./websocket_full.md) (770 lines)
**WebSocket/subscription documentation**

**Availability**: Yes - via GraphQL Subscriptions

**Connection Details**:
- **URL**: `wss://streaming.bitquery.io/graphql?token=YOUR_TOKEN`
- **Protocols**: `graphql-transport-ws` (modern), `graphql-ws` (legacy)
- **Authentication**: OAuth token in URL parameter
- **Keepalive**: Server sends `pong` (modern) or `ka` (legacy) messages

**Subscription Types**:
- All queries can become subscriptions (replace `query` with `subscription`)
- Use `dataset: realtime` for live data
- Examples: Blocks, Transactions, DEX Trades, Transfers, Mempool, Events

**Connection Process**:
1. Connect WebSocket with protocol
2. Send `connection_init` message
3. Await `connection_ack`
4. Send `subscribe` message with GraphQL subscription
5. Receive `next` messages with data

**Limits**:
- Free tier: 2 simultaneous streams
- Commercial: Unlimited streams
- Billing: 40 points/minute per stream

**Critical Notes**:
- Cannot send 'close' messages (must close WebSocket directly)
- Must implement reconnect logic (if no keepalive for 10s)
- Each cube in subscription counts as separate stream

---

### 4. [authentication.md](./authentication_md) (514 lines)
**Authentication mechanisms**

**Method**: OAuth 2.0 (required for all requests)

**Key Format**:
- **Type**: Bearer token
- **Format**: `ory_at_...` (OAuth access token)
- **Length**: ~100+ characters

**How to Authenticate**:

**HTTP (GraphQL POST)**:
```http
Authorization: Bearer ory_at_YOUR_TOKEN
```

**WebSocket**:
```
wss://streaming.bitquery.io/graphql?token=ory_at_YOUR_TOKEN
```

**Token Acquisition**:
1. Sign up at https://account.bitquery.io/auth/signup
2. Generate OAuth token in IDE or via OAuth client credentials flow
3. Use token in Authorization header or URL parameter

**Token Lifecycle**:
- Access tokens expire (OAuth 2.0 standard)
- Refresh via client credentials (re-authenticate)
- IDE handles refresh automatically

**Error Codes**:
- `401`: Unauthorized (invalid/missing token)
- `403`: Forbidden (quota exceeded or insufficient permissions)
- `429`: Too Many Requests (rate limit exceeded)
- `500`: Internal Server Error

**No HMAC/Signature**: Unlike exchanges, Bitquery uses simple OAuth tokens (no request signing).

---

### 5. [tiers_and_limits.md](./tiers_and_limits.md) (481 lines)
**Pricing, quotas, rate limits**

**Free Tier (Developer Plan)**:
- **Cost**: $0/month
- **Points**: 10,000 trial (first month)
- **Rate Limit**: 10 requests/minute
- **Rows per Request**: 10 max
- **WebSocket Streams**: 2 simultaneous
- **Support**: Public Telegram/Community
- **Use Case**: Testing, learning, small projects

**Commercial Plan**:
- **Cost**: Custom (contact sales)
- **Points**: Custom allocation
- **Rate Limit**: No throttling (scalable)
- **Rows per Request**: Unlimited
- **WebSocket Streams**: Unlimited
- **Support**: 24/7 Engineering + Priority Slack
- **Features**: SQL access, Kafka streams, cloud exports (Snowflake, BigQuery, etc.)
- **SLA**: Custom SLA

**Points System**:
- **Realtime queries**: 5 points per cube (flat rate)
- **Archive queries**: Variable (based on complexity and data volume)
- **WebSocket subscriptions**: 40 points/minute per stream
- **Overage**: Blocked when quota exhausted (free tier)

**Rate Limit Handling**:
- HTTP 429 error when rate limited
- Exponential backoff recommended
- No rate limit headers returned

**Optimization Strategies**:
- Use `limit` to reduce data returned
- Narrow time ranges (`since`, `till`)
- Filter by specific entities
- Use realtime dataset for subscriptions (flat cost)
- Cache results locally

---

### 6. [data_types.md](./data_types.md) (594 lines)
**Complete data catalog**

**PRIMARY FOCUS**: On-chain Blockchain Data (40+ chains)

**Available Data Types**:

1. **Blockchain Infrastructure**
   - Blocks, Transactions, Mempool
   - Gas prices, fees
   - Validators/miners

2. **Token Data**
   - Token transfers (ERC-20, BEP-20, SPL, etc.)
   - Token metadata (symbol, name, supply, decimals)
   - Token holders (balances, distribution, Gini coefficient)

3. **DEX Trading**
   - DEX trades (40+ protocols: Uniswap, PancakeSwap, Raydium, etc.)
   - Trade prices, volumes
   - Liquidity pools, reserves
   - OHLCV data (any interval via aggregation)
   - Multi-hop swaps

4. **NFT Data**
   - NFT trades (OpenSea, Blur, Magic Eden, etc.)
   - NFT transfers (ERC-721, ERC-1155)
   - NFT ownership
   - Floor prices, volume analytics

5. **Smart Contracts**
   - Event logs (decoded)
   - Function calls (decoded)
   - Contract deployments
   - Call traces

6. **Wallet/Balance Data**
   - Balance updates (historical and real-time)
   - Wallet activity tracking
   - Portfolio tracking

7. **Staking/Rewards**
   - Block rewards
   - Staking deposits/withdrawals
   - Validator performance

8. **Cross-chain Data**
   - Bridge transactions
   - Multi-chain aggregation

**NOT Available**:
- Traditional finance (stocks, forex, commodities)
- Centralized exchange data (Binance, Coinbase trades)
- Off-chain data (Lightning Network, L2 off-chain txs)
- Social sentiment, news
- Shielded/private transactions (Zcash, Monero)

**Supported Blockchains** (40+):
- EVM: Ethereum, BSC, Polygon, Arbitrum, Base, Optimism, Avalanche, etc.
- Non-EVM: Solana, Bitcoin, Cardano, Ripple, Stellar, Algorand, Cosmos, Tron, etc.

---

### 7. [response_formats.md](./response_formats.md) (929 lines)
**Exact JSON response examples**

**GraphQL Response Structure**:
```json
{
  "data": {
    "EVM": {
      "CubeName": [ ... ]
    }
  },
  "errors": [ ... ]
}
```

**Comprehensive Examples For**:
- Blocks (latest 10 blocks)
- Transactions (by hash)
- Transfers (USDT transfers)
- DEXTrades (Uniswap V3 trades)
- BalanceUpdates (address balance changes)
- Events (Transfer event logs)
- Calls (smart contract function calls)
- MempoolTransactions (pending transactions)
- Aggregated data (total volume by protocol)
- Solana data (SPL transfers)
- Error responses
- WebSocket subscription messages

**All examples are EXACT responses from official Bitquery documentation.**

**Field Types**:
- String (addresses, hashes)
- Int (block numbers, counts)
- Float (amounts, prices)
- Boolean (success flags)
- Timestamp (ISO 8601 UTC)
- Objects (nested structures)
- Arrays (lists)

**Notes**:
- All timestamps UTC (ISO 8601)
- Addresses lowercase checksummed (EIP-55)
- Amounts in token decimals
- Null values possible if data unavailable

---

### 8. [coverage.md](./coverage.md) (541 lines)
**Geographic and data coverage**

**Geographic Coverage**:
- **Global**: Yes (blockchain data is borderless)
- **Restrictions**: Likely standard OFAC compliance (sanctioned countries)
- **VPN**: Not blocked
- **Geo-fencing**: No

**Markets Covered**:
- **Decentralized Exchanges**: 40+ DEX protocols (Uniswap, PancakeSwap, Raydium, etc.)
- **NFT Marketplaces**: OpenSea, Blur, LooksRare, Magic Eden, etc.
- **Centralized Exchanges**: NOT covered (no CEX data)
- **Traditional Markets**: NOT covered (no stocks, forex, commodities)

**Instrument Coverage**:
- **Tokens**: 1,000,000+ across all chains
- **NFT Collections**: 100,000+ on Ethereum, 10,000+ on Solana
- **DEX Pairs**: 100,000+ pairs across all DEXs

**Historical Depth**:
- **Bitcoin**: From Jan 2009 (15+ years)
- **Ethereum**: From Jul 2015 (9+ years)
- **Other chains**: From genesis block
- **Rule**: Full blockchain history available

**Granularity**:
- Block-level (native granularity)
- Transaction-level (individual transactions)
- 1-second to monthly aggregations (via GraphQL grouping)

**Real-time vs Delayed**:
- **Realtime dataset**: <1 second latency
- **Archive dataset**: 1-5 seconds latency
- **No artificial delays** (same speed for free and paid tiers)

**Update Frequency**:
- Blocks: Every block (~12s ETH, ~3s BSC, ~0.4s Solana)
- Transactions, Trades, Transfers: Real-time (as confirmed)
- Mempool: Real-time (pending transactions)

**Data Quality**:
- **Source**: Direct from blockchain nodes (first-party)
- **Validation**: Yes (consensus validation)
- **Reorgs**: Handled automatically
- **Completeness**: Rare gaps (reindexed if detected)
- **Accuracy**: High (verifiable against blockchain)

**Coverage Limitations**:
- No CEX data
- No off-chain transactions
- No private/shielded transactions
- No traditional finance data
- No social/sentiment data

---

## Key Takeaways

### What Bitquery IS:
1. **Blockchain data provider** (40+ chains)
2. **GraphQL API** (flexible querying)
3. **Real-time WebSocket streaming**
4. **Complete historical data** from genesis
5. **DEX trading analytics**
6. **NFT marketplace tracking**
7. **On-chain wallet/balance tracking**
8. **Smart contract event monitoring**

### What Bitquery IS NOT:
1. **NOT a crypto exchange** (no trading, no order execution)
2. **NOT a CEX aggregator** (no Binance/Coinbase data)
3. **NOT a traditional finance data provider** (no stocks/forex)
4. **NOT a social sentiment API** (no Twitter/news)
5. **NOT a Layer 2 off-chain data provider** (only on-chain data)

---

## Quick Reference

| Aspect | Value |
|--------|-------|
| **Provider** | Bitquery |
| **Type** | Data Provider (Blockchain/On-chain) |
| **API Style** | GraphQL (+ WebSocket subscriptions) |
| **Auth** | OAuth 2.0 (Bearer token) |
| **Base URL** | https://streaming.bitquery.io/graphql |
| **WebSocket** | wss://streaming.bitquery.io/graphql |
| **Free Tier** | 10K points, 10 req/min, 2 streams |
| **Paid Tier** | Custom pricing (contact sales) |
| **Blockchains** | 40+ (ETH, BSC, Solana, Bitcoin, etc.) |
| **Historical Depth** | From blockchain genesis |
| **Real-time Latency** | <1 second |
| **Documentation** | https://docs.bitquery.io/ |
| **IDE** | https://ide.bitquery.io |
| **Sign Up** | https://account.bitquery.io/auth/signup |

---

## Implementation Notes

### For V5 Connector

**Bitquery is unique** - it's NOT an exchange, so standard V5 connector patterns don't apply:

1. **No Trading Traits** - Only `MarketData` trait (if creating one)
2. **GraphQL Client** - Need GraphQL client library (not just HTTP requests)
3. **WebSocket Protocol** - Use `graphql-transport-ws` or `graphql-ws` protocol
4. **OAuth Token Management** - Handle token expiration and refresh
5. **Points Tracking** - Monitor quota usage
6. **Error Handling** - GraphQL errors are in `errors` array, not HTTP status
7. **Query Construction** - Build GraphQL queries dynamically
8. **Multi-chain** - Single API for all chains (specify `network` parameter)

**Recommended Implementation Strategy**:
- Create dedicated `bitquery` module (not generic exchange pattern)
- Use `graphql_client` crate for query generation
- Implement WebSocket subscriptions for real-time data
- Cache responses to minimize points usage
- Provide helpers for common queries (DEX trades, NFT sales, balances)

---

## Next Steps

1. **Review Research** - Read all 8 files thoroughly
2. **Design Connector** - Plan Bitquery-specific architecture (GraphQL-first)
3. **Implement Auth** - OAuth token management
4. **Build Query Builder** - GraphQL query construction helpers
5. **WebSocket Client** - Subscription support
6. **Test with Free Tier** - Validate against 10K points quota
7. **Document Use Cases** - DEX analytics, NFT tracking, wallet monitoring

---

## Sources

All research data gathered from official Bitquery documentation:

- [Bitquery V2 Docs](https://docs.bitquery.io/)
- [Bitquery V1 Docs (Legacy)](https://docs.bitquery.io/v1/)
- [GraphQL IDE](https://ide.bitquery.io)
- [Query Principles](https://docs.bitquery.io/docs/graphql/query/)
- [Getting Started](https://docs.bitquery.io/docs/start/)
- [Authentication](https://docs.bitquery.io/docs/authorisation/how-to-use/)
- [WebSocket Subscriptions](https://docs.bitquery.io/docs/subscriptions/websockets/)
- [Pricing](https://bitquery.io/pricing)
- [Starter Queries](https://docs.bitquery.io/docs/start/starter-queries/)
- [DEXTrades Cube](https://docs.bitquery.io/docs/cubes/dextrades/)
- [Balance Updates](https://docs.bitquery.io/docs/cubes/balance-updates-cube/)
- [Points System](https://docs.bitquery.io/docs/ide/points/)
- [Common Errors](https://docs.bitquery.io/docs/start/errors/)
- [Community Forum](https://community.bitquery.io/)
- [GitHub Documentation](https://github.com/bitquery/documentation)

**Research Completed**: 2026-01-26
**Total Research Time**: ~2 hours
**Data Quality**: Exhaustive (4,657 lines of comprehensive documentation)
