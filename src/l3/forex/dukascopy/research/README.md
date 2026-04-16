# Dukascopy API Research - Complete Documentation

**Provider**: Dukascopy Bank SA (Swiss forex data provider)
**Category**: Forex
**Specialty**: Historical tick data with exceptional depth (2003+)
**Research Date**: 2026-01-26

---

## Research Files

This directory contains exhaustive documentation on all Dukascopy API access methods:

1. **[api_overview.md](./api_overview.md)** - Provider information, API types, licensing, support
2. **[endpoints_full.md](./endpoints_full.md)** - Complete endpoint/method reference (JForex SDK, FIX, binary files)
3. **[websocket_full.md](./websocket_full.md)** - WebSocket documentation (third-party only)
4. **[authentication.md](./authentication.md)** - Authentication methods for all access types
5. **[tiers_and_limits.md](./tiers_and_limits.md)** - Pricing, rate limits, free tier details
6. **[data_types.md](./data_types.md)** - Complete catalog of available data types
7. **[response_formats.md](./response_formats.md)** - Response formats (Java objects, binary, FIX, JSON)
8. **[coverage.md](./coverage.md)** - Geographic coverage, instruments, historical depth

---

## Quick Summary

### What Makes Dukascopy Unique

**Primary Strength**: Free, unlimited historical tick data for forex (2003+)

**Key Features**:
- 20+ years of tick-level forex data (free download)
- 100+ currency pairs
- Swiss bank data quality (FINMA regulated)
- No official REST API (Java SDK + FIX protocol)
- Demo account provides full real-time data access
- 10-level order book depth
- Custom bar types (Renko, Kagi, Point & Figure, etc.)

### Access Methods

| Method | Type | Official | Free? | Best For |
|--------|------|----------|-------|----------|
| **Binary Downloads** | .bi5 files | Yes | Yes | Bulk historical tick data |
| **JForex SDK** | Java SDK | Yes | Yes (demo) | Official integration, trading |
| **FIX API** | FIX 4.4 | Yes | No ($100k min) | Professional/institutional trading |
| **Third-Party REST/WS** | Unofficial | No | Yes | Prototyping, non-Java environments |

### Data Coverage

**Instruments**: 1,200+ (forex, crypto CFDs, stocks, indices, commodities)

**Best Coverage**:
- Forex: 100+ pairs, 2003+, tick-level
- Crypto CFDs: 33 instruments, 2017+
- Stocks: 600+ (CFDs, major stocks only)
- Indices: 22 global indices

**Not Available**: Options, crypto derivatives analytics, fundamentals, economic data

### Historical Data Depth

- **Major forex pairs**: 2003+ (20+ years)
- **Minor forex pairs**: 2005-2010+ (15-20 years)
- **Cryptocurrencies**: 2017+ (7+ years)
- **Stocks/indices**: Varies (5-20 years)

**Granularity**: Tick to monthly (tick data is primary strength)

---

## Important Notes

### No Official REST API

Dukascopy does **NOT** provide an official REST API. Access methods are:
1. JForex SDK (Java) - Official
2. FIX 4.4 Protocol - Official
3. Binary file downloads - Official
4. Third-party REST/WebSocket wrappers - Unofficial

### Free Tier is Generous

- **Binary downloads**: No authentication, free, unlimited (with rate limiting)
- **Demo account**: Full real-time + historical data access
- **No API fees**: Only trading costs (if using live account)

### Commercial Use Requires Agreement

- Personal/educational use: Free
- Commercial use: Requires signed supplementary agreement
- Data redistribution: Not allowed without license

---

## Rate Limits

**Binary Downloads**:
- Undocumented (throttling after bulk downloads)
- Fair use policy applies

**JForex SDK**:
- No explicit limits
- Fair use policy

**FIX API**:
- Max 16 orders/second
- Max 100 open positions
- 5 connection attempts per minute per IP

---

## Authentication

### Binary Downloads
- None required (public HTTP access)

### JForex SDK
- Username/password (demo or live account)
- Demo account: Free, extended validity
- Live account: Real trading credentials

### FIX API
- FIX Logon message
- Username/password
- IP registration required
- $100,000 minimum deposit

---

## Data Quality

**Source**: Dukascopy's ECN infrastructure (Swiss bank)
**Regulation**: FINMA (Swiss Financial Market Supervisory Authority)
**Reputation**: Very high (trusted for backtesting)
**Completeness**: Excellent (minimal gaps)
**Accuracy**: Tick-level precision

---

## Best Use Cases

**Ideal For**:
- Forex backtesting (tick-level, 20+ years)
- Spread analysis (historical bid/ask)
- Order book studies (10-level depth)
- Microstructure research
- Academic projects (free data)
- Forex algorithmic trading

**Not Suitable For**:
- Stock fundamentals
- Options trading
- Crypto derivatives analytics
- Economic indicators
- News/sentiment analysis

---

## Implementation Considerations for V5 Connector

### Recommended Approach

**Primary Method**: Binary file downloads (.bi5)
- Pros: Free, no auth, simple HTTP
- Cons: Requires LZMA decompression, hourly files only

**Alternative**: JForex SDK wrapper
- Pros: Real-time data, full API access
- Cons: Requires JNI/FFI, Java dependency

**Not Recommended**: Third-party REST wrapper
- Reason: Unofficial, requires local server

### Binary Download Implementation

**URL Pattern**:
```
https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5
```

**Process**:
1. HTTP GET (no auth)
2. LZMA decompress
3. Parse 20-byte records (big-endian)
4. Convert to tick objects

**Crate Dependencies**:
- `reqwest` - HTTP client
- `lzma-rs` or `xz2` - LZMA decompression
- `byteorder` - Binary parsing

### Rate Limiting Strategy

**Recommendations**:
- 100-500ms delay between requests
- Exponential backoff on 429/503 errors
- Cache downloaded files locally
- Download during off-peak hours for bulk operations

---

## Key Takeaways

1. **Free Tick Data**: Dukascopy's killer feature (2003+, no API fees)
2. **No REST API**: Use binary downloads or JForex SDK
3. **Swiss Quality**: Bank-grade data reliability
4. **Forex Focus**: Best-in-class for forex, limited for other assets
5. **Fair Use**: Generous free tier with undocumented limits

---

## Official Resources

- **Main Website**: https://www.dukascopy.com
- **JForex SDK Docs**: https://www.dukascopy.com/wiki/en/development/strategy-api/
- **Javadocs**: https://www.dukascopy.com/client/javadoc3/com/dukascopy/api/
- **FIX API Spec**: https://www.dukascopy.com/swiss/docs/Dukascopy_FIXAPI-8.0.1.pdf
- **Demo Account**: https://www.dukascopy.com/swiss/english/forex/demo/

---

## Sources

This research is based on:
- [Dukascopy API Documentation](https://www.dukascopy.com/trading-tools/api/documentation)
- [JForex API Javadocs](https://www.dukascopy.com/client/javadoc3/com/dukascopy/api/IHistory.html)
- [Dukascopy FIX API Programming Guide](https://www.dukascopy.com/swiss/docs/Dukascopy_FIXAPI-8.0.1.pdf)
- [Historical Data Service](https://www.dukascopy.com/wiki/en/development/strategy-api/historical-data/historical-data-service/)
- [dukascopy-node GitHub](https://github.com/Leo4815162342/dukascopy-node)
- [dukascopy-api-websocket GitHub](https://github.com/ismailfer/dukascopy-api-websocket)
- [The Dukascopy .bi5 tick data demystified](https://medium.com/@tomas.rampas/the-dukascopy-tick-data-demystified-3af1da80e6c5)
- [Dukascopy Review 2026](https://www.daytrading.com/dukascopy)
- [FIX API Overview](https://www.dukascopy.com/swiss/english/forex/api/fix-api/)
- Community implementations and documentation

---

**Research Complete**: All 8 files documented with exhaustive technical details.
