//! LighterProtocol — WsProtocol implementation for Lighter DEX WebSocket.
//!
//! ## Lighter WebSocket protocol
//!
//! - Mainnet endpoint: `wss://mainnet.zklighter.elliot.ai/stream`
//! - Testnet endpoint: `wss://testnet.zklighter.elliot.ai/stream`
//!
//! ## Subscribe frame
//!
//! ```json
//! {"type":"subscribe","channel":"order_book/0"}
//! ```
//!
//! Channels use slash-separated `<name>/<market_id>` on subscribe.
//! The server echoes the channel with a colon separator in data frames.
//!
//! ## Frame types
//!
//! - `"connected"` — initial connection ack, no data.
//! - `"update/order_book"` — orderbook snapshot/delta.
//! - `"update/trade"` — trade executions (`trades` plural array).
//! - `"update/market_stats"` — market statistics (price, volume, funding).
//! - `"update/ticker"` — lightweight best-bid/ask snapshot.
//! - `"update/candle"` — OHLCV candle update for `candle/{market_id}/{resolution}`.
//!
//! ## Candle subscribe
//!
//! ```json
//! {"type":"subscribe","channel":"candle/1/1m"}
//! ```
//!
//! Supported resolutions: `1m`, `5m`, `15m`, `1h`, `4h`, `1d`.
//!
//! Server data frame:
//! ```json
//! {
//!   "type": "update/candle",
//!   "channel": "candle:1:1m",
//!   "candle": {
//!     "market_id": 1,
//!     "resolution": "1m",
//!     "open": "76500.0",
//!     "high": "76600.0",
//!     "low": "76400.0",
//!     "close": "76550.0",
//!     "base_token_volume": "1.5",
//!     "open_time": 1700000000
//!   }
//! }
//! ```
//!
//! ## Topic key
//!
//! `"<channel_type>:<market_id>"` e.g. `"update/order_book:0"`, `"update/trade:1"`.
//! `channel_type` is taken from the `type` field of the incoming frame.
//! `market_id` is extracted from the `channel` field using `rsplit(':')` or `rsplit('/')`.
//!
//! For candles the channel is `"candle:{market_id}:{resolution}"` (3-part colon form).
//! Topic key strips the resolution: `"update/candle:{market_id}"`. The resolution is
//! stored in a thread-local (set during `extract_topic`) and consumed by the parser.
//!
//! ## Ping / pong
//!
//! Lighter uses standard WebSocket Ping/Pong at protocol level.
//! No application-level ping frame needed. `ping_frame()` returns `None`.

use std::cell::Cell;
use std::sync::OnceLock;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, StreamEvent, WebSocketError, WebSocketResult,
};
use crate::core::websocket::{KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};

use super::endpoints::{LighterUrls, map_kline_interval, symbol_to_market_id};
use super::websocket::{
    parse_candle, parse_orderbook, parse_ticker_channel, parse_trade, parse_market_stats,
};

// ─────────────────────────────────────────────────────────────────────────────
// Thread-local: kline resolution for the current candle frame.
// Set by extract_topic; consumed by wrap_kline parser bridge.
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    static CURRENT_RESOLUTION: Cell<Option<String>> = const { Cell::new(None) };
}

fn set_current_resolution(res: impl Into<String>) {
    CURRENT_RESOLUTION.with(|c| c.set(Some(res.into())));
}

pub(super) fn take_current_resolution() -> String {
    CURRENT_RESOLUTION.with(|c| c.take()).unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache — single registry (Lighter perp-only, single account type)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// LighterProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Lighter DEX WS protocol shim.
///
/// Public market-data channels only (public data by design):
/// orderbook, trades, market stats, ticker.
/// Auth channels (account_all, account_market) are native-only by design.
pub struct LighterProtocol {
    testnet: bool,
}

impl LighterProtocol {
    pub fn new(testnet: bool) -> Self {
        Self { testnet }
    }

    /// Build the channel name and market_id for a StreamSpec.
    ///
    /// Returns `(subscribe_channel, topic_prefix)` where:
    /// - `subscribe_channel` is the slash-form channel sent in the subscribe frame
    ///   (`"order_book/0"`, `"trade/1"`, etc.)
    /// - `topic_prefix` is the type-field prefix used for topic routing
    ///   (`"update/order_book"`, `"update/trade"`, etc.)
    fn channel_for_spec(spec: &StreamSpec) -> Result<(String, &'static str), WebSocketError> {
        let base = extract_base(spec);
        let market_id = symbol_to_market_id(&base)
            .ok_or_else(|| WebSocketError::NotSupported(format!(
                "Lighter: unknown market for '{}'. \
                 Known perp markets: ETH(0), BTC(1), SOL(2), ARB(3), OP(4), DOGE(5), ...",
                base
            )))?;

        let (chan_name, type_prefix) = match &spec.kind {
            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                ("order_book", "update/order_book")
            }
            StreamKind::Trade => {
                ("trade", "update/trade")
            }
            // Lighter has a single trade channel — there is no separate aggregate-trade
            // feed.  Routing AggTrade to the same channel would cause WRONG_TYPE errors
            // in consumers expecting AggTrade events but receiving Trade events.
            // Return NotSupported so callers get a clear error instead of misrouting.
            StreamKind::AggTrade => {
                return Err(WebSocketError::NotSupported(
                    "Lighter has no AggTrade channel — subscribe to Trade instead. \
                     The exchange publishes individual trades only."
                        .into(),
                ));
            }
            StreamKind::Ticker => {
                ("ticker", "update/ticker")
            }
            StreamKind::FundingRate | StreamKind::MarkPrice => {
                ("market_stats", "update/market_stats")
            }
            StreamKind::Kline { interval } => {
                let res = map_kline_interval(interval.as_str());
                // subscribe channel: "candle/<market_id>/<resolution>"
                let subscribe_channel = format!("candle/{}/{}", market_id, res);
                return Ok((subscribe_channel, "update/candle"));
            }
            other => {
                return Err(WebSocketError::NotSupported(format!(
                    "Lighter WS has no public channel for {:?} \
                     (auth-gated channels are native-only by design)",
                    other
                )));
            }
        };

        let subscribe_channel = format!("{}/{}", chan_name, market_id);
        Ok((subscribe_channel, type_prefix))
    }
}

/// Extract the base asset from a StreamSpec symbol.
///
/// Lighter uses only the base asset to identify markets (e.g. `"ETH"`, `"BTC"`).
/// Strips quote asset if present (`"ETH/USDC"` → `"ETH"`).
fn extract_base(spec: &StreamSpec) -> String {
    let raw = spec.symbol.to_string();
    // Handle BASE/QUOTE → BASE
    if let Some(slash) = raw.find('/') {
        raw[..slash].to_uppercase()
    } else {
        raw.to_uppercase()
    }
}

/// Extract the market_id string from a Lighter channel field.
///
/// Lighter sends either colon-separated (`"order_book:0"`) or slash-separated
/// (`"order_book/0"`) channel names. We always take the last segment.
pub(super) fn extract_market_id_from_channel(channel: &str) -> &str {
    // Lighter channels are `<name>:<market_id>`; some server versions use a
    // slash separator (`<name>/<market_id>`). `rsplit(sep).next()` always
    // yields Some (the whole string when the separator is absent), so pick the
    // separator that actually occurs rather than chaining or_else.
    let sep = if channel.contains(':') {
        ':'
    } else {
        '/'
    };
    channel.rsplit(sep).next().unwrap_or(channel)
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for LighterProtocol {
    fn name(&self) -> &'static str {
        "lighter"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        let url = if self.testnet {
            LighterUrls::TESTNET.ws
        } else {
            LighterUrls::MAINNET.ws
        };
        Url::parse(url).expect("lighter ws endpoint url is valid")
    }

    /// Lighter uses standard WebSocket Ping/Pong at protocol level.
    fn ping_frame(&self) -> Option<WsFrame> {
        None
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let (channel, _) = Self::channel_for_spec(spec)?;
        let frame = json!({
            "type": "subscribe",
            "channel": channel,
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let (channel, _) = Self::channel_for_spec(spec)?;
        let frame = json!({
            "type": "unsubscribe",
            "channel": channel,
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    /// Lighter public channels require no authentication.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    /// Suppress `"connected"` system frames.
    ///
    /// The server sends `{"type":"connected","session_id":"..."}` on connect.
    /// Suppress it so `extract_topic` is not called for it.
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        matches!(
            raw.get("type").and_then(|v| v.as_str()),
            Some("connected") | Some("pong")
        )
    }

    /// Extract the routing topic from a Lighter data frame.
    ///
    /// Topic key format: `"<type_field>:<market_id>"`.
    ///
    /// The `type` field (e.g. `"update/order_book"`) combined with the
    /// market_id extracted from the `channel` field forms the routing key.
    ///
    /// For candle frames the channel is `"candle:{market_id}:{resolution}"`.
    /// The resolution is stored in a thread-local so `wrap_kline` can read it.
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let msg_type = raw.get("type").and_then(|v| v.as_str())?;

        let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");

        // Candle frames come in two types:
        //   "update/candle"     — live per-resolution updates
        //   "subscribed/candle" — initial snapshot on first subscribe
        // Both carry candle data in the "candles" array.
        // Route both under the "update/candle:*" registry key.
        if msg_type == "update/candle" || msg_type == "subscribed/candle" {
            // Channel format: "candle:{market_id}:{resolution}" or "candle/{market_id}/{resolution}".
            let sep = if channel.contains(':') { ':' } else { '/' };
            let mut parts = channel.splitn(3, sep);
            let _name = parts.next(); // "candle"
            let market_id = parts.next().unwrap_or("0");
            let resolution = parts.next().unwrap_or("");
            set_current_resolution(resolution);
            return Some(TopicKey::new(&format!("update/candle:{}", market_id)));
        }

        // Only route remaining `update/*` frames.
        if !msg_type.starts_with("update/") {
            return None;
        }

        let market_id = extract_market_id_from_channel(channel);

        Some(TopicKey::new(&format!("{}:{}", msg_type, market_id)))
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        REGISTRY.get_or_init(build_registry)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
//
// Topic keys use `"<type_field>:<market_id>"`.
// Because market_id varies per symbol, wildcard patterns are used:
// `"update/order_book:*"`, `"update/trade:*"`, etc.
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    // Lighter is perp-only — use FuturesCross as canonical AccountType.
    let at = AccountType::FuturesCross;

    // AggTrade is intentionally absent: Lighter has no aggregate-trade channel.
    // subscribe_frame returns NotSupported for AggTrade so callers get a clean error.
    TopicRegistry::builder()
        .register(StreamKind::Orderbook, at, "update/order_book:*", wrap_orderbook)
        .register(StreamKind::OrderbookDelta, at, "update/order_book:*", wrap_orderbook)
        .register(StreamKind::Trade, at, "update/trade:*", wrap_trade)
        .register(StreamKind::Ticker, at, "update/ticker:*", wrap_ticker)
        .register(StreamKind::FundingRate, at, "update/market_stats:*", wrap_market_stats)
        .register(StreamKind::MarkPrice, at, "update/market_stats:*", wrap_market_stats)
        .register(
            StreamKind::Kline { interval: KlineInterval::new("") },
            at,
            "update/candle:*",
            wrap_kline,
        )
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser bridge functions
//
// These thin wrappers adapt the existing parser fns (which live in websocket.rs
// alongside the bespoke LighterWebSocket) to the `fn(&Value) -> WebSocketResult<StreamEvent>`
// signature expected by TopicRegistry.
// ─────────────────────────────────────────────────────────────────────────────

fn wrap_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    parse_orderbook(raw, channel)
        .ok_or_else(|| WebSocketError::Parse("lighter: orderbook parse returned None".into()))
}

fn wrap_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    let events = parse_trade(raw, channel);
    // Use FieldAbsent (not Parse) so the transport silently skips empty frames
    // instead of broadcasting an error that would break the consumer stream.
    // An empty trades array is a valid server message (no trades in the window).
    events
        .into_iter()
        .next()
        .ok_or_else(|| WebSocketError::FieldAbsent("lighter: trades[0]".into()))
}

fn wrap_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    parse_ticker_channel(raw, channel)
        .ok_or_else(|| WebSocketError::Parse("lighter: ticker parse returned None".into()))
}

fn wrap_market_stats(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    parse_market_stats(raw, channel)
        .ok_or_else(|| WebSocketError::Parse("lighter: market_stats parse returned None".into()))
}

fn wrap_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    // Resolution was stored in thread-local by extract_topic (same task context).
    let resolution = take_current_resolution();
    parse_candle(raw, channel, &resolution)
        .ok_or_else(|| WebSocketError::Parse("lighter: candle parse returned None".into()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OwnedSymbolInput;

    fn proto() -> LighterProtocol {
        LighterProtocol::new(false)
    }

    fn make_spec(kind: StreamKind, sym: &str) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: OwnedSymbolInput::Raw(sym.to_string()),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        }
    }

    // ── endpoint ──────────────────────────────────────────────────────────────

    #[test]
    fn mainnet_endpoint() {
        let url = proto().endpoint(AccountType::FuturesCross, false);
        assert_eq!(url.as_str(), "wss://mainnet.zklighter.elliot.ai/stream");
    }

    #[test]
    fn testnet_endpoint() {
        let url = LighterProtocol::new(true).endpoint(AccountType::FuturesCross, true);
        assert_eq!(url.as_str(), "wss://testnet.zklighter.elliot.ai/stream");
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_returns_none() {
        assert!(proto().ping_frame().is_none());
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_orderbook_eth() {
        let spec = make_spec(StreamKind::Orderbook, "ETH");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["type"], "subscribe");
            assert_eq!(v["channel"], "order_book/0"); // ETH = market_id 0
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_orderbook_btc() {
        let spec = make_spec(StreamKind::Orderbook, "BTC");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["channel"], "order_book/1"); // BTC = market_id 1
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_trade_btc() {
        let spec = make_spec(StreamKind::Trade, "BTC");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["channel"], "trade/1");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_ticker_eth() {
        let spec = make_spec(StreamKind::Ticker, "ETH");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["channel"], "ticker/0");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_unknown_market_err() {
        let spec = make_spec(StreamKind::Trade, "NONEXISTENT");
        assert!(proto().subscribe_frame(&spec).is_err());
    }

    #[test]
    fn subscribe_agg_trade_returns_not_supported() {
        // Lighter has no AggTrade channel — subscribing must return NotSupported, not
        // silently misroute to the Trade parser (which causes WRONG_TYPE in consumers).
        let spec = make_spec(StreamKind::AggTrade, "BTC");
        let result = proto().subscribe_frame(&spec);
        assert!(
            matches!(result, Err(WebSocketError::NotSupported(_))),
            "AggTrade must return NotSupported for Lighter, got {:?}",
            result
        );
    }

    #[test]
    fn subscribe_slash_symbol_eth() {
        // ETH/USDC → base=ETH → market_id=0
        let spec = make_spec(StreamKind::Orderbook, "ETH/USDC");
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["channel"], "order_book/0");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── unsubscribe_frame ─────────────────────────────────────────────────────

    #[test]
    fn unsubscribe_orderbook_eth() {
        let spec = make_spec(StreamKind::Orderbook, "ETH");
        let frame = proto().unsubscribe_frame(&spec).expect("unsub frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["type"], "unsubscribe");
            assert_eq!(v["channel"], "order_book/0");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── is_subscribe_ack ──────────────────────────────────────────────────────

    #[test]
    fn ack_connected() {
        let raw = serde_json::json!({"type":"connected","session_id":"abc"});
        assert!(proto().is_subscribe_ack(&raw));
    }

    #[test]
    fn ack_pong() {
        let raw = serde_json::json!({"type":"pong"});
        assert!(proto().is_subscribe_ack(&raw));
    }

    #[test]
    fn ack_update_is_false() {
        let raw = serde_json::json!({
            "type": "update/order_book",
            "channel": "order_book:0",
        });
        assert!(!proto().is_subscribe_ack(&raw));
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn topic_from_update_order_book() {
        let raw = serde_json::json!({
            "type": "update/order_book",
            "channel": "order_book:0",
            "asks": [],
            "bids": []
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("update/order_book:0"))
        );
    }

    #[test]
    fn topic_from_update_trade() {
        let raw = serde_json::json!({
            "type": "update/trade",
            "channel": "trade:1",
            "trades": []
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("update/trade:1"))
        );
    }

    #[test]
    fn topic_from_update_market_stats() {
        let raw = serde_json::json!({
            "type": "update/market_stats",
            "channel": "market_stats:0",
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("update/market_stats:0"))
        );
    }

    #[test]
    fn topic_none_for_connected() {
        let raw = serde_json::json!({"type":"connected","session_id":"x"});
        assert_eq!(proto().extract_topic(&raw), None);
    }

    #[test]
    fn topic_none_for_ping() {
        let raw = serde_json::json!({"type":"ping","timestamp":12345});
        assert_eq!(proto().extract_topic(&raw), None);
    }

    #[test]
    fn topic_slash_channel_format() {
        // Some server versions use slash separator in channel field
        let raw = serde_json::json!({
            "type": "update/order_book",
            "channel": "order_book/0",
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("update/order_book:0"))
        );
    }

    // ── extract_market_id_from_channel ────────────────────────────────────────

    #[test]
    fn extract_market_id_colon() {
        assert_eq!(extract_market_id_from_channel("order_book:0"), "0");
        assert_eq!(extract_market_id_from_channel("market_stats:1"), "1");
    }

    #[test]
    fn extract_market_id_slash() {
        assert_eq!(extract_market_id_from_channel("order_book/0"), "0");
        assert_eq!(extract_market_id_from_channel("trade/1"), "1");
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn subscribe_kline_btc_1m() {
        let spec = make_spec(
            StreamKind::Kline { interval: KlineInterval::new("1m") },
            "BTC",
        );
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            assert_eq!(v["type"], "subscribe");
            // BTC = market_id 1, resolution = 1m
            assert_eq!(v["channel"], "candle/1/1m");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_kline_eth_4h() {
        let spec = make_spec(
            StreamKind::Kline { interval: KlineInterval::new("4h") },
            "ETH",
        );
        let frame = proto().subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).unwrap();
            // ETH = market_id 0, resolution = 4h
            assert_eq!(v["channel"], "candle/0/4h");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn extract_topic_update_candle() {
        // Live frame format: "candles" array with short field names.
        let raw = serde_json::json!({
            "type": "update/candle",
            "channel": "candle:1:1m",
            "timestamp": 1780012392587i64,
            "candles": [{"t": 1780012380000i64, "o": 73517.4, "h": 73522.1, "l": 73517.4, "c": 73520.5, "v": 0.14261}]
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("update/candle:1"))
        );
    }

    #[test]
    fn extract_topic_subscribed_candle_routes_same_key() {
        // Initial snapshot frame type "subscribed/candle" — must route as "update/candle:*".
        let raw = serde_json::json!({
            "type": "subscribed/candle",
            "channel": "candle:1:1m",
            "candles": [{"t": 1780012380000i64, "o": 73517.4, "h": 73522.1, "l": 73517.4, "c": 73520.5, "v": 0.14261}]
        });
        assert_eq!(
            proto().extract_topic(&raw),
            Some(TopicKey::new("update/candle:1"))
        );
    }

    #[test]
    fn extract_topic_candle_stores_resolution() {
        let raw = serde_json::json!({
            "type": "update/candle",
            "channel": "candle:1:4h",
            "candles": []
        });
        proto().extract_topic(&raw);
        // Resolution should be stored in thread-local.
        let res = take_current_resolution();
        assert_eq!(res, "4h");
    }

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
            reg.supports(&StreamKind::Kline { interval: KlineInterval::new("") }, at),
            "Kline"
        );
        // AggTrade intentionally absent from registry — Lighter has no aggregate-trade channel.
        assert!(!reg.supports(&StreamKind::AggTrade, at), "AggTrade must NOT be registered");
    }

    #[test]
    fn registry_wildcard_dispatches() {
        let p = proto();
        let reg = p.topic_registry(AccountType::FuturesCross);
        assert!(reg.dispatch(&TopicKey::new("update/order_book:0")).is_some());
        assert!(reg.dispatch(&TopicKey::new("update/trade:1")).is_some());
        assert!(reg.dispatch(&TopicKey::new("update/ticker:0")).is_some());
        assert!(reg.dispatch(&TopicKey::new("update/market_stats:2")).is_some());
        assert!(reg.dispatch(&TopicKey::new("update/candle:1")).is_some());
    }
}
