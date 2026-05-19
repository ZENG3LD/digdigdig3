# Binance API Research Documentation

This directory contains comprehensive research documentation for implementing the Binance V5 connector.

## Documentation Files

### 1. [endpoints.md](./endpoints.md)
Complete endpoint reference for all V5 connector traits:

- **Base URLs**: Spot, Futures USDT-M, Futures COIN-M, Testnet
- **MarketData Trait**: ping, get_price, get_orderbook, get_klines, get_ticker
- **Trading Trait**: market_order, limit_order, cancel_order, get_order, get_open_orders
- **Account Trait**: get_balance, get_account_info
- **Positions Trait** (Futures): get_positions, get_funding_rate, set_leverage
- **User Data Stream**: listenKey creation, keepalive, close

Each endpoint includes:
- Exact endpoint path
- HTTP method
- All parameters (required and optional)
- Request weight
- Complete response examples

### 2. [authentication.md](./authentication.md)
HMAC-SHA256 authentication implementation guide:

- Required headers (X-MBX-APIKEY)
- Signature algorithm details
- Query string signing process
- recvWindow parameter usage
- Timestamp requirements and synchronization
- Percent-encoding (2026 update)
- Rust implementation examples
- Common authentication errors and solutions
- Security best practices

### 3. [response_formats.md](./response_formats.md)
Complete JSON response structures with all field names and types:

- All MarketData responses
- All Trading responses (ACK, RESULT, FULL formats)
- All Account responses
- All Positions responses (Futures)
- User Data Stream responses
- Error response format
- Time and numeric field formats

### 4. [symbols.md](./symbols.md)
Symbol format specifications and conversion:

- **Spot Format**: BTCUSDT (no separator)
- **Futures USDT-M Format**: BTCUSDT (same as spot)
- **Futures COIN-M Format**: BTCUSD_PERP (with underscore)
- Symbol validation rules
- Symbol conversion utilities
- Exchange info endpoint
- Symbol precision and filters
- Lot size and notional filters
- Rust implementation examples

### 5. [rate_limits.md](./rate_limits.md)
Rate limiting rules and strategies:

- **REQUEST_WEIGHT**: 6,000 per minute (primary limit)
- **RAW_REQUESTS**: 61,000 per 5 minutes
- **ORDERS**: 50 per 10 seconds, 160,000 per 24 hours (Spot)
- Endpoint weights table
- Rate limit headers
- HTTP 429 and 418 handling
- WebSocket limits
- VIP level benefits
- Implementation strategies
- Rust rate limiter example

### 6. [websocket.md](./websocket.md)
WebSocket streams for real-time data:

- Base URLs (Spot, Futures USDT-M, Futures COIN-M)
- Connection requirements and limits
- Market data streams:
  - Ticker streams (@ticker, @miniTicker)
  - Order book streams (@depth, @depth@100ms)
  - Trade streams (@trade, @aggTrade)
  - Kline streams (@kline_1m, etc.)
  - Book ticker (@bookTicker)
- User data stream setup and events
- Combined streams
- Subscribe/Unsubscribe methods
- Rust implementation examples

## Implementation Checklist

### Phase 1: Core Structure
- [ ] Create module structure (mod.rs, endpoints.rs, auth.rs, parser.rs, connector.rs)
- [ ] Define endpoint enum with all required endpoints
- [ ] Implement URL builder with base URL selection

### Phase 2: Authentication
- [ ] Implement HMAC-SHA256 signature generation
- [ ] Add timestamp parameter handling
- [ ] Add recvWindow parameter support
- [ ] Implement query string builder
- [ ] Add X-MBX-APIKEY header

### Phase 3: MarketData Trait
- [ ] ping() - Test connectivity
- [ ] get_price() - Symbol price ticker
- [ ] get_orderbook() - Order book depth
- [ ] get_klines() - Candlestick data
- [ ] get_ticker() - 24hr statistics

### Phase 4: Trading Trait
- [ ] market_order() - Place market order
- [ ] limit_order() - Place limit order
- [ ] cancel_order() - Cancel order
- [ ] get_order() - Query order status
- [ ] get_open_orders() - Query all open orders

### Phase 5: Account Trait
- [ ] get_balance() - Get account balances
- [ ] get_account_info() - Get account information

### Phase 6: Positions Trait (Futures)
- [ ] get_positions() - Get position information
- [ ] get_funding_rate() - Get funding rate
- [ ] set_leverage() - Change leverage

### Phase 7: WebSocket (Optional)
- [ ] Market data streams
- [ ] User data stream
- [ ] Connection management
- [ ] Ping/pong handling

### Phase 8: Rate Limiting
- [ ] Implement local rate limiter
- [ ] Parse rate limit headers
- [ ] Exponential backoff for 429
- [ ] Handle 418 (IP ban)

### Phase 9: Testing
- [ ] Test all MarketData methods
- [ ] Test all Trading methods
- [ ] Test all Account methods
- [ ] Test all Positions methods (Futures)
- [ ] Test rate limiting
- [ ] Test error handling

## Key Differences from KuCoin

| Feature | KuCoin | Binance |
|---------|--------|---------|
| API Key Header | KC-API-KEY | X-MBX-APIKEY |
| Timestamp Location | Header | Query parameter |
| Signature Location | Header | Query parameter |
| Passphrase | Required | Not required |
| Signature Algorithm | HMAC-SHA256 (same) | HMAC-SHA256 (same) |
| Base URL | Single | Multiple options |
| Symbol Format | BTC-USDT | BTCUSDT |
| Rate Limit | Per user | Per IP (mainly) |
| Futures | USDT-M only | USDT-M and COIN-M |

## Official Resources

- [Binance Spot API Documentation](https://developers.binance.com/docs/binance-spot-api-docs/rest-api)
- [Binance Futures USDT-M Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures)
- [Binance Futures COIN-M Documentation](https://developers.binance.com/docs/derivatives/coin-margined-futures)
- [Binance WebSocket Streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)
- [Binance API GitHub Repository](https://github.com/binance/binance-spot-api-docs)

## Notes

- All timestamps are in milliseconds by default
- All numeric values are returned as strings to preserve precision
- Symbols must be uppercase for REST API, lowercase for WebSocket
- Rate limits are strictly enforced (6,000 weight/min)
- WebSocket streams are recommended for real-time data
- Testnet has the same rate limits as production

## Research Date

This research was conducted on **2026-01-20** using the latest official Binance API documentation.
