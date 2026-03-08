# Phase 5: Integration Test Agent Prompt

## Agent Type
`rust-implementer`

## Variables
- `{EXCHANGE}` - Exchange name in lowercase (e.g., "bybit")
- `{Exchange}` - Exchange name in PascalCase (e.g., "Bybit")

---

## Prompt

```
Write integration tests for {EXCHANGE} connector that test REAL API functionality with LIVE data.

Unlike Phase 3 unit tests (which test parsing and basic connectivity), these tests validate:
- Real order placement and cancellation (with minimal amounts)
- Real balance checking
- Real WebSocket data streams over extended periods
- Real market data accuracy (compare REST vs WS prices)
- Rate limit behavior under realistic load
- Error recovery and reconnection

═══════════════════════════════════════════════════════════════════════════════
REFERENCES
═══════════════════════════════════════════════════════════════════════════════

Unit tests: tests/{EXCHANGE}_integration.rs (Phase 3 — already passing)
Research: src/exchanges/{EXCHANGE}/research/

═══════════════════════════════════════════════════════════════════════════════
FILE: tests/{EXCHANGE}_live.rs
═══════════════════════════════════════════════════════════════════════════════

## Setup
- All tests marked #[ignore] by default (run with --ignored flag)
- Require real API credentials from environment
- Use TESTNET/sandbox if available, otherwise use MINIMAL amounts on mainnet
- Include cleanup logic (cancel orders, etc.)

## Required Integration Tests:

### Data Accuracy
- test_live_price_matches_ticker
  - Get price via get_price() and get_ticker()
  - Assert: prices are within 0.1% of each other
  - Assert: prices are in reasonable range (BTC > $10k, < $1M)

- test_live_orderbook_spread
  - Get orderbook, verify spread is < 1% for BTC/USDT
  - Assert: bid < ask
  - Assert: at least 5 levels on each side

- test_live_klines_continuity
  - Get last 100 1-minute candles
  - Assert: timestamps are sequential
  - Assert: no gaps > 2 minutes
  - Assert: close[i] == open[i+1] (approximately)

### WebSocket Live Stream
- test_live_ws_ticker_accuracy
  - Subscribe to ticker via WS
  - Simultaneously fetch REST ticker
  - Assert: WS price within 0.5% of REST price
  - Stream for 60 seconds

- test_live_ws_orderbook_updates
  - Subscribe to orderbook
  - Verify updates arrive within 5 seconds
  - Verify orderbook stays consistent (no crossed orders)
  - Stream for 60 seconds

- test_live_ws_reconnection
  - Connect and subscribe
  - Force disconnect (if possible) or wait for natural disconnect
  - Verify automatic reconnection
  - Verify data resumes

### Rate Limiting
- test_live_rate_limit_handling
  - Send 20 rapid requests
  - Assert: either all succeed or rate limit error is properly returned
  - Assert: no panics or connection drops

### Trading (if credentials available, TESTNET preferred)
- test_live_place_and_cancel_order
  - Place a limit buy order at 50% below market (won't fill)
  - Assert: order ID returned
  - Cancel the order
  - Assert: cancellation confirmed
  - Query open orders
  - Assert: order is gone

- test_live_balance_check
  - Get account balance
  - Assert: at least one non-zero balance (even if tiny)

═══════════════════════════════════════════════════════════════════════════════
PATTERNS
═══════════════════════════════════════════════════════════════════════════════

## Credential Loading
```rust
fn load_live_credentials() -> Option<Credentials> {
    let key = std::env::var("{EXCHANGE}_API_KEY").ok()?;
    let secret = std::env::var("{EXCHANGE}_API_SECRET").ok()?;
    Some(Credentials::new(key, secret, None))
}

fn skip_if_no_credentials() -> Credentials {
    load_live_credentials().expect("Skipping: {EXCHANGE}_API_KEY not set")
}
```

## Ignore by Default
```rust
#[tokio::test]
#[ignore] // Run with: cargo test --test {EXCHANGE}_live -- --ignored --nocapture
async fn test_live_price_matches_ticker() {
    // ...
}
```

═══════════════════════════════════════════════════════════════════════════════
RUN
═══════════════════════════════════════════════════════════════════════════════

# Compile check
cargo test --package digdigdig3 --test {EXCHANGE}_live --no-run

# Run live tests (requires API keys)
cargo test --package digdigdig3 --test {EXCHANGE}_live -- --ignored --nocapture
```

---

## Exit Criteria
- test file created: tests/{EXCHANGE}_live.rs
- Tests compile (--no-run passes)
- Tests are marked #[ignore] by default
