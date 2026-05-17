//! BitgetProtocol — WsProtocol implementation for the Bitget exchange.
//!
//! Declarative shim: supplies endpoint URLs, ping frame, subscribe/unsubscribe
//! frames, topic extraction, and topic registry to UniversalWsTransport.
//!
//! Public WS only (private WS is deferred — auth_frame returns None).

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::core::traits::Credentials;
use crate::core::types::{AccountType, StreamEvent, WebSocketError, WebSocketResult};
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec,
    TopicKey, TopicRegistry,
    WsProtocol,
};
use super::parser::BitgetParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry caches
// ─────────────────────────────────────────────────────────────────────────────

static SPOT_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static FUTURES_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// BitgetProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Bitget WS protocol shim.
pub struct BitgetProtocol {
    _account_type: AccountType,
    _testnet: bool,
}

impl BitgetProtocol {
    pub fn new(account_type: AccountType, testnet: bool) -> Self {
        Self { _account_type: account_type, _testnet: testnet }
    }

    /// Map AccountType → Bitget instType string.
    fn inst_type(account_type: AccountType) -> &'static str {
        match account_type {
            AccountType::Spot | AccountType::Margin => "SPOT",
            AccountType::FuturesCross | AccountType::FuturesIsolated => "USDT-FUTURES",
            AccountType::Options => "USDC-FUTURES",
            _ => "SPOT",
        }
    }

    /// Map StreamKind → Bitget channel name string.
    /// Returns None for unsupported/unknown kinds.
    fn channel_name(kind: &StreamKind) -> Option<String> {
        let name = match kind {
            StreamKind::Ticker => "ticker".to_string(),
            StreamKind::Trade => "trade".to_string(),
            StreamKind::Orderbook => "books".to_string(),
            StreamKind::OrderbookDelta => "books15".to_string(),
            StreamKind::Kline { interval } => format!("candle{}", bitget_kline_interval(interval)),
            StreamKind::MarkPrice => "mark-price".to_string(),
            StreamKind::FundingRate => "funding-rate".to_string(),
            StreamKind::Liquidation => "liq-order".to_string(),
            StreamKind::OrderUpdate => "orders".to_string(),
            StreamKind::BalanceUpdate => "account".to_string(),
            StreamKind::PositionUpdate => "positions".to_string(),
            _ => return None,
        };
        Some(name)
    }

    /// Build subscribe/unsubscribe frame.
    fn build_frame(op: &str, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let channel = Self::channel_name(&spec.kind)
            .ok_or_else(|| WebSocketError::UnsupportedOperation(
                format!("bitget: unsupported stream kind {:?}", spec.kind),
            ))?;

        let inst_type = Self::inst_type(spec.account_type);

        // Private channels use "default" as instId
        let inst_id = if spec.kind.is_private() {
            "default".to_string()
        } else {
            spec.symbol.to_uppercase()
        };

        let frame = json!({
            "op": op,
            "args": [{
                "instType": inst_type,
                "channel": channel,
                "instId": inst_id,
            }]
        });

        Ok(Message::Text(frame.to_string()))
    }

    /// Build the spot topic registry (cached).
    fn spot_registry() -> &'static TopicRegistry {
        SPOT_REGISTRY.get_or_init(|| build_registry(AccountType::Spot))
    }

    /// Build the futures topic registry (cached).
    fn futures_registry() -> &'static TopicRegistry {
        FUTURES_REGISTRY.get_or_init(|| build_registry(AccountType::FuturesCross))
    }
}

impl WsProtocol for BitgetProtocol {
    fn name(&self) -> &'static str {
        "bitget"
    }

    fn endpoint(&self, _account_type: AccountType, testnet: bool) -> Url {
        // Bitget uses same WS URL for all product lines; instType distinguishes in channel args
        let url = if testnet {
            "wss://wspap.bitget.com/v2/ws/public"
        } else {
            "wss://ws.bitget.com/v2/ws/public"
        };
        Url::parse(url).expect("bitget ws url is valid")
    }

    fn ping_frame(&self) -> Option<Message> {
        // Bitget public WS expects literal "ping" text frame every 30s
        Some(Message::Text("ping".into()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        // Public WS only — no auth frame
        None
    }

    fn is_auth_ack(&self, _raw: &Value) -> bool {
        false
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // Bitget responds with literal JSON string or text "pong"
        raw.as_str() == Some("pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        matches!(
            raw.get("event").and_then(|v| v.as_str()),
            Some("subscribe") | Some("unsubscribe")
        )
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Pong text response — no topic
        if raw.as_str() == Some("pong") {
            return None;
        }

        // Event frames (subscribe ack, unsubscribe ack, error, login)
        if raw.get("event").is_some() {
            return None;
        }

        // Data frame format:
        // {"action":"snapshot","arg":{"instType":"SPOT","channel":"ticker","instId":"BTCUSDT"},"data":[...]}
        let channel = raw
            .get("arg")
            .and_then(|a| a.get("channel"))
            .and_then(|c| c.as_str())?;

        Some(TopicKey::new(channel))
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        match account_type {
            AccountType::Spot | AccountType::Margin | AccountType::Earn | AccountType::Lending
            | AccountType::Convert => Self::spot_registry(),
            _ => Self::futures_registry(),
        }
    }

    fn unsupported_by_exchange(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[]
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[StreamKind::OrderUpdate, StreamKind::BalanceUpdate, StreamKind::PositionUpdate]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry(account_type: AccountType) -> TopicRegistry {
    let mut b = TopicRegistry::builder();

    // Standard channels present on both spot and futures
    b = b
        .register(StreamKind::Ticker, account_type, "ticker", parse_ticker)
        .register(StreamKind::Trade, account_type, "trade", parse_trade)
        .register(StreamKind::Orderbook, account_type, "books", parse_orderbook)
        .register(StreamKind::OrderbookDelta, account_type, "books5", parse_orderbook)
        .register(StreamKind::OrderbookDelta, account_type, "books15", parse_orderbook)
        .register(StreamKind::MarkPrice, account_type, "mark-price", parse_mark_price)
        .register(StreamKind::FundingRate, account_type, "funding-rate", parse_funding_rate)
        .register(StreamKind::Liquidation, account_type, "liq-order", parse_liquidation)
        .register(StreamKind::OrderUpdate, account_type, "orders", parse_order_update)
        .register(StreamKind::BalanceUpdate, account_type, "account", parse_balance_update)
        .register(StreamKind::PositionUpdate, account_type, "positions", parse_position_update);

    // Kline channels — Bitget uses "candle<interval>" naming
    for interval in BITGET_KLINE_CHANNELS {
        let kind = StreamKind::Kline {
            interval: KlineInterval::new(internal_kline_interval(interval)),
        };
        b = b.register(kind, account_type, *interval, parse_kline);
    }

    b.build()
}

/// Bitget wire-level kline channel names.
const BITGET_KLINE_CHANNELS: &[&str] = &[
    "candle1m",
    "candle3m",
    "candle5m",
    "candle15m",
    "candle30m",
    "candle1H",
    "candle2H",
    "candle4H",
    "candle6H",
    "candle12H",
    "candle1D",
    "candle3D",
    "candle1W",
    "candle1M",
];

/// Map Bitget wire channel name → internal KlineInterval string.
fn internal_kline_interval(wire: &str) -> &'static str {
    match wire {
        "candle1m"  => "1m",
        "candle3m"  => "3m",
        "candle5m"  => "5m",
        "candle15m" => "15m",
        "candle30m" => "30m",
        "candle1H"  => "1h",
        "candle2H"  => "2h",
        "candle4H"  => "4h",
        "candle6H"  => "6h",
        "candle12H" => "12h",
        "candle1D"  => "1d",
        "candle3D"  => "3d",
        "candle1W"  => "1w",
        "candle1M"  => "1M",
        _           => "1h",
    }
}

/// Map internal KlineInterval → Bitget wire channel suffix.
fn bitget_kline_interval(interval: &KlineInterval) -> &str {
    match interval.as_str() {
        "1m"  => "1m",
        "3m"  => "3m",
        "5m"  => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h"  => "1H",
        "2h"  => "2H",
        "4h"  => "4H",
        "6h"  => "6H",
        "12h" => "12H",
        "1d"  => "1D",
        "3d"  => "3D",
        "1w"  => "1W",
        "1M"  => "1M",
        other => other,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsers (ParserFn = fn(&Value) -> WebSocketResult<StreamEvent>)
//
// Each parser receives the full frame. Bitget data frame shape:
//   {"action":"snapshot","arg":{...},"data":[...]}
// ─────────────────────────────────────────────────────────────────────────────

fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let ticker = BitgetParser::parse_ws_ticker(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Ticker(ticker))
}

fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let inst_id = raw
        .get("arg")
        .and_then(|a| a.get("instId"))
        .and_then(|v| v.as_str());
    let trade = BitgetParser::parse_ws_trade(data, inst_id)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Trade(trade))
}

fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    BitgetParser::parse_ws_orderbook_delta(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))
}

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let kline = BitgetParser::parse_ws_kline(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Kline(kline))
}

fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let inst_id = raw
        .get("arg")
        .and_then(|a| a.get("instId"))
        .and_then(|v| v.as_str());

    let item = first_item(data);
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };

    let symbol = item
        .get("symbol")
        .or_else(|| item.get("instId"))
        .and_then(|v| v.as_str())
        .or(inst_id)
        .unwrap_or("")
        .to_string();

    let mark_price = item
        .get("markPr")
        .or_else(|| item.get("markPrice"))
        .and_then(parse_f64)
        .ok_or_else(|| WebSocketError::Parse("mark-price: missing markPr".into()))?;

    let index_price = item
        .get("indexPr")
        .or_else(|| item.get("indexPrice"))
        .and_then(parse_f64);

    let timestamp = item
        .get("ts")
        .and_then(parse_f64)
        .map(|ms| ms as i64)
        .unwrap_or(0);

    Ok(StreamEvent::MarkPrice { symbol, mark_price, index_price, timestamp })
}

fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let inst_id = raw
        .get("arg")
        .and_then(|a| a.get("instId"))
        .and_then(|v| v.as_str());

    let item = first_item(data);
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };

    let symbol = item
        .get("symbol")
        .or_else(|| item.get("instId"))
        .and_then(|v| v.as_str())
        .or(inst_id)
        .unwrap_or("")
        .to_string();

    let rate = item
        .get("fundingRate")
        .and_then(parse_f64)
        .ok_or_else(|| WebSocketError::Parse("funding-rate: missing fundingRate".into()))?;

    let next_funding_time = item
        .get("fundingTime")
        .and_then(parse_f64)
        .map(|ms| ms as i64);

    let timestamp = item
        .get("ts")
        .and_then(parse_f64)
        .map(|ms| ms as i64)
        .unwrap_or(0);

    Ok(StreamEvent::FundingRate { symbol, rate, next_funding_time, timestamp })
}

fn parse_liquidation(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    let data = frame_data(raw)?;
    let inst_id = raw
        .get("arg")
        .and_then(|a| a.get("instId"))
        .and_then(|v| v.as_str());

    let item = first_item(data);
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };

    let symbol = item
        .get("instId")
        .and_then(|v| v.as_str())
        .or(inst_id)
        .unwrap_or("")
        .to_string();

    let price = item
        .get("price")
        .and_then(parse_f64)
        .ok_or_else(|| WebSocketError::Parse("liq-order: missing price".into()))?;

    let quantity = item.get("size").and_then(parse_f64).unwrap_or(0.0);

    let side = item
        .get("side")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "buy" | "Buy" => TradeSide::Buy,
            _ => TradeSide::Sell,
        })
        .unwrap_or(TradeSide::Sell);

    let timestamp = item
        .get("cTime")
        .and_then(parse_f64)
        .map(|ms| ms as i64)
        .unwrap_or(0);

    Ok(StreamEvent::Liquidation {
        symbol,
        side,
        price,
        quantity,
        value: None,
        timestamp,
    })
}

fn parse_order_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let event = BitgetParser::parse_ws_order_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderUpdate(event))
}

fn parse_balance_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let event = BitgetParser::parse_ws_balance_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::BalanceUpdate(event))
}

fn parse_position_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let event = BitgetParser::parse_ws_position_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::PositionUpdate(event))
}

// ─────────────────────────────────────────────────────────────────────────────
// Frame helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the "data" field from a Bitget data frame.
fn frame_data(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("data")
        .ok_or_else(|| WebSocketError::Parse("bitget frame missing 'data' field".into()))
}

/// Return first item if data is an array, otherwise return data itself.
fn first_item(data: &Value) -> &Value {
    if let Some(arr) = data.as_array() {
        arr.first().unwrap_or(data)
    } else {
        data
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::websocket::StreamSpec;

    fn spot_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTCUSDT".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn topic_registry_non_empty() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        let reg = proto.topic_registry(AccountType::Spot);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "spot registry must have entries");
        // Must include ticker, trade, books, funding-rate, mark-price
        assert!(reg.supports(&StreamKind::Ticker, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Orderbook, AccountType::Spot));
        assert!(reg.supports(&StreamKind::FundingRate, AccountType::Spot));
        assert!(reg.supports(&StreamKind::MarkPrice, AccountType::Spot));
    }

    #[test]
    fn subscribe_frame_spot_ticker() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        let spec = spot_spec(StreamKind::Ticker);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["op"], "subscribe");
        let arg = &v["args"][0];
        assert_eq!(arg["instType"], "SPOT");
        assert_eq!(arg["channel"], "ticker");
        assert_eq!(arg["instId"], "BTCUSDT");
    }

    #[test]
    fn extract_topic_data_frame() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "action": "snapshot",
            "arg": {
                "instType": "SPOT",
                "channel": "ticker",
                "instId": "BTCUSDT"
            },
            "data": []
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "ticker");
    }

    #[test]
    fn extract_topic_pong_returns_none() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        // Bitget pong comes as a JSON string value
        let frame = serde_json::Value::String("pong".to_string());
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn extract_topic_subscribe_ack_returns_none() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "event": "subscribe",
            "arg": { "instType": "SPOT", "channel": "ticker", "instId": "BTCUSDT" }
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn subscribe_frame_kline_1h() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        let spec = spot_spec(StreamKind::Kline {
            interval: KlineInterval::new("1h"),
        });
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["args"][0]["channel"], "candle1H");
    }

    #[test]
    fn is_subscribe_ack_detects_ack() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        let ack = serde_json::json!({"event": "subscribe", "arg": {}});
        assert!(proto.is_subscribe_ack(&ack));
        let not_ack = serde_json::json!({"action": "snapshot", "arg": {}, "data": []});
        assert!(!proto.is_subscribe_ack(&not_ack));
    }

    #[test]
    fn ping_frame_is_literal_ping() {
        let proto = BitgetProtocol::new(AccountType::Spot, false);
        match proto.ping_frame() {
            Some(Message::Text(t)) => assert_eq!(t, "ping"),
            _ => panic!("expected Some(Text('ping'))"),
        }
    }
}
