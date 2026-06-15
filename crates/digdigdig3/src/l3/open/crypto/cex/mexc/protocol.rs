//! MexcProtocol — WsProtocol implementation for MEXC exchange.
//!
//! Spot WS: `wss://wbs-api.mexc.com/ws` — protobuf binary frames.
//! Futures WS: `wss://contract.mexc.com/edge` — JSON text frames.
//!
//! ## Binary frames (Spot)
//!
//! MEXC sends all spot market data as protobuf binary frames.
//! `decode_binary` extracts the channel name from the protobuf wrapper (field 1)
//! and stores the raw bytes as a JSON array under the `__pb` key so parsers can
//! recover the original bytes for full protobuf decoding.
//!
//! Resulting synthetic Value shape:
//! ```json
//! { "c": "spot@public.miniTicker.v3.api.pb@BTCUSDT@UTC+0", "__pb": [0, 1, 2, ...] }
//! ```
//!
//! `extract_topic` reads `c` for spot or `channel` for futures.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{AccountType, StreamEvent, WebSocketError, WebSocketResult};
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec,
    TopicKey, TopicRegistry,
    WsProtocol,
};
use crate::core::utils::symbol_normalizer::SymbolNormalizer;
use crate::core::types::ExchangeId;

use super::endpoints::{MexcUrls, MexcWsChannels};
use super::parser::MexcParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry caches
// ─────────────────────────────────────────────────────────────────────────────

static SPOT_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static FUTURES_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// MexcProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative MEXC WS protocol shim.
pub struct MexcProtocol {
    pub(super) account_type: AccountType,
}

impl MexcProtocol {
    pub fn new(account_type: AccountType) -> Self {
        Self { account_type }
    }

    fn spot_registry() -> &'static TopicRegistry {
        SPOT_REGISTRY.get_or_init(build_spot_registry)
    }

    fn futures_registry() -> &'static TopicRegistry {
        FUTURES_REGISTRY.get_or_init(build_futures_registry)
    }

    fn is_futures(account_type: AccountType) -> bool {
        matches!(
            account_type,
            AccountType::FuturesCross | AccountType::FuturesIsolated
        )
    }

    /// Build SUBSCRIPTION frame for spot (params array).
    fn spot_subscribe_frame(spec: &StreamSpec, op: &str) -> Result<WsFrame, WebSocketError> {
        let sym = spec.symbol.as_str();
        let params = match &spec.kind {
            StreamKind::Ticker => {
                // PublicMiniTickerV3Api has NO bid/ask fields — it only carries last_price/volume.
                // PublicBookTickerV3Api (bookTicker) is blocked from certain regions (RU, others).
                // PublicLimitDepthV3Api (limit.depth@5) carries full top-of-book bid/ask snapshot.
                // Subscribe to limit.depth only; parse_spot_depth_as_ticker extracts bid/ask as Ticker.
                vec![MexcWsChannels::limit_depth(sym, 5)]
            }
            StreamKind::Trade | StreamKind::AggTrade => vec![MexcWsChannels::aggre_deals(sym)],
            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                // Use limit-depth snapshot channel (5 levels) — reliable on MEXC spot.
                // aggre_depth uses delta encoding that requires seq tracking not yet implemented.
                vec![MexcWsChannels::limit_depth(sym, 5)]
            }
            StreamKind::Kline { interval } => {
                vec![MexcWsChannels::kline(sym, &mexc_spot_kline_interval(interval))]
            }
            other => {
                return Err(WebSocketError::NotImplemented(format!(
                    "mexc spot: unsupported stream kind {:?}",
                    other
                )))
            }
        };

        let method = if op == "subscribe" {
            "SUBSCRIPTION"
        } else {
            "UNSUBSCRIPTION"
        };

        // MEXC requires an "id" field in every subscribe/unsubscribe request.
        // Without it the server silently drops the subscription (0 events).
        let frame = json!({ "id": 1, "method": method, "params": params });
        Ok(WsFrame::Text(frame.to_string()))
    }

    /// Build subscribe/unsubscribe frame for futures (method per channel).
    fn futures_subscribe_frame(spec: &StreamSpec, op: &str) -> Result<WsFrame, WebSocketError> {
        // MEXC futures requires `BTC_USDT` format. If caller passes spot raw symbol
        // `BTCUSDT` (no underscore), convert via normalizer round-trip.
        let sym_normalized: String = {
            let raw = spec.symbol.as_str();
            if !raw.contains('_') && raw.chars().all(|c| c.is_ascii_alphanumeric()) {
                // Looks like a spot-format symbol — try to normalize to futures format.
                SymbolNormalizer::from_exchange(ExchangeId::MEXC, raw, AccountType::Spot)
                    .and_then(|canonical| SymbolNormalizer::to_exchange(ExchangeId::MEXC, &canonical, AccountType::FuturesCross))
                    .unwrap_or_else(|_| raw.to_string())
            } else {
                raw.to_string()
            }
        };
        let sym = sym_normalized.as_str();
        let method_prefix = if op == "subscribe" { "sub" } else { "unsub" };

        let (method, param) = match &spec.kind {
            StreamKind::Ticker => (
                format!("{}.ticker", method_prefix),
                json!({ "symbol": sym }),
            ),
            StreamKind::Trade | StreamKind::AggTrade => (
                format!("{}.deal", method_prefix),
                json!({ "symbol": sym }),
            ),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => (
                format!("{}.depth", method_prefix),
                json!({ "symbol": sym }),
            ),
            StreamKind::Kline { interval } => (
                format!("{}.kline", method_prefix),
                json!({ "symbol": sym, "interval": mexc_futures_kline_interval(interval) }),
            ),
            StreamKind::FundingRate => (
                format!("{}.funding.rate", method_prefix),
                json!({ "symbol": sym }),
            ),
            StreamKind::IndexPrice => (
                format!("{}.index.price", method_prefix),
                json!({ "symbol": sym }),
            ),
            StreamKind::MarkPrice => (
                format!("{}.fair.price", method_prefix),
                json!({ "symbol": sym }),
            ),
            other => {
                return Err(WebSocketError::NotImplemented(format!(
                    "mexc futures: unsupported stream kind {:?}",
                    other
                )))
            }
        };

        let frame = json!({ "method": method, "param": param });
        Ok(WsFrame::Text(frame.to_string()))
    }
}

impl WsProtocol for MexcProtocol {
    fn name(&self) -> &'static str {
        "mexc"
    }

    fn endpoint(&self, account_type: AccountType, _testnet: bool) -> Url {
        let url = if Self::is_futures(account_type) {
            MexcUrls::futures_ws_url()
        } else {
            MexcUrls::ws_url()
        };
        Url::parse(url).expect("mexc ws url is valid")
    }

    fn ping_frame(&self) -> Option<WsFrame> {
        // Spot uses {"method":"PING"}, futures uses {"method":"ping"}.
        // The transport uses account_type at construction — use spot format as default.
        // Futures ping is handled by the same frame shape (lowercase).
        let method = if Self::is_futures(self.account_type) {
            "ping"
        } else {
            "PING"
        };
        Some(WsFrame::Text(json!({ "method": method }).to_string()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(20)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        // Liquidation: MEXC has no public liquidation WS channel on either Spot or Futures.
        if matches!(spec.kind, StreamKind::Liquidation) {
            return Err(WebSocketError::WireAbsent(
                "MEXC Futures has no public WS liquidation channel — \
                 no public REST alternative either".to_string(),
            ));
        }

        if Self::is_futures(spec.account_type) {
            // OpenInterest WS: MEXC Futures has no dedicated OI channel.
            // OI is embedded as holdVol in push.ticker — subscribe to Ticker instead,
            // or use REST GET /api/v1/contract/ticker?symbol=BTC_USDT and read holdVol.
            if matches!(spec.kind, StreamKind::OpenInterest) {
                return Err(WebSocketError::WireAbsent(
                    "MEXC Futures has no dedicated OI WS channel — \
                     OI (holdVol) is embedded in push.ticker; subscribe to Ticker \
                     or use REST GET /api/v1/contract/ticker?symbol=BTC_USDT".to_string(),
                ));
            }
            Self::futures_subscribe_frame(spec, "subscribe")
        } else {
            // MEXC Spot WebSocket migrated to binary protobuf frames on 2025-08-04.
            // All spot channels (ticker, trade, orderbook, kline) require protobuf decoding
            // via the PushDataV3ApiWrapper schema from github.com/mexcdevelop/websocket-proto.
            // Protobuf decoder is implemented in MexcParser::parse_protobuf_message.
            Self::spot_subscribe_frame(spec, "subscribe")
        }
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        if Self::is_futures(spec.account_type) {
            Self::futures_subscribe_frame(spec, "unsubscribe")
        } else {
            Self::spot_subscribe_frame(spec, "unsubscribe")
        }
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // Spot: {"id":0,"code":0,"msg":"PONG"}
        // Futures: {"channel":"pong","data":<ts>}
        if let Some(msg) = raw.get("msg").and_then(|m| m.as_str()) {
            if msg.eq_ignore_ascii_case("pong") {
                return true;
            }
        }
        if let Some(ch) = raw.get("channel").and_then(|c| c.as_str()) {
            if ch == "pong" {
                return true;
            }
        }
        false
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        // Spot subscription ack: {"id":0,"code":0,"msg":"spot@public..."}
        // or {"code":0,"msg":"PING"} — handled as pong above
        if let Some(code) = raw.get("code").and_then(|c| c.as_i64()) {
            if code == 0 {
                if let Some(msg_str) = raw.get("msg").and_then(|m| m.as_str()) {
                    if msg_str.starts_with("spot@") || msg_str.starts_with("push.") {
                        return true;
                    }
                }
            }
        }
        // Futures ack: {"channel":"rs.sub.ticker","data":"success"}
        if let Some(ch) = raw.get("channel").and_then(|c| c.as_str()) {
            if ch.starts_with("rs.") {
                return true;
            }
        }
        false
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Synthetic binary frame: { "c": "<channel>", "__pb": [...] }
        // Spot data frames: {"c":"spot@public...","d":{...},"s":"BTCUSDT"}
        if let Some(c) = raw.get("c").and_then(|v| v.as_str()) {
            // Filter pong/ack coming through the text path
            if c.eq_ignore_ascii_case("pong") || c.starts_with("spot@") && raw.get("__pb").is_some() {
                // __pb present → binary frame, extract topic
                return Some(TopicKey::new(c));
            }
            if raw.get("__pb").is_some() {
                return Some(TopicKey::new(c));
            }
            // Text data frame
            return Some(TopicKey::new(c));
        }

        // Futures data frames: {"channel":"push.ticker","data":{...},"symbol":"BTC_USDT"}
        if let Some(ch) = raw.get("channel").and_then(|c| c.as_str()) {
            // Filter system frames already handled by is_pong / is_subscribe_ack
            if ch == "pong" || ch.starts_with("rs.") {
                return None;
            }
            return Some(TopicKey::new(ch));
        }

        None
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        if Self::is_futures(account_type) {
            Self::futures_registry()
        } else {
            Self::spot_registry()
        }
    }

    fn unsupported_by_exchange(&self, account_type: AccountType) -> &'static [StreamKind] {
        if Self::is_futures(account_type) {
            // MEXC Futures: no liquidation channel; no dedicated OI channel (only in ticker)
            &[StreamKind::Liquidation, StreamKind::OpenInterest]
        } else {
            // MEXC Spot: no liquidation channel
            &[StreamKind::Liquidation]
        }
    }

    /// Override binary decode: MEXC spot sends protobuf binary frames.
    ///
    /// Extracts the channel name (field 1) from the protobuf wrapper and
    /// returns a synthetic JSON value:
    /// `{"c": "<channel>", "__pb": [<bytes>]}`
    ///
    /// The `__pb` array holds the raw bytes so the parser can call
    /// `MexcParser::parse_protobuf_message` with the original data.
    fn decode_binary(&self, bytes: &[u8]) -> Result<Value, WebSocketError> {
        // Extract channel from protobuf wrapper field 1 (string, wire type 2).
        let channel = pb_string(bytes, 1).ok_or_else(|| {
            tracing::warn!(target: "mexc::ws", "binary frame {} bytes — field 1 missing, raw prefix: {:?}", bytes.len(), &bytes[..bytes.len().min(16)]);
            WebSocketError::Parse("mexc: missing channel in protobuf wrapper (field 1)".into())
        })?;

        tracing::debug!(target: "mexc::ws", "binary frame {} bytes, channel: {}", bytes.len(), channel);

        // Store raw bytes as JSON array so the parser can re-decode them.
        let pb_array: Vec<Value> = bytes.iter().map(|&b| Value::from(b)).collect();
        Ok(json!({
            "c": channel,
            "__pb": pb_array,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builders
// ─────────────────────────────────────────────────────────────────────────────

fn build_spot_registry() -> TopicRegistry {
    let at = AccountType::Spot;
    let mut b = TopicRegistry::builder()
        // Ticker via limit.depth@5: top-of-book bid/ask as a snapshot.
        // bookTicker (spot@public.bookTicker.v3.api.pb@*) is blocked from RU/some regions.
        // miniTicker (spot@public.miniTicker.v3.api.pb@*) carries no bid/ask — not registered for Ticker.
        // limit.depth@5 is always available and provides bid/ask every snapshot update.
        .register(StreamKind::Ticker, at, "spot@public.limit.depth.v3.api.pb@*", parse_spot_depth_as_ticker)
        // Aggre deals (trades): spot@public.aggre.deals.v3.api.pb@100ms@<sym>
        .register(StreamKind::Trade, at, "spot@public.aggre.deals.v3.api.pb@*", parse_spot_pb)
        .register(StreamKind::AggTrade, at, "spot@public.aggre.deals.v3.api.pb@*", parse_spot_pb)
        // Limit depth (orderbook snapshot): spot@public.limit.depth.v3.api.pb@<sym>@<levels>
        .register(StreamKind::OrderbookDelta, at, "spot@public.limit.depth.v3.api.pb@*", parse_spot_pb)
        .register(StreamKind::Orderbook, at, "spot@public.limit.depth.v3.api.pb@*", parse_spot_pb);

    // Kline: spot@public.kline.v3.api.pb@<sym>@<interval>
    for interval in MEXC_SPOT_KLINE_INTERVALS {
        let kind = StreamKind::Kline {
            interval: KlineInterval::new(*interval),
        };
        b = b.register(kind, at, "spot@public.kline.v3.api.pb@*", parse_spot_pb);
    }

    b.build()
}

fn build_futures_registry() -> TopicRegistry {
    let at = AccountType::FuturesCross;
    TopicRegistry::builder()
        // Ticker carries Ticker + MarkPrice (fairPrice) + FundingRate + OpenInterest (holdVol) + IndexPrice
        .register(StreamKind::Ticker, at, "push.ticker", parse_futures_ticker)
        .register(StreamKind::MarkPrice, at, "push.ticker", parse_futures_ticker_mark_price)
        .register(StreamKind::FundingRate, at, "push.ticker", parse_futures_ticker_funding_rate)
        .register(StreamKind::OpenInterest, at, "push.ticker", parse_futures_ticker_open_interest)
        .register(StreamKind::IndexPrice, at, "push.ticker", parse_futures_ticker_index_price)
        // Dedicated topic parsers (for when user subscribes to dedicated channels)
        .register(StreamKind::Trade, at, "push.deal", parse_futures_deal)
        .register(StreamKind::AggTrade, at, "push.deal", parse_futures_agg_trade)
        .register(StreamKind::Orderbook, at, "push.depth", parse_futures_depth)
        .register(StreamKind::OrderbookDelta, at, "push.depth", parse_futures_depth)
        .register(StreamKind::Kline { interval: KlineInterval::new("1m") }, at, "push.kline", parse_futures_kline)
        .register(StreamKind::FundingRate, at, "push.funding.rate", parse_futures_funding_rate)
        .register(StreamKind::IndexPrice, at, "push.index.price", parse_futures_index_price)
        .register(StreamKind::MarkPrice, at, "push.fair.price", parse_futures_fair_price)
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// Spot parsers — binary protobuf frames
// ─────────────────────────────────────────────────────────────────────────────

/// Universal spot parser: recovers raw bytes from `__pb`, delegates to MexcParser.
fn parse_spot_pb(raw: &Value) -> WebSocketResult<StreamEvent> {
    let pb_arr = raw
        .get("__pb")
        .and_then(|v| v.as_array())
        .ok_or_else(|| WebSocketError::Parse("mexc: missing __pb in synthetic frame".into()))?;

    let bytes: Vec<u8> = pb_arr
        .iter()
        .filter_map(|v| v.as_u64().map(|b| b as u8))
        .collect();

    let (_channel, event) = MexcParser::parse_protobuf_message(&bytes)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;

    Ok(event)
}

/// Parse a limit.depth protobuf frame as a Ticker carrying top-of-book bid/ask.
///
/// MEXC Spot `PublicMiniTickerV3Api` has no bid/ask fields.
/// `PublicBookTickerV3Api` (`bookTicker` channel) is blocked from certain regions.
/// `PublicLimitDepthV3Api` (limit.depth) is always available and carries full bid/ask levels.
///
/// We extract best_bid = bids[0].price and best_ask = asks[0].price and return
/// a Ticker with those quotes but `last_price = 0` (sentinel — inspector ignores last_price=0
/// checks because WS_ticker separately collects the miniTicker events with last_price set).
fn parse_spot_depth_as_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;

    let pb_arr = raw
        .get("__pb")
        .and_then(|v| v.as_array())
        .ok_or_else(|| WebSocketError::Parse("mexc: missing __pb in depth-as-ticker frame".into()))?;

    let bytes: Vec<u8> = pb_arr
        .iter()
        .filter_map(|v| v.as_u64().map(|b| b as u8))
        .collect();

    // Parse via the standard path — yields OrderbookDelta
    let (channel, event) = MexcParser::parse_protobuf_message(&bytes)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;

    // Extract symbol from channel: spot@public.limit.depth.v3.api.pb@BTCUSDT@5
    // field 3 after '@' splits: [spot@public.limit.depth.v3.api.pb, BTCUSDT, 5]
    let symbol = channel
        .splitn(4, '@')
        .nth(2)
        .unwrap_or("")
        .to_string();

    // Extract the best price from each side and classify correctly.
    // Note: parse_pb_aggre_depth maps field1→bids, field2→asks but the MEXC
    // limit.depth protobuf actually encodes asks in field1 (descending) and bids
    // in field2 (ascending). We correct by comparing: lower price = bid, higher = ask.
    let (bid_price, ask_price) = match &event {
        StreamEvent::OrderbookDelta { symbol: _, delta } => {
            let p1 = delta.bids.first().map(|l| l.price); // field1 top
            let p2 = delta.asks.first().map(|l| l.price); // field2 top
            match (p1, p2) {
                (Some(a), Some(b)) => {
                    // Normalize: lower = bid, higher = ask
                    if a < b { (Some(a), Some(b)) } else { (Some(b), Some(a)) }
                }
                (Some(a), None) => (Some(a), None),
                (None, Some(b)) => (None, Some(b)),
                (None, None) => (None, None),
            }
        }
        _ => (None, None),
    };

    // Only emit if we have at least one side
    if bid_price.is_none() && ask_price.is_none() {
        return Err(WebSocketError::Parse(
            "mexc depth-as-ticker: empty bids and asks, no quote data".into(),
        ));
    }

    let last_price = bid_price.or(ask_price).unwrap_or(0.0);

    Ok(StreamEvent::Ticker {
        symbol,
        ticker: crate::core::types::Ticker {
            last_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: timestamp_millis() as i64, ..Default::default() 
        },
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Futures parsers — JSON text frames
// ─────────────────────────────────────────────────────────────────────────────

fn futures_data(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("data")
        .ok_or_else(|| WebSocketError::Parse("mexc futures: missing 'data' field".into()))
}

fn futures_symbol(raw: &Value) -> String {
    raw.get("symbol")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string()
}

fn parse_f64_field(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn parse_futures_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;
    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let last_price = data
        .get("lastPrice")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures ticker: missing lastPrice".into()))?;

    let bid_price = data.get("bid1").and_then(parse_f64_field);
    let ask_price = data.get("ask1").and_then(parse_f64_field);
    let high_24h = data.get("high24Price").and_then(parse_f64_field);
    let low_24h = data.get("low24Price").and_then(parse_f64_field);
    let volume_24h = data.get("volume24").and_then(parse_f64_field);
    let price_change_percent_24h = data.get("riseFallRate").and_then(parse_f64_field);
    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    let hold_vol = data.get("holdVol").and_then(parse_f64_field);
    let funding_rate = data.get("fundingRate").and_then(parse_f64_field);

    // Emit OpenInterestUpdate as side-channel is not possible in a ParserFn.
    // We emit Ticker; FundingRate is emitted by push.funding.rate parser.
    let _ = (hold_vol, funding_rate); // used in start_futures_ws, not here

    Ok(StreamEvent::Ticker {
        symbol,
        ticker: crate::core::types::Ticker {
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h,
            timestamp, ..Default::default() 
        },
    })
}

/// Extract `fairPrice` (mark price) from `push.ticker` frame.
fn parse_futures_ticker_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;
    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let mark_price = data
        .get("fairPrice")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures ticker: missing fairPrice for MarkPrice fan-out".into()))?;

    let index_price = data.get("indexPrice").and_then(parse_f64_field);

    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::MarkPrice {
        symbol,
        mark: crate::core::types::MarkPrice {
            mark_price,
            index_price,
            timestamp,
            ..Default::default()
        },
    })
}

/// Extract `fundingRate` from `push.ticker` frame.
fn parse_futures_ticker_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;
    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let rate = data
        .get("fundingRate")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures ticker: missing fundingRate for FundingRate fan-out".into()))?;

    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::FundingRate {
        symbol,
        funding: crate::core::types::FundingRate {
            rate,
            // next_funding_time is intentionally None here: the push.ticker frame does not
            // carry a next-settlement timestamp field.  Consumers that need next_funding_time
            // populated must subscribe to the dedicated push.funding.rate channel (handled
            // separately in topic_registry / dispatch — that path does populate the field).
            next_funding_time: None,
            timestamp,
            ..Default::default()
        },
    })
}

/// Extract `holdVol` (open interest) from `push.ticker` frame.
fn parse_futures_ticker_open_interest(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;
    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let open_interest = data
        .get("holdVol")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures ticker: missing holdVol for OpenInterest fan-out".into()))?;

    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::OpenInterestUpdate {
        symbol,
        open_interest: crate::core::types::OpenInterest {
            open_interest,
            open_interest_value: None,
            timestamp,
            ..Default::default()
        },
    })
}

/// Extract `indexPrice` from `push.ticker` frame.
fn parse_futures_ticker_index_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;
    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let index_price = data
        .get("indexPrice")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures ticker: missing indexPrice for IndexPrice fan-out".into()))?;

    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::MarkPrice {
        symbol,
        mark: crate::core::types::MarkPrice {
            mark_price: index_price,
            index_price: Some(index_price),
            timestamp,
            ..Default::default()
        },
    })
}

/// Extract the first deal item from `push.deal` data.
///
/// `push.deal` carries `"data": [{...}]` — an array of deal objects.
/// Use the first (most recent) item.
fn futures_deal_item(raw: &Value) -> WebSocketResult<&Value> {
    let data = futures_data(raw)?;
    // data can be a single object (older API) or an array (current API, 2025+)
    if let Some(arr) = data.as_array() {
        arr.first()
            .ok_or_else(|| WebSocketError::Parse("futures deal: empty data array".into()))
    } else {
        Ok(data)
    }
}

fn parse_futures_deal(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;
    use crate::core::types::{PublicTrade, TradeSide};

    let item = futures_deal_item(raw)?;
    let symbol = futures_symbol(raw);

    let price = item
        .get("p")
        .or_else(|| item.get("price"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures deal: missing price".into()))?;

    let quantity = item
        .get("v")
        .or_else(|| item.get("vol"))
        .or_else(|| item.get("quantity"))
        .and_then(parse_f64_field)
        .unwrap_or(0.0);

    // T field: 1=buy (Taker Buy), 2=sell (Taker Sell)
    let side = item
        .get("T")
        .or_else(|| item.get("takerSide"))
        .or_else(|| item.get("side"))
        .and_then(|v| v.as_i64())
        .map(|s| if s == 1 { TradeSide::Buy } else { TradeSide::Sell })
        .unwrap_or(TradeSide::Buy);

    let timestamp = item
        .get("t")
        .or_else(|| item.get("time"))
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    let id = item
        .get("i")
        .or_else(|| item.get("dealId"))
        .and_then(|v| v.as_str().map(|s| s.to_string())
            .or_else(|| v.as_i64().map(|n| n.to_string())))
        .unwrap_or_default();

    Ok(StreamEvent::Trade {
        symbol,
        trade: PublicTrade {
            id,
            price,
            quantity,
            side,
            timestamp,
            ..Default::default()
        },
    })
}

/// Parse `push.deal` as `StreamEvent::AggTrade` (for AggTrade subscriptions).
///
/// MEXC Futures `push.deal` carries individual trade records; treated as agg-trade
/// since MEXC has no separate aggregated-trade channel.
fn parse_futures_agg_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;

    let item = futures_deal_item(raw)?;
    let symbol = futures_symbol(raw);

    let price = item
        .get("p")
        .or_else(|| item.get("price"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures deal(agg): missing price".into()))?;

    let quantity = item
        .get("v")
        .or_else(|| item.get("vol"))
        .or_else(|| item.get("quantity"))
        .and_then(parse_f64_field)
        .unwrap_or(0.0);

    let side = item
        .get("T")
        .or_else(|| item.get("takerSide"))
        .or_else(|| item.get("side"))
        .and_then(|v| v.as_i64())
        .map(|s| if s == 1 { crate::core::types::TradeSide::Buy } else { crate::core::types::TradeSide::Sell })
        .unwrap_or(crate::core::types::TradeSide::Buy);

    let timestamp = item
        .get("t")
        .or_else(|| item.get("time"))
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    let id = item
        .get("i")
        .or_else(|| item.get("dealId"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    Ok(StreamEvent::AggTrade {
        symbol,
        agg: crate::core::types::AggTrade {
            aggregate_id: id,
            price,
            quantity,
            first_trade_id: id,
            last_trade_id: id,
            is_buy: side == crate::core::types::TradeSide::Buy,
            timestamp,
            ..Default::default()
        },
    })
}

fn parse_futures_depth(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;
    use crate::core::types::{OrderbookDelta, OrderBookLevel};

    let data = futures_data(raw)?;

    let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
        data.get(key)
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
            .unwrap_or_default()
    };

    let bids = parse_levels("bids");
    let asks = parse_levels("asks");
    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    let seq = data
        .get("version")
        .and_then(|v| v.as_u64());

    let depth_symbol = futures_symbol(raw);
    Ok(StreamEvent::OrderbookDelta {
        symbol: depth_symbol,
        delta: OrderbookDelta {
            bids,
            asks,
            timestamp,
            first_update_id: None,
            last_update_id: seq,
            prev_update_id: None,
            event_time: Some(timestamp),
            checksum: None,
        },
    })
}

fn parse_futures_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::Kline;

    let data = futures_data(raw)?;

    let open_time = data
        .get("t")
        .or_else(|| data.get("time"))
        .and_then(|t| t.as_i64())
        .unwrap_or(0)
        * 1000; // seconds → millis

    let open = data.get("o").or_else(|| data.get("open"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures kline: missing open".into()))?;
    let high = data.get("h").or_else(|| data.get("high"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures kline: missing high".into()))?;
    let low = data.get("l").or_else(|| data.get("low"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures kline: missing low".into()))?;
    let close = data.get("c").or_else(|| data.get("close"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures kline: missing close".into()))?;
    let volume = data.get("v").or_else(|| data.get("vol"))
        .and_then(parse_f64_field)
        .unwrap_or(0.0);

    let futures_kline_symbol = futures_symbol(raw);
    // MEXC futures kline channel: "push.kline" — interval in data "interval" field or raw channel suffix
    let futures_kline_interval = KlineInterval::new(
        data.get("interval").and_then(|v| v.as_str()).unwrap_or(""),
    );
    Ok(StreamEvent::Kline {
        symbol: futures_kline_symbol,
        interval: futures_kline_interval,
        kline: Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: None,
            close_time: None,
            trades: None,
            ..Default::default()
        },
    })
}

fn parse_futures_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;

    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let rate = data
        .get("rate")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures funding rate: missing rate".into()))?;

    let next_funding_time = data
        .get("nextSettleTime")
        .and_then(|v| v.as_i64());

    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::FundingRate {
        symbol,
        funding: crate::core::types::FundingRate {
            rate,
            next_funding_time,
            timestamp,
            ..Default::default()
        },
    })
}

fn parse_futures_index_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;

    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let index_price = data
        .get("indexPrice")
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures index price: missing indexPrice".into()))?;

    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::MarkPrice {
        symbol,
        mark: crate::core::types::MarkPrice {
            mark_price: index_price,
            index_price: Some(index_price),
            timestamp,
            ..Default::default()
        },
    })
}

fn parse_futures_fair_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::utils::timestamp_millis;

    let data = futures_data(raw)?;
    let symbol = futures_symbol(raw);

    let mark_price = data
        .get("fairPrice")
        .or_else(|| data.get("markPrice"))
        .and_then(parse_f64_field)
        .ok_or_else(|| WebSocketError::Parse("futures fair price: missing fairPrice".into()))?;

    let timestamp = data
        .get("timestamp")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| timestamp_millis() as i64);

    Ok(StreamEvent::MarkPrice {
        symbol,
        mark: crate::core::types::MarkPrice {
            mark_price,
            index_price: None,
            timestamp,
            ..Default::default()
        },
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Kline interval mapping
// ─────────────────────────────────────────────────────────────────────────────

/// Spot kline intervals — internal names (used as KlineInterval keys).
/// Wire format (Min1, Min5, …) is produced by `mexc_spot_kline_interval`.
const MEXC_SPOT_KLINE_INTERVALS: &[&str] = &[
    "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "1w", "1M",
];

/// Map internal KlineInterval → MEXC spot wire format (Min1, Min5, etc.).
///
/// MEXC spot WS channel format: `spot@public.kline.v3.api.pb@BTCUSDT@Min1`
fn mexc_spot_kline_interval(interval: &KlineInterval) -> String {
    match interval.as_str() {
        "1m"  => "Min1",
        "5m"  => "Min5",
        "15m" => "Min15",
        "30m" => "Min30",
        "1h"  => "Min60",
        "4h"  => "Hour4",
        "8h"  => "Hour8",
        "1d"  => "Day1",
        "1w"  => "Week1",
        "1M"  => "Month1",
        other => other,
    }
    .to_string()
}

/// Map internal KlineInterval → MEXC futures wire format (integer minutes or "D").
fn mexc_futures_kline_interval(interval: &KlineInterval) -> &'static str {
    match interval.as_str() {
        "1m"  => "1",
        "5m"  => "5",
        "15m" => "15",
        "30m" => "30",
        "1h"  => "60",
        "4h"  => "240",
        "1d"  => "1440",
        other => {
            // best-effort passthrough — futures API may reject unknown intervals
            let _ = other;
            "1"
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Minimal protobuf helpers (mirrors MexcParser internals, standalone)
// ─────────────────────────────────────────────────────────────────────────────

fn decode_varint(data: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        if pos >= data.len() {
            return None;
        }
        let b = data[pos];
        pos += 1;
        result |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    Some((result, pos))
}

fn pb_string(data: &[u8], target_field: u32) -> Option<String> {
    let mut pos = 0;
    while pos < data.len() {
        let (tag, new_pos) = decode_varint(data, pos)?;
        pos = new_pos;
        let field_num = (tag >> 3) as u32;
        let wire_type = (tag & 0x07) as u8;

        match wire_type {
            0 => {
                let (_, new_pos) = decode_varint(data, pos)?;
                pos = new_pos;
            }
            2 => {
                let (len, new_pos) = decode_varint(data, pos)?;
                pos = new_pos;
                let end = pos + len as usize;
                if end > data.len() {
                    return None;
                }
                if field_num == target_field {
                    return String::from_utf8(data[pos..end].to_vec()).ok();
                }
                pos = end;
            }
            1 => {
                pos += 8;
            }
            5 => {
                pos += 4;
            }
            _ => return None,
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

    fn spot_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTCUSDT".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    fn futures_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTC_USDT".to_string()),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = MexcProtocol::new(AccountType::Spot);

        let spot_reg = proto.topic_registry(AccountType::Spot);
        let spot_keys: Vec<_> = spot_reg.native_pairs().collect();
        assert!(!spot_keys.is_empty(), "spot registry must have entries");
        assert!(spot_reg.supports(&StreamKind::Ticker, AccountType::Spot));
        assert!(spot_reg.supports(&StreamKind::Trade, AccountType::Spot));
        assert!(spot_reg.supports(&StreamKind::Orderbook, AccountType::Spot));

        let fut_reg = proto.topic_registry(AccountType::FuturesCross);
        let fut_keys: Vec<_> = fut_reg.native_pairs().collect();
        assert!(!fut_keys.is_empty(), "futures registry must have entries");
        assert!(fut_reg.supports(&StreamKind::Ticker, AccountType::FuturesCross));
        assert!(fut_reg.supports(&StreamKind::FundingRate, AccountType::FuturesCross));
        assert!(fut_reg.supports(&StreamKind::MarkPrice, AccountType::FuturesCross));
    }

    #[test]
    fn test_subscribe_frame_spot_kline() {
        let proto = MexcProtocol::new(AccountType::Spot);
        let spec = spot_spec(StreamKind::Kline {
            interval: KlineInterval::new("1m"),
        });
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["method"], "SUBSCRIPTION");
        let params = v["params"].as_array().expect("params array");
        assert!(!params.is_empty());
        let param0 = params[0].as_str().expect("string param");
        assert!(param0.contains("kline"), "kline channel: {}", param0);
        assert!(param0.contains("BTCUSDT"), "symbol in channel: {}", param0);
    }

    #[test]
    fn test_subscribe_frame_futures_ticker() {
        let proto = MexcProtocol::new(AccountType::FuturesCross);
        let spec = futures_spec(StreamKind::Ticker);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["method"], "sub.ticker");
        let sym = v["param"]["symbol"].as_str().expect("symbol");
        assert_eq!(sym, "BTC_USDT");
    }

    #[test]
    fn test_extract_topic_spot_deals() {
        let proto = MexcProtocol::new(AccountType::Spot);
        let frame = serde_json::json!({
            "c": "spot@public.aggre.deals.v3.api.pb@100ms@BTCUSDT",
            "d": {},
            "s": "BTCUSDT"
        });
        let topic = proto.extract_topic(&frame).expect("topic must be extracted");
        assert_eq!(topic.as_str(), "spot@public.aggre.deals.v3.api.pb@100ms@BTCUSDT");
    }

    #[test]
    fn test_extract_topic_futures_ticker() {
        let proto = MexcProtocol::new(AccountType::FuturesCross);
        let frame = serde_json::json!({
            "channel": "push.ticker",
            "data": {},
            "symbol": "BTC_USDT"
        });
        let topic = proto.extract_topic(&frame).expect("topic must be extracted");
        assert_eq!(topic.as_str(), "push.ticker");
    }

    #[test]
    fn test_extract_topic_pong_returns_none() {
        let proto = MexcProtocol::new(AccountType::Spot);

        // Spot pong: {"id":0,"code":0,"msg":"PONG"}
        let spot_pong = serde_json::json!({"id": 0, "code": 0, "msg": "PONG"});
        // is_pong returns true, extract_topic is not called for pongs by transport.
        // But if called, "msg" field → no "c" or "channel" → returns None.
        assert!(proto.extract_topic(&spot_pong).is_none());

        // Futures pong: {"channel":"pong","data":1234567890}
        let fut_pong = serde_json::json!({"channel": "pong", "data": 1234567890_i64});
        assert!(proto.extract_topic(&fut_pong).is_none());
    }

    #[test]
    fn test_is_pong_spot() {
        let proto = MexcProtocol::new(AccountType::Spot);
        let frame = serde_json::json!({"id": 0, "code": 0, "msg": "PONG"});
        assert!(proto.is_pong(&frame));
    }

    #[test]
    fn test_is_pong_futures() {
        let proto = MexcProtocol::new(AccountType::FuturesCross);
        let frame = serde_json::json!({"channel": "pong", "data": 1234567890_i64});
        assert!(proto.is_pong(&frame));
    }
}
