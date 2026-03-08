# Phase 2: Implementation Agent Prompt

## Agent Type
`rust-implementer`

## Variables
- `{EXCHANGE}` - Exchange name in lowercase (e.g., "bybit")
- `{Exchange}` - Exchange name in PascalCase (e.g., "Bybit")

---

## Prompt

```
Implement {EXCHANGE} connector for V5 architecture.

═══════════════════════════════════════════════════════════════════════════════
REFERENCE
═══════════════════════════════════════════════════════════════════════════════

Reference implementation: src/exchanges/kucoin/
Research docs: src/exchanges/{EXCHANGE}/research/

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

pub mod {EXCHANGE};
```

---

## Exit Criteria
- All 6 files created
- `cargo check --package digdigdig3` passes
- Exchange added to src/exchanges/mod.rs
