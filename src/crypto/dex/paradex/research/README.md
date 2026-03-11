# Paradex API Research Documentation

## Overview

This directory contains comprehensive research documentation for implementing the Paradex exchange connector in the V5 architecture.

**Exchange**: Paradex
**Website**: https://paradex.trade
**Documentation**: https://docs.paradex.trade
**Type**: Perpetual futures and options (StarkNet-based)

---

## Research Files

### 1. [endpoints.md](./endpoints.md)
Complete REST API endpoint documentation including:
- Base URLs (production and testnet)
- Authentication endpoints
- Market data endpoints (public)
- Account endpoints (private)
- Order management endpoints
- Trade history endpoints
- System endpoints
- Request/response formats for each endpoint

### 2. [authentication.md](./authentication.md)
Authentication system documentation covering:
- JWT token-based authentication
- StarkNet cryptographic signatures
- Authentication flow (onboarding → JWT → API access)
- Signature generation process
- Order signing requirements
- Subkey authentication
- WebSocket authentication
- Security best practices
- Performance benchmarks by language

### 3. [rate_limits.md](./rate_limits.md)
Rate limiting information including:
- Public endpoint limits (1,500 req/min per IP)
- Private endpoint limits (800 req/s for orders, 120 req/s for reads)
- IP-based constraints (1,500 req/min across all accounts)
- Batch operation benefits (50x efficiency)
- JWT refresh strategy
- Best practices for rate limit management
- Multi-account considerations

### 4. [response_formats.md](./response_formats.md)
Response format specifications covering:
- General response structure
- Common field types
- Market data responses
- Account data responses
- Order responses
- Error response formats
- Timestamp formats (Unix milliseconds)
- Decimal precision handling (strings)
- Enumeration values

### 5. [symbols.md](./symbols.md)
Market symbol format documentation:
- Symbol format: `{BASE}-{QUOTE}-{TYPE}`
- Examples: `BTC-USD-PERP`, `ETH-USD-PERP`
- Symbol validation and parsing
- Market discovery via `/markets` endpoint
- Instrument types (PERP, PERP_OPTION)
- Market kinds (cross-margin, isolated)
- Symbol mapping from other exchanges
- WebSocket channel naming

### 6. [websocket.md](./websocket.md)
WebSocket API documentation including:
- Connection details (wss URLs)
- JSON-RPC 2.0 protocol
- Heartbeat mechanism (ping/pong)
- Authentication flow
- Public channels (markets_summary, order_book, bbo, trades)
- Private channels (account, positions, orders, fills)
- Subscription management
- Channel naming conventions
- Best practices
- Complete implementation examples

---

## Key Technical Details

### Base URLs

**Production**:
- REST: `https://api.prod.paradex.trade/v1`
- WebSocket: `wss://ws.api.prod.paradex.trade/v1`

**Testnet**:
- REST: `https://api.testnet.paradex.trade/v1`
- WebSocket: `wss://ws.api.testnet.paradex.trade/v1`

### Authentication

**Type**: JWT tokens via StarkNet signatures

**Process**:
1. Generate StarkNet signature with private key
2. POST to `/v1/auth` with signature headers
3. Receive JWT token (5-minute lifetime)
4. Include JWT in `Authorization: Bearer {token}` header

**Headers**:
- `PARADEX-STARKNET-ACCOUNT`: Account address
- `PARADEX-STARKNET-SIGNATURE`: [r, s] signature array
- `PARADEX-TIMESTAMP`: Unix timestamp

### Rate Limits Summary

| Endpoint Type | Limit | Scope |
|---------------|-------|-------|
| Public | 1,500 req/min | Per IP |
| Auth endpoints | 600 req/min | Per IP |
| Order operations | 800 req/s OR 17,250 req/min | Per account |
| Read operations | 120 req/s OR 600 req/min | Per account |
| All private | 1,500 req/min | Per IP (additional) |

**Key Optimization**: Batch operations count as 1 unit (50x efficiency)

### Symbol Format

```
{BASE}-{QUOTE}-{TYPE}

Examples:
- BTC-USD-PERP
- ETH-USD-PERP
- SOL-USD-PERP
```

**Components**:
- BASE: Cryptocurrency (BTC, ETH, etc.)
- QUOTE: Quote currency (USD)
- TYPE: PERP or PERP_OPTION

### Data Types

**Timestamps**: Unix milliseconds (not seconds)
```json
{
  "created_at": 1681759756789
}
```

**Precision**: Strings for decimal numbers
```json
{
  "price": "65432.50",
  "size": "1.5"
}
```

**Reason**: Avoid floating-point errors, preserve exact values

---

## Implementation Checklist

### Core Requirements

- [ ] REST client with JWT authentication
- [ ] StarkNet signature generation
- [ ] JWT token refresh logic (every 3 minutes)
- [ ] WebSocket client with JSON-RPC 2.0
- [ ] Ping/pong heartbeat handling
- [ ] Rate limit tracking and enforcement
- [ ] Symbol validation and normalization
- [ ] Decimal precision handling (string parsing)
- [ ] Timestamp conversion (Unix ms)

### MarketData Trait

- [ ] `fetch_markets()` - GET /markets
- [ ] `fetch_ticker()` - GET /markets/summary
- [ ] `fetch_orderbook()` - GET /orderbook/:market
- [ ] `fetch_trades()` - GET /trades
- [ ] `fetch_klines()` - GET /klines (if available)

### Trading Trait

- [ ] `place_order()` - POST /orders
- [ ] `cancel_order()` - DELETE /orders/:id
- [ ] `cancel_all_orders()` - DELETE /orders
- [ ] `modify_order()` - PUT /orders/:id
- [ ] `fetch_order()` - GET /orders/:id
- [ ] `fetch_open_orders()` - GET /orders
- [ ] `place_batch_orders()` - POST /orders/batch
- [ ] `cancel_batch_orders()` - DELETE /orders/batch

### Account Trait

- [ ] `fetch_account()` - GET /account
- [ ] `fetch_positions()` - GET /positions
- [ ] `fetch_balance()` - GET /balances
- [ ] `fetch_fills()` - GET /fills
- [ ] `fetch_funding_payments()` - GET /funding/payments

### WebSocket Subscriptions

- [ ] Public: markets_summary, order_book, bbo, trades
- [ ] Private: account, positions, orders, fills
- [ ] Subscription management
- [ ] Authentication handling
- [ ] Reconnection logic

---

## Special Considerations

### 1. StarkNet Integration

Paradex is built on **StarkNet** (Layer 2), requiring:
- StarkNet signature generation (not Ethereum ECDSA)
- StarkNet account addresses (0x-prefixed)
- Off-chain message encoding (EIP-712-inspired)
- Pedersen hash functions

**Rust Library**: Use `starknet-rs` or similar for signing

### 2. Order Signing

**Every order requires signature** with private key:

```rust
struct Order {
    market: String,
    side: Side,
    type: OrderType,
    price: Decimal,
    size: Decimal,
    signature: String,           // Required
    signature_timestamp: i64,     // Required
    // ... other fields
}
```

**Performance**: Pre-sign static components for low latency

### 3. Retail Price Improvement (RPI)

Paradex offers **Retail Price Improvement**:
- Better execution prices for retail orders
- Different fee structure (lower)
- Flagged in fills: `is_rpi: true`

**Order Instruction**: Use `"RPI"` for eligible orders

### 4. Isolated vs Cross Margin

**Cross-margin**:
- Shared margin across positions
- Higher leverage
- Default for most markets

**Isolated margin**:
- Per-position margin
- Limited risk
- Specific market kinds

Check `market_kind` field in market data

### 5. Perpetual Options

In addition to perpetual futures, Paradex offers **perpetual options**:
- `asset_kind: "PERP_OPTION"`
- Additional fields: `option_type`, `strike_price`
- Greeks available in market summary

---

## Error Handling

### Common Errors

| Status | Error | Handling |
|--------|-------|----------|
| 401 | Unauthorized | Refresh JWT token |
| 400 | Invalid signature | Check signature generation |
| 429 | Rate limit | Exponential backoff, use batching |
| 400 | Insufficient margin | Check account balance |
| 404 | Market not found | Validate symbol |

### Retry Strategy

```rust
async fn execute_with_retry<T>(
    operation: impl Fn() -> Future<Output = Result<T>>,
    max_retries: u32,
) -> Result<T> {
    let mut delay_ms = 100;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if e.is_retryable() && attempt < max_retries - 1 {
                    sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2;
                } else {
                    return Err(e);
                }
            }
        }
    }

    Err("Max retries exceeded")
}
```

---

## Testing

### Testnet Access

Use testnet for development:
- REST: `https://api.testnet.paradex.trade/v1`
- WebSocket: `wss://ws.api.testnet.paradex.trade/v1`
- Testnet tokens: Obtain from Paradex faucet

### Integration Tests

**Recommended test coverage**:
1. Authentication flow
2. Market data retrieval
3. Order lifecycle (place → fill → cancel)
4. Position tracking
5. WebSocket subscriptions
6. Rate limit handling
7. Error scenarios
8. Reconnection logic

---

## Performance Optimization

### 1. Batch Operations

Use batch endpoints for efficiency:
- 50x rate limit reduction
- Lower latency
- Better throughput

### 2. WebSocket for Real-Time

**Prefer WebSocket** over polling:
- No rate limits for subscriptions
- Event-driven updates
- Lower latency

### 3. BBO for Prices

Use BBO channel instead of full orderbook for price tracking:
- Event-driven (no throttling)
- Minimal bandwidth
- Optimal for execution

### 4. Connection Pooling

Reuse HTTP connections:
- Connection pooling (HTTP/2)
- Persistent WebSocket connection
- Avoid reconnection overhead

---

## References

### Official Documentation
- **Main Docs**: https://docs.paradex.trade
- **API Reference**: https://docs.paradex.trade/api/general-information
- **WebSocket**: https://docs.paradex.trade/ws/general-information
- **Authentication**: https://docs.paradex.trade/trading/api-authentication
- **Best Practices**: https://docs.paradex.trade/trading/api-best-practices

### Code Repositories
- **Python SDK**: https://github.com/tradeparadex/paradex-py
- **Code Samples**: https://github.com/tradeparadex/code-samples
- **C++ Signing**: https://github.com/tradeparadex/starknet-signing-cpp
- **Documentation**: https://github.com/tradeparadex/paradex-docs

### Libraries
- **StarkNet (Rust)**: `starknet-rs`
- **WebSocket (Rust)**: `tokio-tungstenite`
- **HTTP (Rust)**: `reqwest`
- **Decimal (Rust)**: `rust_decimal`
- **JSON (Rust)**: `serde_json`

---

## Next Steps

1. **Review KuCoin V5 implementation** (reference pattern)
2. **Set up StarkNet signing** (most critical dependency)
3. **Implement authentication module** (`auth.rs`)
4. **Create endpoint definitions** (`endpoints.rs`)
5. **Build parser for responses** (`parser.rs`)
6. **Implement connector** (`connector.rs`)
7. **Add WebSocket support** (`websocket.rs`)
8. **Write integration tests**
9. **Optimize for production** (batching, caching, etc.)

---

## Questions for Paradex Team (if needed)

1. Exact pagination format for list endpoints?
2. WebSocket connection limits per IP?
3. Maximum subscriptions per WebSocket connection?
4. Klines/candles endpoint availability?
5. Public trades endpoint details?
6. Subaccount management endpoints?
7. Block trades API details?

---

## Notes

- Paradex is StarkNet-based (Layer 2 on Ethereum)
- Zero trading fees for makers in many cases
- Privacy-focused (StarkNet benefits)
- High performance (claimed sub-10ms latency)
- Retail Price Improvement (RPI) feature
- Both perpetual futures and options
- Cross-margin and isolated margin support

---

## License

This research documentation is for internal use in the NEMO trading system development.

**Sources**: All information gathered from official Paradex documentation and public resources as of January 2026.
