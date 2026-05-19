# HyperLiquid V5 Connector Research

Complete API documentation for implementing HyperLiquid exchange connector following V5 architecture.

---

## Overview

HyperLiquid is a **hybrid DEX with on-chain order book** running on its own L1 blockchain (HyperCore). It offers perpetual futures and spot trading with unique characteristics:

- **Wallet-based authentication**: EIP-712 signatures instead of API keys
- **Two signing schemes**: L1 actions (phantom agent) vs user-signed actions
- **Unified POST endpoints**: `/info` and `/exchange`
- **WebSocket for real-time data**: Market data and account updates
- **Volume-based rate limits**: 1 request per $1 traded + 10K initial buffer

---

## Research Documents

### 1. [endpoints.md](./endpoints.md)
Complete endpoint reference for all traits:
- **MarketData trait**: Tickers, order book, trades, candles, funding rates
- **Trading trait**: Place/cancel/modify orders, all order types
- **Account trait**: Balances, open orders, trade history, fees
- **Positions trait**: Positions, leverage, margin management
- Additional operations: Transfers, withdrawals, subaccounts

**Key Insights**:
- Info endpoints use symbol names (`"BTC"`)
- Exchange endpoints use asset IDs (`0` for BTC)
- All requests are POST with JSON body
- Spot uses `@index` format or `PAIR/QUOTE`

### 2. [authentication.md](./authentication.md)
EIP-712 wallet-based authentication system:
- **Two signing schemes**: L1 actions vs user-signed actions
- **Nonce management**: Timestamp-based, 100-nonce window
- **Agent wallets**: Delegated signing authorities
- **Common pitfalls**: Address casing, field ordering, numeric precision

**Key Insights**:
- SDK reference highly recommended (don't implement from scratch)
- L1 actions use msgpack + phantom agent
- User-signed actions use direct EIP-712
- Addresses must be lowercase before signing
- Separate agent wallet per process to avoid nonce collisions

### 3. [rate_limits.md](./rate_limits.md)
Multi-tier rate limiting system:
- **IP-based**: 1200 weight/minute, batching reduces weight
- **Address-based**: 1 request per $1 traded + 10K buffer
- **WebSocket**: 100 connections, 1000 subscriptions, 2000 msg/min
- **Open orders**: 1000 base + 1 per 5M volume (max 5000)

**Key Insights**:
- Batch ≤39 orders for optimal weight (weight = 1)
- Cancel requests have 2x allowance
- Use WebSocket for market data (saves REST calls)
- Query `userRateLimit` endpoint to track allowance

### 4. [response_formats.md](./response_formats.md)
Complete JSON response structures:
- **Info responses**: Direct data (no wrapper)
- **Exchange responses**: `{status, response: {type, data}}`
- **Batch responses**: Array of statuses matching request order
- **Error formats**: Per-order errors or global errors

**Key Insights**:
- All numbers are strings (preserve precision)
- Timestamps in milliseconds
- Batch responses maintain request order
- WebSocket uses `{channel, data}` format

### 5. [symbols.md](./symbols.md)
Symbol naming and asset ID conventions:
- **Perpetuals**: Coin name (`"BTC"`) → asset ID `0`
- **Spot**: `@{index}` or `PAIR/QUOTE` → asset ID `10000 + index`
- **Builder DEX perps**: `dex:COIN` → `100000 + dex*10000 + idx`
- **Critical**: Token index ≠ Spot index

**Key Insights**:
- Query metadata on startup (don't hardcode)
- Cache metadata with TTL (1-6 hours)
- Info endpoints use names, exchange endpoints use IDs
- Spot ID = 10000 + spot_index (not token index!)

### 6. [websocket.md](./websocket.md)
Real-time WebSocket API:
- **19 subscription types**: Market data and account updates
- **Connection limits**: 100 connections, 1000 subscriptions per IP
- **Automatic reconnection required**: Server may disconnect without notice
- **Snapshot + incremental**: Handle both message patterns

**Key Insights**:
- Always implement reconnection logic
- Resubscribe on reconnect
- First message may be snapshot (`isSnapshot: true`)
- Use separate connections for market vs user data
- Max 10 unique users across all user subscriptions

---

## API Characteristics

### Base URLs
- **Mainnet REST**: `https://api.hyperliquid.xyz`
- **Testnet REST**: `https://api.hyperliquid-testnet.xyz`
- **Mainnet WS**: `wss://api.hyperliquid.xyz/ws`
- **Testnet WS**: `wss://api.hyperliquid-testnet.xyz/ws`

### Unique Features
1. **No API keys**: Uses Ethereum wallet signatures (EIP-712)
2. **Unified endpoints**: All info via `/info`, all trading via `/exchange`
3. **Nonce flexibility**: 100-nonce window (not sequential like Ethereum)
4. **Volume-based limits**: More trading = more API requests allowed
5. **On-chain settlement**: All trades settle on HyperLiquid L1

### Trade Types Supported
- Spot trading
- Perpetual futures (up to 50x leverage)
- Cross margin and isolated margin
- Market, Limit, Stop, Take Profit orders
- TWAP and Scale orders

---

## Implementation Roadmap

### Phase 1: Core Infrastructure
1. Set up EIP-712 signing with ethers-rs
2. Implement nonce management (atomic counter)
3. Create HTTP client for REST endpoints
4. Implement basic error handling

### Phase 2: MarketData Trait
1. Get exchange info (meta, spotMeta)
2. Get ticker / 24hr stats
3. Get order book (l2Book)
4. Get recent trades
5. Get candles
6. Symbol normalization and caching

### Phase 3: Trading Trait
1. Place limit orders (GTC, IOC, ALO)
2. Cancel orders (by ID and by cloid)
3. Modify orders
4. Batch operations
5. Trigger orders (stop loss, take profit)

### Phase 4: Account Trait
1. Get balances (clearinghouseState, spotClearinghouseState)
2. Get open orders
3. Get order status
4. Get trade history (userFills)
5. Get fees and rate limits

### Phase 5: Positions Trait
1. Get positions
2. Set leverage
3. Update isolated margin

### Phase 6: WebSocket
1. Connection with auto-reconnect
2. Market data subscriptions (trades, l2Book, candles)
3. User data subscriptions (orderUpdates, userFills)
4. Snapshot handling

### Phase 7: Advanced Features
1. Agent wallet management
2. Subaccount support
3. Internal transfers
4. Withdrawals to L1
5. TWAP orders

---

## Testing Strategy

### 1. Testnet First
- URL: `https://api.hyperliquid-testnet.xyz`
- Get testnet funds from faucet
- Test all signature types
- Validate error handling

### 2. Incremental Testing
```
✓ Authentication (user-signed action: usdSend)
✓ Simple limit order (L1 action)
✓ Order cancellation
✓ Market data queries
✓ WebSocket subscriptions
✓ Batch orders
✓ Complex orders (triggers, TP/SL)
```

### 3. Edge Cases
- Nonce outside valid range
- Invalid symbols
- Insufficient margin
- Rate limit errors
- WebSocket reconnection
- Signature errors

---

## Key Differences from Other Exchanges

| Aspect | HyperLiquid | Traditional CEX |
|--------|-------------|-----------------|
| **Authentication** | EIP-712 wallet signatures | API keys (HMAC) |
| **Endpoints** | Unified POST (`/info`, `/exchange`) | Multiple REST paths |
| **Symbol IDs** | Integer indices (query from meta) | Fixed symbols |
| **Rate Limits** | Volume-based + IP-based | IP or account-based |
| **Nonce** | Timestamp, 100-window | Sequential or none |
| **Settlement** | On-chain L1 | Off-chain database |
| **Withdrawal** | ~5 min to L1 | Varies (instant to hours) |

---

## Common Pitfalls

### 1. Signature Issues
- ❌ Wrong signing scheme (L1 vs user-signed)
- ❌ Mixed-case addresses
- ❌ Incorrect field ordering
- ❌ Trailing zeros in numbers
- ✅ Use SDK as reference

### 2. Symbol Confusion
- ❌ Using coin name in exchange endpoint
- ❌ Confusing token index with spot index
- ❌ Hardcoding asset IDs
- ✅ Query metadata, use correct ID format

### 3. Rate Limits
- ❌ Not batching orders
- ❌ Polling instead of WebSocket
- ❌ Ignoring address-based limits
- ✅ Batch efficiently, use WS, monitor limits

### 4. WebSocket
- ❌ No reconnection logic
- ❌ Not resubscribing after reconnect
- ❌ Ignoring snapshot messages
- ✅ Auto-reconnect, handle snapshots

### 5. Agent Wallets
- ❌ Querying with agent address
- ❌ Reusing deregistered agents
- ❌ Shared agent across processes
- ✅ Query with master address, unique agents

---

## Required Rust Crates

```toml
[dependencies]
# EIP-712 signing
ethers = "2.0"

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# WebSocket
tokio-tungstenite = "0.21"
futures-util = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rmp-serde = "1.1"  # Msgpack for L1 actions

# Crypto
hex = "0.4"
sha2 = "0.10"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

---

## Reference Links

### Official Documentation
- [HyperLiquid API Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api)
- [Exchange Endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [Info Endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [WebSocket API](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket)
- [Signing Guide](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/signing)
- [Rate Limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)

### Official SDKs
- [Python SDK](https://github.com/hyperliquid-dex/hyperliquid-python-sdk) (Official)
- [TypeScript SDK](https://github.com/nomeida/hyperliquid) (Community)

### Additional Resources
- [QuickNode HyperLiquid Docs](https://www.quicknode.com/docs/hyperliquid/api-overview)
- [Chainstack Authentication Guide](https://docs.chainstack.com/docs/hyperliquid-authentication-guide)

---

## Architecture Notes (V5 Pattern)

### Module Structure
```
hyperliquid/
├── mod.rs              # Exports
├── endpoints.rs        # URLs, endpoint enum, symbol formatting
├── auth.rs             # EIP-712 signing (L1 + user-signed)
├── parser.rs           # JSON response parsing
├── connector.rs        # Trait implementations
└── websocket.rs        # WebSocket client
```

### Key Differences from KuCoin Reference
1. **No config-based auth**: Uses wallet + EIP-712 instead
2. **Symbol to ID mapping**: Query metadata, convert for requests
3. **Nonce management**: Atomic counter with timestamp validation
4. **Two signing methods**: `sign_l1_action()` and `sign_user_signed_action()`
5. **WebSocket reconnection**: More critical (server disconnects)

---

## Summary

HyperLiquid offers a unique trading infrastructure combining DEX architecture with CEX-like performance. The connector implementation requires:

1. **EIP-712 signature expertise**: Complex but well-documented
2. **Careful symbol handling**: Map names ↔ IDs correctly
3. **Robust rate limiting**: Multi-tier system with volume rewards
4. **Reliable WebSocket**: Auto-reconnection mandatory
5. **Comprehensive testing**: Start with testnet, test incrementally

The research documents provide complete specifications for implementing all required traits. Follow the KuCoin V5 pattern but adapt for HyperLiquid's wallet-based authentication and unique symbol system.

**Next Steps**:
1. Review authentication.md and set up EIP-712 signing
2. Implement endpoints.rs with symbol normalization
3. Build auth.rs with both signing schemes
4. Test on testnet with simple operations
5. Expand to full trait implementation
