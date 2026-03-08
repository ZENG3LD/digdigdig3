# V5 Connector Agent Guide

Руководство для агентов по созданию новых exchange коннекторов.

**See also:**
- `CAROUSEL.md` — полная документация pipeline
- `prompts/` — готовые промпты для каждой фазы

---

## Reference Implementation

**KuCoin** (`src/exchanges/kucoin/`) — полностью рабочий референс с тестами.

```
exchanges/kucoin/
├── mod.rs          # Exports
├── endpoints.rs    # URLs, endpoint enum, symbol formatting
├── auth.rs         # HMAC signature implementation
├── parser.rs       # JSON → domain types
├── connector.rs    # Trait implementations
├── websocket.rs    # WebSocket connector
└── research/       # Research documentation
```

---

## Quick Start

### Для координатора (Opus)

```
1. Read prompts/00_coordinator.md
2. Follow the pipeline for target exchange
```

### Для отдельного агента

```
1. Read prompts/0X_{phase}.md (01_research, 02_implement, etc.)
2. Replace {EXCHANGE} with target exchange name
3. Follow instructions exactly
```

---

## Prompts Directory

```
prompts/
├── 00_coordinator.md  # Full pipeline instructions
├── 01_research.md     # Phase 1: Research agent prompt
├── 02_implement.md    # Phase 2: Implementation agent prompt
├── 03_test.md         # Phase 3: Test agent prompt
└── 04_debug.md        # Phase 4: Debug agent prompt
```

---

## Pipeline Overview

```
┌─────────────────┐
│ Research Agent  │ → research/{exchange}/
└────────┬────────┘
         ▼
┌─────────────────┐
│ Implement Agent │ → src/exchanges/{exchange}/
└────────┬────────┘
         ▼
┌─────────────────┐
│ Test Agent      │ → tests/{exchange}_*.rs
└────────┬────────┘
         ▼
┌─────────────────┐     ┌──────────┐
│ Debug Agent     │────►│ All Pass │
└────────┬────────┘     └──────────┘
         │ failures
         ▼
    [loop back to fix]
```

---

## Lessons Learned (from 5+ exchanges)

### 1. REST vs WebSocket Field Names

**Problem:** Same data, different field names.

```
REST ticker:  {"lastPrice": "50000", "bidPrice": "49999"}
WS ticker:    {"c": "50000", "b": "49999"}
```

**Solution:** Create separate parsers:
- `parse_ticker()` for REST
- `parse_ws_ticker()` for WebSocket

### 2. event_stream() Requires Broadcast Channel

**Problem:** `mpsc` channel doesn't support multiple consumers.

**Solution:**
```rust
// In struct
broadcast_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,

// In connect() - forward from mpsc to broadcast
tokio::spawn(async move {
    while let Some(event) = event_rx.recv().await {
        let _ = broadcast_tx.send(event);
    }
});

// In event_stream()
fn event_stream(&self) -> impl Stream<...> {
    BroadcastStream::new(self.broadcast_tx.subscribe())
        .filter_map(|r| async { r.ok() })
}
```

### 3. Ping/Pong Varies Wildly

| Exchange | Format | Who initiates |
|----------|--------|---------------|
| KuCoin | Binary frames | Either |
| Binance | JSON `{"method":"PING"}` | Client |
| BingX | gzip compressed text | Server |
| Bitfinex | JSON `{"event":"ping"}` | Client |
| Bitget | Text "ping"/"pong" | Server |

**Solution:** Always check `research/websocket.md` for exact format.

### 4. Graceful Test Handling

**Problem:** `assert!(result.is_ok())` panics on network timeout.

**Solution:**
```rust
match ws.subscribe(sub).await {
    Ok(_) => {
        // Test logic
        println!("✓ Test passed");
    }
    Err(e) => {
        println!("⚠ Connection issue: {:?}", e);
        println!("✓ Test completed (with connection issue)");
    }
}
```

### 5. Connection Persistence Test is Critical

Tests the ping/pong heartbeat mechanism:
```rust
// Monitor for 30-45 seconds
// Count events
// Verify connection still alive
```

If this test fails, ping/pong implementation is broken.

### 6. Futures Symbols Often Have Suffix

| Exchange | Spot | Futures |
|----------|------|---------|
| KuCoin | BTC-USDT | XBTUSDTM |
| Binance | BTCUSDT | BTCUSDT |
| BingX | BTC-USDT | BTC-USDT |
| Bitget | BTCUSDT | BTCUSDT |

**Solution:** Always implement `format_symbol()` with `AccountType` parameter.

### 7. Rate Limits Are Separate

- REST: requests per second/minute
- WebSocket: messages per second, max subscriptions

Never mix them up in rate limiter implementation.

### 8. Some Exchanges Use Compression

BingX compresses WebSocket messages with gzip. Must decompress before processing:
```rust
let decompressed = decompress_gzip(&data)?;
let text = String::from_utf8(decompressed)?;
```

---

## Commands

```bash
# Check compilation
cargo check --package digdigdig3

# Run all tests for exchange
cargo test --package digdigdig3 --test {exchange}_integration -- --nocapture

# Run specific test
cargo test --package digdigdig3 --test {exchange}_integration test_get_ticker_spot -- --nocapture

# Run WebSocket tests
cargo test --package digdigdig3 --test {exchange}_websocket -- --nocapture
```

---

## Checklist для нового коннектора

### Phase 1: Research
- [ ] endpoints.md created
- [ ] authentication.md created
- [ ] response_formats.md created
- [ ] symbols.md created
- [ ] rate_limits.md created
- [ ] websocket.md created

### Phase 2: Implement
- [ ] endpoints.rs compiles
- [ ] auth.rs compiles
- [ ] parser.rs compiles (REST + WS parsers!)
- [ ] connector.rs compiles
- [ ] websocket.rs compiles (broadcast channel!)
- [ ] mod.rs exports all types
- [ ] Added to src/exchanges/mod.rs

### Phase 3: Test
- [ ] {exchange}_integration.rs written
- [ ] {exchange}_websocket.rs written
- [ ] Uses graceful timeout handling

### Phase 4: Debug
- [ ] All REST tests pass
- [ ] All WebSocket tests pass
- [ ] event_stream() returns real events
- [ ] Connection persists 30+ seconds

### Final
- [ ] Committed with proper message
- [ ] CAROUSEL.md registry updated
