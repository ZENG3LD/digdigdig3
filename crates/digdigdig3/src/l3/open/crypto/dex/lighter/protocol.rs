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
//!
//! ## Topic key
//!
//! `"<channel_type>:<market_id>"` e.g. `"update/order_book:0"`, `"update/trade:1"`.
//! `channel_type` is taken from the `type` field of the incoming frame.
//! `market_id` is extracted from the `channel` field using `rsplit(':')` or `rsplit('/')`.
//!
//! ## Ping / pong
//!
//! Lighter uses standard WebSocket Ping/Pong at protocol level.
//! No application-level ping frame needed. `ping_frame()` returns `None`.

use std::sync::OnceLock;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, StreamEvent, WebSocketError, WebSocketResult,
};
use crate::core::websocket::{StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};

use super::endpoints::{LighterUrls, symbol_to_market_id};
use super::websocket::{
    parse_orderbook, parse_ticker_channel, parse_trade, parse_market_stats,
};

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
            StreamKind::Trade | StreamKind::AggTrade => {
                ("trade", "update/trade")
            }
            StreamKind::Ticker => {
                ("ticker", "update/ticker")
            }
            StreamKind::FundingRate | StreamKind::MarkPrice => {
                ("market_stats", "update/market_stats")
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
    channel
        .rsplit(':')
        .next()
        .or_else(|| channel.rsplit('/').next())
        .unwrap_or(channel)
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
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let msg_type = raw.get("type").and_then(|v| v.as_str())?;

        // Only route `update/*` frames.
        if !msg_type.starts_with("update/") {
            return None;
        }

        let channel = raw.get("channel").and_then(|v| v.as_str()).unwrap_or("");
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

    TopicRegistry::builder()
        .register(StreamKind::Orderbook, at, "update/order_book:*", wrap_orderbook)
        .register(StreamKind::OrderbookDelta, at, "update/order_book:*", wrap_orderbook)
        .register(StreamKind::Trade, at, "update/trade:*", wrap_trade)
        .register(StreamKind::AggTrade, at, "update/trade:*", wrap_trade)
        .register(StreamKind::Ticker, at, "update/ticker:*", wrap_ticker)
        .register(StreamKind::FundingRate, at, "update/market_stats:*", wrap_market_stats)
        .register(StreamKind::MarkPrice, at, "update/market_stats:*", wrap_market_stats)
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
    events.into_iter().next().ok_or_else(|| WebSocketError::Parse(
        "lighter: no trades in update/trade frame".into()
    ))
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
    fn registry_supports_public_channels() {
        let reg = proto().topic_registry(AccountType::FuturesCross);
        let at = AccountType::FuturesCross;
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook");
        assert!(reg.supports(&StreamKind::Trade, at), "Trade");
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker");
        assert!(reg.supports(&StreamKind::FundingRate, at), "FundingRate");
        assert!(reg.supports(&StreamKind::MarkPrice, at), "MarkPrice");
    }

    #[test]
    fn registry_wildcard_dispatches() {
        let reg = proto().topic_registry(AccountType::FuturesCross);
        assert!(reg.dispatch(&TopicKey::new("update/order_book:0")).is_some());
        assert!(reg.dispatch(&TopicKey::new("update/trade:1")).is_some());
        assert!(reg.dispatch(&TopicKey::new("update/ticker:0")).is_some());
        assert!(reg.dispatch(&TopicKey::new("update/market_stats:2")).is_some());
    }
}
