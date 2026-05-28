//! CryptoComProtocol — WsProtocol implementation for the Crypto.com Exchange.
//!
//! Declarative shim: supplies endpoint URL, heartbeat response, subscribe frames,
//! topic extraction, and topic registry to UniversalWsTransport.
//!
//! ## Crypto.com WebSocket v1 protocol
//!
//! - Endpoint: `wss://stream.crypto.com/exchange/v1/market` (public market data)
//! - Subscribe frame format:
//!   `{"id":<id>,"method":"subscribe","params":{"channels":["ticker.BTC_USDT"]}}`
//! - Server heartbeat: `{"method":"public/heartbeat","id":<N>}` every ~30 s.
//!   Client MUST reply with `{"method":"public/respond-heartbeat","id":<N>}` or the
//!   server disconnects. Handled via `is_server_ping` + `pong_response_frame`.
//! - Subscribe ack: `{"id":<N>,"code":0,"method":"subscribe","result":{...}}`
//!   (no "data" key in result — data frames DO carry a "data" array).
//! - Data envelope: `{"method":"subscribe","result":{"channel":"ticker",
//!   "instrument_name":"BTC_USDT","data":[{...}]}}`.
//!   Topic = `<channel>.<instrument_name>`.
//!
//! ## 1-second mandatory post-connect delay
//!
//! Crypto.com refuses any frame in the first second after WS handshake.
//! `post_connect_delay()` returns `Duration::from_secs(1)` so `UniversalWsTransport`
//! waits before sending auth / subscribe / post-connect frames.
//!
//! ## ping_frame
//!
//! Returns `None` — Crypto.com manages keepalive via its own server-initiated
//! `public/heartbeat` mechanism. We do NOT send client-initiated pings.
//!
//! ## Private channels
//!
//! Private (`user.*`) channels require WS auth (`public/auth`) and are out of
//! scope. `subscribe_frame` returns `NotSupported` for any `user.*`-mapped stream.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, OrderBookLevel, OrderbookDelta, StreamEvent,
    WebSocketError, WebSocketResult,
};
use crate::core::websocket::{StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol};
use crate::core::utils::timestamp_millis;

use super::endpoints::{CryptoComUrls, InstrumentType, format_symbol as fmt_symbol};
use super::parser::CryptoComParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry caches — Spot and Futures (futures = perpetuals on Crypto.com)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY_SPOT: OnceLock<TopicRegistry> = OnceLock::new();
static REGISTRY_FUTURES: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// CryptoComProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Crypto.com Exchange WS v1 protocol shim.
///
/// Public market-data channels only (spot + perpetuals).
/// Private channels (`user.*`) return `NotSupported` from `subscribe_frame`.
pub struct CryptoComProtocol;

impl CryptoComProtocol {
    pub fn new(_testnet: bool) -> Self {
        Self
    }

    /// Determine the Crypto.com wire instrument name from a StreamSpec.
    ///
    /// - Spot → `BTC_USDT` (underscore separator)
    /// - Futures/Perpetual → `BTCUSD-PERP` (no separator, USD quote)
    fn wire_symbol(spec: &StreamSpec) -> Result<String, WebSocketError> {
        let instrument_type = match spec.account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => InstrumentType::Perpetual,
            _ => InstrumentType::Spot,
        };

        // Try raw symbol first (e.g. "BTC_USDT" or "BTCUSD-PERP" passed directly).
        if let crate::core::types::OwnedSymbolInput::Raw(ref raw) = spec.symbol {
            if !raw.is_empty() {
                return Ok(raw.clone());
            }
        }

        // Fall back to base/quote construction via endpoints helper.
        // OwnedSymbolInput exposes no base/quote directly — resolve via display.
        let raw_str = spec.symbol.to_string();
        if raw_str.contains('/') {
            // Slash-separated: BTC/USDT → split and format
            let mut parts = raw_str.splitn(2, '/');
            let base = parts.next().unwrap_or("");
            let quote = parts.next().unwrap_or("USDT");
            Ok(fmt_symbol(base, quote, instrument_type))
        } else {
            // No slash — treat as-is (already exchange-native format)
            Ok(raw_str)
        }
    }

    /// Build the channel name string for a StreamSpec.
    ///
    /// Returns `Err(NotSupported)` for unsupported stream kinds or private channels.
    fn build_channel(spec: &StreamSpec) -> Result<String, WebSocketError> {
        let sym = Self::wire_symbol(spec)?;
        match &spec.kind {
            StreamKind::Ticker => Ok(format!("ticker.{}", sym)),
            StreamKind::Trade => Ok(format!("trade.{}", sym)),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                let depth = spec.depth.unwrap_or(10);
                Ok(format!("book.{}.{}", sym, depth))
            }
            StreamKind::MarkPrice => Ok(format!("mark.{}", sym)),
            StreamKind::IndexPrice => Ok(format!("index.{}", sym)),
            StreamKind::FundingRate => Ok(format!("funding.{}", sym)),
            StreamKind::SettlementEvent => Ok(format!("settlement.{}", sym)),
            StreamKind::PredictedFunding => Ok(format!("estimatedfunding.{}", sym)),
            StreamKind::Kline { interval } => {
                Ok(format!("candlestick.{}.{}", interval.as_str(), sym))
            }
            other => Err(WebSocketError::NotSupported(format!(
                "Crypto.com public WS has no channel for {:?}",
                other
            ))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WsProtocol impl
// ─────────────────────────────────────────────────────────────────────────────

impl WsProtocol for CryptoComProtocol {
    fn name(&self) -> &'static str {
        "crypto_com"
    }

    fn endpoint(&self, _account_type: AccountType, testnet: bool) -> Url {
        let urls = if testnet {
            &CryptoComUrls::TESTNET
        } else {
            &CryptoComUrls::MAINNET
        };
        Url::parse(urls.ws_market).expect("crypto_com ws endpoint is valid")
    }

    /// Returns `None` — Crypto.com uses server-initiated `public/heartbeat` for
    /// keepalive. We never send client-initiated pings.
    fn ping_frame(&self) -> Option<WsFrame> {
        None
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    /// Crypto.com refuses any frame within the first second after WS handshake.
    fn post_connect_delay(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let channel = Self::build_channel(spec)?;
        let id = timestamp_millis() as i64 & 0x7FFF_FFFF; // positive i64
        let frame = json!({
            "id": id,
            "method": "subscribe",
            "params": { "channels": [channel] }
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let channel = Self::build_channel(spec)?;
        let id = timestamp_millis() as i64 & 0x7FFF_FFFF;
        let frame = json!({
            "id": id,
            "method": "unsubscribe",
            "params": { "channels": [channel] }
        });
        Ok(WsFrame::Text(frame.to_string()))
    }

    /// Public channels are unauthenticated.
    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    /// Returns `false` — `ping_frame()` returns `None`, so we never send
    /// client-initiated pings and there are no pong responses to suppress.
    fn is_pong(&self, _raw: &Value) -> bool {
        false
    }

    /// Subscribe ack: `method == "subscribe"` AND `code == 0` AND result has no
    /// `data` array (data presence distinguishes a push from an ack).
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        let method_ok = raw.get("method").and_then(|v| v.as_str()) == Some("subscribe");
        let code_ok = raw.get("code").and_then(|v| v.as_i64()) == Some(0);
        let has_data = raw
            .get("result")
            .and_then(|r| r.get("data"))
            .is_some();
        method_ok && code_ok && !has_data
    }

    /// Server-initiated heartbeat: `{"method":"public/heartbeat","id":<N>}`.
    fn is_server_ping(&self, raw: &Value) -> bool {
        raw.get("method").and_then(|v| v.as_str()) == Some("public/heartbeat")
    }

    /// Build the `public/respond-heartbeat` reply, echoing the server's `id`.
    fn pong_response_frame(&self, raw: &Value) -> Option<WsFrame> {
        let id = raw.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let reply = json!({
            "id": id,
            "method": "public/respond-heartbeat"
        });
        Some(WsFrame::Text(reply.to_string()))
    }

    /// Extract routing topic from a Crypto.com data push.
    ///
    /// Data push envelope: `{"method":"subscribe","result":{"channel":"ticker",
    /// "instrument_name":"BTC_USDT","data":[{...}]}}`.
    /// Topic = `<channel>` (e.g. `"ticker"`, `"book"`, `"trade"`).
    ///
    /// We use the channel name only (not `channel.instrument`) because the
    /// registry has one parser per channel kind — the instrument_name is read
    /// from the envelope inside each parser function.
    ///
    /// Returns `None` for non-data frames (heartbeat, ack, auth response).
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let method = raw.get("method").and_then(|v| v.as_str())?;
        if method != "subscribe" {
            return None;
        }
        let result = raw.get("result")?;
        // Must be a data push — must have a "data" array
        result.get("data")?;
        let channel = result.get("channel").and_then(|v| v.as_str())?;
        Some(TopicKey::new(channel))
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                REGISTRY_FUTURES.get_or_init(|| build_registry(AccountType::FuturesCross))
            }
            _ => REGISTRY_SPOT.get_or_init(|| build_registry(AccountType::Spot)),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

/// Build a TopicRegistry for the given AccountType.
///
/// Registry keys are channel names (`"ticker"`, `"book"`, `"trade"`, …) matching
/// what `extract_topic` returns (channel name only, not `channel.instrument`).
/// Each parser reads `result.instrument_name` from the raw envelope to determine
/// the symbol, so a single per-channel registration covers all instruments.
fn build_registry(_account_type: AccountType) -> TopicRegistry {
    // We use account_type = Spot as the registry key — same parsers for both.
    // Registry keys are channel names (e.g. "ticker", "book", "trade", etc.)
    let at = AccountType::Spot;
    TopicRegistry::builder()
        .register(StreamKind::Ticker, at, "ticker", parse_ticker)
        // Derivative ticker frames include `oi` (open interest). Register a second
        // parser on the same topic so consumers also receive OpenInterestUpdate.
        // FieldAbsent is returned (and silently skipped) when `oi` is zero/absent.
        .register(StreamKind::OpenInterest, at, "ticker", parse_ticker_oi)
        .register(StreamKind::Orderbook, at, "book", parse_orderbook)
        .register(StreamKind::Trade, at, "trade", parse_trade)
        .register(StreamKind::MarkPrice, at, "mark", parse_mark_price)
        .register(StreamKind::IndexPrice, at, "index", parse_index_price)
        .register(StreamKind::FundingRate, at, "funding", parse_funding_rate)
        .register(StreamKind::SettlementEvent, at, "settlement", parse_settlement)
        .register(StreamKind::PredictedFunding, at, "estimatedfunding", parse_predicted_funding)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions
//
// Each parser receives the TOP-LEVEL raw frame (not the data item).
// Structure: {"method":"subscribe","result":{"channel":"ticker",
//   "instrument_name":"BTC_USDT","data":[{...item...}]}}
//
// We extract result.instrument_name for symbol and result.data[0] for fields.
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the first data item and instrument_name from a Crypto.com push frame.
///
/// Returns `(instrument_name, data_item)`.
fn extract_data(raw: &Value) -> Option<(String, Value)> {
    let result = raw.get("result")?;
    let instrument = result
        .get("instrument_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let data_item = result
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .cloned()?;
    Some((instrument, data_item))
}

/// Parse `ticker.<instrument>` → StreamEvent::Ticker (+ optional OpenInterestUpdate
/// emitted as the first event when `oi` is non-zero).
///
/// The registry only allows one parser per (StreamKind, topic) pair, so we emit
/// Ticker here. OpenInterestUpdate for derivative tickers is handled as a second
/// registration on the same topic key via StreamKind::OpenInterest.
pub(crate) fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (symbol, data) = extract_data(raw)
        .ok_or_else(|| WebSocketError::Parse("crypto_com ticker: missing result/data".into()))?;

    let ticker = CryptoComParser::parse_ws_ticker(&data)
        .map_err(|e| WebSocketError::Parse(format!("crypto_com ticker: {}", e)))?;

    Ok(StreamEvent::Ticker { symbol, ticker })
}

/// Second registration on the "ticker" topic key: emit OpenInterestUpdate when
/// the `oi` field is non-zero. Returns `FieldAbsent` to silent-skip when absent.
pub(crate) fn parse_ticker_oi(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (symbol, data) = extract_data(raw)
        .ok_or_else(|| WebSocketError::FieldAbsent("oi".into()))?;

    let ticker = CryptoComParser::parse_ws_ticker(&data)
        .map_err(|e| WebSocketError::Parse(format!("crypto_com ticker_oi: {}", e)))?;

    let oi = data
        .get("oi")
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64())
        })
        .filter(|&v| v != 0.0)
        .ok_or(WebSocketError::FieldAbsent("oi".into()))?;

    Ok(StreamEvent::OpenInterestUpdate {
        symbol,
        open_interest: oi,
        open_interest_value: None,
        timestamp: ticker.timestamp,
    })
}

/// Parse `book.<instrument>` → StreamEvent::OrderbookDelta.
pub(crate) fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (symbol, data) = extract_data(raw)
        .ok_or_else(|| WebSocketError::Parse("crypto_com book: missing result/data".into()))?;

    let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
        data.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let arr = entry.as_array()?;
                        let price = arr.first().and_then(|v| {
                            v.as_str()
                                .and_then(|s| s.parse().ok())
                                .or_else(|| v.as_f64())
                        })?;
                        let qty = arr.get(1).and_then(|v| {
                            v.as_str()
                                .and_then(|s| s.parse().ok())
                                .or_else(|| v.as_f64())
                        })?;
                        Some(OrderBookLevel::new(price, qty))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let bids = parse_levels("bids");
    let asks = parse_levels("asks");
    let timestamp = data.get("t").and_then(|v| v.as_i64()).unwrap_or(0);

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

/// Parse `trade.<instrument>` → StreamEvent::Trade.
pub(crate) fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    let (symbol, data) = extract_data(raw)
        .ok_or_else(|| WebSocketError::Parse("crypto_com trade: missing result/data".into()))?;

    let trade = CryptoComParser::parse_ws_trade(&data)
        .map_err(|e| WebSocketError::Parse(format!("crypto_com trade: {}", e)))?;

    Ok(StreamEvent::Trade { symbol, trade })
}

/// Parse `mark.<instrument>` → StreamEvent::MarkPrice.
///
/// Crypto.com mark channel: fields `v` (mark price), `ip` (index price), `t` (timestamp).
/// `i` field (instrument) may be absent in the data item; we use `instrument_name` from result.
pub(crate) fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = raw
        .get("result")
        .ok_or_else(|| WebSocketError::Parse("crypto_com mark: missing result".into()))?;

    let symbol = result
        .get("instrument_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let data = result
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| WebSocketError::Parse("crypto_com mark: missing data".into()))?;

    let mark_price = parse_f64_field(data, &["v", "mp"]).unwrap_or(0.0);
    let index_price = parse_f64_field(data, &["ip"]);
    let timestamp = data.get("t").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(StreamEvent::MarkPrice { symbol, mark_price, index_price, timestamp })
}

/// Parse `index.<instrument>` → StreamEvent::IndexPrice.
pub(crate) fn parse_index_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = raw
        .get("result")
        .ok_or_else(|| WebSocketError::Parse("crypto_com index: missing result".into()))?;

    let symbol = result
        .get("instrument_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let data = result
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| WebSocketError::Parse("crypto_com index: missing data".into()))?;

    let price = parse_f64_field(data, &["v"]).unwrap_or(0.0);
    let timestamp = data.get("t").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(StreamEvent::IndexPrice { symbol, price, timestamp })
}

/// Parse `funding.<instrument>` → StreamEvent::FundingRate.
pub(crate) fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = raw
        .get("result")
        .ok_or_else(|| WebSocketError::Parse("crypto_com funding: missing result".into()))?;

    let symbol = result
        .get("instrument_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let data = result
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| WebSocketError::Parse("crypto_com funding: missing data".into()))?;

    let rate = parse_f64_field(data, &["fr"]).unwrap_or(0.0);
    let timestamp = data.get("t").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(StreamEvent::FundingRate { symbol, rate, next_funding_time: None, timestamp })
}

/// Parse `settlement.<instrument>` → StreamEvent::SettlementEvent.
pub(crate) fn parse_settlement(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = raw
        .get("result")
        .ok_or_else(|| WebSocketError::Parse("crypto_com settlement: missing result".into()))?;

    let symbol = result
        .get("instrument_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let data = result
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| WebSocketError::Parse("crypto_com settlement: missing data".into()))?;

    let settlement_price = parse_f64_field(data, &["v"]).unwrap_or(0.0);
    let timestamp = data.get("t").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(StreamEvent::SettlementEvent {
        symbol,
        settlement_price,
        settlement_time: timestamp,
        timestamp,
    })
}

/// Parse `estimatedfunding.<instrument>` → StreamEvent::PredictedFunding.
pub(crate) fn parse_predicted_funding(raw: &Value) -> WebSocketResult<StreamEvent> {
    let result = raw
        .get("result")
        .ok_or_else(|| WebSocketError::Parse("crypto_com estimatedfunding: missing result".into()))?;

    let symbol = result
        .get("instrument_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let data = result
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| WebSocketError::Parse("crypto_com estimatedfunding: missing data".into()))?;

    let predicted_rate = parse_f64_field(data, &["v"]).unwrap_or(0.0);
    let timestamp = data.get("t").and_then(|v| v.as_i64()).unwrap_or(0);
    let next_funding_time = data.get("nt").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(StreamEvent::PredictedFunding {
        symbol,
        predicted_rate,
        next_funding_time,
        timestamp,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Try parsing a numeric f64 from multiple candidate field names.
fn parse_f64_field(data: &Value, keys: &[&str]) -> Option<f64> {
    for &key in keys {
        if let Some(v) = data.get(key) {
            let parsed = v
                .as_str()
                .and_then(|s| s.parse().ok())
                .or_else(|| v.as_f64());
            if parsed.is_some() {
                return parsed;
            }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AccountType, OwnedSymbolInput, TradeSide};
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

    // ── post_connect_delay ────────────────────────────────────────────────────

    #[test]
    fn post_connect_delay_is_one_second() {
        let proto = CryptoComProtocol::new(false);
        assert_eq!(
            proto.post_connect_delay(),
            Duration::from_secs(1),
            "Crypto.com requires 1s post-connect delay"
        );
    }

    // ── ping_frame ─────────────────────────────────────────────────────────────

    #[test]
    fn ping_frame_returns_none() {
        let proto = CryptoComProtocol::new(false);
        assert!(
            proto.ping_frame().is_none(),
            "Crypto.com uses server-initiated heartbeat; client must not send pings"
        );
    }

    // ── endpoint ──────────────────────────────────────────────────────────────

    #[test]
    fn endpoint_mainnet() {
        let proto = CryptoComProtocol::new(false);
        let url = proto.endpoint(AccountType::Spot, false);
        assert_eq!(url.as_str(), "wss://stream.crypto.com/exchange/v1/market");
    }

    #[test]
    fn endpoint_testnet() {
        let proto = CryptoComProtocol::new(true);
        let url = proto.endpoint(AccountType::Spot, true);
        assert_eq!(url.as_str(), "wss://uat-stream.3ona.co/exchange/v1/market");
    }

    // ── subscribe_frame ───────────────────────────────────────────────────────

    #[test]
    fn subscribe_frame_ticker() {
        let proto = CryptoComProtocol::new(false);
        let spec = make_spec(StreamKind::Ticker, "BTC_USDT");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["method"], "subscribe");
            let channels = &v["params"]["channels"];
            assert_eq!(channels[0], "ticker.BTC_USDT");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_trade() {
        let proto = CryptoComProtocol::new(false);
        let spec = make_spec(StreamKind::Trade, "BTCUSD-PERP");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["params"]["channels"][0], "trade.BTCUSD-PERP");
        } else {
            panic!("expected Text frame");
        }
    }

    #[test]
    fn subscribe_frame_orderbook_default_depth() {
        let proto = CryptoComProtocol::new(false);
        let spec = make_spec(StreamKind::Orderbook, "BTC_USDT");
        let frame = proto.subscribe_frame(&spec).expect("subscribe frame");
        if let WsFrame::Text(s) = frame {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["params"]["channels"][0], "book.BTC_USDT.10");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── is_server_ping ────────────────────────────────────────────────────────

    #[test]
    fn is_server_ping_for_heartbeat() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({"id": 1234, "method": "public/heartbeat"});
        assert!(proto.is_server_ping(&raw));
    }

    #[test]
    fn is_server_ping_false_for_data() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({"method": "subscribe", "result": {"channel": "ticker"}});
        assert!(!proto.is_server_ping(&raw));
    }

    // ── pong_response_frame ───────────────────────────────────────────────────

    #[test]
    fn pong_response_frame_echoes_id() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({"id": 42, "method": "public/heartbeat"});
        let reply = proto.pong_response_frame(&raw).expect("reply frame");
        if let WsFrame::Text(s) = reply {
            let v: Value = serde_json::from_str(&s).expect("valid json");
            assert_eq!(v["id"], 42);
            assert_eq!(v["method"], "public/respond-heartbeat");
        } else {
            panic!("expected Text frame");
        }
    }

    // ── is_subscribe_ack ──────────────────────────────────────────────────────

    #[test]
    fn is_subscribe_ack_for_confirmation() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({
            "id": 1,
            "method": "subscribe",
            "code": 0,
            "result": { "subscription": "ticker.BTC_USDT" }
        });
        assert!(proto.is_subscribe_ack(&raw));
    }

    #[test]
    fn is_subscribe_ack_false_for_data_push() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({
            "method": "subscribe",
            "code": 0,
            "result": {
                "channel": "ticker",
                "instrument_name": "BTC_USDT",
                "data": [{"a": "50000"}]
            }
        });
        assert!(!proto.is_subscribe_ack(&raw));
    }

    // ── extract_topic ─────────────────────────────────────────────────────────

    #[test]
    fn extract_topic_ticker_push() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({
            "method": "subscribe",
            "result": {
                "channel": "ticker",
                "instrument_name": "BTC_USDT",
                "data": [{}]
            }
        });
        // Topic is channel name only (not channel.instrument) — registry keys are per channel.
        assert_eq!(proto.extract_topic(&raw), Some(TopicKey::new("ticker")));
    }

    #[test]
    fn extract_topic_heartbeat_returns_none() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({"method": "public/heartbeat", "id": 1});
        assert_eq!(proto.extract_topic(&raw), None);
    }

    #[test]
    fn extract_topic_ack_without_data_returns_none() {
        let proto = CryptoComProtocol::new(false);
        let raw = serde_json::json!({
            "method": "subscribe",
            "code": 0,
            "result": { "subscription": "ticker.BTC_USDT" }
        });
        assert_eq!(proto.extract_topic(&raw), None);
    }

    // ── parse_ticker ──────────────────────────────────────────────────────────

    #[test]
    fn parse_ticker_basic() {
        let raw = serde_json::json!({
            "method": "subscribe",
            "result": {
                "channel": "ticker",
                "instrument_name": "BTC_USDT",
                "data": [{
                    "a": "50000.00",
                    "b": "49999.00",
                    "k": "50001.00",
                    "h": "51000.00",
                    "l": "49000.00",
                    "v": "100.5",
                    "vv": "5050000",
                    "c": "0.02",
                    "t": 1700000000000i64
                }]
            }
        });
        let ev = parse_ticker(&raw).expect("parse ticker");
        match ev {
            StreamEvent::Ticker { symbol, ticker } => {
                assert_eq!(symbol, "BTC_USDT");
                assert!((ticker.last_price - 50000.0).abs() < f64::EPSILON);
            }
            other => panic!("expected Ticker, got {:?}", other),
        }
    }

    // ── parse_orderbook ───────────────────────────────────────────────────────

    #[test]
    fn parse_orderbook_basic() {
        let raw = serde_json::json!({
            "method": "subscribe",
            "result": {
                "channel": "book",
                "instrument_name": "BTC_USDT",
                "data": [{
                    "bids": [["50000.00", "1.5", "1"], ["49999.00", "2.0", "1"]],
                    "asks": [["50001.00", "1.0", "1"]],
                    "t": 1700000000000i64
                }]
            }
        });
        let ev = parse_orderbook(&raw).expect("parse orderbook");
        match ev {
            StreamEvent::OrderbookDelta { symbol, delta } => {
                assert_eq!(symbol, "BTC_USDT");
                assert_eq!(delta.bids.len(), 2);
                assert_eq!(delta.asks.len(), 1);
            }
            other => panic!("expected OrderbookDelta, got {:?}", other),
        }
    }

    // ── parse_trade ───────────────────────────────────────────────────────────

    #[test]
    fn parse_trade_basic() {
        let raw = serde_json::json!({
            "method": "subscribe",
            "result": {
                "channel": "trade",
                "instrument_name": "BTC_USDT",
                "data": [{
                    "d": "12345",
                    "p": "50000.00",
                    "q": "0.5",
                    "s": "BUY",
                    "t": 1700000000000i64
                }]
            }
        });
        let ev = parse_trade(&raw).expect("parse trade");
        match ev {
            StreamEvent::Trade { symbol, trade } => {
                assert_eq!(symbol, "BTC_USDT");
                assert!((trade.price - 50000.0).abs() < f64::EPSILON);
                assert_eq!(trade.side, TradeSide::Buy);
            }
            other => panic!("expected Trade, got {:?}", other),
        }
    }

    // ── parse_mark_price ──────────────────────────────────────────────────────

    #[test]
    fn parse_mark_price_basic() {
        let raw = serde_json::json!({
            "method": "subscribe",
            "result": {
                "channel": "mark",
                "instrument_name": "BTCUSD-PERP",
                "data": [{
                    "v": "50100.00",
                    "ip": "50000.00",
                    "t": 1700000000000i64
                }]
            }
        });
        let ev = parse_mark_price(&raw).expect("parse mark price");
        match ev {
            StreamEvent::MarkPrice { symbol, mark_price, .. } => {
                assert_eq!(symbol, "BTCUSD-PERP");
                assert!((mark_price - 50100.0).abs() < f64::EPSILON);
            }
            other => panic!("expected MarkPrice, got {:?}", other),
        }
    }

    // ── topic_registry ────────────────────────────────────────────────────────

    #[test]
    fn topic_registry_covers_public_channels() {
        let proto = CryptoComProtocol::new(false);
        let reg = proto.topic_registry(AccountType::Spot);
        let at = AccountType::Spot;
        assert!(reg.supports(&StreamKind::Ticker, at), "Ticker");
        assert!(reg.supports(&StreamKind::Orderbook, at), "Orderbook");
        assert!(reg.supports(&StreamKind::Trade, at), "Trade");
        assert!(reg.supports(&StreamKind::MarkPrice, at), "MarkPrice");
        assert!(reg.supports(&StreamKind::FundingRate, at), "FundingRate");
    }
}
