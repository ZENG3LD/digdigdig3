# MEXC Exchange API Research

Complete research documentation for implementing MEXC V5 connector.

## Research Documents

### 1. [endpoints.md](./endpoints.md)
Comprehensive list of all REST API endpoints for:
- **MarketData Trait**: Ping, server time, exchange info, depth, trades, klines, tickers
- **Trading Trait**: Order placement, cancellation, query (single, batch, all)
- **Account Trait**: Account info, trade history, fees
- **Positions Trait**: Futures positions, leverage, margin (institutional only)
- **Wallet**: Deposits, withdrawals, transfers

**Key Findings:**
- Base URL Spot: `https://api.mexc.com`
- Base URL Futures: `https://contract.mexc.com`
- Symbol format: Spot uses `BTCUSDT`, Futures uses `BTC_USDT`
- Futures trading API limited to institutional users as of 2026

### 2. [authentication.md](./authentication.md)
Complete authentication specifications:
- **Method**: HMAC SHA256 signature
- **Required Headers**: `X-MEXC-APIKEY`, `Content-Type`
- **Required Parameters**: `timestamp`, `signature`, optional `recvWindow`
- **Signature Process**: `HMAC-SHA256(secretKey, queryString)`
- **Listen Keys**: For WebSocket user data streams
- **Time Validation**: Server validates timestamp within recvWindow (default 5000ms, max 60000ms)

**Code Examples**: Provided for Rust, JavaScript, Python, and command-line

### 3. [response_formats.md](./response_formats.md)
Detailed response structures for all endpoints:
- **Spot Responses**: Direct data format
- **Futures Responses**: Wrapped in `{success, code, data}` structure
- **Field Descriptions**: Complete mapping of all response fields
- **Error Codes**: Common error codes and meanings
- **Data Types**: All numeric values returned as strings for precision
- **Timestamps**: All in milliseconds (not seconds)

### 4. [symbols.md](./symbols.md)
Symbol format and naming conventions:
- **Spot Format**: Concatenated without separator (e.g., `BTCUSDT`)
- **Futures Format**: Underscore separator (e.g., `BTC_USDT`)
- **Case Sensitivity**: All uppercase required
- **Exchange Info**: Get symbol details, filters, permissions
- **Validation Rules**: Status checking, trading permissions
- **Symbol Filters**: LOT_SIZE, MIN_NOTIONAL, PERCENT_PRICE_BY_SIDE

**Best Practice**: Always use `/api/v3/exchangeInfo` to get accurate base/quote assets

### 5. [rate_limits.md](./rate_limits.md)
Complete rate limiting specifications:
- **IP-Based**: 500 requests per 10 seconds (public endpoints)
- **UID-Based**: 500 requests per 10 seconds (private endpoints)
- **Request Weight**: Varies by endpoint (1-50)
- **Ban System**: Automated bans for violations (2 minutes to 3 days)
- **HTTP 429**: Rate limit exceeded error
- **Headers**: `X-RATELIMIT-LIMIT`, `X-RATELIMIT-REMAINING`, `X-RATELIMIT-RESET`

**Recommendations**: Use WebSocket for real-time data, implement exponential backoff, cache static data

### 6. [websocket.md](./websocket.md)
WebSocket API specifications:
- **Spot URL**: `wss://wbs.mexc.com/ws`
- **Futures URL**: `wss://contract.mexc.com/edge`
- **Connection Limits**: Max 30 subscriptions per connection, 24-hour validity
- **Heartbeat**: Ping required every 10-20 seconds (server disconnects after 1 minute)
- **Listen Keys**: Required for user data streams, 60-minute validity, keepalive every 30 minutes
- **Market Streams**: Trades, depth, klines, tickers
- **User Streams**: Account updates, order updates, trade executions

## Implementation Notes

### Futures API Limitation
As of 2026, MEXC Futures API trading is **restricted to institutional users only**. While query endpoints for positions and account information are available, retail users cannot place futures trades via API. Contact institution@mexc.com for institutional access.

### Symbol Format Critical
Different formats for spot vs futures:
- Spot: `BTCUSDT` (no separator)
- Futures: `BTC_USDT` (underscore separator)

Always use uppercase. Validate symbols using `/api/v3/exchangeInfo`.

### Authentication Requirements
All private endpoints require:
1. `X-MEXC-APIKEY` header
2. `timestamp` parameter (milliseconds)
3. `signature` parameter (HMAC SHA256)
4. Optional `recvWindow` (default 5000ms)

### Rate Limit Strategy
- Track request weight per endpoint
- Implement exponential backoff for 429 errors
- Use WebSocket for high-frequency data
- Cache exchange info and symbol data
- Monitor rate limit headers

### WebSocket Best Practices
- Send ping every 15 seconds (well within 60s limit)
- Implement reconnection with exponential backoff
- Manage listen keys with 30-minute keepalive
- Don't exceed 30 subscriptions per connection
- Handle all message types (data, ping, pong, close)

## V5 Connector Implementation Checklist

Based on KuCoin reference structure in `v5/exchanges/kucoin/`:

### Required Files
- [ ] `mod.rs` - Module exports
- [ ] `endpoints.rs` - Endpoint enum, URLs, symbol formatting
- [ ] `auth.rs` - HMAC SHA256 signature implementation
- [ ] `parser.rs` - JSON response parsing
- [ ] `connector.rs` - Trait implementations (MarketData, Trading, Account)
- [ ] `websocket.rs` - WebSocket client (optional but recommended)

### MarketData Trait Methods
- [ ] `ping()` - Test connectivity
- [ ] `get_server_time()` - Server timestamp
- [ ] `get_exchange_info()` - Symbol information
- [ ] `get_order_book()` - Order book depth
- [ ] `get_recent_trades()` - Recent trades
- [ ] `get_klines()` - Candlestick data
- [ ] `get_24hr_ticker()` - 24-hour statistics
- [ ] `get_ticker_price()` - Current price
- [ ] `get_book_ticker()` - Best bid/ask

### Trading Trait Methods
- [ ] `place_order()` - Create new order
- [ ] `cancel_order()` - Cancel order
- [ ] `cancel_all_orders()` - Cancel all open orders
- [ ] `get_order()` - Query order status
- [ ] `get_open_orders()` - Get open orders
- [ ] `get_all_orders()` - Get order history

### Account Trait Methods
- [ ] `get_account_info()` - Account balances
- [ ] `get_account_trades()` - Trade history
- [ ] `get_trade_fees()` - Fee rates

### Positions Trait (Optional - Institutional Only)
- [ ] `get_positions()` - Position information
- [ ] `modify_leverage()` - Change leverage
- [ ] `modify_margin()` - Add/remove margin

### Testing Requirements
- [ ] Unit tests for signature generation
- [ ] Unit tests for endpoint URL construction
- [ ] Unit tests for response parsing
- [ ] Integration tests with testnet (if available)
- [ ] Rate limit handling tests
- [ ] Error response parsing tests

## Sources

All research based on official MEXC API documentation:
- [MEXC API Introduction](https://www.mexc.com/api-docs/spot-v3/introduction)
- [MEXC General Info](https://www.mexc.com/api-docs/spot-v3/general-info)
- [MEXC Market Data Endpoints](https://www.mexc.com/api-docs/spot-v3/market-data-endpoints)
- [MEXC Spot Account/Trade](https://www.mexc.com/api-docs/spot-v3/spot-account-trade)
- [MEXC WebSocket Market Streams](https://www.mexc.com/api-docs/spot-v3/websocket-market-streams)
- [MEXC WebSocket User Data Streams](https://www.mexc.com/api-docs/spot-v3/websocket-user-data-streams)
- [MEXC Futures API](https://www.mexc.com/api-docs/futures/integration-guide)
- [MEXC API Documentation (GitHub)](https://mexcdevelop.github.io/apidocs/spot_v3_en/)

Research completed: 2026-01-20
