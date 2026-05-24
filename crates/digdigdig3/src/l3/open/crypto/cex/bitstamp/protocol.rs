//! BitstampProtocol — WsProtocol implementation for the Bitstamp exchange.
//!
//! Declarative shim: supplies endpoint URLs, ping frame, subscribe/unsubscribe
//! frames, topic extraction, and topic registry to UniversalWsTransport.
//!
//! Bitstamp uses the Pusher protocol over WebSocket:
//! - Subscribe: `{"event":"bts:subscribe","data":{"channel":"<channel>"}}`
//! - Unsubscribe: `{"event":"bts:unsubscribe","data":{"channel":"<channel>"}}`
//! - Subscribe ACK: `{"event":"bts:subscription_succeeded","channel":"...","data":""}`
//! - Heartbeat ping: `{"event":"pusher:ping","data":{}}`
//! - Heartbeat pong: `{"event":"pusher:pong","data":{}}`
//! - Data frames: `{"event":"<name>","channel":"<name>","data":{...}}`
//!
//! Bitstamp has no testnet — the testnet parameter is accepted but ignored.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::core::traits::Credentials;
use crate::core::types::{AccountType, OrderBook, OrderBookLevel, OrderSide, OrderbookDelta, StreamEvent, Ticker, WebSocketError, WebSocketResult};
use crate::core::websocket::{
    StreamKind, StreamSpec,
    TopicKey, TopicRegistry,
    WsProtocol,
};

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache (Bitstamp is spot-only, one registry)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// BitstampProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Bitstamp WS protocol shim.
pub struct BitstampProtocol;

impl BitstampProtocol {
    /// Derive wire channel name from StreamSpec.
    ///
    /// Symbol is expected to already be in exchange-native format (e.g. "btcusd").
    /// The pair comes from `spec.symbol.resolve(ExchangeId::Bitstamp, spec.account_type)`.
    pub(crate) fn channel_name(spec: &StreamSpec) -> Result<String, WebSocketError> {
        // Resolve the symbol to exchange-native pair string.
        // For Bitstamp (spot-only), Raw inputs are passed through verbatim.
        // Canonical inputs are normalized; normalization errors map to NotSupported.
        let pair = spec
            .symbol
            .resolve(crate::core::types::ExchangeId::Bitstamp, spec.account_type)
            .map_err(|e| {
                WebSocketError::NotSupported(format!(
                    "bitstamp: symbol normalization failed: {}",
                    e
                ))
            })?;
        let pair_lc = pair.to_ascii_lowercase();

        let channel = match &spec.kind {
            StreamKind::Trade => format!("live_trades_{}", pair_lc),
            // Both Ticker and Orderbook share the same wire channel.
            StreamKind::Ticker => format!("order_book_{}", pair_lc),
            StreamKind::Orderbook => format!("order_book_{}", pair_lc),
            StreamKind::OrderbookDelta => format!("diff_order_book_{}", pair_lc),
            StreamKind::OrderbookL3 => format!("live_orders_{}", pair_lc),
            other => {
                return Err(WebSocketError::NotSupported(
                    format!("Bitstamp has no WS channel for {:?}", other),
                ));
            }
        };
        Ok(channel)
    }

    /// Build subscribe / unsubscribe frame.
    fn build_frame(op: &str, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let channel = Self::channel_name(spec)?;
        let frame = serde_json::json!({
            "event": op,
            "data": { "channel": channel }
        });
        Ok(Message::Text(frame.to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for BitstampProtocol {
    fn name(&self) -> &'static str {
        "bitstamp"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Bitstamp is spot-only and has no testnet.
        Url::parse("wss://ws.bitstamp.net").expect("bitstamp ws endpoint is valid")
    }

    /// Pusher application-level ping: text frame `{"event":"pusher:ping","data":{}}`.
    fn ping_frame(&self) -> Option<Message> {
        Some(Message::Text(
            r#"{"event":"pusher:ping","data":{}}"#.to_string(),
        ))
    }

    /// 30-second ping interval — Pusher activity_timeout is 120 s, 30 s is safe.
    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("bts:subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("bts:unsubscribe", spec)
    }

    /// Bitstamp public WS is unauthenticated.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        None
    }

    fn is_pong(&self, raw: &Value) -> bool {
        raw.get("event").and_then(|v| v.as_str()) == Some("pusher:pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        matches!(
            raw.get("event").and_then(|v| v.as_str()),
            Some("bts:subscription_succeeded")
        )
    }

    /// Extract routing topic from Pusher data frame.
    ///
    /// The `channel` field is the TopicKey for Bitstamp frames.
    /// Returns None for protocol control frames (connection_established, pong,
    /// subscription_succeeded, error, request_reconnect).
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let event = raw.get("event").and_then(|v| v.as_str())?;
        match event {
            "pusher:connection_established"
            | "pusher:pong"
            | "pusher:error"
            | "bts:subscription_succeeded"
            | "bts:error"
            | "bts:request_reconnect" => None,
            _ => raw
                .get("channel")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(TopicKey::new),
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
        // live_trades_* → Trade
        .register(StreamKind::Trade, at, "live_trades_*", parse_trade)
        // order_book_* → Ticker (synthetic from best bid/ask)
        .register(StreamKind::Ticker, at, "order_book_*", parse_ticker_from_ob)
        // order_book_* → Orderbook snapshot (same channel, both registered → dispatch_all)
        .register(StreamKind::Orderbook, at, "order_book_*", parse_orderbook_snapshot)
        // diff_order_book_* → OrderbookDelta
        .register(StreamKind::OrderbookDelta, at, "diff_order_book_*", parse_orderbook_delta)
        // live_orders_* → OrderbookL3 (per-order create/changed/delete lifecycle)
        .register(StreamKind::OrderbookL3, at, "live_orders_*", parse_live_order)
        // detail_order_book_* → OrderbookL3 (full snapshot, N entries per frame)
        // Registered under Orderbook so dispatch_all can find it; parsers return the first event.
        // Full Vec emission is handled specially by parse_detail_ob_l3_first (see below).
        .register(StreamKind::OrderbookL3, at, "detail_order_book_*", parse_detail_ob_l3_first)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions — ParserFn = fn(&Value) -> WebSocketResult<StreamEvent>
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `live_trades_*` frame → StreamEvent::Trade.
pub(crate) fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw
        .get("channel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("trade: missing channel".into()))?;
    let symbol = channel
        .trim_start_matches("live_trades_")
        .to_ascii_uppercase();

    let trade = crate::l3::open::crypto::cex::bitstamp::parser::BitstampParser::parse_ws_trade(raw)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;

    Ok(StreamEvent::Trade {
        symbol,
        trade: crate::core::types::PublicTrade {
            id: trade.id,
            price: trade.price,
            quantity: trade.quantity,
            side: trade.side,
            timestamp: trade.timestamp,
        },
    })
}

/// Parse `order_book_*` frame → StreamEvent::Ticker (synthetic from best bid/ask).
pub(crate) fn parse_ticker_from_ob(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw
        .get("channel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("ticker_from_ob: missing channel".into()))?;
    let symbol = channel
        .trim_start_matches("order_book_")
        .to_ascii_uppercase();

    let book = parse_orderbook_inner(raw)?;
    let bid = book.bids.first().map(|l| l.price);
    let ask = book.asks.first().map(|l| l.price);
    let last_price = match (bid, ask) {
        (Some(b), Some(a)) => (b + a) / 2.0,
        (Some(b), None) => b,
        (None, Some(a)) => a,
        (None, None) => 0.0,
    };

    let ticker = Ticker {
        last_price,
        bid_price: bid,
        ask_price: ask,
        high_24h: None,
        low_24h: None,
        volume_24h: None,
        quote_volume_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: book.timestamp,
    };
    Ok(StreamEvent::Ticker { symbol, ticker })
}

/// Parse `order_book_*` frame → StreamEvent::OrderbookSnapshot.
pub(crate) fn parse_orderbook_snapshot(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw
        .get("channel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("ob_snapshot: missing channel".into()))?;
    let symbol = channel
        .trim_start_matches("order_book_")
        .to_ascii_uppercase();

    let book = parse_orderbook_inner(raw)?;
    Ok(StreamEvent::OrderbookSnapshot { symbol, book })
}

/// Parse `diff_order_book_*` frame → StreamEvent::OrderbookDelta.
pub(crate) fn parse_orderbook_delta(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw
        .get("channel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("ob_delta: missing channel".into()))?;
    let symbol = channel
        .trim_start_matches("diff_order_book_")
        .to_ascii_uppercase();

    let book = parse_orderbook_inner(raw)?;
    let delta = OrderbookDelta {
        bids: book.bids,
        asks: book.asks,
        timestamp: book.timestamp,
        first_update_id: book.first_update_id,
        last_update_id: book.last_update_id,
        prev_update_id: book.prev_update_id,
        event_time: book.event_time,
        checksum: book.checksum,
    };
    Ok(StreamEvent::OrderbookDelta { symbol, delta })
}

/// Shared orderbook parser (order_book_* and diff_order_book_* have the same shape).
fn parse_orderbook_inner(raw: &Value) -> WebSocketResult<OrderBook> {
    let data = raw
        .get("data")
        .ok_or_else(|| WebSocketError::Parse("orderbook: missing data".into()))?;

    let parse_levels = |arr: &Value| -> Vec<OrderBookLevel> {
        arr.as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|e| {
                        let a = e.as_array()?;
                        let price = a.first()?.as_str()?.parse::<f64>().ok()?;
                        let size = a.get(1)?.as_str()?.parse::<f64>().ok()?;
                        Some(OrderBookLevel::new(price, size))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let bids = parse_levels(data.get("bids").unwrap_or(&Value::Null));
    let asks = parse_levels(data.get("asks").unwrap_or(&Value::Null));

    let timestamp = data
        .get("microtimestamp")
        .or_else(|| data.get("timestamp"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|ts| {
            // microtimestamp is 16 digits (µs), timestamp is 10 digits (s)
            if ts > 1_000_000_000_000_000 {
                ts / 1000
            } else {
                ts * 1000
            }
        })
        .unwrap_or(0);

    Ok(OrderBook {
        bids,
        asks,
        timestamp,
        sequence: None,
        last_update_id: None,
        first_update_id: None,
        prev_update_id: None,
        event_time: None,
        transaction_time: None,
        checksum: None,
    })
}

/// Parse a JSON value that may be either a JSON number or a numeric string to f64.
///
/// Bitstamp live_orders frames send numeric fields (price, amount) as JSON numbers,
/// not strings. Older-format frames (and test fixtures) may use strings. Both are
/// accepted to avoid coupling to any single wire representation.
fn parse_f64_any(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

/// Parse a JSON value that may be either a JSON number or a numeric string to i64.
fn parse_i64_any(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

/// Parse `live_orders_*` frame → StreamEvent::OrderbookL3.
///
/// Event name drives the action field:
/// - `"order_created"` → `"create"`
/// - `"order_changed"` → `"changed"`
/// - `"order_deleted"` → `"delete"`
///
/// Bitstamp sends numeric fields (price, amount, order_type, id) as JSON numbers.
/// Both number and string representations are accepted for robustness.
pub(crate) fn parse_live_order(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw
        .get("channel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("live_orders: missing channel".into()))?;
    let symbol = channel
        .trim_start_matches("live_orders_")
        .to_ascii_uppercase();

    let event_name = raw
        .get("event")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("live_orders: missing event".into()))?;

    let action = match event_name {
        "order_created" => "create",
        "order_changed" => "changed",
        "order_deleted" => "delete",
        other => {
            return Err(WebSocketError::Parse(format!(
                "live_orders: unknown event_name {:?}",
                other
            )));
        }
    }
    .to_string();

    let data = raw
        .get("data")
        .ok_or_else(|| WebSocketError::Parse("live_orders: missing data".into()))?;

    // price: JSON number (f64) on live wire; also accept string for test fixtures.
    let price = data
        .get("price")
        .and_then(parse_f64_any)
        .ok_or_else(|| WebSocketError::Parse("live_orders: missing price".into()))?;

    // amount: JSON number on live wire; 0.0 default on deletion events.
    let quantity = data
        .get("amount")
        .and_then(parse_f64_any)
        .unwrap_or(0.0);

    // order_type: JSON number 0 = bid (Buy), 1 = ask (Sell). Also accept string.
    let side = match data.get("order_type").and_then(parse_i64_any) {
        Some(0) => OrderSide::Buy,
        Some(_) => OrderSide::Sell,
        // String fallback for test fixtures ("0" / "1").
        None => {
            if data
                .get("order_type")
                .and_then(|v| v.as_str())
                .map(|s| s == "0")
                .unwrap_or(true)
            {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            }
        }
    };

    let timestamp = data
        .get("microtimestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|us| us / 1000)
        .unwrap_or(0);

    // order_id: i64 on wire; also accept string. id_str available as fallback.
    let order_id = data
        .get("id")
        .and_then(parse_i64_any)
        .map(|n| n.to_string())
        .or_else(|| data.get("id_str").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .unwrap_or_default();

    Ok(StreamEvent::OrderbookL3 {
        symbol,
        side,
        order_id,
        price,
        quantity,
        action,
        timestamp,
    })
}

/// Parse `detail_order_book_*` frame → first StreamEvent::OrderbookL3.
///
/// `detail_order_book_*` frames carry N bid/ask entries. `ParserFn` can only
/// return one event; the transport's dispatch loop calls this once per frame.
/// For full multi-event emission, callers should use `parse_detail_ob_l3_all`.
/// This fn returns the first bid entry (or first ask if no bids) so the frame
/// is not silently dropped when dispatch_all fires this parser.
///
/// In practice the BitstampWebSocket::subscribe override calls the REST L3
/// snapshot bootstrap for `live_orders_*` (the primary L3 channel). The
/// `detail_order_book_*` channel is a secondary L3 channel that emits partial
/// snapshots. Consumers that need every entry should use `parse_detail_ob_l3_all`
/// via a custom event loop outside the registry dispatch.
pub(crate) fn parse_detail_ob_l3_first(raw: &Value) -> WebSocketResult<StreamEvent> {
    let entries = parse_detail_ob_l3_all(raw)?;
    entries.into_iter().next().ok_or_else(|| {
        WebSocketError::Parse("detail_order_book: frame had no bid/ask entries".into())
    })
}

/// Parse `detail_order_book_*` frame → Vec of StreamEvent::OrderbookL3.
///
/// Every bid/ask entry is emitted as a separate event with action "snapshot".
/// Data shape: `data.bids: [["price","amount","order_id"], ...]`
///             `data.asks: [["price","amount","order_id"], ...]`
pub(crate) fn parse_detail_ob_l3_all(raw: &Value) -> WebSocketResult<Vec<StreamEvent>> {
    let channel = raw
        .get("channel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("detail_ob: missing channel".into()))?;
    let symbol = channel
        .trim_start_matches("detail_order_book_")
        .to_ascii_uppercase();

    let data = raw
        .get("data")
        .ok_or_else(|| WebSocketError::Parse("detail_ob: missing data".into()))?;

    let timestamp = data
        .get("microtimestamp")
        .or_else(|| data.get("timestamp"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|ts| {
            if ts > 1_000_000_000_000_000 {
                ts / 1000
            } else {
                ts * 1000
            }
        })
        .unwrap_or(0);

    let parse_side = |entries: &Value, side: OrderSide| -> Vec<StreamEvent> {
        entries
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let e = entry.as_array()?;
                        let price = e.first()?.as_str()?.parse::<f64>().ok()?;
                        let quantity = e.get(1)?.as_str()?.parse::<f64>().ok()?;
                        let order_id = e.get(2)?.as_str()?.to_string();
                        Some(StreamEvent::OrderbookL3 {
                            symbol: symbol.clone(),
                            side,
                            order_id,
                            price,
                            quantity,
                            action: "snapshot".to_string(),
                            timestamp,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut events: Vec<StreamEvent> = Vec::new();
    events.extend(parse_side(
        data.get("bids").unwrap_or(&Value::Null),
        OrderSide::Buy,
    ));
    events.extend(parse_side(
        data.get("asks").unwrap_or(&Value::Null),
        OrderSide::Sell,
    ));
    Ok(events)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::websocket::WsProtocol;
    use crate::core::types::{AccountType, OwnedSymbolInput};
    use crate::core::websocket::StreamSpec;

    fn make_spec(kind: StreamKind, sym: &str) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: OwnedSymbolInput::Raw(sym.to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    // ── subscribe_frame tests ─────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_trade() {
        let proto = BitstampProtocol;
        let spec = make_spec(StreamKind::Trade, "btcusd");
        let msg = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let Message::Text(s) = msg {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["event"], "bts:subscribe");
            assert_eq!(v["data"]["channel"], "live_trades_btcusd");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_orderbook_delta() {
        let proto = BitstampProtocol;
        let spec = make_spec(StreamKind::OrderbookDelta, "btcusd");
        let msg = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let Message::Text(s) = msg {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["data"]["channel"], "diff_order_book_btcusd");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_l3() {
        let proto = BitstampProtocol;
        let spec = make_spec(StreamKind::OrderbookL3, "btcusd");
        let msg = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let Message::Text(s) = msg {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["data"]["channel"], "live_orders_btcusd");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_unsupported_kline() {
        use crate::core::websocket::KlineInterval;
        let proto = BitstampProtocol;
        let spec = make_spec(StreamKind::Kline { interval: KlineInterval::new("1m") }, "btcusd");
        let result = proto.subscribe_frame(&spec);
        assert!(
            matches!(result, Err(WebSocketError::NotSupported(_))),
            "kline must return NotSupported, got {:?}",
            result
        );
    }

    // ── extract_topic tests ───────────────────────────────────────────────────

    #[test]
    fn extract_topic_trade_frame() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({
            "event": "trade",
            "channel": "live_trades_btcusd",
            "data": {}
        });
        let key = proto.extract_topic(&raw);
        assert_eq!(key, Some(TopicKey::new("live_trades_btcusd")));
    }

    #[test]
    fn extract_topic_pong_returns_none() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({"event": "pusher:pong", "data": {}});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_subscription_succeeded_returns_none() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({
            "event": "bts:subscription_succeeded",
            "channel": "live_trades_btcusd",
            "data": ""
        });
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_connection_established_returns_none() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({
            "event": "pusher:connection_established",
            "data": "{\"socket_id\":\"123\",\"activity_timeout\":120}"
        });
        assert_eq!(proto.extract_topic(&raw), None);
    }

    // ── is_pong / is_subscribe_ack ────────────────────────────────────────────

    #[test]
    fn is_pong_true_for_pusher_pong() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({"event": "pusher:pong", "data": {}});
        assert!(proto.is_pong(&raw));
    }

    #[test]
    fn is_pong_false_for_other() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({"event": "trade", "channel": "live_trades_btcusd", "data": {}});
        assert!(!proto.is_pong(&raw));
    }

    #[test]
    fn is_subscribe_ack_true_for_bts_subscription_succeeded() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({
            "event": "bts:subscription_succeeded",
            "channel": "live_trades_btcusd",
            "data": ""
        });
        assert!(proto.is_subscribe_ack(&raw));
    }

    #[test]
    fn is_subscribe_ack_false_for_pong() {
        let proto = BitstampProtocol;
        let raw = serde_json::json!({"event": "pusher:pong", "data": {}});
        assert!(!proto.is_subscribe_ack(&raw));
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_is_pusher_ping() {
        let proto = BitstampProtocol;
        let frame = proto.ping_frame();
        assert!(frame.is_some(), "ping_frame must be Some");
        if let Some(Message::Text(s)) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["event"], "pusher:ping");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_non_empty() {
        let proto = BitstampProtocol;
        let reg = proto.topic_registry(AccountType::Spot);
        let at = AccountType::Spot;
        assert!(reg.supports(&StreamKind::Trade, at), "Trade must be supported");
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker must be supported");
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook must be supported");
        assert!(reg.supports(&StreamKind::OrderbookDelta, at), "OrderbookDelta must be supported");
        assert!(reg.supports(&StreamKind::OrderbookL3, at), "OrderbookL3 must be supported");
    }

    #[test]
    fn ticker_and_orderbook_both_registered_on_order_book_channel() {
        let proto = BitstampProtocol;
        let reg = proto.topic_registry(AccountType::Spot);
        let key = TopicKey::new("order_book_btcusd");
        let parsers = reg.dispatch_all(&key);
        assert_eq!(
            parsers.len(),
            2,
            "order_book_* channel must have 2 parsers (Ticker + Orderbook), got {}",
            parsers.len()
        );
    }

    // ── parser function tests (migrated from old websocket.rs) ────────────────

    #[test]
    fn live_order_emits_symbol_from_channel() {
        let raw = serde_json::json!({
            "channel": "live_orders_btcusd",
            "event": "order_created",
            "data": {
                "id": 42,
                "price": "50000.0",
                "amount": "0.5",
                "order_type": "0",
                "microtimestamp": "1700000000000000",
            }
        });
        let ev = parse_live_order(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookL3 { symbol, price, .. } => {
                assert_eq!(symbol, "BTCUSD");
                assert!((price - 50000.0).abs() < f64::EPSILON);
            }
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_extracts_xrpusd_from_channel() {
        let raw = serde_json::json!({
            "channel": "live_orders_xrpusd",
            "event": "order_changed",
            "data": {
                "id": 7,
                "price": "0.5",
                "amount": "1000.0",
                "order_type": "1",
                "microtimestamp": "1700000000000000",
            }
        });
        let ev = parse_live_order(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookL3 { symbol, .. } => assert_eq!(symbol, "XRPUSD"),
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_empty_channel_falls_back_to_empty_symbol() {
        let raw = serde_json::json!({
            "channel": "live_orders_",
            "event": "order_created",
            "data": {
                "id": 1,
                "price": "1.0",
                "amount": "1.0",
                "order_type": "0",
                "microtimestamp": "0",
            }
        });
        let ev = parse_live_order(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookL3 { symbol, .. } => assert_eq!(symbol, ""),
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_created_emits_orderbookl3() {
        let raw = serde_json::json!({
            "channel": "live_orders_btcusd",
            "event": "order_created",
            "data": {
                "id": 151771464,
                "price": "607.96",
                "amount": "0.54",
                "order_type": "0",
                "microtimestamp": "1474285223000000",
            }
        });
        let ev = parse_live_order(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookL3 { symbol, side, order_id, price, quantity, action, timestamp } => {
                assert_eq!(symbol, "BTCUSD");
                assert_eq!(side, OrderSide::Buy);
                assert_eq!(order_id, "151771464");
                assert!((price - 607.96).abs() < 1e-9);
                assert!((quantity - 0.54).abs() < 1e-9);
                assert_eq!(action, "create");
                assert_eq!(timestamp, 1474285223000);
            }
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_changed_emits_changed_action() {
        let raw = serde_json::json!({
            "channel": "live_orders_btcusd",
            "event": "order_changed",
            "data": {
                "id": 151771464,
                "price": "607.96",
                "amount": "0.20",
                "order_type": "0",
                "microtimestamp": "1474285224000000",
            }
        });
        let ev = parse_live_order(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookL3 { action, .. } => assert_eq!(action, "changed"),
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_deleted_emits_delete_action_zero_qty() {
        let raw = serde_json::json!({
            "channel": "live_orders_btcusd",
            "event": "order_deleted",
            "data": {
                "id": 151771464,
                "price": "607.96",
                "amount": "0",
                "order_type": "0",
                "microtimestamp": "1474285225000000",
            }
        });
        let ev = parse_live_order(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookL3 { action, quantity, .. } => {
                assert_eq!(action, "delete");
                assert!((quantity - 0.0).abs() < f64::EPSILON);
            }
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_sell_side_emits_ask() {
        let raw = serde_json::json!({
            "channel": "live_orders_btcusd",
            "event": "order_created",
            "data": {
                "id": 999,
                "price": "50100.0",
                "amount": "0.1",
                "order_type": "1",
                "microtimestamp": "1700000000000000",
            }
        });
        let ev = parse_live_order(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookL3 { side, .. } => assert_eq!(side, OrderSide::Sell),
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_unknown_event_name_returns_error() {
        let raw = serde_json::json!({
            "channel": "live_orders_btcusd",
            "event": "order_expired",
            "data": {
                "id": 1,
                "price": "1.0",
                "amount": "1.0",
                "order_type": "0",
                "microtimestamp": "0",
            }
        });
        let result = parse_live_order(&raw);
        assert!(result.is_err(), "unknown event_name must return Err");
    }

    #[test]
    fn detail_order_book_emits_all_entries() {
        let raw = serde_json::json!({
            "channel": "detail_order_book_btcusd",
            "event": "data",
            "data": {
                "bids": [
                    ["50000.0", "1.0", "id1"],
                    ["49999.0", "2.0", "id2"],
                    ["49998.0", "0.5", "id3"]
                ],
                "asks": [
                    ["50001.0", "1.5", "id4"],
                    ["50002.0", "0.3", "id5"]
                ],
                "microtimestamp": "1643643584684047"
            }
        });
        let events = parse_detail_ob_l3_all(&raw).expect("parse");
        assert_eq!(events.len(), 5, "must emit all 5 entries (3 bids + 2 asks)");
        for ev in &events {
            match ev {
                StreamEvent::OrderbookL3 { action, .. } => {
                    assert_eq!(action, "snapshot", "detail_order_book action must be 'snapshot'");
                }
                other => panic!("expected OrderbookL3, got {:?}", other),
            }
        }
        for i in 0..3 {
            if let StreamEvent::OrderbookL3 { side, .. } = &events[i] {
                assert_eq!(*side, OrderSide::Buy);
            }
        }
        for i in 3..5 {
            if let StreamEvent::OrderbookL3 { side, .. } = &events[i] {
                assert_eq!(*side, OrderSide::Sell);
            }
        }
    }

    // ── Real-wire format: numeric price/amount/order_type/id fields ───────────
    // Bitstamp live_orders frames send JSON numbers, not strings.
    // Regression guard — must not break after any parser refactor.

    #[test]
    fn live_order_numeric_fields_parse_correctly() {
        // Matches real wire format captured from wss://ws.bitstamp.net
        let raw = serde_json::json!({
            "channel": "live_orders_btcusd",
            "event": "order_created",
            "data": {
                "id": 2010114986651651_i64,
                "id_str": "2010114986651651",
                "order_type": 0,
                "microtimestamp": "1779585703820000",
                "amount": 0.125_f64,
                "amount_str": "0.12500000",
                "price": 76761.96_f64,
                "price_str": "76761.96",
            }
        });
        let ev = parse_live_order(&raw).expect("parse numeric fields");
        match ev {
            StreamEvent::OrderbookL3 { symbol, side, order_id, price, quantity, action, timestamp } => {
                assert_eq!(symbol, "BTCUSD");
                assert_eq!(side, OrderSide::Buy);
                assert_eq!(order_id, "2010114986651651");
                assert!((price - 76761.96).abs() < 1e-6, "price mismatch: {price}");
                assert!((quantity - 0.125).abs() < 1e-9, "quantity mismatch: {quantity}");
                assert_eq!(action, "create");
                // microtimestamp 1779585703820000 µs → 1779585703820 ms
                assert_eq!(timestamp, 1779585703820000_i64 / 1000);
            }
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }

    #[test]
    fn live_order_numeric_sell_side() {
        let raw = serde_json::json!({
            "channel": "live_orders_ethusd",
            "event": "order_deleted",
            "data": {
                "id": 999_i64,
                "order_type": 1,
                "microtimestamp": "1779500000000000",
                "amount": 0.5_f64,
                "price": 3200.0_f64,
            }
        });
        let ev = parse_live_order(&raw).expect("parse numeric sell");
        match ev {
            StreamEvent::OrderbookL3 { symbol, side, action, .. } => {
                assert_eq!(symbol, "ETHUSD");
                assert_eq!(side, OrderSide::Sell);
                assert_eq!(action, "delete");
            }
            other => panic!("expected OrderbookL3, got {:?}", other),
        }
    }
}
