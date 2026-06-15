//! KrakenProtocol — WsProtocol impl for Kraken WebSocket v2.
//!
//! Endpoint:      wss://ws.kraken.com/v2
//! Symbol format: BTC/USD (slash-separated, v2 uses BTC not XBT)
//! Subscribe:     {"method":"subscribe","params":{"channel":"ticker","symbol":["BTC/USD"]}}
//! Ping (JSON):   {"method":"ping"} — Kraken application-level keepalive
//! Ping interval: 30 s
//!
//! Frame routing: top-level "channel" field.
//! Control frames (method == "pong"/"subscribe"/"unsubscribe"/"heartbeat") return None
//! from extract_topic.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{AccountType, WebSocketError};
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec,
    TopicKey, TopicRegistry,
    WsProtocol,
};

use super::parser as kraken_parser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache — Kraken is spot-only for public channels
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// KrakenProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Kraken WS v2 protocol shim.
///
/// Public channels only. Private channels (executions, balances) require a
/// token in subscribe params — not supported in this implementation (wasm demo
/// scope is public market data only).
pub struct KrakenProtocol;

impl KrakenProtocol {
    /// Map StreamKind → Kraken v2 wire channel name.
    ///
    /// Returns None for stream kinds that have no public channel on Kraken v2.
    pub(crate) fn channel_name(kind: &StreamKind) -> Option<&'static str> {
        match kind {
            StreamKind::Ticker => Some("ticker"),
            StreamKind::Trade => Some("trade"),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => Some("book"),
            StreamKind::Kline { .. } => Some("ohlc"),
            StreamKind::MarketWarning => Some("instrument"),
            _ => None,
        }
    }

    /// Format symbol as BTC/USD (Kraken v2 uses slash, not hyphen, not XBT).
    ///
    /// CRITICAL: Kraken WS v2 uses BTC, NOT XBT. Callers must pass
    /// Symbol::new("BTC", "USD") — "XBT/USD" will be rejected by the exchange.
    fn format_symbol(spec: &StreamSpec) -> Result<String, WebSocketError> {
        let resolved = spec
            .symbol
            .resolve(crate::core::types::ExchangeId::Kraken, spec.account_type)
            .map_err(|e| {
                WebSocketError::WireAbsent(format!(
                    "kraken: symbol normalization failed: {}",
                    e
                ))
            })?;
        Ok(to_kraken_ws_symbol(&resolved.to_string()))
    }

    /// Build subscribe / unsubscribe frame.
    fn build_frame(op: &str, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let channel = Self::channel_name(&spec.kind).ok_or_else(|| {
            WebSocketError::WireAbsent(format!(
                "Kraken v2 has no public WS channel for {:?} — use REST for this data kind",
                spec.kind
            ))
        })?;

        let symbol = Self::format_symbol(spec)?;

        // Base params — symbol is always present for public channels
        let mut params = json!({
            "channel": channel,
            "symbol": [symbol],
        });

        // Channel-specific extra parameters
        match &spec.kind {
            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                let depth = spec.depth.unwrap_or(10) as u64;
                params["depth"] = json!(depth);
            }
            StreamKind::Kline { interval } => {
                let minutes = kline_interval_to_minutes(interval);
                params["interval"] = json!(minutes);
            }
            _ => {}
        }

        let frame = json!({
            "method": op,
            "params": params,
        });

        Ok(WsFrame::Text(frame.to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for KrakenProtocol {
    fn name(&self) -> &'static str {
        "kraken"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Kraken has no testnet for public channels.
        // Private URL (wss://ws-auth.kraken.com/v2) is not in scope for wasm demo.
        Url::parse("wss://ws.kraken.com/v2").expect("kraken ws endpoint is valid")
    }

    /// JSON application-level ping: {"method":"ping"}.
    ///
    /// Kraken v2 responds with {"method":"pong",...}.
    /// The transport also sends native WS Ping frames for RTT measurement.
    fn ping_frame(&self) -> Option<WsFrame> {
        Some(WsFrame::Text(
            r#"{"method":"ping"}"#.to_string(),
        ))
    }

    /// 30-second ping interval — matches the bespoke loop behaviour.
    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    /// Kraken public channels require no authentication.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    fn is_pong(&self, raw: &Value) -> bool {
        raw.get("method").and_then(|v| v.as_str()) == Some("pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        let method = raw.get("method").and_then(|v| v.as_str());
        matches!(method, Some("subscribe") | Some("unsubscribe"))
    }

    /// Extract routing topic from an incoming Kraken v2 frame.
    ///
    /// Data frames carry a top-level "channel" field.
    /// Control frames carry a top-level "method" field (pong, subscribe, heartbeat, status).
    /// Returns None for all control frames; Some(channel) for data frames.
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Control frames have a "method" field — filter them out.
        // This covers: pong, subscribe ACK, unsubscribe ACK, heartbeat, status.
        if raw.get("method").is_some() {
            return None;
        }

        let channel = raw.get("channel").and_then(|v| v.as_str())?;

        // Filter server-side control channels that carry no market data.
        match channel {
            "heartbeat" | "status" => None,
            _ => Some(TopicKey::new(channel)),
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
        // ticker → Ticker
        .register(StreamKind::Ticker, at, "ticker", kraken_parser::parse_ws_ticker)
        // trade → Trade
        .register(StreamKind::Trade, at, "trade", kraken_parser::parse_ws_trade)
        // book → OrderbookSnapshot OR OrderbookDelta (same parser, branches on raw["type"])
        // Registered under both Orderbook and OrderbookDelta so both StreamKind subscriptions
        // are satisfied. The parser returns the appropriate variant per frame.
        .register(StreamKind::Orderbook, at, "book", kraken_parser::parse_ws_book)
        .register(StreamKind::OrderbookDelta, at, "book", kraken_parser::parse_ws_book)
        // ohlc → Kline
        .register(
            StreamKind::Kline { interval: KlineInterval::new("") },
            at,
            "ohlc",
            kraken_parser::parse_ws_ohlc,
        )
        // instrument → MarketWarning (only when status != "online")
        .register(StreamKind::MarketWarning, at, "instrument", kraken_parser::parse_ws_instrument)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a symbol string to Kraken WS v2 slash format.
///
/// Kraken WS v2 requires `"BTC/USD"` (slash, BTC not XBT). The REST API uses the
/// concat form `"XBTUSD"`. This function normalises both forms to the WS format so
/// that consumers using `SymbolNormalizer::to_exchange` (REST) still work correctly
/// when their symbol is forwarded to the WS connector.
///
/// Rules:
/// - Already contains '/' → returned as-is (idempotent).
/// - Otherwise: split a known quote suffix off the right, map XBT→BTC on the base,
///   then join with '/'.
/// - Unknown suffix: return unchanged (safe fallback — Kraken will NAK the subscribe
///   rather than silently drop data).
pub(crate) fn to_kraken_ws_symbol(sym: &str) -> String {
    // Idempotent: already in WS slash form.
    if sym.contains('/') {
        return sym.to_string();
    }

    // Known Kraken quote currencies (longest first to avoid prefix ambiguity).
    const KNOWN_QUOTES: &[&str] = &["USDT", "USDC", "EUR", "USD", "GBP", "AUD", "CHF", "JPY"];

    for q in KNOWN_QUOTES {
        if let Some(base) = sym.to_uppercase().strip_suffix(*q) {
            if !base.is_empty() {
                // XBT is Kraken's legacy REST code for Bitcoin; WS v2 uses BTC.
                let ws_base = if base == "XBT" { "BTC" } else { base };
                return format!("{}/{}", ws_base, q);
            }
        }
    }

    // No known quote matched — return unchanged so the caller gets a clear NAK.
    sym.to_string()
}

/// Convert KlineInterval to Kraken ohlc interval in minutes.
fn kline_interval_to_minutes(interval: &KlineInterval) -> u32 {
    match interval.as_str() {
        "1m" => 1,
        "5m" => 5,
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        "4h" => 240,
        "1d" => 1440,
        "1w" => 10080,
        _ => 1,
    }
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
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_ticker() {
        let proto = KrakenProtocol;
        let spec = make_spec(StreamKind::Ticker, "BTC/USD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["method"], "subscribe");
            assert_eq!(v["params"]["channel"], "ticker");
            assert_eq!(v["params"]["symbol"][0], "BTC/USD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_trade() {
        let proto = KrakenProtocol;
        let spec = make_spec(StreamKind::Trade, "BTC/USD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["params"]["channel"], "trade");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_book_includes_depth() {
        let proto = KrakenProtocol;
        let mut spec = make_spec(StreamKind::Orderbook, "BTC/USD");
        spec.depth = Some(25);
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["params"]["channel"], "book");
            assert_eq!(v["params"]["depth"], 25);
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_ohlc_includes_interval() {
        let proto = KrakenProtocol;
        let spec = make_spec(StreamKind::Kline { interval: KlineInterval::new("1h") }, "BTC/USD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["params"]["channel"], "ohlc");
            assert_eq!(v["params"]["interval"], 60);
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_unsupported_returns_not_supported() {
        let proto = KrakenProtocol;
        let spec = make_spec(StreamKind::Liquidation, "BTC/USD");
        let result = proto.subscribe_frame(&spec);
        assert!(
            matches!(result, Err(WebSocketError::WireAbsent(_))),
            "Liquidation must return WireAbsent, got {:?}",
            result
        );
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn extract_topic_ticker_data_frame() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({
            "channel": "ticker",
            "type": "snapshot",
            "data": []
        });
        assert_eq!(proto.extract_topic(&raw), Some(TopicKey::new("ticker")));
    }

    #[test]
    fn extract_topic_pong_returns_none() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"method": "pong", "time_in": "...", "time_out": "..."});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_subscribe_ack_returns_none() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"method": "subscribe", "success": true});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_heartbeat_returns_none() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"channel": "heartbeat"});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_status_returns_none() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"channel": "status", "data": [{"api_version": "v2"}]});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    // ── is_pong / is_subscribe_ack ────────────────────────────────────────────

    #[test]
    fn is_pong_true_for_method_pong() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"method": "pong"});
        assert!(proto.is_pong(&raw));
    }

    #[test]
    fn is_pong_false_for_data_frame() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"channel": "ticker", "data": []});
        assert!(!proto.is_pong(&raw));
    }

    #[test]
    fn is_subscribe_ack_true_for_subscribe() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"method": "subscribe", "success": true});
        assert!(proto.is_subscribe_ack(&raw));
    }

    #[test]
    fn is_subscribe_ack_true_for_unsubscribe() {
        let proto = KrakenProtocol;
        let raw = serde_json::json!({"method": "unsubscribe", "success": true});
        assert!(proto.is_subscribe_ack(&raw));
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_is_json_ping() {
        let proto = KrakenProtocol;
        let frame = proto.ping_frame();
        assert!(frame.is_some());
        if let Some(WsFrame::Text(s)) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["method"], "ping");
        } else {
            panic!("expected Some(Text)");
        }
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_covers_public_channels() {
        let proto = KrakenProtocol;
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
        assert!(reg.supports(&StreamKind::MarketWarning, at), "MarketWarning");
    }

    #[test]
    fn book_channel_has_two_parsers() {
        let proto = KrakenProtocol;
        let reg = proto.topic_registry(AccountType::Spot);
        let key = TopicKey::new("book");
        let parsers = reg.dispatch_all(&key);
        // Orderbook + OrderbookDelta are registered with the SAME function ptr,
        // so dispatch_all de-duplicates → 1 unique parser.
        assert_eq!(parsers.len(), 1, "book channel must have 1 unique parser (de-duped)");
    }

    // ── to_kraken_ws_symbol ───────────────────────────────────────────────────

    #[test]
    fn ws_symbol_xbtusd_to_btc_usd() {
        assert_eq!(to_kraken_ws_symbol("XBTUSD"), "BTC/USD");
    }

    #[test]
    fn ws_symbol_btcusd_to_btc_usd() {
        assert_eq!(to_kraken_ws_symbol("BTCUSD"), "BTC/USD");
    }

    #[test]
    fn ws_symbol_btc_usd_idempotent() {
        assert_eq!(to_kraken_ws_symbol("BTC/USD"), "BTC/USD");
    }

    #[test]
    fn ws_symbol_xbtusdt_to_btc_usdt() {
        assert_eq!(to_kraken_ws_symbol("XBTUSDT"), "BTC/USDT");
    }

    #[test]
    fn ws_symbol_ethusdt_to_eth_usdt() {
        assert_eq!(to_kraken_ws_symbol("ETHUSDT"), "ETH/USDT");
    }

    #[test]
    fn ws_symbol_ethusd_to_eth_usd() {
        assert_eq!(to_kraken_ws_symbol("ETHUSD"), "ETH/USD");
    }

    #[test]
    fn ws_symbol_unknown_passthrough() {
        // No known quote suffix — returned unchanged so Kraken gives a clear NAK.
        assert_eq!(to_kraken_ws_symbol("BTCXXX"), "BTCXXX");
    }

    // ── subscribe_frame uses WS symbol format ─────────────────────────────────

    #[test]
    fn subscribe_frame_ticker_xbtusd_normalised() {
        // When a consumer passes the REST format "XBTUSD" as raw, the connector
        // must normalise it to "BTC/USD" before sending to Kraken WS v2.
        let proto = KrakenProtocol;
        let spec = make_spec(StreamKind::Ticker, "XBTUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(
                v["params"]["symbol"][0], "BTC/USD",
                "XBTUSD must be normalised to BTC/USD for Kraken WS v2"
            );
        } else {
            panic!("expected Text frame");
        }
    }

    // ── kline interval mapping ────────────────────────────────────────────────

    #[test]
    fn kline_interval_to_minutes_coverage() {
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("1m")), 1);
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("5m")), 5);
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("15m")), 15);
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("30m")), 30);
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("1h")), 60);
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("4h")), 240);
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("1d")), 1440);
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("1w")), 10080);
        // Unknown → 1 (default)
        assert_eq!(kline_interval_to_minutes(&KlineInterval::new("2m")), 1);
    }
}
