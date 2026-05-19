# Crypto.com Exchange API v1 - Research Documentation

Complete API research for implementing Crypto.com V5 connector following the KuCoin reference architecture.

---

## Documentation Files

### 1. [endpoints.md](./endpoints.md)
Comprehensive endpoint reference for all trait implementations:
- **MarketData Trait:** instruments, order book, candlesticks, trades, tickers, valuations
- **Trading Trait:** create/amend/cancel orders, order queries, order history
- **Account Trait:** balance, fees, transactions, sub-accounts
- **Positions Trait:** get positions, close position, margin management

**Key Features:**
- REST base URL: `https://api.crypto.com/exchange/v1/{method}`
- All endpoints with request/response examples
- Order types: LIMIT, MARKET, STOP_LOSS, STOP_LIMIT, TAKE_PROFIT
- Time in force: GOOD_TILL_CANCEL, IMMEDIATE_OR_CANCEL, FILL_OR_KILL

---

### 2. [authentication.md](./authentication.md)
HMAC-SHA256 signature authentication for REST and WebSocket:
- API key generation and management
- Signature algorithm: `method + id + api_key + params_string + nonce`
- REST authentication for private endpoints
- WebSocket authentication flow (`public/auth`)
- Nonce management (milliseconds timestamp)
- Error handling and troubleshooting

**Critical Points:**
- Never expose API secret in requests
- Signature payload must follow exact format
- Parameters must be sorted alphabetically
- All numbers as strings in JSON

---

### 3. [response_formats.md](./response_formats.md)
Complete response structure documentation:
- Standard response format (`id`, `method`, `code`, `result`)
- Market data responses (instruments, order book, candlesticks, trades, tickers)
- Trading responses (create order, cancel order, get orders)
- Account responses (balance, fees, positions)
- Error codes and messages
- Pagination support

**Important:**
- All numeric values are strings: `"50000.00"` not `50000.00`
- Timestamps in milliseconds
- Success: `code: 0`, errors: non-zero codes

---

### 4. [symbols.md](./symbols.md)
Symbol/instrument naming conventions and formatting:
- **Spot:** `BASE_QUOTE` (e.g., `BTC_USDT`)
- **Perpetual:** `BASEQUOTE-PERP` (e.g., `BTCUSD-PERP`)
- **Futures:** `BASEQUOTE-EXPIRY` (e.g., `BTCUSD-210528m2`)
- **Index:** `BASEQUOTE-INDEX` (e.g., `BTCUSD-INDEX`)

**Parsing Functions:**
- Rust implementation for symbol formatting
- Symbol validation
- Instrument type detection
- Common trading pairs reference

---

### 5. [rate_limits.md](./rate_limits.md)
Comprehensive rate limit specifications:
- **Trading endpoints:** 15 req/100ms (create/cancel orders)
- **Order detail:** 30 req/100ms
- **Historical:** 1 req/second
- **Private:** 3 req/100ms
- **Public:** 100 req/second
- **WebSocket User:** 150 req/second
- **WebSocket Market:** 100 req/second

**Implementation:**
- Multi-tier rate limiter example
- Rate limit categories
- Error handling (code 10007: THROTTLE_REACHED)
- Retry strategies with exponential backoff

---

### 6. [websocket.md](./websocket.md)
WebSocket API documentation for real-time data:
- **URLs:**
  - User API: `wss://stream.crypto.com/exchange/v1/user`
  - Market Data: `wss://stream.crypto.com/exchange/v1/market`
- **Authentication:** `public/auth` with HMAC-SHA256
- **Public Channels:** ticker, book, trade, candlestick, funding
- **Private Channels:** user.order, user.trade, user.balance, user.positions
- **Heartbeat mechanism:** respond to `public/heartbeat`
- **Connection management:** 1-second delay after connection

**Critical:**
- ALWAYS wait 1 second after WebSocket connection before sending requests
- Implement reconnection logic with exponential backoff
- Handle heartbeats to maintain connection

---

## Implementation Priority

### Phase 1: Basic Structure
1. Create module structure following KuCoin reference
2. Implement `endpoints.rs` with URL constants and endpoint enum
3. Implement `auth.rs` with HMAC-SHA256 signing

### Phase 2: Core Functionality
4. Implement `parser.rs` for JSON response parsing
5. Implement `connector.rs` with MarketData trait
6. Add rate limiting

### Phase 3: Trading & Account
7. Implement Trading trait (create/cancel orders)
8. Implement Account trait (balance, fees)
9. Implement Positions trait (if needed)

### Phase 4: WebSocket (Optional)
10. Implement `websocket.rs` for real-time data
11. Add WebSocket authentication
12. Implement channel subscriptions

---

## Key Differences from KuCoin

| Feature | Crypto.com | KuCoin |
|---------|------------|--------|
| Symbol Format (Spot) | `BTC_USDT` | `BTC-USDT` |
| Symbol Format (Perp) | `BTCUSD-PERP` | `XBTUSDTM` |
| Auth Method | HMAC-SHA256 | HMAC-SHA256 |
| Signature Payload | `method+id+key+params+nonce` | Different format |
| Numeric Values | Always strings | Mixed |
| WebSocket Auth | `public/auth` | Token-based |
| Rate Limit Window | 100ms for trading | Different |

---

## API Peculiarities

### 1. All Numbers as Strings
```json
{
  "price": "50000.00",     // Correct
  "quantity": "0.5000"     // Correct
}
```

**NOT:**
```json
{
  "price": 50000.00,       // WRONG
  "quantity": 0.5          // WRONG
}
```

### 2. Signature Parameter Sorting
Parameters must be sorted alphabetically before concatenation:
```
instrument_nameBTCUSD-PERPprice50000.00quantity0.5sideBUYtypeLIMIT
```

### 3. WebSocket 1-Second Delay
ALWAYS wait 1 second after WebSocket connection:
```rust
connect_async(url).await?;
tokio::time::sleep(Duration::from_secs(1)).await; // CRITICAL
```

### 4. Request ID Field
Every request needs a unique `id` field (not just nonce):
```json
{
  "id": 1,
  "method": "private/create-order",
  "nonce": 1587523073344
}
```

### 5. Order Status Workflow
```
ACTIVE → FILLED
ACTIVE → CANCELED
ACTIVE → REJECTED
ACTIVE → EXPIRED
```

---

## Testing Strategy

### 1. Sandbox Environment
- REST: `https://uat-api.3ona.co/exchange/v1/{method}`
- WS User: `wss://uat-stream.3ona.co/exchange/v1/user`
- WS Market: `wss://uat-stream.3ona.co/exchange/v1/market`

### 2. Test Sequence
1. Test signature generation with known values
2. Test `public/get-instruments` (no auth)
3. Test `private/user-balance` (simple auth test)
4. Test order creation in sandbox
5. Test WebSocket connection and auth
6. Test rate limiting

### 3. Unit Tests
- Symbol formatting/parsing
- Signature generation
- Rate limiter logic
- JSON parsing

---

## Common Pitfalls

1. **Forgetting 1-second WebSocket delay** → Connection drops
2. **Not sorting parameters** → Invalid signature
3. **Using numeric types instead of strings** → Request rejected
4. **Reusing nonces** → Authentication fails
5. **Exceeding rate limits** → Temporary blocking
6. **Wrong symbol format** → Invalid instrument error
7. **Not handling heartbeats** → WebSocket disconnects

---

## Reference Implementation

Follow the KuCoin V5 connector structure:
```
v5/exchanges/kucoin/
├── mod.rs          # Module exports
├── endpoints.rs    # URLs, endpoint enum
├── auth.rs         # HMAC-SHA256 signing
├── parser.rs       # JSON parsing
├── connector.rs    # Trait implementations
└── websocket.rs    # WebSocket client
```

Apply the same pattern for Crypto.com with adjustments for:
- Different signature algorithm
- Different symbol formats
- String-only numeric values
- Different rate limits

---

## Additional Resources

### Official Documentation
- Main API Docs: https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html
- Exchange Website: https://crypto.com/exchange

### API Versions
- V1 (Current): REST + WebSocket, unified wallet
- V2 (Deprecated): Being phased out
- Use V1 for all new implementations

### Support
- API issues: Contact via Exchange support
- Rate limit increases: Apply for VIP tiers
- Production access: Complete KYC verification

---

## Implementation Checklist

### Module Structure
- [ ] Create `crypto_com/` directory
- [ ] Add `mod.rs` with exports
- [ ] Create `endpoints.rs`
- [ ] Create `auth.rs`
- [ ] Create `parser.rs`
- [ ] Create `connector.rs`
- [ ] Create `websocket.rs` (optional)

### Endpoints Module
- [ ] Define base URLs (prod + sandbox)
- [ ] Create endpoint enum
- [ ] Implement symbol formatting functions
- [ ] Add instrument type enum

### Auth Module
- [ ] Implement HMAC-SHA256 signature function
- [ ] Add parameter sorting logic
- [ ] Add nonce generation
- [ ] Implement WebSocket auth signature

### Parser Module
- [ ] Define response structs
- [ ] Implement JSON parsing
- [ ] Handle error responses
- [ ] Parse all endpoint response types

### Connector Module
- [ ] Implement MarketData trait
- [ ] Implement Trading trait
- [ ] Implement Account trait
- [ ] Implement Positions trait (if needed)
- [ ] Add rate limiting
- [ ] Add error handling

### WebSocket Module (Optional)
- [ ] Connection management
- [ ] Authentication flow
- [ ] Public channel subscriptions
- [ ] Private channel subscriptions
- [ ] Heartbeat handling
- [ ] Reconnection logic

### Testing
- [ ] Unit tests for symbol formatting
- [ ] Unit tests for signature generation
- [ ] Integration tests with sandbox
- [ ] Rate limiter tests
- [ ] WebSocket connection tests
- [ ] Full order lifecycle test

---

## Notes

- Research completed: 2026-01-20
- API Version: Exchange API v1
- Documentation source: Official Crypto.com Exchange API docs
- All examples tested against official documentation
- Rate limits verified from official docs
- Symbol formats confirmed from API specification

---

## Next Steps

1. Review KuCoin V5 implementation in `v5/exchanges/kucoin/`
2. Create Crypto.com module structure
3. Implement authentication and endpoints
4. Test in sandbox environment
5. Implement traits following V5 architecture
6. Add comprehensive error handling
7. Implement rate limiting
8. Test full order lifecycle
9. Add WebSocket support (if needed)
10. Production testing with small orders
