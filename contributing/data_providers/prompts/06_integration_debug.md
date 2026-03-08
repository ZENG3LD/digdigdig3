# Phase 6: Integration Debug Agent Prompt (Data Providers)

## Agent Type
`rust-implementer`

## Variables
- `{PROVIDER}` - Provider name in lowercase (e.g., "polygon")
- `{CATEGORY}` - Category folder (aggregators/forex/stocks/data_feeds)

---

## Prompt

```
Debug and fix failing INTEGRATION tests for {PROVIDER} data provider.

These are LIVE tests that hit the REAL provider API. Unlike Phase 4 (unit test debug),
issues here are typically:
- Real API response format differs from documentation
- Data quality issues (missing fields, stale data, gaps)
- Rate limits triggered during test runs
- WebSocket connections dropped by provider
- Free tier vs paid tier feature differences
- Market hours affecting data availability

═══════════════════════════════════════════════════════════════════════════════
PROCESS
═══════════════════════════════════════════════════════════════════════════════

1. Run integration tests:
   cargo test --package digdigdig3 --test {PROVIDER}_live -- --ignored --nocapture

2. For EACH failure, identify and fix:

═══════════════════════════════════════════════════════════════════════════════
COMMON INTEGRATION ISSUES (DATA PROVIDERS)
═══════════════════════════════════════════════════════════════════════════════

## Data Accuracy/Quality Issues
- Provider may have delayed data → adjust timestamp expectations
- Some instruments unavailable/delisted → skip gracefully or use different symbols
- Bid/ask may be missing for illiquid instruments → make fields optional in tests
- Historical data may have gaps (market closures) → widen tolerance

## Coverage Issues
- Free tier may limit available instruments → reduce test coverage expectations
- Some symbols may use different format → check research/symbols.md
- Regional instruments may not be available → document and skip

## Historical Data Issues
- Klines may not go back as far as expected → reduce requested history
- Weekend/holiday gaps in stocks/forex → account for market hours
- Data aggregation differences (1h might be 4x 15m vs actual 1h candles)

## WebSocket Issues
- WS may not be available on free tier → skip WS tests gracefully
- Provider may rate-limit WS connections → add delays between tests
- Ping interval too long/short → check actual keepalive requirements
- Message format differs from docs → log and adapt

## Rate Limit Issues
- Free tier may have strict limits → add delays: tokio::time::sleep(Duration::from_millis(1000))
- Some providers count failed requests → reduce rapid test count
- Rate limit window may be per-minute not per-second → add longer delays

## API Key/Authentication Issues
- Public endpoints may work, authenticated may fail → test both paths
- API key permissions may be restricted → document what's testable
- IP whitelist may block → document in test output

## Market Hours Issues (Stocks/Forex)
- Tests may run outside market hours → check if data is available
- Delayed data providers only update during market hours
- Weekend forex markets closed → skip or use crypto pairs

═══════════════════════════════════════════════════════════════════════════════
LOOP UNTIL ALL PASS
═══════════════════════════════════════════════════════════════════════════════

Repeat:
1. Run all integration tests
2. Pick first failure
3. Identify: is it a code bug, data quality issue, or test expectation issue?
4. Fix appropriately:
   - Code bug → fix in connector code
   - Data quality issue → adapt test expectations or use different instruments
   - Test expectation → widen tolerances, add market hours checks, or skip gracefully
   - Rate limit → add delays between tests
5. Run single test to verify fix
6. Run all integration tests
7. If failures remain, go to 2

EXIT only when:
cargo test --package digdigdig3 --test {PROVIDER}_live -- --ignored --nocapture

Shows: test result: ok. N passed; 0 failed
Or: All remaining failures are gracefully skipped with documented reasons

═══════════════════════════════════════════════════════════════════════════════
GRACEFUL SKIP PATTERN
═══════════════════════════════════════════════════════════════════════════════

When test cannot pass due to external factors:

```rust
#[tokio::test]
#[ignore]
async fn test_live_ws_ticker_accuracy() {
    let connector = create_connector();

    // Check if WS is available
    match connector.subscribe_ticker("AAPL").await {
        Err(e) if e.to_string().contains("not supported") => {
            println!("⏭️  SKIPPED: WebSocket not available on free tier");
            return; // Test passes by skipping
        }
        Err(e) => panic!("Failed to subscribe: {}", e),
        Ok(_) => {
            // Continue with test
        }
    }

    // ... rest of test
}
```

Document skips in test output:
```
✓ test_live_price_reasonableness ... ok
✓ test_live_ticker_completeness ... ok
⏭️  test_live_ws_ticker_accuracy ... ok (skipped: WS not available on free tier)
✓ test_live_klines_completeness ... ok
```

═══════════════════════════════════════════════════════════════════════════════
PROVIDER-SPECIFIC NOTES
═══════════════════════════════════════════════════════════════════════════════

## Stocks (polygon, finnhub, alpaca, etc.)
- Market hours: 9:30 AM - 4:00 PM ET, Mon-Fri
- Use major tickers: AAPL, MSFT, TSLA, SPY
- Weekend data will be stale
- Some providers delay 15+ minutes

## Forex (alphavantage, oanda, dukascopy)
- 24/5 markets (closed weekends)
- Use major pairs: EUR/USD, GBP/USD, USD/JPY
- Spread varies by liquidity
- Pair format differs by provider

## Aggregators (cryptocompare, defillama, yahoo)
- May aggregate multiple sources → data can differ
- Symbol mapping critical
- Coverage varies widely

## Data Feeds (coinglass, fred, bitquery, whale_alert)
- Specialized data types
- May have unique response formats
- Often read-only
- Rate limits can be strict

```

---

## Exit Criteria
- ALL live tests pass (or are gracefully skipped with documented reasons)
- No panics or crashes
- Connector works with real provider data
- Skip reasons are clear and documented
