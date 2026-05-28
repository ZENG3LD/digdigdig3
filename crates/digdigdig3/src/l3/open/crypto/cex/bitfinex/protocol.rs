//! BitfinexProtocol — WsProtocol impl for Bitfinex WebSocket v2.
//!
//! ## Bitfinex WS v2 protocol
//!
//! - Endpoint: `wss://api-pub.bitfinex.com/ws/2` (public market data)
//! - Messages are either JSON **objects** (events) or JSON **arrays** (data).
//!
//! ### chanId integer routing
//!
//! Subscribe:
//!   `{"event":"subscribe","channel":"ticker","symbol":"tBTCUSD"}`
//! → Server ack:
//!   `{"event":"subscribed","chanId":17,"channel":"ticker","symbol":"tBTCUSD","pair":"BTCUSD"}`
//!
//! ALL subsequent data frames are `[17, [ticker_fields...]]` or `[17,"hb"]`.
//! Routing is by integer chanId, NOT channel name.
//!
//! `BitfinexProtocol` holds an `Arc<StdMutex<HashMap<u64, TopicKey>>>` populated
//! inside `is_subscribe_ack`. `extract_topic` reads `raw[0]` as u64, looks up
//! in the map, and returns the `TopicKey`.
//!
//! ### Topic key format
//!
//! Topics are per-symbol strings:
//!   - `"ticker:tBTCUSD"` — one entry per subscribed symbol
//!   - `"trades:tBTCUSD"`
//!   - `"book:tBTCUSD"`
//!   - `"candles:1m:tBTCUSD"` (key-based subscribe)
//!
//! Registry patterns use wildcards:
//!   - `"ticker:*"` matches any ticker topic
//!   - `"trades:*"` matches any trades topic
//!   - `"book:*"` matches any book topic
//!   - `"candles:*"` matches any candles topic (including `"candles:1m:tBTCUSD"`)
//!
//! This allows ONE parser registration per channel type to cover ALL subscribed
//! symbols.
//!
//! ### Symbol extraction in parsers
//!
//! Bitfinex data frames are `[chanId, data]`. The symbol is NOT embedded in the
//! frame — only the chanId is. Parsers need the symbol to emit `StreamEvent`.
//!
//! **Solution**: store the symbol in a thread-local during `extract_topic` (which
//! runs in the same task context as the subsequent parser dispatch). Parsers read
//! the thread-local to obtain the symbol.
//!
//! This works because `extract_topic` → `registry.dispatch_all()` → `parser(raw)`
//! are called sequentially within the same `async` task, so the thread-local set
//! in `extract_topic` is visible to the parsers.
//!
//! ### Reconnect safety
//!
//! On reconnect the transport replays subscribe frames, generating fresh acks.
//! `is_subscribe_ack` simply overwrites any existing chanId mapping — correct.
//!
//! ### Application-level ping
//!
//! `{"event":"ping","cid":0}` every 20 s.
//! Server replies with `{"event":"pong","ts":...,"cid":0}`.

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, Kline, OrderbookDelta as OrderbookDeltaData, StreamEvent, WebSocketError,
    WebSocketResult,
};
use crate::core::websocket::{KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};

use super::endpoints::format_symbol;
use super::parser::BitfinexParser;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

const WS_PUBLIC_URL: &str = "wss://api-pub.bitfinex.com/ws/2";

// ─────────────────────────────────────────────────────────────────────────────
// Thread-local symbol carrier
//
// Set by extract_topic just before returning; consumed by parser functions.
// Works because transport calls extract_topic → dispatch_all → parser() all
// in the same synchronous sequence within the same async task.
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    /// Symbol for the current frame being dispatched (e.g. `"tBTCUSD"`).
    /// Empty string means "unknown" — callers should handle gracefully.
    static CURRENT_SYMBOL: Cell<Option<String>> = const { Cell::new(None) };
    /// Kline interval for the current candle frame (e.g. `"1m"`).
    static CURRENT_INTERVAL: Cell<Option<String>> = const { Cell::new(None) };
}

fn set_current_symbol(sym: impl Into<String>) {
    CURRENT_SYMBOL.with(|c| c.set(Some(sym.into())));
}

fn take_current_symbol() -> String {
    CURRENT_SYMBOL.with(|c| c.take()).unwrap_or_default()
}

fn set_current_interval(interval: impl Into<String>) {
    CURRENT_INTERVAL.with(|c| c.set(Some(interval.into())));
}

fn take_current_interval() -> String {
    CURRENT_INTERVAL.with(|c| c.take()).unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache — Bitfinex is spot-only for public channels
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// BitfinexProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Bitfinex WS v2 protocol shim.
///
/// Holds `chan_map`: the chanId → TopicKey routing table. Populated lazily
/// from subscribe acks via `is_subscribe_ack` (interior mutability via
/// `Arc<StdMutex<…>>` — safe because no `.await` is held across the lock).
pub struct BitfinexProtocol {
    /// chanId → TopicKey routing table.
    /// Key: integer channel ID assigned by the server per subscription.
    /// Value: full per-symbol topic key (e.g. `"ticker:tBTCUSD"`).
    chan_map: Arc<StdMutex<HashMap<u64, TopicKey>>>,
}

impl BitfinexProtocol {
    pub fn new(_testnet: bool) -> Self {
        Self {
            chan_map: Arc::new(StdMutex::new(HashMap::new())),
        }
    }

    /// Resolve the Bitfinex wire symbol for a StreamSpec.
    ///
    /// Bitfinex symbol rules:
    /// - Trading pair: `t` prefix (e.g. `tBTCUSD`, `tETHUSD`)
    /// - Already prefixed symbols pass through unchanged.
    fn wire_symbol(spec: &StreamSpec) -> String {
        let raw = spec.symbol.to_string();
        // Already has the Bitfinex prefix — pass through.
        if raw.starts_with('t') || raw.starts_with('f') {
            return raw;
        }
        // Slash-separated (e.g. "BTC/USD") — split and format.
        if raw.contains('/') {
            let mut parts = raw.splitn(2, '/');
            let base = parts.next().unwrap_or("");
            let quote = parts.next().unwrap_or("USD");
            // format_symbol adds the "t" prefix for trading pairs.
            return format_symbol(base, quote, AccountType::Spot);
        }
        // Plain string like "BTCUSD" — add the "t" prefix.
        format!("t{}", raw)
    }

    /// Build the chanId → TopicKey string from a subscribe ack payload.
    ///
    /// Called from `is_subscribe_ack` (side-effect: mutates chan_map).
    fn topic_from_ack(channel: &str, symbol: Option<&str>, key: Option<&str>) -> Option<TopicKey> {
        match channel {
            "ticker" | "trades" | "book" => {
                let sym = symbol?;
                Some(TopicKey::new(format!("{}:{}", channel, sym)))
            }
            "candles" => {
                // ack key is "trade:<tf>:<symbol>", e.g. "trade:1m:tBTCUSD"
                // Strip "trade:" prefix → "1m:tBTCUSD", topic = "candles:1m:tBTCUSD"
                let k = key?;
                let stripped = k.strip_prefix("trade:").unwrap_or(k);
                Some(TopicKey::new(format!("candles:{}", stripped)))
            }
            "status" => {
                // status channel (deriv:, liq:global) — not in basic public WS scope.
                None
            }
            _ => None,
        }
    }

    /// Extract the Bitfinex wire symbol from a TopicKey.
    ///
    /// TopicKey format: `"<channel>:<symbol>"` or `"candles:<tf>:<symbol>"`.
    /// Returns the last colon-separated segment.
    fn symbol_from_topic(key: &TopicKey) -> &str {
        key.as_str()
            .rsplit(':')
            .next()
            .unwrap_or("")
    }

    /// Extract the kline interval from a candles TopicKey.
    ///
    /// TopicKey format: `"candles:<tf>:<symbol>"` → `"<tf>"`.
    fn interval_from_candles_topic(key: &TopicKey) -> &str {
        // "candles:1m:tBTCUSD" → split by ':' → ["candles", "1m", "tBTCUSD"]
        let s = key.as_str();
        let mut parts = s.splitn(3, ':');
        parts.next(); // "candles"
        parts.next().unwrap_or("") // "1m"
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for BitfinexProtocol {
    fn name(&self) -> &'static str {
        "bitfinex"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Bitfinex has no testnet for public channels.
        Url::parse(WS_PUBLIC_URL).expect("bitfinex ws endpoint is valid")
    }

    /// Application-level ping: `{"event":"ping","cid":0}` every 20 s.
    /// Server replies with `{"event":"pong","ts":...,"cid":0}`.
    fn ping_frame(&self) -> Option<WsFrame> {
        Some(WsFrame::Text(r#"{"event":"ping","cid":0}"#.to_string()))
    }

    /// 20-second ping interval — matches the bespoke loop behaviour.
    fn ping_interval(&self) -> Duration {
        Duration::from_secs(20)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let sym = Self::wire_symbol(spec);
        let frame = match &spec.kind {
            StreamKind::Ticker => json!({
                "event": "subscribe",
                "channel": "ticker",
                "symbol": sym,
            }),
            StreamKind::Trade => json!({
                "event": "subscribe",
                "channel": "trades",
                "symbol": sym,
            }),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => json!({
                "event": "subscribe",
                "channel": "book",
                "symbol": sym,
                "prec": "P0",
            }),
            StreamKind::Kline { interval } => {
                // Candle channel uses "key" instead of "symbol".
                // key format: "trade:<tf>:<symbol>"  e.g. "trade:1m:tBTCUSD"
                let key = format!("trade:{}:{}", interval.as_str(), sym);
                json!({
                    "event": "subscribe",
                    "channel": "candles",
                    "key": key,
                })
            }
            StreamKind::Liquidation => {
                return Err(WebSocketError::UnsupportedOperation(
                    "not yet implemented — public status channel key liq:global \
                     ({\"event\":\"subscribe\",\"channel\":\"status\",\"key\":\"liq:global\"})"
                        .into(),
                ))
            }
            other => {
                return Err(WebSocketError::NotSupported(format!(
                    "Bitfinex public WS has no channel for {:?}",
                    other
                )))
            }
        };
        Ok(WsFrame::Text(frame.to_string()))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let sym = Self::wire_symbol(spec);
        let topic = match &spec.kind {
            StreamKind::Ticker => format!("ticker:{}", sym),
            StreamKind::Trade => format!("trades:{}", sym),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => format!("book:{}", sym),
            StreamKind::Kline { interval } => {
                format!("candles:{}:{}", interval.as_str(), sym)
            }
            StreamKind::Liquidation => {
                return Err(WebSocketError::UnsupportedOperation(
                    "not yet implemented — public status channel key liq:global \
                     ({\"event\":\"subscribe\",\"channel\":\"status\",\"key\":\"liq:global\"})"
                        .into(),
                ))
            }
            other => {
                return Err(WebSocketError::NotSupported(format!(
                    "Bitfinex public WS has no channel for {:?}",
                    other
                )))
            }
        };
        let topic_key = TopicKey::new(&topic);

        let chan_map = self.chan_map.lock().expect("bitfinex chan_map poisoned");
        let chan_id = chan_map
            .iter()
            .find(|(_, v)| **v == topic_key)
            .map(|(k, _)| *k);
        drop(chan_map);

        match chan_id {
            Some(id) => Ok(WsFrame::Text(json!({"event":"unsubscribe","chanId": id}).to_string())),
            None => Err(WebSocketError::NotSupported(format!(
                "bitfinex: cannot unsubscribe from {} — chanId not yet known (ack pending?)",
                topic
            ))),
        }
    }

    /// Public channels require no authentication.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    /// Pong: `{"event":"pong","ts":...,"cid":...}`.
    fn is_pong(&self, raw: &Value) -> bool {
        raw.get("event").and_then(|v| v.as_str()) == Some("pong")
    }

    /// Subscribe ack / unsubscribe ack / info — suppress the unmatched warn.
    ///
    /// **Side effect** (interior mutability via StdMutex):
    /// - `event == "subscribed"`: insert chanId → TopicKey into `chan_map`.
    /// - `event == "unsubscribed"`: remove chanId from `chan_map`.
    /// - `event == "info"`: no-op.
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        let event = match raw.get("event").and_then(|v| v.as_str()) {
            Some(e) => e,
            None => return false,
        };

        match event {
            "subscribed" => {
                if let Some(chan_id) = raw.get("chanId").and_then(|v| v.as_u64()) {
                    let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");
                    let symbol = raw.get("symbol").and_then(|v| v.as_str());
                    let key = raw.get("key").and_then(|v| v.as_str());

                    if let Some(topic) = Self::topic_from_ack(channel, symbol, key) {
                        let mut map = self.chan_map.lock().expect("bitfinex chan_map poisoned");
                        map.insert(chan_id, topic);
                    }
                }
                true
            }
            "unsubscribed" => {
                if let Some(chan_id) = raw.get("chanId").and_then(|v| v.as_u64()) {
                    let mut map = self.chan_map.lock().expect("bitfinex chan_map poisoned");
                    map.remove(&chan_id);
                }
                true
            }
            // Post-connect info frame — not a subscribe ack, but suppresses the
            // unmatched warn so the transport doesn't log it as an unknown frame.
            "info" => true,
            "error" => false,
            _ => false,
        }
    }

    /// Bitfinex heartbeat is `[chanId, "hb"]` — an array frame handled in
    /// `extract_topic` by returning `None`. No server-initiated ping protocol.
    fn is_server_ping(&self, _raw: &Value) -> bool {
        false
    }

    /// Extract routing topic from an incoming Bitfinex frame.
    ///
    /// Array frames: `[chanId, <data>]` or `[chanId, "hb"]`.
    /// - `raw[0]` is the integer chanId.
    /// - `raw[1] == "hb"` → heartbeat, return `None`.
    /// - Otherwise → look up chanId in `chan_map`, return `Some(TopicKey)`.
    ///
    /// **Side effect**: before returning the topic, stores the symbol and (for
    /// candles) the interval in thread-locals so that parser functions — which
    /// receive only the raw frame — can emit a populated `symbol` field.
    /// This is valid because the transport calls `extract_topic` and then
    /// immediately calls the parser, all within the same task context.
    ///
    /// Object frames (events) have no `[chanId, …]` structure → return `None`.
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let arr = raw.as_array()?;
        if arr.is_empty() {
            return None;
        }

        let chan_id = arr[0].as_u64()?;

        // Heartbeat: [chanId, "hb"] — no dispatch needed.
        if arr.len() >= 2 && arr[1].as_str() == Some("hb") {
            return None;
        }

        let map = self.chan_map.lock().expect("bitfinex chan_map poisoned");
        let topic = map.get(&chan_id).cloned()?;
        drop(map);

        // Store symbol + interval in thread-locals for parser access.
        let sym = Self::symbol_from_topic(&topic).to_string();
        set_current_symbol(sym);

        if topic.as_str().starts_with("candles:") {
            let interval = Self::interval_from_candles_topic(&topic).to_string();
            set_current_interval(interval);
        }

        Some(topic)
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        REGISTRY.get_or_init(build_registry)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
//
// Wildcard patterns cover ALL subscribed symbols with a single registration:
//   "ticker:*"   → matches "ticker:tBTCUSD", "ticker:tETHUSD", etc.
//   "trades:*"   → matches "trades:tBTCUSD", etc.
//   "book:*"     → matches "book:tBTCUSD", etc.
//   "candles:*"  → matches "candles:1m:tBTCUSD", "candles:5m:tBTCUSD", etc.
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    let at = AccountType::Spot;
    TopicRegistry::builder()
        .register(StreamKind::Ticker, at, "ticker:*", parse_ticker_frame)
        .register(StreamKind::Trade, at, "trades:*", parse_trade_frame)
        .register(StreamKind::Orderbook, at, "book:*", parse_book_frame)
        .register(StreamKind::OrderbookDelta, at, "book:*", parse_book_frame)
        .register(
            StreamKind::Kline { interval: KlineInterval::new("") },
            at,
            "candles:*",
            parse_candle_frame,
        )
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions
//
// Each parser receives the full raw frame `[chanId, <data>]` (or variants like
// `[chanId, "te", data_array]` for trade executions).
//
// Symbol is obtained from the thread-local set by `extract_topic` just before
// this parser is invoked. This is sound because the transport dispatches
// extract_topic → parser in the same synchronous sequence within one task.
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `[chanId, [BID, BID_SIZE, ASK, ASK_SIZE, ...]]` → Ticker.
pub(crate) fn parse_ticker_frame(raw: &Value) -> WebSocketResult<StreamEvent> {
    let arr = raw
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("bitfinex ticker: expected array".into()))?;

    if arr.len() < 2 {
        return Err(WebSocketError::Parse("bitfinex ticker: array too short".into()));
    }

    let data = arr[1]
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("bitfinex ticker: data[1] not array".into()))?;

    let symbol = take_current_symbol();

    let ticker = BitfinexParser::parse_ws_ticker(data)
        .map_err(|e| WebSocketError::Parse(format!("bitfinex ticker: {}", e)))?;

    Ok(StreamEvent::Ticker { symbol, ticker })
}

/// Parse `[chanId, "te", [ID, MTS, AMOUNT, PRICE]]` or snapshot `[chanId, [[...],...]]` → Trade.
pub(crate) fn parse_trade_frame(raw: &Value) -> WebSocketResult<StreamEvent> {
    let arr = raw
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("bitfinex trade: expected array".into()))?;

    if arr.len() < 2 {
        return Err(WebSocketError::FieldAbsent("trade data".into()));
    }

    let symbol = take_current_symbol();

    // te frame: [chanId, "te", [ID, MTS, AMOUNT, PRICE]]
    if arr.len() >= 3 && arr[1].as_str() == Some("te") {
        let data = arr[2]
            .as_array()
            .ok_or_else(|| WebSocketError::FieldAbsent("te trade data array".into()))?;
        let trade = BitfinexParser::parse_ws_trade(data)
            .map_err(|e| WebSocketError::Parse(format!("bitfinex trade te: {}", e)))?;
        return Ok(StreamEvent::Trade { symbol, trade });
    }

    // tu (trade update) — duplicate of te, skip to avoid double emission.
    if arr.len() >= 2 && arr[1].as_str() == Some("tu") {
        return Err(WebSocketError::FieldAbsent("trade: tu suppressed (duplicate of te)".into()));
    }

    // Snapshot or single update: [chanId, [[...], ...]] or [chanId, [ID, MTS, AMOUNT, PRICE]]
    if let Some(data) = arr[1].as_array() {
        // Snapshot: first element is itself an array.
        if data.first().map(|v| v.is_array()).unwrap_or(false) {
            // Take the most-recent entry (index 0, Bitfinex newest-first).
            if let Some(inner) = data.first().and_then(|v| v.as_array()) {
                let trade = BitfinexParser::parse_ws_trade(inner)
                    .map_err(|e| WebSocketError::Parse(format!("bitfinex trade snapshot: {}", e)))?;
                return Ok(StreamEvent::Trade { symbol, trade });
            }
            return Err(WebSocketError::FieldAbsent("trade snapshot inner".into()));
        }
        // Single flat update: [ID, MTS, AMOUNT, PRICE]
        let trade = BitfinexParser::parse_ws_trade(data)
            .map_err(|e| WebSocketError::Parse(format!("bitfinex trade single: {}", e)))?;
        return Ok(StreamEvent::Trade { symbol, trade });
    }

    Err(WebSocketError::FieldAbsent("trade data".into()))
}

/// Parse `[chanId, [[PRICE, COUNT, AMOUNT], ...]]` or `[chanId, [PRICE, COUNT, AMOUNT]]` → OrderbookDelta.
pub(crate) fn parse_book_frame(raw: &Value) -> WebSocketResult<StreamEvent> {
    let arr = raw
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("bitfinex book: expected array".into()))?;

    if arr.len() < 2 {
        return Err(WebSocketError::FieldAbsent("book data".into()));
    }

    let symbol = take_current_symbol();

    let data = arr[1]
        .as_array()
        .ok_or_else(|| WebSocketError::FieldAbsent("book data[1] not array".into()))?;

    // Pass data directly to parse_ws_orderbook_delta.
    // For snapshot: data = [[PRICE, COUNT, AMOUNT], ...] → parser iterates outer entries.
    // For single update: data = [PRICE, COUNT, AMOUNT] → parser tries entry.as_array() on
    //   each scalar → returns None → skipped → empty bids/asks (suppressed below).
    // This matches the bespoke connector behaviour exactly.
    let delta: OrderbookDeltaData =
        BitfinexParser::parse_ws_orderbook_delta(data)
            .map_err(|e| WebSocketError::Parse(format!("bitfinex book: {}", e)))?;

    // Suppress pure-remove deltas (bids=[] AND asks=[]) — valid no-op for book state.
    if delta.bids.is_empty() && delta.asks.is_empty() {
        return Err(WebSocketError::FieldAbsent(
            "bitfinex book: pure-remove delta suppressed".into(),
        ));
    }

    Ok(StreamEvent::OrderbookDelta { symbol, delta })
}

/// Parse `[chanId, [MTS, OPEN, CLOSE, HIGH, LOW, VOLUME]]` or snapshot `[chanId, [[...],...]]` → Kline.
pub(crate) fn parse_candle_frame(raw: &Value) -> WebSocketResult<StreamEvent> {
    let arr = raw
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("bitfinex candle: expected array".into()))?;

    if arr.len() < 2 {
        return Err(WebSocketError::FieldAbsent("candle data".into()));
    }

    let symbol = take_current_symbol();
    let interval_str = take_current_interval();
    let interval = KlineInterval::new(&interval_str);

    let data = arr[1]
        .as_array()
        .ok_or_else(|| WebSocketError::FieldAbsent("candle data[1] not array".into()))?;

    let kline_data: &[Value] = if data.first().map(|v| v.is_array()).unwrap_or(false) {
        // Snapshot: [[MTS,O,C,H,L,V], ...] — take first (most recent, Bitfinex newest-first).
        match data.first().and_then(|v| v.as_array()) {
            Some(inner) => inner,
            None => return Err(WebSocketError::FieldAbsent("candle snapshot inner".into())),
        }
    } else {
        data
    };

    let kline: Kline = BitfinexParser::parse_ws_kline(kline_data)
        .map_err(|e| WebSocketError::Parse(format!("bitfinex candle: {}", e)))?;

    Ok(StreamEvent::Kline { symbol, interval, kline })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OwnedSymbolInput;
    use crate::core::websocket::StreamSpec;

    fn make_proto() -> BitfinexProtocol {
        BitfinexProtocol::new(false)
    }

    fn make_spec(kind: StreamKind, sym: &str) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: OwnedSymbolInput::Raw(sym.to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    // ── endpoint ──────────────────────────────────────────────────────────────

    #[test]
    fn endpoint_returns_public_url() {
        let proto = make_proto();
        let url = proto.endpoint(AccountType::Spot, false);
        assert_eq!(url.as_str(), "wss://api-pub.bitfinex.com/ws/2");
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_is_json_event_ping() {
        let proto = make_proto();
        let frame = proto.ping_frame().expect("ping_frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["event"], "ping");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn ping_interval_is_20_seconds() {
        let proto = make_proto();
        assert_eq!(proto.ping_interval(), Duration::from_secs(20));
    }

    // ── is_pong ───────────────────────────────────────────────────────────────

    #[test]
    fn is_pong_true_for_pong_event() {
        let proto = make_proto();
        let raw = serde_json::json!({"event": "pong", "ts": 12345, "cid": 0});
        assert!(proto.is_pong(&raw));
    }

    #[test]
    fn is_pong_false_for_data_array() {
        let proto = make_proto();
        let raw = serde_json::json!([17, [50000.0, 1.0, 50001.0, 1.0, 0.0, 0.0, 50000.0, 100.0, 51000.0, 49000.0]]);
        assert!(!proto.is_pong(&raw));
    }

    // ── is_subscribe_ack ──────────────────────────────────────────────────────

    #[test]
    fn is_subscribe_ack_returns_true_for_subscribed() {
        let proto = make_proto();
        let raw = serde_json::json!({
            "event": "subscribed",
            "channel": "ticker",
            "chanId": 17,
            "symbol": "tBTCUSD",
        });
        assert!(proto.is_subscribe_ack(&raw));
    }

    #[test]
    fn is_subscribe_ack_populates_chan_map() {
        let proto = make_proto();
        let raw = serde_json::json!({
            "event": "subscribed",
            "channel": "ticker",
            "chanId": 17,
            "symbol": "tBTCUSD",
        });
        proto.is_subscribe_ack(&raw);
        let map = proto.chan_map.lock().unwrap();
        assert_eq!(map.get(&17), Some(&TopicKey::new("ticker:tBTCUSD")));
    }

    #[test]
    fn is_subscribe_ack_candles_maps_key() {
        let proto = make_proto();
        let raw = serde_json::json!({
            "event": "subscribed",
            "channel": "candles",
            "chanId": 42,
            "key": "trade:1m:tBTCUSD",
        });
        proto.is_subscribe_ack(&raw);
        let map = proto.chan_map.lock().unwrap();
        assert_eq!(map.get(&42), Some(&TopicKey::new("candles:1m:tBTCUSD")));
    }

    #[test]
    fn is_subscribe_ack_unsubscribed_removes_entry() {
        let proto = make_proto();
        let sub = serde_json::json!({
            "event": "subscribed",
            "channel": "ticker",
            "chanId": 17,
            "symbol": "tBTCUSD",
        });
        proto.is_subscribe_ack(&sub);
        let unsub = serde_json::json!({
            "event": "unsubscribed",
            "chanId": 17,
            "status": "OK",
        });
        proto.is_subscribe_ack(&unsub);
        let map = proto.chan_map.lock().unwrap();
        assert!(!map.contains_key(&17));
    }

    #[test]
    fn is_subscribe_ack_true_for_info() {
        let proto = make_proto();
        let raw = serde_json::json!({"event": "info", "version": 2});
        assert!(proto.is_subscribe_ack(&raw));
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn extract_topic_returns_topic_for_known_chan() {
        let proto = make_proto();
        let ack = serde_json::json!({
            "event": "subscribed",
            "channel": "ticker",
            "chanId": 17,
            "symbol": "tBTCUSD",
        });
        proto.is_subscribe_ack(&ack);

        let data = serde_json::json!([17, [50000.0, 1.0, 50001.0, 1.0, 0.0, 0.0, 50000.0, 100.0, 51000.0, 49000.0]]);
        assert_eq!(
            proto.extract_topic(&data),
            Some(TopicKey::new("ticker:tBTCUSD"))
        );
    }

    #[test]
    fn extract_topic_sets_thread_local_symbol() {
        let proto = make_proto();
        let ack = serde_json::json!({
            "event": "subscribed",
            "channel": "ticker",
            "chanId": 17,
            "symbol": "tBTCUSD",
        });
        proto.is_subscribe_ack(&ack);

        let data = serde_json::json!([17, [50000.0, 1.0, 50001.0, 1.0, 0.0, 0.0, 50000.0, 100.0, 51000.0, 49000.0]]);
        proto.extract_topic(&data);
        // Symbol should now be in the thread-local (consumed by take).
        let sym = take_current_symbol();
        assert_eq!(sym, "tBTCUSD");
    }

    #[test]
    fn extract_topic_returns_none_for_heartbeat() {
        let proto = make_proto();
        let ack = serde_json::json!({
            "event": "subscribed",
            "channel": "ticker",
            "chanId": 17,
            "symbol": "tBTCUSD",
        });
        proto.is_subscribe_ack(&ack);

        let hb = serde_json::json!([17, "hb"]);
        assert_eq!(proto.extract_topic(&hb), None);
    }

    #[test]
    fn extract_topic_returns_none_for_unknown_chan() {
        let proto = make_proto();
        let data = serde_json::json!([999, [50000.0]]);
        assert_eq!(proto.extract_topic(&data), None);
    }

    #[test]
    fn extract_topic_returns_none_for_object_frame() {
        let proto = make_proto();
        let raw = serde_json::json!({"event": "subscribed", "chanId": 17});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_ticker() {
        let proto = make_proto();
        let spec = make_spec(StreamKind::Ticker, "tBTCUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["event"], "subscribe");
            assert_eq!(v["channel"], "ticker");
            assert_eq!(v["symbol"], "tBTCUSD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_trade() {
        let proto = make_proto();
        let spec = make_spec(StreamKind::Trade, "tBTCUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["channel"], "trades");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_orderbook() {
        let proto = make_proto();
        let spec = make_spec(StreamKind::Orderbook, "tBTCUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["channel"], "book");
            assert_eq!(v["symbol"], "tBTCUSD");
            assert_eq!(v["prec"], "P0");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_kline() {
        let proto = make_proto();
        let spec = make_spec(
            StreamKind::Kline { interval: KlineInterval::new("1m") },
            "tBTCUSD",
        );
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["channel"], "candles");
            assert_eq!(v["key"], "trade:1m:tBTCUSD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_plain_symbol_gets_t_prefix() {
        let proto = make_proto();
        let spec = make_spec(StreamKind::Ticker, "BTCUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["symbol"], "tBTCUSD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_liquidation_returns_unsupported_operation() {
        // Bitfinex public status channel liq:global exists — not yet implemented.
        let proto = make_proto();
        let spec = make_spec(StreamKind::Liquidation, "tBTCUSD");
        assert!(matches!(
            proto.subscribe_frame(&spec),
            Err(WebSocketError::UnsupportedOperation(_))
        ));
    }

    #[test]
    fn subscribe_frame_truly_absent_returns_not_supported() {
        // StreamKind::OpenInterest has no Bitfinex public WS channel.
        let proto = make_proto();
        let spec = make_spec(StreamKind::OpenInterest, "tBTCUSD");
        assert!(matches!(
            proto.subscribe_frame(&spec),
            Err(WebSocketError::NotSupported(_))
        ));
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_covers_public_channels() {
        let proto = make_proto();
        let reg = proto.topic_registry(AccountType::Spot);
        let at = AccountType::Spot;
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker");
        assert!(reg.supports(&StreamKind::Trade, at), "Trade");
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook");
        assert!(reg.supports(&StreamKind::OrderbookDelta, at), "OrderbookDelta");
        assert!(
            reg.supports(&StreamKind::Kline { interval: KlineInterval::new("") }, at),
            "Kline"
        );
    }

    #[test]
    fn topic_registry_wildcard_matches_per_symbol_keys() {
        let proto = make_proto();
        let reg = proto.topic_registry(AccountType::Spot);

        // These are the keys that extract_topic would return after a subscribe ack.
        assert!(
            reg.dispatch(&TopicKey::new("ticker:tBTCUSD")).is_some(),
            "ticker:tBTCUSD"
        );
        assert!(
            reg.dispatch(&TopicKey::new("trades:tETHUSD")).is_some(),
            "trades:tETHUSD"
        );
        assert!(
            reg.dispatch(&TopicKey::new("book:tBTCUSD")).is_some(),
            "book:tBTCUSD"
        );
        assert!(
            reg.dispatch(&TopicKey::new("candles:1m:tBTCUSD")).is_some(),
            "candles:1m:tBTCUSD"
        );
    }
}
