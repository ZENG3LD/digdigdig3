# Phase 4: Debug Agent Prompt

## Agent Type
`rust-implementer`

## Variables
- `{EXCHANGE}` - Exchange name in lowercase (e.g., "bybit")

---

## Prompt

```
Debug and fix failing tests for {EXCHANGE} connector.

═══════════════════════════════════════════════════════════════════════════════
PROCESS
═══════════════════════════════════════════════════════════════════════════════

1. Run all tests:
   cargo test --package digdigdig3 --test {EXCHANGE}_integration -- --nocapture
   cargo test --package digdigdig3 --test {EXCHANGE}_websocket -- --nocapture

2. For EACH failure, identify the error type and fix:

═══════════════════════════════════════════════════════════════════════════════
COMMON ERRORS AND FIXES
═══════════════════════════════════════════════════════════════════════════════

## Auth errors (401, Invalid signature)
Location: auth.rs

Checklist:
- [ ] Timestamp format: milliseconds or seconds?
- [ ] Signature string order correct?
- [ ] HMAC algorithm correct (SHA256/SHA512)?
- [ ] Encoding correct (Base64/Hex)?
- [ ] All required headers included?
- [ ] Body included in signature if POST?

Debug: Print signature string before hashing to compare with docs example.

## Parse errors (field not found, wrong type)
Location: parser.rs

Checklist:
- [ ] Field name exact match? (case sensitive!)
- [ ] Spot vs Futures have different field names?
- [ ] Nested structure? data.result vs data?
- [ ] Array vs object?
- [ ] String number needs parsing? "123" -> 123

Debug: Print raw JSON response to see actual structure.

Fix pattern:
```rust
// Try multiple field names
let price = data.get("lastPrice")
    .or_else(|| data.get("last_price"))
    .or_else(|| data.get("c"))
    .and_then(|v| parse_decimal(v))
    .unwrap_or(Decimal::ZERO);
```

## WebSocket parse errors
Location: parser.rs (parse_ws_* functions)

CRITICAL: REST and WebSocket often have DIFFERENT formats!
- REST ticker: {"lastPrice": "50000", "bidPrice": "49999", ...}
- WS ticker: {"c": "50000", "b": "49999", ...}

Check research/websocket.md for WebSocket message formats.

## Symbol errors (symbol not found)
Location: endpoints.rs

Checklist:
- [ ] Spot format correct? (BTC-USDT vs BTCUSDT vs BTC_USDT)
- [ ] Futures format correct? (often needs suffix: BTCUSDTM, BTC-USDT-SWAP)
- [ ] Case correct? (some exchanges want lowercase)

## Connection errors
Location: endpoints.rs, websocket.rs

Checklist:
- [ ] URL correct?
- [ ] WSS not WS?
- [ ] Need token in URL? (KuCoin does this)

## event_stream() returns nothing
Location: websocket.rs

CRITICAL: Must use broadcast channel pattern!

```rust
// In struct
broadcast_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,

// In connect() - forward from mpsc to broadcast
let broadcast_tx = self.broadcast_tx.clone();
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

## Connection drops after 30-60 seconds
Location: websocket.rs

Ping/pong not working. Check:
- [ ] Exchange sends ping, we respond pong?
- [ ] We send ping, wait for pong?
- [ ] Text "ping"/"pong" or JSON {"op":"ping"}?
- [ ] Messages compressed? (BingX uses gzip!)

Debug: Log all incoming messages to see ping format.

═══════════════════════════════════════════════════════════════════════════════
LOOP UNTIL ALL PASS
═══════════════════════════════════════════════════════════════════════════════

Repeat:
1. Run tests
2. Pick first failure
3. Identify cause
4. Fix code
5. Run single test to verify
6. Run all tests
7. If failures remain, go to 2

EXIT only when:
cargo test --package digdigdig3 --test {EXCHANGE}_integration
cargo test --package digdigdig3 --test {EXCHANGE}_websocket

Both show: test result: ok. N passed; 0 failed
```

---

## Exit Criteria
- ALL REST tests pass
- ALL WebSocket tests pass
- Output: "test result: ok. N passed; 0 failed"
