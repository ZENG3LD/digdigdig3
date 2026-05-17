//! BinanceProtocol — WsProtocol implementation for Binance.
//!
//! Declarative shim: supplies endpoint URLs, ping frame (None = native WS ping),
//! subscribe/unsubscribe frames, topic extraction, and topic registry to
//! UniversalWsTransport.
//!
//! ## Combined-stream format
//! All subscriptions use the `/stream` endpoint (combined-stream mode).
//! Frames arrive as `{"stream":"btcusdt@trade","data":{...}}`.
//! The `stream` field IS the topic key.
//!
//! ## Silent-stream fix (spec §3.3)
//! Old code had `_ => Ok(None)` catch-all in `parse_event_by_type`.
//! The framework now emits `tracing::warn!` for every unmatched topic,
//! making silent drops visible.  All known event types are covered here.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, OrderBookLevel, StreamEvent, WebSocketError, WebSocketResult,
    OrderbookDelta as OrderbookDeltaData,
};
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol,
};

use super::endpoints::BinanceUrls;
use super::parser::BinanceParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry caches
// ─────────────────────────────────────────────────────────────────────────────

static SPOT_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static FUTURES_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// BinanceProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Binance WS protocol shim.
pub struct BinanceProtocol {
    _account_type: AccountType,
    _testnet: bool,
    urls: BinanceUrls,
}

impl BinanceProtocol {
    pub fn new(account_type: AccountType, testnet: bool) -> Self {
        let urls = if testnet {
            BinanceUrls::TESTNET
        } else {
            BinanceUrls::MAINNET
        };
        Self { _account_type: account_type, _testnet: testnet, urls }
    }

    fn spot_registry() -> &'static TopicRegistry {
        SPOT_REGISTRY.get_or_init(|| build_registry(AccountType::Spot))
    }

    fn futures_registry() -> &'static TopicRegistry {
        FUTURES_REGISTRY.get_or_init(|| build_registry(AccountType::FuturesCross))
    }

    /// Build the wire stream name for a StreamSpec (Binance combined-stream format).
    ///
    /// Symbol is always lowercase, e.g. "btcusdt".
    fn stream_name(spec: &StreamSpec) -> Result<String, WebSocketError> {
        // spec.symbol is now a raw exchange-native string (e.g. "BTCUSDT").
        // Binance combined-stream format requires it lowercase.
        let symbol = spec.symbol.to_lowercase();

        let name = match &spec.kind {
            StreamKind::Ticker => format!("{}@ticker", symbol),
            StreamKind::Trade => format!("{}@trade", symbol),
            StreamKind::AggTrade => format!("{}@aggTrade", symbol),

            StreamKind::Orderbook => {
                let depth = spec.depth.unwrap_or(20);
                let speed = spec.speed_ms.unwrap_or(100);
                format!("{}@depth{}@{}ms", symbol, depth, speed)
            }
            StreamKind::OrderbookDelta => {
                let speed = spec.speed_ms.unwrap_or(100);
                format!("{}@depth@{}ms", symbol, speed)
            }

            StreamKind::Kline { interval } => format!("{}@kline_{}", symbol, interval.as_str()),
            StreamKind::MarkPriceKline { interval } => {
                format!("{}@markPriceKline_{}", symbol, interval.as_str())
            }
            StreamKind::IndexPriceKline { interval } => {
                format!("{}@indexPriceKline_{}", symbol, interval.as_str())
            }
            StreamKind::PremiumIndexKline { interval } => {
                format!("{}@premiumIndexKline_{}", symbol, interval.as_str())
            }

            StreamKind::MarkPrice => {
                if spec.symbol.is_empty() {
                    "!markPrice@arr@1s".to_string()
                } else {
                    format!("{}@markPrice", symbol)
                }
            }
            StreamKind::FundingRate => format!("{}@markPrice", symbol),

            StreamKind::Liquidation => {
                if spec.symbol.is_empty() {
                    "!forceOrder@arr".to_string()
                } else {
                    format!("{}@forceOrder", symbol)
                }
            }

            StreamKind::CompositeIndex => format!("{}@compositeIndex", symbol),
            StreamKind::IndexPrice => format!("{}@indexPrice@1s", symbol),

            // Private streams — no wire name needed (listenKey URL handles routing).
            StreamKind::OrderUpdate | StreamKind::BalanceUpdate | StreamKind::PositionUpdate => {
                return Err(WebSocketError::UnsupportedOperation(
                    "binance: private streams use listenKey, not subscribe frames".into(),
                ));
            }

            other => {
                return Err(WebSocketError::UnsupportedOperation(format!(
                    "binance: unsupported stream kind {:?}",
                    other
                )));
            }
        };

        Ok(name)
    }

    fn build_sub_frame(op: &str, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let stream = Self::stream_name(spec)?;
        // Use a simple incrementing id via thread-local or constant — framework doesn't
        // inspect the id value, only the server uses it for correlation.
        let frame = json!({
            "method": op,
            "params": [stream],
            "id": 1u64,
        });
        Ok(Message::Text(frame.to_string()))
    }
}

impl WsProtocol for BinanceProtocol {
    fn name(&self) -> &'static str {
        "binance"
    }

    fn endpoint(&self, account_type: AccountType, _testnet: bool) -> Url {
        // Use combined-stream endpoint for multiplexing.
        let base = self.urls.ws_url(account_type);
        let url = format!("{}/stream", base);
        Url::parse(&url).expect("binance ws url is valid")
    }

    /// Binance uses native WS Ping frames; server sends them, tokio-tungstenite
    /// auto-responds with Pong.  No application-level ping frame needed.
    fn ping_frame(&self) -> Option<Message> {
        None
    }

    fn ping_interval(&self) -> Duration {
        // Binance closes after 24h of inactivity; 20s interval keeps connection warm.
        Duration::from_secs(20)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_sub_frame("SUBSCRIBE", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_sub_frame("UNSUBSCRIBE", spec)
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        // Binance public WS — no auth frame.
        // Private streams use listenKey URL rather than an auth frame.
        None
    }

    fn is_auth_ack(&self, _raw: &Value) -> bool {
        false
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // Binance uses native WS pong, not a JSON pong frame.
        // This method is only called for text/binary JSON frames.
        // Native pong frames never reach here (transport handles them).
        // Return false — no JSON pong to recognize.
        let _ = raw;
        false
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        // {"result":null,"id":N} or {"result":[...],"id":N}
        raw.get("id").is_some() && raw.get("result").is_some()
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Subscribe/unsubscribe ack: {"result":null,"id":N}
        if raw.get("id").is_some() && raw.get("result").is_some() {
            return None;
        }

        // Error frame: {"error":{"code":...,"msg":"..."},"id":N}
        if raw.get("error").is_some() {
            return None;
        }

        // Combined-stream frame: {"stream":"btcusdt@trade","data":{...}}
        if let Some(stream) = raw.get("stream").and_then(|s| s.as_str()) {
            return Some(TopicKey::new(stream));
        }

        // Single-stream frame (raw mode): look at "e" event type.
        // In raw mode the stream name isn't in the envelope; we reconstruct
        // a pseudo-topic from the event type so registry dispatch works.
        if let Some(event_type) = raw.get("e").and_then(|e| e.as_str()) {
            return Some(TopicKey::new(event_type));
        }

        // Partial depth snapshot (no "e" field, no "stream", has "lastUpdateId"):
        // These arrive in raw mode; map to a synthetic "partialDepth" topic.
        if raw.get("lastUpdateId").is_some() && raw.get("bids").is_some() {
            return Some(TopicKey::new("partialDepth"));
        }

        None
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        match account_type {
            AccountType::Spot | AccountType::Margin | AccountType::Earn
            | AccountType::Lending | AccountType::Convert => Self::spot_registry(),
            _ => Self::futures_registry(),
        }
    }

    fn unsupported_by_exchange(&self, account_type: AccountType) -> &'static [StreamKind] {
        match account_type {
            AccountType::Spot | AccountType::Margin => SPOT_UNSUPPORTED,
            _ => &[],
        }
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[StreamKind::OrderUpdate, StreamKind::BalanceUpdate, StreamKind::PositionUpdate]
    }
}

static SPOT_UNSUPPORTED: &[StreamKind] = &[
    // Spot has no mark price, funding, or liquidation streams.
    StreamKind::MarkPrice,
    StreamKind::FundingRate,
    StreamKind::Liquidation,
];

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry(account_type: AccountType) -> TopicRegistry {
    let mut b = TopicRegistry::builder();

    // ── Public market streams (spot + futures) ────────────────────────────
    b = b
        .register(StreamKind::Ticker, account_type, "*@ticker", parse_ticker)
        .register(StreamKind::Trade, account_type, "*@trade", parse_trade)
        .register(StreamKind::AggTrade, account_type, "*@aggTrade", parse_agg_trade)
        .register(StreamKind::OrderbookDelta, account_type, "*@depth@*", parse_depth_update)
        // Partial depth snapshots via combined-stream: "btcusdt@depth5@100ms"
        .register(StreamKind::Orderbook, account_type, "*@depth5@*", parse_partial_depth)
        .register(StreamKind::Orderbook, account_type, "*@depth10@*", parse_partial_depth)
        .register(StreamKind::Orderbook, account_type, "*@depth20@*", parse_partial_depth)
        // Raw-mode partial depth (no event type field, synthetic "partialDepth" topic)
        .register(StreamKind::Orderbook, account_type, "partialDepth", parse_partial_depth_raw)
        // miniTicker / bookTicker
        .register(StreamKind::Ticker, account_type, "*@miniTicker", parse_mini_ticker)
        .register(StreamKind::Ticker, account_type, "*@bookTicker", parse_book_ticker);

    // ── Kline streams (all intervals, same parser) ────────────────────────
    for (wire, internal) in BINANCE_KLINE_INTERVALS {
        let kind = StreamKind::Kline {
            interval: KlineInterval::new(*internal),
        };
        let pattern = format!("*@kline_{}", wire);
        b = b.register(kind, account_type, pattern, parse_kline);
    }

    // ── Futures-only streams ──────────────────────────────────────────────
    if !matches!(account_type, AccountType::Spot | AccountType::Margin) {
        b = b
            .register(StreamKind::MarkPrice, account_type, "*@markPrice", parse_mark_price)
            .register(StreamKind::MarkPrice, account_type, "*@markPrice@1s", parse_mark_price)
            .register(StreamKind::MarkPrice, account_type, "!markPrice@arr", parse_mark_price_arr)
            .register(StreamKind::FundingRate, account_type, "*@markPrice", parse_mark_price)
            .register(StreamKind::Liquidation, account_type, "*@forceOrder", parse_force_order)
            .register(StreamKind::Liquidation, account_type, "!forceOrder@arr", parse_force_order_arr)
            .register(StreamKind::CompositeIndex, account_type, "*@compositeIndex", parse_composite_index)
            .register(StreamKind::IndexPrice, account_type, "*@indexPrice@1s", parse_index_price);

        // Futures kline variants
        for (wire, internal) in BINANCE_KLINE_INTERVALS {
            let mk_kind = StreamKind::MarkPriceKline {
                interval: KlineInterval::new(*internal),
            };
            let ix_kind = StreamKind::IndexPriceKline {
                interval: KlineInterval::new(*internal),
            };
            let pm_kind = StreamKind::PremiumIndexKline {
                interval: KlineInterval::new(*internal),
            };
            b = b
                .register(mk_kind, account_type, format!("*@markPriceKline_{}", wire), parse_mark_price_kline)
                .register(ix_kind, account_type, format!("*@indexPriceKline_{}", wire), parse_index_price_kline)
                .register(pm_kind, account_type, format!("*@premiumIndexKline_{}", wire), parse_premium_index_kline);
        }

        // Private stream event types (dispatched via raw event type key)
        b = b
            .register(StreamKind::OrderUpdate, account_type, "executionReport", parse_execution_report)
            .register(StreamKind::OrderUpdate, account_type, "ORDER_TRADE_UPDATE", parse_futures_order_update)
            .register(StreamKind::BalanceUpdate, account_type, "outboundAccountPosition", parse_account_position)
            .register(StreamKind::BalanceUpdate, account_type, "balanceUpdate", parse_balance_update)
            .register(StreamKind::BalanceUpdate, account_type, "ACCOUNT_UPDATE", parse_futures_account_update);
    } else {
        // Spot private streams
        b = b
            .register(StreamKind::OrderUpdate, account_type, "executionReport", parse_execution_report)
            .register(StreamKind::BalanceUpdate, account_type, "outboundAccountPosition", parse_account_position)
            .register(StreamKind::BalanceUpdate, account_type, "balanceUpdate", parse_balance_update);
    }

    b.build()
}

/// Binance wire kline suffixes → internal interval strings.
const BINANCE_KLINE_INTERVALS: &[(&str, &str)] = &[
    ("1m", "1m"),
    ("3m", "3m"),
    ("5m", "5m"),
    ("15m", "15m"),
    ("30m", "30m"),
    ("1h", "1h"),
    ("2h", "2h"),
    ("4h", "4h"),
    ("6h", "6h"),
    ("8h", "8h"),
    ("12h", "12h"),
    ("1d", "1d"),
    ("3d", "3d"),
    ("1w", "1w"),
    ("1M", "1M"),
];

// ─────────────────────────────────────────────────────────────────────────────
// Parsers (fn(&Value) -> WebSocketResult<StreamEvent>)
//
// Each parser receives the full combined-stream frame:
//   {"stream":"btcusdt@trade","data":{...}}
// or the raw data object in raw-mode.
//
// Helper: extract "data" field from combined-stream envelope, or use frame
// directly if it looks like a raw-mode frame.
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the inner data object from a combined-stream frame.
/// If "data" field exists, return it; otherwise return the frame itself.
fn frame_data(raw: &Value) -> &Value {
    raw.get("data").unwrap_or(raw)
}

fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let ticker = BinanceParser::parse_ticker(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Ticker(ticker))
}

fn parse_mini_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::Ticker;

    let data = frame_data(raw);
    let parse_f64 = |key: &str| -> Option<f64> {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| data.get(key).and_then(|v| v.as_f64()))
    };

    Ok(StreamEvent::Ticker(Ticker {
        symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        last_price: parse_f64("c").unwrap_or(0.0),
        bid_price: None,
        ask_price: None,
        high_24h: parse_f64("h"),
        low_24h: parse_f64("l"),
        volume_24h: parse_f64("v"),
        quote_volume_24h: parse_f64("q"),
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: data.get("E").and_then(|t| t.as_i64()).unwrap_or(0),
    }))
}

fn parse_book_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::Ticker;

    let data = frame_data(raw);
    let parse_f64 = |key: &str| -> Option<f64> {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| data.get(key).and_then(|v| v.as_f64()))
    };

    let bid = parse_f64("b");
    let ask = parse_f64("a");
    let last_price = bid.unwrap_or(0.0);

    Ok(StreamEvent::Ticker(Ticker {
        symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        last_price,
        bid_price: bid,
        ask_price: ask,
        high_24h: None,
        low_24h: None,
        volume_24h: None,
        quote_volume_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: data.get("T").and_then(|t| t.as_i64()).unwrap_or(0),
    }))
}

fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let trade = BinanceParser::parse_ws_trade(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Trade(trade))
}

fn parse_agg_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    let data = frame_data(raw);
    let parse_f64 = |key: &str| -> Option<f64> {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| data.get(key).and_then(|v| v.as_f64()))
    };

    let is_buyer_maker = data.get("m").and_then(|m| m.as_bool()).unwrap_or(false);
    let side = if is_buyer_maker { TradeSide::Sell } else { TradeSide::Buy };

    Ok(StreamEvent::AggTrade {
        symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        aggregate_id: data.get("a").and_then(|a| a.as_i64()).unwrap_or(0),
        price: parse_f64("p").unwrap_or(0.0),
        quantity: parse_f64("q").unwrap_or(0.0),
        first_trade_id: data.get("f").and_then(|f| f.as_i64()).unwrap_or(0),
        last_trade_id: data.get("l").and_then(|l| l.as_i64()).unwrap_or(0),
        side,
        timestamp: data.get("T").and_then(|t| t.as_i64()).unwrap_or(0),
    })
}

fn parse_levels(data: &Value, key: &str) -> Vec<OrderBookLevel> {
    data.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|pair| {
                    let p = pair.get(0)?.as_str()?.parse().ok()?;
                    let s = pair.get(1)?.as_str()?.parse().ok()?;
                    Some(OrderBookLevel::new(p, s))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_depth_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event_time = data.get("E").and_then(|e| e.as_i64());

    Ok(StreamEvent::OrderbookDelta(OrderbookDeltaData {
        bids: parse_levels(data, "b"),
        asks: parse_levels(data, "a"),
        timestamp: event_time.unwrap_or(0),
        first_update_id: data.get("U").and_then(|v| v.as_u64()),
        last_update_id: data.get("u").and_then(|v| v.as_u64()),
        prev_update_id: data.get("pu").and_then(|v| v.as_u64()),
        event_time,
        checksum: None,
    }))
}

fn parse_partial_depth(raw: &Value) -> WebSocketResult<StreamEvent> {
    // Combined-stream partial depth: {"stream":"btcusdt@depth20@100ms","data":{...}}
    // data has "lastUpdateId", "bids", "asks" — no "e" event type.
    let data = frame_data(raw);
    parse_partial_depth_inner(data)
}

fn parse_partial_depth_raw(raw: &Value) -> WebSocketResult<StreamEvent> {
    // Raw-mode partial depth (single-stream URL): the frame IS the data.
    parse_partial_depth_inner(raw)
}

fn parse_partial_depth_inner(data: &Value) -> WebSocketResult<StreamEvent> {
    let event_time = data.get("E").and_then(|e| e.as_i64());

    Ok(StreamEvent::OrderbookSnapshot(crate::core::OrderBook {
        bids: parse_levels(data, "bids"),
        asks: parse_levels(data, "asks"),
        timestamp: event_time.unwrap_or(0),
        sequence: None,
        last_update_id: data.get("lastUpdateId").and_then(|v| v.as_u64()),
        first_update_id: None,
        prev_update_id: None,
        event_time,
        transaction_time: None,
        checksum: None,
    }))
}

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let kline = BinanceParser::parse_ws_kline(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Kline(kline))
}

fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let parse_f64 = |key: &str| -> Option<f64> {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| data.get(key).and_then(|v| v.as_f64()))
    };

    Ok(StreamEvent::MarkPrice {
        symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        mark_price: parse_f64("p").unwrap_or(0.0),
        index_price: parse_f64("i"),
        timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
    })
}

fn parse_mark_price_arr(raw: &Value) -> WebSocketResult<StreamEvent> {
    // !markPrice@arr arrives as {"stream":"!markPrice@arr","data":[{...},{...}]}
    // Emit first element; the transport's multi-emit logic handles arrays.
    let data = frame_data(raw);
    let item = if let Some(arr) = data.as_array() {
        arr.first().unwrap_or(data)
    } else {
        data
    };
    let parse_f64 = |key: &str| -> Option<f64> {
        item.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| item.get(key).and_then(|v| v.as_f64()))
    };
    Ok(StreamEvent::MarkPrice {
        symbol: item.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        mark_price: parse_f64("p").unwrap_or(0.0),
        index_price: parse_f64("i"),
        timestamp: item.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
    })
}

fn parse_force_order(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    let data = frame_data(raw);
    let o = data.get("o").ok_or_else(|| {
        WebSocketError::Parse("forceOrder: missing 'o' field".into())
    })?;

    let parse_f64 = |key: &str| -> Option<f64> {
        o.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| o.get(key).and_then(|v| v.as_f64()))
    };

    let side = match o.get("S").and_then(|s| s.as_str()).unwrap_or("") {
        "SELL" => TradeSide::Buy,
        _ => TradeSide::Sell,
    };

    let price = parse_f64("ap").unwrap_or_else(|| parse_f64("p").unwrap_or(0.0));
    let quantity = parse_f64("q").unwrap_or(0.0);

    Ok(StreamEvent::Liquidation {
        symbol: o.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        side,
        price,
        quantity,
        timestamp: o.get("T").and_then(|t| t.as_i64()).unwrap_or(0),
        value: Some(price * quantity),
    })
}

fn parse_force_order_arr(raw: &Value) -> WebSocketResult<StreamEvent> {
    // !forceOrder@arr: {"stream":"!forceOrder@arr","data":[{...}]}
    let data = frame_data(raw);
    let item = if let Some(arr) = data.as_array() {
        arr.first().unwrap_or(data)
    } else {
        data
    };
    // Wrap single item to reuse parse_force_order
    let wrapped = json!({"o": item});
    parse_force_order(&wrapped)
}

fn parse_composite_index(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let parse_f64_field = |key: &str| -> Option<f64> {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| data.get(key).and_then(|v| v.as_f64()))
    };

    let symbol = data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string();
    let price = parse_f64_field("p").unwrap_or(0.0);
    let timestamp = data.get("E").and_then(|e| e.as_i64()).unwrap_or(0);

    let components: Vec<(String, f64)> = data
        .get("c")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let base = item.get("b").and_then(|v| v.as_str()).unwrap_or("");
                    let quote = item.get("q").and_then(|v| v.as_str()).unwrap_or("");
                    let comp_symbol = format!("{}{}", base, quote);
                    let weight = item
                        .get("W")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| item.get("W").and_then(|v| v.as_f64()))
                        .or_else(|| {
                            item.get("w")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok())
                        })
                        .or_else(|| item.get("w").and_then(|v| v.as_f64()))
                        .unwrap_or(0.0);
                    if comp_symbol.is_empty() {
                        None
                    } else {
                        Some((comp_symbol, weight))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(StreamEvent::CompositeIndex { symbol, price, components, timestamp })
}

fn parse_index_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let parse_f64_field = |key: &str| -> Option<f64> {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| data.get(key).and_then(|v| v.as_f64()))
    };

    let symbol = data
        .get("i")
        .and_then(|s| s.as_str())
        .or_else(|| data.get("s").and_then(|s| s.as_str()))
        .unwrap_or("")
        .to_string();

    Ok(StreamEvent::IndexPrice {
        symbol,
        price: parse_f64_field("p").unwrap_or(0.0),
        timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
    })
}

fn parse_mark_price_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_mark_price_kline(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(event)
}

fn parse_index_price_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_index_price_kline(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(event)
}

fn parse_premium_index_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_premium_index_kline(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(event)
}

fn parse_execution_report(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_execution_report(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderUpdate(event))
}

fn parse_futures_order_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_futures_order_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderUpdate(event))
}

fn parse_account_position(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_account_position(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    match event {
        Some(ev) => Ok(StreamEvent::BalanceUpdate(ev)),
        None => Err(WebSocketError::Parse(
            "outboundAccountPosition: no non-zero balance found".into(),
        )),
    }
}

fn parse_balance_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_balance_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::BalanceUpdate(event))
}

fn parse_futures_account_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw);
    let event = BinanceParser::parse_ws_futures_account_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    match event {
        Some(ev) => Ok(StreamEvent::BalanceUpdate(ev)),
        None => Err(WebSocketError::Parse(
            "ACCOUNT_UPDATE: no balance entry found".into(),
        )),
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
            symbol: "BTCUSDT".to_string(),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        let reg = proto.topic_registry(AccountType::Spot);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "spot registry must have entries");
        assert!(reg.supports(&StreamKind::Ticker, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(reg.supports(
            &StreamKind::Kline { interval: KlineInterval::new("1m") },
            AccountType::Spot
        ));

        let futures_proto = BinanceProtocol::new(AccountType::FuturesCross, false);
        let freg = futures_proto.topic_registry(AccountType::FuturesCross);
        assert!(freg.supports(&StreamKind::MarkPrice, AccountType::FuturesCross));
        assert!(freg.supports(&StreamKind::Liquidation, AccountType::FuturesCross));
    }

    #[test]
    fn test_subscribe_frame_spot_trade() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        let spec = spot_spec(StreamKind::Trade);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["method"], "SUBSCRIBE");
        let params = v["params"].as_array().expect("params array");
        assert_eq!(params.len(), 1);
        // Symbol must be lowercase
        assert_eq!(params[0], "btcusdt@trade");
    }

    #[test]
    fn test_extract_topic_combined_stream() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "stream": "btcusdt@trade",
            "data": {"e": "trade", "s": "BTCUSDT", "p": "50000", "q": "0.1"}
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "btcusdt@trade");
    }

    #[test]
    fn test_extract_topic_subscribe_ack() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        let ack = serde_json::json!({"result": null, "id": 1});
        assert!(proto.extract_topic(&ack).is_none());
    }

    #[test]
    fn test_is_subscribe_ack() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        let ack = serde_json::json!({"result": null, "id": 1});
        assert!(proto.is_subscribe_ack(&ack));
        let not_ack = serde_json::json!({"stream": "btcusdt@trade", "data": {}});
        assert!(!proto.is_subscribe_ack(&not_ack));
    }

    #[test]
    fn test_ping_frame_is_none() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        assert!(
            proto.ping_frame().is_none(),
            "Binance uses native WS ping, not application-level"
        );
    }

    #[test]
    fn test_kline_registry_all_intervals() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        let reg = proto.topic_registry(AccountType::Spot);
        for (_, internal) in BINANCE_KLINE_INTERVALS {
            let kind = StreamKind::Kline {
                interval: KlineInterval::new(*internal),
            };
            assert!(
                reg.supports(&kind, AccountType::Spot),
                "spot registry missing kline interval {}",
                internal
            );
        }
    }

    #[test]
    fn test_subscribe_kline_frame() {
        let proto = BinanceProtocol::new(AccountType::Spot, false);
        let spec = spot_spec(StreamKind::Kline {
            interval: KlineInterval::new("1h"),
        });
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["params"][0], "btcusdt@kline_1h");
    }
}
