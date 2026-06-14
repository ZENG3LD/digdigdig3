//! BingxProtocol — WsProtocol implementation for BingX WebSocket.
//!
//! ## BingX WebSocket protocol
//!
//! - Endpoint: `wss://open-api-swap.bingx.com/swap-market` (public swap/perp market data)
//! - Subscribe frame format:
//!   `{"id":"<uuid>","reqType":"sub","dataType":"<topic>"}`
//!   Where `<topic>` is e.g. `"BTCUSDT@bookTicker"`, `"BTC-USDT@depth5"`.
//!
//! ## Binary frames (GZIP)
//!
//! All data frames from BingX are GZIP-compressed binary. The default
//! `decode_binary` implementation in `WsProtocol` already tries gzip → zlib →
//! deflate → UTF-8, so no override is needed. The transport decompresses and
//! JSON-parses transparently.
//!
//! ## Server-initiated ping
//!
//! BingX sends pings in two forms:
//! - `{"ping":"<id>","time":"..."}` as a (gzip-compressed) JSON object.
//!   Client must reply with `{"pong":"<id>","time":"..."}`.
//! - Plain `"Ping"` string (gzip-compressed). After decompression the deserialized
//!   JSON `Value` is a `Value::String("Ping")`. Client must reply `"Pong"`.
//!
//! Both shapes are handled by `is_server_ping` / `pong_response_frame`.
//!
//! ## Client-initiated pings
//!
//! BingX does not require client-initiated pings. `ping_frame()` returns `None`.
//!
//! ## Subscribe ack
//!
//! `{"id":"<uuid>","code":0,"msg":""}` — present `code == 0` with no `dataType`.
//!
//! ## Supported channels
//!
//! | Stream kind    | dataType topic suffix    | Notes                              |
//! |---------------|--------------------------|-------------------------------------|
//! | Ticker        | `@bookTicker`            | Best bid/ask quotes                |
//! | Trade         | `@trade`                 |                                    |
//! | Orderbook     | `@depth5` / `@depth10` / `@depth20` |                         |
//! | Kline         | `@kline_<tf>`            | e.g. `@kline_1m`, `@kline_1h`     |
//! | MarkPrice     | `@markPrice`             |                                    |
//! | FundingRate   | N/A — not supported      | BingX swap WS returns code 80015 "dataType not support" for `@fundingRate` |
//! | Liquidation   | N/A — not supported      | BingX swap WS returns code 80015 for `@forceOrder` |
//! | OpenInterest  | N/A — not supported      | BingX swap WS returns code 80015 for `@openInterest` |
//! | AggTrade      | N/A — not supported      | BingX swap WS returns code 80015 for `@aggTrade` |

use std::sync::OnceLock;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, StreamEvent, WebSocketError, WebSocketResult,
};
use crate::core::utils::timestamp_millis;
use crate::core::websocket::{KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};

use super::endpoints::{format_symbol, map_kline_interval};
use super::parser::BingxParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache — one shared registry (BingX uses a single public WS endpoint)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// BingxProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative BingX WS protocol shim.
///
/// Public market-data channels (swap/perpetual endpoint):
/// ticker, trade, orderbook depth, kline, mark price.
///
/// FundingRate, Liquidation, OpenInterest, AggTrade are NOT available on
/// the public swap-market WS endpoint — BingX returns code 80015
/// ("dataType not support") for `@fundingRate`, `@forceOrder`,
/// `@openInterest`, and `@aggTrade`. Verified live 2026-05-29.
pub struct BingxProtocol;

impl BingxProtocol {
    pub fn new(_testnet: bool) -> Self {
        Self
    }

    /// Build the `dataType` topic string for a StreamSpec.
    ///
    /// Returns `Err(NotSupported)` for channels the BingX swap-market endpoint
    /// rejects with code 80015 ("dataType not support") — they are wire-absent.
    fn build_data_type(spec: &StreamSpec) -> Result<String, WebSocketError> {
        let sym = wire_symbol(spec);
        match &spec.kind {
            StreamKind::Ticker => Ok(format!("{}@bookTicker", sym)),
            StreamKind::Trade => Ok(format!("{}@trade", sym)),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                let depth = spec.depth.unwrap_or(5);
                Ok(format!("{}@depth{}", sym, depth))
            }
            StreamKind::Kline { interval } => {
                let tf = map_kline_interval(interval.as_str());
                Ok(format!("{}@kline_{}", sym, tf))
            }
            StreamKind::MarkPrice => Ok(format!("{}@markPrice", sym)),
            // Wire-not-present: BingX swap-market WS rejects these with
            // code 80015 "dataType not support". Verified live 2026-05-29 via
            // raw tungstenite probe — the channels do not exist on the public
            // endpoint, so this is NotSupported (not a TODO).
            StreamKind::FundingRate => Err(WebSocketError::NotSupported(
                "BingX swap WS has no @fundingRate channel (server: code 80015 dataType not support) — use REST".into(),
            )),
            StreamKind::Liquidation => Err(WebSocketError::NotSupported(
                "BingX swap WS has no @forceOrder channel (server: code 80015 dataType not support)".into(),
            )),
            StreamKind::OpenInterest => Err(WebSocketError::NotSupported(
                "BingX swap WS has no @openInterest channel (server: code 80015 dataType not support) — use REST".into(),
            )),
            StreamKind::AggTrade => Err(WebSocketError::NotSupported(
                "BingX swap WS has no @aggTrade channel (server: code 80015 dataType not support)".into(),
            )),
            other => Err(WebSocketError::NotSupported(format!(
                "BingX swap-market WS has no public channel for {:?}",
                other
            ))),
        }
    }
}

/// Resolve the BingX wire symbol from a StreamSpec.
///
/// BingX uses `BTC-USDT` hyphenated format for both spot and futures.
/// Accepts:
/// - Raw `"BTC-USDT"` or `"BTCUSDT"` passed directly.
/// - Slash-separated `"BTC/USDT"` — converted via `format_symbol`.
/// - Base/quote via OwnedSymbolInput::Raw display.
fn wire_symbol(spec: &StreamSpec) -> String {
    let raw = spec.symbol.to_string();
    if raw.contains('/') {
        let mut parts = raw.splitn(2, '/');
        let base = parts.next().unwrap_or("");
        let quote = parts.next().unwrap_or("USDT");
        return format_symbol(base, quote, spec.account_type);
    }
    // Already exchange-native format (e.g. "BTC-USDT" or "BTCUSDT@...")
    raw
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for BingxProtocol {
    fn name(&self) -> &'static str {
        "bingx"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // BingX has no dedicated testnet. Same endpoint for all account types.
        Url::parse("wss://open-api-swap.bingx.com/swap-market")
            .expect("bingx ws endpoint is valid")
    }

    /// BingX manages keepalive via server-initiated pings.
    /// Client does NOT send application-level pings.
    fn ping_frame(&self) -> Option<WsFrame> {
        None
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let data_type = Self::build_data_type(spec)?;
        let id = format!("{:x}", timestamp_millis());
        let frame = json!({
            "id": id,
            "reqType": "sub",
            "dataType": data_type,
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let data_type = Self::build_data_type(spec)?;
        let id = format!("{:x}", timestamp_millis());
        let frame = json!({
            "id": id,
            "reqType": "unsub",
            "dataType": data_type,
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    /// Public channels require no authentication.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    /// BingX does not send application-level pong frames — no client-initiated pings.
    fn is_pong(&self, _raw: &Value) -> bool {
        false
    }

    /// Subscribe ack: `{"id":"...","code":0,"msg":""}` (no `dataType` field).
    ///
    /// Data frames also carry `code` but always include `dataType` — the absence
    /// of `dataType` is the discriminator.
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        let code_ok = raw.get("code").and_then(|v| v.as_i64()) == Some(0);
        let no_data_type = raw.get("dataType").is_none();
        // Must have the id field to confirm it's a protocol ack frame.
        let has_id = raw.get("id").is_some();
        code_ok && no_data_type && has_id
    }

    /// BingX sends server-initiated pings in two forms:
    /// 1. `{"ping":"<id>","time":"..."}` — JSON object with a `ping` field.
    /// 2. Gzip-compressed `"Ping"` string — after decompression the Value is
    ///    `Value::String("Ping")`.
    fn is_server_ping(&self, raw: &Value) -> bool {
        // Form 1: {"ping": "<id>", ...}
        if raw.get("ping").is_some() {
            return true;
        }
        // Form 2: plain "Ping" string (after gzip decompress + JSON parse)
        if raw.as_str() == Some("Ping") {
            return true;
        }
        false
    }

    /// Build the pong reply matching the ping form.
    ///
    /// - For `{"ping":"<id>","time":"..."}` → `{"pong":"<id>","time":"<now>"}`
    /// - For `"Ping"` string → `WsFrame::Text("Pong")`
    fn pong_response_frame(&self, raw: &Value) -> Option<WsFrame> {
        // Form 1: JSON ping object with an id value
        if let Some(id) = raw.get("ping") {
            let reply = json!({
                "pong": id,
                "time": timestamp_millis().to_string(),
            });
            return Some(WsFrame::Text(reply.to_string()));
        }
        // Form 2: plain "Ping" string
        if raw.as_str() == Some("Ping") {
            return Some(WsFrame::Text("Pong".to_string()));
        }
        None
    }

    /// Extract the routing topic from an incoming BingX data frame.
    ///
    /// BingX data frames carry `"dataType"` as the routing key:
    /// `{"dataType":"BTCUSDT@bookTicker","code":0,"data":{...}}`.
    ///
    /// Returns `None` for ack frames (no `dataType`) and ping frames.
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let data_type = raw.get("dataType").and_then(|v| v.as_str())?;
        if data_type.is_empty() {
            return None;
        }
        Some(TopicKey::new(data_type))
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        REGISTRY.get_or_init(build_registry)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
//
// BingX dataType values are per-symbol (e.g. "BTCUSDT@bookTicker").
// We use wildcard patterns ("*@bookTicker", "*@trade", etc.) so a single
// parser registration covers all subscribed symbols.
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    let at = AccountType::Spot;
    TopicRegistry::builder()
        .register(StreamKind::Ticker, at, "*@bookTicker", parse_book_ticker)
        .register(StreamKind::Trade, at, "*@trade", parse_trade)
        .register(StreamKind::Orderbook, at, "*@depth5", parse_orderbook)
        .register(StreamKind::Orderbook, at, "*@depth10", parse_orderbook)
        .register(StreamKind::Orderbook, at, "*@depth20", parse_orderbook)
        .register(StreamKind::Orderbook, at, "*@depth50", parse_orderbook)
        .register(StreamKind::Orderbook, at, "*@depth100", parse_orderbook)
        .register(
            StreamKind::Kline { interval: KlineInterval::new("") },
            at,
            "*@kline_*",
            parse_kline,
        )
        .register(StreamKind::MarkPrice, at, "*@markPrice", parse_mark_price)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions
//
// Each parser receives the TOP-LEVEL raw frame from BingX:
// { "dataType": "BTCUSDT@bookTicker", "code": 0, "data": { ... } }
//
// Symbol is extracted from `dataType` (before the '@').
// ─────────────────────────────────────────────────────────────────────────────

/// Extract `(symbol, data)` from a BingX push frame.
///
/// `data_type` is the full `dataType` string (e.g. `"BTCUSDT@bookTicker"`).
/// Symbol is the part before `'@'`.
fn extract_payload(raw: &Value) -> Option<(String, &Value)> {
    let data_type = raw.get("dataType").and_then(|v| v.as_str())?;
    let symbol = data_type.split('@').next().unwrap_or("").to_string();
    let data = raw.get("data")?;
    Some((symbol, data))
}

/// Parse `*@bookTicker` → StreamEvent::Ticker.
pub(crate) fn parse_book_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data_type = raw
        .get("dataType")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let (symbol, data) = extract_payload(raw).ok_or_else(|| {
        WebSocketError::Parse("bingx bookTicker: missing dataType or data".into())
    })?;
    let ticker = BingxParser::parse_ws_book_ticker(data_type, data)
        .map_err(|e| WebSocketError::Parse(format!("bingx bookTicker: {}", e)))?;
    Ok(StreamEvent::Ticker { symbol, ticker })
}

/// Parse `*@trade` → StreamEvent::Trade.
pub(crate) fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (_, data) = extract_payload(raw).ok_or_else(|| {
        WebSocketError::Parse("bingx trade: missing dataType or data".into())
    })?;
    // BingX trade frame has symbol in data["s"]
    let symbol = data
        .get("s")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let trade = BingxParser::parse_ws_trade(data)
        .map_err(|e| WebSocketError::Parse(format!("bingx trade: {}", e)))?;
    Ok(StreamEvent::Trade { symbol, trade })
}

/// Parse `*@depth5` / `*@depth10` / `*@depth20` → StreamEvent::OrderbookDelta.
pub(crate) fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (symbol, data) = extract_payload(raw).ok_or_else(|| {
        WebSocketError::Parse("bingx depth: missing dataType or data".into())
    })?;
    let delta = BingxParser::parse_ws_orderbook(data)
        .map_err(|e| WebSocketError::Parse(format!("bingx depth: {}", e)))?;
    Ok(StreamEvent::OrderbookDelta { symbol, delta })
}

/// Parse `*@kline_<tf>` → StreamEvent::Kline.
pub(crate) fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data_type = raw
        .get("dataType")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // Extract interval from dataType: "BTCUSDT@kline_1m" → "1m"
    let interval_str = data_type
        .split("@kline_")
        .nth(1)
        .unwrap_or("");
    let interval = KlineInterval::new(interval_str);
    let (symbol, data) = extract_payload(raw).ok_or_else(|| {
        WebSocketError::Parse("bingx kline: missing dataType or data".into())
    })?;
    let kline = BingxParser::parse_ws_kline(data)
        .map_err(|e| WebSocketError::Parse(format!("bingx kline: {}", e)))?;
    Ok(StreamEvent::Kline { symbol, interval, kline })
}

/// Parse `*@markPrice` → StreamEvent::MarkPrice.
pub(crate) fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (symbol, data) = extract_payload(raw).ok_or_else(|| {
        WebSocketError::Parse("bingx markPrice: missing dataType or data".into())
    })?;
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };
    let mark_price = data
        .get("markPrice")
        .and_then(|v| parse_f64(v))
        .ok_or_else(|| WebSocketError::Parse("bingx markPrice: missing markPrice field".into()))?;
    let index_price = data.get("indexPrice").and_then(|v| parse_f64(v));
    let timestamp = data
        .get("ts")
        .or_else(|| data.get("time"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    Ok(StreamEvent::MarkPrice {
        symbol,
        mark: crate::core::types::MarkPrice {
            mark_price,
            index_price,
            timestamp,
            ..Default::default()
        },
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AccountType, OwnedSymbolInput};
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

    fn proto() -> BingxProtocol {
        BingxProtocol::new(false)
    }

    // ── endpoint ──────────────────────────────────────────────────────────────

    #[test]
    fn endpoint_returns_swap_market_url() {
        let url = proto().endpoint(AccountType::Spot, false);
        assert_eq!(url.as_str(), "wss://open-api-swap.bingx.com/swap-market");
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_returns_none() {
        assert!(proto().ping_frame().is_none());
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_ticker() {
        let spec = make_spec(StreamKind::Ticker, "BTC-USDT");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["reqType"], "sub");
            assert_eq!(v["dataType"], "BTC-USDT@bookTicker");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_orderbook_default_depth_5() {
        let spec = make_spec(StreamKind::Orderbook, "BTC-USDT");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["dataType"], "BTC-USDT@depth5");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_kline() {
        let spec = StreamSpec {
            kind: StreamKind::Kline { interval: KlineInterval::new("1m") },
            symbol: OwnedSymbolInput::Raw("BTC-USDT".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        };
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["dataType"], "BTC-USDT@kline_1m");
        } else {
            panic!("expected Text frame");
        }
    }

    // BingX swap WS returns code 80015 "dataType not support" for the following channels.
    // subscribe_frame correctly returns UnsupportedOperation so callers can handle gracefully.

    // BingX swap WS rejects these with code 80015 (wire-absent) — verified live.
    #[test]
    fn subscribe_frame_funding_rate_not_supported() {
        let spec = make_spec(StreamKind::FundingRate, "BTC-USDT");
        assert!(matches!(
            proto().subscribe_frame(&spec),
            Err(WebSocketError::NotSupported(_))
        ));
    }

    #[test]
    fn subscribe_frame_liquidation_not_supported() {
        let spec = make_spec(StreamKind::Liquidation, "BTC-USDT");
        assert!(matches!(
            proto().subscribe_frame(&spec),
            Err(WebSocketError::NotSupported(_))
        ));
    }

    #[test]
    fn subscribe_frame_open_interest_not_supported() {
        let spec = make_spec(StreamKind::OpenInterest, "BTC-USDT");
        assert!(matches!(
            proto().subscribe_frame(&spec),
            Err(WebSocketError::NotSupported(_))
        ));
    }

    #[test]
    fn subscribe_frame_agg_trade_not_supported() {
        let spec = make_spec(StreamKind::AggTrade, "BTC-USDT");
        assert!(matches!(
            proto().subscribe_frame(&spec),
            Err(WebSocketError::NotSupported(_))
        ));
    }


    // ── is_server_ping ────────────────────────────────────────────────────────

    #[test]
    fn is_server_ping_json_ping_object() {
        let raw = serde_json::json!({"ping": "some-id", "time": "1234567890"});
        assert!(proto().is_server_ping(&raw));
    }

    #[test]
    fn is_server_ping_plain_ping_string() {
        let raw = Value::String("Ping".to_string());
        assert!(proto().is_server_ping(&raw));
    }

    #[test]
    fn is_server_ping_false_for_data_frame() {
        let raw = serde_json::json!({"dataType": "BTCUSDT@bookTicker", "code": 0, "data": {}});
        assert!(!proto().is_server_ping(&raw));
    }

    // ── pong_response_frame ───────────────────────────────────────────────────

    #[test]
    fn pong_response_for_json_ping_echoes_id() {
        let raw = serde_json::json!({"ping": "some-id", "time": "1234567890"});
        let reply = proto().pong_response_frame(&raw).expect("pong frame");
        if let WsFrame::Text(s) = reply {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["pong"], "some-id");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn pong_response_for_plain_ping_is_pong_text() {
        let raw = Value::String("Ping".to_string());
        let reply = proto().pong_response_frame(&raw).expect("pong frame");
        assert_eq!(reply, WsFrame::Text("Pong".to_string()));
    }

    // ── is_subscribe_ack ──────────────────────────────────────────────────────

    #[test]
    fn is_subscribe_ack_for_code_zero_no_data_type() {
        let raw = serde_json::json!({"id": "abc", "code": 0, "msg": ""});
        assert!(proto().is_subscribe_ack(&raw));
    }

    #[test]
    fn is_subscribe_ack_false_for_data_frame() {
        let raw = serde_json::json!({
            "dataType": "BTCUSDT@bookTicker",
            "code": 0,
            "data": {}
        });
        assert!(!proto().is_subscribe_ack(&raw));
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn extract_topic_returns_data_type() {
        let raw = serde_json::json!({"dataType": "BTCUSDT@bookTicker", "code": 0, "data": {}});
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("BTCUSDT@bookTicker"))
        );
    }

    #[test]
    fn extract_topic_none_for_ack() {
        let raw = serde_json::json!({"id": "abc", "code": 0, "msg": ""});
        assert_eq!(proto().extract_topic(&raw), None);
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_covers_supported_channels() {
        let p = proto();
        let reg = p.topic_registry(AccountType::Spot);
        let at = AccountType::Spot;
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker");
        assert!(reg.supports(&StreamKind::Trade, at), "Trade");
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook");
        assert!(
            reg.supports(&StreamKind::Kline { interval: KlineInterval::new("") }, at),
            "Kline"
        );
        assert!(reg.supports(&StreamKind::MarkPrice, at), "MarkPrice");
        // FundingRate, Liquidation, OpenInterest, AggTrade: NOT registered — BingX
        // swap WS returns code 80015 for these channels (verified live 2026-05-29).
    }

    #[test]
    fn topic_registry_wildcard_matches_per_symbol_keys() {
        let p = proto();
        let reg = p.topic_registry(AccountType::Spot);
        assert!(
            reg.dispatch(&TopicKey::new("BTCUSDT@bookTicker")).is_some(),
            "BTCUSDT@bookTicker"
        );
        assert!(
            reg.dispatch(&TopicKey::new("BTC-USDT@depth5")).is_some(),
            "BTC-USDT@depth5"
        );
        assert!(
            reg.dispatch(&TopicKey::new("ETH-USDT@kline_1m")).is_some(),
            "ETH-USDT@kline_1m"
        );
        assert!(
            reg.dispatch(&TopicKey::new("BTCUSDT@markPrice")).is_some(),
            "BTCUSDT@markPrice"
        );
    }
}
