//! DeribitProtocol — WsProtocol implementation for Deribit (JSON-RPC 2.0).
//!
//! Deribit uses JSON-RPC 2.0 over WebSocket.  Subscribe/unsubscribe frames carry
//! a monotonically increasing `id` field.  Data frames arrive as:
//!   `{"jsonrpc":"2.0","method":"subscription","params":{"channel":"...","data":{...}}}`
//!
//! Topic routing key = `params.channel`.
//!
//! ## Options note
//! Options channels require a concrete `instrument_name` (e.g. `BTC-30MAY26-50000-C`).
//! The registry patterns `book.*.100ms` match them naturally.  Consumers MUST
//! supply instrument-resolved StreamSpec for Options (generic Symbol is not enough).
//!
//! ## JSON-RPC ping
//! Client sends `{"jsonrpc":"2.0","id":N,"method":"public/test"}` every 30 s.
//! Server replies `{"jsonrpc":"2.0","id":N,"result":{"version":"..."}}`.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chrono::Utc;
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, BalanceChangeReason, BalanceUpdateEvent, PositionSide,
    StreamEvent, Ticker, TradeSide, WebSocketError, WebSocketResult,
};
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol,
};

use super::parser::DeribitParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache (single registry — Deribit channel namespace is unified)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// DeribitProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// WsProtocol shim for Deribit JSON-RPC 2.0 WebSocket API.
pub struct DeribitProtocol {
    _account_type: AccountType,
    _testnet: bool,
    next_id: AtomicU64,
}

impl DeribitProtocol {
    pub fn new(account_type: AccountType, testnet: bool) -> Self {
        Self {
            _account_type: account_type,
            _testnet: testnet,
            next_id: AtomicU64::new(1),
        }
    }

    /// Fetch-and-increment JSON-RPC request id.
    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Convert StreamSpec → Deribit channel name.
    ///
    /// Returns `None` for unsupported kinds.
    fn channel_name(spec: &StreamSpec) -> Option<String> {
        let instrument = deribit_instrument(&spec.symbol.base, &spec.symbol.quote);

        let ch = match &spec.kind {
            StreamKind::Ticker => format!("ticker.{}.100ms", instrument),
            StreamKind::Trade => format!("trades.{}.100ms", instrument),
            StreamKind::Orderbook => format!("book.{}.100ms", instrument),
            StreamKind::OrderbookDelta => format!("book.{}.100ms", instrument),
            StreamKind::Kline { interval } => {
                // Deribit uses chart.trades.<instrument>.<resolution>
                // Resolution values: 1 3 5 10 15 30 60 120 180 360 720 1D
                let res = deribit_kline_resolution(interval);
                format!("chart.trades.{}.{}", instrument, res)
            }
            StreamKind::MarkPrice => format!("mark_price.{}", instrument),
            StreamKind::FundingRate => format!("perpetual.{}.100ms", instrument),
            StreamKind::IndexPrice => {
                // e.g. deribit_price_index.btc_usd — base in lowercase + _usd
                let idx = format!("{}_usd", spec.symbol.base.to_lowercase());
                format!("deribit_price_index.{}", idx)
            }
            StreamKind::OptionGreeks => format!("ticker.{}.100ms", instrument),
            StreamKind::VolatilityIndex => {
                let idx = format!("{}_usd", spec.symbol.base.to_lowercase());
                format!("deribit_volatility_index.{}", idx)
            }
            StreamKind::OrderUpdate => "user.orders.any.any.raw".to_string(),
            StreamKind::BalanceUpdate => {
                // Multiple settlement currencies — comma-joined; caller must fan out.
                // The subscribe_frame will call subscribe once per channel.
                // We return only BTC for the primary channel; the multi-currency
                // fan-out is handled by the connector layer (not protocol).
                "user.portfolio.BTC,user.portfolio.ETH,user.portfolio.USDC,user.portfolio.USDT,user.portfolio.SOL".to_string()
            }
            StreamKind::PositionUpdate => "user.changes.any.any.raw".to_string(),
            StreamKind::BlockTrade => "block_trade_confirmations".to_string(),
            _ => return None,
        };

        Some(ch)
    }

    /// Build subscribe or unsubscribe JSON-RPC 2.0 frame.
    fn build_sub_frame(&self, op: &str, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let channel_str = Self::channel_name(spec)
            .ok_or_else(|| WebSocketError::UnsupportedOperation(
                format!("deribit: unsupported stream kind {:?}", spec.kind),
            ))?;

        // Handle comma-joined multi-channel (BalanceUpdate fan-out)
        let channels: Vec<String> = channel_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let is_private = spec.kind.is_private();
        let method = if is_private {
            if op == "subscribe" { "private/subscribe" } else { "private/unsubscribe" }
        } else if op == "subscribe" {
            "public/subscribe"
        } else {
            "public/unsubscribe"
        };

        let id = self.next_id();
        let frame = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": { "channels": channels }
        });

        Ok(Message::Text(frame.to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for DeribitProtocol {
    fn name(&self) -> &'static str {
        "deribit"
    }

    fn endpoint(&self, _account_type: AccountType, testnet: bool) -> Url {
        let url = if testnet {
            "wss://test.deribit.com/ws/api/v2"
        } else {
            "wss://www.deribit.com/ws/api/v2"
        };
        Url::parse(url).expect("deribit ws url is valid")
    }

    fn ping_frame(&self) -> Option<Message> {
        // JSON-RPC public/test ping
        let id = self.next_id();
        let frame = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "public/test"
        });
        Some(Message::Text(frame.to_string()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        self.build_sub_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        self.build_sub_frame("unsubscribe", spec)
    }

    fn auth_frame(&self, credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        // Deribit auth: public/auth with client_credentials grant
        let id = self.next_id();
        let frame = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "public/auth",
            "params": {
                "grant_type": "client_credentials",
                "client_id": credentials.api_key,
                "client_secret": credentials.api_secret,
            }
        });
        Some(Ok(Message::Text(frame.to_string())))
    }

    fn auth_ack_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn is_auth_ack(&self, raw: &Value) -> bool {
        // Auth success: {"jsonrpc":"2.0","id":N,"result":{"access_token":...}}
        raw.get("result")
            .and_then(|r| r.get("access_token"))
            .is_some()
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // Deribit ping response: {"jsonrpc":"2.0","id":N,"result":{"version":"X.Y.Z"}}
        // Also heartbeat test_request reply goes here as response
        if raw.get("id").is_some() {
            if let Some(result) = raw.get("result") {
                // public/test response has "version" field
                if result.get("version").is_some() {
                    return true;
                }
                // subscribe ack: result is an array of channel strings
                if result.is_array() {
                    return false; // let is_subscribe_ack handle it
                }
            }
        }
        false
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        // Subscribe response: {"jsonrpc":"2.0","id":N,"result":["channel1","channel2"]}
        // Result is an array of strings (the subscribed channels)
        if raw.get("id").is_some() {
            if let Some(result) = raw.get("result") {
                if let Some(arr) = result.as_array() {
                    return arr.iter().all(|v| v.is_string());
                }
                // null result (unsubscribe from nothing, or empty result)
                if result.is_null() {
                    return true;
                }
            }
        }
        false
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Deribit server heartbeat: {"method":"heartbeat","params":{"type":"test_request"}}
        // — not a data frame, return None
        if raw.get("method").and_then(|m| m.as_str()) == Some("heartbeat") {
            return None;
        }

        // Data frame: {"jsonrpc":"2.0","method":"subscription","params":{"channel":"...","data":{...}}}
        if raw.get("method").and_then(|m| m.as_str()) == Some("subscription") {
            let channel = raw
                .get("params")
                .and_then(|p| p.get("channel"))
                .and_then(|c| c.as_str())?;
            return Some(TopicKey::new(channel));
        }

        // All other frames with id (subscribe ack, ping response, auth response):
        // pong / subscribe_ack handlers cover these → return None
        None
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        // Single unified registry — Deribit channel namespace does not split by account type
        REGISTRY.get_or_init(build_registry)
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[StreamKind::OrderUpdate, StreamKind::BalanceUpdate, StreamKind::PositionUpdate]
    }
}

// Override endpoint to use instance testnet flag (protocol stores it)
// The trait method signature takes testnet as param — we delegate correctly above.
// Note: DeribitProtocol stores `testnet` only for future use; the trait param is the source of truth.
// The stored field is used when ping_frame needs to embed the real endpoint (not needed).

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    let at = AccountType::FuturesCross; // representative; dispatch ignores account_type

    let mut b = TopicRegistry::builder();

    // Orderbook — snapshot + delta share same channel pattern
    b = b
        .register(StreamKind::Orderbook,      at, "book.*.raw",    parse_orderbook)
        .register(StreamKind::Orderbook,      at, "book.*.100ms",  parse_orderbook)
        .register(StreamKind::OrderbookDelta, at, "book.*.raw",    parse_orderbook)
        .register(StreamKind::OrderbookDelta, at, "book.*.100ms",  parse_orderbook);

    // Trades
    b = b
        .register(StreamKind::Trade, at, "trades.*.raw",   parse_trade)
        .register(StreamKind::Trade, at, "trades.*.100ms", parse_trade);

    // Ticker (also sources OptionGreeks + MarkPrice + FundingRate)
    b = b
        .register(StreamKind::Ticker,       at, "ticker.*.raw",   parse_ticker)
        .register(StreamKind::Ticker,       at, "ticker.*.100ms", parse_ticker)
        .register(StreamKind::OptionGreeks, at, "ticker.*.raw",   parse_ticker)
        .register(StreamKind::OptionGreeks, at, "ticker.*.100ms", parse_ticker);

    // Quote (best bid/ask — high frequency)
    b = b.register(StreamKind::Ticker, at, "quote.*", parse_quote);

    // Kline (chart.trades.<instrument>.<resolution>)
    for res in DERIBIT_KLINE_RESOLUTIONS {
        let kind = StreamKind::Kline {
            interval: KlineInterval::new(internal_kline_interval(res)),
        };
        let pattern = format!("chart.trades.*.{}", res);
        b = b.register(kind, at, pattern, parse_kline);
    }

    // Mark price
    b = b.register(StreamKind::MarkPrice, at, "mark_price.*", parse_mark_price);

    // Perpetual interest rate (→ FundingRate)
    b = b
        .register(StreamKind::FundingRate, at, "perpetual.*.raw",   parse_perpetual)
        .register(StreamKind::FundingRate, at, "perpetual.*.100ms", parse_perpetual);

    // Index price
    b = b.register(StreamKind::IndexPrice, at, "deribit_price_index.*", parse_index_price);

    // Estimated expiration price (→ IndexPrice)
    b = b.register(StreamKind::IndexPrice, at, "estimated_expiration_price.*", parse_estimated_expiration);

    // Volatility index
    b = b.register(StreamKind::VolatilityIndex, at, "deribit_volatility_index.*", parse_volatility_index);

    // Mark prices for all options on an index
    b = b.register(StreamKind::MarkPrice, at, "markprice.options.*.*", parse_markprice_options);

    // Private streams
    b = b
        .register(StreamKind::OrderUpdate,    at, "user.orders.*",    parse_order_update)
        .register(StreamKind::BalanceUpdate,  at, "user.portfolio.*", parse_portfolio)
        .register(StreamKind::PositionUpdate, at, "user.changes.*",   parse_position_update);

    // Block trades
    b = b.register(StreamKind::BlockTrade, at, "block_trade_confirmations", parse_block_trade);

    b.build()
}

/// Deribit wire-level kline resolution strings.
const DERIBIT_KLINE_RESOLUTIONS: &[&str] = &[
    "1", "3", "5", "10", "15", "30", "60", "120", "180", "360", "720", "1D",
];

/// Map Deribit wire resolution → internal KlineInterval string.
fn internal_kline_interval(res: &str) -> &'static str {
    match res {
        "1"   => "1m",
        "3"   => "3m",
        "5"   => "5m",
        "10"  => "10m",
        "15"  => "15m",
        "30"  => "30m",
        "60"  => "1h",
        "120" => "2h",
        "180" => "3h",
        "360" => "6h",
        "720" => "12h",
        "1D"  => "1d",
        _     => "1h",
    }
}

/// Map internal KlineInterval string → Deribit wire resolution.
pub fn deribit_kline_resolution(interval: &KlineInterval) -> &'static str {
    match interval.as_str() {
        "1m"  => "1",
        "3m"  => "3",
        "5m"  => "5",
        "10m" => "10",
        "15m" => "15",
        "30m" => "30",
        "1h"  => "60",
        "2h"  => "120",
        "3h"  => "180",
        "6h"  => "360",
        "12h" => "720",
        "1d"  => "1D",
        _     => "60",
    }
}

/// Format Deribit instrument name from Symbol base+quote.
///
/// If `quote` is empty or "USD"/"PERP" — perpetual convention.
/// If `quote` is "USDC" → linear perpetual like `SOL_USDC-PERPETUAL`.
/// If base already contains '-' it is returned verbatim (e.g. option names).
pub fn deribit_instrument(base: &str, quote: &str) -> String {
    let base_up = base.to_uppercase();
    // Already a fully-formed Deribit instrument name (options, dated futures)
    if base_up.contains('-') {
        return base_up;
    }
    match quote.to_uppercase().as_str() {
        "" | "USD" | "PERP" => format!("{}-PERPETUAL", base_up),
        "USDC" => format!("{}_USDC-PERPETUAL", base_up),
        "USDT" => format!("{}_USDT-PERPETUAL", base_up),
        other => format!("{}-{}", base_up, other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsers (ParserFn = fn(&Value) -> WebSocketResult<StreamEvent>)
//
// Each parser receives the full JSON-RPC subscription frame:
//   {"jsonrpc":"2.0","method":"subscription","params":{"channel":"...","data":{...}}}
// ─────────────────────────────────────────────────────────────────────────────

fn frame_data(raw: &Value) -> WebSocketResult<(&Value, &str)> {
    let params = raw
        .get("params")
        .ok_or_else(|| WebSocketError::Parse("deribit frame missing 'params'".into()))?;
    let channel = params
        .get("channel")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("deribit frame missing 'params.channel'".into()))?;
    let data = params
        .get("data")
        .ok_or_else(|| WebSocketError::Parse("deribit frame missing 'params.data'".into()))?;
    Ok((data, channel))
}

fn get_f64(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(|x| x.as_f64())
}

fn get_i64(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(|x| x.as_i64())
}

fn get_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(|x| x.as_str())
}

// ── Orderbook ────────────────────────────────────────────────────────────────

fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, _channel) = frame_data(raw)?;
    DeribitParser::parse_ws_orderbook(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))
}

// ── Trade ────────────────────────────────────────────────────────────────────

fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, _channel) = frame_data(raw)?;
    let trade = DeribitParser::parse_ws_trade(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Trade(trade))
}

// ── Ticker ───────────────────────────────────────────────────────────────────

fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, _channel) = frame_data(raw)?;
    let ticker = DeribitParser::parse_ws_ticker(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Ticker(ticker))
}

// ── Quote (best bid/ask) ─────────────────────────────────────────────────────

fn parse_quote(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, channel) = frame_data(raw)?;
    let instrument = get_str(data, "instrument_name")
        .unwrap_or_else(|| channel.strip_prefix("quote.").unwrap_or(channel));
    let timestamp = get_i64(data, "timestamp").unwrap_or(0);
    let bid_price = get_f64(data, "best_bid_price");
    let ask_price = get_f64(data, "best_ask_price");
    let ticker = Ticker {
        symbol: instrument.to_string(),
        bid_price,
        ask_price,
        last_price: bid_price.unwrap_or(0.0),
        volume_24h: None,
        high_24h: None,
        low_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        quote_volume_24h: None,
        timestamp,
    };
    Ok(StreamEvent::Ticker(ticker))
}

// ── Kline ────────────────────────────────────────────────────────────────────

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    // chart.trades frame data: {"tick":1234,"open":...,"high":...,"low":...,"close":...,"volume":...,"cost":...}
    let (data, _channel) = frame_data(raw)?;
    use crate::core::types::Kline;
    let open_time = get_i64(data, "tick").unwrap_or(0);
    let kline = Kline {
        open_time,
        open: get_f64(data, "open").unwrap_or(0.0),
        high: get_f64(data, "high").unwrap_or(0.0),
        low: get_f64(data, "low").unwrap_or(0.0),
        close: get_f64(data, "close").unwrap_or(0.0),
        volume: get_f64(data, "volume").unwrap_or(0.0),
        quote_volume: get_f64(data, "cost"),
        close_time: None,
        trades: None,
    };
    Ok(StreamEvent::Kline(kline))
}

// ── Mark price ───────────────────────────────────────────────────────────────

fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, _channel) = frame_data(raw)?;
    let symbol = get_str(data, "instrument_name").unwrap_or("").to_string();
    let mark_price = get_f64(data, "mark_price")
        .ok_or_else(|| WebSocketError::Parse("mark_price missing".into()))?;
    let index_price = get_f64(data, "index_price");
    let timestamp = get_i64(data, "timestamp").unwrap_or(0);
    Ok(StreamEvent::MarkPrice { symbol, mark_price, index_price, timestamp })
}

// ── Perpetual (interest rate → FundingRate) ──────────────────────────────────

fn parse_perpetual(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, channel) = frame_data(raw)?;
    let timestamp = get_i64(data, "timestamp").unwrap_or(0);
    let instrument = get_str(data, "instrument_name")
        .unwrap_or_else(|| channel.split('.').nth(1).unwrap_or(""));
    let rate = get_f64(data, "interest_rate")
        .ok_or_else(|| WebSocketError::Parse("perpetual: missing interest_rate".into()))?;
    Ok(StreamEvent::FundingRate {
        symbol: instrument.to_string(),
        rate,
        next_funding_time: None,
        timestamp,
    })
}

// ── Index price ──────────────────────────────────────────────────────────────

fn parse_index_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, channel) = frame_data(raw)?;
    let price = get_f64(data, "price")
        .ok_or_else(|| WebSocketError::Parse("deribit_price_index: missing price".into()))?;
    let timestamp = get_i64(data, "timestamp").unwrap_or(0);
    let index_name = get_str(data, "index_name")
        .unwrap_or_else(|| channel.strip_prefix("deribit_price_index.").unwrap_or(channel));
    Ok(StreamEvent::IndexPrice {
        symbol: index_name.to_string(),
        price,
        timestamp,
    })
}

// ── Estimated expiration price ───────────────────────────────────────────────

fn parse_estimated_expiration(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, channel) = frame_data(raw)?;
    let price = get_f64(data, "price")
        .ok_or_else(|| WebSocketError::Parse("estimated_expiration_price: missing price".into()))?;
    let timestamp = get_i64(data, "timestamp").unwrap_or(0);
    let index_name = get_str(data, "index_name")
        .unwrap_or_else(|| channel.strip_prefix("estimated_expiration_price.").unwrap_or(channel));
    Ok(StreamEvent::IndexPrice {
        symbol: index_name.to_string(),
        price,
        timestamp,
    })
}

// ── Volatility index ─────────────────────────────────────────────────────────

fn parse_volatility_index(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, channel) = frame_data(raw)?;
    let index_name = get_str(data, "index_name")
        .unwrap_or_else(|| channel.strip_prefix("deribit_volatility_index.").unwrap_or(channel));
    let timestamp = get_i64(data, "timestamp").unwrap_or(0);
    let value = get_f64(data, "volatility")
        .ok_or_else(|| WebSocketError::Parse("deribit_volatility_index: missing volatility".into()))?;
    Ok(StreamEvent::VolatilityIndex {
        symbol: index_name.to_string(),
        value,
        timestamp,
    })
}

// ── markprice.options (array of option mark prices) ──────────────────────────

fn parse_markprice_options(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, _channel) = frame_data(raw)?;
    // data is an array; emit MarkPrice for first item (framework dispatches one event per call)
    let item = if let Some(arr) = data.as_array() {
        arr.first().ok_or_else(|| WebSocketError::Parse("markprice.options: empty array".into()))?
    } else {
        data
    };
    let symbol = get_str(item, "instrument_name")
        .ok_or_else(|| WebSocketError::Parse("markprice.options: missing instrument_name".into()))?
        .to_string();
    let mark_price = get_f64(item, "mark_price")
        .ok_or_else(|| WebSocketError::Parse("markprice.options: missing mark_price".into()))?;
    let timestamp = get_i64(item, "timestamp").unwrap_or(0);
    Ok(StreamEvent::MarkPrice {
        symbol,
        mark_price,
        index_price: None,
        timestamp,
    })
}

// ── Private: order update ────────────────────────────────────────────────────

fn parse_order_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, _channel) = frame_data(raw)?;
    let event = DeribitParser::parse_ws_order_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderUpdate(event))
}

// ── Private: portfolio / balance ─────────────────────────────────────────────

fn parse_portfolio(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, channel) = frame_data(raw)?;
    let currency = channel.strip_prefix("user.portfolio.").unwrap_or("");
    let get = |key: &str| -> f64 { get_f64(data, key).unwrap_or(0.0) };
    let total = get("equity");
    let available = get("available_funds");
    let event = BalanceUpdateEvent {
        asset: currency.to_string(),
        free: available,
        locked: (total - available).max(0.0),
        total,
        delta: None,
        reason: Some(BalanceChangeReason::Other),
        timestamp: Utc::now().timestamp_millis(),
    };
    Ok(StreamEvent::BalanceUpdate(event))
}

// ── Private: position/changes ────────────────────────────────────────────────

fn parse_position_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    // user.changes frame: {"data":{"positions":[...],"orders":[...],"trades":[...],"instrument_name":"..."}}
    let (data, _channel) = frame_data(raw)?;

    // Extract first position from the changes data
    let positions = data
        .get("positions")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first());

    let pos_data = positions.unwrap_or(data);

    let symbol = get_str(pos_data, "instrument_name").unwrap_or("").to_string();
    let size = get_f64(pos_data, "size").unwrap_or(0.0);
    let direction = get_str(pos_data, "direction").unwrap_or("buy");
    let side = match direction {
        "sell" => PositionSide::Short,
        "buy" => PositionSide::Long,
        _ => PositionSide::Both,
    };

    use crate::core::types::{MarginType, PositionUpdateEvent};
    let event = PositionUpdateEvent {
        symbol,
        side,
        quantity: size.abs(),
        entry_price: get_f64(pos_data, "average_price").unwrap_or(0.0),
        mark_price: get_f64(pos_data, "mark_price"),
        unrealized_pnl: get_f64(pos_data, "floating_profit_loss").unwrap_or(0.0),
        realized_pnl: get_f64(pos_data, "realized_profit_loss"),
        leverage: get_f64(pos_data, "leverage").map(|l| l as u32),
        liquidation_price: get_f64(pos_data, "estimated_liquidation_price"),
        margin_type: Some(MarginType::Cross),
        reason: None,
        timestamp: get_i64(pos_data, "last_update_timestamp").unwrap_or(0),
    };
    Ok(StreamEvent::PositionUpdate(event))
}

// ── Block trade ──────────────────────────────────────────────────────────────

fn parse_block_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (data, _channel) = frame_data(raw)?;
    let symbol = get_str(data, "instrument_name").unwrap_or("").to_string();
    let block_id = data
        .get("block_trade_id")
        .or_else(|| data.get("trade_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let price = get_f64(data, "price").unwrap_or(0.0);
    let quantity = get_f64(data, "amount").unwrap_or(0.0);
    let timestamp = get_i64(data, "timestamp").unwrap_or(0);
    let is_iv = data.get("iv").and_then(|v| v.as_f64()).is_some();
    let side = match get_str(data, "direction") {
        Some("buy") => TradeSide::Buy,
        _ => TradeSide::Sell,
    };
    Ok(StreamEvent::BlockTrade {
        symbol,
        block_id,
        price,
        quantity,
        side,
        timestamp,
        is_iv,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AccountType, Symbol};
    use crate::core::websocket::StreamSpec;

    fn futures_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: Symbol::new("BTC", "USD"),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = DeribitProtocol::new(AccountType::FuturesCross, false);
        let reg = proto.topic_registry(AccountType::FuturesCross);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "registry must have entries");
        assert!(reg.supports(&StreamKind::Trade, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Ticker, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Orderbook, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::FundingRate, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::MarkPrice, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::IndexPrice, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::VolatilityIndex, AccountType::FuturesCross));
    }

    #[test]
    fn test_subscribe_frame_book_jsonrpc() {
        let proto = DeribitProtocol::new(AccountType::FuturesCross, false);
        let spec = futures_spec(StreamKind::Orderbook);
        let msg = proto.subscribe_frame(&spec).expect("must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "public/subscribe");
        let channels = v["params"]["channels"].as_array().expect("channels array");
        assert!(!channels.is_empty());
        let ch = channels[0].as_str().expect("channel string");
        assert!(ch.starts_with("book.BTC-PERPETUAL."), "channel={}", ch);
    }

    #[test]
    fn test_extract_topic_subscription_frame() {
        let proto = DeribitProtocol::new(AccountType::FuturesCross, false);
        let frame = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "subscription",
            "params": {
                "channel": "book.BTC-PERPETUAL.100ms",
                "data": {}
            }
        });
        let topic = proto.extract_topic(&frame).expect("must extract topic");
        assert_eq!(topic.as_str(), "book.BTC-PERPETUAL.100ms");
    }

    #[test]
    fn test_extract_topic_subscribe_response_returns_none() {
        let proto = DeribitProtocol::new(AccountType::FuturesCross, false);
        // Subscribe ack: result is array of channel strings
        let frame = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": ["book.BTC-PERPETUAL.100ms"]
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_extract_topic_ping_response_returns_none() {
        let proto = DeribitProtocol::new(AccountType::FuturesCross, false);
        // public/test response
        let frame = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "result": { "version": "1.2.26" }
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_jsonrpc_id_counter_monotonic() {
        let proto = DeribitProtocol::new(AccountType::FuturesCross, false);
        let id1 = proto.next_id();
        let id2 = proto.next_id();
        let id3 = proto.next_id();
        assert!(id1 < id2);
        assert!(id2 < id3);
    }

    #[test]
    fn test_deribit_instrument_perpetual() {
        assert_eq!(deribit_instrument("BTC", "USD"), "BTC-PERPETUAL");
        assert_eq!(deribit_instrument("ETH", ""), "ETH-PERPETUAL");
    }

    #[test]
    fn test_deribit_instrument_usdc_linear() {
        assert_eq!(deribit_instrument("SOL", "USDC"), "SOL_USDC-PERPETUAL");
    }

    #[test]
    fn test_deribit_instrument_option_passthrough() {
        // Option names already contain '-' — must be returned verbatim
        assert_eq!(
            deribit_instrument("BTC-30MAY26-50000-C", ""),
            "BTC-30MAY26-50000-C"
        );
    }
}
