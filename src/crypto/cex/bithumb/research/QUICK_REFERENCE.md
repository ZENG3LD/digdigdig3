# Bithumb API Quick Reference

## Two Platforms

| Feature | Bithumb Korea | Bithumb Pro |
|---------|---------------|-------------|
| **Base URL** | `https://api.bithumb.com` | `https://global-openapi.bithumb.pro/openapi/v1` |
| **Quote Currency** | KRW (Korean Won) | USDT, BTC |
| **Symbol Format** | `BTC_KRW` (underscore) | `BTC-USDT` (hyphen) |
| **Auth Method** | JWT with HMAC-SHA256 | Parameter signing HMAC-SHA256 |
| **Response Status** | `"0000"` = success | `"0"` or `< 10000` = success |
| **Documentation** | Limited | Well-documented |

---

## Authentication

### Bithumb Korea (JWT)
```rust
// 1. Create payload
let payload = {
    access_key: "API_KEY",
    nonce: uuid_v4(),
    timestamp: now_ms(),
    query_hash: sha512(query_string),  // if params exist
    query_hash_alg: "SHA512"
};

// 2. Sign with HS256
let jwt = sign(payload, secret_key, HS256);

// 3. Add header
Authorization: Bearer {jwt}
```

### Bithumb Pro (Parameter Signing)
```rust
// 1. Add apiKey and timestamp to params
params.insert("apiKey", api_key);
params.insert("timestamp", now_ms());

// 2. Sort alphabetically and join
let sig_string = params.sorted().join("&");
// "apiKey=X&price=50000&quantity=1&symbol=BTC-USDT&timestamp=123"

// 3. Sign with HMAC-SHA256 (lowercase)
let signature = hmac_sha256(sig_string, secret_key).to_lowercase();
params.insert("signature", signature);
```

---

## Key Endpoints

### Market Data (Public)

| Endpoint | Korea | Pro |
|----------|-------|-----|
| **Ticker** | `GET /public/ticker/BTC_KRW` | `GET /spot/ticker?symbol=BTC-USDT` |
| **Order Book** | `GET /public/orderbook/BTC_KRW` | `GET /spot/orderBook?symbol=BTC-USDT` |
| **Trades** | `GET /public/transaction_history/BTC_KRW` | `GET /spot/trades?symbol=BTC-USDT` |
| **OHLCV** | `GET /public/candlestick/BTC_KRW/24h` | `GET /spot/kline?symbol=BTC-USDT&type=m1` |

### Trading (Private)

| Endpoint | Korea | Pro |
|----------|-------|-----|
| **Create Order** | `POST /trade/place` | `POST /spot/placeOrder` |
| **Cancel Order** | `POST /trade/cancel` | `POST /spot/cancelOrder` |
| **Open Orders** | `POST /info/orders` | `POST /spot/openOrders` |
| **Order Detail** | `POST /info/order_detail` | `POST /spot/orderDetail` |

### Account (Private)

| Endpoint | Korea | Pro |
|----------|-------|-----|
| **Balance** | `POST /info/balance` | `POST /spot/account` |
| **Withdraw** | `POST /trade/btc_withdrawal` | `POST /withdraw` |

---

## Rate Limits

| Type | Korea | Pro |
|------|-------|-----|
| **Public REST** | ~10 req/s | ~20 req/s |
| **Private REST** | ~5 req/s | ~10 req/s |
| **Trading** | ~5 req/s | **10 req/s** (documented) |
| **WebSocket** | **5 req/s, 100 req/min** | No specific limit |

---

## WebSocket (Bithumb Pro)

**URL**: `wss://global-api.bithumb.pro/message/realtime`

### Public Topics
- `TICKER:BTC-USDT` - Price updates
- `ORDERBOOK:BTC-USDT` - Order book changes
- `TRADE:BTC-USDT` - Recent trades

### Private Topics (requires auth)
- `ORDER` - Order updates
- `CONTRACT_ORDER` - Futures order updates
- `CONTRACT_ASSET` - Balance changes
- `CONTRACT_POSITION` - Position updates

### Commands
```json
{"cmd": "subscribe", "args": ["TICKER:BTC-USDT"]}
{"cmd": "unSubscribe", "args": ["TICKER:BTC-USDT"]}
{"cmd": "ping", "args": []}  // Every 30 seconds
{"cmd": "authKey", "args": ["apiKey", "timestamp", "signature"]}
```

### Response Codes
- `00006` - Snapshot (full orderbook)
- `00007` - Update (incremental)
- `00000` - Auth success
- `0` - Pong
- `10000+` - Errors

---

## Response Formats

### Korea
```json
{
  "status": "0000",
  "data": {
    "opening_price": "50000000",
    "closing_price": "51000000",
    ...
  }
}
```

### Pro
```json
{
  "code": "0",
  "success": true,
  "msg": "success",
  "data": {
    "c": "51000.00",
    "h": "52000.00",
    ...
  },
  "params": []
}
```

---

## Symbol Conversion

```rust
// Internal unified format
let unified = "BTC/KRW";

// Convert to Korea
let korea = unified.replace("/", "_");  // "BTC_KRW"

// Convert to Pro
let pro = unified.replace("/", "-");    // "BTC-KRW"

// Extract parameters (Korea)
let (order_currency, payment_currency) = korea.split_once("_").unwrap();
// ("BTC", "KRW")
```

---

## Error Codes

### Korea
| Code | Meaning |
|------|---------|
| `0000` | Success |
| `5300` | Invalid API Key |
| `5500` | Invalid Parameter |
| `5600` | Maintenance |

### Pro
| Code | Meaning |
|------|---------|
| `0` | Success |
| `10005` | Invalid API Key |
| `10006` | Invalid Signature |
| `10008` | Rate Limit Exceeded |
| `20001` | Insufficient Balance |

---

## Implementation Order

1. **Symbol Converter** - Handle both formats
2. **Authentication** - Both JWT and parameter signing
3. **Response Parser** - Parse both response formats
4. **Rate Limiter** - Token bucket algorithm
5. **MarketData Trait** - Public endpoints first
6. **Trading Trait** - With careful testing
7. **Account Trait** - Balance and account info
8. **WebSocket** - Real-time data (Pro only)

---

## Critical Notes

1. **Two Platforms**: Korea (KRW) vs Pro (USDT) - different APIs entirely
2. **Symbol Format**: Underscore vs Hyphen - validate before calling
3. **Authentication**: Different methods - implement both
4. **Rate Limits**: Trading 10 req/s (Pro) - use token bucket
5. **WebSocket**: Pro only documented - ping every 30 seconds
6. **Order Book**: Version tracking - handle snapshots (00006) and updates (00007)
7. **Precision**: KRW prices are integers, USDT has decimals
8. **Testing**: Start with public endpoints, small amounts for trading

---

## Files in This Directory

1. **README.md** - Overview and implementation guide
2. **endpoints.md** - Complete endpoint reference
3. **authentication.md** - Auth methods with code examples
4. **response_formats.md** - JSON structures and parsing
5. **symbols.md** - Symbol formats and conversion
6. **rate_limits.md** - Rate limiting strategies
7. **websocket.md** - WebSocket implementation
8. **QUICK_REFERENCE.md** - This file

---

## Sources

- [Bithumb Pro REST API](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/rest-api.md)
- [Bithumb Pro WebSocket](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/ws-api.md)
- [Bithumb Korea API](https://apidocs.bithumb.com/)
- [Bithumb Korea Auth](https://apidocs.bithumb.com/docs/인증-헤더-만들기)
- [CCXT Bithumb](https://github.com/ccxt/ccxt/blob/master/python/ccxt/async_support/bithumb.py)
