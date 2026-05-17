//! OkxProtocol — WsProtocol implementation for the OKX exchange.
//!
//! Single registry covering all public channels (public + business endpoints
//! share the same channel namespace).  Auth frame builds the OKX WS login
//! message.  Ping is literal text `"ping"` every 30s.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::core::traits::Credentials;
use crate::core::types::{AccountType, StreamEvent, TradeSide, WebSocketError, WebSocketResult};
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol,
};
use crate::core::{encode_base64, hmac_sha256, timestamp_iso8601};
use crate::core::types::OrderbookDelta as OrderbookDeltaType;
use crate::core::types::OrderBook;

use super::parser::OkxParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache (one registry covers all OKX account types)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

fn registry() -> &'static TopicRegistry {
    REGISTRY.get_or_init(build_registry)
}

// ─────────────────────────────────────────────────────────────────────────────
// OkxProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative OKX WS protocol shim.
pub struct OkxProtocol {
    _account_type: AccountType,
    testnet: bool,
    /// Connect to business endpoint (mark-price-candle*, index-candle*).
    pub is_business: bool,
}

impl OkxProtocol {
    pub fn new(account_type: AccountType, testnet: bool) -> Self {
        Self {
            _account_type: account_type,
            testnet,
            is_business: false,
        }
    }

    pub fn new_business(account_type: AccountType, testnet: bool) -> Self {
        Self {
            _account_type: account_type,
            testnet,
            is_business: true,
        }
    }

    /// Map internal interval → OKX wire suffix (e.g. "1h" → "1H").
    fn okx_interval(interval: &KlineInterval) -> &str {
        match interval.as_str() {
            "1m"  => "1m",
            "3m"  => "3m",
            "5m"  => "5m",
            "15m" => "15m",
            "30m" => "30m",
            "1h"  => "1H",
            "2h"  => "2H",
            "4h"  => "4H",
            "6h"  => "6H",
            "12h" => "12H",
            "1d"  => "1D",
            "2d"  => "2D",
            "3d"  => "3D",
            "1w"  => "1W",
            "1M"  => "1M",
            "3M"  => "3M",
            other => other,
        }
    }

    /// Build subscribe/unsubscribe frame for standard channels (channel + instId).
    fn build_instid_frame(op: &str, channel: &str, spec: &StreamSpec) -> Message {
        let inst_id = spec.symbol.as_str();
        let frame = json!({
            "op": op,
            "args": [{ "channel": channel, "instId": inst_id }]
        });
        Message::Text(frame.to_string())
    }

    /// Extract the OKX instFamily from a raw instrument ID.
    ///
    /// `"BTC-USDT"` → `"BTC-USDT"` (unchanged)
    /// `"BTC-USDT-SWAP"` → `"BTC-USDT"`
    /// `"BTC-USD-260925"` → `"BTC-USD"`
    fn okx_inst_family(raw: &str) -> String {
        let parts: Vec<&str> = raw.split('-').collect();
        if parts.len() >= 2 {
            format!("{}-{}", parts[0].to_uppercase(), parts[1].to_uppercase())
        } else {
            raw.to_uppercase()
        }
    }

    /// Map StreamKind → OKX channel name string for standard instId-based channels.
    /// Returns None for kinds that need custom frame construction.
    fn channel_name(kind: &StreamKind) -> Option<String> {
        let name = match kind {
            StreamKind::Ticker => "tickers".to_string(),
            StreamKind::Trade => "trades".to_string(),
            StreamKind::Orderbook => "books".to_string(),
            StreamKind::OrderbookDelta => "books-l2-tbt".to_string(),
            StreamKind::MarkPrice => "mark-price".to_string(),
            StreamKind::FundingRate => "funding-rate".to_string(),
            StreamKind::IndexPrice => "index-tickers".to_string(),
            StreamKind::OrderUpdate => "orders".to_string(),
            StreamKind::BalanceUpdate => "account".to_string(),
            StreamKind::PositionUpdate => "positions".to_string(),
            StreamKind::OpenInterest => "open-interest".to_string(),
            StreamKind::Kline { interval } => {
                format!("candle{}", Self::okx_interval(interval))
            }
            StreamKind::MarkPriceKline { interval } => {
                format!("mark-price-candle{}", Self::okx_interval(interval))
            }
            StreamKind::IndexPriceKline { interval } => {
                format!("index-candle{}", Self::okx_interval(interval))
            }
            _ => return None,
        };
        Some(name)
    }
}

impl WsProtocol for OkxProtocol {
    fn name(&self) -> &'static str {
        "okx"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Use the testnet/business flags stored at construction time.
        let s = if self.testnet {
            if self.is_business {
                "wss://wspap.okx.com:8443/ws/v5/business"
            } else {
                "wss://wspap.okx.com:8443/ws/v5/public"
            }
        } else if self.is_business {
            "wss://ws.okx.com:8443/ws/v5/business"
        } else {
            "wss://ws.okx.com:8443/ws/v5/public"
        };
        Url::parse(s).expect("okx ws url is valid")
    }

    fn ping_frame(&self) -> Option<Message> {
        Some(Message::Text("ping".into()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        // Special cases first.
        match &spec.kind {
            StreamKind::Liquidation => {
                let frame = json!({
                    "op": "subscribe",
                    "args": [{ "channel": "liquidation-orders", "instType": "SWAP" }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            StreamKind::BlockTrade => {
                let frame = json!({
                    "op": "subscribe",
                    "args": [{ "channel": "public-block-trades", "instId": spec.symbol.as_str() }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            StreamKind::SettlementEvent => {
                // spec.symbol is raw e.g. "BTC-USDT" or "BTC-USDT-SWAP";
                // instFamily for OKX estimated-price is base-quote: "BTC-USDT".
                let inst_family = Self::okx_inst_family(spec.symbol.as_str());
                let frame = json!({
                    "op": "subscribe",
                    "args": [{ "channel": "estimated-price", "instType": "FUTURES", "instFamily": inst_family }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            StreamKind::OptionGreeks => {
                let uly = Self::okx_inst_family(spec.symbol.as_str());
                let frame = json!({
                    "op": "subscribe",
                    "args": [{ "channel": "opt-summary", "uly": uly }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            _ => {}
        }

        let channel = Self::channel_name(&spec.kind).ok_or_else(|| {
            WebSocketError::UnsupportedOperation(format!(
                "okx: unsupported stream kind {:?}",
                spec.kind
            ))
        })?;
        Ok(Self::build_instid_frame("subscribe", &channel, spec))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        match &spec.kind {
            StreamKind::Liquidation => {
                let frame = json!({
                    "op": "unsubscribe",
                    "args": [{ "channel": "liquidation-orders", "instType": "SWAP" }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            StreamKind::BlockTrade => {
                let frame = json!({
                    "op": "unsubscribe",
                    "args": [{ "channel": "public-block-trades", "instId": spec.symbol.as_str() }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            StreamKind::SettlementEvent => {
                let inst_family = Self::okx_inst_family(spec.symbol.as_str());
                let frame = json!({
                    "op": "unsubscribe",
                    "args": [{ "channel": "estimated-price", "instType": "FUTURES", "instFamily": inst_family }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            StreamKind::OptionGreeks => {
                let uly = Self::okx_inst_family(spec.symbol.as_str());
                let frame = json!({
                    "op": "unsubscribe",
                    "args": [{ "channel": "opt-summary", "uly": uly }]
                });
                return Ok(Message::Text(frame.to_string()));
            }
            _ => {}
        }

        let channel = Self::channel_name(&spec.kind).ok_or_else(|| {
            WebSocketError::UnsupportedOperation(format!(
                "okx: unsupported stream kind {:?}",
                spec.kind
            ))
        })?;
        Ok(Self::build_instid_frame("unsubscribe", &channel, spec))
    }

    fn auth_frame(&self, credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        let passphrase = credentials.passphrase.as_deref()?;
        let timestamp = timestamp_iso8601();
        let prehash = format!("{}GET/users/self/verify", timestamp);
        let signature = encode_base64(&hmac_sha256(
            credentials.api_secret.as_bytes(),
            prehash.as_bytes(),
        ));
        let login = json!({
            "op": "login",
            "args": [{
                "apiKey": credentials.api_key,
                "passphrase": passphrase,
                "timestamp": timestamp,
                "sign": signature,
            }]
        });
        Some(Ok(Message::Text(login.to_string())))
    }

    fn is_auth_ack(&self, raw: &Value) -> bool {
        matches!(
            raw.get("event").and_then(|v| v.as_str()),
            Some("login")
        )
    }

    fn is_pong(&self, raw: &Value) -> bool {
        raw.as_str() == Some("pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        matches!(
            raw.get("event").and_then(|v| v.as_str()),
            Some("subscribe") | Some("unsubscribe")
        )
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Literal "pong" text response.
        if raw.as_str() == Some("pong") {
            return None;
        }

        // Event frames: subscribe ack, unsubscribe ack, login ack, error.
        if raw.get("event").is_some() {
            return None;
        }

        // Data frame: {"arg":{"channel":"trades","instId":"BTC-USDT"},"data":[...]}
        let channel = raw
            .get("arg")
            .and_then(|a| a.get("channel"))
            .and_then(|c| c.as_str())?;

        Some(TopicKey::new(channel))
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        registry()
    }

    fn unsupported_by_exchange(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[]
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[
            StreamKind::OrderUpdate,
            StreamKind::BalanceUpdate,
            StreamKind::PositionUpdate,
        ]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    let mut b = TopicRegistry::builder();
    let at = AccountType::Spot; // OKX uses single registry; account_type in key is Spot.

    // Standard channels.
    b = b
        .register(StreamKind::Ticker, at, "tickers", parse_tickers)
        .register(StreamKind::Trade, at, "trades", parse_trades)
        .register(StreamKind::Trade, at, "trades-all", parse_trades)
        .register(StreamKind::Orderbook, at, "books", parse_books)
        .register(StreamKind::Orderbook, at, "books5", parse_books)
        .register(StreamKind::Orderbook, at, "bbo-tbt", parse_books)
        .register(StreamKind::OrderbookDelta, at, "books-l2-tbt", parse_books)
        .register(StreamKind::OrderbookDelta, at, "books50-l2-tbt", parse_books)
        .register(StreamKind::MarkPrice, at, "mark-price", parse_mark_price)
        .register(StreamKind::FundingRate, at, "funding-rate", parse_funding_rate)
        .register(StreamKind::Liquidation, at, "liquidation-orders", parse_liquidation_orders)
        .register(StreamKind::IndexPrice, at, "index-tickers", parse_index_tickers)
        .register(StreamKind::OpenInterest, at, "open-interest", parse_open_interest)
        .register(StreamKind::BlockTrade, at, "public-block-trades", parse_block_trades)
        .register(StreamKind::BlockTrade, at, "block-trades", parse_block_trades)
        .register(StreamKind::SettlementEvent, at, "estimated-price", parse_estimated_price)
        .register(StreamKind::OptionGreeks, at, "opt-summary", parse_opt_summary)
        // price-limit: no matching StreamEvent — register so it doesn't warn as unmatched.
        .register(StreamKind::Orderbook, at, "price-limit", parse_price_limit)
        // instruments / status: informational channels.
        .register(StreamKind::Ticker, at, "instruments", parse_instruments)
        .register(StreamKind::Ticker, at, "status", parse_status_channel)
        // Private channels.
        .register(StreamKind::OrderUpdate, at, "orders", parse_orders)
        .register(StreamKind::BalanceUpdate, at, "account", parse_account)
        .register(StreamKind::PositionUpdate, at, "positions", parse_positions);

    // Kline channels.
    for (wire, internal) in OKX_KLINE_CHANNELS {
        let kind = StreamKind::Kline {
            interval: KlineInterval::new(*internal),
        };
        b = b.register(kind, at, *wire, parse_kline);
    }

    // Mark-price kline channels (business endpoint, same topic key).
    for (wire, internal) in OKX_MARK_PRICE_KLINE_CHANNELS {
        let kind = StreamKind::MarkPriceKline {
            interval: KlineInterval::new(*internal),
        };
        b = b.register(kind, at, *wire, parse_mark_price_kline);
    }

    // Index kline channels.
    for (wire, internal) in OKX_INDEX_KLINE_CHANNELS {
        let kind = StreamKind::IndexPriceKline {
            interval: KlineInterval::new(*internal),
        };
        b = b.register(kind, at, *wire, parse_index_kline);
    }

    b.build()
}

/// (wire_channel_name, internal_interval) pairs for regular klines.
const OKX_KLINE_CHANNELS: &[(&str, &str)] = &[
    ("candle1m",  "1m"),
    ("candle3m",  "3m"),
    ("candle5m",  "5m"),
    ("candle15m", "15m"),
    ("candle30m", "30m"),
    ("candle1H",  "1h"),
    ("candle2H",  "2h"),
    ("candle4H",  "4h"),
    ("candle6H",  "6h"),
    ("candle12H", "12h"),
    ("candle1D",  "1d"),
    ("candle2D",  "2d"),
    ("candle3D",  "3d"),
    ("candle1W",  "1w"),
    ("candle1M",  "1M"),
    ("candle3M",  "3M"),
];

/// (wire_channel_name, internal_interval) pairs for mark-price klines.
const OKX_MARK_PRICE_KLINE_CHANNELS: &[(&str, &str)] = &[
    ("mark-price-candle1m",  "1m"),
    ("mark-price-candle3m",  "3m"),
    ("mark-price-candle5m",  "5m"),
    ("mark-price-candle15m", "15m"),
    ("mark-price-candle30m", "30m"),
    ("mark-price-candle1H",  "1h"),
    ("mark-price-candle2H",  "2h"),
    ("mark-price-candle4H",  "4h"),
    ("mark-price-candle6H",  "6h"),
    ("mark-price-candle12H", "12h"),
    ("mark-price-candle1D",  "1d"),
    ("mark-price-candle2D",  "2d"),
    ("mark-price-candle3D",  "3d"),
    ("mark-price-candle1W",  "1w"),
    ("mark-price-candle1M",  "1M"),
    ("mark-price-candle3M",  "3M"),
];

/// (wire_channel_name, internal_interval) pairs for index klines.
const OKX_INDEX_KLINE_CHANNELS: &[(&str, &str)] = &[
    ("index-candle1m",  "1m"),
    ("index-candle3m",  "3m"),
    ("index-candle5m",  "5m"),
    ("index-candle15m", "15m"),
    ("index-candle30m", "30m"),
    ("index-candle1H",  "1h"),
    ("index-candle2H",  "2h"),
    ("index-candle4H",  "4h"),
    ("index-candle6H",  "6h"),
    ("index-candle12H", "12h"),
    ("index-candle1D",  "1d"),
    ("index-candle2D",  "2d"),
    ("index-candle3D",  "3d"),
    ("index-candle1W",  "1w"),
    ("index-candle1M",  "1M"),
    ("index-candle3M",  "3M"),
];

// ─────────────────────────────────────────────────────────────────────────────
// Parser helpers
// ─────────────────────────────────────────────────────────────────────────────

fn parse_f64_field(v: &Value) -> Option<f64> {
    v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
}

/// Extract first data element from `{"arg":{...},"data":[item,...]}` frame.
fn first_data_item(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("data")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| WebSocketError::Parse("okx frame: missing or empty 'data' array".into()))
}

/// Extract all data elements from frame.
fn data_array(raw: &Value) -> WebSocketResult<&Vec<Value>> {
    raw.get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| WebSocketError::Parse("okx frame: 'data' is not an array".into()))
}

fn arg_inst_id(raw: &Value) -> &str {
    raw.get("arg")
        .and_then(|a| a.get("instId"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
}

fn arg_channel(raw: &Value) -> &str {
    raw.get("arg")
        .and_then(|a| a.get("channel"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsers
// ─────────────────────────────────────────────────────────────────────────────

fn parse_tickers(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let ticker = OkxParser::parse_ws_ticker(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Ticker(ticker))
}

fn parse_trades(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let trade = OkxParser::parse_ws_trade(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Trade(trade))
}

fn parse_books(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let action = raw.get("action").and_then(|a| a.as_str());
    let (asks, bids) = OkxParser::parse_ws_orderbook(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let timestamp = OkxParser::get_i64(data, "ts").unwrap_or(0);
    let seq_id = data.get("seqId").and_then(|v| v.as_u64());
    let prev_seq_id = data.get("prevSeqId").and_then(|v| v.as_u64());
    let checksum = data.get("checksum").and_then(|v| v.as_i64());

    if action == Some("snapshot") {
        let ob = OrderBook {
            asks,
            bids,
            timestamp,
            sequence: None,
            last_update_id: seq_id,
            first_update_id: seq_id,
            prev_update_id: prev_seq_id,
            event_time: Some(timestamp),
            transaction_time: None,
            checksum,
        };
        Ok(StreamEvent::OrderbookSnapshot(ob))
    } else {
        let delta = OrderbookDeltaType {
            asks,
            bids,
            timestamp,
            first_update_id: seq_id,
            last_update_id: seq_id,
            prev_update_id: prev_seq_id,
            event_time: Some(timestamp),
            checksum,
        };
        Ok(StreamEvent::OrderbookDelta(delta))
    }
}

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let kline = OkxParser::parse_ws_kline(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::Kline(kline))
}

fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mark_price = data.get("markPx")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("mark-price: missing markPx".into()))?;
    let timestamp = data.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    Ok(StreamEvent::MarkPrice { symbol, mark_price, index_price: None, timestamp })
}

fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let rate = data.get("fundingRate")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("funding-rate: missing fundingRate".into()))?;
    let next_funding_time = data.get("nextFundingTime")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64);
    let timestamp = data.get("fundingTime")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    Ok(StreamEvent::FundingRate { symbol, rate, next_funding_time, timestamp })
}

fn parse_liquidation_orders(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let details = data.get("details")
        .and_then(|d| d.as_array())
        .ok_or_else(|| WebSocketError::Parse("liquidation-orders: missing details".into()))?;
    let detail = details.first()
        .ok_or_else(|| WebSocketError::Parse("liquidation-orders: empty details".into()))?;

    let side_str = detail.get("side").and_then(|s| s.as_str()).unwrap_or("buy");
    let side = match side_str {
        "buy" => TradeSide::Buy,
        _ => TradeSide::Sell,
    };
    let price = detail.get("fillPx")
        .or_else(|| detail.get("bkPx"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("liquidation-orders: missing price".into()))?;
    let quantity = detail.get("sz")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("liquidation-orders: missing sz".into()))?;
    let timestamp = detail.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    Ok(StreamEvent::Liquidation {
        symbol,
        side,
        price,
        quantity,
        value: Some(price * quantity),
        timestamp,
    })
}

fn parse_index_tickers(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let price = data.get("idxPx")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("index-tickers: missing idxPx".into()))?;
    let timestamp = data.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    Ok(StreamEvent::IndexPrice { symbol, price, timestamp })
}

fn parse_open_interest(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let open_interest = data.get("oi")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("open-interest: missing oi".into()))?;
    let open_interest_value = data.get("oiCcy").and_then(parse_f64_field);
    let timestamp = data.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    Ok(StreamEvent::OpenInterestUpdate { symbol, open_interest, open_interest_value, timestamp })
}

fn parse_block_trades(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let price = data.get("px")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("block-trades: missing px".into()))?;
    let quantity = data.get("sz")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("block-trades: missing sz".into()))?;
    let side = match data.get("side").and_then(|v| v.as_str()).unwrap_or("buy") {
        "sell" => TradeSide::Sell,
        _ => TradeSide::Buy,
    };
    let timestamp = data.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    let block_id = data.get("tradeId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let is_iv = data.get("fillVol")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    Ok(StreamEvent::BlockTrade { symbol, block_id, price, quantity, side, timestamp, is_iv })
}

fn parse_estimated_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let settlement_price = data.get("settlePx")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("estimated-price: missing settlePx".into()))?;
    let timestamp = data.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    Ok(StreamEvent::SettlementEvent {
        symbol,
        settlement_price,
        settlement_time: timestamp,
        timestamp,
    })
}

fn parse_opt_summary(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let timestamp = data.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    let get_greek = |name: &str, alt: &str| -> Option<f64> {
        data.get(name)
            .and_then(parse_f64_field)
            .or_else(|| data.get(alt).and_then(parse_f64_field))
    };
    Ok(StreamEvent::OptionGreeks {
        symbol,
        delta: get_greek("delta", "deltaBS"),
        gamma: get_greek("gamma", "gammaBS"),
        vega: get_greek("vega", "vegaBS"),
        theta: get_greek("theta", "thetaBS"),
        rho: None,
        mark_iv: data.get("markVol").and_then(parse_f64_field),
        bid_iv: data.get("bidVol").and_then(parse_f64_field),
        ask_iv: data.get("askVol").and_then(parse_f64_field),
        timestamp,
    })
}

fn parse_mark_price_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let kline = OkxParser::parse_ws_price_candle(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let channel = arg_channel(raw);
    let interval = channel.trim_start_matches("mark-price-candle").to_string();
    let symbol = arg_inst_id(raw).to_string();
    Ok(StreamEvent::MarkPriceKline { symbol, interval, kline })
}

fn parse_index_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let kline = OkxParser::parse_ws_price_candle(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let channel = arg_channel(raw);
    let interval = channel.trim_start_matches("index-candle").to_string();
    let symbol = arg_inst_id(raw).to_string();
    Ok(StreamEvent::IndexPriceKline { symbol, interval, kline })
}

/// price-limit channel has no matching StreamEvent — emit a synthetic MarkPrice
/// so the framework doesn't warn about unmatched topic.
fn parse_price_limit(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let symbol = data.get("instId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    // Use buyLmt as a proxy for mark price; no better variant available.
    let mark_price = data.get("buyLmt")
        .and_then(parse_f64_field)
        .unwrap_or(0.0);
    let timestamp = data.get("ts")
        .and_then(parse_f64_field)
        .map(|ms| ms as i64)
        .unwrap_or(0);
    Ok(StreamEvent::MarkPrice { symbol, mark_price, index_price: None, timestamp })
}

/// instruments channel — informational, no standard event. Return a dummy Ticker.
fn parse_instruments(_raw: &Value) -> WebSocketResult<StreamEvent> {
    Err(WebSocketError::Parse("instruments: no StreamEvent mapping".into()))
}

/// status channel — informational. No standard event.
fn parse_status_channel(_raw: &Value) -> WebSocketResult<StreamEvent> {
    Err(WebSocketError::Parse("status: no StreamEvent mapping".into()))
}

fn parse_orders(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let ev = OkxParser::parse_ws_order_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderUpdate(ev))
}

fn parse_account(raw: &Value) -> WebSocketResult<StreamEvent> {
    // OKX account frame: {"data":[{"details":[...]}]}
    let arr = data_array(raw)?;
    let item = arr.first()
        .ok_or_else(|| WebSocketError::Parse("account: empty data".into()))?;
    if let Some(details) = item.get("details").and_then(|d| d.as_array()) {
        if let Some(detail) = details.first() {
            let ev = OkxParser::parse_ws_balance_update(detail)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            return Ok(StreamEvent::BalanceUpdate(ev));
        }
    }
    Err(WebSocketError::Parse("account: no details found".into()))
}

fn parse_positions(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = first_data_item(raw)?;
    let ev = OkxParser::parse_ws_position_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::PositionUpdate(ev))
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
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTC-USDT".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = OkxProtocol::new(AccountType::Spot, false);
        let reg = proto.topic_registry(AccountType::Spot);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "registry must have entries");
        assert!(reg.supports(&StreamKind::Ticker, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Orderbook, AccountType::Spot));
        assert!(reg.supports(&StreamKind::FundingRate, AccountType::Spot));
        assert!(reg.supports(&StreamKind::MarkPrice, AccountType::Spot));
        assert!(reg.supports(
            &StreamKind::Kline { interval: KlineInterval::new("1h") },
            AccountType::Spot
        ));
    }

    #[test]
    fn test_subscribe_frame_trades() {
        let proto = OkxProtocol::new(AccountType::Spot, false);
        let spec = spot_spec(StreamKind::Trade);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame ok");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["op"], "subscribe");
        let arg = &v["args"][0];
        assert_eq!(arg["channel"], "trades");
        assert_eq!(arg["instId"], "BTC-USDT");
    }

    #[test]
    fn test_extract_topic_trades_frame() {
        let proto = OkxProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "arg": { "channel": "trades", "instId": "BTC-USDT" },
            "data": []
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "trades");
    }

    #[test]
    fn test_extract_topic_ping_returns_none() {
        let proto = OkxProtocol::new(AccountType::Spot, false);
        let frame = serde_json::Value::String("pong".to_string());
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_extract_topic_subscribe_ack_returns_none() {
        let proto = OkxProtocol::new(AccountType::Spot, false);
        let frame = serde_json::json!({
            "event": "subscribe",
            "arg": { "channel": "trades", "instId": "BTC-USDT" }
        });
        assert!(proto.extract_topic(&frame).is_none());
    }
}
