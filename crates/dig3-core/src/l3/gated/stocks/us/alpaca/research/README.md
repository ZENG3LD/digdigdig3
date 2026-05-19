# Alpaca API Research - Complete Documentation

This folder contains exhaustive research on the Alpaca API for V5 connector implementation.

## Research Files

1. **api_overview.md** - Provider info, API types, base URLs, documentation quality, licensing
2. **endpoints_full.md** - Complete endpoint reference with all parameters (Trading + Market Data)
3. **websocket_full.md** - WebSocket documentation (Market Data + Trading Updates streams)
4. **authentication.md** - Authentication methods, API keys, OAuth, security practices
5. **tiers_and_limits.md** - Pricing tiers, rate limits, quotas, tier comparison
6. **data_types.md** - Complete catalog of available data types and coverage
7. **response_formats.md** - JSON response examples for all major endpoints
8. **coverage.md** - Geographic, market, instrument coverage and limitations

## Quick Reference

### Provider Type
- **US Stock Broker + Market Data Provider**
- Supports both TRADING and DATA access
- Paper trading (free globally) + Live trading (US only)

### Base URLs

**Trading API:**
- Live: `https://api.alpaca.markets`
- Paper: `https://paper-api.alpaca.markets`

**Market Data API:**
- Production: `https://data.alpaca.markets`
- Sandbox: `https://data.sandbox.alpaca.markets`

**WebSocket:**
- Market Data: `wss://stream.data.alpaca.markets/v2/{iex|sip}`
- Trading Updates: `wss://api.alpaca.markets/stream` (live) / `wss://paper-api.alpaca.markets/stream` (paper)

### Authentication
- **Method:** API Key ID + Secret in headers
- **Headers:** `APCA-API-KEY-ID`, `APCA-API-SECRET-KEY`
- **No HMAC signatures required** (simple auth)

### Tiers

| Feature | Free (IEX) | Algo Trader Plus ($99/mo) |
|---------|------------|---------------------------|
| REST Rate Limit | 200/min | Unlimited |
| WebSocket Symbols | 30 max | Unlimited |
| Stock Feed | IEX only | All US exchanges (SIP) |
| Options Data | Indicative | Real-time OPRA |

### Key Features

**Trading:**
- ✅ Commission-free (stocks, ETFs, options, crypto)
- ✅ Paper trading (free forever)
- ✅ Fractional shares (2,000+ symbols)
- ✅ Margin trading (up to 4X intraday)
- ✅ Options (up to Level 3)
- ✅ Crypto (24/7 spot trading)

**Market Data:**
- ✅ Real-time WebSocket streams
- ✅ 7+ years historical data (stocks)
- ✅ 6+ years historical data (crypto)
- ✅ News API (Benzinga, others)
- ✅ Corporate actions (dividends, splits)
- ✅ Extended hours data (paid tier)

**Limitations:**
- ❌ US markets only (no international stocks)
- ❌ No fundamental data (financials, earnings)
- ❌ No futures/derivatives (options only)
- ❌ Limited forex (basic rates only)
- ❌ No Level 2 orderbook for stocks (crypto only)

### Data Coverage

**Instruments:**
- Stocks: ~8,000 US equities
- ETFs: ~3,000
- Options: 1,000+ underlyings with full OPRA feed
- Crypto: 50-100 trading pairs (Alpaca + Kraken)

**Historical Depth:**
- Stocks: 7+ years (back to 2016)
- Crypto: 6+ years
- Minute bars: 5 years
- Daily bars: 7+ years

**Update Frequency:**
- Real-time: <100ms latency (WebSocket)
- Minute bars: Every 60 seconds
- News: Real-time
- Corporate actions: Next business day

### Implementation Priority

**Phase 1 - Market Data (Read-Only):**
1. REST endpoints: bars, trades, quotes, snapshots
2. WebSocket: trades, quotes, bars channels
3. News API integration

**Phase 2 - Trading (if needed):**
1. Account management
2. Order placement (market, limit, stop)
3. Position tracking
4. WebSocket trade updates

**Phase 3 - Advanced Features:**
1. Options trading
2. Crypto trading
3. Corporate actions
4. Extended hours data

## V5 Connector Architecture

**Recommended module structure:**
```
src/stocks/us/alpaca/
├── research/           # This folder
├── endpoints.rs        # URLs, endpoint enum, symbol formatting
├── auth.rs            # API key authentication (simple, no HMAC)
├── parser.rs          # JSON parsing for all response types
├── connector.rs       # Trait implementations (MarketData, Trading)
└── websocket.rs       # WebSocket streams (market data + trading updates)
```

**Key differences from crypto exchanges:**
- No HMAC signatures (simple API key auth)
- Two separate WebSocket systems (market data vs trading)
- Pagination via `next_page_token` (not limit/offset)
- Multiple base URLs (paper vs live, trading vs market data)
- Feed selection (IEX vs SIP for stocks)

## Sources

All information gathered from official Alpaca documentation:
- Main docs: https://docs.alpaca.markets/docs
- API reference: https://docs.alpaca.markets/reference
- Community forum: https://forum.alpaca.markets/
- GitHub: https://github.com/alpacahq

## Next Steps

1. Review all research files thoroughly
2. Decide on implementation scope (market data only vs full trading)
3. Create connector implementation following KuCoin V5 pattern
4. Test with paper trading account (free)
5. Implement rate limiting (200/min for free tier)
6. Add WebSocket reconnection logic
7. Handle pagination correctly

## Notes

- **Paper trading highly recommended for testing** - Free, unlimited, real-time data
- **Start with IEX feed (free)** before implementing SIP feed
- **Focus on market data first** - Trading can be added later
- **WebSocket preferred for real-time** - More efficient than polling REST
- **Respect rate limits** - 200/min on free tier, exponential backoff on 429
