//! KuCoinProtocol — WsProtocol implementation for the KuCoin exchange.
//!
//! Declarative shim: supplies endpoint URLs, ping frame, subscribe/unsubscribe
//! frames, topic extraction, and topic registry to UniversalWsTransport.
//!
//! ## KuCoin-unique: bullet-public pre-connect
//!
//! KuCoin requires a POST to `/api/v1/bullet-public` (spot) or the futures
//! equivalent before opening the WebSocket connection. The response returns
//! the actual WS endpoint URL and a token. Both are baked into the WS URL
//! as query parameters.
//!
//! This is handled in `KuCoinWebSocket::new()`: the resolved URL is passed
//! into `KuCoinProtocol::new(account_type, testnet, resolved_url)`.
//! `endpoint()` simply returns the stored URL; `pre_connect_hook` stays
//! at the default `Ok(None)`.

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
use crate::core::timestamp_millis;

use super::parser::KuCoinParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry caches (one per account class)
// ─────────────────────────────────────────────────────────────────────────────

static SPOT_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static FUTURES_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// KuCoinProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative KuCoin WS protocol shim.
///
/// The resolved WebSocket URL (including token + connectId query params) is
/// stored at construction time. `endpoint()` returns it directly.
pub struct KuCoinProtocol {
    _account_type: AccountType,
    _testnet: bool,
    /// Resolved WS URL returned by bullet-public (includes token + connectId).
    resolved_url: Url,
    /// Ping interval from bullet-public response (milliseconds).
    ping_interval_ms: u64,
}

impl KuCoinProtocol {
    pub fn new(
        account_type: AccountType,
        testnet: bool,
        resolved_url: Url,
        ping_interval_ms: u64,
    ) -> Self {
        Self {
            _account_type: account_type,
            _testnet: testnet,
            resolved_url,
            ping_interval_ms,
        }
    }

    /// Build subscribe/unsubscribe JSON frame.
    fn build_frame(op: &str, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let topic = Self::build_topic(spec)?;
        let is_private = spec.kind.is_private();

        let frame = if is_private {
            json!({
                "id": timestamp_millis().to_string(),
                "type": op,
                "topic": topic,
                "privateChannel": true,
                "response": true
            })
        } else {
            json!({
                "id": timestamp_millis().to_string(),
                "type": op,
                "topic": topic,
                "privateChannel": false,
                "response": true
            })
        };

        Ok(Message::Text(frame.to_string()))
    }

    /// Build the full KuCoin topic string for a StreamSpec.
    fn build_topic(spec: &StreamSpec) -> Result<String, WebSocketError> {
        let at = spec.account_type;
        let is_futures = matches!(at, AccountType::FuturesCross | AccountType::FuturesIsolated);
        let sym = spec.symbol.as_str();

        let topic = match &spec.kind {
            StreamKind::Ticker => {
                if is_futures {
                    format!("/contractMarket/tickerV2:{}", sym)
                } else {
                    format!("/market/ticker:{}", sym)
                }
            }
            StreamKind::Trade => {
                if is_futures {
                    format!("/contractMarket/execution:{}", sym)
                } else {
                    format!("/market/match:{}", sym)
                }
            }
            StreamKind::Orderbook => {
                // Snapshot channel: server sends 5-level snapshot on every update.
                if is_futures {
                    format!("/contractMarket/level2Depth5:{}", sym)
                } else {
                    format!("/spotMarket/level2Depth5:{}", sym)
                }
            }
            StreamKind::OrderbookDelta => {
                // Full delta channel: requires client-side sequence tracking.
                if is_futures {
                    format!("/contractMarket/level2:{}", sym)
                } else {
                    format!("/market/level2:{}", sym)
                }
            }
            StreamKind::Kline { interval } => {
                let wire = kucoin_kline_interval(interval);
                if is_futures {
                    format!("/contractMarket/limitCandle:{}_{}", sym, wire)
                } else {
                    format!("/market/candles:{}_{}", sym, wire)
                }
            }
            StreamKind::MarkPrice => format!("/contract/instrument:{}", sym),
            StreamKind::IndexPrice => format!("/contract/instrument:{}", sym),
            StreamKind::FundingRate => format!("/contract/instrument:{}", sym),
            StreamKind::OrderUpdate => {
                if is_futures {
                    "/contractMarket/tradeOrders".to_string()
                } else {
                    "/spotMarket/tradeOrdersV2".to_string()
                }
            }
            StreamKind::BalanceUpdate => {
                if is_futures {
                    "/contractAccount/wallet".to_string()
                } else {
                    "/account/balance".to_string()
                }
            }
            StreamKind::PositionUpdate => "/contract/positionAll".to_string(),
            StreamKind::Liquidation => {
                return Err(WebSocketError::NotSupported(
                    "KuCoin /contractMarket/liquidationOrders requires authentication — \
                     not available as a public WS feed".to_string(),
                ));
            }
            StreamKind::OpenInterest => {
                return Err(WebSocketError::NotSupported(
                    "KuCoin Futures has no public OI WS channel — \
                     use REST GET /api/v1/contracts/{symbol}".to_string(),
                ));
            }
            StreamKind::AggTrade => {
                return Err(WebSocketError::NotSupported(
                    "KuCoin Futures has no aggregated trade WS channel — \
                     use /contractMarket/execution for raw trades".to_string(),
                ));
            }
            other => {
                return Err(WebSocketError::UnsupportedOperation(format!(
                    "kucoin: unsupported stream kind {:?}",
                    other
                )));
            }
        };

        Ok(topic)
    }

    fn spot_registry() -> &'static TopicRegistry {
        SPOT_REGISTRY.get_or_init(|| build_registry(AccountType::Spot))
    }

    fn futures_registry() -> &'static TopicRegistry {
        FUTURES_REGISTRY.get_or_init(|| build_registry(AccountType::FuturesCross))
    }

    /// Pre-check whether a StreamSpec is supported before queuing to the transport.
    ///
    /// UniversalWsTransport.subscribe() is fire-and-forget via an internal
    /// channel — errors from subscribe_frame() are only logged, never returned
    /// to the caller. This method lets KuCoinWebSocket detect NotSupported early
    /// and return the error synchronously so e2e_smoke can display `--` rather
    /// than `silent_0_events / ERR`.
    pub fn check_subscribe(spec: &StreamSpec) -> WebSocketResult<()> {
        Self::build_topic(spec).map(|_| ())
    }
}

impl WsProtocol for KuCoinProtocol {
    fn name(&self) -> &'static str {
        "kucoin"
    }

    fn endpoint(&self, _account_type: AccountType, _testnet: bool) -> Url {
        // Resolved at construction time via bullet-public
        self.resolved_url.clone()
    }

    fn ping_frame(&self) -> Option<Message> {
        let ping = json!({
            "id": timestamp_millis().to_string(),
            "type": "ping"
        });
        Some(Message::Text(ping.to_string()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_millis(self.ping_interval_ms)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
        // KuCoin uses token-based auth baked into the WS URL; no separate auth frame
        None
    }

    fn is_auth_ack(&self, _raw: &Value) -> bool {
        false
    }

    fn is_pong(&self, raw: &Value) -> bool {
        raw.get("type").and_then(|v| v.as_str()) == Some("pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        raw.get("type").and_then(|v| v.as_str()) == Some("ack")
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let msg_type = raw.get("type").and_then(|v| v.as_str())?;

        match msg_type {
            "pong" | "ack" | "welcome" | "error" => return None,
            "message" | "notice" => {}
            _ => return None,
        }

        let topic = raw.get("topic").and_then(|v| v.as_str())?;

        // Build registry key from incoming topic:
        //
        // - No ':' → use topic as-is (e.g. "/spotMarket/tradeOrdersV2")
        // - Has ':' → replace symbol prefix with '*', preserving interval suffix
        //   if present (kline topics encode both symbol and interval).
        //
        // Examples:
        //   "/market/ticker:BTC-USDT"           → "/market/ticker:*"
        //   "/contractMarket/tickerV2:XBTUSDTM" → "/contractMarket/tickerV2:*"
        //   "/market/candles:BTC-USDT_1min"     → "/market/candles:*_1min"
        //   "/contractMarket/limitCandle:XBTUSDTM_1" → "/contractMarket/limitCandle:*_1"
        //   "/spotMarket/level2Depth5:BTC-USDT" → "/spotMarket/level2Depth5:*"
        let key = if let Some(colon_pos) = topic.find(':') {
            let channel = &topic[..colon_pos];
            let sym_and_suffix = &topic[colon_pos + 1..];

            // Kline topics use '_' to separate symbol from interval (e.g. "BTC-USDT_1min").
            // KuCoin symbols use '-' (spot) or no separator (futures), never '_'.
            // So if '_' is present after ':', it marks the interval boundary.
            if let Some(underscore_pos) = sym_and_suffix.rfind('_') {
                let interval_suffix = &sym_and_suffix[underscore_pos..]; // e.g. "_1min"
                format!("{}:*{}", channel, interval_suffix)
            } else {
                format!("{}:*", channel)
            }
        } else {
            topic.to_string()
        };

        Some(TopicKey::new(key))
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => Self::futures_registry(),
            _ => Self::spot_registry(),
        }
    }

    fn unsupported_by_exchange(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[]
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[StreamKind::OrderUpdate, StreamKind::BalanceUpdate, StreamKind::PositionUpdate]
    }

    fn decode_binary(&self, bytes: &[u8]) -> Result<Value, WebSocketError> {
        crate::core::websocket::transport::decode_binary_default(bytes)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builders
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry(account_type: AccountType) -> TopicRegistry {
    let is_futures = matches!(
        account_type,
        AccountType::FuturesCross | AccountType::FuturesIsolated
    );

    let mut b = TopicRegistry::builder();

    if is_futures {
        // Futures channels
        // Orderbook (snapshot) subscribes to level2Depth5 — registry key must match incoming topic.
        // OrderbookDelta (delta) subscribes to level2 — registry key must match.
        // Both level2Depth5 and level2Depth50 push snapshots, mapped to OrderbookDelta for compat.
        // /contract/instrument:* carries both mark.index.price (MarkPrice) and funding.rate (FundingRate)
        // subjects — register both to the same topic key so the dispatcher routes them.
        b = b
            .register(StreamKind::Ticker,        account_type, "/contractMarket/tickerV2:*",        parse_ticker)
            .register(StreamKind::Trade,          account_type, "/contractMarket/execution:*",       parse_trade)
            .register(StreamKind::Orderbook,      account_type, "/contractMarket/level2Depth5:*",    parse_orderbook_delta)
            .register(StreamKind::OrderbookDelta, account_type, "/contractMarket/level2:*",          parse_orderbook_delta)
            .register(StreamKind::OrderbookDelta, account_type, "/contractMarket/level2Depth50:*",   parse_orderbook_delta)
            .register(StreamKind::MarkPrice,      account_type, "/contract/instrument:*",            parse_mark_price)
            .register(StreamKind::IndexPrice,     account_type, "/contractMarket/indexPrice:*",      parse_index_price)
            .register(StreamKind::FundingRate,    account_type, "/contract/instrument:*",            parse_funding_rate)
            // Liquidation: requires auth — NotSupported as public feed. Registry entry removed.
            .register(StreamKind::OrderUpdate,    account_type, "/contractMarket/tradeOrders",       parse_order_update)
            .register(StreamKind::BalanceUpdate,  account_type, "/contractAccount/wallet",           parse_balance_update)
            .register(StreamKind::PositionUpdate, account_type, "/contract/positionAll",             parse_position_update);

        for (wire, internal) in FUTURES_KLINE_CHANNELS {
            b = b.register(
                StreamKind::Kline { interval: KlineInterval::new(*internal) },
                account_type,
                format!("/contractMarket/limitCandle:*_{}", wire),
                parse_kline,
            );
        }
    } else {
        // Spot channels
        // Orderbook (snapshot) subscribes to spotMarket/level2Depth5 — registry key must match.
        // OrderbookDelta (delta) subscribes to market/level2 — registry key must match.
        b = b
            .register(StreamKind::Ticker,        account_type, "/market/ticker:*",               parse_ticker)
            .register(StreamKind::Ticker,        account_type, "/market/snapshot:*",             parse_snapshot_ticker)
            .register(StreamKind::Trade,          account_type, "/market/match:*",                parse_trade)
            .register(StreamKind::Orderbook,      account_type, "/spotMarket/level2Depth5:*",     parse_orderbook_delta)
            .register(StreamKind::OrderbookDelta, account_type, "/market/level2:*",               parse_orderbook_delta)
            .register(StreamKind::OrderbookDelta, account_type, "/spotMarket/level2Depth50:*",    parse_orderbook_delta)
            .register(StreamKind::MarkPrice,      account_type, "/indicator/markPrice:*",         parse_mark_price)
            .register(StreamKind::IndexPrice,     account_type, "/indicator/index:*",             parse_index_price)
            .register(StreamKind::OrderUpdate,    account_type, "/spotMarket/tradeOrdersV2",      parse_order_update)
            .register(StreamKind::BalanceUpdate,  account_type, "/account/balance",               parse_balance_update);

        for (wire, internal) in SPOT_KLINE_CHANNELS {
            b = b.register(
                StreamKind::Kline { interval: KlineInterval::new(*internal) },
                account_type,
                format!("/market/candles:*_{}", wire),
                parse_kline,
            );
        }
    }

    b.build()
}

/// KuCoin spot kline wire names → internal names.
const SPOT_KLINE_CHANNELS: &[(&str, &str)] = &[
    ("1min",   "1m"),
    ("3min",   "3m"),
    ("5min",   "5m"),
    ("15min",  "15m"),
    ("30min",  "30m"),
    ("1hour",  "1h"),
    ("2hour",  "2h"),
    ("4hour",  "4h"),
    ("6hour",  "6h"),
    ("8hour",  "8h"),
    ("12hour", "12h"),
    ("1day",   "1d"),
    ("1week",  "1w"),
    ("1month", "1M"),
];

/// KuCoin futures kline wire names → internal names.
const FUTURES_KLINE_CHANNELS: &[(&str, &str)] = &[
    ("1",    "1m"),
    ("5",    "5m"),
    ("15",   "15m"),
    ("30",   "30m"),
    ("60",   "1h"),
    ("120",  "2h"),
    ("240",  "4h"),
    ("480",  "8h"),
    ("720",  "12h"),
    ("1440", "1d"),
    ("10080","1w"),
];

/// Map internal KlineInterval → KuCoin spot wire interval string.
fn kucoin_kline_interval(interval: &KlineInterval) -> &'static str {
    match interval.as_str() {
        "1m"  => "1min",
        "3m"  => "3min",
        "5m"  => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "1h"  => "1hour",
        "2h"  => "2hour",
        "4h"  => "4hour",
        "6h"  => "6hour",
        "8h"  => "8hour",
        "12h" => "12hour",
        "1d"  => "1day",
        "1w"  => "1week",
        "1M"  => "1month",
        _     => "1min",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsers  (fn(&Value) -> WebSocketResult<StreamEvent>)
//
// KuCoin data frame shape:
//   {"type":"message","topic":"/market/ticker:BTC-USDT","subject":"trade.ticker","data":{...}}
// ─────────────────────────────────────────────────────────────────────────────

fn frame_data(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("data")
        .ok_or_else(|| WebSocketError::Parse("kucoin frame missing 'data' field".into()))
}

/// Extract symbol from KuCoin topic string (after the ':').
/// e.g. "/market/ticker:BTC-USDT" → "BTC-USDT"
/// e.g. "/contractMarket/tickerV2:XBTUSDTM" → "XBTUSDTM"
fn topic_symbol(raw: &Value) -> String {
    raw.get("topic")
        .and_then(|t| t.as_str())
        .and_then(|t| t.split(':').nth(1))
        .unwrap_or("")
        .to_string()
}

fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let ticker = KuCoinParser::parse_ws_ticker(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let symbol = topic_symbol(raw);
    Ok(StreamEvent::Ticker { symbol, ticker })
}

fn parse_snapshot_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    // Snapshot wraps stats one level deeper under "data"
    let inner = data.get("data").unwrap_or(data);
    let ticker = KuCoinParser::parse_ws_snapshot_ticker(inner)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let symbol = topic_symbol(raw);
    Ok(StreamEvent::Ticker { symbol, ticker })
}

fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let trade = KuCoinParser::parse_ws_trade(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let symbol = topic_symbol(raw);
    Ok(StreamEvent::Trade { symbol, trade })
}

fn parse_orderbook_delta(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let delta = KuCoinParser::parse_ws_orderbook_delta(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let symbol = topic_symbol(raw);
    Ok(StreamEvent::OrderbookDelta { symbol, delta })
}

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let kline = KuCoinParser::parse_ws_kline(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    // topic: "/market/candles:BTC-USDT_1hour" → symbol="BTC-USDT", interval="1hour"
    let sym_interval = topic_symbol(raw);
    let (symbol, interval) = if let Some(pos) = sym_interval.find('_') {
        (sym_interval[..pos].to_string(), KlineInterval::new(&sym_interval[pos + 1..]))
    } else {
        (sym_interval, KlineInterval::new(""))
    };
    Ok(StreamEvent::Kline { symbol, interval, kline })
}

fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    KuCoinParser::parse_ws_mark_price(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))
}

fn parse_index_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let parse_f64 = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };
    let symbol = data
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let price = data
        .get("indexPrice")
        .and_then(parse_f64)
        .ok_or_else(|| WebSocketError::Parse("index_price: missing indexPrice".into()))?;
    let timestamp = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
    Ok(StreamEvent::IndexPrice { symbol, price, timestamp })
}

fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    KuCoinParser::parse_ws_funding_rate(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))
}

fn parse_order_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let symbol = data.get("symbol").and_then(|s| s.as_str()).unwrap_or("").to_string();
    let event = KuCoinParser::parse_ws_order_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderUpdate { symbol, event })
}

fn parse_balance_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let event = KuCoinParser::parse_ws_balance_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::BalanceUpdate(event))
}

fn parse_position_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let symbol = data.get("symbol").and_then(|s| s.as_str()).unwrap_or("").to_string();
    let event = KuCoinParser::parse_ws_position_update(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::PositionUpdate { symbol, event })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::websocket::StreamSpec;

    fn make_protocol() -> KuCoinProtocol {
        KuCoinProtocol::new(
            AccountType::Spot,
            false,
            Url::parse("wss://ws-api-spot.kucoin.com/?token=test&connectId=test").unwrap(),
            18_000,
        )
    }

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
        let proto = make_protocol();
        let reg = proto.topic_registry(AccountType::Spot);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "spot registry must have entries");
        assert!(reg.supports(&StreamKind::Ticker, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(reg.supports(&StreamKind::Orderbook, AccountType::Spot));
    }

    #[test]
    fn test_subscribe_frame_market_ticker() {
        let proto = make_protocol();
        let spec = spot_spec(StreamKind::Ticker);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["type"], "subscribe");
        assert!(
            v["topic"].as_str().unwrap().starts_with("/market/ticker:"),
            "topic should start with /market/ticker:"
        );
        assert_eq!(v["privateChannel"], false);
    }

    #[test]
    fn test_extract_topic_ticker_frame() {
        let proto = make_protocol();
        let frame = serde_json::json!({
            "type": "message",
            "topic": "/market/ticker:BTC-USDT",
            "subject": "trade.ticker",
            "data": {}
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "/market/ticker:*");
    }

    #[test]
    fn test_extract_topic_pong_returns_none() {
        let proto = make_protocol();
        let frame = serde_json::json!({
            "id": "12345",
            "type": "pong"
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_extract_topic_welcome_returns_none() {
        let proto = make_protocol();
        let frame = serde_json::json!({
            "id": "abc",
            "type": "welcome"
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_extract_topic_ack_returns_none() {
        let proto = make_protocol();
        let frame = serde_json::json!({
            "id": "1",
            "type": "ack"
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_is_pong() {
        let proto = make_protocol();
        let pong = serde_json::json!({"id": "1", "type": "pong"});
        assert!(proto.is_pong(&pong));
        let not_pong = serde_json::json!({"type": "message"});
        assert!(!proto.is_pong(&not_pong));
    }

    #[test]
    fn test_is_subscribe_ack() {
        let proto = make_protocol();
        let ack = serde_json::json!({"id": "1", "type": "ack"});
        assert!(proto.is_subscribe_ack(&ack));
        let not_ack = serde_json::json!({"type": "message"});
        assert!(!proto.is_subscribe_ack(&not_ack));
    }

    #[test]
    fn test_futures_registry_non_empty() {
        let proto = KuCoinProtocol::new(
            AccountType::FuturesCross,
            false,
            Url::parse("wss://ws-api-futures.kucoin.com/?token=test&connectId=test").unwrap(),
            18_000,
        );
        let reg = proto.topic_registry(AccountType::FuturesCross);
        assert!(reg.supports(&StreamKind::Ticker, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Trade, AccountType::FuturesCross));
        // Liquidation removed from public feed (requires auth) — no longer in registry.
        assert!(!reg.supports(&StreamKind::Liquidation, AccountType::FuturesCross));
    }

    #[test]
    fn test_ping_frame_is_json() {
        let proto = make_protocol();
        match proto.ping_frame() {
            Some(Message::Text(t)) => {
                let v: serde_json::Value = serde_json::from_str(&t).expect("ping must be valid JSON");
                assert_eq!(v["type"], "ping");
            }
            _ => panic!("expected Some(Text(...))"),
        }
    }

    #[test]
    fn test_kline_subscribe_frame() {
        let proto = make_protocol();
        let spec = spot_spec(StreamKind::Kline { interval: KlineInterval::new("1h") });
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        let topic = v["topic"].as_str().unwrap();
        assert!(topic.contains("1hour"), "1h maps to '1hour' on KuCoin spot: got {}", topic);
    }
}
