//! GeminiProtocol — WsProtocol implementation for the Gemini exchange.
//!
//! Declarative shim: supplies endpoint URL, ping frame, subscribe/unsubscribe
//! frames, topic extraction, and topic registry to UniversalWsTransport.
//!
//! ## Gemini v2 marketdata protocol
//!
//! - Endpoint: `wss://api.gemini.com/v2/marketdata` (mainnet + testnet)
//! - Subscribe:
//!   `{"type":"subscribe","subscriptions":[{"name":"l2","symbols":["BTCUSD"]}]}`
//! - Unsubscribe: Gemini does not support per-stream unsubscribe; this impl
//!   returns the subscribe frame again as a no-op (reconnect clears state).
//! - Channels routed by `type` field:
//!   - `l2_updates`              → Orderbook delta (via `changes`) + Trade (via `trades`) + Ticker (synthetic)
//!   - `candles_<interval>_updates` → Kline
//!   - `auction_*`               → AuctionEvent
//!   - `heartbeat`               → ignored (route to None)
//!   - `subscribed`              → subscribe ACK, route to None
//!
//! ## Ticker synthesis
//!
//! Gemini has no dedicated ticker stream. Ticker is synthesised from the `l2`
//! subscription, which carries both book deltas (`changes`) and trades in one
//! frame. A process-wide `BOOK_STATE` map (keyed by symbol) tracks the running
//! best-bid, best-ask, and last-trade price. The ticker parser is registered
//! alongside Trade and Orderbook on the `l2_updates` topic and returns
//! `WebSocketError::FieldAbsent` (silent skip) until both bid and ask are known.
//!
//! ## Ping discipline — CRITICAL
//!
//! `ping_frame()` returns `None`. Gemini **disconnects the connection** if it
//! receives a WebSocket Ping frame from the client. Do not change this.

use std::collections::HashMap;
use std::sync::{Mutex as StdMutex, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, Kline, OrderBookLevel, OrderbookDelta, PublicTrade,
    StreamEvent, Ticker, TradeSide, WebSocketError, WebSocketResult,
};
use crate::core::websocket::{KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};
use crate::core::utils::timestamp_millis;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache — Gemini is spot-only for public market-data channels
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// Synthetic ticker state
//
// The `l2_updates` frame carries book deltas (changes) and trades in one shot.
// To synthesise a Ticker we track the running best-bid, best-ask, and last
// trade price per symbol in a process-wide map. The ticker parser reads this
// state and emits a Ticker event once both bid and ask are known.
//
// StdMutex is correct here — no `.await` is held across the lock.
// ─────────────────────────────────────────────────────────────────────────────

/// Per-symbol top-of-book state used to synthesise Ticker from l2_updates.
#[derive(Debug, Clone, Default)]
struct GeminiTickerState {
    /// Running best bid (highest buy price). `f64::NEG_INFINITY` = unknown.
    best_bid: f64,
    /// Running best ask (lowest sell price). `f64::INFINITY` = unknown.
    best_ask: f64,
    /// Price of the most recent trade. 0.0 = not yet seen.
    last_trade: f64,
    /// Timestamp of the most recent update (millis since epoch).
    last_ts: i64,
}

impl GeminiTickerState {
    fn new() -> Self {
        Self {
            best_bid: f64::NEG_INFINITY,
            best_ask: f64::INFINITY,
            last_trade: 0.0,
            last_ts: 0,
        }
    }

    fn has_bid_ask(&self) -> bool {
        self.best_bid > f64::NEG_INFINITY && self.best_ask < f64::INFINITY
    }
}

static BOOK_STATE: OnceLock<StdMutex<HashMap<String, GeminiTickerState>>> = OnceLock::new();

fn book_state() -> &'static StdMutex<HashMap<String, GeminiTickerState>> {
    BOOK_STATE.get_or_init(|| StdMutex::new(HashMap::new()))
}

// ─────────────────────────────────────────────────────────────────────────────
// GeminiProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Gemini v2 WebSocket protocol shim.
///
/// Public market-data channels only. Order-events (private) require auth-header
/// handshake and are not handled here.
pub struct GeminiProtocol;

impl GeminiProtocol {
    /// Map StreamKind → Gemini wire subscription name.
    ///
    /// Returns the name to use in the `subscriptions[].name` field.
    fn subscription_name(spec: &StreamSpec) -> Result<String, WebSocketError> {
        match &spec.kind {
            // Ticker rides the l2 feed — same subscription as Trade/Orderbook.
            StreamKind::Trade | StreamKind::Orderbook | StreamKind::Ticker => {
                Ok("l2".to_string())
            }
            StreamKind::Kline { interval } => {
                // Gemini candle feed name: "candles_1m", "candles_5m", etc.
                Ok(format!("candles_{}", interval.as_str()))
            }
            other => Err(WebSocketError::NotSupported(format!(
                "Gemini has no public WS channel for {:?}",
                other
            ))),
        }
    }

    /// Resolve symbol to Gemini wire format (uppercase, no separator).
    fn resolve_symbol(spec: &StreamSpec) -> Result<String, WebSocketError> {
        spec.symbol
            .resolve(crate::core::types::ExchangeId::Gemini, spec.account_type)
            .map(|s| s.to_ascii_uppercase())
            .map_err(|e| {
                WebSocketError::NotSupported(format!(
                    "gemini: symbol normalization failed: {}",
                    e
                ))
            })
    }

    /// Build subscribe frame.
    fn build_subscribe_frame(spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let name = Self::subscription_name(spec)?;
        let symbol = Self::resolve_symbol(spec)?;
        let frame = json!({
            "type": "subscribe",
            "subscriptions": [{ "name": name, "symbols": [symbol] }]
        });
        Ok(WsFrame::Text(frame.to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for GeminiProtocol {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn endpoint(&self, _account_type: AccountType, testnet: bool) -> Url {
        // Both mainnet and testnet share the same v2/marketdata path structure.
        let base = if testnet {
            "wss://api.sandbox.gemini.com/v2/marketdata"
        } else {
            "wss://api.gemini.com/v2/marketdata"
        };
        Url::parse(base).expect("gemini ws endpoint is valid")
    }

    /// Returns `None` — Gemini DISCONNECTS the connection if it receives a
    /// WebSocket Ping frame from the client. Never send client-initiated pings.
    fn ping_frame(&self) -> Option<WsFrame> {
        None
    }

    /// Gemini DISCONNECTS on any client-initiated WS Ping. The transport must
    /// never fall back to native Ping — liveness relies on Gemini's server-side
    /// heartbeats + the silent-stream watchdog.
    fn uses_native_ping(&self) -> bool {
        false
    }

    /// Interval is set to 30 s but is unused (no ping frame, no native ping).
    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_subscribe_frame(spec)
    }

    /// Gemini has no per-stream unsubscribe — re-send subscribe (idempotent on server).
    /// A real unsubscribe requires a full reconnect; that is left to the transport's
    /// reconnect logic.
    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_subscribe_frame(spec)
    }

    /// Gemini public market-data channels are unauthenticated.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    fn is_pong(&self, _raw: &Value) -> bool {
        // Gemini has no application-level pong (server does not respond to our pings,
        // and we never send pings — ping_frame returns None).
        false
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        raw.get("type").and_then(|v| v.as_str()) == Some("subscribed")
    }

    /// Extract routing topic from an incoming Gemini v2 frame.
    ///
    /// Gemini frames carry a `type` field. We use it as the TopicKey with a
    /// coarse grouping strategy:
    ///
    /// - `l2_updates`                  → TopicKey("l2_updates")
    /// - `candles_1m_updates`          → TopicKey("candles_updates") (strip interval)
    /// - `auction_*`                   → TopicKey("auction")
    /// - `heartbeat` / `subscribed`    → None (no dispatch)
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let t = raw.get("type").and_then(|v| v.as_str())?;
        match t {
            "heartbeat" | "subscribed" => None,
            "l2_updates" => Some(TopicKey::new("l2_updates")),
            t if t.starts_with("candles_") && t.ends_with("_updates") => {
                Some(TopicKey::new("candles_updates"))
            }
            t if t.starts_with("auction_") => Some(TopicKey::new("auction")),
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
        // l2_updates → Orderbook (changes array), Trade (trades array), and
        // Ticker (synthetic from accumulated top-of-book + last trade).
        // All three are registered under the same topic key; dispatch_all fires all.
        .register(StreamKind::Orderbook, at, "l2_updates", parse_l2_orderbook)
        .register(StreamKind::Trade, at, "l2_updates", parse_l2_trade)
        .register(StreamKind::Ticker, at, "l2_updates", parse_l2_ticker)
        // candles_updates → Kline (interval extracted from the `type` field inside parser)
        .register(
            StreamKind::Kline { interval: KlineInterval::new("") },
            at,
            "candles_updates",
            parse_candle,
        )
        // auction → AuctionEvent
        .register(StreamKind::AuctionEvent, at, "auction", parse_auction)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `l2_updates` frame → StreamEvent::OrderbookDelta.
///
/// Changes format: `[["buy","50000.00","1.5"],["sell","50001.00","0.8"]]`
/// Each change is `[side, price, qty]` — all strings.
pub(crate) fn parse_l2_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    let changes = raw
        .get("changes")
        .and_then(|c| c.as_array())
        .ok_or_else(|| WebSocketError::Parse("gemini l2_updates: missing changes".into()))?;

    let mut bids = Vec::new();
    let mut asks = Vec::new();

    for change in changes {
        let arr = match change.as_array() {
            Some(a) if a.len() >= 3 => a,
            _ => continue,
        };
        let side = arr[0].as_str().unwrap_or("");
        let price = parse_f64_any(&arr[1]).unwrap_or(0.0);
        let qty = parse_f64_any(&arr[2]).unwrap_or(0.0);

        match side {
            "buy" => bids.push(OrderBookLevel::new(price, qty)),
            "sell" => asks.push(OrderBookLevel::new(price, qty)),
            _ => {}
        }
    }

    let timestamp = raw
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    let symbol = raw
        .get("symbol")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    Ok(StreamEvent::OrderbookDelta {
        symbol,
        delta: OrderbookDelta {
            bids,
            asks,
            timestamp,
            first_update_id: None,
            last_update_id: None,
            prev_update_id: None,
            event_time: None,
            checksum: None,
        },
    })
}

/// Parse `l2_updates` frame → StreamEvent::Trade (from the `trades` array).
///
/// A single l2_updates frame may carry zero or more trades. This parser emits
/// the **last** trade entry (matching the bespoke loop behaviour). When no
/// trades are present the frame is a pure book-delta; return a Parse error so
/// the registry dispatcher drops this invocation silently.
pub(crate) fn parse_l2_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let trades = raw
        .get("trades")
        .and_then(|t| t.as_array())
        .ok_or_else(|| WebSocketError::Parse("gemini l2_updates: no trades array".into()))?;

    let trade_val = trades
        .last()
        .ok_or_else(|| WebSocketError::Parse("gemini l2_updates: empty trades array".into()))?;

    let price = parse_f64_any(trade_val.get("price").unwrap_or(&Value::Null))
        .ok_or_else(|| WebSocketError::Parse("gemini trade: missing price".into()))?;

    let quantity = parse_f64_any(trade_val.get("amount").unwrap_or(&Value::Null))
        .or_else(|| parse_f64_any(trade_val.get("quantity").unwrap_or(&Value::Null)))
        .unwrap_or(0.0);

    let timestamp = trade_val
        .get("timestampms")
        .and_then(|v| v.as_i64())
        .or_else(|| trade_val.get("timestamp").and_then(|v| v.as_i64()))
        .unwrap_or(0);

    let id = trade_val
        .get("tid")
        .and_then(|v| v.as_i64())
        .or_else(|| trade_val.get("event_id").and_then(|v| v.as_i64()))
        .map(|n| n.to_string())
        .unwrap_or_default();

    // "makerSide":"bid" → maker was buyer → taker sold → Sell
    // "makerSide":"ask" → maker was seller → taker bought → Buy
    let side = match trade_val.get("makerSide").and_then(|v| v.as_str()) {
        Some("bid") => TradeSide::Sell,
        Some("ask") => TradeSide::Buy,
        _ => match trade_val.get("side").and_then(|v| v.as_str()) {
            Some("sell") => TradeSide::Sell,
            _ => TradeSide::Buy,
        },
    };

    let symbol = raw
        .get("symbol")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    Ok(StreamEvent::Trade {
        symbol,
        trade: PublicTrade {
            id,
            price,
            quantity,
            side,
            timestamp,
        },
    })
}

/// Parse `l2_updates` frame → synthetic StreamEvent::Ticker.
///
/// Gemini has no native ticker channel. This parser accumulates top-of-book
/// state (best bid = highest buy, best ask = lowest sell) and last trade price
/// across frames in a process-wide map keyed by symbol. Once both bid and ask
/// are known it emits a `StreamEvent::Ticker`.
///
/// Returns `WebSocketError::FieldAbsent` (silent skip, NOT a hard error) when:
/// - The frame carries no `changes` and no `trades` (pure heartbeat delta).
/// - The accumulated state is not yet sufficient (bid or ask unknown).
pub(crate) fn parse_l2_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let symbol = raw
        .get("symbol")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    let ts = raw
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    let mut state_map = book_state()
        .lock()
        .expect("gemini book_state poisoned");
    let state = state_map
        .entry(symbol.clone())
        .or_insert_with(GeminiTickerState::new);

    // Update top-of-book from changes array.
    // Each entry is ["buy"|"sell", price, qty]. qty == "0" means removal.
    // We track the single best bid / best ask heuristically:
    //   - For buys: new price > current best_bid → update.
    //   - For sells: new price < current best_ask → update.
    // On removal (qty == 0) of the current best, we reset to unknown so the
    // next matching change re-establishes it. This gives approximate top-of-book
    // without maintaining a full sorted book.
    if let Some(changes) = raw.get("changes").and_then(|c| c.as_array()) {
        for change in changes {
            let arr = match change.as_array() {
                Some(a) if a.len() >= 3 => a,
                _ => continue,
            };
            let side = arr[0].as_str().unwrap_or("");
            let price = match parse_f64_any(&arr[1]) {
                Some(p) => p,
                None => continue,
            };
            let qty = parse_f64_any(&arr[2]).unwrap_or(0.0);

            match side {
                "buy" => {
                    if qty == 0.0 {
                        // Removal: reset best_bid if it matched this price.
                        if (state.best_bid - price).abs() < f64::EPSILON {
                            state.best_bid = f64::NEG_INFINITY;
                        }
                    } else if price > state.best_bid {
                        state.best_bid = price;
                    }
                }
                "sell" => {
                    if qty == 0.0 {
                        // Removal: reset best_ask if it matched this price.
                        if (state.best_ask - price).abs() < f64::EPSILON {
                            state.best_ask = f64::INFINITY;
                        }
                    } else if price < state.best_ask {
                        state.best_ask = price;
                    }
                }
                _ => {}
            }
        }
    }

    // Update last trade price from trades array.
    if let Some(trades) = raw.get("trades").and_then(|t| t.as_array()) {
        if let Some(last_trade) = trades.last() {
            if let Some(p) = parse_f64_any(last_trade.get("price").unwrap_or(&Value::Null)) {
                if p > 0.0 {
                    state.last_trade = p;
                }
            }
        }
    }

    state.last_ts = ts;

    // Emit only once we have a valid bid+ask.
    if !state.has_bid_ask() {
        return Err(WebSocketError::FieldAbsent(
            "gemini ticker: top-of-book not yet established".into(),
        ));
    }

    let best_bid = state.best_bid;
    let best_ask = state.best_ask;
    let last_trade = state.last_trade;

    // Use last_trade as last_price; fall back to mid if no trade seen yet.
    let last_price = if last_trade > 0.0 {
        last_trade
    } else {
        (best_bid + best_ask) / 2.0
    };

    let ticker = Ticker {
        last_price,
        bid_price: Some(best_bid),
        ask_price: Some(best_ask),
        high_24h: None,
        low_24h: None,
        volume_24h: None,
        quote_volume_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: ts,
    };

    Ok(StreamEvent::Ticker { symbol, ticker })
}

/// Parse `candles_*_updates` frame → StreamEvent::Kline.
///
/// Gemini candle frame shape:
/// ```json
/// {"type":"candles_1m_updates","symbol":"BTCUSD","changes":[[ts,open,high,low,close,vol]]}
/// ```
/// Interval is extracted from the `type` field.
pub(crate) fn parse_candle(raw: &Value) -> WebSocketResult<StreamEvent> {
    let msg_type = raw
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("candles__updates");

    // "candles_1m_updates" → strip "candles_" prefix and "_updates" suffix
    let interval_str = msg_type
        .strip_prefix("candles_")
        .unwrap_or("")
        .strip_suffix("_updates")
        .unwrap_or("");
    let interval = KlineInterval::new(interval_str);

    let changes = raw
        .get("changes")
        .and_then(|c| c.as_array())
        .ok_or_else(|| WebSocketError::Parse("gemini candle: missing changes".into()))?;

    let candle = changes
        .first()
        .and_then(|c| c.as_array())
        .ok_or_else(|| WebSocketError::Parse("gemini candle: empty/invalid changes".into()))?;

    if candle.len() < 6 {
        return Err(WebSocketError::Parse(
            "gemini candle: expected 6 elements [ts,o,h,l,c,v]".into(),
        ));
    }

    let open_time = parse_f64_any(&candle[0])
        .map(|t| t as i64)
        .unwrap_or(0);

    let kline = Kline {
        open_time,
        open: parse_f64_any(&candle[1]).unwrap_or(0.0),
        high: parse_f64_any(&candle[2]).unwrap_or(0.0),
        low: parse_f64_any(&candle[3]).unwrap_or(0.0),
        close: parse_f64_any(&candle[4]).unwrap_or(0.0),
        volume: parse_f64_any(&candle[5]).unwrap_or(0.0),
        quote_volume: None,
        close_time: None,
        trades: None,
    };

    let symbol = raw
        .get("symbol")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    Ok(StreamEvent::Kline { symbol, interval, kline })
}

/// Parse `auction_*` frame → StreamEvent::AuctionEvent.
///
/// Auction types: `auction_open`, `auction_indicative_price`,
/// `auction_result`, `auction_outcome`.
pub(crate) fn parse_auction(raw: &Value) -> WebSocketResult<StreamEvent> {
    let state = raw
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("gemini auction: missing type".into()))?
        .to_string();

    let symbol = raw
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let auction_id = raw
        .get("auction_id")
        .and_then(|v| v.as_i64())
        .map(|id| id.to_string())
        .unwrap_or_default();

    let indicative_price = raw
        .get("indicative_price")
        .or_else(|| raw.get("price"))
        .and_then(|v| parse_f64_any(v));

    let indicative_qty = raw
        .get("indicative_quantity")
        .or_else(|| raw.get("quantity"))
        .and_then(|v| parse_f64_any(v));

    let timestamp = raw
        .get("timestampms")
        .and_then(|v| v.as_i64())
        .or_else(|| raw.get("timestamp").and_then(|v| v.as_i64()).map(|s| s * 1000))
        .unwrap_or(0);

    Ok(StreamEvent::AuctionEvent {
        symbol,
        auction_id,
        indicative_price,
        indicative_qty,
        state,
        timestamp,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a JSON value that is either a JSON number or a numeric string to f64.
fn parse_f64_any(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
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

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_returns_none() {
        // CRITICAL: Gemini disconnects on client Ping — must remain None.
        let proto = GeminiProtocol;
        assert!(
            proto.ping_frame().is_none(),
            "ping_frame must return None — Gemini disconnects on client Ping"
        );
    }

    // ── endpoint ──────────────────────────────────────────────────────────────

    #[test]
    fn endpoint_mainnet() {
        let proto = GeminiProtocol;
        let url = proto.endpoint(AccountType::Spot, false);
        assert_eq!(url.as_str(), "wss://api.gemini.com/v2/marketdata");
    }

    #[test]
    fn endpoint_testnet() {
        let proto = GeminiProtocol;
        let url = proto.endpoint(AccountType::Spot, true);
        assert_eq!(url.as_str(), "wss://api.sandbox.gemini.com/v2/marketdata");
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_orderbook() {
        let proto = GeminiProtocol;
        let spec = make_spec(StreamKind::Orderbook, "BTCUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["type"], "subscribe");
            let subs = &v["subscriptions"][0];
            assert_eq!(subs["name"], "l2");
            assert_eq!(subs["symbols"][0], "BTCUSD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_trade() {
        let proto = GeminiProtocol;
        let spec = make_spec(StreamKind::Trade, "ETHUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["subscriptions"][0]["name"], "l2");
            assert_eq!(v["subscriptions"][0]["symbols"][0], "ETHUSD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_kline() {
        let proto = GeminiProtocol;
        let spec = make_spec(StreamKind::Kline { interval: KlineInterval::new("1m") }, "BTCUSD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["subscriptions"][0]["name"], "candles_1m");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_ticker_sends_l2_subscription() {
        // Ticker is now implemented via l2 synthesis — subscribe_frame must succeed.
        let proto = GeminiProtocol;
        let spec = make_spec(StreamKind::Ticker, "BTCUSD");
        let frame = proto.subscribe_frame(&spec).expect("Ticker subscribe_frame must succeed");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["type"], "subscribe");
            // Ticker rides the l2 channel.
            assert_eq!(v["subscriptions"][0]["name"], "l2");
            assert_eq!(v["subscriptions"][0]["symbols"][0], "BTCUSD");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn extract_topic_l2_updates() {
        let proto = GeminiProtocol;
        let raw = serde_json::json!({"type": "l2_updates", "symbol": "BTCUSD", "changes": []});
        assert_eq!(proto.extract_topic(&raw), Some(TopicKey::new("l2_updates")));
    }

    #[test]
    fn extract_topic_candles() {
        let proto = GeminiProtocol;
        let raw = serde_json::json!({"type": "candles_1m_updates", "symbol": "BTCUSD", "changes": []});
        assert_eq!(proto.extract_topic(&raw), Some(TopicKey::new("candles_updates")));
    }

    #[test]
    fn extract_topic_auction_indicative() {
        let proto = GeminiProtocol;
        let raw = serde_json::json!({"type": "auction_indicative_price", "symbol": "BTCUSD"});
        assert_eq!(proto.extract_topic(&raw), Some(TopicKey::new("auction")));
    }

    #[test]
    fn extract_topic_heartbeat_returns_none() {
        let proto = GeminiProtocol;
        let raw = serde_json::json!({"type": "heartbeat"});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_subscribed_returns_none() {
        let proto = GeminiProtocol;
        let raw = serde_json::json!({"type": "subscribed", "subscriptions": []});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    // ── is_subscribe_ack ──────────────────────────────────────────────────────

    #[test]
    fn is_subscribe_ack_for_subscribed() {
        let proto = GeminiProtocol;
        let raw = serde_json::json!({"type": "subscribed", "subscriptions": []});
        assert!(proto.is_subscribe_ack(&raw));
    }

    #[test]
    fn is_subscribe_ack_false_for_data() {
        let proto = GeminiProtocol;
        let raw = serde_json::json!({"type": "l2_updates", "symbol": "BTCUSD"});
        assert!(!proto.is_subscribe_ack(&raw));
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_covers_public_channels() {
        let proto = GeminiProtocol;
        let reg = proto.topic_registry(AccountType::Spot);
        let at = AccountType::Spot;
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook");
        assert!(reg.supports(&StreamKind::Trade, at), "Trade");
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker");
        assert!(
            reg.supports(&StreamKind::Kline { interval: KlineInterval::new("") }, at),
            "Kline"
        );
        assert!(reg.supports(&StreamKind::AuctionEvent, at), "AuctionEvent");
    }

    #[test]
    fn l2_updates_channel_has_three_parsers() {
        // Orderbook, Trade, and Ticker are all registered on "l2_updates".
        let proto = GeminiProtocol;
        let reg = proto.topic_registry(AccountType::Spot);
        let key = TopicKey::new("l2_updates");
        let parsers = reg.dispatch_all(&key);
        assert_eq!(
            parsers.len(),
            3,
            "l2_updates must have 3 parsers (Orderbook + Trade + Ticker)"
        );
    }

    // ── parse_l2_orderbook ────────────────────────────────────────────────────

    #[test]
    fn parse_l2_orderbook_basic() {
        let raw = serde_json::json!({
            "type": "l2_updates",
            "symbol": "BTCUSD",
            "changes": [
                ["buy",  "50000.00", "1.5"],
                ["sell", "50001.00", "0.8"]
            ]
        });
        let ev = parse_l2_orderbook(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookDelta { symbol, delta } => {
                assert_eq!(symbol, "BTCUSD");
                assert_eq!(delta.bids.len(), 1);
                assert_eq!(delta.asks.len(), 1);
                assert!((delta.bids[0].price - 50000.0).abs() < f64::EPSILON);
            }
            other => panic!("expected OrderbookDelta, got {:?}", other),
        }
    }

    #[test]
    fn parse_l2_orderbook_uses_receive_time_when_no_timestamp() {
        let raw = serde_json::json!({
            "type": "l2_updates",
            "symbol": "BTCUSD",
            "changes": [["buy", "50000.00", "1.0"]]
        });
        let ev = parse_l2_orderbook(&raw).expect("parse");
        if let StreamEvent::OrderbookDelta { delta, .. } = ev {
            assert!(delta.timestamp > 0, "timestamp must be non-zero (receive time)");
        }
    }

    // ── parse_l2_trade ────────────────────────────────────────────────────────

    #[test]
    fn parse_l2_trade_missing_trades_returns_err() {
        let raw = serde_json::json!({
            "type": "l2_updates",
            "symbol": "BTCUSD",
            "changes": [["buy", "50000.00", "1.0"]]
        });
        assert!(parse_l2_trade(&raw).is_err());
    }

    #[test]
    fn parse_l2_trade_maker_bid_is_sell() {
        let raw = serde_json::json!({
            "type": "l2_updates",
            "symbol": "BTCUSD",
            "changes": [],
            "trades": [{
                "type": "trade",
                "tid": 12345,
                "price": "50000.00",
                "amount": "0.5",
                "makerSide": "bid",
                "timestampms": 1700000000000i64
            }]
        });
        let ev = parse_l2_trade(&raw).expect("parse");
        match ev {
            StreamEvent::Trade { symbol, trade } => {
                assert_eq!(symbol, "BTCUSD");
                assert_eq!(trade.side, TradeSide::Sell);
                assert!((trade.price - 50000.0).abs() < f64::EPSILON);
            }
            other => panic!("expected Trade, got {:?}", other),
        }
    }

    // ── parse_candle ──────────────────────────────────────────────────────────

    #[test]
    fn parse_candle_1m() {
        let raw = serde_json::json!({
            "type": "candles_1m_updates",
            "symbol": "BTCUSD",
            "changes": [[1700000000000i64, "50000", "51000", "49000", "50500", "100.5"]]
        });
        let ev = parse_candle(&raw).expect("parse");
        match ev {
            StreamEvent::Kline { symbol, interval, kline } => {
                assert_eq!(symbol, "BTCUSD");
                assert_eq!(interval.as_str(), "1m");
                assert!((kline.open - 50000.0).abs() < f64::EPSILON);
                assert!((kline.volume - 100.5).abs() < f64::EPSILON);
            }
            other => panic!("expected Kline, got {:?}", other),
        }
    }

    // ── parse_auction ─────────────────────────────────────────────────────────

    #[test]
    fn parse_auction_indicative_price() {
        let raw = serde_json::json!({
            "type": "auction_indicative_price",
            "symbol": "BTCUSD",
            "auction_id": 42,
            "indicative_price": "50080",
            "indicative_quantity": "1.5",
            "timestampms": 1609459200000i64
        });
        let ev = parse_auction(&raw).expect("parse");
        match ev {
            StreamEvent::AuctionEvent { symbol, auction_id, indicative_price, state, .. } => {
                assert_eq!(symbol, "BTCUSD");
                assert_eq!(auction_id, "42");
                assert_eq!(state, "auction_indicative_price");
                assert!((indicative_price.unwrap() - 50080.0).abs() < f64::EPSILON);
            }
            other => panic!("expected AuctionEvent, got {:?}", other),
        }
    }
}
