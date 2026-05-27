//! CoinbaseProtocol — WsProtocol impl for Coinbase Advanced Trade WS.
//!
//! Public channels only (no JWT auth on wasm scope). Wire format:
//! - Public endpoint: `wss://advanced-trade-ws.coinbase.com`
//! - Private endpoint: `wss://advanced-trade-ws-user.coinbase.com`
//! - Subscribe: `{"type":"subscribe","channel":"ticker","product_ids":["BTC-USD"]}`
//! - On connect: must ALSO subscribe to "heartbeats" (connection-level keepalive)
//! - Ping: None (server sends heartbeats every ~1s, native WS pings handle RTT)
//! - Frame routing key: top-level "channel" field
//!
//! ### subscribe/receive channel asymmetry
//!
//! Client subscribes with `"channel": "level2"`.
//! Server responds with frames bearing `"channel": "l2_data"`.
//! `extract_topic` reads the *incoming* channel, so the registry registers
//! under `"l2_data"` — no mismatch.
//!
//! ### Heartbeats
//!
//! The thin `CoinbaseWebSocket` wrapper subscribes to "heartbeats" after
//! `inner.connect()`.  The protocol shim itself is stateless — it returns
//! `None` from `ping_frame()` and filters heartbeat frames in `extract_topic`.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::Value;
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, OrderBook, OrderBookLevel, OrderbookDelta, PublicTrade, StreamEvent, Ticker,
    TradeSide, WebSocketError, WebSocketResult,
};
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol,
};

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// CoinbaseProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Coinbase Advanced Trade WS protocol shim.
///
/// Handles public channels only (for wasm compatibility).
/// JWT-authenticated channels (private) remain available on native
/// via the inherent methods on `CoinbaseConnector`.
pub struct CoinbaseProtocol {
    /// Whether to use the authenticated private endpoint.
    pub(crate) use_private: bool,
}

impl CoinbaseProtocol {
    /// Public endpoint — no credentials needed.
    pub fn public() -> Self {
        Self { use_private: false }
    }

    /// Private endpoint — JWT auth per subscription.
    pub fn private() -> Self {
        Self { use_private: true }
    }

    /// Build subscribe or unsubscribe frame for a stream spec.
    fn build_frame(op: &str, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        // Resolve the product_id.  Coinbase uses hyphen-separated uppercase: "BTC-USD".
        // For Raw inputs the value passes through verbatim.
        // For Canonical inputs the normalizer produces the correct wire string.
        let product_id = spec
            .symbol
            .resolve(crate::core::types::ExchangeId::Coinbase, spec.account_type)
            .map_err(|e| {
                WebSocketError::NotSupported(format!(
                    "coinbase: symbol normalization failed: {}",
                    e
                ))
            })?;

        // Map StreamKind → Coinbase wire channel name.
        // Note: subscribe uses "level2"; server responds with "l2_data" (handled in extract_topic).
        let channel = match &spec.kind {
            StreamKind::Ticker => "ticker",
            StreamKind::Trade => "market_trades",
            StreamKind::Orderbook | StreamKind::OrderbookDelta => "level2",
            StreamKind::Kline { .. } => "candles",
            StreamKind::BlockTrade => "rfq_matches",
            other => {
                return Err(WebSocketError::NotSupported(format!(
                    "coinbase: no WS channel for {:?}",
                    other
                )));
            }
        };

        // Optional granularity for candles.
        let granularity: Option<&'static str> = if let StreamKind::Kline { interval } = &spec.kind {
            Some(map_kline_interval(interval))
        } else {
            None
        };

        let mut msg = serde_json::json!({
            "type": op,
            "channel": channel,
            "product_ids": [product_id],
        });

        if let Some(gran) = granularity {
            msg["granularity"] = serde_json::Value::String(gran.to_string());
        }

        Ok(WsFrame::Text(msg.to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for CoinbaseProtocol {
    fn name(&self) -> &'static str {
        "coinbase"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Coinbase has no testnet for Advanced Trade API.
        if self.use_private {
            Url::parse("wss://advanced-trade-ws-user.coinbase.com")
                .expect("coinbase private ws endpoint is valid")
        } else {
            Url::parse("wss://advanced-trade-ws.coinbase.com")
                .expect("coinbase public ws endpoint is valid")
        }
    }

    /// No application-level ping — Coinbase handles keepalive via the "heartbeats"
    /// channel (subscribed on connect by the thin wrapper).
    /// Native WebSocket ping frames are sent by the transport for RTT measurement.
    fn ping_frame(&self) -> Option<WsFrame> {
        None
    }

    fn ping_interval(&self) -> Duration {
        // Matches the bespoke loop's 5-second RTT cadence.
        // Effectively unused since ping_frame() returns None.
        Duration::from_secs(5)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    /// Public WS is unauthenticated. JWT auth for private channels is
    /// native-only and lives outside this shim.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    /// Subscribe to the Coinbase "heartbeats" channel on every connect/reconnect.
    ///
    /// Coinbase requires a heartbeats subscription for connection-level keepalive.
    /// The server sends heartbeat frames every ~1 second; these are filtered by
    /// `extract_topic` (returns None) so they never reach the broadcast channel.
    /// Without this subscription the server still delivers data, but the heartbeat
    /// health signal (and some server-side keepalive logic) is absent.
    ///
    /// Uses empty product_ids — heartbeats are connection-level, not per-product.
    fn post_connect_frames(&self) -> Vec<WsFrame> {
        let frame = serde_json::json!({
            "type": "subscribe",
            "channel": "heartbeats",
            "product_ids": []
        });
        vec![WsFrame::Text(frame.to_string())]
    }

    /// Coinbase has no application-level pong concept.
    /// Native WS Pong frames are handled transparently by the transport.
    fn is_pong(&self, _raw: &Value) -> bool {
        false
    }

    /// Coinbase confirms subscriptions by replaying the active subscription list
    /// in a frame with `"type": "subscriptions"`.
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        raw.get("type").and_then(|v| v.as_str()) == Some("subscriptions")
    }

    /// Extract routing topic from an incoming data frame.
    ///
    /// Returns `None` for:
    /// - `heartbeats` — server-side keepalive; no event to emit.
    /// - `subscriptions` — subscribe ACK; handled by `is_subscribe_ack`.
    /// - `status` — exchange status frames; not modelled.
    ///
    /// Returns `Some(TopicKey("l2_data"))` for level2 data frames (the server
    /// channel name, not the subscribe-time "level2" — registry uses "l2_data").
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let channel = raw.get("channel").and_then(|v| v.as_str())?;
        match channel {
            "heartbeats" | "subscriptions" | "status" => None,
            other => Some(TopicKey::new(other)),
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
        // ticker / ticker_batch → Ticker
        .register(StreamKind::Ticker, at, "ticker", parse_ticker)
        .register(StreamKind::Ticker, at, "ticker_batch", parse_ticker)
        // l2_data → OrderbookSnapshot or OrderbookDelta (dispatched internally by "type" field)
        .register(StreamKind::Orderbook, at, "l2_data", parse_l2_data)
        .register(StreamKind::OrderbookDelta, at, "l2_data", parse_l2_data)
        // market_trades → Trade
        .register(StreamKind::Trade, at, "market_trades", parse_market_trades)
        // candles → Kline
        .register(StreamKind::Kline { interval: KlineInterval::new("") }, at, "candles", parse_candles)
        // rfq_matches → BlockTrade (publicly visible RFQ block trades)
        .register(StreamKind::BlockTrade, at, "rfq_matches", parse_rfq_matches)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Parse RFC3339 timestamp string to milliseconds.
fn parse_rfc3339_ms(ts: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

/// Parse Unix seconds string to milliseconds.
fn parse_unix_seconds_ms(s: &str) -> Option<i64> {
    s.parse::<i64>().ok().map(|secs| secs * 1_000)
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions  (ParserFn = fn(&Value) -> WebSocketResult<StreamEvent>)
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `ticker` / `ticker_batch` frame → StreamEvent::Ticker.
///
/// Wire shape:
/// ```json
/// {
///   "channel": "ticker",
///   "timestamp": "2024-01-01T00:00:00Z",
///   "sequence_num": 1,
///   "events": [{
///     "type": "ticker",
///     "tickers": [{
///       "product_id": "BTC-USD",
///       "price": "50000.0",
///       "volume_24_h": "1234.5",
///       "high_24_h": "51000.0",
///       "low_24_h": "49000.0",
///       "best_bid": "49999.0",
///       "best_ask": "50001.0",
///       "price_percent_chg_24_h": "2.5"
///     }]
///   }]
/// }
/// ```
pub(crate) fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let events = raw
        .get("events")
        .and_then(|e| e.as_array())
        .ok_or_else(|| WebSocketError::Parse("ticker: missing events".into()))?;

    let event = events
        .first()
        .ok_or_else(|| WebSocketError::Parse("ticker: empty events".into()))?;

    let ticker_data = event
        .get("tickers")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| WebSocketError::Parse("ticker: missing tickers array".into()))?;

    let symbol = ticker_data
        .get("product_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let last_price = ticker_data
        .get("price")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("ticker: missing price".into()))?;

    let bid_price = ticker_data
        .get("best_bid")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let ask_price = ticker_data
        .get("best_ask")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let volume_24h = ticker_data
        .get("volume_24_h")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let high_24h = ticker_data
        .get("high_24_h")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let low_24h = ticker_data
        .get("low_24_h")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let price_change_percent_24h = ticker_data
        .get("price_percent_chg_24_h")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let timestamp = raw
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(parse_rfc3339_ms)
        .unwrap_or(0);

    Ok(StreamEvent::Ticker {
        symbol,
        ticker: Ticker {
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h,
            timestamp,
        },
    })
}

/// Parse `l2_data` frame → StreamEvent::OrderbookSnapshot or OrderbookDelta.
///
/// Coinbase emits both snapshot and delta frames on the same "l2_data" channel.
/// The sub-type is carried in `events[0].type`:
/// - `"snapshot"` → `OrderbookSnapshot` (full book, zero-qty levels excluded)
/// - `"update"`   → `OrderbookDelta`    (incremental; zero-qty = remove level)
///
/// Wire shape:
/// ```json
/// {
///   "channel": "l2_data",
///   "timestamp": "2024-01-01T00:00:00Z",
///   "sequence_num": 42,
///   "events": [{
///     "type": "snapshot",
///     "product_id": "BTC-USD",
///     "updates": [
///       {"side":"bid","price_level":"50000.0","new_quantity":"1.5"},
///       {"side":"offer","price_level":"50001.0","new_quantity":"0.5"}
///     ]
///   }]
/// }
/// ```
pub(crate) fn parse_l2_data(raw: &Value) -> WebSocketResult<StreamEvent> {
    let events = raw
        .get("events")
        .and_then(|e| e.as_array())
        .ok_or_else(|| WebSocketError::Parse("l2_data: missing events".into()))?;

    let event = events
        .first()
        .ok_or_else(|| WebSocketError::Parse("l2_data: empty events".into()))?;

    let event_type = event
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("snapshot");

    let symbol = event
        .get("product_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let timestamp = raw
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(parse_rfc3339_ms)
        .unwrap_or(0);

    let sequence_num = raw.get("sequence_num").and_then(|v| v.as_u64());

    let is_snapshot = event_type == "snapshot";
    let updates = event.get("updates").and_then(|u| u.as_array());

    let mut bids: Vec<OrderBookLevel> = Vec::new();
    let mut asks: Vec<OrderBookLevel> = Vec::new();

    if let Some(updates) = updates {
        for update in updates {
            let side = update.get("side").and_then(|s| s.as_str()).unwrap_or("");
            let price = update
                .get("price_level")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());
            let qty = update
                .get("new_quantity")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            if let (Some(price), Some(qty)) = (price, qty) {
                // For snapshots, skip zero-quantity levels (meaningless).
                if is_snapshot && qty == 0.0 {
                    continue;
                }
                match side {
                    "bid" => bids.push(OrderBookLevel::new(price, qty)),
                    "offer" | "ask" => asks.push(OrderBookLevel::new(price, qty)),
                    _ => {}
                }
            }
        }
    }

    if is_snapshot {
        Ok(StreamEvent::OrderbookSnapshot {
            symbol,
            book: OrderBook {
                bids,
                asks,
                timestamp,
                sequence: sequence_num.map(|n| n.to_string()),
                last_update_id: None,
                first_update_id: None,
                prev_update_id: None,
                event_time: None,
                transaction_time: None,
                checksum: None,
            },
        })
    } else {
        Ok(StreamEvent::OrderbookDelta {
            symbol,
            delta: OrderbookDelta {
                bids,
                asks,
                timestamp,
                first_update_id: sequence_num,
                last_update_id: sequence_num,
                prev_update_id: None,
                event_time: None,
                checksum: None,
            },
        })
    }
}

/// Parse `market_trades` frame → StreamEvent::Trade.
///
/// Wire shape:
/// ```json
/// {
///   "channel": "market_trades",
///   "events": [{
///     "type": "snapshot",
///     "trades": [{
///       "product_id": "BTC-USD",
///       "trade_id": "123",
///       "price": "50000.0",
///       "size": "0.001",
///       "side": "BUY",
///       "time": "2024-01-01T00:00:00Z"
///     }]
///   }]
/// }
/// ```
pub(crate) fn parse_market_trades(raw: &Value) -> WebSocketResult<StreamEvent> {
    let events = raw
        .get("events")
        .and_then(|e| e.as_array())
        .ok_or_else(|| WebSocketError::Parse("market_trades: missing events".into()))?;

    let event = events
        .first()
        .ok_or_else(|| WebSocketError::Parse("market_trades: empty events".into()))?;

    let trade_data = event
        .get("trades")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| WebSocketError::Parse("market_trades: missing trades array".into()))?;

    let symbol = trade_data
        .get("product_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let price = trade_data
        .get("price")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("market_trades: missing price".into()))?;

    let quantity = trade_data
        .get("size")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("market_trades: missing size".into()))?;

    let side_str = trade_data
        .get("side")
        .and_then(|v| v.as_str())
        .unwrap_or("BUY");

    let side = if side_str.eq_ignore_ascii_case("sell") {
        TradeSide::Sell
    } else {
        TradeSide::Buy
    };

    let timestamp = trade_data
        .get("time")
        .and_then(|v| v.as_str())
        .and_then(parse_rfc3339_ms)
        .unwrap_or(0);

    let id = trade_data
        .get("trade_id")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
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

/// Parse `candles` frame → StreamEvent::Kline.
///
/// Wire shape:
/// ```json
/// {
///   "channel": "candles",
///   "events": [{
///     "type": "candle",
///     "product_id": "BTC-USD",
///     "candles": [{
///       "start": "1698315900",
///       "high": "51000.0",
///       "low": "49000.0",
///       "open": "50000.0",
///       "close": "50500.0",
///       "volume": "123.45"
///     }]
///   }]
/// }
/// ```
pub(crate) fn parse_candles(raw: &Value) -> WebSocketResult<StreamEvent> {
    let events = raw
        .get("events")
        .and_then(|e| e.as_array())
        .ok_or_else(|| WebSocketError::Parse("candles: missing events".into()))?;

    let event = events
        .first()
        .ok_or_else(|| WebSocketError::Parse("candles: empty events".into()))?;

    let symbol = event
        .get("product_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let candles = event
        .get("candles")
        .and_then(|c| c.as_array())
        .ok_or_else(|| WebSocketError::Parse("candles: missing candles array".into()))?;

    let candle = candles
        .first()
        .ok_or_else(|| WebSocketError::Parse("candles: empty candles array".into()))?;

    let start_str = candle
        .get("start")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebSocketError::Parse("candles: missing start".into()))?;

    let open_time = parse_unix_seconds_ms(start_str)
        .ok_or_else(|| WebSocketError::Parse("candles: invalid start timestamp".into()))?;

    let open = candle
        .get("open")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("candles: missing open".into()))?;

    let high = candle
        .get("high")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("candles: missing high".into()))?;

    let low = candle
        .get("low")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("candles: missing low".into()))?;

    let close = candle
        .get("close")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("candles: missing close".into()))?;

    let volume = candle
        .get("volume")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("candles: missing volume".into()))?;

    Ok(StreamEvent::Kline {
        symbol,
        interval: KlineInterval::new(""),
        kline: crate::core::types::Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: None,
            close_time: Some(open_time),
            trades: None,
        },
    })
}

/// Parse `rfq_matches` frame → StreamEvent::BlockTrade.
///
/// RFQ matches are publicly visible block-trade executions via Coinbase's
/// Request-For-Quote matching engine.
///
/// Wire shape:
/// ```json
/// {
///   "channel": "rfq_matches",
///   "events": [{
///     "type": "rfq_match",
///     "rfq_match_id": "abc123",
///     "product_id": "BTC-USD",
///     "side": "BUY",
///     "size": "0.1",
///     "price": "50000",
///     "time": "2024-01-01T00:00:00Z"
///   }]
/// }
/// ```
pub(crate) fn parse_rfq_matches(raw: &Value) -> WebSocketResult<StreamEvent> {
    let events = raw
        .get("events")
        .and_then(|e| e.as_array())
        .ok_or_else(|| WebSocketError::Parse("rfq_matches: missing events".into()))?;

    let event = events
        .first()
        .ok_or_else(|| WebSocketError::Parse("rfq_matches: empty events".into()))?;

    let block_id = event
        .get("rfq_match_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let symbol = event
        .get("product_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let side_str = event
        .get("side")
        .and_then(|v| v.as_str())
        .unwrap_or("BUY");

    let side = if side_str.eq_ignore_ascii_case("sell") {
        TradeSide::Sell
    } else {
        TradeSide::Buy
    };

    let price = event
        .get("price")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let quantity = event
        .get("size")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let timestamp = event
        .get("time")
        .and_then(|v| v.as_str())
        .and_then(parse_rfc3339_ms)
        .unwrap_or(0);

    Ok(StreamEvent::BlockTrade {
        symbol,
        block_id,
        price,
        quantity,
        side,
        timestamp,
        is_iv: false,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Kline interval mapping
// ─────────────────────────────────────────────────────────────────────────────

/// Map a KlineInterval to the Coinbase granularity enum string.
pub fn map_kline_interval(interval: &KlineInterval) -> &'static str {
    match interval.0.as_str() {
        "1m" => "ONE_MINUTE",
        "5m" => "FIVE_MINUTE",
        "15m" => "FIFTEEN_MINUTE",
        "30m" => "THIRTY_MINUTE",
        "1h" => "ONE_HOUR",
        "2h" => "TWO_HOUR",
        "6h" => "SIX_HOUR",
        "1d" => "ONE_DAY",
        _ => "ONE_HOUR", // default
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

    // ── subscribe_frame tests ─────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_ticker() {
        let proto = CoinbaseProtocol::public();
        let spec = make_spec(StreamKind::Ticker, "BTC-USD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["type"], "subscribe");
            assert_eq!(v["channel"], "ticker");
            assert_eq!(v["product_ids"][0], "BTC-USD");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_orderbook_uses_level2() {
        let proto = CoinbaseProtocol::public();
        let spec = make_spec(StreamKind::Orderbook, "BTC-USD");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["channel"], "level2");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_candles_includes_granularity() {
        let proto = CoinbaseProtocol::public();
        let spec = make_spec(
            StreamKind::Kline { interval: KlineInterval::new("1h") },
            "BTC-USD",
        );
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["channel"], "candles");
            assert_eq!(v["granularity"], "ONE_HOUR");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_unsupported_liquidation() {
        let proto = CoinbaseProtocol::public();
        let spec = make_spec(StreamKind::Liquidation, "BTC-USD");
        let result = proto.subscribe_frame(&spec);
        assert!(
            matches!(result, Err(WebSocketError::NotSupported(_))),
            "Liquidation must return NotSupported, got {:?}",
            result
        );
    }

    // ── extract_topic tests ───────────────────────────────────────────────────

    #[test]
    fn extract_topic_ticker_returns_key() {
        let proto = CoinbaseProtocol::public();
        let raw = serde_json::json!({"channel": "ticker", "events": []});
        assert_eq!(proto.extract_topic(&raw), Some(TopicKey::new("ticker")));
    }

    #[test]
    fn extract_topic_l2_data_returns_key() {
        let proto = CoinbaseProtocol::public();
        let raw = serde_json::json!({"channel": "l2_data", "events": []});
        assert_eq!(proto.extract_topic(&raw), Some(TopicKey::new("l2_data")));
    }

    #[test]
    fn extract_topic_heartbeats_returns_none() {
        let proto = CoinbaseProtocol::public();
        let raw = serde_json::json!({"channel": "heartbeats", "events": []});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_subscriptions_returns_none() {
        let proto = CoinbaseProtocol::public();
        let raw = serde_json::json!({"type": "subscriptions", "channel": "subscriptions"});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_status_returns_none() {
        let proto = CoinbaseProtocol::public();
        let raw = serde_json::json!({"channel": "status", "events": []});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    // ── is_subscribe_ack tests ────────────────────────────────────────────────

    #[test]
    fn is_subscribe_ack_for_subscriptions_type() {
        let proto = CoinbaseProtocol::public();
        let raw = serde_json::json!({"type": "subscriptions", "subscriptions": {}});
        assert!(proto.is_subscribe_ack(&raw));
    }

    #[test]
    fn is_subscribe_ack_false_for_ticker() {
        let proto = CoinbaseProtocol::public();
        let raw = serde_json::json!({"type": "ticker", "channel": "ticker"});
        assert!(!proto.is_subscribe_ack(&raw));
    }

    // ── ping_frame ────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_is_none() {
        let proto = CoinbaseProtocol::public();
        assert!(proto.ping_frame().is_none(), "Coinbase ping_frame must be None");
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_supports_expected_kinds() {
        let proto = CoinbaseProtocol::public();
        let reg = proto.topic_registry(AccountType::Spot);
        let at = AccountType::Spot;
        assert!(reg.supports(&StreamKind::Ticker, at));
        assert!(reg.supports(&StreamKind::Trade, at));
        assert!(reg.supports(&StreamKind::Orderbook, at));
        assert!(reg.supports(&StreamKind::OrderbookDelta, at));
        assert!(reg.supports(&StreamKind::Kline { interval: KlineInterval::new("") }, at));
        assert!(reg.supports(&StreamKind::BlockTrade, at));
    }

    #[test]
    fn l2_data_topic_dispatches_to_parser() {
        let proto = CoinbaseProtocol::public();
        let reg = proto.topic_registry(AccountType::Spot);
        let key = TopicKey::new("l2_data");
        assert!(
            reg.dispatch(&key).is_some(),
            "l2_data must dispatch to a parser"
        );
    }

    // ── parse_ticker ─────────────────────────────────────────────────────────

    #[test]
    fn parse_ticker_extracts_fields() {
        let raw = serde_json::json!({
            "channel": "ticker",
            "timestamp": "2024-01-01T00:00:00Z",
            "events": [{
                "type": "ticker",
                "tickers": [{
                    "product_id": "BTC-USD",
                    "price": "50000.0",
                    "best_bid": "49999.0",
                    "best_ask": "50001.0",
                    "volume_24_h": "1234.5",
                    "high_24_h": "51000.0",
                    "low_24_h": "49000.0",
                    "price_percent_chg_24_h": "2.5"
                }]
            }]
        });
        let ev = parse_ticker(&raw).expect("parse ticker");
        match ev {
            StreamEvent::Ticker { symbol, ticker } => {
                assert_eq!(symbol, "BTC-USD");
                assert!((ticker.last_price - 50000.0).abs() < 1e-9);
                assert_eq!(ticker.bid_price, Some(49999.0));
                assert_eq!(ticker.ask_price, Some(50001.0));
            }
            other => panic!("expected Ticker, got {:?}", other),
        }
    }

    // ── parse_l2_data ─────────────────────────────────────────────────────────

    #[test]
    fn parse_l2_data_snapshot_emits_orderbook_snapshot() {
        let raw = serde_json::json!({
            "channel": "l2_data",
            "timestamp": "2024-01-01T00:00:00Z",
            "sequence_num": 1,
            "events": [{
                "type": "snapshot",
                "product_id": "BTC-USD",
                "updates": [
                    {"side": "bid", "price_level": "50000.0", "new_quantity": "1.5"},
                    {"side": "offer", "price_level": "50001.0", "new_quantity": "0.5"}
                ]
            }]
        });
        let ev = parse_l2_data(&raw).expect("parse l2_data snapshot");
        match ev {
            StreamEvent::OrderbookSnapshot { symbol, book } => {
                assert_eq!(symbol, "BTC-USD");
                assert_eq!(book.bids.len(), 1);
                assert_eq!(book.asks.len(), 1);
            }
            other => panic!("expected OrderbookSnapshot, got {:?}", other),
        }
    }

    #[test]
    fn parse_l2_data_update_emits_orderbook_delta() {
        let raw = serde_json::json!({
            "channel": "l2_data",
            "timestamp": "2024-01-01T00:00:00Z",
            "sequence_num": 2,
            "events": [{
                "type": "update",
                "product_id": "BTC-USD",
                "updates": [
                    {"side": "bid", "price_level": "49999.0", "new_quantity": "0.0"}
                ]
            }]
        });
        let ev = parse_l2_data(&raw).expect("parse l2_data update");
        match ev {
            StreamEvent::OrderbookDelta { symbol, delta } => {
                assert_eq!(symbol, "BTC-USD");
                assert_eq!(delta.bids.len(), 1);
                // Zero-quantity levels are kept in deltas (they signal removal).
                assert!((delta.bids[0].size - 0.0).abs() < 1e-9);
            }
            other => panic!("expected OrderbookDelta, got {:?}", other),
        }
    }

    #[test]
    fn parse_l2_data_snapshot_skips_zero_qty_levels() {
        let raw = serde_json::json!({
            "channel": "l2_data",
            "timestamp": "2024-01-01T00:00:00Z",
            "sequence_num": 3,
            "events": [{
                "type": "snapshot",
                "product_id": "BTC-USD",
                "updates": [
                    {"side": "bid", "price_level": "50000.0", "new_quantity": "1.5"},
                    {"side": "bid", "price_level": "49000.0", "new_quantity": "0.0"}
                ]
            }]
        });
        let ev = parse_l2_data(&raw).expect("parse");
        match ev {
            StreamEvent::OrderbookSnapshot { book, .. } => {
                assert_eq!(book.bids.len(), 1, "zero-qty level must be skipped in snapshot");
            }
            other => panic!("expected OrderbookSnapshot, got {:?}", other),
        }
    }

    // ── parse_market_trades ───────────────────────────────────────────────────

    #[test]
    fn parse_market_trades_emits_trade() {
        let raw = serde_json::json!({
            "channel": "market_trades",
            "events": [{
                "type": "update",
                "trades": [{
                    "product_id": "BTC-USD",
                    "trade_id": "abc",
                    "price": "50000.0",
                    "size": "0.001",
                    "side": "BUY",
                    "time": "2024-01-01T00:00:00Z"
                }]
            }]
        });
        let ev = parse_market_trades(&raw).expect("parse market_trades");
        match ev {
            StreamEvent::Trade { symbol, trade } => {
                assert_eq!(symbol, "BTC-USD");
                assert!((trade.price - 50000.0).abs() < 1e-9);
                assert_eq!(trade.side, TradeSide::Buy);
            }
            other => panic!("expected Trade, got {:?}", other),
        }
    }

    // ── parse_candles ─────────────────────────────────────────────────────────

    #[test]
    fn parse_candles_emits_kline() {
        let raw = serde_json::json!({
            "channel": "candles",
            "events": [{
                "type": "candle",
                "product_id": "BTC-USD",
                "candles": [{
                    "start": "1698315900",
                    "high": "51000.0",
                    "low": "49000.0",
                    "open": "50000.0",
                    "close": "50500.0",
                    "volume": "123.45"
                }]
            }]
        });
        let ev = parse_candles(&raw).expect("parse candles");
        match ev {
            StreamEvent::Kline { symbol, kline, .. } => {
                assert_eq!(symbol, "BTC-USD");
                assert_eq!(kline.open_time, 1698315900_i64 * 1000);
                assert!((kline.open - 50000.0).abs() < 1e-9);
                assert!((kline.volume - 123.45).abs() < 1e-9);
            }
            other => panic!("expected Kline, got {:?}", other),
        }
    }

    // ── parse_rfq_matches ─────────────────────────────────────────────────────

    #[test]
    fn parse_rfq_matches_emits_block_trade() {
        let raw = serde_json::json!({
            "channel": "rfq_matches",
            "events": [{
                "type": "rfq_match",
                "rfq_match_id": "match-001",
                "product_id": "BTC-USD",
                "side": "SELL",
                "size": "10.0",
                "price": "50000.0",
                "time": "2024-01-01T00:00:00Z"
            }]
        });
        let ev = parse_rfq_matches(&raw).expect("parse rfq_matches");
        match ev {
            StreamEvent::BlockTrade { symbol, block_id, side, price, quantity, .. } => {
                assert_eq!(symbol, "BTC-USD");
                assert_eq!(block_id, "match-001");
                assert_eq!(side, TradeSide::Sell);
                assert!((price - 50000.0).abs() < 1e-9);
                assert!((quantity - 10.0).abs() < 1e-9);
            }
            other => panic!("expected BlockTrade, got {:?}", other),
        }
    }
}
