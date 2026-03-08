# Phemex V5 Connector Research

Complete API documentation for implementing Phemex exchange connector following V5 architecture.

## Overview

Phemex is a cryptocurrency derivatives and spot exchange offering:
- **Spot Trading**: Fiat and crypto pairs
- **Perpetual Futures**: Coin-margined (BTC, ETH) and USDT-margined contracts
- **Hedged Mode**: Separate long/short positions per symbol
- **High Leverage**: Up to 100x on some contracts

## Documentation Structure

| File | Description | Coverage |
|------|-------------|----------|
| [endpoints.md](./endpoints.md) | All REST API endpoints | MarketData, Trading, Account, Positions traits |
| [authentication.md](./authentication.md) | HMAC SHA256 authentication | REST & WebSocket auth, signature generation |
| [response_formats.md](./response_formats.md) | API response structures | JSON schemas, error codes, field formats |
| [symbols.md](./symbols.md) | Symbol formats & scaling | Ep/Er/Ev scaling, currency info, conversions |
| [rate_limits.md](./rate_limits.md) | Rate limiting system | Groups, weights, headers, best practices |
| [websocket.md](./websocket.md) | WebSocket API | Subscriptions, channels, authentication |

## Quick Reference

### Base URLs

```
Production REST:    https://api.phemex.com
VIP REST:           https://vapi.phemex.com
Testnet REST:       https://testnet-api.phemex.com

Production WS:      wss://phemex.com/ws
VIP WS:             wss://vapi.phemex.com/ws
Testnet WS:         wss://testnet.phemex.com/ws
```

### Symbol Formats

| Market | Format | Example |
|--------|--------|---------|
| Spot | `s{BASE}{QUOTE}` | `sBTCUSDT` |
| Contract (Coin) | `{BASE}{QUOTE}` | `BTCUSD` |
| Contract (USDT) | `u{BASE}{QUOTE}` | `uBTCUSD` |

### Authentication Headers

```
x-phemex-access-token:      <api_key>
x-phemex-request-expiry:    <unix_timestamp>
x-phemex-request-signature: <hmac_sha256_hex>
```

### Signature Formula

```
HMAC_SHA256(secret, path + query + expiry + body)
```

### Value Scaling

| Suffix | Type | Scale Factor | Example |
|--------|------|--------------|---------|
| `Ep` | Price | `priceScale` (4 or 8) | `priceEp = 87700000` → `8770.0` (scale=4) |
| `Er` | Ratio | `ratioScale` (8) | `leverageEr = 2000000` → `0.02` or 20x |
| `Ev` | Value | `valueScale` (4 or 8) | `balanceEv = 100000000` → `1.0 BTC` (scale=8) |

**Conversion:**
```rust
actual = scaled / 10^scale
scaled = (actual * 10^scale).round()
```

### Rate Limits

| Group | Capacity | Window |
|-------|----------|--------|
| Contract | 500 req | 1 min |
| SpotOrder | 500 req | 1 min |
| Others | 100 req | 1 min |
| IP Global | 5000 req | 5 min |

### Order Types

- `Limit` - Standard limit order
- `Market` - Market order
- `Stop` / `StopLimit` - Stop-loss orders
- `MarketIfTouched` / `LimitIfTouched` - Take-profit orders
- `MarketAsLimit` / `StopAsLimit` / `MarketIfTouchedAsLimit` - Hybrid orders

### Order Status

- `New` - Active order
- `PartiallyFilled` - Partially executed
- `Filled` - Completely executed
- `Canceled` - Canceled by user
- `Rejected` - Rejected by system
- `Triggered` - Conditional order triggered
- `Untriggered` - Conditional order pending

## Implementation Checklist

### Core Structure (following KuCoin V5 pattern)

- [ ] `mod.rs` - Module exports
- [ ] `endpoints.rs` - Endpoint URLs, enums, symbol formatting
- [ ] `auth.rs` - HMAC SHA256 signature implementation
- [ ] `parser.rs` - JSON response parsing
- [ ] `connector.rs` - Trait implementations
- [ ] `websocket.rs` - WebSocket client (optional)

### MarketData Trait

- [ ] `get_server_time()` → `GET /public/time`
- [ ] `get_symbols()` → `GET /public/products`
- [ ] `get_orderbook(symbol)` → `GET /md/orderbook`
- [ ] `get_recent_trades(symbol)` → `GET /md/trade`
- [ ] `get_ticker(symbol)` → `GET /md/ticker/24hr`
- [ ] `get_klines(symbol, interval)` → `GET /exchange/public/md/v2/kline`

### Trading Trait

**Spot:**
- [ ] `place_order()` → `POST /spot/orders`
- [ ] `amend_order()` → `PUT /spot/orders`
- [ ] `cancel_order()` → `DELETE /spot/orders`
- [ ] `cancel_all_orders()` → `DELETE /spot/orders/all`
- [ ] `get_open_orders()` → `GET /spot/orders/active`

**Contract:**
- [ ] `place_order()` → `POST /orders`
- [ ] `amend_order()` → `PUT /orders/replace`
- [ ] `cancel_order()` → `DELETE /orders`
- [ ] `cancel_all_orders()` → `DELETE /orders/all`
- [ ] `get_open_orders()` → `GET /orders/activeList`
- [ ] `get_closed_orders()` → `GET /exchange/order/list`
- [ ] `get_order()` → `GET /exchange/order`
- [ ] `get_trades()` → `GET /exchange/order/trade`

### Account Trait

- [ ] `get_balance()` (spot) → `GET /spot/wallets`
- [ ] `get_balance()` (contract) → `GET /accounts/accountPositions`
- [ ] `transfer()` → `POST /assets/transfer`
- [ ] `get_transfer_history()` → `GET /assets/transfer`

### Positions Trait

- [ ] `get_positions()` → `GET /accounts/accountPositions`
- [ ] `set_leverage()` → `PUT /positions/leverage`
- [ ] `set_risk_limit()` → `PUT /positions/riskLimit`
- [ ] `assign_balance()` → `POST /positions/assign`

### WebSocket (optional)

- [ ] Connection management with heartbeat
- [ ] Authentication for private channels
- [ ] Order book subscription
- [ ] Trades subscription
- [ ] Klines subscription
- [ ] AOP (Account-Order-Position) subscription
- [ ] Reconnection handling
- [ ] Sequence number tracking

## Key Implementation Notes

### 1. Value Scaling System

**Always fetch product information first:**
```rust
let products = api.get_products().await?;
let symbol_registry = SymbolRegistry::from_products(&products);
```

**Use registry for all conversions:**
```rust
let actual_price = symbol_registry.unscale_price("BTCUSD", price_ep)?;
let scaled_price = symbol_registry.scale_price("BTCUSD", 8770.0)?;
```

### 2. Symbol Validation

**Spot symbols must start with 's':**
```rust
fn normalize_spot(base: &str, quote: &str) -> String {
    format!("s{}{}", base.to_uppercase(), quote.to_uppercase())
}
```

**Contract symbols have no 's' prefix:**
```rust
fn normalize_contract(base: &str, quote: &str) -> String {
    format!("{}{}", base.to_uppercase(), quote.to_uppercase())
}
```

### 3. Leverage Handling

**Sign indicates margin mode:**
- Positive `leverageEr` = Isolated margin (e.g., 2000000 = 20x isolated)
- Zero/Negative `leverageEr` = Cross margin

```rust
fn set_leverage(leverage: u8, isolated: bool) -> i64 {
    let leverage_er = (leverage as f64 * 1e6) as i64;
    if isolated {
        leverage_er  // Positive
    } else {
        -leverage_er  // Negative for cross
    }
}
```

### 4. Rate Limit Tracking

**Parse response headers:**
```rust
fn update_rate_limits(headers: &HeaderMap) {
    for (name, value) in headers {
        if name.as_str().starts_with("x-ratelimit-") {
            // Store remaining capacity
        }
    }
}
```

**Implement adaptive throttling:**
```rust
if rate_limiter.should_throttle("CONTRACT", 0.9) {
    tokio::time::sleep(Duration::from_millis(1000)).await;
}
```

### 5. Error Handling

**Check bizError field:**
```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "bizError": 11001,  // TE_NO_ENOUGH_AVAILABLE_BALANCE
    ...
  }
}
```

**Handle HTTP 429:**
```rust
if response.status() == 429 {
    let retry_after = response.headers()
        .get("x-ratelimit-retry-after-contract")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60);

    tokio::time::sleep(Duration::from_secs(retry_after)).await;
}
```

### 6. Timestamp Handling

**Most timestamps are nanoseconds:**
```rust
fn timestamp_to_seconds(timestamp_ns: u64) -> u64 {
    timestamp_ns / 1_000_000_000
}
```

**Kline timestamps are seconds:**
```rust
// Kline response: [timestamp_seconds, interval, ...]
let timestamp_s = kline[0];
```

## Testing Strategy

### 1. Unit Tests

- Value scaling/unscaling functions
- Signature generation
- Symbol normalization
- Error parsing

### 2. Integration Tests

**Testnet credentials required:**
```rust
const TESTNET_URL: &str = "https://testnet-api.phemex.com";
const TESTNET_API_KEY: &str = env!("PHEMEX_TESTNET_KEY");
const TESTNET_SECRET: &str = env!("PHEMEX_TESTNET_SECRET");
```

**Test endpoints:**
- Public endpoints (no auth required)
- Account balance queries
- Order placement/cancellation
- Position queries

### 3. WebSocket Tests

- Connection establishment
- Authentication
- Subscription/unsubscription
- Heartbeat maintenance
- Reconnection handling

## Common Pitfalls

1. **Forgetting symbol prefix:**
   - Spot: `"BTCUSDT"` ✗ → `"sBTCUSDT"` ✓
   - Contract: `"sBTCUSD"` ✗ → `"BTCUSD"` ✓

2. **Not scaling values:**
   - Sending `8770.0` as `priceEp` ✗ → Send `87700000` ✓

3. **Wrong signature order:**
   - `expiry + path + query + body` ✗
   - `path + query + expiry + body` ✓

4. **Query string with '?':**
   - Signature includes `?symbol=BTC` ✗
   - Signature includes `symbol=BTC` ✓

5. **Leverage sign:**
   - `leverageEr = 20` for 20x ✗ → `leverageEr = 2000000` (positive) ✓

6. **Missing heartbeat:**
   - Connection drops after 30 seconds without ping

7. **Ignoring bizError:**
   - Checking only `code == 0` ✗ → Also check `data.bizError == 0` ✓

## Resources

### Official Documentation

- Main API Docs: https://phemex-docs.github.io/
- GitHub Repository: https://github.com/phemex/phemex-api-docs
- Spot API: https://github.com/phemex/phemex-api-docs/blob/master/Public-Spot-API-en.md
- Contract API: https://github.com/phemex/phemex-api-docs/blob/master/Public-Contract-API-en.md
- Error Codes: https://github.com/phemex/phemex-api-docs/blob/master/TradingErrorCode.md

### Support

- API Telegram: Phemex API
- VIP Access: VIP@phemex.com

### Reference Implementation

Follow the KuCoin V5 connector structure:
```
v5/src/exchanges/kucoin/
├── mod.rs
├── endpoints.rs
├── auth.rs
├── parser.rs
└── connector.rs
```

## Version History

- **2026-01-20**: Initial research compilation
- API version: Current production (2026)
- Documentation based on official Phemex API docs

## Notes

- Phemex uses integer scaling for all numeric values (prices, quantities, balances)
- Scale factors vary by symbol and must be fetched from `/public/products`
- Rate limits are enforced per user AND per IP
- WebSocket requires heartbeat every 5 seconds (recommended)
- VIP users get symbol-level rate limits via vapi.phemex.com
- Hedged mode requires `posSide` parameter for position operations
