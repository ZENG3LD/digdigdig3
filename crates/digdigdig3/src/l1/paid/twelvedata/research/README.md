# Twelvedata API Research - Complete Documentation

**Provider**: Twelvedata
**Category**: stocks/us (multi-asset provider)
**Documentation URL**: https://twelvedata.com/docs
**Research Completed**: 2026-01-26
**Total Documentation**: 3,682 lines across 8 files

---

## Research Files Overview

| File | Lines | Description |
|------|-------|-------------|
| **api_overview.md** | 91 | Provider information, API types, base URLs, documentation quality, licensing |
| **endpoints_full.md** | 315 | COMPLETE endpoint reference - all categories, parameters, credit costs |
| **websocket_full.md** | 367 | WebSocket documentation - connection, channels, messages, heartbeat |
| **authentication.md** | 377 | API key authentication, error codes, security best practices |
| **tiers_and_limits.md** | 432 | Pricing tiers, rate limits, quota system, WebSocket credits |
| **data_types.md** | 502 | Comprehensive catalog of all available data types |
| **response_formats.md** | 1,058 | Exact JSON/CSV response formats for all major endpoints |
| **coverage.md** | 540 | Geographic coverage, exchanges, instruments, historical depth |
| **README.md** | (this file) | Navigation and summary |

**Total**: 3,682 lines of exhaustive documentation

---

## Quick Navigation

### For Implementation Planning
1. Start with **api_overview.md** - understand provider characteristics
2. Read **endpoints_full.md** - identify which endpoints you need
3. Check **tiers_and_limits.md** - understand rate limits and costs
4. Review **authentication.md** - implement API key handling

### For Connector Development
1. **response_formats.md** - exact JSON structures for parsing
2. **endpoints_full.md** - parameter requirements
3. **websocket_full.md** - if implementing real-time streaming
4. **authentication.md** - error handling patterns

### For Feature Planning
1. **data_types.md** - see what data is available
2. **coverage.md** - check instrument/exchange coverage
3. **tiers_and_limits.md** - understand tier requirements for features

---

## Key Findings Summary

### Provider Type
- **Multi-asset data provider**: Stocks, Forex, Crypto, ETFs, Commodities, Indices
- **NOT a trading exchange**: Data provider only (no order execution)
- **Unified API**: Same endpoints work across all asset types

### Authentication
- **Simple API key**: No complex HMAC/signing required
- **Header-based** (recommended): `Authorization: apikey YOUR_KEY`
- **Query parameter** (alternative): `?apikey=YOUR_KEY`
- **Demo key available**: `apikey=demo` for testing

### API Structure
- **REST API**: https://api.twelvedata.com
- **WebSocket**: wss://ws.twelvedata.com (Pro+ plans only)
- **No GraphQL/gRPC**: REST only

### Pricing Model
- **Credit-based system**: Different endpoints cost different credits
- **Free tier**: 8 calls/min, 800/day (Basic plan)
- **Paid tiers**: Grow ($29+), Pro ($99+), Ultra ($329+), Enterprise (custom)
- **WebSocket**: Separate credits, Pro+ plans only

### Rate Limits
| Tier | Calls/Min | Calls/Day | WebSocket | Real-time |
|------|-----------|-----------|-----------|-----------|
| Basic (Free) | 8 | 800 | No | No |
| Grow | 55-377 | ~79K-543K | No | No |
| Pro | 610-1,597 | ~878K-2.3M | Yes (8-32) | Yes |
| Ultra | 2,584-16,721 | ~3.7M-24M | Yes (2.5K-16K) | Yes |

### Endpoint Categories (Total: 100+ endpoints)
1. **Core Market Data** (7 endpoints): time_series, quote, price, eod, etc.
2. **Reference Data** (7 endpoints): stocks, forex_pairs, cryptocurrencies, etf, etc.
3. **Discovery** (3 endpoints): symbol_search, cross_listings, earliest_timestamp
4. **Markets Info** (6 endpoints): exchanges, market_state, exchange_schedule, etc.
5. **Technical Indicators** (100+ endpoints): RSI, MACD, Bollinger Bands, SMA, EMA, etc.
6. **Fundamentals** (30+ endpoints): financials, earnings, dividends, analyst ratings
7. **ETF Data** (5 endpoints): composition, performance, ratings
8. **Mutual Funds** (3 endpoints): ratings, purchase info, sustainability
9. **Market Movers** (5 endpoints): top gainers/losers across asset types

### WebSocket (Pro+ Only)
- **URL**: wss://ws.twelvedata.com/v1/quotes/price?apikey=KEY
- **Latency**: ~170ms average
- **Max connections**: 3 per API key
- **Heartbeat required**: Every 10 seconds
- **Channels**: Price events only (no orderbook/trades/klines)
- **Multi-asset**: Stocks, forex, crypto in same connection

### Data Coverage
- **Stocks**: 60,000+ symbols (90+ exchanges globally)
- **Forex**: 200+ pairs (majors, minors, exotics)
- **Crypto**: Thousands of pairs (180+ exchanges)
- **ETFs**: 5,000+ (US + international)
- **Mutual Funds**: 20,000+
- **Commodities**: 50+ (metals, energy, agriculture)
- **Indices**: 100+ global

### Historical Depth
- **Stocks**: Back to 1980s-1990s for major companies
- **Intraday**: 1-2 years (Basic), 5+ years (Grow), unlimited (Pro+)
- **Daily**: Decades for most assets
- **Fundamentals**: Back to 1980s-1990s (financials, earnings, dividends)

### Unique Features
1. **100+ technical indicators** - comprehensive library
2. **Cross-rate calculation** - exotic pairs on-the-fly (5 credits)
3. **Extended hours data** - US pre/post-market (Pro+ plans)
4. **Batch requests** - 120 symbols per call (1 credit per 100 symbols)
5. **Multi-format output** - JSON, CSV with configurable delimiters
6. **FIGI/ISIN/CUSIP support** - multiple identifier types (Ultra+ plans)
7. **Unified multi-asset API** - stocks/forex/crypto with same endpoints

### What's NOT Available
- ❌ Options chains/data (no options support)
- ❌ Futures contracts (no futures data)
- ❌ Level 2 order book (bid/ask only, no depth)
- ❌ Crypto derivatives (funding rates, liquidations, OI)
- ❌ On-chain data (blockchain metrics)
- ❌ Economic indicators (GDP, CPI, unemployment)
- ❌ News feeds (no news articles)
- ❌ Social sentiment (no sentiment analysis)
- ❌ DEX data (centralized exchanges only)

### Best Use Cases
✅ **Multi-asset price monitoring** (stocks, forex, crypto)
✅ **US stock fundamental analysis** (financials, earnings, ratings)
✅ **Technical analysis** (100+ indicators on all assets)
✅ **Charting/dashboard applications**
✅ **Historical data analysis**
✅ **Real-time price streaming** (Pro+ WebSocket)

❌ **NOT suitable for**: Options trading, futures, crypto derivatives, on-chain analysis, economic forecasting

---

## Response Format Highlights

### All Endpoints Return
```json
{
  "data": [...],
  "status": "ok"
}
```

### Error Format
```json
{
  "code": 400,
  "message": "Detailed error message",
  "status": "error"
}
```

### Null Values
**CRITICAL**: Many fields may return `null` when data unavailable. Always implement defensive checks.

```json
{
  "day_volume": null,
  "fifty_two_week": {
    "high": null
  }
}
```

### Time Series (OHLCV)
**Note**: Numeric values returned as **strings** to preserve precision.

```json
{
  "values": [
    {
      "datetime": "2024-01-26",
      "open": "149.50000",
      "high": "151.20000",
      "low": "148.80000",
      "close": "150.25000",
      "volume": "65432100"
    }
  ]
}
```

### WebSocket Price Event
```json
{
  "event": "price",
  "symbol": "AAPL",
  "exchange": "NASDAQ",
  "type": "Common Stock",
  "timestamp": 1706284800,
  "price": 150.25,
  "day_volume": 65432100
}
```

---

## Implementation Checklist

### Phase 1: REST API
- [ ] API key authentication (header-based)
- [ ] Error handling (400, 401, 403, 404, 429, 500)
- [ ] Rate limit tracking (X-RateLimit headers)
- [ ] Exponential backoff for 429 errors
- [ ] Null value defensive handling
- [ ] Parse string numerics (time_series)

### Phase 2: Core Endpoints
- [ ] `/price` - latest price
- [ ] `/quote` - full quote data
- [ ] `/time_series` - OHLCV bars
- [ ] Batch requests - multiple symbols

### Phase 3: Reference Data
- [ ] `/stocks` - symbol catalog
- [ ] `/forex_pairs` - forex catalog
- [ ] `/cryptocurrencies` - crypto catalog
- [ ] `/symbol_search` - search symbols

### Phase 4: WebSocket (Optional, Pro+ only)
- [ ] Connection with API key in URL
- [ ] Subscribe message handling
- [ ] Price event parsing
- [ ] Heartbeat every 10 seconds
- [ ] Reconnection logic
- [ ] Connection limit (max 3)

### Phase 5: Advanced Features (Optional)
- [ ] Technical indicators (RSI, MACD, etc.)
- [ ] Fundamentals (if Grow+ plan)
- [ ] Market movers (if Pro+ plan)
- [ ] Extended hours data (if Pro+ plan)

---

## Credit Costs Quick Reference

| Endpoint Type | Credits | Notes |
|---------------|---------|-------|
| Price/Quote/EOD | 1 | Per symbol |
| Time Series | 1 | Per symbol |
| Cross Rate | 5 | Per symbol |
| Market Movers | 100 | Per request |
| Exchange Schedule | 100 | Per request, Ultra+ |
| Cross Listings | 40 | Per request, Grow+ |
| Profile | 10 | Per symbol, Grow+ |
| Earnings/Dividends | 20 | Per symbol |
| Financials | High | Varies, 50+, Grow+ |
| Batch (100 symbols) | 1 | vs 100 individually |

---

## Rate Limit Strategy

### Proactive Rate Limiting
```rust
// Track remaining quota from headers
let remaining = response.headers()
    .get("X-RateLimit-Remaining")
    .and_then(|h| h.to_str().ok())
    .and_then(|s| s.parse::<u32>().ok())
    .unwrap_or(0);

if remaining < 5 {
    // Approaching limit, slow down
    tokio::time::sleep(Duration::from_secs(10)).await;
}
```

### Exponential Backoff (429 Errors)
```rust
let mut backoff = Duration::from_secs(2);
for retry in 0..5 {
    match send_request().await {
        Err(e) if e.status() == 429 => {
            let retry_after = e.headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(backoff);

            tokio::time::sleep(retry_after).await;
            backoff *= 2; // Exponential backoff
        }
        Ok(response) => return Ok(response),
        Err(e) => return Err(e),
    }
}
```

---

## Next Steps

1. **Review all 8 research files** to understand API completely
2. **Choose tier** based on requirements (free Basic for testing, Pro+ for real-time)
3. **Design connector architecture** following V5 pattern (endpoints.rs, auth.rs, parser.rs, connector.rs)
4. **Implement authentication** (simple API key, no HMAC)
5. **Build core endpoints** (price, quote, time_series)
6. **Add error handling** (especially 429 rate limits)
7. **Implement rate limit tracking** (use response headers)
8. **Test with demo key** (`apikey=demo`)
9. **Add batch optimization** (120 symbols per call)
10. **Consider WebSocket** (if Pro+ plan and real-time needed)

---

## Sources

All information gathered from official Twelvedata documentation and support articles:

- [API Documentation](https://twelvedata.com/docs)
- [Pricing](https://twelvedata.com/pricing)
- [WebSocket Streaming Guide](https://support.twelvedata.com/en/articles/5620516-how-to-stream-the-data)
- [Symbol Lookup Guide](https://support.twelvedata.com/en/articles/5620513-how-to-find-all-available-symbols-at-twelve-data)
- [Historical Data Guide](https://support.twelvedata.com/en/articles/5214728-getting-historical-data)
- [Python SDK](https://github.com/twelvedata/twelvedata-python)
- [Support Portal](https://support.twelvedata.com/)

---

**Research completed**: 2026-01-26
**Researched by**: Claude Sonnet 4.5 (research-agent)
**Ready for**: Phase 2 (Connector Implementation)
