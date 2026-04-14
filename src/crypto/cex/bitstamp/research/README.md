# Bitstamp Exchange API Research

Complete research documentation for implementing the Bitstamp V5 connector.

**Research Date**: 2026-01-20

**API Version**: v2

**Base URL**: `https://www.bitstamp.net`

**WebSocket URL**: `wss://ws.bitstamp.net`

---

## Documentation Structure

### 1. [endpoints.md](./endpoints.md)
Complete catalog of all REST API endpoints organized by trait:
- **MarketData Trait**: Public endpoints (ticker, order book, OHLC, etc.)
- **Trading Trait**: Order placement, cancellation, and management
- **Account Trait**: Balances, transactions, fees, deposits, withdrawals

### 2. [authentication.md](./authentication.md)
Authentication mechanisms and signature generation:
- **V2 Authentication** (recommended): HMAC-SHA256 with comprehensive string-to-sign
- **Legacy Authentication**: Form-based signature (backward compatibility)
- Code examples and implementation notes
- Error handling and common issues

### 3. [response_formats.md](./response_formats.md)
JSON response structures for all endpoints:
- Market data responses (ticker, order book, trades, OHLC)
- Trading responses (order creation, status, cancellation)
- Account responses (balances, transactions, fees)
- WebSocket message formats
- Error response formats

### 4. [symbols.md](./symbols.md)
Trading pair symbol format and conventions:
- Symbol format: lowercase, no separators (e.g., `btcusd`)
- Symbol parsing and validation
- Converting between formats (standard vs Bitstamp)
- List of common trading pairs
- Currency codes and trading pair information

### 5. [rate_limits.md](./rate_limits.md)
Rate limiting policies and best practices:
- **Limits**: 400 requests/second, 10,000 requests/10 minutes
- Rate limit error handling (HTTP 429, error code 400.002)
- Best practices: WebSocket for real-time data, request spacing, caching
- Implementation strategies and monitoring

### 6. [websocket.md](./websocket.md)
WebSocket API for real-time data streaming:
- **Endpoint**: `wss://ws.bitstamp.net`
- **Channels**: `live_trades_{pair}`, `order_book_{pair}`, `diff_order_book_{pair}`
- Subscription/unsubscription messages
- Order book management with differential updates
- Connection management and reconnection logic
- Implementation examples

---

## Quick Reference

### API Endpoints Summary

**Public Endpoints** (GET):
- `/api/v2/ticker/{pair}/` - Ticker data
- `/api/v2/order_book/{pair}/` - Order book
- `/api/v2/transactions/{pair}/` - Recent trades
- `/api/v2/ohlc/{pair}/` - OHLC candlestick data
- `/api/v2/markets/` - Trading pairs info

**Trading Endpoints** (POST, authenticated):
- `/api/v2/buy/{pair}/` - Buy limit order
- `/api/v2/sell/{pair}/` - Sell limit order
- `/api/v2/buy/market/{pair}/` - Buy market order
- `/api/v2/sell/market/{pair}/` - Sell market order
- `/api/v2/cancel_order/` - Cancel order
- `/api/v2/order_status/` - Order status
- `/api/v2/open_orders/all/` - All open orders

**Account Endpoints** (POST, authenticated):
- `/api/v2/account_balances/` - All balances
- `/api/v2/user_transactions/` - Transaction history
- `/api/v2/fees/trading/` - Trading fees
- `/api/v2/fees/withdrawal/` - Withdrawal fees

### Authentication Quick Start

**V2 Method** (recommended):

Headers:
```
X-Auth: BITSTAMP {api_key}
X-Auth-Signature: {hmac_sha256_hex_uppercase}
X-Auth-Nonce: {uuid_v4}
X-Auth-Timestamp: {unix_timestamp_millis}
X-Auth-Version: v2
```

String to sign:
```
BITSTAMP {api_key}
{method}
{host}
{path}
{query}
{content_type}
{nonce}
{timestamp}
{version}
{body}
```

Signature: `HMAC-SHA256(api_secret, string_to_sign)` as uppercase hex

### Rate Limits Quick Reference

- **400 requests/second** (burst)
- **10,000 requests/10 minutes** (sustained)
- Error code: `400.002` or HTTP 429
- Use WebSocket to avoid REST limits

### Symbol Format Quick Reference

- **Format**: Lowercase, no separator
- **Examples**: `btcusd`, `etheur`, `xrpbtc`
- **API Usage**: `/api/v2/ticker/btcusd/`
- **Display**: `BTC/USD` (with slash, uppercase)

### WebSocket Quick Reference

- **URL**: `wss://ws.bitstamp.net`
- **Subscribe**: `{"event":"bts:subscribe","data":{"channel":"live_trades_btcusd"}}`
- **Channels**:
  - `live_trades_{pair}` - Real-time trades
  - `order_book_{pair}` - Full order book snapshots
  - `diff_order_book_{pair}` - Differential order book updates

---

## Implementation Checklist

### Phase 1: Core Infrastructure
- [ ] Implement V2 authentication (HMAC-SHA256)
- [ ] Implement rate limiter (400/s, 10k/10min)
- [ ] Implement HTTP client with proper headers
- [ ] Implement error handling (429, API errors)
- [ ] Implement symbol normalization (format conversion)

### Phase 2: MarketData Trait
- [ ] `get_ticker()` - `/api/v2/ticker/{pair}/`
- [ ] `get_order_book()` - `/api/v2/order_book/{pair}/`
- [ ] `get_recent_trades()` - `/api/v2/transactions/{pair}/`
- [ ] `get_klines()` - `/api/v2/ohlc/{pair}/`
- [ ] `get_markets()` - `/api/v2/markets/`

### Phase 3: Trading Trait
- [ ] `create_order()` - Buy/sell limit/market orders
- [ ] `cancel_order()` - `/api/v2/cancel_order/`
- [ ] `get_order_status()` - `/api/v2/order_status/`
- [ ] `get_open_orders()` - `/api/v2/open_orders/all/`

### Phase 4: Account Trait
- [ ] `get_balances()` - `/api/v2/account_balances/`
- [ ] `get_account_info()` - Account information
- [ ] `get_trade_history()` - `/api/v2/user_transactions/`
- [ ] `get_fees()` - `/api/v2/fees/trading/`

### Phase 5: WebSocket (Optional)
- [ ] WebSocket connection management
- [ ] Subscribe to channels
- [ ] Handle trade events
- [ ] Handle order book updates (differential)
- [ ] Reconnection logic with backoff

---

## Key Differences from Other Exchanges

### Bitstamp-Specific Features

1. **V2 Authentication**: Uses comprehensive string-to-sign (includes method, path, query, body)
2. **Symbol Format**: Pure lowercase, no separators (unlike `BTC-USD` or `BTC_USD`)
3. **Decimal Strings**: All numbers returned as strings for precision
4. **No Positions API**: Bitstamp is spot-only (no futures/derivatives in standard API)
5. **Cached Endpoints**: Open orders cached for 10 seconds
6. **WebSocket No Snapshot**: `diff_order_book` requires REST API for initial snapshot

### Similarities to Other Exchanges

1. **Rate Limits**: Similar to other exchanges (burst + sustained limits)
2. **WebSocket**: Standard JSON-based WebSocket protocol
3. **Error Codes**: Standard HTTP status codes + custom error codes
4. **REST + WebSocket**: Hybrid approach (REST for operations, WS for data)

---

## Common Pitfalls

### Authentication
- **String-to-sign construction**: Easy to get wrong (newlines, content-type handling)
- **Timestamp in milliseconds**: Not seconds
- **Signature uppercase**: Must be uppercase hex
- **Nonce uniqueness**: Must be unique for each request (use UUID)

### Symbol Format
- **No separators**: Don't use `BTC-USD` or `BTC_USD`
- **Always lowercase**: Don't use `BTCUSD`
- **Symbol parsing**: Can't reliably split without markets list

### Rate Limits
- **Shared limits**: Public and private endpoints share the same pool
- **No headers**: Bitstamp doesn't return rate limit info in headers
- **WebSocket preferred**: Use WebSocket for real-time data

### Order Book
- **No initial snapshot**: `diff_order_book` requires REST fetch first
- **Amount zero means delete**: Amount "0.00000000" removes price level
- **String decimals**: All prices/amounts are strings

---

## Reference Links

### Official Documentation
- **Main API**: https://www.bitstamp.net/api/
- **WebSocket v2**: https://www.bitstamp.net/websocket/v2/
- **Markets**: https://www.bitstamp.net/markets/
- **Support**: support@bitstamp.net

### Third-Party Resources
- **CCXT Bitstamp**: https://github.com/ccxt/ccxt/blob/master/python/ccxt/bitstamp.py
- **Node.js Client**: https://github.com/krystianity/node-bitstamp

### Research Sources
- [Bitstamp API Documentation](https://www.bitstamp.net/api/)
- [WebSocket API v2](https://www.bitstamp.net/websocket/v2/)
- [CCXT Implementation](https://github.com/ccxt/ccxt)
- [Bitstamp FAQ](https://www.bitstamp.net/faq/)

---

## Implementation Reference

For implementation, follow the V5 connector pattern established in:
- `v5/exchanges/kucoin/` (reference implementation)

### Module Structure
```
exchanges/bitstamp/
├── mod.rs           # Module exports
├── endpoints.rs     # Endpoint URLs, enum, symbol formatting
├── auth.rs          # V2 signature implementation
├── parser.rs        # JSON response parsing
├── connector.rs     # Trait implementations
└── websocket.rs     # WebSocket client (optional)
```

---

## Notes

- **Bitstamp is spot-only**: No futures/perpetual contracts in standard API
- **No testnet**: Use sandbox environment: `https://sandbox.bitstamp.net`
- **Customer ID not needed**: V2 authentication doesn't use customer ID
- **WebSocket channels**: Channel names include the pair (e.g., `live_trades_btcusd`)
- **API v1 deprecated**: Only use v2 endpoints for new implementations

---

## Research Status

- [x] Endpoints documentation
- [x] Authentication methods
- [x] Response formats
- [x] Symbol format
- [x] Rate limits
- [x] WebSocket API

**Status**: Complete - Ready for implementation

**Next Steps**: Begin V5 connector implementation following KuCoin reference pattern.
