# Phase 5: Integration Test Agent Prompt (Data Providers)

## Agent Type
`rust-implementer`

## Variables
- `{PROVIDER}` - Provider name in lowercase (e.g., "polygon")
- `{Provider}` - Provider name in PascalCase (e.g., "Polygon")
- `{CATEGORY}` - Category folder (aggregators/forex/stocks/data_feeds)

---

## Prompt

```
Write integration tests for {PROVIDER} data provider that test REAL API functionality with LIVE data.

Unlike Phase 3 unit tests (which test parsing and basic connectivity), these tests validate:
- Real data accuracy and quality
- Historical data completeness
- Real-time feed reliability
- Data coverage across multiple instruments
- Rate limit behavior under realistic load
- Error recovery and reconnection

NOTE: Data providers are typically READ-ONLY. No trading tests required unless this is a broker API (Alpaca, Zerodha, OANDA).

═══════════════════════════════════════════════════════════════════════════════
REFERENCES
═══════════════════════════════════════════════════════════════════════════════

Unit tests: tests/{PROVIDER}_integration.rs (Phase 3 — already passing)
Research: src/{CATEGORY}/{PROVIDER}/research/

═══════════════════════════════════════════════════════════════════════════════
FILE: tests/{PROVIDER}_live.rs
═══════════════════════════════════════════════════════════════════════════════

## Setup
- All tests marked #[ignore] by default (run with --ignored flag)
- Require real API credentials from environment (if applicable)
- Use free tier or test credentials when available
- No cleanup needed (read-only operations)

## Required Integration Tests:

### Data Accuracy
- test_live_price_reasonableness
  - Get price for major instrument (AAPL for stocks, BTC/USDT for crypto, EUR/USD for forex)
  - Assert: price is within reasonable range (e.g., AAPL: $50-$500, BTC: $10k-$100k)
  - Assert: price is not zero or negative

- test_live_ticker_completeness
  - Get ticker for major instrument
  - Assert: last price exists
  - Assert: bid/ask exists and bid < ask
  - Assert: volume is non-negative
  - Assert: 24h high >= low

- test_live_orderbook_quality
  - Get orderbook (if supported)
  - Assert: bid < ask (no crossed orders)
  - Assert: at least 3 levels on each side
  - Assert: prices are sorted correctly (bids descending, asks ascending)

### Historical Data Quality
- test_live_klines_completeness
  - Get last 100 1-hour candles for major instrument
  - Assert: returned at least 50 candles (accounting for market hours)
  - Assert: timestamps are sequential
  - Assert: no gaps > 2 hours (except weekends for stocks)
  - Assert: OHLC relationships valid (L <= O,C <= H)

- test_live_klines_accuracy
  - Get last 24 hours of 1-minute data
  - Assert: close[i] ≈ open[i+1] (within 1% for liquid instruments)
  - Assert: volume is non-negative
  - Assert: no missing data during market hours

### Data Coverage
- test_live_instrument_coverage
  - Query multiple instruments (e.g., AAPL, MSFT, TSLA for stocks)
  - Assert: at least 80% return valid data
  - Document which instruments failed (may be delisted/unavailable)

- test_live_data_freshness
  - Get latest ticker/price
  - Assert: timestamp is within last 60 seconds (for real-time providers)
  - Or: timestamp is today (for delayed providers)

### WebSocket Live Stream (if supported)
- test_live_ws_ticker_accuracy
  - Subscribe to ticker via WS
  - Simultaneously fetch REST ticker
  - Assert: WS price within 1% of REST price (accounting for delay)
  - Stream for 60 seconds

- test_live_ws_data_continuity
  - Subscribe to trades or ticker
  - Assert: updates arrive within 30 seconds
  - Stream for 60 seconds
  - Assert: no connection drops

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

### Trading/Account (ONLY for broker APIs like Alpaca, Zerodha, OANDA)
- test_live_balance_check (if applicable)
  - Get account balance
  - Assert: at least one non-zero balance

- test_live_unsupported_operations (for non-broker providers)
  - Attempt place_order()
  - Assert: returns UnsupportedOperation error
  - Attempt cancel_order()
  - Assert: returns UnsupportedOperation error

═══════════════════════════════════════════════════════════════════════════════
PATTERNS
═══════════════════════════════════════════════════════════════════════════════

## Credential Loading (if API key required)
```rust
fn load_live_credentials() -> Option<Credentials> {
    let key = std::env::var("{PROVIDER}_API_KEY").ok()?;
    Some(Credentials::new(key, String::new(), None))
}

fn skip_if_no_credentials() -> Credentials {
    load_live_credentials().expect("Skipping: {PROVIDER}_API_KEY not set")
}
```

## No Credentials (public APIs)
```rust
fn create_connector() -> {Provider}Connector {
    {Provider}Connector::new(None)
}
```

## Ignore by Default
```rust
#[tokio::test]
#[ignore] // Run with: cargo test --test {PROVIDER}_live -- --ignored --nocapture
async fn test_live_price_reasonableness() {
    // ...
}
```

## Instrument Selection by Category
```rust
// Stocks: use tickers without quote asset
let symbol = "AAPL";

// Forex: use pairs with separator
let symbol = "EUR/USD";

// Crypto aggregators: use standard pairs
let symbol = "BTC/USDT";

// Data feeds: use provider-specific identifiers
let symbol = "SP500"; // for fred
let symbol = "binance:BTCUSDT"; // for coinglass
```

═══════════════════════════════════════════════════════════════════════════════
RUN
═══════════════════════════════════════════════════════════════════════════════

# Compile check
cargo test --package digdigdig3 --test {PROVIDER}_live --no-run

# Run live tests (may require API keys)
cargo test --package digdigdig3 --test {PROVIDER}_live -- --ignored --nocapture
```

---

## Exit Criteria
- test file created: tests/{PROVIDER}_live.rs
- Tests compile (--no-run passes)
- Tests are marked #[ignore] by default
- Test selection appropriate for provider category (no trading tests for read-only providers)
