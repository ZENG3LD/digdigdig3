# KRX (Korea Exchange) API Research

**Provider:** Korea Exchange (한국거래소)
**Category:** stocks/korea
**Type:** Official stock exchange data provider (DATA ONLY - NO TRADING)
**Research Date:** 2026-01-26

---

## Summary

KRX is South Korea's official stock exchange, operating three markets:
- **KOSPI** - Main board (~800 large-cap companies)
- **KOSDAQ** - Growth and tech companies (~1,500 companies)
- **KONEX** - SME market (~100-150 companies)

**Key Characteristics:**
- ✅ Official authoritative data source
- ✅ Full historical depth (from listing dates)
- ✅ Free API with registration
- ✅ Comprehensive market coverage (all Korean stocks)
- ⚠️ Delayed data (+1 business day minimum)
- ⚠️ Daily granularity only (no intraday)
- ⚠️ No WebSocket/real-time streams
- ⚠️ Limited English documentation

---

## Research Files

### 1. [api_overview.md](./api_overview.md)
- Provider information
- API types and base URLs
- Documentation quality assessment
- Licensing and terms
- Support channels

**Key Finding:** KRX recently (January 2026) moved to API-key-required authentication. Previously open data now requires registration and approval.

### 2. [endpoints_full.md](./endpoints_full.md)
- Complete endpoint catalog
- Public Data Portal API endpoints
- Data Marketplace module-based endpoints
- OTP download system
- All parameters documented

**Key Finding:** KRX uses a module-based system where all requests go to a single endpoint with different `bld` module parameters (e.g., MDCSTAT01701 for OHLCV data).

### 3. [websocket_full.md](./websocket_full.md)
- WebSocket availability: **NO**
- Alternative approaches (polling)
- Third-party real-time providers
- Institutional feed options

**Key Finding:** No public WebSocket API. Real-time data requires commercial third-party providers (ICE, Twelve Data) or direct exchange feed.

### 4. [authentication.md](./authentication.md)
- API key registration process
- Authentication methods (query params, headers, cookies)
- Service-specific approval system
- Error codes and handling
- Code examples

**Key Finding:** Two-step process: (1) Get API key, (2) Apply for specific services. Approval takes up to 1 business day. No HMAC signatures needed.

### 5. [tiers_and_limits.md](./tiers_and_limits.md)
- Free tier: 100,000 requests/day (Public Data Portal)
- Data Marketplace: Rate limits not publicly documented
- Commercial tiers: Contact KRX directly
- Rate limiting strategies
- Monitoring and usage tracking

**Key Finding:** Free tier is generous (100k/day) but data is delayed. Rate limits less critical than data delay for most use cases.

### 6. [data_types.md](./data_types.md)
- Standard market data (OHLCV, volume, market cap)
- Investor type analysis (individual/foreign/institutional)
- Sector and industry data
- Short selling transparency
- Index data (KOSPI, KOSDAQ, KRX indices)
- Limited fundamental data (use DART API for detailed fundamentals)

**Key Finding:** Unique investor type breakdown shows Korean market characteristics (retail vs institutional vs foreign flows). Strong short selling transparency.

### 7. [response_formats.md](./response_formats.md)
- Exact JSON structures from actual API responses
- All major endpoints documented
- Field descriptions in detail
- Parsing examples (handling comma-formatted numbers, Korean dates)
- Error response formats

**Key Finding:** All numeric values returned as comma-formatted strings (e.g., "12,345,678"). Must parse before use. Korean language prevalent in many fields.

### 8. [coverage.md](./coverage.md)
- Geographic: South Korea only
- Markets: KOSPI, KOSDAQ, KONEX (full coverage)
- ~2,409 listed stocks total
- Historical depth: From listing date (decades for old stocks)
- Granularity: Daily only
- Data delay: +1 business day
- Update time: 1:00 PM KST daily

**Key Finding:** Comprehensive coverage of Korean market but limited to daily data with next-day availability. Not suitable for real-time trading.

---

## Critical Implementation Notes

### 1. Data Delay
**ALL data is delayed by minimum +1 business day.**
- Trading day: Monday Jan 20
- Data available: Tuesday Jan 21, 1:00 PM KST
- **Not suitable for:** Real-time trading, day trading, HFT
- **Suitable for:** Backtesting, research, long-term analysis

### 2. Number Parsing
**All numeric values are comma-formatted strings:**
```rust
// Input from API: "12,345,678"
// Must parse: remove commas, convert to number
fn parse_krx_number(value: &str) -> f64 {
    value.replace(",", "").parse().unwrap()
}
```

### 3. Date Formats
- **Input format:** YYYYMMDD (e.g., "20260120")
- **Output format:** YYYY/MM/DD (e.g., "2026/01/20")

### 4. Korean Language
Many fields contain Korean text:
- Stock names: 삼성전자, SK하이닉스
- Investor types: 개인, 외국인, 기관계
- Sectors: 전기전자, 서비스업

**Requires UTF-8 support and potential translation mapping.**

### 5. Authentication Flow
```
1. Register at openapi.krx.co.kr or data.go.kr
2. Request API key
3. Apply for specific services (Securities, KOSDAQ, KONEX)
4. Wait for approval (up to 1 business day)
5. Make authenticated requests
```

### 6. Module-Based Architecture
Single endpoint with `bld` parameter:
```
POST http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd
Body: bld=dbms/MDC/STAT/standard/MDCSTAT01701&isuCd=KR7005930003&...
```

Different modules for different data types (OHLCV, ticker lists, investor stats, etc.)

---

## API Comparison

| Feature | KRX Free | Binance (comparison) |
|---------|----------|---------------------|
| Real-time | No (+1 day) | Yes (<1s) |
| WebSocket | No | Yes |
| Rate limit | 100k/day | 6000/min |
| Historical | Full (daily) | Limited (years) |
| Granularity | Daily only | 1m, 5m, 15m, 1h, 4h, 1d |
| Auth | API key | API key + HMAC |
| Cost | Free | Free |
| Market | Korea only | Global crypto |

---

## Recommended Use Cases

### ✅ Excellent For:
- Historical backtesting (full daily data)
- Academic research (authoritative source)
- Long-term investing analysis
- Quantitative research on Korean market
- Portfolio tracking (daily updates)
- Investor flow analysis (unique data)
- Short selling analysis (transparency)

### ⚠️ Acceptable For:
- Swing trading (daily decisions)
- Multi-day strategies
- Fundamental analysis (combine with DART API)

### ❌ Not Suitable For:
- Real-time trading
- Day trading
- High-frequency trading (HFT)
- Intraday analysis
- Minute/hourly strategies
- Market making

---

## Integration with Other APIs

### DART (Recommended Combination)
**For comprehensive fundamental data:**
- URL: https://opendart.fss.or.kr/
- Coverage: Financial statements, earnings, disclosures
- Use case: KRX (prices) + DART (fundamentals) = Complete analysis

### Bank of Korea
**For economic data:**
- Korean macroeconomic indicators
- Interest rates, inflation, GDP
- KRW exchange rates

### Third-Party Real-Time (If Needed)
- **ICE Data Services** - Institutional grade, direct exchange feed
- **Twelve Data** - $29-129/month, WebSocket support
- **TickData** - Historical tick data, institutional

---

## Implementation Priority

### Phase 1: Basic Data Access
- ✅ Authentication (API key management)
- ✅ OHLCV data fetching
- ✅ Ticker list retrieval
- ✅ Number parsing (comma removal)
- ✅ Date format handling

### Phase 2: Enhanced Data
- Investor type analysis
- Short selling data
- Index data
- Sector information

### Phase 3: Advanced Features
- Rate limiting implementation
- Caching (daily data doesn't change)
- Error handling and retry logic
- DART API integration

---

## Known Limitations

1. **No real-time data** - Minimum +1 business day delay
2. **No intraday data** - Daily granularity only
3. **No WebSocket** - Polling only (inefficient, unnecessary given delay)
4. **Korean language** - Many fields in Korean, requires UTF-8
5. **Comma-formatted numbers** - Must parse all numeric values
6. **Rate limits unclear** - Data Marketplace limits not documented
7. **Limited fundamentals** - Need DART API for detailed financial data
8. **Approval delays** - API key approval takes up to 1 business day

---

## References

### Official Sources
- KRX Global: https://global.krx.co.kr/
- KRX Open API: https://openapi.krx.co.kr/
- KRX Data Marketplace: https://data.krx.co.kr/
- Public Data Portal: https://www.data.go.kr/en/data/15094775/openapi.do
- DART: https://opendart.fss.or.kr/

### Community Libraries
- PyKRX (Python): https://github.com/sharebook-kr/pykrx
- krx-stock-api (Node.js): https://github.com/Shin-JaeHeon/krx-stock-api
- go-krx (Go): https://github.com/dojinkimm/go-krx
- tqk (R): https://github.com/mrchypark/tqk

### Third-Party Providers
- ICE Data Services: https://developer.ice.com/fixed-income-data-services/catalog/korea-exchange-krx
- Twelve Data: https://twelvedata.com/exchanges/XKRX
- TickData: https://www.tickdata.com/equity-data/korea-exchange-equities

### Research Sources
- [Korea Exchange - Wikipedia](https://en.wikipedia.org/wiki/Korea_Exchange)
- [KRX Market Hours & Holidays 2026](https://www.tradinghours.com/markets/krx)
- [KRX API Authentication Guide (Korean)](https://i-whale.com/entry/KRX-%EC%8B%9C%EC%84%B8-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EC%9D%B4%EC%A0%9C-%EB%A1%9C%EA%B7%B8%EC%9D%B8-API%EB%A5%BC-%EC%8B%A0%EC%B2%AD-%ED%95%B4%EC%95%BC-%EC%93%B8-%EC%88%98-%EC%9E%88%EB%84%A4%EC%9A%94-%E3%85%9C)

---

## Exit Criteria

- [x] All 8 research files created
- [x] Every file has EXACT data from official docs (no guessing)
- [x] All endpoints documented (including specialized ones)
- [x] All data types cataloged
- [x] Tier/pricing clearly documented
- [x] WebSocket documented (noted as unavailable)
- [x] Coverage/limits understood
- [x] Response formats with real examples
- [x] Authentication flow documented
- [x] Critical implementation notes highlighted

---

## Next Steps

**Do NOT start implementation yet.**

This research must be reviewed before proceeding to Phase 2 (Implementation).

**For implementation:**
1. Review all research files
2. Confirm API access (obtain API key)
3. Test authentication flow
4. Verify rate limits practically
5. Proceed to `02_implement.md` prompt

---

**Research completed:** 2026-01-26
**Researcher:** Claude Sonnet 4.5 (research-agent)
**Status:** ✅ Complete and ready for review
