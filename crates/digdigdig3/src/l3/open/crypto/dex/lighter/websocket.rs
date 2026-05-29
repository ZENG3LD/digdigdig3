//! LighterWebSocket — thin wrapper around UniversalWsTransport<LighterProtocol>.
//!
//! ## Public data only
//!
//! Lighter public channels (orderbook, trades, market stats, ticker) require
//! zero authentication. Private channels (account_all, account_market) are
//! NOT supported through this connector by design — they are native-only
//! and require ECDSA wallet signing.
//!
//! ## Wasm support
//!
//! `LighterProtocol` passes the standard WS transport requirements (text frames,
//! no binary frames, standard Ping/Pong). `UniversalWsTransport` compiles to
//! wasm32 via `web-sys`. No `#[cfg(not(target_arch = "wasm32"))]` gates needed.
//!
//! ## Topic routing
//!
//! Topics use `"<type_field>:<market_id>"` string keys.
//! E.g. `"update/order_book:0"`, `"update/trade:1"`.

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::core::traits::WebSocketConnector;
use crate::core::types::{
    AccountType, ConnectionStatus, Kline, OrderBook, OrderbookCapabilities, PublicTrade,
    StreamEvent, SubscriptionRequest, Ticker, TradeSide, WebSocketResult,
    WsBookChannel,
};
use crate::core::types::OrderBookLevel;
use crate::core::websocket::{KlineInterval, StreamSpec, UniversalWsTransport};

use super::protocol::LighterProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// LighterWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Lighter DEX WebSocket connector backed by UniversalWsTransport.
///
/// Public market-data only. Compiles to wasm32 without any cfg gates.
pub struct LighterWebSocket {
    inner: UniversalWsTransport<LighterProtocol>,
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl LighterWebSocket {
    /// Create a new connector. Does NOT connect yet.
    pub fn new(testnet: bool, account_type: AccountType) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                LighterProtocol::new(testnet),
                account_type,
                testnet,
                None, // public streams only — no credentials
            ),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a public connector (alias for `new`).
    pub fn public(testnet: bool) -> Self {
        Self::new(testnet, AccountType::FuturesCross)
    }
}

impl Default for LighterWebSocket {
    fn default() -> Self {
        Self::new(false, AccountType::FuturesCross)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for LighterWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        self.inner.connect().await
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        self.inner.disconnect().await
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.inner.connection_status()
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.inner.subscribe(spec).await
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.inner.unsubscribe(spec).await
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        Box::pin(self.inner.event_stream())
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.inner
            .active_subscriptions()
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect()
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        // Lighter uses protocol-level Ping/Pong (no application-level ping).
        Some(Arc::clone(&self.ws_ping_rtt_ms))
    }

    /// Lighter orderbook capabilities.
    ///
    /// Single channel `order_book/{market}`: snapshot on first subscribe, then
    /// incremental deltas batched every 50 ms. No configurable depth or speed.
    /// Carries `nonce` (current sequence) and `begin_nonce` (previous sequence)
    /// enabling in-message gap detection. Same mechanics for perp markets.
    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("order_book", None, Some(50)),
        ];
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: Some(250),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[50],
            default_speed_ms: Some(50),
            ws_channels: CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: true,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions
//
// Parsing logic factored from the original bespoke websocket.rs.
// These are called by protocol.rs registry bridge functions (wrap_*).
// ─────────────────────────────────────────────────────────────────────────────

// ── Value helpers ──────────────────────────────────────────────────────────

fn val_f64(obj: &Value, field: &str) -> Option<f64> {
    obj.get(field).and_then(|v| {
        v.as_str().and_then(|s| s.parse::<f64>().ok())
            .or_else(|| v.as_f64())
    })
}

fn val_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
    obj.get(field).and_then(|v| v.as_str())
}

fn val_i64(obj: &Value, field: &str) -> Option<i64> {
    obj.get(field).and_then(|v| v.as_i64())
}

fn val_u64(obj: &Value, field: &str) -> Option<u64> {
    obj.get(field).and_then(|v| v.as_u64())
}

fn val_bool(obj: &Value, field: &str) -> Option<bool> {
    obj.get(field).and_then(|v| v.as_bool())
}

fn json_val_to_f64(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
}

/// Parse price/size levels from a JSON array into `OrderBookLevel` list.
///
/// Supports both object format `{"price":"2738","size":"1.5"}` and
/// legacy array format `["2738","1.5"]`.
fn parse_levels(arr: &Value) -> Vec<OrderBookLevel> {
    arr.as_array()
        .map(|levels| {
            levels.iter().filter_map(|entry| {
                if let Some(obj) = entry.as_object() {
                    let price = obj.get("price").and_then(json_val_to_f64)?;
                    let size = obj.get("size").and_then(json_val_to_f64)?;
                    Some(OrderBookLevel::new(price, size))
                } else if let Some(pair) = entry.as_array() {
                    if pair.len() >= 2 {
                        let price = json_val_to_f64(&pair[0])?;
                        let size = json_val_to_f64(&pair[1])?;
                        Some(OrderBookLevel::new(price, size))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect()
        })
        .unwrap_or_default()
}

/// Normalize a timestamp from Lighter.
///
/// Lighter sends seconds-precision timestamps (~10 digits).
/// Multiply by 1000 to convert to milliseconds.
fn normalize_ts(ts: i64) -> i64 {
    if ts > 0 && ts < 1_000_000_000_000 {
        ts * 1000
    } else {
        ts
    }
}

// ── Public parse fns (used by protocol.rs registry bridges) ───────────────

/// Parse `update/order_book` frame → `StreamEvent::OrderbookSnapshot`.
///
/// Lighter sends either:
/// 1. Flat top-level: `{"asks":[...],"bids":[...],"nonce":N,"timestamp":T}`
/// 2. Nested `"order_book"` object: `{"order_book":{"asks":...,"bids":...}}`
pub(super) fn parse_orderbook(raw: &Value, _channel: &str) -> Option<StreamEvent> {
    let data = raw.get("order_book").unwrap_or(raw);

    let asks = data.get("asks").map(parse_levels).unwrap_or_default();
    let bids = data.get("bids").map(parse_levels).unwrap_or_default();

    if asks.is_empty() && bids.is_empty() {
        return None;
    }

    let timestamp_raw = val_i64(raw, "timestamp")
        .or_else(|| val_i64(data, "timestamp"))
        .unwrap_or(0);
    let timestamp = normalize_ts(timestamp_raw);

    let sequence = val_i64(data, "nonce")
        .or_else(|| val_i64(raw, "nonce"))
        .map(|n| n.to_string());

    Some(StreamEvent::OrderbookSnapshot {
        symbol: String::new(), // transport overwrites via relay
        book: OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        },
    })
}

/// Parse `update/trade` frame → vec of `StreamEvent::Trade`.
///
/// Live format uses `"trades"` plural array. Legacy format uses singular `"trade"`.
pub(super) fn parse_trade(raw: &Value, channel: &str) -> Vec<StreamEvent> {
    use super::protocol::extract_market_id_from_channel;
    let market_id = extract_market_id_from_channel(channel);

    let parse_one = |entry: &Value| -> Option<StreamEvent> {
        let price = val_f64(entry, "price")?;
        let quantity = val_f64(entry, "size")?;
        let timestamp_raw = val_i64(entry, "timestamp")
            .or_else(|| val_i64(raw, "timestamp"))
            .unwrap_or(0);
        let timestamp = normalize_ts(timestamp_raw);
        let trade_id = val_u64(entry, "trade_id").unwrap_or(0);

        let side = if let Some(side_str) = val_str(entry, "side") {
            match side_str {
                "buy" => TradeSide::Buy,
                "sell" => TradeSide::Sell,
                _ => {
                    if val_bool(entry, "is_maker_ask").unwrap_or(false) {
                        TradeSide::Buy
                    } else {
                        TradeSide::Sell
                    }
                }
            }
        } else if val_bool(entry, "is_maker_ask").unwrap_or(false) {
            TradeSide::Buy
        } else {
            TradeSide::Sell
        };

        Some(StreamEvent::Trade {
            symbol: market_id.to_string(),
            trade: PublicTrade {
                id: trade_id.to_string(),
                price,
                quantity,
                side,
                timestamp,
            },
        })
    };

    // Primary: "trades" array (live Lighter WS format)
    if let Some(arr) = raw.get("trades").and_then(|v| v.as_array()) {
        return arr.iter().filter_map(parse_one).collect();
    }

    // Fallback: singular "trade" object
    if let Some(entry) = raw.get("trade") {
        return parse_one(entry).into_iter().collect();
    }

    Vec::new()
}

/// Parse `update/market_stats` frame → `StreamEvent::Ticker`.
///
/// Data is nested inside `"market_stats"` object. Falls back to top-level.
pub(super) fn parse_market_stats(raw: &Value, channel: &str) -> Option<StreamEvent> {
    use super::protocol::extract_market_id_from_channel;
    let data = raw.get("market_stats").unwrap_or(raw);

    let last_price = val_f64(data, "last_trade_price")
        .or_else(|| val_f64(data, "last_price"))
        .or_else(|| val_f64(data, "mark_price"))?;

    let market_id = extract_market_id_from_channel(channel);
    let symbol_name = val_str(data, "symbol").unwrap_or(market_id);

    let high_24h = val_f64(data, "daily_price_high").or_else(|| val_f64(data, "daily_high"));
    let low_24h = val_f64(data, "daily_price_low").or_else(|| val_f64(data, "daily_low"));
    let volume_24h = val_f64(data, "daily_volume").or_else(|| val_f64(data, "daily_base_token_volume"));
    let price_change_24h = val_f64(data, "daily_price_change").or_else(|| val_f64(data, "daily_change"));

    let timestamp = val_i64(raw, "timestamp")
        .or_else(|| val_i64(data, "timestamp"))
        .unwrap_or(0);

    let price_change_percent_24h = price_change_24h.and_then(|change| {
        let open = last_price - change;
        if open.abs() > 1e-10 {
            Some((change / open) * 100.0)
        } else {
            None
        }
    });

    Some(StreamEvent::Ticker {
        symbol: symbol_name.to_string(),
        ticker: Ticker {
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h,
            price_change_percent_24h,
            timestamp,
        },
    })
}

/// Parse `update/ticker` frame → `StreamEvent::Ticker`.
///
/// Delivers lightweight best-bid/ask from `ticker.b` / `ticker.a` sub-objects.
/// Timestamp in `last_updated_at` is microseconds — divide by 1000 for ms.
pub(super) fn parse_ticker_channel(raw: &Value, channel: &str) -> Option<StreamEvent> {
    use super::protocol::extract_market_id_from_channel;
    let data = raw.get("ticker").unwrap_or(raw);

    let ask_price = data.get("a")
        .and_then(|a| a.get("price"))
        .and_then(json_val_to_f64);
    let bid_price = data.get("b")
        .and_then(|b| b.get("price"))
        .and_then(json_val_to_f64);

    let last_price = match (bid_price, ask_price) {
        (Some(b), Some(a)) => (b + a) / 2.0,
        (Some(b), None) => b,
        (None, Some(a)) => a,
        (None, None) => return None,
    };

    let market_id = extract_market_id_from_channel(channel);
    let symbol_name = val_str(data, "s").unwrap_or(market_id);

    // Lighter returns microseconds — divide by 1000 to get ms.
    let timestamp = val_i64(data, "last_updated_at")
        .or_else(|| val_i64(raw, "last_updated_at"))
        .map(|us| us / 1000)
        .unwrap_or(0);

    Some(StreamEvent::Ticker {
        symbol: symbol_name.to_string(),
        ticker: Ticker {
            last_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        },
    })
}

/// Parse `update/candle` or `subscribed/candle` frame → `StreamEvent::Kline`.
///
/// Lighter sends candle data inside a `"candles"` array with short field names.
/// Frame shape (live verified 2026-05-29):
/// ```json
/// {
///   "type": "update/candle",
///   "channel": "candle:1:1m",
///   "timestamp": 1780012392587,
///   "candles": [
///     {
///       "t": 1780012380000,
///       "o": 73517.4,
///       "h": 73522.1,
///       "l": 73517.4,
///       "c": 73520.5,
///       "v": 0.14261,
///       "V": 10485.53,
///       "i": 21019917923
///     }
///   ]
/// }
/// ```
///
/// `t` = open_time in milliseconds (already ms — no scaling needed).
/// `o`/`h`/`l`/`c` = OHLC as f64. `v` = base volume, `V` = quote volume.
///
/// `resolution` is passed from the thread-local set by `extract_topic`.
pub(super) fn parse_candle(raw: &Value, channel: &str, resolution: &str) -> Option<StreamEvent> {
    use super::protocol::extract_market_id_from_channel;

    // Primary: "candles" array with short-field candle objects (live format).
    let candles_arr = raw.get("candles").and_then(|v| v.as_array())?;
    let entry = candles_arr.first()?;

    // Short field names: t, o, h, l, c, v, V
    let open_time = entry.get("t").and_then(|v| v.as_i64())?;
    let open = entry.get("o").and_then(|v| v.as_f64())?;
    let high = entry.get("h").and_then(|v| v.as_f64())?;
    let low = entry.get("l").and_then(|v| v.as_f64())?;
    let close = entry.get("c").and_then(|v| v.as_f64())?;
    let volume = entry.get("v").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let quote_volume = entry.get("V").and_then(|v| v.as_f64());

    // Channel format for candles: "candle:{market_id}:{resolution}" (3 parts).
    // `extract_market_id_from_channel` takes the LAST segment, which for a 3-part
    // candle channel gives the resolution, not the market_id. Extract market_id
    // explicitly as the second segment.
    let market_id = {
        let sep = if channel.contains(':') { ':' } else { '/' };
        let mut parts = channel.splitn(3, sep);
        parts.next(); // "candle"
        parts.next().unwrap_or_else(|| extract_market_id_from_channel(channel))
    };

    Some(StreamEvent::Kline {
        symbol: market_id.to_string(),
        interval: KlineInterval::new(resolution),
        kline: Kline {
            // open_time is already in milliseconds (live-verified).
            open_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume,
            close_time: None,
            trades: None,
        },
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn construction_is_disconnected() {
        let ws = LighterWebSocket::new(false, AccountType::FuturesCross);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_disconnected() {
        let ws = LighterWebSocket::default();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn public_is_disconnected() {
        let ws = LighterWebSocket::public(false);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    // ── parse_orderbook ───────────────────────────────────────────────────────

    #[test]
    fn parse_orderbook_flat_format() {
        let raw = serde_json::json!({
            "type": "update/order_book",
            "channel": "order_book:0",
            "asks": [{"price":"3024.66","size":"1.5"}],
            "bids": [{"price":"3024.00","size":"2.0"}],
            "nonce": 12345,
            "timestamp": 1700000000
        });
        let event = parse_orderbook(&raw, "order_book:0").expect("should parse");
        if let StreamEvent::OrderbookSnapshot { book, .. } = event {
            assert_eq!(book.asks.len(), 1);
            assert_eq!(book.bids.len(), 1);
            assert!((book.asks[0].price - 3024.66).abs() < 1e-6);
            assert!((book.bids[0].price - 3024.00).abs() < 1e-6);
            assert_eq!(book.timestamp, 1700000000 * 1000);
            assert_eq!(book.sequence.as_deref(), Some("12345"));
        } else {
            panic!("expected OrderbookSnapshot");
        }
    }

    #[test]
    fn parse_orderbook_empty_returns_none() {
        let raw = serde_json::json!({
            "type": "update/order_book",
            "channel": "order_book:0",
            "asks": [],
            "bids": []
        });
        assert!(parse_orderbook(&raw, "order_book:0").is_none());
    }

    // ── parse_trade ───────────────────────────────────────────────────────────

    #[test]
    fn parse_trade_plural_array() {
        let raw = serde_json::json!({
            "type": "update/trade",
            "channel": "trade:1",
            "trades": [
                {"trade_id":1,"price":"76500","size":"0.1","side":"buy","timestamp":1700000000}
            ]
        });
        let events = parse_trade(&raw, "trade:1");
        assert_eq!(events.len(), 1);
        if let StreamEvent::Trade { trade, .. } = &events[0] {
            assert!((trade.price - 76500.0).abs() < 1e-6);
            assert_eq!(trade.side, TradeSide::Buy);
        } else {
            panic!("expected Trade");
        }
    }

    #[test]
    fn parse_trade_empty_array() {
        let raw = serde_json::json!({
            "type": "update/trade",
            "channel": "trade:1",
            "trades": []
        });
        assert!(parse_trade(&raw, "trade:1").is_empty());
    }

    // ── parse_ticker_channel ──────────────────────────────────────────────────

    // ── parse_candle ──────────────────────────────────────────────────────────

    /// Sample frame captured live from Lighter WS on 2026-05-29.
    /// Short field names: t=open_time (ms), o/h/l/c=OHLC, v=base_vol, V=quote_vol, i=trade_id.
    #[test]
    fn parse_candle_live_format() {
        let raw = serde_json::json!({
            "type": "update/candle",
            "channel": "candle:1:1m",
            "timestamp": 1780012392587i64,
            "candles": [
                {
                    "t": 1780012380000i64,
                    "o": 73517.4,
                    "h": 73522.1,
                    "l": 73517.4,
                    "c": 73520.5,
                    "v": 0.14261,
                    "V": 10485.53,
                    "i": 21019917923i64
                }
            ]
        });
        let event = parse_candle(&raw, "candle:1:1m", "1m").expect("should parse");
        if let StreamEvent::Kline { symbol, interval, kline } = event {
            assert_eq!(symbol, "1");
            assert_eq!(interval.as_str(), "1m");
            assert!((kline.open - 73517.4).abs() < 1e-3);
            assert!((kline.high - 73522.1).abs() < 1e-3);
            assert!((kline.low - 73517.4).abs() < 1e-3);
            assert!((kline.close - 73520.5).abs() < 1e-3);
            assert!((kline.volume - 0.14261).abs() < 1e-5);
            // open_time is already in milliseconds (no conversion)
            assert_eq!(kline.open_time, 1780012380000);
            assert!(kline.quote_volume.is_some());
        } else {
            panic!("expected Kline event, got {:?}", event);
        }
    }

    #[test]
    fn parse_candle_missing_candles_array_returns_none() {
        let raw = serde_json::json!({
            "type": "update/candle",
            "channel": "candle:1:1m",
        });
        assert!(parse_candle(&raw, "candle:1:1m", "1m").is_none());
    }

    #[test]
    fn parse_candle_empty_candles_array_returns_none() {
        let raw = serde_json::json!({
            "type": "update/candle",
            "channel": "candle:1:1m",
            "candles": []
        });
        assert!(parse_candle(&raw, "candle:1:1m", "1m").is_none());
    }

    #[test]
    fn parse_ticker_bid_ask_midpoint() {
        let raw = serde_json::json!({
            "channel": "ticker:0",
            "type": "update/ticker",
            "last_updated_at": 1700000000000000i64,
            "ticker": {
                "s": "ETH-PERP",
                "a": {"price": "3500.50", "size": "1.2"},
                "b": {"price": "3500.00", "size": "0.8"},
                "last_updated_at": 1700000000000000i64
            }
        });
        let event = parse_ticker_channel(&raw, "ticker:0").expect("should parse");
        if let StreamEvent::Ticker { symbol, ticker: t } = event {
            assert_eq!(symbol, "ETH-PERP");
            assert!((t.bid_price.unwrap() - 3500.00).abs() < 1e-6);
            assert!((t.ask_price.unwrap() - 3500.50).abs() < 1e-6);
            assert!((t.last_price - 3500.25).abs() < 1e-6);
            assert_eq!(t.timestamp, 1700000000000);
        } else {
            panic!("expected Ticker");
        }
    }
}
