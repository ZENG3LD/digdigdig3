//! GateIoProtocol — WsProtocol implementation for the Gate.io exchange.
//!
//! Declarative shim: supplies endpoint URLs, ping frame, subscribe/unsubscribe
//! frames, topic extraction, and topic registry to UniversalWsTransport.
//!
//! Gate.io uses per-product-line WebSocket URLs and channel prefixes:
//!   - Spot:              spot.*
//!   - Futures (USDT):    futures.*
//!   - Futures (BTC):     futures.*   (different URL)
//!   - Delivery futures:  delivery.*
//!   - Options:           options.*
//!
//! Symbol format: BASE_QUOTE (underscore separator), e.g. BTC_USDT.

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
use crate::core::timestamp_seconds;

use super::parser::GateioParser;

// ─────────────────────────────────────────────────────────────────────────────
// Category enum — maps to endpoint + channel prefix
// ─────────────────────────────────────────────────────────────────────────────

/// Gate.io product line, determines WS endpoint URL and channel prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateIoCategory {
    Spot,
    FuturesUsdt,
    FuturesBtc,
    DeliveryUsdt,
    Options,
}

impl GateIoCategory {
    /// Channel prefix for this category (e.g. "spot" → "spot.trades").
    pub fn channel_prefix(self) -> &'static str {
        match self {
            GateIoCategory::Spot => "spot",
            GateIoCategory::FuturesUsdt | GateIoCategory::FuturesBtc => "futures",
            GateIoCategory::DeliveryUsdt => "delivery",
            GateIoCategory::Options => "options",
        }
    }

    /// WS ping channel for this category.
    pub fn ping_channel(self) -> &'static str {
        match self {
            GateIoCategory::Spot => "spot.ping",
            GateIoCategory::FuturesUsdt | GateIoCategory::FuturesBtc => "futures.ping",
            GateIoCategory::DeliveryUsdt => "delivery.ping",
            GateIoCategory::Options => "options.ping",
        }
    }

    /// Map AccountType → GateIoCategory.
    pub fn from_account_type(account_type: AccountType) -> Self {
        match account_type {
            AccountType::Spot | AccountType::Margin => GateIoCategory::Spot,
            AccountType::FuturesCross | AccountType::FuturesIsolated => GateIoCategory::FuturesUsdt,
            AccountType::Options => GateIoCategory::Options,
            _ => GateIoCategory::Spot,
        }
    }

    /// Mainnet WS endpoint URL.
    pub fn ws_url(self, testnet: bool) -> &'static str {
        if testnet {
            return match self {
                GateIoCategory::Spot => "wss://api-testnet.gateapi.io/ws/v4/",
                GateIoCategory::FuturesUsdt | GateIoCategory::FuturesBtc => {
                    "wss://fx-ws-testnet.gateio.ws/v4/ws/usdt"
                }
                GateIoCategory::DeliveryUsdt => "wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt",
                GateIoCategory::Options => "wss://op-ws-testnet.gateio.live/v4/ws",
            };
        }
        match self {
            GateIoCategory::Spot => "wss://api.gateio.ws/ws/v4/",
            GateIoCategory::FuturesUsdt => "wss://fx-ws.gateio.ws/v4/ws/usdt",
            GateIoCategory::FuturesBtc => "wss://fx-ws.gateio.ws/v4/ws/btc",
            GateIoCategory::DeliveryUsdt => "wss://fx-ws.gateio.ws/v4/ws/delivery/usdt",
            GateIoCategory::Options => "wss://op-ws.gateio.live/v4/ws",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry caches — one per category
// ─────────────────────────────────────────────────────────────────────────────

static SPOT_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static FUTURES_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static DELIVERY_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static OPTIONS_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// GateIoProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Gate.io WS protocol shim.
pub struct GateIoProtocol {
    account_type: AccountType,
}

impl GateIoProtocol {
    pub fn new(account_type: AccountType, _testnet: bool) -> Self {
        Self { account_type }
    }

    fn category(&self) -> GateIoCategory {
        GateIoCategory::from_account_type(self.account_type)
    }

    /// Build subscribe/unsubscribe frame.
    fn build_frame(op: &str, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let category = GateIoCategory::from_account_type(spec.account_type);
        let prefix = category.channel_prefix();

        let (channel, payload) = channel_and_payload(prefix, spec)?;

        let ts = timestamp_seconds() as i64;
        let frame = if payload.is_empty() {
            json!({
                "time": ts,
                "channel": channel,
                "event": op,
            })
        } else {
            json!({
                "time": ts,
                "channel": channel,
                "event": op,
                "payload": payload,
            })
        };

        Ok(Message::Text(frame.to_string()))
    }

    fn spot_registry() -> &'static TopicRegistry {
        SPOT_REGISTRY.get_or_init(|| build_registry(GateIoCategory::Spot))
    }

    fn futures_registry() -> &'static TopicRegistry {
        FUTURES_REGISTRY.get_or_init(|| build_registry(GateIoCategory::FuturesUsdt))
    }

    fn delivery_registry() -> &'static TopicRegistry {
        DELIVERY_REGISTRY.get_or_init(|| build_registry(GateIoCategory::DeliveryUsdt))
    }

    fn options_registry() -> &'static TopicRegistry {
        OPTIONS_REGISTRY.get_or_init(|| build_registry(GateIoCategory::Options))
    }
}

impl WsProtocol for GateIoProtocol {
    fn name(&self) -> &'static str {
        "gateio"
    }

    fn endpoint(&self, account_type: AccountType, testnet: bool) -> Url {
        let cat = GateIoCategory::from_account_type(account_type);
        Url::parse(cat.ws_url(testnet)).expect("gateio ws url is valid")
    }

    fn ping_frame(&self) -> Option<Message> {
        let ping_channel = self.category().ping_channel();
        let ts = timestamp_seconds() as i64;
        let frame = json!({ "time": ts, "channel": ping_channel });
        Some(Message::Text(frame.to_string()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(20)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        // Public WS only — Gate.io private channels use per-message auth in the payload
        None
    }

    fn is_auth_ack(&self, _raw: &Value) -> bool {
        false
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // Gate.io pong: {"channel":"spot.pong"} or {"channel":"futures.pong"}
        raw.get("channel")
            .and_then(|c| c.as_str())
            .map(|c| c.ends_with(".pong"))
            .unwrap_or(false)
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        // {"event":"subscribe","result":{"status":"success"}} or {"event":"unsubscribe",...}
        let event = raw.get("event").and_then(|v| v.as_str());
        matches!(event, Some("subscribe") | Some("unsubscribe"))
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Pong frames
        if self.is_pong(raw) {
            return None;
        }

        // Subscribe/unsubscribe ack
        let event = raw.get("event").and_then(|v| v.as_str());
        if matches!(event, Some("subscribe") | Some("unsubscribe")) {
            return None;
        }

        // Data frames: {"event":"update","channel":"spot.trades","result":{...}}
        let channel = raw.get("channel").and_then(|c| c.as_str())?;

        // Only emit topics for "update" events (not acks)
        if event != Some("update") {
            return None;
        }

        Some(TopicKey::new(channel))
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        match GateIoCategory::from_account_type(account_type) {
            GateIoCategory::Spot => Self::spot_registry(),
            GateIoCategory::FuturesUsdt | GateIoCategory::FuturesBtc => Self::futures_registry(),
            GateIoCategory::DeliveryUsdt => Self::delivery_registry(),
            GateIoCategory::Options => Self::options_registry(),
        }
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[StreamKind::OrderUpdate, StreamKind::BalanceUpdate, StreamKind::PositionUpdate]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Channel + payload builder
// ─────────────────────────────────────────────────────────────────────────────

/// Format a Gate.io symbol from base+quote: BTC_USDT.
pub fn format_gateio_symbol(base: &str, quote: &str) -> String {
    format!("{}_{}",  base.to_uppercase(), quote.to_uppercase())
}

/// Map StreamSpec → (channel, payload) for Gate.io.
fn channel_and_payload(
    prefix: &str,
    spec: &StreamSpec,
) -> Result<(String, Vec<String>), WebSocketError> {
    let sym = format_gateio_symbol(&spec.symbol.base, &spec.symbol.quote);

    let (channel_suffix, payload) = match &spec.kind {
        StreamKind::Ticker => ("tickers", vec![sym]),
        StreamKind::Trade => ("trades", vec![sym]),
        StreamKind::Orderbook => {
            let depth = spec.depth.unwrap_or(20).to_string();
            let speed = spec.speed_ms
                .map(|ms| format!("{}ms", ms))
                .unwrap_or_else(|| "1000ms".to_string());
            ("order_book", vec![sym, depth, speed])
        }
        StreamKind::OrderbookDelta => {
            let depth = spec.depth.unwrap_or(20).to_string();
            let speed = spec.speed_ms
                .map(|ms| format!("{}ms", ms))
                .unwrap_or_else(|| "1000ms".to_string());
            ("order_book_update", vec![sym, depth, speed])
        }
        StreamKind::Kline { interval } => {
            // Gate.io candlestick payload: [interval_str, symbol]
            ("candlesticks", vec![interval.as_str().to_string(), sym])
        }
        StreamKind::MarkPriceKline { interval } => {
            // mark price candles: symbol prefixed with "mark_"
            ("candlesticks", vec![interval.as_str().to_string(), format!("mark_{}", sym)])
        }
        StreamKind::MarkPrice => ("tickers", vec![sym]),
        StreamKind::FundingRate => ("tickers", vec![sym]),
        StreamKind::Liquidation => ("liquidates", vec![sym]),
        StreamKind::OrderUpdate => ("orders", vec![sym]),
        StreamKind::BalanceUpdate => ("balances", vec![]),
        StreamKind::PositionUpdate => ("positions", vec![sym]),
        // Spec §3.3: register all known futures-only channels
        StreamKind::OpenInterest => ("contract_stats", vec![sym]),
        other => {
            return Err(WebSocketError::UnsupportedOperation(format!(
                "gateio: unsupported stream kind {:?}",
                other
            )));
        }
    };

    Ok((format!("{}.{}", prefix, channel_suffix), payload))
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry(category: GateIoCategory) -> TopicRegistry {
    let mut b = TopicRegistry::builder();
    let prefix = category.channel_prefix();

    // Channels present in ALL categories
    b = b
        .register(StreamKind::Ticker,        AccountType::Spot, format!("{}.tickers", prefix), parse_ticker)
        .register(StreamKind::Trade,         AccountType::Spot, format!("{}.trades", prefix), parse_trade)
        .register(StreamKind::Orderbook,     AccountType::Spot, format!("{}.order_book", prefix), parse_orderbook)
        .register(StreamKind::OrderbookDelta, AccountType::Spot, format!("{}.order_book_update", prefix), parse_orderbook_delta)
        .register(StreamKind::OrderUpdate,   AccountType::Spot, format!("{}.orders", prefix), parse_order_update)
        .register(StreamKind::BalanceUpdate, AccountType::Spot, format!("{}.balances", prefix), parse_balance_update);

    // Candlestick — single pattern covers all intervals (channel name stays the same)
    b = b.register(
        StreamKind::Kline { interval: KlineInterval::new("1m") },
        AccountType::Spot,
        format!("{}.candlesticks", prefix),
        parse_kline,
    );

    // Futures-only channels
    match category {
        GateIoCategory::FuturesUsdt
        | GateIoCategory::FuturesBtc
        | GateIoCategory::DeliveryUsdt => {
            b = b
                .register(StreamKind::MarkPrice,      AccountType::Spot, format!("{}.mark_price", prefix), parse_mark_price)
                .register(StreamKind::FundingRate,     AccountType::Spot, format!("{}.funding_rate", prefix), parse_funding_rate)
                .register(StreamKind::Liquidation,     AccountType::Spot, format!("{}.liquidates", prefix), parse_liquidation)
                .register(StreamKind::PositionUpdate,  AccountType::Spot, format!("{}.positions", prefix), parse_position_update)
                .register(StreamKind::OpenInterest,    AccountType::Spot, format!("{}.contract_stats", prefix), parse_contract_stats);
        }
        GateIoCategory::Spot | GateIoCategory::Options => {}
    }

    b.build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsers  (receive full Gate.io frame: {"event":"update","channel":"...","result":{...}})
// ─────────────────────────────────────────────────────────────────────────────

/// Extract `result` from a Gate.io data frame.
fn frame_result(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("result")
        .ok_or_else(|| WebSocketError::Parse("gateio frame missing 'result' field".into()))
}

fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = frame_result(raw)?;
    let ticker = GateioParser::parse_ws_ticker(result)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Ticker(ticker))
}

fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = frame_result(raw)?;
    let trade = GateioParser::parse_ws_trade(result)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Trade(trade))
}

fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::OrderBookLevel;
    let result = frame_result(raw)?;

    let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
        result
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|level| {
                        let pair = level.as_array()?;
                        if pair.len() < 2 {
                            return None;
                        }
                        let price = pair[0].as_str()?.parse::<f64>().ok()?;
                        let size = pair[1].as_str()?.parse::<f64>().ok()?;
                        Some(OrderBookLevel::new(price, size))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    Ok(StreamEvent::OrderbookSnapshot(crate::core::OrderBook {
        timestamp: result.get("t").and_then(|t| t.as_i64()).unwrap_or(0),
        bids: parse_levels("bids"),
        asks: parse_levels("asks"),
        sequence: result
            .get("lastUpdateId")
            .and_then(|s| s.as_i64())
            .map(|n| n.to_string()),
        last_update_id: None,
        first_update_id: None,
        prev_update_id: None,
        event_time: None,
        transaction_time: None,
        checksum: None,
    }))
}

fn parse_orderbook_delta(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::{OrderbookDelta as OrderbookDeltaData, OrderBookLevel};
    let result = frame_result(raw)?;

    let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
        result
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|level| {
                        let pair = level.as_array()?;
                        if pair.len() < 2 {
                            return None;
                        }
                        let price = pair[0].as_str()?.parse::<f64>().ok()?;
                        let size = pair[1].as_str()?.parse::<f64>().ok()?;
                        Some(OrderBookLevel::new(price, size))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let delta = OrderbookDeltaData {
        bids: parse_levels("bids"),
        asks: parse_levels("asks"),
        timestamp: result.get("t").and_then(|v| v.as_i64()).unwrap_or(0),
        last_update_id: result.get("lastUpdateId").and_then(|v| v.as_u64()),
        first_update_id: None,
        prev_update_id: None,
        event_time: None,
        checksum: None,
    };
    Ok(StreamEvent::OrderbookDelta(delta))
}

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    // Gate.io candlestick result: {"t":"ts","v":"vol","c":"close","h":"high","l":"low","o":"open","n":"symbol","a":"quote_vol"}
    // Mark price candlestick: "n" field starts with "mark_"
    let result = frame_result(raw)?;

    let symbol_name = result.get("n").and_then(|v| v.as_str()).unwrap_or("");
    let kline = parse_kline_data(result)?;

    if symbol_name.starts_with("mark_") {
        let clean_symbol = symbol_name.strip_prefix("mark_").unwrap_or(symbol_name).to_string();
        Ok(StreamEvent::MarkPriceKline {
            symbol: clean_symbol,
            interval: String::new(),
            kline,
        })
    } else if symbol_name.starts_with("premium_index_") {
        let clean_symbol = symbol_name
            .strip_prefix("premium_index_")
            .unwrap_or(symbol_name)
            .to_string();
        Ok(StreamEvent::IndexPriceKline {
            symbol: clean_symbol,
            interval: String::new(),
            kline,
        })
    } else {
        Ok(StreamEvent::Kline(kline))
    }
}

fn parse_kline_data(data: &Value) -> WebSocketResult<crate::core::Kline> {
    let open_time = data
        .get("t")
        .and_then(|t| t.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0)
        * 1000; // seconds → ms

    let parse_f64 = |key: &str| -> f64 {
        data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
    };

    Ok(crate::core::Kline {
        open_time,
        open: parse_f64("o"),
        high: parse_f64("h"),
        low: parse_f64("l"),
        close: parse_f64("c"),
        volume: parse_f64("v"),
        quote_volume: Some(parse_f64("a")),
        close_time: None,
        trades: None,
    })
}

fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = frame_result(raw)?;
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };
    let symbol = result.get("contract").and_then(|v| v.as_str())
        .or_else(|| result.get("s").and_then(|v| v.as_str()))
        .unwrap_or("").to_string();
    let mark_price = parse_f64(result.get("mark_price").unwrap_or(&Value::Null))
        .or_else(|| parse_f64(result.get("p").unwrap_or(&Value::Null)))
        .unwrap_or(0.0);
    let index_price = parse_f64(result.get("index_price").unwrap_or(&Value::Null));
    let timestamp = result.get("t").and_then(|v| v.as_i64()).unwrap_or(0);
    Ok(StreamEvent::MarkPrice { symbol, mark_price, index_price, timestamp })
}

fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = frame_result(raw)?;
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };
    let symbol = result.get("contract").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let rate = parse_f64(result.get("r").unwrap_or(&Value::Null)).unwrap_or(0.0);
    let next_funding_time = result.get("t").and_then(|v| v.as_i64());
    let timestamp = result.get("t").and_then(|v| v.as_i64()).unwrap_or(0);
    Ok(StreamEvent::FundingRate { symbol, rate, next_funding_time, timestamp })
}

fn parse_liquidation(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    let result = frame_result(raw)?;
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };
    // data may be array-wrapped
    let item = if let Some(arr) = result.as_array() {
        arr.first().cloned().unwrap_or(Value::Null)
    } else {
        result.clone()
    };
    let symbol = item.get("contract").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let price = parse_f64(item.get("price").unwrap_or(&Value::Null)).unwrap_or(0.0);
    let quantity = parse_f64(item.get("size").unwrap_or(&Value::Null))
        .map(|v| v.abs())
        .unwrap_or(0.0);
    // is_short=true → short position was liquidated (forced buy to close)
    let side = item
        .get("is_short")
        .and_then(|v| v.as_bool())
        .map(|is_short| if is_short { TradeSide::Buy } else { TradeSide::Sell })
        .unwrap_or(TradeSide::Sell);
    let timestamp = item.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
    Ok(StreamEvent::Liquidation { symbol, side, price, quantity, value: None, timestamp })
}

fn parse_order_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = frame_result(raw)?;
    let event = GateioParser::parse_ws_order_update(result)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderUpdate(event))
}

fn parse_balance_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = frame_result(raw)?;
    let event = GateioParser::parse_ws_balance_update(result)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::BalanceUpdate(event))
}

fn parse_position_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = frame_result(raw)?;
    let event = GateioParser::parse_ws_position_update(result)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::PositionUpdate(event))
}

fn parse_contract_stats(raw: &Value) -> WebSocketResult<StreamEvent> {
    // Gate.io contract_stats: open interest + volume + long/short ratio
    let result = frame_result(raw)?;
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };
    let symbol = result.get("contract").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let open_interest = parse_f64(result.get("open_interest").unwrap_or(&Value::Null)).unwrap_or(0.0);
    let timestamp = result.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
    Ok(StreamEvent::OpenInterestUpdate { symbol, open_interest, open_interest_value: None, timestamp })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Symbol;
    use crate::core::websocket::StreamSpec;

    fn spot_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: Symbol::new("BTC", "USDT"),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = GateIoProtocol::new(AccountType::Spot, false);
        let reg = proto.topic_registry(AccountType::Spot);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "spot registry must have entries");
        assert!(reg.supports(&StreamKind::Ticker, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(reg.supports(
            &StreamKind::Kline { interval: KlineInterval::new("1m") },
            AccountType::Spot
        ));
    }

    #[test]
    fn test_subscribe_frame_spot_trades() {
        let proto = GateIoProtocol::new(AccountType::Spot, false);
        let spec = spot_spec(StreamKind::Trade);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["event"], "subscribe");
        assert_eq!(v["channel"], "spot.trades");
        let payload = v["payload"].as_array().expect("payload array");
        assert_eq!(payload[0], "BTC_USDT");
    }

    #[test]
    fn test_extract_topic_trades_frame() {
        let proto = GateIoProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "time": 1234567890,
            "channel": "spot.trades",
            "event": "update",
            "result": { "id": 1, "create_time": 1234 }
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "spot.trades");
    }

    #[test]
    fn test_extract_topic_subscribe_ack_returns_none() {
        let proto = GateIoProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "time": 1234,
            "channel": "spot.trades",
            "event": "subscribe",
            "result": { "status": "success" }
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_extract_topic_pong_returns_none() {
        let proto = GateIoProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({ "channel": "spot.pong" });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_symbol_format_underscore() {
        let sym = format_gateio_symbol("BTC", "USDT");
        assert_eq!(sym, "BTC_USDT");
        assert!(!sym.contains('-'));
        assert!(!sym.contains("BTCUSDT"));
    }

    #[test]
    fn test_ping_frame_contains_channel() {
        let proto = GateIoProtocol::new(AccountType::Spot, false);
        let frame = proto.ping_frame().expect("ping frame must exist");
        let text = match frame {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["channel"], "spot.ping");
    }

    #[test]
    fn test_futures_registry_has_liquidation() {
        let proto = GateIoProtocol::new(AccountType::FuturesCross, false);
        let reg = proto.topic_registry(AccountType::FuturesCross);
        assert!(reg.supports(&StreamKind::Liquidation, AccountType::Spot));
        assert!(reg.supports(&StreamKind::FundingRate, AccountType::Spot));
        assert!(reg.supports(&StreamKind::MarkPrice, AccountType::Spot));
    }
}
