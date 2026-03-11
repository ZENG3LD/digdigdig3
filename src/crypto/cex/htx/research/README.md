# HTX (Huobi) Exchange API Research

Complete API documentation for implementing HTX V5 connector.

## Research Documents

### 1. [endpoints.md](./endpoints.md)
Complete REST API endpoint documentation covering:
- **MarketData Trait**: Server time, symbols, tickers, order book, trades, klines
- **Account Trait**: Account list, balances, asset valuation, ledger, transfers
- **Trading Trait**: Order placement, cancellation, queries, match results, fees
- **Positions Trait**: Margin positions (spot uses balances)
- **Wallet**: Deposits, withdrawals, addresses, quotas, history

**Key Points:**
- Base URL: `https://api.huobi.pro`
- All timestamps in milliseconds
- Prices/amounts as strings for precision
- Two response formats: V1 (status/data) and V2 (code/data)

### 2. [authentication.md](./authentication.md)
HMAC SHA256 signature authentication:
- Signature process (5 steps)
- Required parameters (AccessKeyId, SignatureMethod, SignatureVersion, Timestamp)
- GET and POST request examples
- WebSocket v2 authentication
- Common errors and troubleshooting
- Security best practices

**Key Points:**
- Signature version: 2 (REST), 2.1 (WebSocket)
- Timestamp format: `YYYY-MM-DDThh:mm:ss` (UTC)
- Valid window: ±5 minutes
- Parameters sorted in ASCII order
- HMAC SHA256 → Base64 → URL encode

### 3. [response_formats.md](./response_formats.md)
JSON response structure and data types:
- V1 format: `{status, ch, ts, data}`
- V2 format: `{code, message, data}`
- Error responses with err-code/err-msg
- All endpoint response examples
- Data type reference (string for decimals)
- Timestamp format (milliseconds)
- Pagination (cursor-based)

**Key Points:**
- Check `status` field ("ok" or "error")
- Parse numeric strings to Decimal
- Rate limit headers in all responses
- Order states: submitted, partial-filled, filled, canceled

### 4. [symbols.md](./symbols.md)
Symbol format and trading pair information:
- Format: lowercase, no separator (`btcusdt`)
- Symbol endpoint: `GET /v2/settings/common/symbols`
- Precision and limits (price, amount, min/max order value)
- Symbol validation and parsing
- Quote currencies: usdt, btc, eth, husd, usdc, ht
- Symbol discovery and filtering

**Key Points:**
- All lowercase: `btcusdt` not `BTCUSDT`
- No separators: `btcusdt` not `btc-usdt`
- V2 endpoint provides complete metadata
- Respect precision and order limits
- Cache symbol information

### 5. [rate_limits.md](./rate_limits.md)
Rate limiting rules and best practices:
- **UID-based**: Limits per user account across all API keys
- Public endpoints: 800/sec (IP-based)
- Private endpoints: 100/sec trading, 50/sec queries (UID-based)
- WebSocket: 10 connections per API key
- Response headers: X-HB-RateLimit-Requests-Remain

**Key Limits:**
- Public market data: 800 req/sec per IP
- Account queries: 100 req/sec per UID
- Order placement: 100 req/sec per UID
- Order queries: 50 req/sec per UID
- Wallet operations: 20 req/sec per UID
- WebSocket connections: 10 per API key

**Best Practices:**
- Implement client-side rate limiting
- Use exponential backoff on errors
- Prefer WebSocket for real-time data
- Batch operations when possible
- Monitor rate limit headers

### 6. [websocket.md](./websocket.md)
WebSocket API for real-time data:
- Three endpoints: market data, MBP feed, account/orders
- GZIP compression on all messages
- Heartbeat mechanism (ping/pong)
- Authentication for private channels
- Market channels: kline, depth, trade, ticker, bbo
- Account channels: orders, trade.clearing, accounts.update

**Key Points:**
- Market data: `wss://api.huobi.pro/ws`
- MBP feed: `wss://api.huobi.pro/feed`
- Account/orders: `wss://api.huobi.pro/ws/v2` (auth required)
- All messages GZIP compressed
- Ping/pong every 5s (v1) or 20s (v2)
- Track sequence numbers for MBP
- Max 10 connections per API key (v2)

## Implementation Checklist

### Phase 1: Core Structure
- [ ] Create module structure (mod.rs, endpoints.rs, auth.rs, parser.rs, connector.rs)
- [ ] Define endpoint enum
- [ ] Implement symbol formatting functions
- [ ] Create error types

### Phase 2: Authentication
- [ ] Implement HMAC SHA256 signature
- [ ] Build signed request function
- [ ] Handle timestamp formatting
- [ ] Parameter sorting (ASCII order)
- [ ] Base64 and URL encoding

### Phase 3: MarketData Trait
- [ ] get_server_time()
- [ ] get_symbols()
- [ ] get_ticker()
- [ ] get_order_book()
- [ ] get_recent_trades()
- [ ] get_klines()

### Phase 4: Account Trait
- [ ] get_account_balance()
- [ ] get_account_info()
- [ ] Helper: get_account_id() for spot account

### Phase 5: Trading Trait
- [ ] place_order()
- [ ] cancel_order()
- [ ] get_order()
- [ ] get_open_orders()
- [ ] get_order_history()

### Phase 6: Positions Trait
- [ ] Use account balances for spot
- [ ] Optional: Margin position endpoints

### Phase 7: WebSocket (Optional)
- [ ] Market data WebSocket
- [ ] GZIP decompression
- [ ] Ping/pong heartbeat
- [ ] Authentication for private channels

### Phase 8: Rate Limiting
- [ ] Implement rate limiter per endpoint category
- [ ] Parse rate limit headers
- [ ] Exponential backoff on errors

### Phase 9: Testing
- [ ] cargo check passes
- [ ] Test all trait methods
- [ ] Error handling tests
- [ ] Rate limit compliance

## API Characteristics

### Strengths
- Comprehensive REST API
- Two symbol endpoints (V1 legacy, V2 recommended)
- Good documentation at huobiapi.github.io
- GZIP compression for WebSocket efficiency
- MBP feed for HFT use cases

### Considerations
- UID-based rate limits (shared across API keys)
- Two response formats (V1 and V2)
- Requires account ID for most operations
- GZIP compression on WebSocket (extra processing)
- Timestamp must be within ±5 minutes

### Differences from Other Exchanges

| Feature | HTX | Binance | KuCoin |
|---------|-----|---------|--------|
| Symbol format | `btcusdt` | `BTCUSDT` | `BTC-USDT` |
| Auth method | HMAC SHA256 | HMAC SHA256 | HMAC SHA256 |
| Signature version | 2 (REST), 2.1 (WS) | N/A | 2 |
| Response format | Two formats (V1/V2) | Single format | Single format |
| Rate limit basis | UID | IP + UID | IP + UID |
| WebSocket compression | GZIP (required) | Optional | Optional |
| Timestamp window | ±5 minutes | ±5 seconds | ±5 seconds |

## Quick Reference

### Base URLs
```
REST: https://api.huobi.pro
WebSocket Market: wss://api.huobi.pro/ws
WebSocket MBP: wss://api.huobi.pro/feed
WebSocket Private: wss://api.huobi.pro/ws/v2
```

### Authentication Parameters
```
AccessKeyId: <api-key>
SignatureMethod: HmacSHA256
SignatureVersion: 2
Timestamp: 2023-01-20T12:34:56
Signature: <computed-signature>
```

### Common Error Codes
- `api-signature-not-valid`: Invalid signature
- `invalid-parameter`: Missing/invalid parameter
- `login-required`: Missing authentication
- `invalid-timestamp`: Timestamp out of range
- `api-request-too-frequent`: Rate limit exceeded
- `order-orderstate-error`: Invalid order state

### Symbol Format
```rust
format!("{}{}", base.to_lowercase(), quote.to_lowercase())
// Examples: btcusdt, ethbtc, bnbusdt
```

### Signature Computation
```rust
let pre_sign = format!("{}\n{}\n{}\n{}", method, host, path, sorted_params);
let signature = hmac_sha256(secret_key, pre_sign);
let signature_b64 = base64_encode(signature);
let signature_encoded = url_encode(signature_b64);
```

## Resources

### Official Documentation
- Main docs: https://huobiapi.github.io/docs/spot/v1/en/
- New API portal: https://www.htx.com/en-us/opend/newApiPages/
- System status: https://status.huobigroup.com/

### SDK References
- Python: https://github.com/HuobiRDCenter/huobi_Python
- C#: https://github.com/JKorf/HTX.Net
- Go: https://github.com/HuobiRDCenter/huobi_Golang

## Notes

1. HTX was formerly known as Huobi Global
2. API keys work across spot, futures, swap, and options
3. Sub-users share rate limits with parent account
4. IP whitelisting expires after 90 days without renewal
5. Use V2 endpoints for new implementations
6. WebSocket v1 deprecated, use v2 for private data
7. MBP feed requires sequence number tracking
8. All numeric values returned as strings

## Implementation Pattern

Follow KuCoin reference implementation in `v5/exchanges/kucoin/`:
1. endpoints.rs: URL constants, endpoint enum, symbol formatting
2. auth.rs: HMAC signature, signed request builder
3. parser.rs: JSON parsing, response structs
4. connector.rs: Trait implementations
5. mod.rs: Module exports

Reference: `zengeld-terminal/crates/connectors/crates/v5/src/exchanges/kucoin/`
