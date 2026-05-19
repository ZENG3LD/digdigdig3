//! HtxProtocol — WsProtocol implementation for HTX (Huobi).
//!
//! Declarative shim: supplies endpoint URLs, subscribe/unsubscribe frames,
//! topic extraction (with gzip decode), and topic registry to
//! UniversalWsTransport.
//!
//! ## Key HTX specifics
//! - All frames are gzip-compressed binary — `decode_binary` is overridden.
//! - Server sends `{"ping":<ts>}` heartbeats; client replies `{"pong":<ts>}`.
//!   The framework has no server-pong hook; `extract_topic` filters ping frames
//!   (returns `None`). The pong reply is NOT sent — see KNOWN LIMITATION below.
//! - Subscribe: `{"sub":"market.btcusdt.kline.1min","id":"id1"}`
//! - Frame topic field: `ch`
//!
//! ## KNOWN LIMITATION
//! HTX requires echoing each `{"ping":N}` with `{"pong":N}`. The framework's
//! `WsProtocol` trait has no `on_server_message` hook for server-initiated
//! heartbeats, so pong replies are not sent. HTX idle-disconnects after ~30s
//! without a pong; `UniversalWsTransport` auto-reconnect+subscription-replay
//! compensates with brief event gaps. Follow-up: add a `WsProtocol` hook for
//! server-initiated heartbeats (Wave 3).

use std::io::Read as IoRead;
use std::sync::OnceLock;
use std::time::Duration;

use flate2::read::GzDecoder;
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


// ─────────────────────────────────────────────────────────────────────────────
// Registry caches — one per product line (spot vs linear-swap)
// ─────────────────────────────────────────────────────────────────────────────

static SPOT_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static FUTURES_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// HtxProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative HTX WS protocol shim.
pub struct HtxProtocol {
    _account_type: AccountType,
    _testnet: bool,
    /// Monotonically increasing counter for subscription IDs.
    id_counter: std::sync::atomic::AtomicU64,
}

impl HtxProtocol {
    pub fn new(account_type: AccountType, testnet: bool) -> Self {
        Self {
            _account_type: account_type,
            _testnet: testnet,
            id_counter: std::sync::atomic::AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> String {
        let n = self.id_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("id{n}")
    }

    /// Build the HTX topic string for a given StreamSpec.
    fn topic_for(spec: &StreamSpec) -> Result<String, WebSocketError> {
        // symbol is already the raw exchange-native string (e.g. "btcusdt" for spot,
        // "BTC-USDT" for futures). HTX topics use lowercase for spot and mixed-case
        // for futures — normalizer guarantees the correct casing.
        let sym = spec.symbol.as_str();
        let is_futures = matches!(
            spec.account_type,
            AccountType::FuturesCross | AccountType::FuturesIsolated
        );
        match &spec.kind {
            // For futures, subscribe to .bbo (has bid/ask); for spot, .detail has bid/ask inline.
            StreamKind::Ticker if is_futures => Ok(format!("market.{sym}.bbo")),
            StreamKind::Ticker => Ok(format!("market.{sym}.detail")),
            StreamKind::Trade => Ok(format!("market.{sym}.trade.detail")),
            StreamKind::Orderbook => Ok(format!("market.{sym}.depth.step0")),
            StreamKind::OrderbookDelta => Ok(format!("market.{sym}.mbp.150")),
            StreamKind::Kline { interval } => {
                Ok(format!("market.{sym}.kline.{}", htx_kline_wire(interval)))
            }
            StreamKind::FundingRate => Ok(format!("public.{sym}.funding_rate")),
            StreamKind::Liquidation => Ok(format!("public.{sym}.liquidation_orders")),
            StreamKind::AggTrade => Err(WebSocketError::NotSupported(
                "HTX has no aggregated trade WS channel — \
                 subscribe StreamKind::Trade for raw fills via market.{sym}.trade.detail".to_string(),
            )),
            StreamKind::MarkPrice => Err(WebSocketError::NotSupported(
                "HTX does not have a direct WS mark price channel — \
                 use kline market.{sym}.mark_price.1min or REST mark_price endpoint".to_string(),
            )),
            StreamKind::OpenInterest => Err(WebSocketError::NotSupported(
                "HTX does not expose a realtime WS open interest feed — \
                 use REST GET /linear-swap-api/v1/swap_open_interest".to_string(),
            )),
            StreamKind::IndexPriceKline { .. } => Err(WebSocketError::NotSupported(
                "HTX: IndexPriceKline not available via WebSocket — use REST".into(),
            )),
            other => Err(WebSocketError::UnsupportedOperation(format!(
                "htx: unsupported stream kind {other:?}"
            ))),
        }
    }

    /// Spot registry (cached).
    fn spot_registry() -> &'static TopicRegistry {
        SPOT_REGISTRY.get_or_init(|| build_registry(AccountType::Spot))
    }

    /// Futures registry (cached).
    fn futures_registry() -> &'static TopicRegistry {
        FUTURES_REGISTRY.get_or_init(|| build_registry(AccountType::FuturesCross))
    }
}

impl WsProtocol for HtxProtocol {
    fn name(&self) -> &'static str {
        "htx"
    }

    fn endpoint(&self, account_type: AccountType, testnet: bool) -> Url {
        let url = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                HtxUrls::ws_linear_swap_url(testnet)
            }
            _ => HtxUrls::ws_market_url(testnet),
        };
        Url::parse(url).expect("htx ws url is valid")
    }

    /// HTX heartbeat is server-initiated — client does NOT send periodic pings.
    fn ping_frame(&self) -> Option<Message> {
        None
    }

    fn ping_interval(&self) -> Duration {
        // Not used (ping_frame returns None) but set to a sane value.
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let topic = Self::topic_for(spec)?;
        let frame = json!({ "sub": topic, "id": self.next_id() });
        Ok(Message::Text(frame.to_string()))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let topic = Self::topic_for(spec)?;
        let frame = json!({ "unsub": topic, "id": self.next_id() });
        Ok(Message::Text(frame.to_string()))
    }

    /// Public channels only — no auth frame.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        None
    }

    fn is_auth_ack(&self, _raw: &Value) -> bool {
        false
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // HTX pong frames are server-sent pings echoed by client — not applicable here.
        // Mark server "pong" acks if any: HTX doesn't send a pong to our pong.
        raw.get("pong").is_some()
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        // Subscription ack: {"id":"id1","status":"ok","subbed":"market.btcusdt.kline.1min","ts":...}
        raw.get("subbed").is_some()
            || raw.get("unsubbed").is_some()
            || (raw.get("status").and_then(|v| v.as_str()) == Some("ok")
                && raw.get("subbed").is_some())
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Filter server ping: {"ping": <ts>}
        if raw.get("ping").is_some() {
            return None;
        }

        // Filter subscription acks: {"status":"ok","subbed":...} or {"status":"ok","unsubbed":...}
        if raw.get("subbed").is_some() || raw.get("unsubbed").is_some() {
            return None;
        }

        // Filter pong echo (shouldn't appear, but guard)
        if raw.get("pong").is_some() {
            return None;
        }

        // Data frame: {"ch":"market.btcusdt.kline.1min","ts":...,"tick":{...}}
        let ch = raw.get("ch").and_then(|v| v.as_str())?;
        Some(TopicKey::new(ch))
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated | AccountType::Options => {
                Self::futures_registry()
            }
            _ => Self::spot_registry(),
        }
    }

    fn unsupported_by_exchange(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[StreamKind::MarkPrice, StreamKind::IndexPrice]
    }

    /// Override binary decode to gunzip HTX frames before JSON parsing.
    fn decode_binary(&self, bytes: &[u8]) -> Result<Value, WebSocketError> {
        let mut decoder = GzDecoder::new(bytes);
        let mut text = String::new();
        decoder
            .read_to_string(&mut text)
            .map_err(|e| WebSocketError::Parse(format!("htx gzip decode: {e}")))?;
        serde_json::from_str(&text)
            .map_err(|e| WebSocketError::Parse(format!("htx json parse: {e}")))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// URL helper (re-uses HtxUrls from endpoints)
// ─────────────────────────────────────────────────────────────────────────────

use super::endpoints::HtxUrls;

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry(account_type: AccountType) -> TopicRegistry {
    let mut b = TopicRegistry::builder();

    // Standard channels
    b = b
        .register(StreamKind::Ticker, account_type, "market.*.detail", parse_ticker)
        // BBO channel provides bid/ask for futures (market.BTC-USDT.bbo).
        // Spot .detail already carries bid/ask; futures .detail does NOT.
        .register(StreamKind::Ticker, account_type, "market.*.bbo", parse_bbo)
        .register(StreamKind::Trade, account_type, "market.*.trade.detail", parse_trade)
        .register(StreamKind::Orderbook, account_type, "market.*.depth.step0", parse_orderbook)
        .register(StreamKind::Orderbook, account_type, "market.*.depth.step1", parse_orderbook)
        .register(StreamKind::Orderbook, account_type, "market.*.depth.step2", parse_orderbook)
        .register(StreamKind::Orderbook, account_type, "market.*.depth.step3", parse_orderbook)
        .register(StreamKind::Orderbook, account_type, "market.*.depth.step4", parse_orderbook)
        .register(StreamKind::Orderbook, account_type, "market.*.depth.step5", parse_orderbook)
        .register(StreamKind::OrderbookDelta, account_type, "market.*.mbp.5", parse_orderbook_delta)
        .register(StreamKind::OrderbookDelta, account_type, "market.*.mbp.10", parse_orderbook_delta)
        .register(StreamKind::OrderbookDelta, account_type, "market.*.mbp.20", parse_orderbook_delta)
        .register(StreamKind::OrderbookDelta, account_type, "market.*.mbp.150", parse_orderbook_delta)
        .register(StreamKind::OrderbookDelta, account_type, "market.*.mbp.400", parse_orderbook_delta)
        .register(StreamKind::FundingRate, account_type, "public.*.funding_rate", parse_funding_rate)
        .register(StreamKind::Liquidation, account_type, "public.*.liquidation_orders", parse_liquidation);

    // Kline channels — one registry entry per HTX wire interval
    for (wire, internal) in HTX_KLINE_CHANNELS {
        let kind = StreamKind::Kline {
            interval: KlineInterval::new(*internal),
        };
        b = b.register(kind, account_type, format!("market.*.kline.{wire}"), parse_kline);
    }

    b.build()
}

/// HTX wire kline interval → internal KlineInterval string pairs.
const HTX_KLINE_CHANNELS: &[(&str, &str)] = &[
    ("1min",  "1m"),
    ("5min",  "5m"),
    ("15min", "15m"),
    ("30min", "30m"),
    ("60min", "1h"),
    ("4hour", "4h"),
    ("1day",  "1d"),
    ("1week", "1w"),
    ("1mon",  "1M"),
];

/// Map internal KlineInterval → HTX wire interval string.
fn htx_kline_wire(interval: &KlineInterval) -> &'static str {
    match interval.as_str() {
        "1m"  => "1min",
        "5m"  => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "1h"  => "60min",
        "4h"  => "4hour",
        "1d"  => "1day",
        "1w"  => "1week",
        "1M"  => "1mon",
        _     => "1min",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsers
//
// Each parser receives the full gzip-decoded JSON frame.
// HTX data frame shape: {"ch":"...","ts":<ms>,"tick":{...}}
// ─────────────────────────────────────────────────────────────────────────────

fn tick_data(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("tick")
        .ok_or_else(|| WebSocketError::Parse("htx frame missing 'tick' field".into()))
}

fn parse_f64_field(v: &Value) -> Option<f64> {
    v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
}

fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::Ticker;
    use crate::core::timestamp_millis;

    let channel = raw
        .get("ch")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let parts: Vec<&str> = channel.split('.').collect();
    let symbol = parts.get(1).copied().unwrap_or("").to_uppercase();

    let data = tick_data(raw)?;

    let last_price = data
        .get("close")
        .or_else(|| data.get("last_px"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx ticker: missing close/last_px".into()))?;

    let bid_price = data
        .get("bid")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(parse_f64_field);

    let ask_price = data
        .get("ask")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(parse_f64_field);

    Ok(StreamEvent::Ticker(Ticker {
        symbol,
        last_price,
        bid_price,
        ask_price,
        high_24h: data.get("high").and_then(parse_f64_field),
        low_24h: data.get("low").and_then(parse_f64_field),
        volume_24h: data.get("amount").and_then(parse_f64_field),
        quote_volume_24h: data.get("vol").and_then(parse_f64_field),
        price_change_24h: {
            let close = data.get("close").or_else(|| data.get("last_px")).and_then(parse_f64_field);
            let open = data.get("open").and_then(parse_f64_field);
            match (close, open) {
                (Some(c), Some(o)) => Some(c - o),
                _ => None,
            }
        },
        price_change_percent_24h: {
            let close = data.get("close").or_else(|| data.get("last_px")).and_then(parse_f64_field);
            let open = data.get("open").and_then(parse_f64_field);
            match (close, open) {
                (Some(c), Some(o)) if o != 0.0 => Some(((c - o) / o) * 100.0),
                _ => None,
            }
        },
        timestamp: raw.get("ts").and_then(|v| v.as_i64()).unwrap_or_else(|| timestamp_millis() as i64),
    }))
}

/// Parse HTX BBO frame: `{"ch":"market.BTC-USDT.bbo","ts":...,"tick":{"bid":[price,size],"ask":[price,size],...}}`
fn parse_bbo(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::Ticker;
    use crate::core::timestamp_millis;

    let channel = raw
        .get("ch")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let parts: Vec<&str> = channel.split('.').collect();
    let symbol = parts.get(1).copied().unwrap_or("").to_uppercase();

    let data = tick_data(raw)?;

    let bid_price = data
        .get("bid")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(parse_f64_field);

    let ask_price = data
        .get("ask")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(parse_f64_field);

    // BBO frames have no last price — use bid as a proxy so downstream has a non-zero value.
    let last_price = bid_price
        .ok_or_else(|| WebSocketError::Parse("htx bbo: missing bid".into()))?;

    Ok(StreamEvent::Ticker(Ticker {
        symbol,
        last_price,
        bid_price,
        ask_price,
        high_24h: None,
        low_24h: None,
        volume_24h: None,
        quote_volume_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: raw
            .get("ts")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| timestamp_millis() as i64),
    }))
}

fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::{PublicTrade, TradeSide};

    let channel = raw.get("ch").and_then(|v| v.as_str()).unwrap_or("");
    let parts: Vec<&str> = channel.split('.').collect();
    let symbol = parts.get(1).copied().unwrap_or("").to_uppercase();

    let data = tick_data(raw)?;

    // HTX trade tick: {"id":...,"ts":...,"data":[{"id":...,"ts":...,"tradeId":...,"amount":...,"price":...,"direction":"buy|sell"}]}
    let trades_arr = data
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| WebSocketError::Parse("htx trade tick missing data array".into()))?;

    if trades_arr.is_empty() {
        return Err(WebSocketError::Parse("htx trade tick: empty data array".into()));
    }

    let t = &trades_arr[0];
    let price = t
        .get("price")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx trade: missing price".into()))?;
    let quantity = t
        .get("amount")
        .and_then(parse_f64_field)
        .unwrap_or(0.0);
    let side = t
        .get("direction")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "buy" | "Buy" => TradeSide::Buy,
            _ => TradeSide::Sell,
        })
        .unwrap_or(TradeSide::Buy);
    let timestamp = t.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
    let trade_id = t
        .get("tradeId")
        .or_else(|| t.get("id"))
        .and_then(|v| v.as_i64())
        .map(|id| id.to_string())
        .unwrap_or_default();

    Ok(StreamEvent::Trade(PublicTrade {
        id: trade_id,
        symbol,
        price,
        quantity,
        side,
        timestamp,
    }))
}

fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::{OrderBook, OrderBookLevel};
    use crate::core::timestamp_millis;

    let data = tick_data(raw)?;

    let bids = data
        .get("bids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| WebSocketError::Parse("htx orderbook: missing bids".into()))?
        .iter()
        .filter_map(|entry| {
            let arr = entry.as_array()?;
            let price = arr.first().and_then(parse_f64_field)?;
            let size = arr.get(1).and_then(parse_f64_field)?;
            Some(OrderBookLevel::new(price, size))
        })
        .collect();

    let asks = data
        .get("asks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| WebSocketError::Parse("htx orderbook: missing asks".into()))?
        .iter()
        .filter_map(|entry| {
            let arr = entry.as_array()?;
            let price = arr.first().and_then(parse_f64_field)?;
            let size = arr.get(1).and_then(parse_f64_field)?;
            Some(OrderBookLevel::new(price, size))
        })
        .collect();

    let timestamp = raw
        .get("ts")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);
    let sequence = data.get("version").and_then(|v| v.as_i64()).map(|v| v.to_string());

    Ok(StreamEvent::OrderbookSnapshot(OrderBook {
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
    }))
}

fn parse_orderbook_delta(raw: &Value) -> WebSocketResult<StreamEvent> {
    // HTX MBP deltas — emit OrderbookDelta
    use crate::core::types::{OrderBookLevel, OrderbookDelta};
    use crate::core::timestamp_millis;

    let data = tick_data(raw)?;

    let bids = data
        .get("bids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    let a = entry.as_array()?;
                    let price = a.first().and_then(parse_f64_field)?;
                    let size = a.get(1).and_then(parse_f64_field)?;
                    Some(OrderBookLevel::new(price, size))
                })
                .collect()
        })
        .unwrap_or_default();

    let asks = data
        .get("asks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    let a = entry.as_array()?;
                    let price = a.first().and_then(parse_f64_field)?;
                    let size = a.get(1).and_then(parse_f64_field)?;
                    Some(OrderBookLevel::new(price, size))
                })
                .collect()
        })
        .unwrap_or_default();

    let timestamp = raw
        .get("ts")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::OrderbookDelta(OrderbookDelta {
        bids,
        asks,
        timestamp,
        first_update_id: None,
        last_update_id: None,
        prev_update_id: None,
        event_time: None,
        checksum: None,
    }))
}

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::Kline;

    let data = tick_data(raw)?;

    let open_time = data
        .get("id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| WebSocketError::Parse("htx kline: missing id".into()))?
        * 1000; // seconds → ms
    let open = data
        .get("open")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx kline: missing open".into()))?;
    let high = data
        .get("high")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx kline: missing high".into()))?;
    let low = data
        .get("low")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx kline: missing low".into()))?;
    let close = data
        .get("close")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx kline: missing close".into()))?;
    let volume = data.get("amount").and_then(parse_f64_field).unwrap_or(0.0);
    let quote_volume = data.get("vol").and_then(parse_f64_field);
    let trades = data.get("count").and_then(|v| v.as_i64()).map(|c| c as u64);

    Ok(StreamEvent::Kline(Kline {
        open_time,
        open,
        high,
        low,
        close,
        volume,
        quote_volume,
        close_time: None,
        trades,
    }))
}

fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let channel = raw.get("ch").and_then(|v| v.as_str()).unwrap_or("");
    let symbol = channel.split('.').nth(1).unwrap_or("").to_uppercase();

    let data = tick_data(raw)?;

    let rate = data
        .get("funding_rate")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx funding_rate: missing funding_rate".into()))?;
    let next_funding_time = data
        .get("funding_time")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64);
    let timestamp = data
        .get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);

    Ok(StreamEvent::FundingRate {
        symbol,
        rate,
        next_funding_time,
        timestamp,
    })
}

fn parse_liquidation(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    // liquidation_orders: topic is "public.<code>.liquidation_orders"
    let channel = raw.get("ch").and_then(|v| v.as_str()).unwrap_or("");
    let symbol = channel.split('.').nth(1).unwrap_or("").to_uppercase();

    let data = tick_data(raw)?;

    let price = data
        .get("price")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("htx liquidation: missing price".into()))?;
    let quantity = data
        .get("amount")
        .or_else(|| data.get("volume"))
        .and_then(parse_f64_field)
        .unwrap_or(0.0);
    let side = data
        .get("direction")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "buy" | "Buy" => TradeSide::Buy,
            _ => TradeSide::Sell,
        })
        .unwrap_or(TradeSide::Sell);
    let timestamp = data
        .get("ts")
        .and_then(parse_f64_field)
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
            symbol: crate::core::types::OwnedSymbolInput::Raw("btcusdt".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = HtxProtocol::new(AccountType::Spot, false);
        let reg = proto.topic_registry(AccountType::Spot);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "spot registry must have entries");
        assert!(reg.supports(&StreamKind::Ticker, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Orderbook, AccountType::Spot));
        assert!(reg.supports(&StreamKind::FundingRate, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Liquidation, AccountType::Spot));
        assert!(reg.supports(
            &StreamKind::Kline { interval: KlineInterval::new("1m") },
            AccountType::Spot
        ));
    }

    #[test]
    fn test_subscribe_frame_kline() {
        let proto = HtxProtocol::new(AccountType::Spot, false);
        let spec = spot_spec(StreamKind::Kline { interval: KlineInterval::new("1m") });
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["sub"], "market.btcusdt.kline.1min");
        assert!(v["id"].as_str().is_some());
    }

    #[test]
    fn test_extract_topic_kline_frame() {
        let proto = HtxProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "ch": "market.btcusdt.kline.1min",
            "ts": 1629384000000i64,
            "tick": {
                "id": 1629384000i64,
                "open": 48000.0,
                "close": 49500.0,
                "low": 47500.0,
                "high": 50000.0,
                "amount": 18344.5,
                "vol": 896748251.0,
                "count": 89472
            }
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "market.btcusdt.kline.1min");
    }

    #[test]
    fn test_extract_topic_ping_returns_none() {
        let proto = HtxProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({ "ping": 1629384000000i64 });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_gzip_decode() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let json_str = r#"{"ping":1629384000000}"#;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json_str.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        let proto = HtxProtocol::new(AccountType::Spot, false);
        let val = proto.decode_binary(&compressed).expect("gzip decode must succeed");
        assert_eq!(val["ping"], 1629384000000i64);
    }
}
