# Wave 0 Foundation Plan

## Goal

Replace ~5 000 LOC of duplicated per-exchange connect/ping/reconnect/dispatch loops with a
single generic transport `UniversalWsTransport<P: WsProtocol>`.  Each exchange shrinks to a
thin declarative shim (`WsProtocol` impl) that provides: endpoint URL, ping frame, topic
subscription frames, a topic extractor, and a `TopicRegistry` that maps topic keys to parser
functions.  The framework owns ALL connection lifecycle, ping scheduling, subscription replay,
frame routing, and unmatched-frame logging.  Silent drops become impossible — every unmatched
frame produces a `tracing::warn!`.  Capabilities are derived mechanically from the registry;
the 4-bool flat struct is replaced by a queryable trait.

---

## File layout

```
digdigdig3/src/core/websocket/
├── mod.rs              — pub re-exports (WsProtocol, UniversalWsTransport, TopicRegistry,
│                          StreamKind, SupportLevel, StreamSpec, CapabilityProvider)
├── protocol.rs         — trait WsProtocol
├── transport.rs        — struct UniversalWsTransport<P> + state machine
├── topic_registry.rs   — TopicRegistry, TopicPattern, TopicKey, ParserFn
├── stream_kind.rs      — enum StreamKind (all 34 variants)
├── support_level.rs    — enum SupportLevel
├── reconnect.rs        — ReconnectConfig (repromote from base_websocket.rs) + BackoffState
└── stream_spec.rs      — struct StreamSpec (replaces SubscriptionRequest inside the framework)
```

`base_websocket.rs` remains compiled-out (module comment already gates it with `// mod
base_websocket`).  It is NOT deleted yet — Wave 1 migrates exchanges one by one; dead code
removal is Wave 2.

---

## Type definitions

### StreamKind

Groups and whether each carries parameters:

```rust
/// Full enumeration of all known WebSocket stream kinds across all supported exchanges.
///
/// Variants with `{ interval: KlineInterval }` carry a parameter.
/// All other variants are unit variants (no parameters).
///
/// Partitioned into groups for documentation; the enum itself is flat.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamKind {
    // ── Market (price / ticker) ───────────────────────────────────────────────
    /// Full 24-h rolling ticker (OHLCV + last price + volume)
    Ticker,
    /// Index price feed (spot → perpetual fair value)
    IndexPrice,
    /// Mark price feed (settlement-reference price)
    MarkPrice,
    /// Composite index price (constructed from multiple underlying sources)
    CompositeIndex,

    // ── OrderBook ────────────────────────────────────────────────────────────
    /// Level-2 full-depth snapshot
    Orderbook,
    /// Level-2 incremental delta stream
    OrderbookDelta,
    /// Level-3 per-order (L3) full orderbook
    OrderbookL3,

    // ── Trade ────────────────────────────────────────────────────────────────
    /// Individual public trades (time-and-sales)
    Trade,
    /// Aggregated trades (multiple fills at same price collapsed)
    AggTrade,
    /// Block trade / RFQ event (large off-book transactions)
    BlockTrade,

    // ── Kline (all carry KlineInterval parameter) ─────────────────────────
    /// Standard OHLCV candlestick
    Kline { interval: KlineInterval },
    /// Mark-price candlestick (futures)
    MarkPriceKline { interval: KlineInterval },
    /// Index-price candlestick (futures)
    IndexPriceKline { interval: KlineInterval },
    /// Premium-index candlestick (futures; basis ≈ mark−index)
    PremiumIndexKline { interval: KlineInterval },

    // ── Funding ──────────────────────────────────────────────────────────────
    /// Live funding rate (updates intraperiod)
    FundingRate,
    /// Predicted funding rate before settlement window opens
    PredictedFunding,
    /// Actual funding settlement event (rate + charged amount)
    FundingSettlement,

    // ── Risk / Open Interest / Sentiment ─────────────────────────────────
    /// Open interest snapshot / update
    OpenInterest,
    /// Long/short ratio (market sentiment)
    LongShortRatio,
    /// Insurance fund balance update
    InsuranceFund,
    /// Risk limit tier update (margin tiers)
    RiskLimit,
    /// Basis stream (futures price − spot price)
    Basis,
    /// Forced-liquidation event (public)
    Liquidation,

    // ── Options-specific ─────────────────────────────────────────────────
    /// Option greeks: delta/gamma/vega/theta/rho + IV
    OptionGreeks,
    /// Volatility index (e.g. DVOL on Deribit)
    VolatilityIndex,
    /// Historical realized volatility feed
    HistoricalVolatility,

    // ── Lifecycle / Market Events ─────────────────────────────────────────
    /// Settlement / expiry delivery event
    SettlementEvent,
    /// Auction event (indicative price, crossing state)
    AuctionEvent,
    /// Market warning / trading halt notification
    MarketWarning,

    // ── Private streams (auth-required) ──────────────────────────────────
    /// Order lifecycle events (create/fill/cancel/expire)
    OrderUpdate,
    /// Account balance changes
    BalanceUpdate,
    /// Futures position changes
    PositionUpdate,
}
```

Total: **34 variants** (4 market, 3 orderbook, 3 trade, 4 kline, 3 funding, 6 risk/OI,
3 options, 3 lifecycle, 3 private).

`KlineInterval` is an existing or new simple newtype:

```rust
/// Typed kline interval.  Inner string is the exchange-canonical form after formatting
/// (e.g. "1m", "1h", "1D").  Equality is by the inner str.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KlineInterval(pub String);

impl KlineInterval {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

`StreamKind` MUST derive `Hash + Eq` — it is used as registry key.  For variants with
`interval`, `PartialEq` + `Hash` already work correctly via derive because `KlineInterval`
is a newtype over `String`.

---

### SupportLevel

```rust
/// What level of support a connector has for a given (StreamKind, AccountType) pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportLevel {
    /// Parser registered in TopicRegistry; events flow.
    Native,
    /// dig3 has not yet implemented this exchange's channel for this stream kind.
    /// The channel likely exists on the exchange; it just hasn't been wired.
    NotImplemented,
    /// Exchange itself has no such channel for this account type.
    UnsupportedByExchange,
    /// Channel exists but requires authentication credentials (e.g. Binance forceOrders).
    RequiresAuth,
}
```

**No `Silent` variant.**  The new architecture makes "silent" impossible by construction: if
a parser is registered, frames ARE routed to it; if not, a `warn!` fires and the event is NOT
emitted (not silently dropped — loudly dropped).  `Silent` was a runtime symptom, not a
persistent state.  Historical smoke data about "silent producers" is documentation only.

---

### TopicKey / TopicPattern / TopicRegistry

**Matching strategy decision:** Use prefix-match patterns with a single wildcard `*` (replaces
the per-symbol suffix).  Exact reasoning:

- Binance: `btcusdt@trade`, `ethusdt@trade` → pattern `*@trade`
- OKX: `trades.BTC-USDT-SWAP` → pattern `trades.*`
- Bybit: `publicTrade.BTCUSDT` → pattern `publicTrade.*`
- KuCoin: `/market/ticker:BTC-USDT` → pattern `/market/ticker:*`
- HTX: `market.BTC-USDT.trade.detail` → pattern `market.*.trade.detail`
- Gateio: `spot.trades` with symbol in payload body → pattern `spot.trades` (exact)

Single-star wildcard covering one segment is sufficient for all 9 exchanges.  Regex is
overkill and adds parse overhead.  The `*` matches any contiguous sequence of non-separator
characters OR may appear at the end to match a suffix.

```rust
/// Raw topic string extracted from a frame (e.g. "btcusdt@trade", "trades.BTC-USDT-SWAP").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicKey(pub String);

impl TopicKey {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

/// A pattern used to register parsers.  Supports a single `*` wildcard.
///
/// Examples:
///   "*@trade"          — matches any Binance trade topic
///   "publicTrade.*"    — matches any Bybit trade topic
///   "market.*.depth"   — matches any HTX depth topic with symbol in segment 2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicPattern(pub String);

impl TopicPattern {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }

    /// Returns true if this pattern matches the given key.
    /// At most one `*` wildcard is supported.
    pub fn matches(&self, key: &TopicKey) -> bool {
        topic_pattern_matches(&self.0, &key.0)
    }
}

/// fn topic_pattern_matches(pattern: &str, key: &str) -> bool
/// Splits pattern on `*` → at most 2 parts: prefix + suffix.
/// key must start_with(prefix) AND end_with(suffix).
/// If no `*`, requires exact equality.
```

```rust
/// Parse a raw JSON frame into a StreamEvent.
/// Receives the full frame Value so parsers can read any field.
pub type ParserFn = fn(&Value) -> Result<StreamEvent, WebSocketError>;

/// Registry key: (stream kind, account type) → (topic pattern, parser).
///
/// Keyed by (StreamKind, AccountType) so that the same stream kind can have
/// different topic patterns on spot vs futures (e.g. Binance "@trade" vs "@aggTrade"
/// on futures).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegistryKey {
    pub kind: StreamKind,
    pub account_type: AccountType,
}

/// One registered entry: the wire topic pattern + parser function.
#[derive(Clone)]
pub struct RegistryEntry {
    pub pattern: TopicPattern,
    pub parser: ParserFn,
}

/// Maps (StreamKind, AccountType) → (TopicPattern, ParserFn).
///
/// Also maintains a reverse map TopicPattern → RegistryKey for O(patterns)
/// dispatch per frame (typically 5-40 patterns per exchange).
pub struct TopicRegistry {
    /// Primary map: per kind+account what pattern to subscribe with.
    entries: HashMap<RegistryKey, RegistryEntry>,

    /// Flattened list of (pattern, parser) for frame dispatch.
    /// Built once at construction; not mutated at runtime.
    dispatch: Vec<(TopicPattern, ParserFn)>,
}

impl TopicRegistry {
    pub fn builder() -> TopicRegistryBuilder { TopicRegistryBuilder::default() }

    /// Look up a parser for an incoming frame's topic key.
    /// Returns the first pattern that matches.  O(patterns).
    pub fn dispatch(&self, key: &TopicKey) -> Option<ParserFn> { ... }

    /// Returns true if (kind, account) has a registered parser.
    pub fn supports(&self, kind: &StreamKind, account: AccountType) -> bool { ... }

    /// Returns all (kind, account) pairs with Native support.
    pub fn native_pairs(&self) -> impl Iterator<Item = (&StreamKind, AccountType)> + '_ { ... }
}

#[derive(Default)]
pub struct TopicRegistryBuilder {
    entries: Vec<(RegistryKey, RegistryEntry)>,
}

impl TopicRegistryBuilder {
    /// Register a (kind, account_type, pattern, parser) entry.
    pub fn register(
        mut self,
        kind: StreamKind,
        account_type: AccountType,
        pattern: impl Into<String>,
        parser: ParserFn,
    ) -> Self { ... }

    pub fn build(self) -> TopicRegistry { ... }
}
```

**Dispatch cost**: ~10-40 pattern comparisons per frame.  Each comparison is two string
`starts_with`/`ends_with` calls.  At 800 events/s this is negligible.

---

### StreamSpec

Replaces `SubscriptionRequest` inside the framework layer.  `SubscriptionRequest` is kept for
the public `WebSocketConnector` trait (backward compat); `StreamSpec` is the internal
representation.

```rust
/// Internal subscription specification used by UniversalWsTransport.
///
/// Converted from SubscriptionRequest at subscribe() time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamSpec {
    pub kind: StreamKind,
    pub symbol: Symbol,
    pub account_type: AccountType,
    /// Depth hint for orderbook channels. None = exchange default.
    pub depth: Option<u32>,
    /// Speed hint in ms. None = exchange default.
    pub speed_ms: Option<u32>,
}

impl TryFrom<SubscriptionRequest> for StreamSpec {
    type Error = WebSocketError;
    fn try_from(req: SubscriptionRequest) -> Result<Self, Self::Error> { ... }
}
```

`StreamType` → `StreamKind` conversion is lossless (all `StreamType` variants map 1:1 to
`StreamKind`; `StreamType` is the old name).  The old `StreamType` enum in
`core/types/websocket.rs` is NOT removed in Wave 0 — it remains for backward compat with old
connectors.

---

## Traits

### WsProtocol

```rust
/// Per-exchange protocol shim.  Implement this for each exchange.
/// All methods are sync (no I/O).  The transport calls them to construct frames
/// and route incoming data.
///
/// Implementors MUST be Send + Sync + 'static (Arc-shared across tasks).
pub trait WsProtocol: Send + Sync + 'static {
    // ── Identity ──────────────────────────────────────────────────────────

    /// Short exchange name for log targets (e.g. "binance", "okx").
    fn name(&self) -> &'static str;

    /// WebSocket endpoint URL for given account type and network.
    fn endpoint(&self, account_type: AccountType, testnet: bool) -> url::Url;

    // ── Heartbeat ────────────────────────────────────────────────────────

    /// Frame to send as application-level ping.
    /// Return `None` to use native WebSocket Ping frames instead.
    ///
    /// - Bitget: `Some(Message::Text("ping".into()))`
    /// - OKX:    `Some(Message::Text("ping".into()))`
    /// - Binance: `None` (native WebSocket ping)
    /// - KuCoin: `Some(Message::Text(json!({"id":..,"type":"ping"}).to_string()))`
    fn ping_frame(&self) -> Option<Message>;

    /// Interval between application-level pings.
    /// Default: 30 seconds.
    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    // ── Subscription frames ───────────────────────────────────────────────

    /// Build the subscribe frame for one StreamSpec.
    /// Returns Err if the stream kind is not supported.
    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError>;

    /// Build the unsubscribe frame for one StreamSpec.
    /// Returns Err if the stream kind is not supported.
    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError>;

    // ── Auth ──────────────────────────────────────────────────────────────

    /// Optional authentication frame sent BEFORE any subscribe frames.
    ///
    /// Return `None` for fully public connectors (Binance public, Kraken, etc.).
    /// Return `Some(msg)` for exchanges that require LOGIN before SUBSCRIBE:
    /// OKX, HTX, KuCoin futures (token-based), Bitget private.
    ///
    /// The transport sends this frame immediately after connection is established
    /// and waits `auth_ack_timeout()` for an ack before proceeding.
    fn auth_frame(&self, credentials: &Credentials) -> Option<Result<Message, WebSocketError>>;

    /// How long to wait for the auth ack before timing out.
    /// Only relevant when `auth_frame` returns `Some(_)`.
    fn auth_ack_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    /// Returns true if the given raw frame is an auth success acknowledgment.
    /// Called only when `auth_frame` is `Some(_)`.
    fn is_auth_ack(&self, raw: &Value) -> bool {
        let _ = raw;
        false
    }

    // ── Frame classification ──────────────────────────────────────────────

    /// Extract the routing topic from an incoming frame.
    ///
    /// Returns `None` for:
    /// - Pong frames ("pong" text body on OKX/Bitget)
    /// - Subscribe ack frames
    /// - Auth ack frames
    /// - Heartbeat frames
    ///
    /// Returns `Some(TopicKey)` for data frames.
    ///
    /// The transport calls this, looks up in TopicRegistry, calls parser if found,
    /// or emits `tracing::warn!` if not found.
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey>;

    /// Returns true if the frame is a pong response to our ping.
    /// Used to suppress warn! for unmatched pong frames.
    fn is_pong(&self, raw: &Value) -> bool {
        let _ = raw;
        false
    }

    /// Returns true if the frame is a subscribe/unsubscribe acknowledgment.
    /// Used to suppress warn! for unmatched ack frames.
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        let _ = raw;
        false
    }

    // ── Registry ─────────────────────────────────────────────────────────

    /// Return the topic registry for this exchange+account_type combination.
    ///
    /// Called once at transport construction.  The registry is built at impl time
    /// and cached — this method does NOT allocate per-call.
    ///
    /// Most exchanges need one registry per AccountType (spot vs futures have
    /// different topic formats).  Pattern: cache in `OnceLock<TopicRegistry>`.
    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry;
}
```

**Why `topic_registry` is a method (not associated const):**  Associated consts cannot be
generic over `AccountType` at compile time without complex const generics.  A `&TopicRegistry`
method with `OnceLock`-backed lazy init is zero-cost after first call.

**Auth handling detail:**
- Pre-auth exchanges (OKX, HTX, KuCoin token, Bitget private): implement `auth_frame` +
  `is_auth_ack`.  Transport sends auth frame, reads frames until `is_auth_ack` returns true or
  timeout fires.
- Token-based auth (KuCoin): the token is fetched via REST before connect; it becomes part of
  the WS URL.  `auth_frame` returns `None`; URL already contains token.
- Binance private (listenKey): same pattern — listenKey in URL, `auth_frame` returns `None`.

---

### CapabilityProvider

```rust
/// Query a connector's WebSocket stream capabilities.
///
/// Implemented by UniversalWsTransport<P> via derivation from the TopicRegistry.
pub trait CapabilityProvider: Send + Sync {
    /// What level of support exists for (kind, account_type)?
    ///
    /// - `Native` ← registry has a parser entry for (kind, account)
    /// - `UnsupportedByExchange` ← no registry entry AND exchange impl
    ///   explicitly tagged this kind as "exchange has no channel"
    /// - `NotImplemented` ← no registry entry AND no explicit tag
    ///   (dig3 hasn't wired it yet)
    /// - `RequiresAuth` ← registry entry tagged with RequiresAuth
    fn supports(&self, kind: &StreamKind, account: AccountType) -> SupportLevel;

    /// Convenience: returns true iff supports() == Native.
    fn is_native(&self, kind: &StreamKind, account: AccountType) -> bool {
        self.supports(kind, account) == SupportLevel::Native
    }
}
```

`UniversalWsTransport<P>` derives `CapabilityProvider` by:
1. Calling `P::topic_registry(account_type).supports(kind, account_type)` → `Native` if true.
2. Otherwise consulting an optional `P::unsupported_kinds(account_type) -> &'static [StreamKind]`
   method (defaulting to `&[]`) to distinguish `UnsupportedByExchange` from `NotImplemented`.
3. A third optional tag set `P::requires_auth_kinds() -> &'static [StreamKind]` → `RequiresAuth`.

Add these to `WsProtocol` as optional methods with default `&[]` returns:

```rust
// ── Capability hints (optional, all default to empty) ─────────────────

/// Stream kinds this exchange has NO channel for (not a dig3 gap — exchange itself
/// does not provide it for the given account type).
fn unsupported_by_exchange(&self, account_type: AccountType) -> &'static [StreamKind] {
    let _ = account_type; &[]
}

/// Stream kinds that nominally exist but require credentials even for public data.
fn requires_auth_kinds(&self, account_type: AccountType) -> &'static [StreamKind] {
    let _ = account_type; &[]
}
```

---

## UniversalWsTransport\<P\>

### State machine

```
Disconnected
     │ connect() called
     ▼
 Connecting
     │ TCP + TLS handshake
     │ (auth_frame send + ack wait if P::auth_frame returns Some)
     │ (replay all active subscriptions)
     ▼
 Connected ──────────────────────────────────┐
     │                                       │
     │ Close frame / IO error                │ subscribe() / unsubscribe() commands
     ▼                                       │ Incoming frames dispatched
 Reconnecting                                │
     │ backoff sleep (BackoffState)          │
     │ loop back to Connecting               │
     ▼ (max_attempts reached OR Disconnect cmd)
 Disconnected
```

State is stored as `Arc<AtomicU8>` with a repr enum `TransportState`; no `RwLock` for status
reads (avoids contention on high-frequency reads from the ping task).

### Responsibilities

**Framework owns:**

| Concern | Detail |
|---|---|
| TCP + TLS connect | `tokio_tungstenite::connect_async` with `connection_timeout` |
| Auth handshake | Send `P::auth_frame()`, wait for `P::is_auth_ack()`, timeout → reconnect |
| Ping scheduler | `tokio::time::interval` ticking at `P::ping_interval()`; sends `P::ping_frame()` or native Ping frame |
| Subscription replay | On every successful connect, iterate `active_subs` set, call `P::subscribe_frame()`, send each |
| Frame routing | For every Text/Binary frame: decode JSON → `P::extract_topic()` → `TopicRegistry::dispatch()` → parser → emit |
| Unmatched frames | `tracing::warn!(target: "dig3::ws::unmatched", exchange = P::name(), topic = ?key, len = raw.to_string().len())` |
| Per-frame tracing | `tracing::trace!(target: "dig3::ws::frame", exchange = P::name(), frame = %raw)` |
| Binary decode | Detect gzip/deflate (MEXC, HTX compressed) by checking frame header; decompress before JSON parse |
| Reconnect backoff | `BackoffState` (initial 1s, max 30s, 2× multiplier + ±20% jitter) |
| Broadcast | `tokio::sync::broadcast::Sender<WebSocketResult<StreamEvent>>` — new receivers on each `event_stream()` call |

**Protocol shim owns:**

| Concern | Detail |
|---|---|
| URL construction | `fn endpoint(account_type, testnet) -> Url` |
| Subscribe frame format | JSON body (per-exchange wire format) |
| Topic extraction | Read `["stream"]`, `["e"]`, `["arg"]["channel"]`, etc. |
| Pong detection | `is_pong` returns true for `"pong"` text body or exchange-specific format |
| Auth frame | Optional LOGIN message |
| Registry | `OnceLock<TopicRegistry>` with per-(kind, account_type) entries |

### Internal structure

```rust
pub struct UniversalWsTransport<P: WsProtocol> {
    protocol:       Arc<P>,
    account_type:   AccountType,
    testnet:        bool,
    credentials:    Option<Credentials>,
    reconnect_cfg:  ReconnectConfig,

    // Runtime state (Arc-shared with tasks)
    state:          Arc<AtomicU8>,                   // TransportState repr
    active_subs:    Arc<TokioRwLock<HashSet<StreamSpec>>>,
    event_tx:       broadcast::Sender<WebSocketResult<StreamEvent>>,
    cmd_tx:         tokio::sync::mpsc::UnboundedSender<TransportCmd>,
}

enum TransportCmd {
    Subscribe(StreamSpec),
    Unsubscribe(StreamSpec),
    Shutdown,
}

#[repr(u8)]
enum TransportState {
    Disconnected = 0,
    Connecting   = 1,
    Connected    = 2,
    Reconnecting = 3,
}
```

The internal driver task (`tokio::spawn`) owns the actual WsStream split halves, the command
receiver, and the reconnect loop.  It sends events to `event_tx`.  The `UniversalWsTransport`
struct is `Clone`able (all fields are `Arc`/`mpsc` clone-safe).

### Public API

```rust
impl<P: WsProtocol> UniversalWsTransport<P> {
    /// Construct.  Does NOT connect yet.
    pub fn new(
        protocol: P,
        account_type: AccountType,
        testnet: bool,
        credentials: Option<Credentials>,
    ) -> Self;

    /// Construct with custom reconnect config.
    pub fn with_reconnect(
        protocol: P,
        account_type: AccountType,
        testnet: bool,
        credentials: Option<Credentials>,
        reconnect_cfg: ReconnectConfig,
    ) -> Self;

    /// Initiate connection.  Returns when Connected or times out.
    pub async fn connect(&self) -> WebSocketResult<()>;

    /// Graceful shutdown.
    pub async fn disconnect(&self) -> WebSocketResult<()>;

    /// Subscribe to a stream.  Returns immediately after queuing.
    /// Frame is sent to exchange inside the driver task.
    pub async fn subscribe(&self, spec: StreamSpec) -> WebSocketResult<()>;

    /// Unsubscribe from a stream.
    pub async fn unsubscribe(&self, spec: StreamSpec) -> WebSocketResult<()>;

    /// Returns a broadcast receiver stream.  Multiple callers → independent streams.
    /// Lag capacity: 4096 events (broadcast channel buffer).
    pub fn event_stream(&self) -> impl Stream<Item = WebSocketResult<StreamEvent>> + Send;

    /// Snapshot of current connection state.
    pub fn connection_status(&self) -> ConnectionStatus;

    /// Active subscriptions.
    pub fn active_subscriptions(&self) -> Vec<StreamSpec>;
}

impl<P: WsProtocol> CapabilityProvider for UniversalWsTransport<P> {
    fn supports(&self, kind: &StreamKind, account: AccountType) -> SupportLevel { ... }
}
```

**Broadcast channel vs mpsc:** Use `tokio::sync::broadcast` (not mpsc).  This allows multiple
independent consumers (e.g. ExchangeHub + mli-collector) to each receive all events without
cloning inside the hot path.  Lag/slow-consumer: events are silently dropped per broadcast
semantics; the driver logs `tracing::warn!` if it detects receiver lag > 512 events.  Buffer
capacity: 4096.

**Binary frame decode:**  `transport.rs` checks `Message::Binary`: if first 2 bytes are
`[0x1f, 0x8b]` → gzip; if first byte is `0x78` → zlib.  Decompress to UTF-8 string then
parse JSON.  MEXC sends deflate-compressed frames.  OKX Futures sends gzip.  This is transport
concern, not protocol concern.

---

## ReconnectConfig (promote from base_websocket.rs)

```rust
/// Reconnect backoff configuration.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 0 = infinite
    pub max_attempts:        u32,
    pub initial_delay_ms:    u64,
    pub max_delay_ms:        u64,
    pub backoff_multiplier:  f64,
    pub jitter_factor:       f64,   // 0.2 = ±20% randomization
    pub connection_timeout_ms: u64,
    /// Delay after auth failure before retry (longer than normal backoff).
    pub auth_failure_delay_ms: u64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts:         0,
            initial_delay_ms:     1_000,
            max_delay_ms:         30_000,
            backoff_multiplier:   2.0,
            jitter_factor:        0.2,
            connection_timeout_ms: 10_000,
            auth_failure_delay_ms: 5_000,
        }
    }
}

/// Mutable backoff state (held inside the driver task, not shared).
struct BackoffState {
    cfg:     ReconnectConfig,
    attempt: u32,
    current_delay_ms: u64,
}

impl BackoffState {
    fn next_delay(&mut self) -> Duration { ... } // applies multiplier + jitter
    fn reset(&mut self) { self.attempt = 0; self.current_delay_ms = self.cfg.initial_delay_ms; }
}
```

`BackoffState` lives exclusively inside the reconnect loop task — never shared.  No mutex.

---

## Migration adapter (WebSocketConnector blanket impl)

`WebSocketConnector` (`core/traits/websocket.rs`) is the existing public trait consumed by
`ExchangeHub` and `WebSocketPool`.  During Wave 1-2 migration it must be satisfied by both old
connector structs AND new `UniversalWsTransport<P>`.

**Strategy: blanket impl on `UniversalWsTransport<P>` that delegates to its own API.**

```rust
// In transport.rs or a new migration.rs:

#[async_trait]
impl<P: WsProtocol> WebSocketConnector for UniversalWsTransport<P> {
    async fn connect(&self, account_type: AccountType) -> WebSocketResult<()> {
        // account_type ignored — transport is already bound at construction
        self.connect().await
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        self.disconnect().await
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.connection_status()
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.subscribe(spec).await
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.unsubscribe(spec).await
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        Box::pin(self.event_stream())
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.active_subscriptions()
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect()
    }
}
```

`StreamSpec → SubscriptionRequest` and `SubscriptionRequest → StreamSpec` conversions are
implemented as `From`/`TryFrom` in `stream_spec.rs`.

Old connector structs (`BinanceWebSocket`, `OkxWebSocket`, etc.) remain untouched during Wave
0.  They already implement `WebSocketConnector`.  Migration happens in Wave 1 (one exchange at
a time): replace `BinanceWebSocket` with `UniversalWsTransport<BinanceProtocol>` at the
connector.rs site.

**No wrapper struct needed.**  The blanket impl above is sufficient.  Both old and new types
implement `WebSocketConnector`; `ExchangeHub` holds `Box<dyn WebSocketConnector>` — zero
changes required at the hub.

---

## Invariants

The implementer MUST preserve all of the following.  These are non-negotiable compile-time or
lint-enforced rules:

1. **No `_ => Ok(None)` in any frame handler.**  The only catch-all for unmatched topics is
   the `tracing::warn!` path in `transport.rs::dispatch_frame`.  Exchange shims (`WsProtocol`
   impls) parse specific known variants only; unrecognized frames return `None` from
   `extract_topic` (not `Ok(None)`) and the transport logs them.

2. **No `_ => {}` silent drop in `extract_topic`.**  If a frame has a topic field but the
   shim cannot recognize its format, return `Some(TopicKey("unknown:<raw>"))` rather than
   `None`.  The transport will log it.

3. **No sync `Mutex` held across `.await`.**  The driver task splits the WsStream immediately
   (`.split()`) and gives write-half exclusively to the command processor, read-half exclusively
   to the frame reader.  Neither side needs to share a mutex.

4. **`event_tx.take()` FORBIDDEN.**  Bitget's current bug.  The broadcast channel sender is
   stored behind `Arc` and is never taken or dropped during reconnect.  Receivers automatically
   receive a `Lagged` error if the buffer overflows, not a dead channel.

5. **Subscription replay is ALWAYS performed after reconnect.**  After every successful
   `Connected` state entry (including reconnects), the driver iterates `active_subs` and sends
   all subscribe frames before accepting new commands.

6. **`tracing::trace!` on EVERY data frame** (target `"dig3::ws::frame"`).  Conditional on
   `tracing` level, not on `DEBUG_WS` env var.  Do not use `eprintln!` in production code.

7. **`tracing::warn!` on EVERY unmatched topic** (target `"dig3::ws::unmatched"`).  Exchange
   name and raw topic string MUST appear in the log.  Never silently discard.

8. **`TopicRegistry` is immutable after construction.**  Built once via `TopicRegistryBuilder`,
   then frozen.  No runtime insertion.  Exchange shims use `OnceLock<TopicRegistry>`.

9. **`WsProtocol` impls are NOT allowed to spawn tasks.**  The transport owns all async
   execution.  Protocol methods are sync; they may not call `tokio::spawn`.

10. **Binary decompression in transport, not in shim.**  If an exchange sends gzip or deflate
    binary frames, the transport detects and decompresses.  The shim only receives a
    `serde_json::Value`.

11. **`StreamKind` variants with `interval` field MUST use `KlineInterval` (not `String`).**
    This ensures `Hash + Eq` correctness.

---

## Risks / Open questions

### R1 — KuCoin dynamic token URL
KuCoin requires a REST call to `/api/v1/bullet-public` before WS connect to get the token and
endpoint.  This is per-connection state, not a static URL.  `WsProtocol::endpoint()` returns
a `Url` but is sync.

**Resolution**: Add an optional async hook:
```rust
async fn pre_connect_hook(&self, http: &HttpClient) -> Result<Option<url::Url>, WebSocketError> {
    Ok(None)  // default: no pre-connect, use endpoint() directly
}
```
KuCoin's `WsProtocol` impl overrides this to fetch the token and return the dynamic URL.  The
transport calls `pre_connect_hook` first; if `Some(url)` is returned it overrides `endpoint()`.
Requires passing an `HttpClient` into the transport (already available since connectors already
hold one).

### R2 — HTX / OKX / MEXC binary frames
These exchanges send gzip or deflate-compressed binary frames.  Transport-level detection
covers the common cases.  MEXC specifically uses deflate-raw (no zlib header).  Need to confirm
the exact detection heuristic for deflate-raw vs zlib during Wave 1.

**Mitigation**: `WsProtocol` gets an optional method:
```rust
fn decode_binary(&self, bytes: &[u8]) -> Result<Value, WebSocketError> {
    // default: try gzip, then zlib, then raw deflate, then UTF-8
    transport_default_decode(bytes)
}
```
Override only when the exchange uses a non-standard encoding.

### R3 — Gateio "channel" topic vs symbol-in-payload
Gateio futures topics are e.g. `"futures.trades"` with symbol embedded in the payload body
(`data[0]["contract"]`).  `extract_topic` for Gateio must return a TopicKey that includes the
symbol suffix (e.g. `"futures.trades.BTC_USDT"`) for the pattern `"futures.trades.*"` to work.
This requires `extract_topic` to look at both `channel` field AND payload data.

**Resolution**: `extract_topic` receives the full `Value` (not just the topic field) so it can
composite the key.  This is already in the trait signature.

### R4 — `async_trait` removal
`WsProtocol` is fully sync — no `async fn` in the trait.  `WebSocketConnector` still uses
`async_trait` macro (existing code).  The blanket impl for `UniversalWsTransport` will also use
`async_trait` for compat.  Wave 3 can remove `async_trait` from `WebSocketConnector` once all
old connectors are migrated (AFIT is stable since 1.75).

### R5 — Broadcast lag on slow consumers
If a consumer (e.g. mli-collector with expensive processing) lags behind 4096 events, broadcast
will return `RecvError::Lagged`.  The consumer must handle this gracefully.  Document in
`event_stream()` rustdoc that callers MUST process or discard events promptly.

### R6 — `ConnectorCapabilities` flat bool struct migration
`ConnectorCapabilities` in `capabilities.rs:819-879` has 6 WS bool fields.  The spec requests
25 total (19 new).  In Wave 0 we add `CapabilityProvider` trait alongside; the flat struct is
NOT removed.  Wave 2 replaces the flat struct with a call to `CapabilityProvider::supports()`.
This avoids a breaking API change in Wave 0.

---

## Files to create

- `digdigdig3/src/core/websocket/stream_kind.rs` — `StreamKind` enum, `KlineInterval` newtype
- `digdigdig3/src/core/websocket/support_level.rs` — `SupportLevel` enum
- `digdigdig3/src/core/websocket/topic_registry.rs` — `TopicKey`, `TopicPattern`, `RegistryKey`, `RegistryEntry`, `TopicRegistry`, `TopicRegistryBuilder`, `ParserFn`, `topic_pattern_matches`
- `digdigdig3/src/core/websocket/stream_spec.rs` — `StreamSpec`, `TryFrom<SubscriptionRequest>`, `From<StreamSpec> for SubscriptionRequest`
- `digdigdig3/src/core/websocket/protocol.rs` — `WsProtocol` trait
- `digdigdig3/src/core/websocket/reconnect.rs` — `ReconnectConfig`, `BackoffState`
- `digdigdig3/src/core/websocket/transport.rs` — `UniversalWsTransport<P>`, `TransportCmd`, `TransportState`, blanket `WebSocketConnector` impl, `CapabilityProvider` impl
- `digdigdig3/src/core/websocket/mod.rs` — re-export all public types

## Files to modify

- `digdigdig3/src/core/websocket/mod.rs` — uncomment and redirect (currently `// mod base_websocket; // pub use base_websocket::...`)
- `digdigdig3/src/core/websocket/base_websocket.rs` — add `#[allow(dead_code)]` or gate with `#[cfg(test)]` temporarily; do NOT delete yet
- `digdigdig3/src/core/types/mod.rs` — re-export `StreamKind`, `SupportLevel`, `StreamSpec`, `KlineInterval` from `core::websocket`
- `digdigdig3/src/core/traits/mod.rs` — re-export `CapabilityProvider`
- `digdigdig3/Cargo.toml` — confirm `url` crate is in dependencies (needed for `Url` in `WsProtocol::endpoint`)

No exchange websocket.rs files are modified in Wave 0.  They are untouched until Wave 1.
