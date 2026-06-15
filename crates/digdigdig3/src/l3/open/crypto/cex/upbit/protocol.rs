//! UpbitProtocol — WsProtocol implementation for the Upbit exchange.
//!
//! ## Upbit WebSocket protocol
//!
//! - Endpoint: `wss://api.upbit.com/websocket/v1` (Korea / KRW markets — hardcoded)
//! - Subscribe frame format (3-element array):
//!   ```json
//!   [
//!     {"ticket":"<uuid>"},
//!     {"type":"trade","codes":["KRW-BTC"],"is_only_realtime":true},
//!     {"format":"DEFAULT"}
//!   ]
//!   ```
//! - No unsubscribe frame — Upbit does not support per-channel unsubscribe.
//!
//! ## Binary UTF-8 frames
//!
//! Upbit sends all data frames as `Message::Binary(utf8_json_bytes)` when using
//! `{"format":"DEFAULT"}`. The default `decode_binary` fallback chain in
//! `WsProtocol` ends with `String::from_utf8` then `serde_json::from_str` —
//! handles this transparently. No override needed.
//!
//! ## Ping/Pong
//!
//! Upbit uses standard WS-level `Message::Ping` / `Message::Pong`. The transport
//! layer handles autoreply. `ping_frame()` returns `None`.
//!
//! Upbit also sends `{"status":"UP"}` JSON liveness pings. These are matched in
//! `is_pong` so the transport suppresses the "unmatched topic" warning.
//!
//! ## No subscribe ACK
//!
//! Upbit starts sending data immediately after subscription; no ACK frame.
//! `is_subscribe_ack` always returns `false`.
//!
//! ## Channel routing
//!
//! The `type` field routes frames:
//! - `trade`     → `StreamEvent::Trade`
//! - `orderbook` → `StreamEvent::OrderbookSnapshot`
//! - `ticker`    → `StreamEvent::Ticker` (last price + 24h stats; bid/ask are None)
//!
//! ## Ticker
//!
//! Upbit exposes a native `ticker` channel carrying last price, 24h high/low/volume,
//! and change stats. Bid/ask are NOT part of the ticker frame — emitted as `None`.
//! Subscribe with `StreamKind::Ticker` to receive native `StreamEvent::Ticker` events.

use std::sync::OnceLock;

use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, OrderBook, OrderBookLevel, StreamEvent, WebSocketError, WebSocketResult,
};
use crate::core::websocket::{StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};

use super::parser::UpbitParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// UpbitProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Upbit WebSocket protocol shim.
///
/// Korea endpoint only (`wss://api.upbit.com/websocket/v1`). KRW-* pairs.
/// Supported: Trade, OrderbookSnapshot, Ticker.
pub struct UpbitProtocol;

impl UpbitProtocol {
    pub fn new(_testnet: bool) -> Self {
        Self
    }

    /// Build the 3-element Upbit subscribe frame for a single symbol+type.
    ///
    /// Format: `[{"ticket":"<uuid>"}, {"type":"<t>","codes":["<sym>"],"is_only_realtime":true}, {"format":"DEFAULT"}]`
    fn build_subscribe(upbit_type: &str, code: &str) -> WsFrame {
        let frame = json!([
            {"ticket": Uuid::new_v4().to_string()},
            {"type": upbit_type, "codes": [code], "is_only_realtime": true},
            {"format": "DEFAULT"}
        ]);
        WsFrame::Text(frame.to_string())
    }

    /// Resolve the Upbit wire symbol (`QUOTE-BASE`) from a StreamSpec.
    ///
    /// Upbit uses `KRW-BTC` format. Callers should pass raw Upbit-native codes
    /// (e.g. `"KRW-BTC"`) via `OwnedSymbolInput::Raw`.
    fn resolve_code(spec: &StreamSpec) -> Result<String, WebSocketError> {
        spec.symbol
            .resolve(crate::core::types::ExchangeId::Upbit, spec.account_type)
            .map(|s| s.to_ascii_uppercase())
            .map_err(|e| {
                WebSocketError::WireAbsent(format!(
                    "upbit: symbol normalization failed: {}",
                    e
                ))
            })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for UpbitProtocol {
    fn name(&self) -> &'static str {
        "upbit"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // KRW markets — Korea endpoint. sg/id/th endpoints serve different
        // quote currencies; factory.rs hardcodes "kr" (Korea).
        Url::parse("wss://api.upbit.com/websocket/v1")
            .expect("upbit ws endpoint is valid")
    }

    /// Returns `None`. Upbit accepts client WS Ping per spec, but under our
    /// read/write task split the auto-Pong is not flushed until the next
    /// outgoing frame (≈one ping_interval later) — longer than Upbit's Pong
    /// timeout, so the server drops the connection before data arrives.
    /// Disable native ping; rely on Upbit's `{"status":"UP"}` server ping +
    /// the silent-stream watchdog.
    fn ping_frame(&self) -> Option<WsFrame> {
        None
    }

    fn uses_native_ping(&self) -> bool {
        false
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let code = Self::resolve_code(spec)?;
        match &spec.kind {
            StreamKind::Trade => Ok(Self::build_subscribe("trade", &code)),
            StreamKind::Orderbook => Ok(Self::build_subscribe("orderbook", &code)),
            StreamKind::Ticker => Ok(Self::build_subscribe("ticker", &code)),
            other => Err(WebSocketError::WireAbsent(format!(
                "Upbit WS has no public channel for {:?}",
                other
            ))),
        }
    }

    /// Upbit does not support per-channel unsubscribe.
    ///
    /// A full reconnect is required to clear subscriptions. The transport's
    /// reconnect-on-resub logic handles this path.
    fn unsubscribe_frame(&self, _spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Err(WebSocketError::WireAbsent(
            "Upbit does not support per-channel unsubscribe — reconnect required".into(),
        ))
    }

    /// Public market-data channels are unauthenticated.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    /// Suppress Upbit's `{"status":"UP"}` liveness ping.
    ///
    /// Upbit sends this JSON-body frame as a keepalive signal. Returning `true`
    /// here prevents the transport from logging an "unmatched topic" warning.
    fn is_pong(&self, raw: &Value) -> bool {
        raw.get("status")
            .and_then(|v| v.as_str())
            .map(|s| s == "UP")
            .unwrap_or(false)
    }

    /// Upbit does not send subscribe ACK frames — data flows immediately.
    fn is_subscribe_ack(&self, _raw: &Value) -> bool {
        false
    }

    /// Extract routing topic from an Upbit data frame.
    ///
    /// Upbit frames carry a `type` (or short-form `ty`) field:
    /// - `"trade"`     → `TopicKey("trade")`
    /// - `"orderbook"` → `TopicKey("orderbook")`
    /// - `"ticker"`    → `TopicKey("ticker")`
    /// - `{"status":"UP"}` → `None` (already handled by `is_pong`)
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let t = raw
            .get("type")
            .or_else(|| raw.get("ty"))
            .and_then(|v| v.as_str())?;
        match t {
            "trade" | "orderbook" | "ticker" => Some(TopicKey::new(t)),
            _ => None,
        }
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        REGISTRY.get_or_init(build_registry)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    let at = AccountType::Spot;
    TopicRegistry::builder()
        .register(StreamKind::Trade, at, "trade", parse_trade)
        .register(StreamKind::Orderbook, at, "orderbook", parse_orderbook)
        .register(StreamKind::Ticker, at, "ticker", parse_ticker)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `trade` frame → `StreamEvent::Trade`.
///
/// Upbit trade frame fields:
/// - `code`: symbol (e.g. `"KRW-BTC"`)
/// - `trade_price`: price (f64)
/// - `trade_volume`: quantity (f64)
/// - `trade_timestamp`: ms since epoch
/// - `sequential_id`: unique trade ID (i64)
/// - `ask_bid`: `"ASK"` (taker sold) | `"BID"` (taker bought)
pub(crate) fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let trade = UpbitParser::parse_ws_trade(raw)
        .map_err(|e| WebSocketError::Parse(format!("upbit trade: {}", e)))?;

    Ok(StreamEvent::Trade { symbol, trade })
}

/// Parse `orderbook` frame → `StreamEvent::OrderbookSnapshot`.
///
/// Upbit orderbook frame fields:
/// - `code`: symbol
/// - `orderbook_units`: array of `{bid_price, bid_size, ask_price, ask_size}`
/// - `timestamp`: ms since epoch
pub(crate) fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let units = raw
        .get("orderbook_units")
        .and_then(|v| v.as_array())
        .ok_or_else(|| WebSocketError::Parse("upbit orderbook: missing orderbook_units".into()))?;

    let mut bids = Vec::new();
    let mut asks = Vec::new();

    for unit in units {
        let bid_price = unit.get("bid_price").and_then(|v| v.as_f64());
        let bid_size = unit.get("bid_size").and_then(|v| v.as_f64());
        let ask_price = unit.get("ask_price").and_then(|v| v.as_f64());
        let ask_size = unit.get("ask_size").and_then(|v| v.as_f64());

        if let (Some(p), Some(q)) = (bid_price, bid_size) {
            bids.push(OrderBookLevel::new(p, q));
        }
        if let (Some(p), Some(q)) = (ask_price, ask_size) {
            asks.push(OrderBookLevel::new(p, q));
        }
    }

    let timestamp = raw
        .get("timestamp")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let book = OrderBook {
        timestamp,
        bids,
        asks,
        sequence: None,
        last_update_id: None,
        first_update_id: None,
        prev_update_id: None,
        event_time: None,
        transaction_time: None,
        checksum: None,
        ..Default::default()
    };

    Ok(StreamEvent::OrderbookSnapshot { symbol, book })
}

/// Parse `ticker` frame → `StreamEvent::Ticker`.
///
/// Upbit ticker frame carries last-price + 24h stats. Bid/ask are NOT part
/// of the native ticker channel and are emitted as `None`.
///
/// Key fields:
/// - `code`: symbol (e.g. `"KRW-BTC"`)
/// - `trade_price`: last trade price
/// - `high_price`, `low_price`, `opening_price`: 24h candle
/// - `acc_trade_volume_24h`: base volume over 24h
/// - `acc_trade_price_24h`: quote volume over 24h
/// - `change_price`: absolute price change
/// - `change_rate`: fractional change (multiply × 100 for percent)
/// - `timestamp`: server send time (ms)
pub(crate) fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let ticker = UpbitParser::parse_ws_ticker(raw)
        .map_err(|e| WebSocketError::Parse(format!("upbit ticker: {}", e)))?;

    Ok(StreamEvent::Ticker { symbol, ticker })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AccountType, OwnedSymbolInput, TradeSide};
    use crate::core::websocket::{StreamSpec, WsProtocol};

    fn make_spec(kind: StreamKind, sym: &str) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: OwnedSymbolInput::Raw(sym.to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    fn proto() -> UpbitProtocol {
        UpbitProtocol::new(false)
    }

    // ── endpoint ──────────────────────────────────────────────────────────────

    #[test]
    fn endpoint_is_korea() {
        let url = proto().endpoint(AccountType::Spot, false);
        assert_eq!(url.as_str(), "wss://api.upbit.com/websocket/v1");
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_returns_none() {
        assert!(proto().ping_frame().is_none());
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_trade_is_3_element_array() {
        let spec = make_spec(StreamKind::Trade, "KRW-BTC");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            let arr = v.as_array().expect("outer array");
            assert_eq!(arr.len(), 3, "subscribe frame must be 3-element array");
            assert!(arr[0].get("ticket").is_some(), "element 0 must have ticket");
            assert_eq!(arr[1].get("type").and_then(|v| v.as_str()), Some("trade"));
            assert_eq!(arr[1]["codes"][0], "KRW-BTC");
            assert_eq!(arr[2].get("format").and_then(|v| v.as_str()), Some("DEFAULT"));
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_orderbook_is_3_element_array() {
        let spec = make_spec(StreamKind::Orderbook, "KRW-BTC");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            let arr = v.as_array().expect("outer array");
            assert_eq!(arr[1].get("type").and_then(|v| v.as_str()), Some("orderbook"));
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_ticker_is_3_element_array() {
        let spec = make_spec(StreamKind::Ticker, "KRW-BTC");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            let arr = v.as_array().expect("outer array");
            assert_eq!(arr.len(), 3, "subscribe frame must be 3-element array");
            assert!(arr[0].get("ticket").is_some(), "element 0 must have ticket");
            assert_eq!(arr[1].get("type").and_then(|v| v.as_str()), Some("ticker"));
            assert_eq!(arr[1]["codes"][0], "KRW-BTC");
            assert_eq!(arr[2].get("format").and_then(|v| v.as_str()), Some("DEFAULT"));
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn unsubscribe_frame_returns_not_supported() {
        let spec = make_spec(StreamKind::Trade, "KRW-BTC");
        let result = proto().unsubscribe_frame(&spec);
        assert!(
            matches!(result, Err(WebSocketError::WireAbsent(_))),
            "unsubscribe must return WireAbsent, got {:?}",
            result
        );
    }

    // ── is_pong ───────────────────────────────────────────────────────────────

    #[test]
    fn is_pong_matches_status_up() {
        let raw = serde_json::json!({"status": "UP"});
        assert!(proto().is_pong(&raw));
    }

    #[test]
    fn is_pong_false_for_data_frame() {
        let raw = serde_json::json!({"type": "trade", "code": "KRW-BTC"});
        assert!(!proto().is_pong(&raw));
    }

    // ── is_subscribe_ack ──────────────────────────────────────────────────────

    #[test]
    fn is_subscribe_ack_always_false() {
        let raw = serde_json::json!({"status": "UP"});
        assert!(!proto().is_subscribe_ack(&raw));
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn extract_topic_trade() {
        let raw = serde_json::json!({"type": "trade", "code": "KRW-BTC"});
        assert_eq!(proto().extract_topic(&raw), Some(TopicKey::new("trade")));
    }

    #[test]
    fn extract_topic_orderbook() {
        let raw = serde_json::json!({"type": "orderbook", "code": "KRW-BTC"});
        assert_eq!(proto().extract_topic(&raw), Some(TopicKey::new("orderbook")));
    }

    #[test]
    fn extract_topic_ticker() {
        let raw = serde_json::json!({"type": "ticker", "code": "KRW-BTC"});
        assert_eq!(proto().extract_topic(&raw), Some(TopicKey::new("ticker")));
    }

    #[test]
    fn extract_topic_status_up_returns_none() {
        let raw = serde_json::json!({"status": "UP"});
        assert_eq!(proto().extract_topic(&raw), None);
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_covers_trade_and_orderbook_and_ticker() {
        let p = proto();
        let reg = p.topic_registry(AccountType::Spot);
        let at = AccountType::Spot;
        assert!(reg.supports(&StreamKind::Trade, at), "Trade");
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook");
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker");
    }

    #[test]
    fn topic_registry_dispatches_by_topic_key() {
        let p = proto();
        let reg = p.topic_registry(AccountType::Spot);
        assert!(reg.dispatch(&TopicKey::new("trade")).is_some(), "trade");
        assert!(reg.dispatch(&TopicKey::new("orderbook")).is_some(), "orderbook");
        assert!(reg.dispatch(&TopicKey::new("ticker")).is_some(), "ticker registered");
    }

    // ── parse_trade ───────────────────────────────────────────────────────────

    #[test]
    fn parse_trade_basic() {
        let raw = serde_json::json!({
            "type": "trade",
            "code": "KRW-BTC",
            "trade_price": 90000000.0,
            "trade_volume": 0.001,
            "trade_timestamp": 1700000000000i64,
            "sequential_id": 12345,
            "ask_bid": "ASK"
        });
        let ev = parse_trade(&raw).expect("parse");
        match ev {
            StreamEvent::Trade { symbol, trade } => {
                assert_eq!(symbol, "KRW-BTC");
                assert!((trade.price - 90_000_000.0).abs() < f64::EPSILON);
                assert_eq!(trade.side, TradeSide::Sell);
            }
            other => panic!("expected Trade, got {:?}", other),
        }
    }

    // ── parse_ticker ──────────────────────────────────────────────────────────

    #[test]
    fn parse_ticker_basic() {
        let raw = serde_json::json!({
            "type": "ticker",
            "code": "KRW-BTC",
            "trade_price": 90500000.0,
            "high_price": 91000000.0,
            "low_price": 87500000.0,
            "acc_trade_volume_24h": 3200.0,
            "acc_trade_price_24h": 290000000000.0,
            "change_price": 2500000.0,
            "change_rate": 0.0284,
            "timestamp": 1718782303500i64
        });
        let ev = parse_ticker(&raw).expect("parse");
        match ev {
            StreamEvent::Ticker { symbol, ticker } => {
                assert_eq!(symbol, "KRW-BTC");
                assert!(ticker.last_price > 0.0, "last_price > 0");
                assert_eq!(ticker.bid_price, None);
                assert_eq!(ticker.ask_price, None);
                assert!(ticker.high_24h.is_some());
            }
            other => panic!("expected Ticker, got {:?}", other),
        }
    }

    // ── parse_orderbook ───────────────────────────────────────────────────────

    #[test]
    fn parse_orderbook_basic() {
        let raw = serde_json::json!({
            "type": "orderbook",
            "code": "KRW-BTC",
            "timestamp": 1700000000000i64,
            "orderbook_units": [
                {"bid_price": 89990000.0, "bid_size": 0.5, "ask_price": 90010000.0, "ask_size": 0.3}
            ]
        });
        let ev = parse_orderbook(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookSnapshot { symbol, book } => {
                assert_eq!(symbol, "KRW-BTC");
                assert_eq!(book.bids.len(), 1);
                assert_eq!(book.asks.len(), 1);
                assert!((book.bids[0].price - 89_990_000.0).abs() < f64::EPSILON);
            }
            other => panic!("expected OrderbookSnapshot, got {:?}", other),
        }
    }

    #[test]
    fn parse_orderbook_missing_units_returns_err() {
        let raw = serde_json::json!({"type": "orderbook", "code": "KRW-BTC"});
        assert!(parse_orderbook(&raw).is_err());
    }
}
