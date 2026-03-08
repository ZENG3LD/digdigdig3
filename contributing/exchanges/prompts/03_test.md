# Phase 3: Test Agent Prompt

## Agent Type
`rust-implementer`

## Variables
- `{EXCHANGE}` - Exchange name in lowercase (e.g., "bybit")
- `{Exchange}` - Exchange name in PascalCase (e.g., "Bybit")

---

## Prompt

```
Write comprehensive tests for {EXCHANGE} connector.

═══════════════════════════════════════════════════════════════════════════════
REFERENCES
═══════════════════════════════════════════════════════════════════════════════

REST tests reference: tests/kucoin_integration.rs
WebSocket tests reference: tests/kucoin_websocket.rs

═══════════════════════════════════════════════════════════════════════════════
FILE 1: tests/{EXCHANGE}_integration.rs
═══════════════════════════════════════════════════════════════════════════════

## Helper functions
- btc_usdt() -> Symbol
- eth_usdt() -> Symbol
- load_credentials() -> Option<Credentials>
- rate_limit_delay() // 100-200ms between tests

## Required tests (ALL must be implemented):

### Basic connectivity
- test_exchange_identity
- test_ping

### Market data - Spot
- test_get_price_spot
- test_get_ticker_spot
  - Assert: last_price > 0
  - Assert: bid_price < ask_price
  - Assert: volume_24h >= 0
- test_get_orderbook_spot
  - Assert: bids not empty
  - Assert: asks not empty
  - Assert: bids sorted descending
  - Assert: asks sorted ascending
- test_get_klines_spot
  - Assert: returns multiple candles
  - Assert: each has open/high/low/close > 0
  - Assert: high >= low
- test_get_symbols_spot

### Market data - Futures (if available)
- test_get_price_futures
- test_get_ticker_futures
- test_get_orderbook_futures
- test_get_klines_futures
- test_get_symbols_futures

### Error handling
- test_invalid_symbol
  - Assert: returns error, not panic

### Multiple requests
- test_multiple_intervals

### Auth tests (skip if no credentials)
- test_get_balance_with_auth

═══════════════════════════════════════════════════════════════════════════════
FILE 2: tests/{EXCHANGE}_websocket.rs
═══════════════════════════════════════════════════════════════════════════════

## Required tests:

### Connection
- test_websocket_connect_public_spot
- test_websocket_connect_public_futures
- test_websocket_connect_private (skip if no creds)
- test_disconnect_without_connect
- test_subscribe_without_connect

### Subscriptions - Spot
- test_subscribe_ticker_spot
  - Use GRACEFUL match pattern (not assert!)
  - Handle connection timeout
- test_subscribe_orderbook_spot
- test_subscribe_trades_spot
- test_subscribe_klines_spot

### Subscriptions - Futures (if available)
- test_subscribe_ticker_futures
- test_subscribe_orderbook_futures

### Multiple subscriptions
- test_multiple_subscriptions

### Event receiving (CRITICAL!)
- test_receive_ticker_events
  - Subscribe to ticker
  - Get event_stream()
  - Wait for events with timeout
  - Assert: received > 0 events
  - Assert: event data is valid (price > 0)

### Connection persistence (CRITICAL!)
- test_connection_persistence
  - Connect and subscribe
  - Monitor for 30-45 seconds
  - Check connection status every 5-15 seconds
  - Count received events
  - Assert: connection still alive at end
  - Assert: received multiple events throughout
  - This tests ping/pong heartbeat!

═══════════════════════════════════════════════════════════════════════════════
CRITICAL PATTERNS
═══════════════════════════════════════════════════════════════════════════════

## Graceful timeout handling for WebSocket tests

DON'T:
```rust
let result = ws.subscribe(sub).await;
assert!(result.is_ok()); // Will panic on timeout!
```

DO:
```rust
match ws.subscribe(sub).await {
    Ok(_) => {
        // Test logic here
        let _ = ws.disconnect().await;
        println!("✓ Test passed");
    }
    Err(e) => {
        println!("⚠ Subscribe failed (connection may have timed out): {:?}", e);
        let _ = ws.disconnect().await;
        println!("✓ Test completed (with connection issue)");
    }
}
```

## Event stream testing

```rust
let mut stream = ws.event_stream();
let mut event_count = 0;

while event_count < 3 {
    match timeout(Duration::from_secs(10), stream.next()).await {
        Ok(Some(Ok(event))) => {
            println!("Event: {:?}", event);
            event_count += 1;
            // Verify event data
            if let StreamEvent::Ticker(t) = &event {
                assert!(t.last_price > 0.0);
            }
        }
        Ok(Some(Err(e))) => println!("Error: {:?}", e),
        Ok(None) => break,
        Err(_) => println!("Timeout"),
    }
}
```

═══════════════════════════════════════════════════════════════════════════════
RUN TESTS
═══════════════════════════════════════════════════════════════════════════════

# REST tests
cargo test --package digdigdig3 --test {EXCHANGE}_integration -- --nocapture

# WebSocket tests
cargo test --package digdigdig3 --test {EXCHANGE}_websocket -- --nocapture
```

---

## Exit Criteria
- Both test files created
- Tests compile
- Tests run (failures expected, will fix in Phase 4)
