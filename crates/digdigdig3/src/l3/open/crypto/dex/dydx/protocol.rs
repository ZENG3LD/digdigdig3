//! DydxProtocol — WsProtocol implementation for dYdX v4 Indexer WebSocket.
//!
//! ## dYdX v4 Indexer WebSocket protocol
//!
//! - Mainnet endpoint: `wss://indexer.dydx.trade/v4/ws`
//! - Testnet endpoint: `wss://indexer.v4testnet.dydx.exchange/v4/ws`
//!
//! ## Subscribe frame
//!
//! ```json
//! {"type":"subscribe","channel":"v4_orderbook","id":"BTC-USD"}
//! ```
//!
//! For candles the id carries `<SYMBOL>/<RESOLUTION>` e.g. `"BTC-USD/1MIN"`.
//!
//! ## Frame types
//!
//! - `"connected"` — initial connection notification, no data.
//! - `"subscribed"` — subscription ack + initial snapshot in `contents`.
//!   Routed through `extract_topic` so parsers receive the snapshot.
//! - `"channel_data"` — incremental update.
//! - `"channel_batch_data"` — array of incremental updates.
//! - `"unsubscribed"` — unsub ack, suppressed by `is_subscribe_ack`.
//! - `"error"` — protocol error.
//!
//! ## Topic key
//!
//! `"<channel>:<id>"` e.g. `"v4_orderbook:BTC-USD"`, `"v4_candles:BTC-USD/1MIN"`.
//! For channels without an `id` field (e.g. `v4_markets`), key is `"<channel>:"`.
//!
//! ## Ping / pong
//!
//! dYdX v4 uses standard WebSocket Ping/Pong at protocol level.
//! No application-level ping frame needed. `ping_frame()` returns `None`.

use std::sync::OnceLock;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, StreamEvent, WebSocketError, WebSocketResult,
};
use crate::core::websocket::{KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};

use super::parser::DydxParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache — single registry (dYdX perp-only, no account type split)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// DydxProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative dYdX v4 Indexer WS protocol shim.
///
/// Public market-data channels only (public data by design):
/// orderbook, trades, candles, markets (ticker / funding / mark-price).
pub struct DydxProtocol {
    testnet: bool,
}

impl DydxProtocol {
    pub fn new(testnet: bool) -> Self {
        Self { testnet }
    }

    /// Build the `id` string for a subscribe frame.
    ///
    /// - Most channels use the dYdX market symbol directly: `"BTC-USD"`.
    /// - `v4_candles` appends the resolution: `"BTC-USD/1MIN"`.
    fn build_channel_id(spec: &StreamSpec) -> Result<(&'static str, String), WebSocketError> {
        let sym = wire_symbol(spec);
        match &spec.kind {
            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                Ok(("v4_orderbook", sym))
            }
            StreamKind::Trade | StreamKind::AggTrade => {
                Ok(("v4_trades", sym))
            }
            StreamKind::Kline { interval } => {
                let res = map_kline_to_dydx(interval.as_str());
                Ok(("v4_candles", format!("{}/{}", sym, res)))
            }
            StreamKind::Ticker | StreamKind::FundingRate | StreamKind::MarkPrice | StreamKind::MarketWarning => {
                // v4_markets is a global channel — id is the market symbol
                Ok(("v4_markets", sym))
            }
            other => Err(WebSocketError::NotSupported(format!(
                "dYdX v4 WS has no channel for {:?} (public data only; private channels are native-only by design)",
                other
            ))),
        }
    }
}

/// Map a common kline interval string to dYdX v4 candle resolution.
///
/// dYdX resolutions: `1MIN`, `5MINS`, `15MINS`, `30MINS`, `1HOUR`, `4HOURS`, `1DAY`.
pub(crate) fn map_kline_to_dydx(interval: &str) -> &'static str {
    match interval {
        "1m" | "1min" | "1MIN" => "1MIN",
        "5m" | "5min" | "5MINS" => "5MINS",
        "15m" | "15min" | "15MINS" => "15MINS",
        "30m" | "30min" | "30MINS" => "30MINS",
        "1h" | "1hour" | "1HOUR" | "60m" => "1HOUR",
        "4h" | "4hour" | "4HOURS" => "4HOURS",
        "1d" | "1day" | "1DAY" => "1DAY",
        _ => "1MIN",
    }
}

/// Resolve the dYdX wire symbol from a StreamSpec.
///
/// dYdX uses `"BTC-USD"` format. Converts `"BTC/USD"` → `"BTC-USD"`.
/// Identity for already-hyphenated symbols.
fn wire_symbol(spec: &StreamSpec) -> String {
    let raw = spec.symbol.to_string();
    if raw.contains('/') {
        // "BTC/USD" → "BTC-USD"
        raw.replace('/', "-")
    } else {
        raw
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for DydxProtocol {
    fn name(&self) -> &'static str {
        "dydx"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Use the testnet flag set at construction (stored in self.testnet).
        // The `_testnet` parameter from the transport is ignored intentionally —
        // the testnet mode is fixed at protocol construction time.
        let url = if self.testnet {
            "wss://indexer.v4testnet.dydx.exchange/v4/ws"
        } else {
            "wss://indexer.dydx.trade/v4/ws"
        };
        Url::parse(url).expect("dydx ws endpoint is valid")
    }

    /// dYdX v4 uses standard WebSocket Ping/Pong at protocol level.
    /// No application-level ping frame needed.
    fn ping_frame(&self) -> Option<WsFrame> {
        None
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let (channel, id) = Self::build_channel_id(spec)?;
        let frame = json!({
            "type": "subscribe",
            "channel": channel,
            "id": id,
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let (channel, id) = Self::build_channel_id(spec)?;
        let frame = json!({
            "type": "unsubscribe",
            "channel": channel,
            "id": id,
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    /// Public channels require no authentication.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    /// Suppress `"unsubscribed"` and `"connected"` system frames.
    ///
    /// `"subscribed"` is NOT suppressed here — it carries the initial snapshot
    /// in `contents` and must be routed through `extract_topic` to parsers.
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        match raw.get("type").and_then(|v| v.as_str()) {
            Some("unsubscribed") | Some("connected") => true,
            _ => false,
        }
    }

    /// Extract the topic key from an incoming dYdX frame.
    ///
    /// Routes `subscribed` (initial snapshot), `channel_data`, and
    /// `channel_batch_data`. Other types return `None`.
    ///
    /// Topic key format: `"<channel>:<id>"` e.g. `"v4_orderbook:BTC-USD"`.
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let msg_type = raw.get("type").and_then(|v| v.as_str())?;
        match msg_type {
            "subscribed" | "channel_data" | "channel_batch_data" => {}
            _ => return None,
        }
        let channel = raw.get("channel").and_then(|v| v.as_str())?;
        let id = raw.get("id").and_then(|v| v.as_str()).unwrap_or("");
        Some(TopicKey::new(&format!("{}:{}", channel, id)))
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        REGISTRY.get_or_init(build_registry)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
//
// Topic keys use `"<channel>:<id>"` format. Because the `id` varies per symbol
// we use wildcard patterns: `"v4_orderbook:*"`, `"v4_trades:*"`, etc.
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    // dYdX is perp-only — use FuturesCross as the canonical AccountType.
    let at = AccountType::FuturesCross;

    let mut b = TopicRegistry::builder()
        .register(StreamKind::Orderbook, at, "v4_orderbook:*", parse_orderbook)
        .register(StreamKind::OrderbookDelta, at, "v4_orderbook:*", parse_orderbook)
        .register(StreamKind::Trade, at, "v4_trades:*", parse_trade)
        .register(StreamKind::Ticker, at, "v4_markets:*", parse_markets_ticker)
        .register(StreamKind::FundingRate, at, "v4_markets:*", parse_markets_funding)
        .register(StreamKind::MarkPrice, at, "v4_markets:*", parse_markets_mark_price)
        .register(StreamKind::MarketWarning, at, "v4_markets:*", parse_markets_warning);

    // v4_candles: one parser per dYdX resolution, keyed as "v4_candles:<SYM>/<RES>"
    for res in DYDX_CANDLE_RESOLUTIONS {
        b = b.register(
            StreamKind::Kline { interval: KlineInterval::new(*res) },
            at,
            "v4_candles:*",
            parse_candle,
        );
    }

    b.build()
}

/// dYdX v4 supported candle resolutions.
const DYDX_CANDLE_RESOLUTIONS: &[&str] = &[
    "1MIN", "5MINS", "15MINS", "30MINS", "1HOUR", "4HOURS", "1DAY",
];

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions
//
// Each receives the full raw frame from dYdX:
// { "type": "...", "channel": "...", "id": "BTC-USD", "contents": {...} }
//
// The DydxParser methods expect this full-frame shape (they read `contents`
// and `id` directly from the top-level object).
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `v4_orderbook:*` → StreamEvent::OrderbookSnapshot or OrderbookDelta.
pub(crate) fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    DydxParser::parse_ws_orderbook_delta(raw)
        .map_err(|e| WebSocketError::Parse(format!("dydx orderbook: {}", e)))
}

/// Parse `v4_trades:*` → StreamEvent::Trade.
pub(crate) fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let trade = DydxParser::parse_ws_trade(raw)
        .map_err(|e| WebSocketError::Parse(format!("dydx trade: {}", e)))?;
    Ok(StreamEvent::Trade { symbol, trade })
}

/// Parse `v4_markets:*` → StreamEvent::Ticker.
///
/// The `id` field in the frame carries the subscribed market symbol.
/// Passed to `parse_ws_ticker` as `target_symbol` to extract that specific
/// market from the global markets map.
pub(crate) fn parse_markets_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let ticker = DydxParser::parse_ws_ticker(raw, &symbol)
        .map_err(|e| WebSocketError::Parse(format!("dydx markets ticker: {}", e)))?;
    Ok(StreamEvent::Ticker { symbol, ticker })
}

/// Parse `v4_markets:*` → StreamEvent::FundingRate.
pub(crate) fn parse_markets_funding(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let contents = raw.get("contents")
        .ok_or_else(|| WebSocketError::Parse("dydx markets/funding: missing 'contents'".into()))?;

    let market = extract_market_entry(contents, &symbol)
        .ok_or_else(|| WebSocketError::Parse(format!(
            "dydx markets/funding: market '{}' not found", symbol
        )))?;

    let rate = parse_f64_str(market, "nextFundingRate")
        .ok_or_else(|| WebSocketError::Parse("dydx markets/funding: missing 'nextFundingRate'".into()))?;

    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::FundingRate {
        symbol,
        rate,
        next_funding_time: None,
        timestamp: now,
    })
}

/// Parse `v4_markets:*` → StreamEvent::MarkPrice (via `oraclePrice`).
pub(crate) fn parse_markets_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let contents = raw.get("contents")
        .ok_or_else(|| WebSocketError::Parse("dydx markets/mark: missing 'contents'".into()))?;

    let market = extract_market_entry(contents, &symbol)
        .ok_or_else(|| WebSocketError::Parse(format!(
            "dydx markets/mark: market '{}' not found", symbol
        )))?;

    let mark_price = parse_f64_str(market, "oraclePrice")
        .ok_or_else(|| WebSocketError::Parse("dydx markets/mark: missing 'oraclePrice'".into()))?;

    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::MarkPrice {
        symbol,
        mark_price,
        index_price: None,
        timestamp: now,
    })
}

/// Parse `v4_markets:*` → StreamEvent::MarketWarning.
///
/// dYdX `v4_markets` delta frames can carry `status` field changes.
/// We emit a `MarketWarning` when `status != "ACTIVE"`.
pub(crate) fn parse_markets_warning(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let contents = raw.get("contents")
        .ok_or_else(|| WebSocketError::Parse("dydx markets/warning: missing 'contents'".into()))?;

    let market = extract_market_entry(contents, &symbol)
        .ok_or_else(|| WebSocketError::Parse(format!(
            "dydx markets/warning: market '{}' not found", symbol
        )))?;

    let status = market.get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("ACTIVE");

    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::MarketWarning {
        symbol: Some(symbol),
        warning_kind: format!("status:{}", status),
        message: format!("dYdX market status changed to {}", status),
        timestamp: now,
    })
}

/// Parse `v4_candles:*` → StreamEvent::Kline.
pub(crate) fn parse_candle(raw: &Value) -> WebSocketResult<StreamEvent> {
    DydxParser::parse_ws_candle(raw)
        .map_err(|e| WebSocketError::Parse(format!("dydx candle: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// v4_markets helpers
//
// The `v4_markets` channel is global. Its frame shape is:
//
// Snapshot (subscribed):
//   {"contents":{"markets":{"BTC-USD":{...},"ETH-USD":{...}}}}
//
// Delta (channel_data):
//   {"contents":{"BTC-USD":{...}}}  OR  {"contents":{"markets":{"BTC-USD":{...}}}}
// ─────────────────────────────────────────────────────────────────────────────

fn extract_market_entry<'a>(contents: &'a Value, symbol: &str) -> Option<&'a Value> {
    // Snapshot: contents.markets.{SYM}
    if let Some(entry) = contents.get("markets").and_then(|m| m.get(symbol)) {
        return Some(entry);
    }
    // Delta: contents.{SYM} directly
    contents.get(symbol)
}

fn parse_f64_str(obj: &Value, key: &str) -> Option<f64> {
    obj.get(key).and_then(|v| {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OwnedSymbolInput;
    use crate::core::websocket::StreamSpec;

    fn make_spec(kind: StreamKind, sym: &str) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: OwnedSymbolInput::Raw(sym.to_string()),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        }
    }

    fn proto() -> DydxProtocol {
        DydxProtocol::new(false)
    }

    // ── endpoint ──────────────────────────────────────────────────────────────

    #[test]
    fn mainnet_endpoint() {
        let url = proto().endpoint(AccountType::FuturesCross, false);
        assert_eq!(url.as_str(), "wss://indexer.dydx.trade/v4/ws");
    }

    #[test]
    fn testnet_endpoint() {
        let url = DydxProtocol::new(true).endpoint(AccountType::FuturesCross, true);
        assert_eq!(url.as_str(), "wss://indexer.v4testnet.dydx.exchange/v4/ws");
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_returns_none() {
        assert!(proto().ping_frame().is_none());
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_orderbook() {
        let spec = make_spec(StreamKind::Orderbook, "BTC-USD");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["type"], "subscribe");
            assert_eq!(v["channel"], "v4_orderbook");
            assert_eq!(v["id"], "BTC-USD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_trade() {
        let spec = make_spec(StreamKind::Trade, "ETH-USD");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["channel"], "v4_trades");
            assert_eq!(v["id"], "ETH-USD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_kline_1min() {
        let spec = StreamSpec {
            kind: StreamKind::Kline { interval: KlineInterval::new("1m") },
            symbol: OwnedSymbolInput::Raw("BTC-USD".to_string()),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        };
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["channel"], "v4_candles");
            assert_eq!(v["id"], "BTC-USD/1MIN");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_markets_ticker() {
        let spec = make_spec(StreamKind::Ticker, "BTC-USD");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["channel"], "v4_markets");
            assert_eq!(v["id"], "BTC-USD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_slash_symbol_normalised() {
        // BTC/USD → BTC-USD on the wire
        let spec = make_spec(StreamKind::Trade, "BTC/USD");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["id"], "BTC-USD");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── unsubscribe_frame ─────────────────────────────────────────────────────

    #[test]
    fn unsubscribe_orderbook() {
        let spec = make_spec(StreamKind::Orderbook, "BTC-USD");
        let frame = proto().unsubscribe_frame(&spec).expect("unsub frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["type"], "unsubscribe");
            assert_eq!(v["channel"], "v4_orderbook");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── is_subscribe_ack ──────────────────────────────────────────────────────

    #[test]
    fn ack_connected() {
        let raw = serde_json::json!({"type":"connected","connection_id":"abc"});
        assert!(proto().is_subscribe_ack(&raw));
    }

    #[test]
    fn ack_unsubscribed() {
        let raw = serde_json::json!({"type":"unsubscribed","channel":"v4_orderbook","id":"BTC-USD"});
        assert!(proto().is_subscribe_ack(&raw));
    }

    #[test]
    fn ack_subscribed_is_false() {
        // "subscribed" must NOT be suppressed — it carries the initial snapshot
        let raw = serde_json::json!({"type":"subscribed","channel":"v4_orderbook","id":"BTC-USD","contents":{}});
        assert!(!proto().is_subscribe_ack(&raw));
    }

    #[test]
    fn ack_channel_data_is_false() {
        let raw = serde_json::json!({"type":"channel_data","channel":"v4_orderbook","id":"BTC-USD","contents":{}});
        assert!(!proto().is_subscribe_ack(&raw));
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn topic_from_subscribed() {
        let raw = serde_json::json!({
            "type": "subscribed",
            "channel": "v4_orderbook",
            "id": "BTC-USD",
            "contents": {}
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("v4_orderbook:BTC-USD"))
        );
    }

    #[test]
    fn topic_from_channel_data() {
        let raw = serde_json::json!({
            "type": "channel_data",
            "channel": "v4_trades",
            "id": "ETH-USD",
            "contents": {}
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("v4_trades:ETH-USD"))
        );
    }

    #[test]
    fn topic_none_for_connected() {
        let raw = serde_json::json!({"type":"connected","connection_id":"x"});
        assert_eq!(proto().extract_topic(&raw), None);
    }

    #[test]
    fn topic_none_for_unsubscribed() {
        let raw = serde_json::json!({"type":"unsubscribed","channel":"v4_orderbook","id":"BTC-USD"});
        assert_eq!(proto().extract_topic(&raw), None);
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn registry_supports_public_channels() {
        let p = proto();
        let reg = p.topic_registry(AccountType::FuturesCross);
        let at = AccountType::FuturesCross;
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook");
        assert!(reg.supports(&StreamKind::Trade, at), "Trade");
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker");
        assert!(reg.supports(&StreamKind::FundingRate, at), "FundingRate");
        assert!(reg.supports(&StreamKind::MarkPrice, at), "MarkPrice");
        assert!(
            reg.supports(&StreamKind::Kline { interval: KlineInterval::new("1MIN") }, at),
            "Kline"
        );
    }

    #[test]
    fn registry_wildcard_dispatches_per_symbol() {
        let p = proto();
        let reg = p.topic_registry(AccountType::FuturesCross);
        assert!(reg.dispatch(&TopicKey::new("v4_orderbook:BTC-USD")).is_some());
        assert!(reg.dispatch(&TopicKey::new("v4_trades:ETH-USD")).is_some());
        assert!(reg.dispatch(&TopicKey::new("v4_markets:BTC-USD")).is_some());
        assert!(reg.dispatch(&TopicKey::new("v4_candles:BTC-USD/1MIN")).is_some());
    }

    // ── kline interval mapping ────────────────────────────────────────────────

    #[test]
    fn kline_interval_mapping() {
        assert_eq!(map_kline_to_dydx("1m"), "1MIN");
        assert_eq!(map_kline_to_dydx("5m"), "5MINS");
        assert_eq!(map_kline_to_dydx("15m"), "15MINS");
        assert_eq!(map_kline_to_dydx("30m"), "30MINS");
        assert_eq!(map_kline_to_dydx("1h"), "1HOUR");
        assert_eq!(map_kline_to_dydx("4h"), "4HOURS");
        assert_eq!(map_kline_to_dydx("1d"), "1DAY");
    }
}
