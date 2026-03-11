# Bitget API Research Documentation

Complete research documentation for implementing Bitget V5 connector following the KuCoin reference architecture.

## Documentation Files

### 1. [endpoints.md](./endpoints.md) - 21KB
Complete list of all API endpoints for V5 connector traits:

- **MarketData Trait**: Server time, symbols, tickers, order book, candles, trades
- **Trading Trait**: Place/cancel/modify orders, batch operations, order details
- **Account Trait**: Account info, balances, fees, transfers, bills
- **Positions Trait**: Get positions, historical positions, leverage/margin management

Includes both **Spot** and **Futures** endpoints with request/response examples.

### 2. [authentication.md](./authentication.md) - 11KB
HMAC SHA256 signature-based authentication:

- API key components (API Key, Secret Key, Passphrase)
- Signature generation process (step-by-step)
- Required headers (ACCESS-KEY, ACCESS-SIGN, ACCESS-TIMESTAMP, ACCESS-PASSPHRASE)
- Prehash string construction for GET/POST requests
- WebSocket authentication (different signature format)
- Python and Rust implementation examples
- Common authentication errors and troubleshooting
- Time synchronization requirements

### 3. [response_formats.md](./response.md) - 16KB
JSON response structure and formats:

- Standard response structure (`code`, `msg`, `requestTime`, `data`)
- Success response (code "00000")
- Error codes and HTTP status codes
- Market data response formats (tickers, order book, candles, trades)
- Trading response formats (orders, fills, batch operations)
- Account response formats (balances, fees, bills)
- Position response formats (single/all positions, historical)
- WebSocket message formats
- Pagination format (V2 APIs)

### 4. [symbols.md](./symbols.md) - 14KB
Symbol naming conventions and handling:

- **Spot format**: `{BASE}{QUOTE}_SPBL` (e.g., `BTCUSDT_SPBL`)
- **Futures formats**:
  - USDT-margined: `{BASE}{QUOTE}_UMCBL`
  - Coin-margined: `{BASE}{QUOTE}_DMCBL`
  - USDC-margined: `{BASE}{QUOTE}_CMCBL`
  - Simulated: `{BASE}{QUOTE}_SUMCBL`
- Product types (umcbl, dmcbl, cmcbl, sumcbl)
- API V1 vs V2 differences
- Symbol information endpoints
- Symbol fields (precision, limits, fees)
- Case sensitivity requirements
- Special symbol differences (LUNA/LUNA2, $ALT/ALT, MEMECOIN/MEME)
- Symbol parsing and construction examples
- WebSocket symbol format

### 5. [rate_limits.md](./rate_limits.md) - 16KB
Complete rate limiting information:

- **Global limits**: 6,000 requests/min per IP (5 min recovery)
- **Public endpoints**: 20 req/sec per IP
- **Trading endpoints**: 10 req/sec per UID
- **Account endpoints**: 10 req/sec per UID
- **Leverage/margin**: 5 req/sec per UID
- **Order limits**: 400 max orders (spot), 400 max orders (futures)
- **WebSocket limits**: 240 subs/hour, 1,000 channels max, 10 msg/sec
- Rate limiter implementation examples (Token Bucket, Sliding Window)
- Multi-tier rate limiting strategy
- Best practices (caching, batching, WebSocket preference)
- Error handling and retry strategies

### 6. [websocket.md](./websocket.md) - 18KB
WebSocket API for real-time data:

- **URLs**:
  - Public: `wss://ws.bitget.com/v2/ws/public`
  - Private: `wss://ws.bitget.com/v2/ws/private`
- Connection limits and flow
- Heartbeat mechanism (ping/pong every 30 sec)
- Authentication for private channels
- Subscription format and channel types
- **Public channels**: ticker, candles, order book, trades, funding rate
- **Private channels**: orders, fills, account, positions, plan orders
- Message actions (snapshot, update)
- Reconnection strategy
- Complete Rust implementation example
- Best practices

## Quick Reference

### Base URL
```
https://api.bitget.com
```

### Key Endpoints

**Spot Market Data:**
- GET `/api/spot/v1/public/products` - Get all symbols
- GET `/api/spot/v1/market/ticker?symbol=BTCUSDT_SPBL` - Get ticker
- GET `/api/spot/v1/market/depth?symbol=BTCUSDT_SPBL` - Order book
- GET `/api/spot/v1/market/candles?symbol=BTCUSDT_SPBL&period=1min` - Candles

**Spot Trading:**
- POST `/api/spot/v1/trade/orders` - Place order
- POST `/api/spot/v1/trade/cancel-order` - Cancel order
- GET `/api/spot/v1/trade/open-orders` - Get open orders

**Spot Account:**
- GET `/api/spot/v1/account/assets` - Get balances
- GET `/api/spot/v1/account/getInfo` - Account info
- POST `/api/spot/v1/wallet/transfer` - Transfer between accounts

**Futures Market Data:**
- GET `/api/mix/v1/market/contracts?productType=umcbl` - Get symbols
- GET `/api/mix/v1/market/ticker?symbol=BTCUSDT_UMCBL&productType=umcbl` - Ticker
- GET `/api/mix/v1/market/candles?symbol=BTCUSDT_UMCBL&productType=umcbl&granularity=1m` - Candles

**Futures Trading:**
- POST `/api/mix/v1/order/placeOrder` - Place order
- POST `/api/mix/v1/order/cancel-order` - Cancel order
- GET `/api/mix/v1/order/current` - Get open orders

**Futures Account:**
- GET `/api/mix/v1/account/accounts?productType=umcbl` - Get accounts
- POST `/api/mix/v1/account/setLeverage` - Set leverage
- POST `/api/mix/v1/account/setMarginMode` - Set margin mode

**Futures Positions:**
- GET `/api/mix/v1/position/allPosition?productType=umcbl` - All positions
- GET `/api/mix/v1/position/singlePosition?symbol=...&productType=umcbl` - Single position

### Authentication Headers

```
ACCESS-KEY: <api_key>
ACCESS-SIGN: <base64_hmac_sha256_signature>
ACCESS-TIMESTAMP: <timestamp_milliseconds>
ACCESS-PASSPHRASE: <passphrase>
Content-Type: application/json
```

### Response Structure

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": { ... }
}
```

### Rate Limits Summary

| Category | Limit | Unit |
|----------|-------|------|
| Overall IP | 6000/min | IP |
| Public Endpoints | 20/sec | IP |
| Trading | 10/sec | UID |
| Account | 10/sec | UID |
| Leverage/Margin | 5/sec | UID |
| Max Spot Orders | 400 total | UID |
| Max Futures Orders | 400 total | UID |

## Implementation Checklist

Follow this checklist when implementing the Bitget connector:

### Module Structure (follow KuCoin)
- [ ] `mod.rs` - Exports and module organization
- [ ] `endpoints.rs` - URL constants, endpoint enum, symbol formatting
- [ ] `auth.rs` - HMAC SHA256 signature implementation
- [ ] `parser.rs` - JSON parsing for all response types
- [ ] `connector.rs` - Trait implementations (MarketData, Trading, Account, Positions)
- [ ] `websocket.rs` - WebSocket implementation (optional)

### MarketData Trait
- [ ] `get_server_time()` - `/api/spot/v1/public/time`
- [ ] `get_symbols()` - Spot: `/api/spot/v1/public/products`, Futures: `/api/mix/v1/market/contracts`
- [ ] `get_ticker()` - Spot/Futures ticker endpoints
- [ ] `get_order_book()` - Depth endpoints
- [ ] `get_recent_trades()` - Fills/trades endpoints
- [ ] `get_klines()` - Candles endpoints with timeframe support

### Trading Trait
- [ ] `place_order()` - Spot: `/api/spot/v1/trade/orders`, Futures: `/api/mix/v1/order/placeOrder`
- [ ] `cancel_order()` - Cancel endpoints
- [ ] `get_order()` - Order details endpoints
- [ ] `get_open_orders()` - Open orders endpoints
- [ ] `get_order_history()` - Historical orders
- [ ] Batch operations (optional)

### Account Trait
- [ ] `get_account_info()` - `/api/spot/v1/account/getInfo`
- [ ] `get_balances()` - Spot: `/api/spot/v1/account/assets`, Futures: `/api/mix/v1/account/accounts`
- [ ] `get_fees()` - `/api/user/v1/fee/query`
- [ ] Transfer operations (optional)

### Positions Trait (Futures)
- [ ] `get_positions()` - `/api/mix/v1/position/allPosition`
- [ ] `get_position()` - `/api/mix/v1/position/singlePosition`
- [ ] `set_leverage()` - `/api/mix/v1/account/setLeverage`
- [ ] `set_margin_mode()` - `/api/mix/v1/account/setMarginMode`

### Additional Requirements
- [ ] Rate limiting implementation
- [ ] Error handling (parse error codes)
- [ ] Symbol validation and caching
- [ ] Proper precision handling (price/quantity)
- [ ] Time synchronization
- [ ] Unit tests
- [ ] `cargo check` passes

## Product Types Reference

When working with futures endpoints:

```rust
pub enum ProductType {
    UsdtFutures,  // "umcbl" - USDT-margined perpetual
    CoinFutures,  // "dmcbl" - Coin-margined perpetual
    UsdcFutures,  // "cmcbl" - USDC-margined perpetual
    Simulated,    // "sumcbl" - Demo/testnet
}

impl ProductType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::UsdtFutures => "umcbl",
            Self::CoinFutures => "dmcbl",
            Self::UsdcFutures => "cmcbl",
            Self::Simulated => "sumcbl",
        }
    }
}
```

## Common Errors

| Code | HTTP | Description | Solution |
|------|------|-------------|----------|
| 40001 | 400 | Invalid parameter | Validate parameters |
| 40015 | 401 | Invalid timestamp | Sync server time |
| 40016 | 401 | Invalid API key | Check API key |
| 40017 | 401 | Invalid passphrase | Check passphrase |
| 40018 | 401 | Invalid signature | Check signature generation |
| 40020 | 403 | Permission denied | Update API key permissions |
| 40808 | 400 | Parameter verification | Check parameter format |
| 43019 | 400 | Order does not exist | Verify order ID |
| 43115 | 400 | Insufficient balance | Check account balance |
| 429 | 429 | Too many requests | Implement rate limiting |

## Testing

Use Bitget's demo trading environment for testing:
- WebSocket Public: `wss://wspap.bitget.com/v2/ws/public`
- WebSocket Private: `wss://wspap.bitget.com/v2/ws/private`

## Additional Resources

- **Official API Docs**: https://www.bitget.com/api-doc/common/intro
- **Spot API Reference**: https://bitgetlimited.github.io/apidoc/en/spot/
- **Futures API Reference**: https://bitgetlimited.github.io/apidoc/en/mix/
- **API Guide**: https://wundertrading.com/journal/en/learn/article/bitget-api

## Notes

- Bitget uses V1 and V2 APIs. This research focuses on V1 (widely used and stable)
- Always use uppercase for symbols
- Futures endpoints require `productType` parameter
- Rate limit recovery is 5 minutes after triggering IP limit
- WebSocket heartbeat is critical (30 sec ping required)
- Symbol suffixes differ between spot (_SPBL) and futures (_UMCBL, etc.)

---

**Research completed:** 2026-01-20
**Total documentation:** ~96 KB across 6 files
**Reference implementation:** `v5/exchanges/kucoin/`
