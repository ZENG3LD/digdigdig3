# Kraken API Research - V5 Connector Implementation

Comprehensive research documentation for implementing the Kraken V5 connector following the KuCoin reference architecture.

## Research Documents

### 1. [endpoints.md](./endpoints.md)
Complete API endpoint documentation for all trait methods:

**Spot REST API** (`https://api.kraken.com`):
- **MarketData Trait**: get_price, get_orderbook, get_klines, get_ticker, ping
- **Trading Trait**: market_order, limit_order, cancel_order, get_order, get_open_orders
- **Account Trait**: get_balance, get_account_info

**Futures REST API** (`https://futures.kraken.com`):
- **Positions Trait**: get_positions, get_funding_rate, set_leverage
- **Futures Trading**: send_order, cancel_order, get_accounts

Includes:
- Base URLs for Spot and Futures (separate APIs)
- HTTP methods and endpoint paths
- Request parameters and response formats
- Endpoint mapping table for all traits

---

### 2. [authentication.md](./authentication.md)
Authentication mechanisms for all Kraken APIs:

**Spot REST Authentication**:
- HMAC-SHA512 signature generation
- API-Key and API-Sign headers
- Nonce requirements and handling
- Step-by-step Rust implementation

**Futures REST Authentication**:
- New authentication method (post-October 2025)
- APIKey and Authent headers

**WebSocket Authentication**:
- Spot: Token-based (via REST GetWebSocketsToken)
- Futures: Challenge-response signing
- Token expiry and reuse

Includes:
- Complete Rust code examples
- Error handling
- Security best practices

---

### 3. [response_formats.md](./response_formats.md)
Detailed JSON response structures:

**General Structure**:
- Spot: `{"error": [], "result": {}}`
- Futures: `{"result": "success", ...}`

**Error Messages**:
- Format: `<severity><category>: <description>`
- Common error codes and resolutions

**Endpoint Responses**:
- Get Ticker Information
- Get Order Book
- Get OHLC Data
- Add Order / Cancel Order
- Get Balance / Trade Balance
- Futures responses (positions, accounts, funding)

**WebSocket Messages**:
- Subscription acknowledgments
- Snapshot and update messages
- Ticker, book, trade formats

Includes:
- Complete JSON examples
- Field descriptions and data types
- Error message catalog

---

### 4. [symbols.md](./symbols.md)
Symbol naming conventions and translation:

**Format Variations**:
- Spot REST Request: `XBTUSD` (simplified)
- Spot REST Response: `XXBTZUSD` (full ISO)
- WebSocket v1: `XBT/USD` (with slash)
- WebSocket v2: `BTC/USD` (BTC instead of XBT!)
- Futures: `PI_XBTUSD` (with prefix)

**Prefix Conventions**:
- `X`: Cryptocurrencies (XBT, XETH, etc.)
- `Z`: Fiat currencies (ZUSD, ZEUR, etc.)

**Futures Product Types**:
- `PI_`: Perpetual Inverse
- `PF_`: Perpetual Forward/Linear
- `FI_`: Fixed maturity Inverse
- `FF_`: Fixed maturity Forward

Includes:
- Symbol translation functions
- AssetPairs endpoint usage
- Rust implementation examples
- Common pitfalls and solutions

---

### 5. [rate_limits.md](./rate_limits.md)
Comprehensive rate limiting documentation:

**Spot REST API**:
- Call counter mechanism (starts at 0)
- Tier-based limits (Starter: 15, Intermediate/Pro: 20)
- Decay rates by verification tier
- Endpoint-specific costs (Ledger: 2 points)

**Trading Engine Limits**:
- Separate points system for orders
- Per-pair limits
- Independent from REST counter

**Futures API**:
- 500 points per 10 seconds window

**WebSocket**:
- Connection limits
- Ping requirements (Futures: every 60 seconds)
- Subscription limits (Level 3: 200 symbols max)

Includes:
- Rust rate limiter implementation
- Exponential backoff examples
- Best practices and strategies
- Error handling

---

### 6. [websocket.md](./websocket.md)
Real-time WebSocket API documentation:

**Spot WebSocket v2** (`wss://ws.kraken.com/v2`):
- Public channels: ticker, book, trade, ohlc
- Private channels: executions, balances
- Token-based authentication
- Subscription and message formats

**Futures WebSocket** (`wss://futures.kraken.com/ws/v1`):
- Public feeds: ticker, book, trade
- Private feeds: fills, balances
- Challenge-response authentication
- Ping requirement (every 60 seconds)

**Order Book Maintenance**:
- Snapshot handling
- Update application with sequence numbers
- Checksum validation

Includes:
- Connection URLs
- Complete message examples
- Rust WebSocket client implementation
- Order book maintenance code
- Reconnection logic

---

## Key Findings

### API Architecture Differences

**Spot vs Futures**: Completely separate APIs
- Different base URLs
- Different authentication methods
- Different symbol formats
- Different response structures

**WebSocket Versions**:
- v2 recommended for new Spot implementations
- v1 is legacy but still supported
- Futures has separate WebSocket

---

## Implementation Notes for V5 Connector

### Module Structure (Follow KuCoin Pattern)

```
exchanges/kraken/
├── mod.rs          # Exports
├── endpoints.rs    # URLs, endpoint enum, symbol formatting
├── auth.rs         # HMAC-SHA512 signing for Spot, different for Futures
├── parser.rs       # JSON parsing (handle error/result structure)
├── connector.rs    # Trait implementations
└── websocket.rs    # WebSocket (optional)
```

### Critical Implementation Details

1. **Symbol Handling**:
   - Request: Accept simplified format (`XBTUSD`)
   - Response: Parse full format (`XXBTZUSD`)
   - Use AssetPairs endpoint to build mapping

2. **Authentication**:
   - Spot: HMAC-SHA512 with nonce
   - Futures: Different algorithm (check latest docs)
   - WebSocket: Token via REST for Spot, challenge-response for Futures

3. **Error Handling**:
   - Always check `error` array in Spot responses
   - Empty array = success
   - Parse error format: `<severity><category>: <description>`

4. **Rate Limiting**:
   - Implement local counter tracking
   - Ledger queries cost 2 points
   - Trading has separate limits

5. **Response Parsing**:
   - Prices/amounts are strings (preserve precision)
   - Timestamps vary: integers, floats, or ISO strings
   - Handle balance extensions (.F, .B, .T)

---

## API Quirks and Edge Cases

### Request vs Response Symbol Mismatch
```rust
// Request
GET /0/public/Ticker?pair=XBTUSD

// Response key
response["result"]["XXBTZUSD"] // Note the XX prefix
```

### WebSocket Symbol Differences
- v1: `XBT/USD`
- v2: `BTC/USD` (uses BTC not XBT!)

### Nonce Requirements
- Must strictly increase per API key
- Use milliseconds, not seconds
- Clock drift can cause issues

### Order Book Updates
- `qty: 0` means remove price level
- Track sequence numbers to detect gaps
- Validate checksums on Spot v2

---

## Testing Recommendations

1. **Authentication**: Test with Balance endpoint first
2. **Symbols**: Test XBTUSD/XXBTZUSD translation
3. **Rate Limits**: Test local counter decay
4. **WebSocket**: Test reconnection logic
5. **Order Book**: Verify checksum validation

---

## Source Links

All information sourced from official Kraken documentation:

- [Kraken API Center](https://docs.kraken.com/)
- [Spot REST API Documentation](https://docs.kraken.com/api/)
- [Spot REST Authentication](https://docs.kraken.com/api/docs/guides/spot-rest-auth/)
- [Spot REST Rate Limits](https://docs.kraken.com/api/docs/guides/spot-rest-ratelimits/)
- [Get Ticker Information](https://docs.kraken.com/api/docs/rest-api/get-ticker-information/)
- [Get Order Book](https://docs.kraken.com/api/docs/rest-api/get-order-book/)
- [Get OHLC Data](https://docs.kraken.com/api/docs/rest-api/get-ohlc-data/)
- [Add Order](https://docs.kraken.com/api/docs/rest-api/add-order/)
- [Cancel Order](https://docs.kraken.com/api/docs/rest-api/cancel-order/)
- [Get Account Balance](https://docs.kraken.com/api/docs/rest-api/get-account-balance/)
- [Get Open Orders](https://docs.kraken.com/api/docs/rest-api/get-open-orders/)
- [Query Orders Info](https://docs.kraken.com/api/docs/rest-api/get-orders-info/)
- [Futures Introduction](https://docs.kraken.com/api/docs/guides/futures-introduction/)
- [Futures REST API](https://docs.kraken.com/api/docs/guides/futures-rest/)
- [Get Open Positions](https://docs.kraken.com/api/docs/futures-api/trading/get-open-positions/)
- [Historical Funding Rates](https://docs.kraken.com/api/docs/futures-api/trading/historical-funding-rates/)
- [Set Leverage Setting](https://docs.kraken.com/api/docs/futures-api/trading/set-leverage-setting/)
- [Get Accounts (Futures)](https://docs.kraken.com/api/docs/futures-api/trading/get-accounts/)
- [Send Order (Futures)](https://docs.kraken.com/api/docs/futures-api/trading/send-order/)
- [WebSocket v2 Ticker](https://docs.kraken.com/api/docs/websocket-v2/ticker/)
- [WebSocket v2 Book](https://docs.kraken.com/api/docs/websocket-v2/book/)
- [WebSocket v2 Trade](https://docs.kraken.com/api/docs/websocket-v2/trade/)
- [Spot WebSocket Authentication](https://docs.kraken.com/api/docs/guides/spot-ws-auth/)
- [Futures WebSockets](https://docs.kraken.com/api/docs/guides/futures-websockets/)
- [Futures WebSocket Ticker](https://docs.kraken.com/api/docs/futures-api/websocket/ticker/)
- [Futures WebSocket Book](https://docs.kraken.com/api/docs/futures-api/websocket/book/)
- [API Symbols and Tickers](https://support.kraken.com/articles/360000920306-api-symbols-and-tickers)
- [Spot Error Messages](https://docs.kraken.com/api/docs/guides/spot-errors/)
- [API Error Messages](https://support.kraken.com/articles/360001491786-api-error-messages)

---

## Research Completion Summary

**Date**: 2026-01-20

**Status**: Complete

**Files Created**:
1. endpoints.md - All endpoint documentation
2. authentication.md - Authentication mechanisms
3. response_formats.md - JSON response structures
4. symbols.md - Symbol formats and translation
5. rate_limits.md - Rate limiting documentation
6. websocket.md - WebSocket API documentation

**Coverage**:
- All MarketData trait methods documented
- All Trading trait methods documented
- All Account trait methods documented
- All Positions trait methods documented (Futures)
- Authentication fully documented with code examples
- Rate limits comprehensively covered
- WebSocket APIs (Spot v1, v2, and Futures) documented

**Ready for Implementation**: Yes

The research is complete and comprehensive enough to begin implementing the Kraken V5 connector following the KuCoin reference architecture in `v5/exchanges/kucoin/`.
