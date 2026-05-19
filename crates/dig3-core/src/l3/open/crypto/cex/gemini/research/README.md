# Gemini Exchange API Research - Complete Documentation

Research documentation for implementing Gemini V5 connector following KuCoin reference architecture.

---

## Documentation Index

| Document | Description | Key Topics |
|----------|-------------|------------|
| **[endpoints.md](./endpoints.md)** | All REST API endpoints | MarketData, Trading, Account, Positions traits |
| **[authentication.md](./authentication.md)** | Auth implementation | HMAC-SHA384, headers, nonce, signing |
| **[response_formats.md](./response_formats.md)** | JSON response structures | Parser implementation, data types |
| **[symbols.md](./symbols.md)** | Symbol formatting | btcusd format, normalization, validation |
| **[rate_limits.md](./rate_limits.md)** | Rate limiting | 120/600 req/min, throttling, retry logic |
| **[websocket.md](./websocket.md)** | WebSocket streams | Market data v2, order events |

---

## Quick Reference

### Base URLs

```
Production:  https://api.gemini.com
Sandbox:     https://api.sandbox.gemini.com
WS Market:   wss://api.gemini.com/v2/marketdata
WS Orders:   wss://api.gemini.com/v1/order/events
```

### Authentication (REST)

```rust
// Required headers
X-GEMINI-APIKEY:     {api_key}
X-GEMINI-PAYLOAD:    {base64(json_payload)}
X-GEMINI-SIGNATURE:  {hex(hmac_sha384(base64_payload, secret))}
Content-Type:        text/plain
Content-Length:      0
```

### Symbol Format

```
Spot:        btcusd, ethusd, ethbtc (lowercase, no separator)
Perpetuals:  btcgusdperp, ethgusdperp (ends with "perp")
```

### Rate Limits

```
Public:   120 req/min  (max 1 req/sec recommended)
Private:  600 req/min  (max 5 req/sec recommended)
Burst:    5 additional requests queued
Error:    429 Too Many Requests
```

---

## Implementation Checklist

### Module Structure (following V5 pattern)

```
exchanges/gemini/
├── mod.rs              - Exports
├── endpoints.rs        - URL builders, symbol formatting
├── auth.rs             - HMAC-SHA384 signing
├── parser.rs           - JSON parsing, response types
├── connector.rs        - Trait implementations
└── websocket.rs        - WebSocket streams (optional)
```

### Core Components

#### 1. endpoints.rs

**Responsibilities**:
- [ ] Symbol normalization (to lowercase)
- [ ] Endpoint URL builders
- [ ] Symbol validation helpers
- [ ] Instrument type detection (spot vs perpetual)

**Key Functions**:
```rust
pub fn format_symbol(symbol: &str) -> String;
pub fn ticker_url(symbol: &str) -> String;
pub fn orderbook_url(symbol: &str) -> String;
pub fn candles_url(symbol: &str, timeframe: &str) -> String;
pub fn new_order_url() -> String;
pub fn balances_url() -> String;
pub fn positions_url() -> String;
```

#### 2. auth.rs

**Responsibilities**:
- [ ] Nonce generation (millisecond timestamps)
- [ ] JSON payload creation
- [ ] Base64 encoding
- [ ] HMAC-SHA384 signature
- [ ] Header construction

**Key Functions**:
```rust
pub fn generate_nonce() -> u64;
pub fn sign_request(endpoint: &str, params: HashMap<String, String>, secret: &str) -> HashMap<String, String>;
pub fn sign_websocket_request(endpoint: &str, secret: &str) -> HashMap<String, String>;
```

**Dependencies**:
```toml
hmac = "0.12"
sha2 = "0.10"
base64 = "0.21"
hex = "0.4"
```

#### 3. parser.rs

**Responsibilities**:
- [ ] JSON deserialization
- [ ] Error response handling
- [ ] Type conversions (string -> Decimal for prices/amounts)
- [ ] Response structures for all endpoints

**Key Structures**:
```rust
pub struct Ticker { ... }
pub struct OrderBook { ... }
pub struct Trade { ... }
pub struct Candle { ... }
pub struct Order { ... }
pub struct Balance { ... }
pub struct Position { ... }
pub struct ApiError { result: String, reason: String, message: String }
```

#### 4. connector.rs

**Responsibilities**:
- [ ] Implement MarketData trait
- [ ] Implement Trading trait
- [ ] Implement Account trait
- [ ] Implement Positions trait
- [ ] Rate limiting (120/600 req/min)
- [ ] Error handling

**Trait Methods** (see endpoints.md for complete list):

**MarketData**:
- `get_symbols()`
- `get_ticker(symbol)`
- `get_orderbook(symbol, depth)`
- `get_trades(symbol, limit)`
- `get_candles(symbol, interval, limit)`

**Trading**:
- `create_order(symbol, side, order_type, amount, price)`
- `cancel_order(order_id)`
- `cancel_all_orders()`
- `get_order_status(order_id)`
- `get_active_orders()`
- `get_order_history(symbol, limit)`

**Account**:
- `get_balances()`
- `get_deposit_address(currency)`
- `withdraw(currency, amount, address)`
- `get_deposit_history()`
- `get_withdrawal_history()`

**Positions**:
- `get_positions()`
- `get_position(symbol)`
- `get_margin_info()`
- `get_funding_payments()`

#### 5. websocket.rs (Optional)

**Responsibilities**:
- [ ] Market data v2 connection
- [ ] Order events connection (authenticated)
- [ ] Subscription management
- [ ] Event parsing
- [ ] Reconnection logic

**Key Functions**:
```rust
pub async fn connect_market_data() -> Result<WebSocketStream>;
pub async fn subscribe_orderbook(symbols: Vec<&str>) -> Result<()>;
pub async fn connect_order_events(api_key: &str, api_secret: &str) -> Result<WebSocketStream>;
```

---

## Critical Implementation Details

### 1. Symbol Handling

**Always normalize to lowercase**:
```rust
let symbol = user_input.to_lowercase(); // "BTCUSD" -> "btcusd"
```

**Check instrument type**:
```rust
let is_perpetual = symbol.ends_with("perp");
```

### 2. Authentication

**Payload structure**:
```json
{
  "request": "/v1/order/new",
  "nonce": 1640000000000,
  "symbol": "btcusd",
  "amount": "0.5",
  "price": "50000.00",
  "side": "buy",
  "type": "exchange limit"
}
```

**Nonce requirements**:
- Strictly increasing
- Millisecond precision recommended
- Must be within ±30 seconds of server time

**Signature**:
```rust
let payload_str = serde_json::to_string(&payload)?;
let b64_payload = BASE64.encode(payload_str);
let mut mac = HmacSha384::new_from_slice(secret.as_bytes())?;
mac.update(b64_payload.as_bytes());
let signature = hex::encode(mac.finalize().into_bytes());
```

### 3. Response Parsing

**Check for errors first**:
```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Success(T),
    Error(ApiError),
}

match response.json::<ApiResponse<Ticker>>().await? {
    ApiResponse::Success(ticker) => Ok(ticker),
    ApiResponse::Error(err) => Err(ExchangeError::Api(err.message)),
}
```

**Preserve precision**:
```rust
#[derive(Deserialize)]
pub struct Order {
    #[serde(deserialize_with = "string_to_decimal")]
    pub price: Decimal,

    #[serde(deserialize_with = "string_to_decimal")]
    pub amount: Decimal,
}
```

### 4. Rate Limiting

**Implement per-endpoint tracking**:
```rust
pub struct GeminiConnector {
    public_limiter: RateLimiter,    // 120/min
    private_limiter: RateLimiter,   // 600/min
}

// Before each request
self.private_limiter.throttle().await;
let response = self.client.post(url).headers(headers).send().await?;

// Handle 429
if response.status() == 429 {
    // Exponential backoff retry
}
```

### 5. Error Handling

**Common errors**:
- `InvalidSignature` - Check auth implementation
- `InvalidNonce` - Ensure nonce is increasing
- `InsufficientFunds` - Account balance too low
- `RateLimitExceeded` (429) - Implement backoff
- `InvalidSymbol` - Validate symbol exists

---

## Testing Strategy

### 1. Unit Tests

**Symbol formatting**:
```rust
#[test]
fn test_symbol_normalization() {
    assert_eq!(format_symbol("BTCUSD"), "btcusd");
    assert_eq!(format_symbol("btcusd"), "btcusd");
}
```

**Authentication**:
```rust
#[test]
fn test_signature_generation() {
    let payload = r#"{"request":"/v1/balances","nonce":1234567890}"#;
    let secret = "test-secret";
    let signature = sign_payload(payload, secret).unwrap();
    assert_eq!(signature.len(), 96); // SHA384 = 48 bytes * 2 (hex)
}
```

**Parsing**:
```rust
#[test]
fn test_parse_ticker() {
    let json = r#"{"bid":"50000.00","ask":"50001.00","last":"50000.50"}"#;
    let ticker: Ticker = serde_json::from_str(json).unwrap();
    assert_eq!(ticker.bid, Decimal::from_str("50000.00").unwrap());
}
```

### 2. Integration Tests

**Sandbox environment**:
- Base URL: `https://api.sandbox.gemini.com`
- Create sandbox API keys at sandbox.gemini.com
- Same rate limits as production

**Test flow**:
1. Get symbols (public)
2. Get ticker (public)
3. Get balances (private)
4. Create and cancel test order (private)
5. Get order status (private)

### 3. WebSocket Tests

**Market data**:
```rust
#[tokio::test]
async fn test_market_data_subscription() {
    let mut ws = connect_market_data().await.unwrap();
    subscribe_orderbook(&mut ws, vec!["BTCUSD"]).await.unwrap();

    // Wait for subscription confirmation
    let msg = ws.next().await.unwrap().unwrap();
    // Assert subscription_ack received
}
```

**Order events**:
```rust
#[tokio::test]
async fn test_order_events_connection() {
    let ws = connect_order_events(API_KEY, API_SECRET).await.unwrap();
    // Wait for subscription_ack
}
```

---

## Common Pitfalls

### 1. ❌ Using uppercase symbols in requests

```rust
// WRONG
let url = format!("/v1/pubticker/BTCUSD");

// CORRECT
let url = format!("/v1/pubticker/{}", symbol.to_lowercase());
```

### 2. ❌ Parsing numbers as Decimal directly

```rust
// WRONG - JSON has strings
#[derive(Deserialize)]
struct Order {
    price: Decimal, // Won't deserialize from "50000.00"
}

// CORRECT
#[derive(Deserialize)]
struct Order {
    #[serde(deserialize_with = "string_to_decimal")]
    price: Decimal,
}
```

### 3. ❌ Forgetting Content-Type header

```rust
// WRONG - missing Content-Type
headers.insert("X-GEMINI-APIKEY", api_key);
headers.insert("X-GEMINI-PAYLOAD", payload);
headers.insert("X-GEMINI-SIGNATURE", signature);

// CORRECT
headers.insert("X-GEMINI-APIKEY", api_key);
headers.insert("X-GEMINI-PAYLOAD", payload);
headers.insert("X-GEMINI-SIGNATURE", signature);
headers.insert("Content-Type", "text/plain");
headers.insert("Content-Length", "0");
```

### 4. ❌ Not handling 429 errors

```rust
// WRONG
let response = client.get(url).send().await?;
let data = response.json().await?;

// CORRECT
let response = client.get(url).send().await?;
if response.status() == 429 {
    return Err(ExchangeError::RateLimit);
}
let data = response.json().await?;
```

### 5. ❌ Reusing nonce

```rust
// WRONG - static nonce
let nonce = 1640000000000;

// CORRECT - always increasing
let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as u64;
```

---

## Differences from KuCoin Reference

While following the V5 architecture pattern from KuCoin, note these Gemini-specific differences:

| Aspect | KuCoin | Gemini |
|--------|--------|--------|
| **Symbol Format** | BTC-USDT | btcusd (lowercase, no dash) |
| **Auth Algorithm** | HMAC-SHA256 | HMAC-SHA384 |
| **Auth Headers** | KC-API-* | X-GEMINI-* |
| **Payload Location** | Query/Body | Always in header (base64) |
| **Nonce Name** | timestamp | nonce |
| **Rate Limit** | Headers provided | Client-side tracking |
| **WebSocket Auth** | Token-based | Same as REST (headers) |
| **Perpetuals** | Symbol suffix | "gusdperp" suffix |
| **Order Types** | limit, market, etc. | "exchange limit", "exchange market" |

---

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Cryptography
hmac = "0.12"
sha2 = "0.10"
base64 = "0.21"
hex = "0.4"

# Decimal handling
rust_decimal = "1.32"

# WebSocket (optional)
tokio-tungstenite = "0.20"

# Error handling
thiserror = "1.0"

# Utilities
chrono = "0.4"
```

---

## Next Steps

### Implementation Order

1. **Phase 1: Core Infrastructure**
   - [ ] Create module structure (mod.rs)
   - [ ] Implement endpoints.rs (symbol formatting, URL builders)
   - [ ] Implement auth.rs (signing logic)
   - [ ] Write unit tests for auth and endpoints

2. **Phase 2: MarketData Trait**
   - [ ] Implement parser.rs (public endpoint responses)
   - [ ] Implement MarketData trait in connector.rs
   - [ ] Add rate limiter for public endpoints
   - [ ] Integration tests with sandbox

3. **Phase 3: Trading Trait**
   - [ ] Add order-related parsers
   - [ ] Implement Trading trait
   - [ ] Add rate limiter for private endpoints
   - [ ] Test order creation/cancellation

4. **Phase 4: Account & Positions**
   - [ ] Add balance/position parsers
   - [ ] Implement Account trait
   - [ ] Implement Positions trait
   - [ ] Complete integration tests

5. **Phase 5: WebSocket (Optional)**
   - [ ] Implement market data WebSocket
   - [ ] Implement order events WebSocket
   - [ ] Add reconnection logic
   - [ ] WebSocket integration tests

### Validation

Run `cargo check` after each phase:
```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo check
```

### Testing Checklist

- [ ] All public endpoints return valid data
- [ ] Authentication works (balances endpoint)
- [ ] Order creation/cancellation works
- [ ] Rate limiting prevents 429 errors
- [ ] Error responses parsed correctly
- [ ] WebSocket connections stable
- [ ] Reconnection logic works
- [ ] All tests pass in sandbox

---

## Support & References

### Official Documentation

- **Main Docs**: https://docs.gemini.com
- **REST API**: https://docs.gemini.com/rest/market-data
- **WebSocket**: https://docs.gemini.com/websocket-api
- **Authentication**: https://docs.gemini.com/authentication/api-key
- **Sandbox**: https://sandbox.gemini.com

### Community Resources

- **Support Email**: trading@gemini.com
- **API Status**: https://status.gemini.com

### Internal References

- **V5 Architecture**: See `v5/exchanges/kucoin/` for reference implementation
- **V4 Binance**: See `v4/exchanges/binance/` for alternative pattern
- **Traits**: See `v5/traits/` for trait definitions

---

## Summary

This research provides complete API specifications for implementing the Gemini V5 connector:

✅ **All endpoints documented** (MarketData, Trading, Account, Positions)
✅ **Authentication fully specified** (HMAC-SHA384, headers, nonce)
✅ **Response formats detailed** (JSON structures, parsers)
✅ **Symbol handling explained** (btcusd format, normalization)
✅ **Rate limits defined** (120/600 req/min, throttling)
✅ **WebSocket streams covered** (Market data v2, Order events)

**Ready for implementation following KuCoin reference architecture.**

---

**Research completed**: 2026-01-20
**Documentation version**: 1.0
**Status**: Ready for implementation
