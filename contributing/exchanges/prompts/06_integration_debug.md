# Phase 6: Integration Debug Agent Prompt

## Agent Type
`rust-implementer`

## Variables
- `{EXCHANGE}` - Exchange name in lowercase (e.g., "bybit")

---

## Prompt

```
Debug and fix failing INTEGRATION tests for {EXCHANGE} connector.

These are LIVE tests that hit the REAL exchange API. Unlike Phase 4 (unit test debug),
issues here are typically:
- Real API response format differs from documentation
- Rate limits triggered during test runs
- WebSocket connections dropped by exchange
- Testnet vs mainnet behavior differences
- Authentication edge cases not covered in docs

═══════════════════════════════════════════════════════════════════════════════
PROCESS
═══════════════════════════════════════════════════════════════════════════════

1. Run integration tests:
   cargo test --package digdigdig3 --test {EXCHANGE}_live -- --ignored --nocapture

2. For EACH failure, identify and fix:

═══════════════════════════════════════════════════════════════════════════════
COMMON INTEGRATION ISSUES
═══════════════════════════════════════════════════════════════════════════════

## Price/data accuracy failures
- Exchange may have low liquidity → widen tolerance (0.1% → 1%)
- Prices change between REST and WS calls → add timing tolerance
- Some pairs have very different orderbook depth

## WebSocket live stream issues
- Exchange rate-limits WS connections → add delays between tests
- Ping interval too long/short → check actual keepalive requirements
- Message format on mainnet differs from docs → log and adapt

## Rate limit hits
- Add delays between test functions: tokio::time::sleep(Duration::from_millis(500))
- Reduce number of rapid requests in rate limit test
- Check if exchange has separate limits for REST vs WS

## Trading test issues
- Testnet may be down → gracefully skip with warning
- Minimum order size differs → check exchange minimums
- Order might partially fill → handle partial fills in cancel logic

## Credential issues
- API key may have restricted permissions → test only what's allowed
- IP whitelist may block → document in test output
- Subaccount vs main account differences

═══════════════════════════════════════════════════════════════════════════════
LOOP UNTIL ALL PASS
═══════════════════════════════════════════════════════════════════════════════

Repeat:
1. Run all integration tests
2. Pick first failure
3. Identify: is it a code bug, API issue, or test expectation issue?
4. Fix appropriately:
   - Code bug → fix in connector code
   - API issue → adapt test expectations or add retry logic
   - Test expectation → widen tolerances or add skip conditions
5. Run single test to verify fix
6. Run all integration tests
7. If failures remain, go to 2

EXIT only when:
cargo test --package digdigdig3 --test {EXCHANGE}_live -- --ignored --nocapture

Shows: test result: ok. N passed; 0 failed
Or: All remaining failures are gracefully skipped with documented reasons
```

---

## Exit Criteria
- ALL live tests pass (or are gracefully skipped with documented reasons)
- No panics or crashes
- Connector works with real exchange data
