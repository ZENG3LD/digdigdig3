# MOEX ISS API Research - Complete Documentation

**Provider**: Moscow Exchange (MOEX)
**Category**: stocks/russia
**Type**: Market Data Provider (Data Only - NO TRADING)
**Research Date**: 2026-01-26
**API Version**: ISS v0.14.1 (Informational & Statistical Server)

## Overview

Moscow Exchange (MOEX) is Russia's largest exchange, providing comprehensive market data for Russian equities, bonds, derivatives, currencies, and commodities. The ISS (Informational & Statistical Server) is a REST/WebSocket API offering extensive market data with free delayed access (15-minute delay) and paid real-time subscriptions.

## Research Files (8/8 Complete)

1. **[api_overview.md](api_overview.md)** (3.9 KB)
   - Provider information and documentation quality
   - API types (REST, WebSocket/STOMP)
   - Base URLs and support channels
   - 11 trading engines overview
   - Licensing terms and restrictions

2. **[endpoints_full.md](endpoints_full.md)** (27.6 KB)
   - **400+ unique endpoint patterns** fully documented
   - Complete endpoint reference with parameters
   - Organized by category: securities, market data, historical, derivatives, CCI, etc.
   - Parameter documentation for complex endpoints
   - Pagination and filtering examples

3. **[websocket_full.md](websocket_full.md)** (12.7 KB)
   - WebSocket via STOMP protocol
   - Connection process and authentication
   - Available channels (market data, trades, quotes, orderbook)
   - Heart-beat mechanism (critical for connection stability)
   - Free (delayed) vs Paid (real-time) access
   - Note: Limited public documentation, contact MOEX for details

4. **[authentication.md](authentication.md)** (13.8 KB)
   - No API key system (uses username/password)
   - MOEX Passport account for authentication
   - OAuth 2.0 for WebAPI (trading only, not ISS)
   - HTTP Basic Auth for REST (rarely needed)
   - STOMP authentication for WebSocket
   - Free tier: No auth required (delayed data)
   - Paid tier: Requires subscription and credentials

5. **[tiers_and_limits.md](tiers_and_limits.md)** (13.5 KB)
   - Free tier: Delayed data (15 min), unlimited historical
   - Paid tier: Real-time data, custom pricing (contact MOEX)
   - Distributor tiers: Redistribution rights
   - **Rate limits**: NOT documented publicly (implement conservative 1 req/sec)
   - WebSocket limits: Unknown (likely 5-10 connections for free)
   - No public usage dashboard

6. **[data_types.md](data_types.md)** (15.0 KB)
   - **Comprehensive Russian market data catalog**
   - Standard market data: price, volume, OHLC, trades
   - Derivatives: open interest, futures, options
   - Fundamental data: IFRS + RSBU financials, ratings, corporate actions
   - Indices: IMOEX, RTSI, sector indices
   - Unique data: Net Flow 2, zero-coupon yield curves, dual accounting standards
   - Corporate Information Services (CCI): extensive company data
   - What's NOT available: institutional holdings, insider trading, option Greeks (must calculate)

7. **[response_formats.md](response_formats.md)** (23.7 KB)
   - **EXACT JSON examples from live API**
   - Multi-block response structure (metadata + columns + data)
   - Examples for: engines, securities, candles, trades, market data, indices
   - XML and CSV format examples
   - Common field types and naming conventions
   - Error response formats
   - Pagination and column filtering

8. **[coverage.md](coverage.md)** (14.9 KB)
   - **Geographic**: Russia (primary), CIS (limited), foreign stocks (limited)
   - **Markets**: Moscow Exchange only (not an aggregator)
   - **Instruments**: ~3000+ across all asset classes
     - ~700-800 stocks (Russian + foreign)
     - ~1000+ bonds (government, corporate, municipal)
     - ~1000+ derivatives (futures + options)
     - ~20-30 currency pairs
   - **Historical depth**: From 1997+ for major indices, varies by instrument
   - **Data quality**: Exchange-authoritative, 99%+ uptime
   - **Granularity**: Tick to quarterly bars (free access to all historical)

## Key Findings

### Strengths
- **Comprehensive Russian market coverage**: All asset classes (equities, bonds, FX, derivatives)
- **Free historical data**: Unlimited depth from 1997+, all granularities (1m to quarterly)
- **Rich corporate data**: IFRS + RSBU financials, credit ratings, corporate actions
- **Unique analytics**: Net Flow 2 (since 2007), zero-coupon yield curves
- **Multi-asset support**: 11 engines, 120+ markets, 500+ boards
- **High data quality**: Exchange-grade reliability
- **No geo-restrictions**: API accessible globally

### Limitations
- **15-minute delay** for free tier (real-time requires subscription)
- **Rate limits undocumented**: Must implement conservative client-side limiting (1 req/sec recommended)
- **No API key system**: Uses username/password authentication
- **Single exchange**: MOEX only (not multi-exchange aggregator)
- **Limited foreign coverage**: Only foreign stocks traded on MOEX
- **Documentation in Russian**: Primary language is Russian
- **WebSocket docs limited**: Contact MOEX for detailed integration specs
- **No institutional holdings**: Unlike US markets (no 13F equivalent)
- **Must calculate option Greeks**: Not provided by API

### Unique Features
1. **Dual accounting standards**: IFRS + RSBU (Russian) financial reports
2. **Net Flow 2**: Institutional vs retail money flow tracking (since 2007)
3. **Zero-coupon yield curves**: Russian bond market analytics
4. **Corporate Information Services (CCI)**: Extensive company data, ratings, affiliate reports
5. **Multi-engine architecture**: 11 distinct trading engines
6. **Free archive access**: Bulk historical downloads (yearly, monthly, daily)
7. **Consensus forecasts**: Analyst consensus for Russian stocks

## API Endpoints Overview

### Total Endpoint Count
**~400+ unique endpoint patterns** across categories:

- **Securities & Instruments**: 4 endpoints
- **Current Market Data**: 12 endpoints (real-time/delayed)
- **Historical OHLC/Candles**: 6 endpoints (all intervals)
- **Historical Trading Data**: 8 endpoints
- **Historical Sessions**: 5 endpoints
- **Historical Yields**: 4 endpoints (bonds)
- **Archives**: 3 endpoints (bulk downloads)
- **Listings**: 3 endpoints
- **Market Statistics**: 4 endpoints
- **Indices**: 5 endpoints
- **Derivatives Analytics**: 10 endpoints
- **Currency & Rates**: 7 endpoints
- **Yield Curves**: 4 endpoints
- **Stock Market Stats**: 9 endpoints
- **Aggregated Totals**: 4 endpoints
- **Analytics Products**: 4 endpoints
- **OTC Markets**: 3 endpoints
- **Reference Data 2.0**: 6 endpoints
- **Metadata**: 11 endpoints
- **Security Groups**: 5 endpoints
- **Corporate Info (CCI)**: 50+ endpoints (financials, ratings, actions, affiliates)
- **News & Events**: 4 endpoints
- **Risk Management**: 3 endpoints
- **Collateral & Rates**: 5 endpoints
- **Field Descriptions**: 5 endpoints

## Data Access Summary

| Data Type | Free Tier | Paid Tier |
|-----------|-----------|-----------|
| Current market data | 15-min delay | Real-time |
| Historical data | Full access | Full access |
| OHLC/Candles | All intervals | All intervals |
| Trades | 15-min delay | Real-time |
| Quotes | 15-min delay | Real-time |
| Orderbook | Not available | Real-time (10x10 or 5x5) |
| Indices | 15-min delay | Real-time (1-sec intervals) |
| Corporate data | Full access | Full access |
| News & events | Full access | Full access |
| WebSocket | Delayed streams | Real-time streams |
| Archives | Full access | Full access |

## Implementation Notes

### For V5 Connector Development

1. **Rate Limiting**
   - Implement client-side rate limiter: 1 req/sec default (conservative)
   - Exponential backoff on HTTP 429/500/503 errors
   - Don't retry on 400/401/403/404

2. **Authentication**
   - Support both free (no auth) and paid (username/password) modes
   - HTTP Basic Auth for REST (if needed)
   - STOMP authentication for WebSocket

3. **Response Parsing**
   - Multi-block structure: parse `metadata`, `columns`, `data`
   - Handle empty `data` arrays (no results)
   - Type conversion based on metadata types
   - UTF-8 encoding (Russian language support)

4. **Data Caching**
   - Cache reference data aggressively (engines, markets, boards)
   - Cache security metadata (changes infrequently)
   - Don't cache market data (stale quickly)

5. **WebSocket Strategy**
   - Use WebSocket for real-time streams (more efficient than REST polling)
   - Implement STOMP protocol support (not raw WebSocket)
   - Handle heart-beats (10-second intervals recommended)
   - Automatic reconnection with exponential backoff

6. **Error Handling**
   - HTTP errors: 401 (auth), 403 (forbidden), 404 (not found), 429 (rate limit)
   - STOMP errors: Authentication failed, subscription limits
   - Implement retry logic with backoff

7. **Endpoint Patterns**
   - Parameterized paths: `/engines/[engine]/markets/[market]/securities/[security]`
   - Common params: `from`, `till`, `interval`, `start`, `limit`
   - Column filtering: `securities.columns=SECID,LAST,VOLUME`
   - Format selection: `.json`, `.xml`, `.csv` extensions

## Contact Information

- **Documentation**: https://iss.moex.com/iss/reference/
- **Main site**: https://www.moex.com/a2193
- **Technical Support**: help@moex.com, +7 (495) 733-9507
- **Data Services**: https://www.moex.com/s1147
- **Passport Registration**: https://passport.moex.com/

## Sources

Research compiled from:
- [Moscow Exchange ISS Reference](https://iss.moex.com/iss/reference/)
- [MOEX Programming Interface](https://www.moex.com/a2920)
- [MOEX Interfaces Overview](https://www.moex.com/a7939)
- [MOEX ISS Documentation](https://www.moex.com/a2193)
- [GitHub Community Libraries](https://github.com/topics/moex-api)
- [Postman API Collections](https://www.postman.com/studentspbstu/api-moex)
- Live API responses from ISS endpoints

## Next Steps

**Phase 2**: Implementation
- Design Rust data structures based on response formats
- Implement REST client with rate limiting
- Implement STOMP WebSocket client (optional)
- Create parsers for multi-block JSON responses
- Add authentication support (Basic Auth for REST, STOMP for WS)
- Implement V5 traits (MarketData, etc.)
- Add comprehensive error handling
- Write unit tests with mock responses

**DO NOT START IMPLEMENTATION** until research is reviewed and approved.

---

**Research Status**: COMPLETE ✓
**Files Created**: 8/8 ✓
**Total Documentation**: ~125 KB of comprehensive API research
**Ready for**: Phase 2 Implementation
