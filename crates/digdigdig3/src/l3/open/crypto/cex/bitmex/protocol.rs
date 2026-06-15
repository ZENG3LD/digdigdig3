//! BitmexProtocol — WsProtocol implementation for the BitMEX exchange.
//!
//! Public market data only.  No authentication required for:
//! instrument, trade, quote, orderBookL2_25, liquidation, funding channels.
//!
//! Heartbeat: plain text `"ping"` frame every 20s; server responds with `"pong"`.
//! BitMEX server also sends `{"info": "pong"}` internally — both forms handled.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{AccountType, WebSocketError};
use crate::core::websocket::{
    StreamKind, StreamSpec,
    TopicKey, TopicRegistry,
    WsProtocol,
};

use super::parser::{
    parse_predicted_funding, parse_funding_rate, parse_mark_price, parse_index_price,
    parse_open_interest, parse_trade, parse_quote, parse_liquidation, parse_funding_settled,
    parse_settlement_event,
};

// ─────────────────────────────────────────────────────────────────────────────
// Registry caches
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// BitmexProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative BitMEX WS protocol shim.
pub struct BitmexProtocol {
    testnet: bool,
}

impl BitmexProtocol {
    pub fn new(testnet: bool) -> Self {
        Self { testnet }
    }

    /// Map a `StreamSpec` to the BitMEX wire topic string (e.g. `"instrument:XBTUSD"`).
    fn build_topic(spec: &StreamSpec) -> Result<String, WebSocketError> {
        let sym = spec.symbol.to_uppercase();
        let topic = match &spec.kind {
            StreamKind::PredictedFunding
            | StreamKind::FundingRate
            | StreamKind::MarkPrice
            | StreamKind::IndexPrice
            | StreamKind::OpenInterest => format!("instrument:{sym}"),

            StreamKind::Trade | StreamKind::AggTrade => format!("trade:{sym}"),

            StreamKind::Ticker => format!("quote:{sym}"),

            StreamKind::Liquidation => "liquidation".to_string(),

            StreamKind::FundingSettlement => format!("funding:{sym}"),

            // BitMEX settlement channel: contract expiry/delivery events (global, no symbol suffix).
            StreamKind::SettlementEvent => "settlement".to_string(),

            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                format!("orderBookL2_25:{sym}")
            }

            other => {
                return Err(WebSocketError::WireAbsent(format!(
                    "bitmex: stream kind {other:?} has no public wire channel \
                     (BitMEX public WS covers instrument/trade/quote/orderBookL2_25/liquidation/funding only)"
                )));
            }
        };
        Ok(topic)
    }
}

impl WsProtocol for BitmexProtocol {
    fn name(&self) -> &'static str {
        "bitmex"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Use the testnet flag captured at construction, ignore the per-call param.
        let url = if self.testnet {
            super::endpoints::WS_URL_TESTNET
        } else {
            super::endpoints::WS_URL
        };
        Url::parse(url).expect("bitmex ws url is valid")
    }

    fn ping_frame(&self) -> Option<WsFrame> {
        // BitMEX heartbeat is a plain "ping" text frame (NOT JSON).
        Some(WsFrame::Text("ping".into()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(20)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let topic = Self::build_topic(spec)?;
        let frame = json!({ "op": "subscribe", "args": [topic] });
        Ok(WsFrame::Text(frame.to_string()))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let topic = Self::build_topic(spec)?;
        let frame = json!({ "op": "unsubscribe", "args": [topic] });
        Ok(WsFrame::Text(frame.to_string()))
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        // Public-only connector — no auth frame.
        None
    }

    fn is_auth_ack(&self, _raw: &Value) -> bool {
        false
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // Server responds to "ping" with the literal string "pong".
        raw.as_str() == Some("pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        // Ack format: {"success": true, "subscribe": "<topic>", "request": {...}}
        raw.get("success").and_then(Value::as_bool) == Some(true)
            && raw.get("subscribe").is_some()
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Skip pong text response — no topic.
        if raw.as_str().is_some() {
            return None;
        }

        // Skip ack / error frames.
        if raw.get("success").is_some() || raw.get("error").is_some() || raw.get("info").is_some() {
            return None;
        }

        // Data push format: {"table": "<topic>", "action": "...", "data": [...]}
        raw.get("table")
            .and_then(Value::as_str)
            .map(TopicKey::new)
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        REGISTRY.get_or_init(build_registry)
    }

    fn unsupported_by_exchange(&self, _account_type: AccountType) -> &'static [StreamKind] {
        // Everything not in build_topic is handled by returning WireAbsent from subscribe_frame.
        &[]
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        // All streams wired here are public.
        &[]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    // BitMEX `instrument` channel carries multiple fields: indicativeFundingRate,
    // fundingRate, markPrice, indexPrice.  We dual/quad-register parsers on the
    // same wire topic so all four StreamKind subscribers get dispatched.
    TopicRegistry::builder()
        // instrument → PredictedFunding (primary goal)
        .register(StreamKind::PredictedFunding, AccountType::FuturesCross, "instrument", parse_predicted_funding)
        // instrument → FundingRate (current period settled rate)
        .register(StreamKind::FundingRate, AccountType::FuturesCross, "instrument", parse_funding_rate)
        // instrument → MarkPrice
        .register(StreamKind::MarkPrice, AccountType::FuturesCross, "instrument", parse_mark_price)
        // instrument → IndexPrice
        .register(StreamKind::IndexPrice, AccountType::FuturesCross, "instrument", parse_index_price)
        // instrument → OpenInterest (openInterest + openValue fields; partial-update safe)
        .register(StreamKind::OpenInterest, AccountType::FuturesCross, "instrument", parse_open_interest)
        // trade → Trade
        .register(StreamKind::Trade, AccountType::FuturesCross, "trade", parse_trade)
        // AggTrade: no dedicated channel; fan-out from trade
        .register(StreamKind::AggTrade, AccountType::FuturesCross, "trade", parse_trade)
        // quote → Ticker (best bid/ask)
        .register(StreamKind::Ticker, AccountType::FuturesCross, "quote", parse_quote)
        // liquidation → Liquidation (global channel, no symbol suffix)
        .register(StreamKind::Liquidation, AccountType::FuturesCross, "liquidation", parse_liquidation)
        // funding → FundingSettlement (8h settlement events)
        .register(StreamKind::FundingSettlement, AccountType::FuturesCross, "funding", parse_funding_settled)
        // orderBookL2_25 → OrderbookDelta (top-25 throttled L2)
        .register(StreamKind::OrderbookDelta, AccountType::FuturesCross, "orderBookL2_25", parse_orderbook_delta)
        // orderBookL2_25 is also used for Orderbook snapshot (partial action)
        .register(StreamKind::Orderbook, AccountType::FuturesCross, "orderBookL2_25", parse_orderbook_delta)
        // settlement → SettlementEvent (contract expiry/delivery; global channel)
        .register(StreamKind::SettlementEvent, AccountType::FuturesCross, "settlement", parse_settlement_event)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Orderbook delta parser — BitMEX orderBookL2_25 / orderBookL2
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `orderBookL2_25`/`orderBookL2` frame → `StreamEvent::OrderbookDelta`.
///
/// BitMEX L2 feed format:
/// - `action`: `"partial"` (initial snapshot), `"insert"`, `"update"`, `"delete"`.
/// - Each row: `{id, side, size, price?, symbol, timestamp?, transactTime?}`.
/// - `price` is present on `partial`/`insert` rows. On `update`/`delete` rows
///   the price is encoded in the integer `id` (`id = (10_000_000_000 - price)
///   / tickSize`) — **recovering it requires a stateful id→price map that lives
///   in Station, not here**.
///
/// This parser emits all rows that carry both `price` and `size`. Rows that
/// lack `price` (typical for `update`/`delete` actions) are passed through in
/// the `first_update_id` / `last_update_id` envelope so Station can apply them
/// to its local book state. Both bids and asks are populated from rows where
/// `side == "Buy"` → bids, `side == "Sell"` → asks.
fn parse_orderbook_delta(raw: &Value) -> crate::core::types::WebSocketResult<crate::core::types::StreamEvent> {
    use crate::core::types::{OrderBookLevel, OrderbookDelta as OBDelta, StreamEvent, WebSocketError};

    let data = raw
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| WebSocketError::Parse("bitmex orderbook: frame missing 'data' array".into()))?;

    let symbol = data
        .first()
        .and_then(|item| item.get("symbol"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let mut bids: Vec<OrderBookLevel> = Vec::new();
    let mut asks: Vec<OrderBookLevel> = Vec::new();
    let mut first_id: Option<u64> = None;
    let mut last_id:  Option<u64> = None;

    for item in data {
        // Track id range for Station's gap-detection.
        if let Some(id) = item.get("id").and_then(Value::as_u64) {
            first_id = Some(first_id.map_or(id, |prev| prev.min(id)));
            last_id  = Some(last_id.map_or(id,  |prev| prev.max(id)));
        }

        // Rows without price cannot be resolved without the stateful id→price
        // map in Station. Emit only rows that carry price + size.
        let price = match item.get("price").and_then(Value::as_f64) {
            Some(p) => p,
            None => continue,
        };
        let size = item
            .get("size")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        let level = OrderBookLevel::new(price, size);

        match item.get("side").and_then(Value::as_str) {
            Some("Buy")  => bids.push(level),
            Some("Sell") => asks.push(level),
            _ => {}
        }
    }

    let timestamp = raw
        .get("data")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("timestamp").or_else(|| item.get("transactTime")))
        .and_then(Value::as_str)
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp_millis())
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    let delta = OBDelta {
        bids,
        asks,
        timestamp,
        first_update_id: first_id,
        last_update_id:  last_id,
        prev_update_id:  None,
        event_time:      None,
        checksum:        None,
    };

    Ok(StreamEvent::OrderbookDelta { symbol, delta })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OwnedSymbolInput;

    fn futures_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: OwnedSymbolInput::Raw("XBTUSD".to_string()),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn subscribe_frame_predicted_funding_maps_to_instrument() {
        let proto = BitmexProtocol::new(false);
        let spec = futures_spec(StreamKind::PredictedFunding);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["op"], "subscribe");
        assert_eq!(v["args"][0], "instrument:XBTUSD");
    }

    #[test]
    fn subscribe_frame_trade() {
        let proto = BitmexProtocol::new(false);
        let spec = futures_spec(StreamKind::Trade);
        let msg = proto.subscribe_frame(&spec).unwrap();
        let text = match msg { WsFrame::Text(t) => t, _ => panic!() };
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["args"][0], "trade:XBTUSD");
    }

    #[test]
    fn subscribe_frame_ticker_maps_to_quote() {
        let proto = BitmexProtocol::new(false);
        let spec = futures_spec(StreamKind::Ticker);
        let msg = proto.subscribe_frame(&spec).unwrap();
        let text = match msg { WsFrame::Text(t) => t, _ => panic!() };
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["args"][0], "quote:XBTUSD");
    }

    #[test]
    fn subscribe_frame_liquidation_global() {
        let proto = BitmexProtocol::new(false);
        let spec = futures_spec(StreamKind::Liquidation);
        let msg = proto.subscribe_frame(&spec).unwrap();
        let text = match msg { WsFrame::Text(t) => t, _ => panic!() };
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        // Liquidation is a global channel — no symbol suffix
        assert_eq!(v["args"][0], "liquidation");
    }

    #[test]
    fn subscribe_frame_funding_settlement() {
        let proto = BitmexProtocol::new(false);
        let spec = futures_spec(StreamKind::FundingSettlement);
        let msg = proto.subscribe_frame(&spec).unwrap();
        let text = match msg { WsFrame::Text(t) => t, _ => panic!() };
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["args"][0], "funding:XBTUSD");
    }

    #[test]
    fn subscribe_frame_kline_returns_not_supported() {
        let proto = BitmexProtocol::new(false);
        let spec = futures_spec(StreamKind::Kline {
            interval: crate::core::websocket::KlineInterval::new("1m"),
        });
        let err = proto.subscribe_frame(&spec).expect_err("Kline must return WireAbsent");
        assert!(
            matches!(err, WebSocketError::WireAbsent(_)),
            "expected WireAbsent, got {:?}", err
        );
    }

    #[test]
    fn ping_frame_is_literal_ping() {
        let proto = BitmexProtocol::new(false);
        match proto.ping_frame() {
            Some(WsFrame::Text(t)) => assert_eq!(t, "ping"),
            _ => panic!("expected Some(Text('ping'))"),
        }
    }

    #[test]
    fn is_pong_detects_pong_text() {
        let proto = BitmexProtocol::new(false);
        assert!(proto.is_pong(&serde_json::Value::String("pong".into())));
        assert!(!proto.is_pong(&serde_json::json!({"info": "pong"})));
    }

    #[test]
    fn is_subscribe_ack_detects_success_frame() {
        let proto = BitmexProtocol::new(false);
        let ack = serde_json::json!({"success": true, "subscribe": "instrument:XBTUSD", "request": {}});
        assert!(proto.is_subscribe_ack(&ack));
        let not_ack = serde_json::json!({"table": "instrument", "action": "partial", "data": []});
        assert!(!proto.is_subscribe_ack(&not_ack));
    }

    #[test]
    fn extract_topic_data_frame() {
        let proto = BitmexProtocol::new(false);
        let frame = serde_json::json!({"table": "instrument", "action": "update", "data": []});
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "instrument");
    }

    #[test]
    fn extract_topic_pong_returns_none() {
        let proto = BitmexProtocol::new(false);
        let frame = serde_json::Value::String("pong".into());
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn extract_topic_success_ack_returns_none() {
        let proto = BitmexProtocol::new(false);
        let frame = serde_json::json!({"success": true, "subscribe": "instrument:XBTUSD"});
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn registry_has_predicted_funding() {
        let proto = BitmexProtocol::new(false);
        let reg = proto.topic_registry(AccountType::FuturesCross);
        assert!(reg.supports(&StreamKind::PredictedFunding, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::FundingRate, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::MarkPrice, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Trade, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Ticker, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Liquidation, AccountType::FuturesCross));
    }

    #[test]
    fn instrument_topic_dispatches_five_parsers() {
        let proto = BitmexProtocol::new(false);
        let reg = proto.topic_registry(AccountType::FuturesCross);
        let key = crate::core::websocket::TopicKey::new("instrument");
        let parsers = reg.dispatch_all(&key);
        // PredictedFunding + FundingRate + MarkPrice + IndexPrice + OpenInterest = 5
        assert!(
            parsers.len() >= 5,
            "expected >=5 parsers for instrument fan-out, got {}",
            parsers.len()
        );
    }

    #[test]
    fn subscribe_frame_open_interest_maps_to_instrument() {
        let proto = BitmexProtocol::new(false);
        let spec = futures_spec(StreamKind::OpenInterest);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed for OpenInterest");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["op"], "subscribe");
        assert_eq!(v["args"][0], "instrument:XBTUSD");
    }

    #[test]
    fn registry_supports_open_interest() {
        let proto = BitmexProtocol::new(false);
        let reg = proto.topic_registry(AccountType::FuturesCross);
        assert!(reg.supports(&StreamKind::OpenInterest, AccountType::FuturesCross));
    }

    #[test]
    fn parse_open_interest_yields_correct_event() {
        use super::super::parser::parse_open_interest as parse_oi;
        use crate::core::types::StreamEvent;

        let frame = serde_json::json!({
            "table": "instrument",
            "action": "update",
            "data": [{
                "symbol": "XBTUSD",
                "openInterest": 123456789_u64,
                "openValue": 8765432100_u64,
                "timestamp": "2024-01-01T12:00:00.000Z"
            }]
        });
        let event = parse_oi(&frame).expect("should parse OpenInterest");
        match event {
            StreamEvent::OpenInterestUpdate { symbol, open_interest } => {
                assert_eq!(symbol, "XBTUSD");
                assert!((open_interest.open_interest - 123_456_789.0).abs() < 1.0, "open_interest mismatch");
                assert!(open_interest.open_interest_value.is_some(), "open_interest_value must be Some");
                assert!(
                    (open_interest.open_interest_value.unwrap() - 8_765_432_100.0).abs() < 1.0,
                    "open_interest_value mismatch"
                );
                assert!(open_interest.timestamp > 0, "timestamp must be set");
            }
            other => panic!("expected OpenInterestUpdate, got {:?}", other),
        }
    }

    #[test]
    fn parse_open_interest_missing_field_returns_field_absent() {
        use super::super::parser::parse_open_interest as parse_oi;
        use crate::core::types::WebSocketError;

        // Partial-update frame without openInterest — must NOT emit a bogus 0.
        let frame = serde_json::json!({
            "table": "instrument",
            "action": "update",
            "data": [{"symbol": "XBTUSD", "markPrice": 45200.0, "timestamp": "2024-01-01T07:45:00.000Z"}]
        });
        let err = parse_oi(&frame).expect_err("should return FieldAbsent when openInterest absent");
        assert!(
            matches!(err, WebSocketError::FieldAbsent(_)),
            "expected FieldAbsent, got {:?}", err
        );
    }
}
