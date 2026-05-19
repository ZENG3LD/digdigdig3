# OKX API v5 Research Documentation

Comprehensive research documentation for implementing OKX exchange connector following V5 architecture.

## Documentation Files

### 1. [endpoints.md](./endpoints.md)
Complete endpoint reference organized by trait implementation:

- **Base URLs** - REST and WebSocket production/demo URLs
- **MarketData Trait** - `get_price`, `get_orderbook`, `get_klines`, `get_ticker`, `ping`
- **Trading Trait** - `market_order`, `limit_order`, `cancel_order`, `get_order`, `get_open_orders`
- **Account Trait** - `get_balance`, `get_account_info`
- **Positions Trait** - `get_positions`, `get_funding_rate`, `set_leverage`

### 2. [authentication.md](./authentication.md)
Authentication mechanism and signature generation:

- **Required Headers** - `OK-ACCESS-KEY`, `OK-ACCESS-SIGN`, `OK-ACCESS-TIMESTAMP`, `OK-ACCESS-PASSPHRASE`
- **Signature Algorithm** - HMAC SHA256 with Base64 encoding
- **Pre-hash String** - `timestamp + method + requestPath + body`
- **WebSocket Authentication** - Login message format and signature
- **Error Codes** - 50xxx authentication error codes
- **Implementation Examples** - Rust, Python, JavaScript

### 3. [response_formats.md](./response_formats.md)
Unified JSON response structure and field descriptions:

- **Standard Format** - `{code, msg, data}` structure
- **Success Responses** - `code: "0"` with data array
- **Error Responses** - Non-zero code with error message
- **Market Data Formats** - Ticker, order book, candlestick array structures
- **Trading Responses** - Order placement, cancellation, status updates
- **Account/Position Data** - Balance and position response structures
- **WebSocket Messages** - Push notification formats

### 4. [symbols.md](./symbols.md)
Instrument types and symbol formatting:

- **Instrument Types** - SPOT, MARGIN, SWAP, FUTURES, OPTION
- **Symbol Formats**
  - SPOT/MARGIN: `BTC-USDT`
  - SWAP: `BTC-USDT-SWAP`
  - FUTURES: `BTC-USD-240329`
  - OPTION: `BTC-USD-240329-50000-C`
- **Linear vs Inverse** - USDT-margined vs coin-margined contracts
- **Underlying/Family** - Grouping related instruments
- **Symbol Parsing** - Extracting components from instrument IDs

### 5. [rate_limits.md](./rate_limits.md)
Comprehensive rate limit documentation:

- **REST Limits** - IP-based (public) and User ID-based (private)
- **Public Endpoints** - 20 requests per 2 seconds (IP)
- **Trading Limits** - 60/2s per instrument, 1,000/2s per sub-account
- **Independent Limits** - Place/amend/cancel have separate limits
- **WebSocket Limits** - 480 operations/hour, 3 subscriptions/second
- **VIP Benefits** - Up to 10,000 requests/2s for VIP5+
- **Error Codes** - 50011 (rate limit), 50061 (order limit)
- **Rate Limiting Strategies** - Token bucket, queuing, exponential backoff

### 6. [websocket.md](./websocket.md)
WebSocket API implementation guide:

- **Connection URLs** - Public, private, and business channels
- **Authentication** - Login message with signature
- **Public Channels** - Tickers, order books, trades, candles, funding rates
- **Private Channels** - Account, positions, orders, balance updates
- **Trading via WebSocket** - Place, cancel, amend orders
- **Subscription Management** - Subscribe, unsubscribe, limits
- **Connection Management** - Ping/pong, heartbeat, reconnection
- **Implementation Examples** - Rust WebSocket code

---

## Quick Reference

### Base URLs

**REST:**
```
Production: https://www.okx.com
Demo: https://www.okx.com (header: x-simulated-trading: 1)
```

**WebSocket:**
```
Public:  wss://ws.okx.com:8443/ws/v5/public
Private: wss://ws.okx.com:8443/ws/v5/private
```

### Authentication Headers

```rust
headers.insert("OK-ACCESS-KEY", api_key);
headers.insert("OK-ACCESS-SIGN", signature);
headers.insert("OK-ACCESS-TIMESTAMP", timestamp);
headers.insert("OK-ACCESS-PASSPHRASE", passphrase);
headers.insert("Content-Type", "application/json");
```

### Signature Formula

```rust
let prehash = format!("{}{}{}{}", timestamp, method, path, body);
let signature = Base64(HMAC-SHA256(prehash, secret_key));
```

### Rate Limits Summary

| Category | Limit | Basis |
|----------|-------|-------|
| Public REST | 20/2s | IP |
| Private REST | Varies | User ID |
| Trading | 60/2s | Instrument |
| Sub-account | 1,000/2s | User ID |
| WebSocket Sub | 3/s | Connection |
| WebSocket Ops | 480/hour | Connection |

### Symbol Examples

| Type | Format | Example |
|------|--------|---------|
| SPOT | `BASE-QUOTE` | `BTC-USDT` |
| SWAP | `BASE-QUOTE-SWAP` | `BTC-USDT-SWAP` |
| FUTURES | `BASE-QUOTE-YYMMDD` | `BTC-USD-240329` |

### Response Structure

```json
{
  "code": "0",
  "msg": "",
  "data": [{ ... }]
}
```

---

## Implementation Checklist

### Module Structure (Following KuCoin Reference)

```
okx/
├── mod.rs          # Exports
├── endpoints.rs    # URLs, endpoint enum, symbol formatting
├── auth.rs         # Signature implementation
├── parser.rs       # JSON parsing
├── connector.rs    # Trait implementations
└── websocket.rs    # WebSocket (optional)
```

### endpoints.rs
- [ ] Define base URLs (REST, WebSocket)
- [ ] Create `Endpoint` enum for all API endpoints
- [ ] Implement `to_okx_symbol()` for symbol formatting
- [ ] Add endpoint path helpers

### auth.rs
- [ ] Implement HMAC SHA256 signature generation
- [ ] Generate ISO 8601 timestamp
- [ ] Create pre-hash string builder
- [ ] Add header construction functions
- [ ] WebSocket login signature (if implementing WS)

### parser.rs
- [ ] Parse standard response structure (`code`, `msg`, `data`)
- [ ] Parse ticker response
- [ ] Parse order book response (array format)
- [ ] Parse candlestick response (array format)
- [ ] Parse order response (`ordId`, `clOrdId`, `sCode`, `sMsg`)
- [ ] Parse balance response
- [ ] Parse positions response
- [ ] Handle error responses

### connector.rs

**MarketData Trait:**
- [ ] `get_price` - `GET /api/v5/market/ticker`
- [ ] `get_orderbook` - `GET /api/v5/market/books`
- [ ] `get_klines` - `GET /api/v5/market/candles`
- [ ] `get_ticker` - `GET /api/v5/market/ticker`
- [ ] `ping` - `GET /api/v5/public/time`

**Trading Trait:**
- [ ] `market_order` - `POST /api/v5/trade/order` (ordType: market)
- [ ] `limit_order` - `POST /api/v5/trade/order` (ordType: limit)
- [ ] `cancel_order` - `POST /api/v5/trade/cancel-order`
- [ ] `get_order` - `GET /api/v5/trade/order`
- [ ] `get_open_orders` - `GET /api/v5/trade/orders-pending`

**Account Trait:**
- [ ] `get_balance` - `GET /api/v5/account/balance`
- [ ] `get_account_info` - `GET /api/v5/account/config`

**Positions Trait:**
- [ ] `get_positions` - `GET /api/v5/account/positions`
- [ ] `get_funding_rate` - `GET /api/v5/public/funding-rate`
- [ ] `set_leverage` - `POST /api/v5/account/set-leverage`

---

## Key Implementation Notes

### 1. Symbol Formatting
OKX uses hyphens: `BTC-USDT`, `BTC-USDT-SWAP`

### 2. Trade Mode Parameter
SPOT orders require `tdMode: "cash"`, margin uses `"cross"` or `"isolated"`

### 3. Array Responses
Order book and candlestick data use **arrays** not objects:
- Order book: `[price, size, deprecated, amount]`
- Candle: `[timestamp, open, high, low, close, vol, volCcy, volCcyQuote, confirm]`

### 4. Timestamp Format
- **REST Authentication:** ISO 8601 with milliseconds (`2020-12-08T09:08:57.715Z`)
- **Response Data:** Milliseconds since epoch (`1672841403093`)

### 5. Success Code
Success code is **string** `"0"`, not integer `0`

### 6. Error Handling
Check both:
- Top-level `code` field
- Individual `sCode` field (batch operations)

### 7. Rate Limiting
- Implement client-side rate limiting
- Place/amend/cancel have **independent** limits
- WebSocket and REST share trading rate limits

### 8. Decimal Precision
Prices and sizes are **strings** to preserve precision: `"43250.5"`

---

## Testing Strategy

### Unit Tests
- [ ] Symbol formatting (spot, swap, futures)
- [ ] Signature generation (GET, POST)
- [ ] Response parsing (success, error)
- [ ] Error code handling

### Integration Tests
- [ ] Public endpoints (no auth)
- [ ] Authenticated endpoints (with test API key)
- [ ] Order placement and cancellation
- [ ] Rate limit handling

### Edge Cases
- [ ] Empty order book
- [ ] Partial order fills
- [ ] Invalid symbols
- [ ] Expired timestamps
- [ ] Rate limit errors (50011, 50061)

---

## References

### Official Documentation
- [OKX API v5 Docs](https://www.okx.com/docs-v5/en/)
- [REST API Reference](https://www.okx.com/docs-v5/en/#rest-api)
- [WebSocket API Reference](https://www.okx.com/docs-v5/en/#websocket-api)

### Code Examples
- KuCoin V5 Implementation: `v5/exchanges/kucoin/`
- Binance V4 Implementation: `v4/exchanges/binance/`

---

## Common Pitfalls

### 1. Timestamp Expiration
Requests expire **30 seconds** after timestamp. Sync system clock or query `/api/v5/public/time` first.

### 2. GET Request Body
GET requests should **never** include body in signature. Query parameters go in `requestPath`.

### 3. Pre-hash Order
Pre-hash string order is: `timestamp + method + path + body` (NO separators)

### 4. Signature Encoding
Use **Base64** encoding, not hex encoding

### 5. Empty Data Array
Even single-object responses wrap data in array: `"data": [{ ... }]`

### 6. Instrument vs User ID Limits
Trading endpoints use **instrument-level** limits, account endpoints use **User ID** limits

### 7. SPOT vs MARGIN
Same symbol (`BTC-USDT`) for both; differentiated by `tdMode` parameter

### 8. Leverage Independence
Leverage for `BTC-USD-SWAP` and `BTC-USD-FUTURES` are **separate** settings

---

## Support and Resources

### Getting Help
- Official Telegram: OKX API Support
- Email: support@okx.com
- API Status: https://www.okx.com/status

### Rate Limit Monitoring
- Track 50011 and 50061 error frequency
- Implement exponential backoff
- Use batch endpoints to reduce request count

### Security
- Never commit API keys to git
- Use IP whitelisting
- Minimal permissions (read/trade only, never withdraw)
- Rotate keys regularly

---

## Summary

This research provides everything needed to implement a complete OKX V5 connector following the KuCoin reference architecture. All endpoints, authentication, response formats, symbols, rate limits, and WebSocket details are documented with examples.

**Next Steps:**
1. Review KuCoin V5 implementation structure
2. Create `okx/` module following the pattern
3. Implement endpoints.rs with symbol formatting
4. Implement auth.rs with HMAC SHA256 signing
5. Implement parser.rs for response structures
6. Implement connector.rs with trait methods
7. Test with OKX demo environment
8. Verify with production API (read-only operations first)

All documentation includes Rust code examples and references official OKX API v5 specifications from 2026.
