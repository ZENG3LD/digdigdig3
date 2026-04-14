# Bithumb Exchange API Research

## Overview

This directory contains comprehensive research documentation for implementing a Bithumb V5 connector. Bithumb operates **two distinct platforms** with different APIs:

1. **Bithumb Korea** - Main Korean exchange (KRW markets)
   - Base URL: `https://api.bithumb.com`
   - Primary quote: KRW (Korean Won)
   - Symbol format: `BTC_KRW` (underscore)

2. **Bithumb Pro** - Global exchange (USDT markets)
   - Base URL: `https://global-openapi.bithumb.pro`
   - Primary quote: USDT
   - Symbol format: `BTC-USDT` (hyphen)

---

## Documentation Files

### 1. [endpoints.md](./endpoints.md)
Complete API endpoint reference for both platforms covering:
- **MarketData Trait**: Ticker, order book, trades, OHLCV, server time
- **Trading Trait**: Create/cancel orders, query orders, order history
- **Account Trait**: Balance, deposits, withdrawals, account info

**Key Differences**:
- Korea uses parameter-based API (`order_currency`, `payment_currency`)
- Pro uses unified symbol in query params
- Different response structures and status codes

### 2. [authentication.md](./authentication.md)
Authentication mechanisms for both platforms:

**Bithumb Korea**:
- JWT-based authentication with HMAC-SHA256
- Requires: `access_key`, `nonce` (UUID), `timestamp`, `query_hash` (SHA512)
- Authorization header: `Bearer {jwt_token}`

**Bithumb Pro**:
- Parameter signing with HMAC-SHA256
- Sort params alphabetically, join with `&`, sign
- Signature must be lowercase
- Include in request body or query params

**Implementation**: Complete Rust code examples provided

### 3. [response_formats.md](./response_formats.md)
JSON response structures for all endpoints:

**Bithumb Korea Format**:
```json
{
  "status": "0000",
  "data": {...}
}
```

**Bithumb Pro Format**:
```json
{
  "code": "0",
  "success": true,
  "msg": "success",
  "data": {...},
  "params": []
}
```

Includes parsing guidelines, error codes, and Rust struct examples.

### 4. [symbols.md](./symbols.md)
Symbol format specifications and conversion:

**Korea Format**: `BTC_KRW` (underscore-separated)
- Separate parameters: `order_currency=BTC`, `payment_currency=KRW`

**Pro Format**: `BTC-USDT` (hyphen-separated)
- Unified symbol in requests

**Implementation**: Symbol converter, validation, precision handling

### 5. [rate_limits.md](./rate_limits.md)
Rate limiting policies and implementation:

**Bithumb Korea**:
- Public WebSocket: 5 req/s, 100 req/min (documented)
- REST: ~10 req/s (conservative estimate)

**Bithumb Pro**:
- Trading: 10 req/s (documented)
- Public: ~20 req/s (conservative)
- Private: ~10 req/s (conservative)

**Implementation**: Token bucket algorithm, weight-based limiting, retry logic

### 6. [websocket.md](./websocket.md)
WebSocket API for real-time data:

**Bithumb Pro** (well-documented):
- URL: `wss://global-api.bithumb.pro/message/realtime`
- Public topics: TICKER, ORDERBOOK, TRADE
- Private topics: ORDER, CONTRACT_ORDER, CONTRACT_ASSET, CONTRACT_POSITION
- Heartbeat: Ping every 30 seconds
- Order book: Snapshot (code 00006) + Updates (code 00007)

**Bithumb Korea**: Limited documentation (use REST or Pro WebSocket)

---

## Implementation Recommendations

### Platform Choice

**Option 1: Bithumb Korea Only**
- Pros: Direct access to KRW markets, high liquidity for Korean users
- Cons: Limited documentation, JWT complexity, KRW-only

**Option 2: Bithumb Pro Only**
- Pros: Well-documented, USDT markets, WebSocket support, international
- Cons: Less liquidity than Korea platform

**Option 3: Both Platforms** (Recommended)
- Support both with abstraction layer
- Use unified internal symbol format (`BTC/KRW`, `BTC/USDT`)
- Convert symbols based on target platform

### Architecture

Follow V5 connector pattern (see KuCoin reference):

```
exchanges/bithumb/
├── mod.rs              # Exports and platform enum
├── endpoints.rs        # URL builders, endpoint enums
├── auth.rs             # JWT + parameter signing
├── parser.rs           # JSON parsing for both formats
├── connector.rs        # Trait implementations
└── websocket.rs        # WebSocket client
```

### Key Implementation Steps

1. **Symbol Management**
   - Unified format internally (`BTC/KRW`)
   - Convert to platform-specific on API calls
   - Validation and precision handling

2. **Authentication**
   - JWT generator for Korea (with SHA512 query hash)
   - Parameter signer for Pro (with alphabetical sorting)
   - Secure credential storage

3. **Response Parsing**
   - Generic response wrapper for both formats
   - Status code checking
   - Error mapping to `ExchangeError`

4. **Rate Limiting**
   - Separate limiters for public/private/trading
   - Token bucket implementation
   - Exponential backoff on 429

5. **WebSocket**
   - Focus on Bithumb Pro (well-documented)
   - Order book management with snapshots/updates
   - Heartbeat every 30 seconds
   - Reconnection with backoff

---

## Testing Checklist

### MarketData Trait
- [ ] Get ticker (single symbol)
- [ ] Get ticker (all symbols)
- [ ] Get order book
- [ ] Get recent trades
- [ ] Get OHLCV/candlestick data
- [ ] Get server time
- [ ] Get exchange info

### Trading Trait
- [ ] Create limit order
- [ ] Create market order
- [ ] Cancel order
- [ ] Query order details
- [ ] Query open orders
- [ ] Query order history

### Account Trait
- [ ] Get balance
- [ ] Get account info
- [ ] Get deposit address
- [ ] Get deposit history
- [ ] Get withdrawal history
- [ ] Withdraw (with caution on testnet if available)

### WebSocket
- [ ] Connect and authenticate
- [ ] Subscribe to public topics (ticker, orderbook, trades)
- [ ] Subscribe to private topics (orders, balances)
- [ ] Maintain order book with updates
- [ ] Heartbeat/ping-pong
- [ ] Reconnection on disconnect

### Error Handling
- [ ] Invalid symbol
- [ ] Invalid credentials
- [ ] Rate limit exceeded
- [ ] Insufficient balance
- [ ] Order not found
- [ ] Network errors

---

## Reference Links

### Official Documentation

**Bithumb Pro**:
- REST API: https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/rest-api.md
- WebSocket: https://github.com/bithumb-pro/bithumb.pro-official-api-docs/blob/master/ws-api.md
- GitHub: https://github.com/bithumb-pro/bithumb.pro-official-api-docs

**Bithumb Korea**:
- API Docs: https://apidocs.bithumb.com/
- Authentication: https://apidocs.bithumb.com/docs/인증-헤더-만들기

**Bithumb Futures**:
- Documentation: https://bithumbfutures.github.io/bithumb-futures-api-doc/

### Third-Party References

**CCXT Implementation**:
- Python: https://github.com/ccxt/ccxt/blob/master/python/ccxt/async_support/bithumb.py
- Useful for endpoint discovery and response formats

---

## Quick Start

### 1. Read Documentation
```bash
# Read in order:
cat endpoints.md          # Understand available endpoints
cat authentication.md     # Learn auth methods
cat symbols.md            # Symbol format conversion
cat response_formats.md   # Parse responses
cat rate_limits.md        # Avoid rate limits
cat websocket.md          # Real-time data
```

### 2. Create Module Structure
```bash
cd zengeld-terminal/crates/connectors/crates/v5/src/exchanges/
mkdir -p bithumb
cd bithumb
touch mod.rs endpoints.rs auth.rs parser.rs connector.rs websocket.rs
```

### 3. Implement Core Components
```rust
// mod.rs
pub mod endpoints;
pub mod auth;
pub mod parser;
pub mod connector;
pub mod websocket;

pub use connector::BithumbConnector;

pub enum Platform {
    Korea,
    Pro,
}
```

### 4. Test with Public Endpoints
```rust
// Start with unauthenticated market data
let connector = BithumbConnector::new(Platform::Pro, None);
let ticker = connector.get_ticker("BTC/USDT").await?;
println!("BTC price: {}", ticker.last_price);
```

### 5. Add Authentication
```rust
// Add credentials for private endpoints
let connector = BithumbConnector::new(
    Platform::Korea,
    Some(("api_key".to_string(), "secret_key".to_string()))
);
let balance = connector.get_balance().await?;
```

---

## Common Pitfalls

### 1. Symbol Format Confusion
**Problem**: Using wrong separator (underscore vs hyphen)
**Solution**: Use symbol converter, validate before API calls

### 2. Authentication Errors
**Problem**: JWT signature mismatch or wrong parameter order
**Solution**:
- Korea: Ensure SHA512 for query_hash, HS256 for JWT
- Pro: Sort params alphabetically, lowercase signature

### 3. Rate Limiting
**Problem**: Getting 429 errors
**Solution**: Implement token bucket, use WebSocket for real-time data

### 4. Order Book Management
**Problem**: Missing updates or stale data
**Solution**: Track version numbers, handle snapshots (00006) vs updates (00007)

### 5. Precision Errors
**Problem**: Order rejection due to wrong decimal places
**Solution**: Use precision helpers, round based on symbol config

---

## Support and Issues

For questions or issues:
1. Check this documentation first
2. Review official API docs (links above)
3. Check CCXT implementation for reference
4. Test with small amounts first
5. Use conservative rate limits initially

---

## Version History

**Version 1.0** (2026-01-20)
- Initial research documentation
- Coverage: Both Bithumb Korea and Bithumb Pro
- Complete endpoint mapping
- Authentication methods
- WebSocket implementation guide
- Rate limiting strategies

---

## Next Steps

1. **Review KuCoin V5 Implementation** for reference architecture
2. **Implement Symbol Converter** as first component
3. **Create Authentication Module** (both JWT and parameter signing)
4. **Build Response Parser** with error handling
5. **Implement MarketData Trait** (public endpoints first)
6. **Add Rate Limiting** before implementing Trading
7. **Implement Trading Trait** with careful testing
8. **Add WebSocket Support** for real-time data
9. **Write Integration Tests** for all components
10. **Document Usage Examples** and edge cases

---

## License and Disclaimer

This documentation is for educational and development purposes. Always:
- Test with small amounts
- Use API keys with restricted permissions
- Implement proper error handling
- Follow exchange Terms of Service
- Monitor API usage and rate limits

Trading cryptocurrencies involves risk. This connector is provided as-is without warranty.
