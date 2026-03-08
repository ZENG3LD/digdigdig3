# Phase 4: Debug Agent Prompt - Data Providers

## Agent Type
`rust-implementer`

## Variables
- `{PROVIDER}` - Provider name in lowercase
- `{CATEGORY}` - Category (aggregators, forex, stocks, data_feeds)

---

## Mission

Debug and fix tests until they return REAL data from {PROVIDER}.

**Goal:** All tests should either:
- ✅ Return real data (prices, tickers, events)
- ✅ Return UnsupportedOperation (for trading/account methods)
- ✅ Handle errors gracefully (API key, rate limits)

**NOT acceptable:**
- ❌ Tests panic or crash
- ❌ Tests return fake/stub data
- ❌ Compilation errors

---

## Debug Loop

**Repeat until all criteria met:**

1. Run tests
2. Analyze failures
3. Fix implementation
4. Goto 1

---

## Commands

```bash
# Run integration tests
cargo test --package digdigdig3 --test {provider}_integration -- --nocapture

# Run WebSocket tests (if applicable)
cargo test --package digdigdig3 --test {provider}_websocket -- --nocapture

# Run all provider tests
cargo test --package digdigdig3 {provider} -- --nocapture

# Check compilation
cargo check --package digdigdig3
```

---

## Common Issues & Fixes

### Issue 1: Missing API Key

**Symptom:**
```
Error: Authentication failed
Error: 401 Unauthorized
```

**Fix:**
1. Check research/authentication.md for how to get API key
2. Set environment variable:
   ```bash
   export PROVIDER_API_KEY="your_key_here"
   ```
3. Verify auth.rs correctly adds key to headers/params

**Code locations:**
- `src/{CATEGORY}/{PROVIDER}/auth.rs` - Check sign_headers() or sign_query()
- `src/{CATEGORY}/{PROVIDER}/connector.rs` - Verify get() calls use auth

---

### Issue 2: Rate Limit Exceeded

**Symptom:**
```
Error: HTTP 429 Too Many Requests
Error: Rate limit exceeded
```

**Fix:**
1. Check research/tiers_and_limits.md for actual limits
2. Add delays between test API calls:
   ```rust
   tokio::time::sleep(Duration::from_millis(1000)).await;
   ```
3. Consider using paid tier if free tier too restrictive
4. Tests should handle 429 gracefully:
   ```rust
   match connector.get_price(...).await {
       Err(ExchangeError::RateLimit(msg)) => {
           println!("⚠ Rate limit: {}", msg);
           println!("✓ Test passed (rate limit expected on free tier)");
       }
       ...
   }
   ```

---

### Issue 3: Wrong Endpoint URL

**Symptom:**
```
Error: 404 Not Found
Error: Invalid endpoint
```

**Fix:**
1. Re-check research/endpoints_full.md
2. Verify endpoint paths in endpoints.rs:
   ```rust
   Self::Price => "/v1/price",  // Check exact path
   ```
3. Verify base URL in endpoints.rs:
   ```rust
   rest_base: "https://api.example.com",  // No trailing slash?
   ```

---

### Issue 4: JSON Parse Error

**Symptom:**
```
Error: Parse("Missing 'price' field")
Error: JSON structure doesn't match
```

**Fix:**
1. Re-check research/response_formats.md
2. Make actual API call to see real response:
   ```bash
   curl -H "X-API-Key: xxx" "https://api.example.com/v1/price?symbol=AAPL"
   ```
3. Update parser.rs to match actual JSON structure
4. Check for nested fields:
   ```rust
   // Maybe it's nested: response.data.price
   response.get("data").and_then(|d| d.get("price"))
   ```

**Common mistakes:**
- Field name differs: `lastPrice` vs `last_price` vs `price`
- Value type differs: string "150.25" vs number 150.25
- Array vs object: `[{...}]` vs `{...}`
- Nested structure: `{data: {price: 150.25}}` vs `{price: 150.25}`

---

### Issue 5: Symbol Format Wrong

**Symptom:**
```
Error: Invalid symbol
Error: Symbol not found
```

**Fix:**
1. Re-check research/data_formats.md or coverage.md
2. Update format_symbol() in endpoints.rs:
   ```rust
   // Provider expects: "AAPL" not "AAPL-USD"
   symbol.base.to_uppercase()

   // Provider expects: "EUR/USD" not "EUR_USD"
   format!("{}/{}", symbol.base, symbol.quote)

   // Provider expects: "btcusdt" (lowercase) not "BTCUSDT"
   format!("{}{}", symbol.base, symbol.quote).to_lowercase()
   ```

---

### Issue 6: WebSocket Connection Fails

**Symptom:**
```
Error: Connection refused
Error: TLS handshake failed
Error: WebSocket upgrade failed
```

**Fix:**
1. Re-check research/websocket_full.md for correct URL
2. Check if auth required for WebSocket connection:
   ```rust
   // Maybe auth in URL params?
   format!("wss://ws.example.com?apiKey={}", api_key)
   ```
3. Check for initial auth message after connect:
   ```rust
   // Send auth message after connection
   send_json({{"op": "auth", "key": api_key}})
   ```
4. Try connection from command line:
   ```bash
   websocat "wss://ws.example.com"
   ```

---

### Issue 7: WebSocket No Events Received

**Symptom:**
```
✓ Connected
✓ Subscribed
⚠ Timeout waiting for events
```

**Fix:**
1. Re-check research/websocket_full.md for subscription format
2. Verify subscribe message is correct:
   ```rust
   // Check exact JSON format provider expects
   {"type": "subscribe", "channel": "ticker", "symbol": "AAPL"}
   // vs
   {"op": "subscribe", "args": ["ticker:AAPL"]}
   ```
3. Add debug logging to see what messages are received:
   ```rust
   println!("Received WS message: {}", msg);
   ```
4. Check if messages are compressed (some providers use gzip)
5. Check ping/pong handling - connection may be timing out

---

### Issue 8: Price Values Unrealistic

**Symptom:**
```
Stock price: $0.00015  // Should be ~$150
FX rate: 150000.0      // Should be ~1.5
```

**Fix:**
1. Check if price needs scaling:
   ```rust
   // Provider returns cents, not dollars
   price / 100.0

   // Provider uses different precision
   price / 10000.0  // 4 decimal places

   // Provider uses scientific notation
   price_str.parse::<f64>()? / 1e8
   ```
2. Re-check research/response_formats.md for price field format

---

### Issue 9: Compilation Errors After Changes

**Symptom:**
```
error[E0425]: cannot find function `format_symbol`
error[E0599]: no method named `parse_price`
```

**Fix:**
1. Ensure all functions are `pub`:
   ```rust
   pub fn format_symbol(...)  // Not just fn
   pub fn parse_price(...)
   ```
2. Check imports in connector.rs:
   ```rust
   use super::endpoints::*;
   use super::parser::*;
   ```
3. Run `cargo check` frequently

---

## Verification Checklist

Before considering Phase 4 complete, verify:

### Integration Tests (REST API)

- [ ] test_exchange_identity passes (always should)
- [ ] test_get_price returns realistic price
  - [ ] Stock: $1 - $10,000 range
  - [ ] Forex: 0.01 - 1000 range
  - [ ] Crypto: Reasonable for that asset
- [ ] test_get_ticker returns full ticker with:
  - [ ] last_price > 0
  - [ ] bid < ask (if both present)
  - [ ] volume_24h reasonable (if present)
- [ ] test_get_klines returns candles with valid OHLC:
  - [ ] high >= low
  - [ ] high >= open, close
  - [ ] low <= open, close
- [ ] test_get_orderbook either:
  - [ ] Returns real orderbook with bid < ask
  - [ ] OR returns UnsupportedOperation (if no orderbook)
- [ ] test_get_symbols returns non-empty list
- [ ] test_invalid_symbol handles error gracefully
- [ ] test_trading_not_supported returns UnsupportedOperation
- [ ] test_account_not_supported returns UnsupportedOperation (unless broker)

### WebSocket Tests (if applicable)

- [ ] test_websocket_connect succeeds OR handles timeout gracefully
- [ ] test_subscribe_ticker succeeds OR handles connection issue gracefully
- [ ] test_receive_ticker_events receives REAL events with data
- [ ] test_connection_persistence doesn't disconnect early (30+ seconds)
- [ ] test_multiple_subscriptions handles multiple symbols

### Code Quality

- [ ] No compilation errors
- [ ] No panics in tests (all errors handled gracefully)
- [ ] Clear println! output showing what's happening
- [ ] Comments explain any workarounds or quirks

---

## Debug Strategy

### Step 1: Verify Basics

```bash
# Can you compile?
cargo check --package digdigdig3

# Can you run any test?
cargo test --package digdigdig3 --test {provider}_integration test_exchange_identity -- --nocapture
```

### Step 2: Test Authentication

```bash
# Test with actual API call
curl -H "X-API-Key: $PROVIDER_API_KEY" "https://api.example.com/v1/price?symbol=AAPL"

# Or if query param:
curl "https://api.example.com/v1/price?symbol=AAPL&apiKey=$PROVIDER_API_KEY"
```

### Step 3: Fix One Endpoint at a Time

1. Start with simplest: `test_get_price`
2. Get it returning real data
3. Move to next: `test_get_ticker`
4. Continue until all data endpoints work

### Step 4: Verify Trading/Account Return UnsupportedOperation

These should be quick wins if implementation followed Phase 2 template.

### Step 5: WebSocket (if available)

- Test connection first
- Then subscription
- Then event receiving
- Finally connection persistence (ping/pong)

---

## When to Ask for Help

If stuck for >30 minutes on same issue:

1. Document what you tried
2. Include error messages
3. Show relevant code snippets
4. Note what research says vs what you're seeing

**Common reasons to escalate:**
- Provider changed API (research docs outdated)
- Provider requires OAuth flow (complex authentication)
- Provider uses proprietary protocol (not standard REST/WS)
- Provider has geo-blocking (VPN needed)

---

## Exit Criteria

**Phase 4 is COMPLETE when:**

1. ✅ All integration tests compile
2. ✅ All WebSocket tests compile (if WS available)
3. ✅ At least one data test returns REAL data:
   - test_get_price shows realistic price
   - test_get_ticker shows real market data
   - OR test_receive_ticker_events shows real WS events
4. ✅ Trading tests return UnsupportedOperation
5. ✅ Account tests return UnsupportedOperation (unless broker)
6. ✅ No panics/crashes in tests
7. ✅ Error handling is graceful (network/API errors don't crash)

**Output should look like:**
```
✓ Exchange name: polygon
✓ Price for AAPL/USD: $150.25
✓ Ticker for AAPL/USD: last=$150.25, volume=12345678
✓ Retrieved 10 klines
✓ Trading correctly marked as unsupported
...
test result: ok. 9 passed; 0 failed; 0 ignored
```

---

## After Phase 4

When all criteria met:
1. Commit changes with message format:
   ```
   feat(v5): add {PROVIDER} connector ({CATEGORY})

   - Research: 8 documentation files
   - Implementation: REST + WebSocket support
   - Tests: X/X passing with real data
   - Data types: [list what's available]

   Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
   ```

2. Update ../../exchanges/CAROUSEL.md registry (if exists)

3. Document any quirks/learnings in connector README

**Connector is now production-ready! 🎉**
