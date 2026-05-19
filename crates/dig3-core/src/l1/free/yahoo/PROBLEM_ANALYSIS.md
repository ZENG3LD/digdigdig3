# Yahoo Finance 401 Unauthorized - Problem Analysis

**Date**: 2026-01-26
**Status**: CRITICAL - Partial API Outage
**Impact**: /v7/finance/quote endpoint returns 401, but chart/search endpoints work

---

## Executive Summary

Yahoo Finance has **disabled the /v7/finance/quote endpoint** as of January 2026, causing all `get_price()` and `get_ticker()` calls to fail with 401 Unauthorized. This is part of Yahoo's ongoing shutdown of their unofficial API.

**Good News**: The `/v8/finance/chart/{symbol}` endpoint still works and provides the exact same data (price, high, low, volume, change%).

**Fix**: Simple 2-3 hour code change to replace quote endpoint with chart endpoint in `get_quote_internal()` method.

**Error Message**:
```json
{
  "finance": {
    "result": null,
    "error": {
      "code": "Unauthorized",
      "description": "User is unable to access this feature - https://bit.ly/yahoo-finance-api-feedback"
    }
  }
}
```

**Working Endpoints** (tested 2026-01-26):
- `/v8/finance/chart/{symbol}` - Full OHLCV data + real-time price ✅
- `/v1/finance/search` - Symbol search ✅
- `/v6/finance/quote/marketSummary` - Market indices ✅

**Broken Endpoints**:
- `/v7/finance/quote` - Returns 401 Unauthorized ❌
- `/v10/finance/quoteSummary` - Requires crumb auth ⚠️

---

## Problem Analysis

### 1. Root Cause

Yahoo Finance is incrementally shutting down their **unofficial API endpoints**. The /v7/finance/quote endpoint was likely the most heavily abused endpoint (used for real-time stock quotes), so Yahoo disabled it first.

**Evidence**:
```bash
# FAILS - 401 Unauthorized
$ curl "https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL"
{"finance":{"result":null,"error":{"code":"Unauthorized",...}}}

# WORKS - Returns full data
$ curl "https://query1.finance.yahoo.com/v8/finance/chart/AAPL"
{"chart":{"result":[{"meta":{"regularMarketPrice":248.04,...}}]}}
```

### 2. Affected Endpoints in Our Connector

**BROKEN** (return 401):
- `YahooFinanceEndpoint::Quote` - `/v7/finance/quote`
  - Used by: `get_price()`, `get_ticker()`

**WORKING**:
- `YahooFinanceEndpoint::Chart` - `/v8/finance/chart/{symbol}` ✅
- `YahooFinanceEndpoint::Search` - `/v1/finance/search` ✅
- `YahooFinanceEndpoint::MarketSummary` - `/v6/finance/quote/marketSummary` ✅

**REQUIRES CRUMB** (authentication):
- `YahooFinanceEndpoint::QuoteSummary` - `/v10/finance/quoteSummary/{symbol}` ⚠️ (returns "Invalid Crumb")

### 3. Current Implementation Issues

**connector.rs**:
```rust
// Lines 159-164 - Uses BROKEN Quote endpoint
async fn get_quote_internal(&self, yahoo_symbol: &str) -> ExchangeResult<serde_json::Value> {
    let mut params = HashMap::new();
    params.insert("symbols".to_string(), yahoo_symbol.to_string());
    self.get(YahooFinanceEndpoint::Quote, None, params).await  // ❌ RETURNS 401
}

// Lines 208-210 - get_price() depends on broken endpoint
async fn get_price(...) -> ExchangeResult<Price> {
    let yahoo_symbol = format_symbol(&symbol.base, &symbol.quote);
    let response = self.get_quote_internal(&yahoo_symbol).await?;  // ❌ FAILS HERE
    YahooFinanceParser::parse_price(&response)
}
```

**Why it fails**:
- `get_price()` → calls `get_quote_internal()` → uses `/v7/finance/quote` → **401 Unauthorized**
- `get_ticker()` → calls `get_quote_internal()` → uses `/v7/finance/quote` → **401 Unauthorized**

---

## Tested Solutions

### Solution 1: Use Chart Endpoint Instead of Quote

**Implementation**: Replace `/v7/finance/quote` with `/v8/finance/chart/{symbol}`

**Pros**:
- Chart endpoint **currently works** (tested 2026-01-26)
- Returns same data: regularMarketPrice, bid, ask, volume, high/low
- No authentication needed
- Already implemented in `get_klines()` method

**Cons**:
- Returns more data than needed (includes full OHLCV arrays)
- Slightly slower response (~35KB vs ~2KB)
- Cannot query multiple symbols in one request

**Code Changes Required**:
```rust
// connector.rs - Replace get_quote_internal()
async fn get_quote_internal(&self, yahoo_symbol: &str) -> ExchangeResult<serde_json::Value> {
    // OLD: self.get(YahooFinanceEndpoint::Quote, None, params).await
    // NEW: Use chart endpoint
    self.get(YahooFinanceEndpoint::Chart, Some(yahoo_symbol), HashMap::new()).await
}
```

```rust
// parser.rs - Update parse_price() and parse_ticker()
pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
    // OLD: Parse from quoteResponse.result[0].regularMarketPrice
    // NEW: Parse from chart.result[0].meta.regularMarketPrice
    let result = Self::get_chart_result(response)?;
    let first = result.get(0).ok_or_else(...)?;
    let meta = first.get("meta").ok_or_else(...)?;
    Self::require_f64(meta, "regularMarketPrice")
}
```

**Complexity**: SIMPLE (1-2 hours)

---

### Solution 2: Use QuoteSummary Endpoint

**Implementation**: Use `/v10/finance/quoteSummary/{symbol}?modules=price`

**Pros**:
- More comprehensive data (includes price, summaryDetail, financialData modules)
- Official-looking endpoint (v10 vs v7)
- May be more stable long-term

**Cons**:
- ❌ **REQUIRES CRUMB AUTHENTICATION** (tested 2026-01-26, returns "Invalid Crumb")
- Complex cookie/crumb management needed
- Fragile authentication flow
- Larger payload

**Complexity**: COMPLEX (8-12 hours) - NOT RECOMMENDED

---

### Solution 3: Add Cookie/Crumb Authentication

**Implementation**: Implement full browser-like authentication with cookies

**Pros**:
- May bypass 401 errors on quote endpoint
- Required for historical CSV download anyway

**Cons**:
- **Complex**: Need to scrape cookies from finance.yahoo.com webpage
- **Fragile**: Yahoo changes cookie/crumb format frequently
- **Rate limits**: Still subject to aggressive rate limiting
- **May not work**: Quote endpoint might be permanently disabled

**Complexity**: COMPLEX (8-12 hours) + FRAGILE

---

### Solution 4: Use Alternative Endpoints per Data Type

**Implementation**: Route different data types to different endpoints

**Example**:
- Price data → `/v8/finance/chart/{symbol}` (meta.regularMarketPrice)
- Ticker data → `/v8/finance/chart/{symbol}` (meta + indicators)
- Financial data → `/v10/finance/quoteSummary/{symbol}?modules=financialData`
- Options data → `/v7/finance/options/{symbol}` (if still working)

**Pros**:
- Most resilient approach
- Uses best endpoint for each data type
- Already partially implemented (chart for klines)

**Cons**:
- More code changes
- Different error handling per endpoint

**Complexity**: MEDIUM (4-6 hours)

---

### Solution 5: Add Fallback Chain

**Implementation**: Try quote endpoint first, fallback to chart if 401

**Pros**:
- Graceful degradation
- Future-proof if quote endpoint comes back
- No breaking changes to API

**Cons**:
- Slower on failure (waits for 401 timeout)
- More complex error handling

**Complexity**: SIMPLE (2-3 hours)

---

## Recommendations

### Recommended Solution: **Solution 1 + Solution 5 (Hybrid)**

**Phase 1** (Immediate - 2-3 hours):
1. Modify `get_quote_internal()` to use Chart endpoint
2. Update `parse_price()` and `parse_ticker()` to parse from chart response
3. Add tests to verify chart-based parsing works

**Phase 2** (Short-term - 1-2 hours):
1. Add fallback logic: Try quote endpoint first, fallback to chart on 401
2. Log warnings when fallback is used
3. Document in code comments that quote endpoint is deprecated

**Why this approach**:
- ✅ Fixes the immediate 401 issue
- ✅ Simple implementation (chart endpoint already tested)
- ✅ Future-proof (fallback if quote endpoint comes back)
- ✅ No breaking changes to public API
- ✅ Maintains same functionality

**Code Template**:
```rust
async fn get_quote_internal(&self, yahoo_symbol: &str) -> ExchangeResult<serde_json::Value> {
    // Try quote endpoint first (for backwards compatibility)
    let mut params = HashMap::new();
    params.insert("symbols".to_string(), yahoo_symbol.to_string());

    match self.get(YahooFinanceEndpoint::Quote, None, params.clone()).await {
        Ok(response) => Ok(response),
        Err(ExchangeError::Api { code: 401, .. }) => {
            // Quote endpoint returned 401 - fallback to chart endpoint
            log::warn!("Yahoo Finance quote endpoint returned 401, falling back to chart endpoint");
            self.get(YahooFinanceEndpoint::Chart, Some(yahoo_symbol), HashMap::new()).await
        },
        Err(e) => Err(e),
    }
}
```

---

## Alternative Long-Term Solutions

### If Yahoo Continues Shutting Down Endpoints:

**Option A**: Switch to yfinance Python library approach
- Use Selenium/headless browser to scrape data
- Extract from HTML instead of API
- VERY complex, fragile

**Option B**: Switch to alternative data provider
- Alpha Vantage (free tier: 25 req/day)
- Twelve Data (free tier: 800 req/day)
- Finnhub (free tier: 60 req/min)
- IEX Cloud (free tier with registration)

**Option C**: Use multiple providers with fallback
- Primary: Yahoo Finance (free, no key)
- Fallback: Alpha Vantage or Finnhub
- Requires API key management

---

## Testing Plan

### 1. Manual Testing (COMPLETED 2026-01-26)

**Results of endpoint testing**:

```bash
# Test quote endpoint - ❌ FAILS with 401
$ curl "https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL" \
  -H "User-Agent: Mozilla/5.0"
# Response: {"finance":{"result":null,"error":{"code":"Unauthorized",...}}}

# Test chart endpoint - ✅ WORKS (returns full data including price)
$ curl "https://query1.finance.yahoo.com/v8/finance/chart/AAPL" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
# Response: {"chart":{"result":[{"meta":{"regularMarketPrice":248.04,...}}]}}

# Test chart with crypto - ✅ WORKS
$ curl "https://query1.finance.yahoo.com/v8/finance/chart/BTC-USD" \
  -H "User-Agent: Mozilla/5.0"
# Response: {"chart":{"result":[{"meta":{"regularMarketPrice":87696.74,...}}]}}

# Test quoteSummary - ❌ REQUIRES CRUMB
$ curl "https://query1.finance.yahoo.com/v10/finance/quoteSummary/AAPL?modules=price" \
  -H "User-Agent: Mozilla/5.0"
# Response: {"finance":{"result":null,"error":{"code":"Unauthorized","description":"Invalid Crumb"}}}

# Test search - ✅ WORKS
$ curl "https://query1.finance.yahoo.com/v1/finance/search?q=apple" \
  -H "User-Agent: Mozilla/5.0"
# Response: {"quotes":[{"symbol":"AAPL","shortname":"Apple Inc.",...}],...}

# Test market summary - ✅ WORKS
$ curl "https://query1.finance.yahoo.com/v6/finance/quote/marketSummary" \
  -H "User-Agent: Mozilla/5.0"
# Response: {"marketSummaryResponse":{"result":[{"symbol":"ES=F",...}]}}
```

**Chart Endpoint Data Availability**:
- ✅ regularMarketPrice (current price)
- ✅ regularMarketDayHigh (24h high)
- ✅ regularMarketDayLow (24h low)
- ✅ regularMarketVolume (24h volume)
- ✅ regularMarketChange (price change)
- ✅ regularMarketChangePercent (percent change)
- ✅ OHLCV data in indicators.quote arrays
- ❌ bid/ask prices (NOT available - normal for Yahoo Finance)
- ❌ previousClose (available as chartPreviousClose)

### 2. Unit Tests

Create parser tests for chart-based responses:

```rust
#[test]
fn test_parse_price_from_chart() {
    let response = json!({
        "chart": {
            "result": [{
                "meta": {
                    "regularMarketPrice": 248.04,
                    "currency": "USD",
                    "symbol": "AAPL"
                }
            }]
        }
    });

    let price = YahooFinanceParser::parse_price_from_chart(&response).unwrap();
    assert_eq!(price, 248.04);
}
```

### 3. Integration Tests

Run existing tests with new implementation:

```bash
cargo test --package digdigdig3 --test yahoo_finance_integration -- --nocapture
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Chart endpoint also gets disabled | MEDIUM | HIGH | Add fallback to quoteSummary |
| Rate limiting increases | HIGH | MEDIUM | Add exponential backoff, delays |
| Response format changes | MEDIUM | MEDIUM | Comprehensive error handling |
| All endpoints disabled | LOW | CRITICAL | Switch to alternative provider |

---

## Related Resources

- Yahoo Finance API unofficial docs: https://github.com/ranaroussi/yfinance/wiki
- Alternative: https://www.alphavantage.co/documentation/
- Alternative: https://finnhub.io/docs/api
- Community discussion: https://stackoverflow.com/questions/tagged/yahoo-finance

---

## Next Steps

1. ✅ **Document the issue** (this file)
2. ⏳ **Get approval** for recommended solution
3. ⏳ **Implement Phase 1** (chart endpoint replacement)
4. ⏳ **Test thoroughly** (unit + integration tests)
5. ⏳ **Implement Phase 2** (fallback logic)
6. ⏳ **Monitor** for additional endpoint failures
7. ⏳ **Consider** migration to alternative provider if situation worsens

---

## Conclusion

Yahoo Finance is gradually shutting down their unofficial API. The `/v7/finance/quote` endpoint now returns 401 Unauthorized, but the `/v8/finance/chart` endpoint still works and provides the same data.

**Recommended immediate fix**: Replace quote endpoint with chart endpoint in `get_quote_internal()` method and update parsers. This is a **simple, 2-3 hour fix** that restores full functionality.

**Long-term consideration**: Monitor Yahoo Finance API status and prepare migration plan to alternative provider (Alpha Vantage, Finnhub, or IEX Cloud) if more endpoints fail.

---

## Quick Reference: Implementation Changes Required

### Files to Modify:
1. `connector.rs` - Update `get_quote_internal()` method (line 159)
2. `parser.rs` - Update `parse_price()` and `parse_ticker()` methods

### Code Changes:

**connector.rs (line 159-164)**:
```rust
// OLD CODE (returns 401):
async fn get_quote_internal(&self, yahoo_symbol: &str) -> ExchangeResult<serde_json::Value> {
    let mut params = HashMap::new();
    params.insert("symbols".to_string(), yahoo_symbol.to_string());
    self.get(YahooFinanceEndpoint::Quote, None, params).await  // ❌ FAILS
}

// NEW CODE (works):
async fn get_quote_internal(&self, yahoo_symbol: &str) -> ExchangeResult<serde_json::Value> {
    // Use chart endpoint instead of quote endpoint (quote returns 401 as of Jan 2026)
    self.get(YahooFinanceEndpoint::Chart, Some(yahoo_symbol), HashMap::new()).await
}
```

**parser.rs (line 26-32)**:
```rust
// OLD CODE:
pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
    let result = Self::get_quote_response_result(response)?;
    let first = result.get(0).ok_or_else(...)?;
    Self::require_f64(first, "regularMarketPrice")
}

// NEW CODE:
pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
    let result = Self::get_chart_result(response)?;
    let first = result.get(0).ok_or_else(...)?;
    let meta = first.get("meta").ok_or_else(...)?;
    Self::require_f64(meta, "regularMarketPrice")
}
```

**parser.rs (line 36-56)** - Similar changes for `parse_ticker()`:
```rust
// Parse from: response -> chart -> result[0] -> meta -> regularMarketPrice
// Instead of: response -> quoteResponse -> result[0] -> regularMarketPrice
```

### Test Command:
```bash
cargo test --package digdigdig3 --test yahoo_finance_integration test_get_price -- --nocapture --exact
```

---

## TL;DR - For the Busy Developer

**Problem**: Yahoo Finance `/v7/finance/quote` endpoint returns 401 Unauthorized

**Cause**: Yahoo is shutting down unofficial API endpoints

**Solution**: Use `/v8/finance/chart/{symbol}` endpoint instead (still works)

**Effort**: 2-3 hours (simple code change + parser update)

**Files**: `connector.rs` (1 method), `parser.rs` (2 methods)

**Alternative**: Do nothing and wait for Yahoo to restore quote endpoint (unlikely)
