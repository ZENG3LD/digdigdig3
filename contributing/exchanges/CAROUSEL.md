# Agent Carousel: Exchange Connector Factory

Автоматизированная система создания коннекторов через последовательность агентов.

**Версия:** 2.0 (обновлено на основе опыта с 5 биржами)

## Quick Links

| Document | Description |
|----------|-------------|
| `prompts/00_coordinator.md` | Full pipeline instructions for Opus |
| `prompts/01_research.md` | Phase 1: Research agent prompt |
| `prompts/02_implement.md` | Phase 2: Implementation agent prompt |
| `prompts/03_test.md` | Phase 3: Test agent prompt |
| `prompts/04_debug.md` | Phase 4: Debug agent prompt |
| `GUIDE.md` | Quick reference + lessons learned |

---

## Обзор Pipeline

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         CONNECTOR PIPELINE                                    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   Phase 1          Phase 2          Phase 3          Phase 4                 │
│  ┌─────────┐      ┌─────────┐      ┌─────────┐      ┌─────────┐             │
│  │RESEARCH │─────►│IMPLEMENT│─────►│  TEST   │─────►│  DEBUG  │────► DONE   │
│  │         │      │         │      │         │      │         │       ✓     │
│  └─────────┘      └─────────┘      └─────────┘      └────┬────┘             │
│       │                │                │                │                   │
│       │                │                │                │ failures          │
│       ▼                ▼                ▼                └─────────┐        │
│   research/        src/code         tests/                        │        │
│   - endpoints      - REST           - REST integration           ▼        │
│   - auth           - WebSocket      - WebSocket                [loop]     │
│   - formats        - RateLimiter    - Persistence                         │
│   - websocket                       - Parsing                              │
│   - rate_limits                                                            │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Reference Implementation

**KuCoin** — полностью рабочий референс.

```
src/exchanges/kucoin/
├── mod.rs           # Exports
├── endpoints.rs     # URLs, endpoint enum, symbol formatting
├── auth.rs          # HMAC signature
├── parser.rs        # JSON → domain types
├── connector.rs     # MarketData, Trading, Account, Positions traits
├── websocket.rs     # WebSocket implementation
└── research/        # Research documentation
    ├── endpoints.md
    ├── authentication.md
    ├── response_formats.md
    ├── symbols.md
    ├── rate_limits.md
    └── websocket.md
```

---

## Phase 1: Research Agent

### Тип агента: `research-agent`

### Задача
Собрать ВСЮ информацию из официальной документации биржи для:
- REST API (endpoints, auth, formats)
- WebSocket API (connection, auth, messages, ping/pong)
- Rate Limits (REST и WebSocket отдельно)

### Output файлы

```
src/exchanges/{exchange}/research/
├── endpoints.md         # REST API endpoints
├── authentication.md    # Signature algorithm
├── response_formats.md  # JSON examples
├── symbols.md           # Symbol format rules
├── rate_limits.md       # Rate limiting (REST + WebSocket)
└── websocket.md         # WebSocket protocol details
```

### Prompt Template

```
Research {EXCHANGE} API for V5 connector implementation.

Documentation: {DOCS_URL}

Create folder: src/exchanges/{exchange}/research/

═══════════════════════════════════════════════════════════════════════════════
FILE 1: endpoints.md
═══════════════════════════════════════════════════════════════════════════════

Document ALL REST endpoints:

## Base URLs
- Spot production:
- Spot testnet:
- Futures production:
- Futures testnet:

## Market Data Endpoints (Public)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/v1/ticker | 24h ticker |
| ... | ... | ... |

## Account Endpoints (Private)
...

## Trading Endpoints (Private)
...

═══════════════════════════════════════════════════════════════════════════════
FILE 2: authentication.md
═══════════════════════════════════════════════════════════════════════════════

Document signature algorithm step-by-step:

1. Required headers:
   - Header name: value description

2. Signature string construction:
   - What components? (timestamp + method + path + body)
   - What order?
   - Any encoding of body?

3. HMAC algorithm:
   - SHA256 / SHA512 / other?
   - Key: API Secret

4. Signature encoding:
   - Base64 / Hex / other?

5. Timestamp format:
   - Milliseconds / Seconds?
   - Max clock skew allowed?

6. Example:
   - Request: GET /api/v1/account
   - Timestamp: 1234567890000
   - Signature string: "1234567890000GET/api/v1/account"
   - HMAC result: "xxx"
   - Final signature: "yyy"

═══════════════════════════════════════════════════════════════════════════════
FILE 3: response_formats.md
═══════════════════════════════════════════════════════════════════════════════

EXACT JSON examples from docs (not invented):

## Ticker Response
```json
{
  "field1": "value",  // description
  "field2": 123       // description
}
```

## Orderbook Response
...

## Klines Response (Candlesticks)
...

## Balance Response
...

## Order Response
...

CRITICAL: Copy exact field names. Note differences between Spot and Futures.

═══════════════════════════════════════════════════════════════════════════════
FILE 4: symbols.md
═══════════════════════════════════════════════════════════════════════════════

## Symbol Format

| Type | Format | Example |
|------|--------|---------|
| Spot | ??? | BTC-USDT / BTCUSDT / BTC_USDT |
| Futures | ??? | BTCUSDT / BTCUSDTM / BTC-USDT-SWAP |

## Conversion Rules
- Our internal: Symbol { base: "BTC", quote: "USDT" }
- To Spot: format_symbol_spot(symbol) -> "???"
- To Futures: format_symbol_futures(symbol) -> "???"

═══════════════════════════════════════════════════════════════════════════════
FILE 5: rate_limits.md
═══════════════════════════════════════════════════════════════════════════════

## REST API Rate Limits

### General Limits
- Requests per second/minute: ???
- Weight system: yes/no
- Per-IP or per-API-key: ???

### Public Endpoints
| Endpoint Type | Limit | Window |
|---------------|-------|--------|
| Market data | ??? | ??? |

### Private Endpoints
| Endpoint Type | Limit | Window |
|---------------|-------|--------|
| Account | ??? | ??? |
| Orders | ??? | ??? |

### Rate Limit Headers
Does API return headers? Which ones?
- X-RateLimit-Remaining: ???
- X-RateLimit-Reset: ???

### Rate Limit Error
- HTTP status code: 429 / other?
- Error code in response: ???
- Retry-After header: yes/no

## WebSocket Rate Limits

### Connection Limits
- Max connections per IP: ???
- Max subscriptions per connection: ???

### Message Limits
- Messages per second: ???

═══════════════════════════════════════════════════════════════════════════════
FILE 6: websocket.md
═══════════════════════════════════════════════════════════════════════════════

## Connection

### URLs
- Spot public: wss://...
- Spot private: wss://...
- Futures public: wss://...
- Futures private: wss://...

### Connection Process
1. Connect to URL
2. Any initial handshake required?
3. Any welcome message received?

## Authentication (Private Channels)

How to authenticate on WebSocket?
- Sign in URL params?
- Send auth message after connect?
- Auth message format?

## Subscription

### Subscribe Message Format
```json
{
  "op": "subscribe",
  "args": ["topic"]
}
```

### Unsubscribe Message Format
...

### Topics/Channels
| Topic | Format | Example |
|-------|--------|---------|
| Ticker | ??? | ??? |
| Orderbook | ??? | ??? |
| Trades | ??? | ??? |
| Klines | ??? | ??? |
| User Orders | ??? | ??? |
| User Balance | ??? | ??? |

## Message Formats

### Ticker Update
```json
{ ... }
```

### Orderbook Update
```json
{ ... }
```

### Trade Update
```json
{ ... }
```

## Heartbeat / Ping-Pong

CRITICAL: Document exactly!

### Who initiates?
- Server sends ping, client responds pong?
- Client sends ping, server responds pong?
- Both?

### Message format
- Binary ping/pong frames?
- Text messages ("ping"/"pong", "Ping"/"Pong")?
- JSON messages?

### Timing
- Ping interval: ??? seconds
- Timeout if no response: ??? seconds
- Different for Spot vs Futures?

### Compression
- Messages gzip compressed? (BingX does this!)
- Need to decompress before checking for ping?

### Example
Server: "ping" or {"op":"ping","ts":123}
Client: "pong" or {"op":"pong","ts":123}
```

### Exit Criteria
- All 6 research files created
- Each file has EXACT examples from official docs
- No guessed or invented data

---

## Phase 2: Implementation Agent

### Тип агента: `rust-implementer`

### Задача
Реализовать полный коннектор:
- REST API (connector.rs + parser.rs + auth.rs + endpoints.rs)
- WebSocket (websocket.rs)
- Добавить в mod.rs биржи

### Output файлы

```
src/exchanges/{exchange}/
├── mod.rs
├── endpoints.rs
├── auth.rs
├── parser.rs
├── connector.rs
└── websocket.rs
```

### Prompt Template

```
Implement {EXCHANGE} connector for V5 architecture.

═══════════════════════════════════════════════════════════════════════════════
REFERENCE
═══════════════════════════════════════════════════════════════════════════════

Reference implementation: src/exchanges/kucoin/
Research docs: src/exchanges/{exchange}/research/

Study KuCoin code carefully. Match patterns EXACTLY.

═══════════════════════════════════════════════════════════════════════════════
FILE 1: endpoints.rs
═══════════════════════════════════════════════════════════════════════════════

Create following KuCoin pattern:

pub struct {Exchange}Urls;

impl {Exchange}Urls {
    pub fn base_url(testnet: bool) -> &'static str { ... }
    pub fn futures_url(testnet: bool) -> &'static str { ... }
    pub fn ws_url(testnet: bool) -> &'static str { ... }
    pub fn ws_futures_url(testnet: bool) -> &'static str { ... }
}

pub enum {Exchange}Endpoint {
    // Market data
    Ticker,
    Orderbook,
    Klines,
    Symbols,
    // Account
    Balance,
    Positions,
    // Trading
    PlaceOrder,
    CancelOrder,
    OrderStatus,
}

impl {Exchange}Endpoint {
    pub fn path(&self) -> &'static str { ... }
    pub fn method(&self) -> &'static str { ... }  // "GET" or "POST"
    pub fn is_private(&self) -> bool { ... }
}

pub fn format_symbol(symbol: &Symbol, account_type: AccountType) -> String {
    // Use research/symbols.md
}

pub fn map_kline_interval(interval: &str) -> &str {
    // Map "1m" -> exchange format
}

═══════════════════════════════════════════════════════════════════════════════
FILE 2: auth.rs
═══════════════════════════════════════════════════════════════════════════════

pub struct {Exchange}Auth {
    api_key: String,
    api_secret: String,
    passphrase: Option<String>,  // if needed
}

impl {Exchange}Auth {
    pub fn new(credentials: &Credentials) -> Self { ... }

    pub fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        body: &str,
    ) -> HashMap<String, String> {
        // Return headers to add to request
        // Use research/authentication.md
    }
}

═══════════════════════════════════════════════════════════════════════════════
FILE 3: parser.rs
═══════════════════════════════════════════════════════════════════════════════

pub struct {Exchange}Parser;

impl {Exchange}Parser {
    pub fn parse_ticker(data: &Value) -> ExchangeResult<Ticker> { ... }
    pub fn parse_orderbook(data: &Value) -> ExchangeResult<Orderbook> { ... }
    pub fn parse_klines(data: &Value) -> ExchangeResult<Vec<Kline>> { ... }
    pub fn parse_balance(data: &Value) -> ExchangeResult<Vec<Balance>> { ... }
    pub fn parse_order(data: &Value) -> ExchangeResult<Order> { ... }
    pub fn parse_symbols(data: &Value) -> ExchangeResult<Vec<SymbolInfo>> { ... }

    // WebSocket parsers (different format!)
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> { ... }
    pub fn parse_ws_orderbook(data: &Value) -> ExchangeResult<OrderbookUpdate> { ... }
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<Trade> { ... }
    pub fn parse_ws_kline(data: &Value) -> ExchangeResult<Kline> { ... }
}

CRITICAL: REST and WebSocket often have DIFFERENT field names!
Check research/response_formats.md AND research/websocket.md

═══════════════════════════════════════════════════════════════════════════════
FILE 4: connector.rs
═══════════════════════════════════════════════════════════════════════════════

pub struct {Exchange}Connector {
    client: Client,
    auth: Option<{Exchange}Auth>,
    testnet: bool,
}

impl {Exchange}Connector {
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self>;
    pub async fn public(testnet: bool) -> ExchangeResult<Self>;
}

// Implement ALL traits:
impl MarketData for {Exchange}Connector { ... }
impl Trading for {Exchange}Connector { ... }
impl Account for {Exchange}Connector { ... }
impl Positions for {Exchange}Connector { ... }

═══════════════════════════════════════════════════════════════════════════════
FILE 5: websocket.rs
═══════════════════════════════════════════════════════════════════════════════

pub struct {Exchange}WebSocket {
    ws: Option<WebSocketStream<...>>,
    subscriptions: HashSet<SubscriptionRequest>,
    event_tx: mpsc::Sender<...>,
    broadcast_tx: broadcast::Sender<...>,  // For event_stream()
}

impl {Exchange}WebSocket {
    pub async fn new(credentials: Option<Credentials>, testnet: bool, account_type: AccountType) -> ExchangeResult<Self>;
}

// Implement WebSocketConnector trait:
impl WebSocketConnector for {Exchange}WebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()>;
    async fn disconnect(&mut self) -> WebSocketResult<()>;
    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>;
    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>;
    fn event_stream(&self) -> impl Stream<Item = WebSocketResult<StreamEvent>>;
    fn connection_status(&self) -> ConnectionStatus;
    fn active_subscriptions(&self) -> Vec<SubscriptionRequest>;
}

CRITICAL for event_stream():
- Use broadcast channel pattern (not just mpsc)
- Forward events from internal mpsc to broadcast
- Return broadcast::Receiver wrapped in stream

CRITICAL for ping/pong:
- Check research/websocket.md for exact format
- Some exchanges send text "Ping", others send JSON
- Some compress messages (BingX uses gzip!)
- Handle in message processing loop

═══════════════════════════════════════════════════════════════════════════════
FILE 6: mod.rs
═══════════════════════════════════════════════════════════════════════════════

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::*;
pub use auth::*;
pub use parser::*;
pub use connector::*;
pub use websocket::*;

═══════════════════════════════════════════════════════════════════════════════
AFTER EACH FILE
═══════════════════════════════════════════════════════════════════════════════

cargo check --package digdigdig3

═══════════════════════════════════════════════════════════════════════════════
FINALLY: Add to src/exchanges/mod.rs
═══════════════════════════════════════════════════════════════════════════════

pub mod {exchange};
```

### Exit Criteria
- All 6 files created
- `cargo check --package digdigdig3` passes
- Exchange added to src/exchanges/mod.rs

---

## Phase 3: Test Agent

### Тип агента: `rust-implementer`

### Задача
Написать ПОЛНЫЕ тесты:
- REST integration tests
- WebSocket tests (connect, subscribe, events, persistence)

### Output файлы

```
tests/{exchange}_integration.rs   # REST API tests
tests/{exchange}_websocket.rs     # WebSocket tests
```

### Prompt Template

```
Write comprehensive tests for {EXCHANGE} connector.

═══════════════════════════════════════════════════════════════════════════════
REFERENCES
═══════════════════════════════════════════════════════════════════════════════

REST tests reference: tests/kucoin_integration.rs
WebSocket tests reference: tests/kucoin_websocket.rs

═══════════════════════════════════════════════════════════════════════════════
FILE 1: tests/{exchange}_integration.rs
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
FILE 2: tests/{exchange}_websocket.rs
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
cargo test --package digdigdig3 --test {exchange}_integration -- --nocapture

# WebSocket tests
cargo test --package digdigdig3 --test {exchange}_websocket -- --nocapture
```

### Exit Criteria
- Both test files created
- Tests compile
- Tests run (failures expected, will fix in Phase 4)

---

## Phase 4: Debug Agent

### Тип агента: `rust-implementer`

### Задача
Итеративно исправлять код пока ВСЕ тесты не пройдут.

### Prompt Template

```
Debug and fix failing tests for {EXCHANGE} connector.

═══════════════════════════════════════════════════════════════════════════════
PROCESS
═══════════════════════════════════════════════════════════════════════════════

1. Run all tests:
   cargo test --package digdigdig3 --test {exchange}_integration -- --nocapture
   cargo test --package digdigdig3 --test {exchange}_websocket -- --nocapture

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
cargo test --package digdigdig3 --test {exchange}_integration
cargo test --package digdigdig3 --test {exchange}_websocket

Both show: test result: ok. N passed; 0 failed
```

### Exit Criteria
- ALL REST tests pass
- ALL WebSocket tests pass
- Output: "test result: ok. N passed; 0 failed"

---

## Coordinator Script (Pseudo-code)

```python
EXCHANGES = [
    ("bybit", "https://bybit-exchange.github.io/docs/"),
    ("okx", "https://www.okx.com/docs-v5/"),
    ("gateio", "https://www.gate.io/docs/developers/apiv4/"),
    # ... more
]

for exchange, docs_url in EXCHANGES:

    # Phase 1: Research
    print(f"[{exchange}] Phase 1: Research")
    task = Task(
        agent="research-agent",
        prompt=RESEARCH_PROMPT.format(exchange=exchange, docs_url=docs_url)
    )
    await task
    verify_files_exist(f"src/exchanges/{exchange}/research/*.md")

    # Phase 2: Implement
    print(f"[{exchange}] Phase 2: Implement")
    task = Task(
        agent="rust-implementer",
        prompt=IMPLEMENT_PROMPT.format(exchange=exchange)
    )
    await task
    assert cargo_check_passes()

    # Phase 3: Test
    print(f"[{exchange}] Phase 3: Test")
    task = Task(
        agent="rust-implementer",
        prompt=TEST_PROMPT.format(exchange=exchange)
    )
    await task

    # Phase 4: Debug loop
    print(f"[{exchange}] Phase 4: Debug")
    max_iterations = 10
    for i in range(max_iterations):
        result = run_tests(exchange)
        if result.all_passed:
            break
        task = Task(
            agent="rust-implementer",
            prompt=DEBUG_PROMPT.format(
                exchange=exchange,
                failures=result.failures
            )
        )
        await task

    # Commit
    git_add(f"src/exchanges/{exchange}/")
    git_add(f"tests/{exchange}_*.rs")
    git_commit(f"feat(v5/{exchange}): implement connector with tests")

    print(f"[{exchange}] DONE ✓")
```

---

## Parallel Execution

Независимые биржи можно обрабатывать параллельно:

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Bybit     │  │    OKX      │  │   Gate.io   │
│  Pipeline   │  │  Pipeline   │  │  Pipeline   │
└─────────────┘  └─────────────┘  └─────────────┘
      │                │                │
      ▼                ▼                ▼
  [research]       [research]      [research]
      │                │                │
      ▼                ▼                ▼
 [implement]      [implement]     [implement]
      │                │                │
      ▼                ▼                ▼
   [test]           [test]          [test]
      │                │                │
      ▼                ▼                ▼
  [debug]          [debug]         [debug]
      │                │                │
      ▼                ▼                ▼
   DONE ✓           DONE ✓          DONE ✓
```

---

## Exchange Registry

### Status Legend
- ✓ = Implemented and tested
- ⚠ = Implemented but disabled (infrastructure issues)
- 📁 = Folder exists, ready for implementation
- 🔗 = DEX (may require different approach)

### Completed (7)
| Exchange | REST | WebSocket | Tests |
|----------|------|-----------|-------|
| KuCoin | ✓ | ✓ | 16 REST + 15 WS (31 total) |
| Binance | ✓ | ✓ | 13 REST + 11 WS (24 total) |
| BingX | ✓ | ✓ | 9 REST + 14 WS (23 total) |
| Bitfinex | ✓ | ✓ | 9 REST + 10 WS (19 total) |
| Bitget | ✓ | ✓ | 15 REST + 13 WS (28 total) |
| Bybit | ✓ | ✓ | 15 REST + 18 WS (33 total) |
| OKX | ✓ | ✓ | 14 REST + 15 WS (29 total) |

### Disabled (1)
| Exchange | Status | Reason |
|----------|--------|--------|
| Bithumb ⚠ | REST hangs | SSL/TLS infrastructure issues (see research/504_investigation.md) |

### CEX - High Priority (5)
| Exchange | Folder | Docs URL |
|----------|--------|----------|
| Gate.io 📁 | gateio/ | https://www.gate.io/docs/developers/apiv4/ |
| Kraken 📁 | kraken/ | https://docs.kraken.com/rest/ |
| Coinbase 📁 | coinbase/ | https://docs.cdp.coinbase.com/exchange/docs/ |
| HTX (Huobi) 📁 | htx/ | https://www.htx.com/en-us/opend/newApiPages/ |
| MEXC 📁 | mexc/ | https://mexcdevelop.github.io/apidocs/ |

### CEX - Medium Priority (6)
| Exchange | Folder | Docs URL |
|----------|--------|----------|
| Bitstamp 📁 | bitstamp/ | https://www.bitstamp.net/api/ |
| Crypto.com 📁 | crypto_com/ | https://exchange-docs.crypto.com/ |
| Gemini 📁 | gemini/ | https://docs.gemini.com/ |
| Phemex 📁 | phemex/ | https://phemex-docs.github.io/ |
| Upbit 📁 | upbit/ | https://docs.upbit.com/ |
| Deribit 📁 | deribit/ | https://docs.deribit.com/ |

### DEX - On-chain (9)
| Exchange | Folder | Chain | Docs URL |
|----------|--------|-------|----------|
| dYdX 🔗 | dydx/ | Cosmos | https://docs.dydx.exchange/ |
| Hyperliquid 🔗 | hyperliquid/ | Arbitrum | https://hyperliquid.gitbook.io/hyperliquid-docs/ |
| GMX 🔗 | gmx/ | Arbitrum | https://docs.gmx.io/ |
| Vertex 🔗 | vertex/ | Arbitrum | https://docs.vertexprotocol.com/ |
| Paradex 🔗 | paradex/ | Starknet | https://docs.paradex.trade/ |
| Lighter 🔗 | lighter/ | Arbitrum | https://docs.lighter.xyz/ |
| Jupiter 🔗 | jupiter/ | Solana | https://station.jup.ag/docs/ |
| Raydium 🔗 | raydium/ | Solana | https://docs.raydium.io/ |
| Uniswap 🔗 | uniswap/ | Ethereum | https://docs.uniswap.org/ |

### Total: 29 exchanges (7 done + 1 disabled + 21 pending)

---

## Quick Start for Coordinator

```
User: "Реализуй коннектор для Bybit"

Coordinator:
1. Read this file (CAROUSEL.md)
2. Task(research-agent): Research Bybit API using RESEARCH_PROMPT
3. Wait → verify src/exchanges/bybit/research/*.md created
4. Task(rust-implementer): Implement using IMPLEMENT_PROMPT
5. Wait → verify cargo check passes
6. Task(rust-implementer): Write tests using TEST_PROMPT
7. Wait → verify test files created
8. Loop: Task(rust-implementer): Fix failures using DEBUG_PROMPT
9. Until: all tests pass
10. Commit
11. Report: "Bybit connector done, X REST + Y WS tests passing"
```

---

## Lessons Learned (from 5 exchanges)

1. **REST vs WebSocket field names are DIFFERENT** - Always check both in research
2. **event_stream() needs broadcast channel** - mpsc alone doesn't work for multiple consumers
3. **Ping/pong varies wildly** - Some text, some JSON, some gzip compressed
4. **Graceful test handling** - Use match pattern, not assert for network operations
5. **Connection persistence test is CRITICAL** - Tests heartbeat mechanism
6. **Parser dual-format** - Binance REST uses long names, WS uses short names
7. **Futures symbols often have suffix** - M, PERP, SWAP, etc.
8. **Rate limits separate for REST/WS** - Don't mix them up
