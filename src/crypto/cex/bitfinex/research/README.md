# Bitfinex API v2 Research Summary

Complete research documentation for implementing Bitfinex exchange connector in the V5 architecture.

## Overview

Bitfinex API v2 provides comprehensive REST and WebSocket APIs for:
- Market data (tickers, orderbooks, trades, candles)
- Trading operations (orders, positions)
- Account management (wallets, balances)

**Base URLs**:
- Public REST: `https://api-pub.bitfinex.com/v2`
- Authenticated REST: `https://api.bitfinex.com/v2`
- Public WebSocket: `wss://api-pub.bitfinex.com/ws/2`
- Authenticated WebSocket: `wss://api.bitfinex.com/ws/2`

## Key Characteristics

### API Design
- **Array-based responses** (not JSON objects)
- **Symbol prefixes required**: `t` for trading pairs, `f` for funding
- **Case-sensitive**: All symbols must be UPPERCASE
- **Timestamps**: Milliseconds since Unix epoch (UTC)
- **Amount signs**: Positive = buy/bid, negative = sell/ask

### Authentication
- **Method**: HMAC-SHA384 signature
- **Headers**: `bfx-apikey`, `bfx-nonce`, `bfx-signature`
- **Nonce**: Microsecond timestamp, must be strictly increasing
- **Signature**: HMAC-SHA384(`/api/{path}{nonce}{body}`, api_secret)

### Rate Limits
- **REST Authenticated**: 90 requests/minute
- **REST Public**: 10-90 requests/minute (varies by endpoint)
- **WebSocket Authenticated**: 5 connections per 15 seconds
- **WebSocket Public**: 20 connections per minute
- **Channel Subscriptions**: 30 per connection (25 for public data)

## Documentation Files

### 1. [endpoints.md](./endpoints.md)
Complete endpoint reference for all traits:

**MarketData Trait**:
- Platform status: `GET /platform/status`
- Ticker: `GET /ticker/{symbol}`
- Order book: `GET /book/{symbol}/{precision}`
- Trades: `GET /trades/{symbol}/hist`
- Candles: `GET /candles/{candle}/{section}`
- Configuration: `GET /conf/pub:{action}:{object}:{detail}`

**Trading Trait**:
- Submit order: `POST /auth/w/order/submit`
- Cancel order: `POST /auth/w/order/cancel`
- Cancel multiple: `POST /auth/w/order/cancel/multi`
- Update order: `POST /auth/w/order/update`
- Retrieve orders: `POST /auth/r/orders`
- Order history: `POST /auth/r/orders/hist`

**Account Trait**:
- Wallets: `POST /auth/r/wallets`
- Trade history: `POST /auth/r/trades/hist`
- User info: `POST /auth/r/info/user`

**Positions Trait**:
- Retrieve positions: `POST /auth/r/positions`
- Position history: `POST /auth/r/positions/hist`

### 2. [authentication.md](./authentication.md)
Authentication implementation details:

**Headers Required**:
```
Content-Type: application/json
bfx-nonce: <microsecond_timestamp>
bfx-apikey: <api_key>
bfx-signature: <hmac_sha384_hex>
```

**Signature Algorithm**:
1. Generate nonce: `(epoch_ms * 1000).to_string()`
2. Create signature string: `/api/{path}{nonce}{json_body}`
3. Calculate HMAC-SHA384(signature_string, api_secret)
4. Encode as hexadecimal

**Rust Implementation Example**:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha384;

type HmacSha384 = Hmac<Sha384>;

fn generate_signature(
    api_path: &str,
    body: &str,
    api_secret: &str,
) -> Result<(String, String)> {
    let nonce = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() * 1000)
        .to_string();

    let signature_string = format!("/api/{}{}{}", api_path, nonce, body);

    let mut mac = HmacSha384::new_from_slice(api_secret.as_bytes())?;
    mac.update(signature_string.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    Ok((nonce, signature))
}
```

### 3. [response_formats.md](./response_formats.md)
Detailed response structures for all endpoints:

**Ticker Response** (10 fields):
```json
[BID, BID_SIZE, ASK, ASK_SIZE, DAILY_CHANGE, DAILY_CHANGE_RELATIVE,
 LAST_PRICE, VOLUME, HIGH, LOW]
```

**Order Book** (3 fields):
```json
[PRICE, COUNT, AMOUNT]
```

**Order Object** (32 fields):
```json
[ID, GID, CID, SYMBOL, MTS_CREATE, MTS_UPDATE, AMOUNT, AMOUNT_ORIG,
 TYPE, TYPE_PREV, MTS_TIF, ..., FLAGS, ORDER_STATUS, ..., META]
```

**Trade Object** (12 fields):
```json
[ID, SYMBOL, MTS, ORDER_ID, EXEC_AMOUNT, EXEC_PRICE, ORDER_TYPE,
 ORDER_PRICE, MAKER, FEE, FEE_CURRENCY, CID]
```

**Position Object** (18 fields):
```json
[SYMBOL, STATUS, AMOUNT, BASE_PRICE, FUNDING, FUNDING_TYPE, PL, PL_PERC,
 PRICE_LIQ, LEVERAGE, PLACEHOLDER, POSITION_ID, MTS_CREATE, MTS_UPDATE,
 TYPE, COLLATERAL, COLLATERAL_MIN, META]
```

**Error Format**:
```json
["error", ERROR_CODE, "error message"]
```

### 4. [symbols.md](./symbols.md)
Symbol format and conventions:

**Format Rules**:
- Trading pairs: `t[BASE][QUOTE]` (e.g., `tBTCUSD`, `tETHBTC`)
- Funding currencies: `f[CURRENCY]` (e.g., `fUSD`, `fBTC`)
- Derivatives: `t[BASE]F0:[QUOTE]F0` (e.g., `tBTCF0:USTF0`)
- **Must be UPPERCASE**

**Common Pairs**:
- `tBTCUSD` - Bitcoin vs US Dollar
- `tETHUSD` - Ethereum vs US Dollar
- `tETHBTC` - Ethereum vs Bitcoin
- `tLTCUSD` - Litecoin vs US Dollar

**Symbol Validation**:
```rust
fn is_valid_symbol(symbol: &str) -> bool {
    if symbol.len() < 4 {
        return false;
    }
    let first_char = symbol.chars().next().unwrap();
    if first_char != 't' && first_char != 'f' {
        return false;
    }
    symbol.chars().all(|c| c.is_uppercase() || c == ':' || c.is_numeric())
}
```

**Fetching Symbol List**:
```
GET /v2/conf/pub:list:pair:exchange
```

### 5. [rate_limits.md](./rate_limits.md)
Comprehensive rate limiting information:

**REST API**:
- Authenticated: 90 req/min
- Public: 10-90 req/min
- Penalty: 60-second IP block

**WebSocket**:
- Authenticated: 5 connections per 15 seconds
- Public: 20 connections per minute
- Subscriptions: 30 per connection

**Best Practices**:
1. Implement request tracking and throttling
2. Use WebSocket for real-time data
3. Batch requests where possible
4. Implement exponential backoff
5. Separate API keys for multiple clients

**Error Response**:
```json
{"error": "ERR_RATE_LIMIT"}
```

### 6. [websocket.md](./websocket.md)
WebSocket API implementation guide:

**Connection Flow**:
1. Connect to WebSocket URL
2. Receive info message
3. (Optional) Configure flags
4. Subscribe to channels
5. Receive data
6. Handle heartbeats

**Public Channels**:
- `ticker` - Real-time price updates
- `trades` - Trade feed
- `book` - Order book (with precision levels P0-P4, R0)
- `candles` - OHLC data (1m, 5m, 15m, 30m, 1h, 3h, 6h, 12h, 1D, 1W, 14D, 1M)
- `status` - Platform status

**Authentication**:
```json
{
  "event": "auth",
  "apiKey": "YOUR_API_KEY",
  "authSig": "SIGNATURE",
  "authNonce": "NONCE",
  "authPayload": "AUTH{nonce}",
  "dms": 4
}
```

**Account Updates** (Channel 0):
- `os` - Order snapshot
- `on` - Order new
- `ou` - Order update
- `oc` - Order cancel
- `te` - Trade executed
- `ws` - Wallet snapshot
- `wu` - Wallet update
- `ps` - Position snapshot

**Message Format**:
```json
[CHANNEL_ID, DATA]
```

**Heartbeat**:
```json
[CHANNEL_ID, "hb"]
```

## Implementation Roadmap

### Phase 1: Basic Structure
1. Create module structure following V5 pattern:
   - `mod.rs` - Exports
   - `endpoints.rs` - URL constants and endpoint enum
   - `auth.rs` - Signature generation
   - `parser.rs` - JSON parsing
   - `connector.rs` - Trait implementations

### Phase 2: MarketData Trait
1. Implement `get_ticker()` - Parse 10-field array response
2. Implement `get_orderbook()` - Handle precision levels, parse 3-field arrays
3. Implement `get_trades()` - Parse 4-field arrays
4. Implement `get_klines()` - Handle candle key format, parse 6-field arrays
5. Implement symbol formatting helpers

### Phase 3: Trading Trait
1. Implement `place_order()` - Build request body, parse 32-field order array
2. Implement `cancel_order()` - Handle both id and cid+date cancellation
3. Implement `get_order()` - Parse order array
4. Implement `get_open_orders()` - Parse array of orders
5. Handle order status mapping

### Phase 4: Account Trait
1. Implement `get_balance()` - Parse wallet arrays
2. Implement `get_account_info()` - Parse user info
3. Implement order history endpoint
4. Implement trade history endpoint

### Phase 5: Positions Trait
1. Implement `get_positions()` - Parse 18-field position arrays
2. Implement position history
3. Handle position status mapping

### Phase 6: WebSocket (Optional)
1. Implement WebSocket connection
2. Implement channel subscriptions
3. Implement authentication
4. Handle account updates
5. Implement reconnection logic

## Key Implementation Notes

### Symbol Handling
```rust
// Always add 't' prefix for trading pairs
fn format_symbol(pair: &str) -> String {
    format!("t{}", pair.to_uppercase())
}

// Remove prefix for display
fn parse_symbol(symbol: &str) -> String {
    symbol.trim_start_matches('t').to_string()
}
```

### Array Response Parsing
```rust
// Ticker response
fn parse_ticker(data: &[Value]) -> Result<Ticker> {
    Ok(Ticker {
        bid: data[0].as_f64().ok_or(ParseError)?,
        bid_size: data[1].as_f64().ok_or(ParseError)?,
        ask: data[2].as_f64().ok_or(ParseError)?,
        ask_size: data[3].as_f64().ok_or(ParseError)?,
        daily_change: data[4].as_f64().ok_or(ParseError)?,
        daily_change_perc: data[5].as_f64().ok_or(ParseError)?,
        last_price: data[6].as_f64().ok_or(ParseError)?,
        volume: data[7].as_f64().ok_or(ParseError)?,
        high: data[8].as_f64().ok_or(ParseError)?,
        low: data[9].as_f64().ok_or(ParseError)?,
    })
}
```

### Error Handling
```rust
fn parse_error(data: &Value) -> Option<ExchangeError> {
    if let Some(arr) = data.as_array() {
        if arr.len() >= 3 && arr[0].as_str() == Some("error") {
            return Some(ExchangeError::Api {
                code: arr[1].as_i64().unwrap_or(0),
                message: arr[2].as_str().unwrap_or("Unknown").to_string(),
            });
        }
    }
    None
}
```

### Authentication Headers
```rust
fn build_auth_headers(
    api_path: &str,
    body: &str,
    api_key: &str,
    api_secret: &str,
) -> Result<HashMap<String, String>> {
    let (nonce, signature) = generate_signature(api_path, body, api_secret)?;

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("bfx-nonce".to_string(), nonce);
    headers.insert("bfx-apikey".to_string(), api_key.to_string());
    headers.insert("bfx-signature".to_string(), signature);

    Ok(headers)
}
```

## Testing Strategy

### Unit Tests
1. Symbol formatting/validation
2. Signature generation
3. Response parsing for each endpoint
4. Error parsing

### Integration Tests
1. Public endpoints (no auth required)
2. Authenticated endpoints (requires test API keys)
3. WebSocket connections
4. Rate limit handling

### Test Symbols
- `tBTCUSD` - Most liquid pair
- `tETHUSD` - Second most liquid
- `tETHBTC` - Crypto-crypto pair

## Common Pitfalls

1. **Forgetting 't' prefix** - All trading pairs need `t` prefix
2. **Lowercase symbols** - Must be uppercase or API returns error 10020
3. **Array index errors** - Responses are arrays, not objects
4. **Nonce not increasing** - Must track and increment nonce
5. **Wrong signature format** - Must be `/api/{path}{nonce}{body}`
6. **Rate limiting** - Easy to exceed 90/min limit during testing
7. **Order amount signs** - Positive = buy, negative = sell
8. **Timestamp units** - Milliseconds, not seconds

## Dependencies

Required Rust crates:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
tokio = { version = "1", features = ["full"] }
```

## Sources

- [Bitfinex API Documentation](https://docs.bitfinex.com/docs/introduction)
- [REST API General](https://docs.bitfinex.com/docs/rest-general)
- [REST Public Endpoints](https://docs.bitfinex.com/docs/rest-public)
- [REST Authenticated Endpoints](https://docs.bitfinex.com/docs/rest-auth)
- [WebSocket General](https://docs.bitfinex.com/docs/ws-general)
- [WebSocket Public Channels](https://docs.bitfinex.com/docs/ws-public)
- [WebSocket Authenticated](https://docs.bitfinex.com/docs/ws-auth)
- [Requirements and Limitations](https://docs.bitfinex.com/docs/requirements-and-limitations)
- [Bitfinex API v2 REST Documentation](https://bitfinex.readthedocs.io/en/latest/restv2.html)
- [GitHub - bitfinex-api-go](https://github.com/bitfinexcom/bitfinex-api-go/blob/master/docs/rest_v2.md)

## Next Steps

1. Review KuCoin V5 implementation as reference pattern
2. Create module structure in `v5/src/exchanges/bitfinex/`
3. Implement `endpoints.rs` with URL constants
4. Implement `auth.rs` with HMAC-SHA384 signature
5. Implement `parser.rs` for array-based responses
6. Implement `connector.rs` with trait methods
7. Add comprehensive unit tests
8. Test against Bitfinex testnet/sandbox if available
9. Document any deviations from this research
10. Update this README with implementation notes
