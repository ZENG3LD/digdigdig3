# Futu OpenAPI Research - Complete Documentation

**Research Date**: 2026-01-26
**API Version**: v9.6
**Provider**: Futu Securities / moomoo
**Category**: stocks/china (multi-market broker)
**Documentation**: https://openapi.futunn.com/futu-api-doc/en/

---

## Research Files

All 8 required research files have been completed:

1. [api_overview.md](./api_overview.md) - Provider information, API architecture, documentation quality
2. [endpoints_full.md](./endpoints_full.md) - Complete endpoint reference (100+ methods documented)
3. [websocket_full.md](./websocket_full.md) - Custom TCP protocol with subscription system
4. [authentication.md](./authentication.md) - OpenD gateway authentication, trade unlock, quote rights
5. [tiers_and_limits.md](./tiers_and_limits.md) - Rate limits, subscription quotas, pricing tiers
6. [data_types.md](./data_types.md) - Complete data type catalog
7. [response_formats.md](./response_formats.md) - Exact response formats with examples
8. [coverage.md](./coverage.md) - Market coverage, instruments, historical depth

---

## Key Findings

### API Architecture
- **NOT REST or WebSocket** - Uses custom TCP protocol with Protocol Buffers
- **OpenD Gateway**: Local/remote gateway translates requests to Futu servers
- **Multi-language SDKs**: Python, Java, C#, C++, JavaScript
- **Order latency**: "as fast as 0.0014s" (advertised)

### Supported Markets
- Hong Kong (HKEX)
- United States (NYSE, NASDAQ, AMEX)
- China A-Shares (SSE, SZSE via Connect)
- Singapore (futures)
- Japan (futures)
- Australia (ASX)
- Malaysia (Bursa)
- Canada (TSX)

### Subscription Quota System
| Tier | Real-time Quota | Historical Quota | Criteria |
|------|----------------|------------------|----------|
| Basic | 100 | 100 | New account |
| Standard | 300 | 300 | Assets >10K HKD or high volume |
| High Volume | 1,000 | 1,000 | Assets >100K HKD or high volume |
| Premium | 2,000 | 2,000 | Assets >500K HKD or high volume |

### Rate Limits
- **Standard**: 60 requests per 30 seconds
- **Trading**: 15 requests per 30 seconds per account
- **High-frequency**: No limit after subscription (push-based)

### Trading Capabilities
- **Order Types**: Limit, Market, Stop, Stop Limit, MIT, LIT, Trailing
- **Markets**: HK, US, A-shares, SG, JP, AU, MY, CA
- **Paper Trading**: Full simulated accounts with real data
- **Extended Hours**: US pre-market, after-hours, overnight

### Unique Features
1. **Broker Queue Data** (HK) - LV2 subscription shows broker IDs at each price level
2. **Multi-Market Single API** - 8 markets in one unified API
3. **Capital Flow Analysis** - Large/medium/small order flow tracking (HK)
4. **Dark Pool Status** - Dark pool trading indicators (HK)
5. **Warrant Coverage** - Extensive HK warrant data
6. **IPO Data** - Upcoming IPO schedules and pricing

### Historical Data
- **Daily bars**: Up to 20 years
- **Intraday bars**: 1m, 3m, 5m, 15m, 30m, 1h (depth varies)
- **Adjustment types**: Forward, backward, unadjusted

### Pricing
- **API Usage**: FREE (no per-request charges)
- **Trading Fees**: Same as mobile app (no extra OpenAPI fee)
- **Quote Subscriptions**: $0-50/month (optional, for advanced market data)
  - HK LV2: ~$10-20/month (free for mainland China users)
  - US Nasdaq TotalView: ~$30-50/month
  - A-share LV1: Free (mainland China) / Paid (others)

---

## Implementation Considerations

### Architecture Differences
Unlike standard REST/WebSocket exchanges, Futu requires:
1. **OpenD Gateway**: Must be running (local or remote)
2. **Persistent TCP Connection**: Not HTTP request/response
3. **Protocol Buffers**: Binary serialization format
4. **SDK Wrappers**: Must use official SDKs or implement protocol

### Rust Implementation Challenges
- **No Official Rust SDK**: Must implement custom protocol client
- **Protocol Buffers**: Need to use `prost` or `protobuf` crate
- **TCP Client**: Persistent connection management
- **Callback System**: Push-based updates require async handling

### Alternative Approaches
1. **Use Python SDK via PyO3**: Wrap official Python SDK in Rust
2. **Implement Protocol Client**: Reverse-engineer protocol (complex)
3. **Use OpenD + REST Wrapper**: Run OpenD, create local REST server (workaround)

### Recommended Approach for V5
**Option 1 (Easiest)**: Python SDK wrapper
- Use PyO3 to call official Python SDK
- Minimal protocol knowledge required
- Battle-tested official SDK
- Cons: Python dependency

**Option 2 (Advanced)**: Native Rust protocol client
- Implement Protocol Buffer client
- Parse OpenD protocol definitions
- Handle TCP connection, heartbeat, reconnection
- Cons: High complexity, no official spec

**Option 3 (Pragmatic)**: OpenD + HTTP bridge
- Run OpenD gateway
- Create thin HTTP/WebSocket bridge in Rust
- Bridge translates REST → OpenD protocol
- Cons: Extra layer, OpenD dependency

---

## Critical Notes for Implementation

### Authentication Flow
1. User must have Futubull/moomoo account
2. User must download and configure OpenD
3. OpenD authenticates to Futu servers (user credentials)
4. Client connects to OpenD (local: no auth, remote: RSA key)
5. For trading: Must unlock with trade password

### Subscription Requirements
- **Must subscribe before getting data**: `subscribe()` → `get_stock_quote()`
- **Quota management critical**: Limited subscriptions based on account tier
- **1-minute wait**: Must wait 1 minute after subscribe before unsubscribe

### Rate Limit Handling
- **Client-side throttling recommended**: Track requests, don't rely on server errors
- **Trading operations stricter**: 15/30s for orders, 0.02s minimum gap
- **Exponential backoff**: Implement retry logic for "freq limit" errors

### Data Quality
- **Real-time, not delayed**: No 15-minute delay (subject to quote rights)
- **Exchange-sourced**: High quality, direct from exchanges
- **Corporate actions**: Automatically adjusted (via adjustment type parameter)

### Market Hours
- **Must check market status**: Use `get_market_state()` before trading
- **Extended hours**: Requires explicit parameters (`extended_time`, `session`)
- **Market-specific**: Each market has different hours

---

## Next Steps

### Phase 2: Implementation
After research approval, proceed to implementation:

1. **Choose Implementation Approach** (PyO3 wrapper vs native protocol)
2. **Create Module Structure**: `futu/mod.rs`, `endpoints.rs`, `auth.rs`, `parser.rs`, `connector.rs`
3. **Implement Core Traits**: `MarketData`, `Trading`, `Account` (if trading supported)
4. **Handle OpenD Connection**: Connection management, reconnection logic
5. **Implement Subscription System**: Track quotas, manage subscriptions
6. **Rate Limiting**: Client-side rate limiter
7. **Error Handling**: Map Futu errors to `ExchangeError`

### Phase 3: Testing
1. **Unit Tests**: Mock OpenD responses
2. **Integration Tests**: Against paper trading account
3. **Subscription Tests**: Quota management, rotation
4. **Rate Limit Tests**: Verify throttling works
5. **Reconnection Tests**: Connection drops, OpenD restarts

### Phase 4: Documentation
1. **Setup Guide**: How to install OpenD, configure account
2. **Usage Examples**: Subscribe, get quotes, place orders
3. **Troubleshooting**: Common errors and solutions

---

## Resources

- **Official Docs**: https://openapi.futunn.com/futu-api-doc/en/
- **Python SDK**: https://github.com/FutunnOpen/py-futu-api
- **OpenD Download**: https://www.futunn.com/en/download/OpenAPI
- **Help Center**: https://www.futuhk.com/en/support/
- **Community**: https://q.futunn.com/en/

---

## Research Completeness

- [x] All 8 research files created
- [x] Every endpoint documented (100+ methods)
- [x] All subscription types cataloged
- [x] Complete tier/quota system documented
- [x] Exact response formats from official docs
- [x] Trading capabilities fully cataloged
- [x] Market coverage clearly specified
- [x] OpenD custom protocol architecture explained
- [x] Implementation considerations noted
- [x] Alternative approaches evaluated

**Status**: ✅ RESEARCH COMPLETE - Ready for Phase 2 (Implementation)
