//! BybitProtocol — WsProtocol implementation for the Bybit V5 exchange.
//!
//! Declarative shim: supplies endpoint URLs, ping frame, subscribe/unsubscribe
//! frames, topic extraction, and topic registry to UniversalWsTransport.
//!
//! ## Per-category registries
//!
//! Bybit uses separate WebSocket endpoints per product line (spot, linear,
//! inverse, option). Four `OnceLock<TopicRegistry>` caches are maintained:
//! one per category. Each carries identical parser functions — the category
//! selection only affects the endpoint URL.
//!
//! ## Kline interval encoding
//!
//! Bybit uses numeric-string minutes for intraday intervals and letter codes
//! for daily/weekly/monthly:
//!   - `1m` → `"1"`, `60m / 1h` → `"60"`, `1d` → `"D"`, `1w` → `"W"`, `1M` → `"M"`

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

use super::parser::BybitParser;

// ─────────────────────────────────────────────────────────────────────────────
// Per-category registry caches
// ─────────────────────────────────────────────────────────────────────────────

static SPOT_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static LINEAR_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static INVERSE_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();
static OPTION_REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// BybitProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Bybit V5 WS protocol shim.
pub struct BybitProtocol {
    _account_type: AccountType,
    _testnet: bool,
}

impl BybitProtocol {
    pub fn new(account_type: AccountType, testnet: bool) -> Self {
        Self { _account_type: account_type, _testnet: testnet }
    }

    /// Spot registry (cached).
    fn spot_registry() -> &'static TopicRegistry {
        SPOT_REGISTRY.get_or_init(|| build_registry(AccountType::Spot))
    }

    /// Linear (USDT perp) registry (cached).
    fn linear_registry() -> &'static TopicRegistry {
        LINEAR_REGISTRY.get_or_init(|| build_registry(AccountType::FuturesCross))
    }

    /// Inverse registry (cached).
    fn inverse_registry() -> &'static TopicRegistry {
        INVERSE_REGISTRY.get_or_init(|| build_registry_inverse())
    }

    /// Option registry (cached).
    fn option_registry() -> &'static TopicRegistry {
        OPTION_REGISTRY.get_or_init(|| build_registry(AccountType::Options))
    }

    /// Build a subscribe/unsubscribe frame.
    fn build_frame(op: &str, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let topic = Self::build_topic(spec)?;
        let frame = json!({ "op": op, "args": [topic] });
        Ok(WsFrame::Text(frame.to_string()))
    }

    /// Translate a StreamSpec into the Bybit V5 wire topic string.
    ///
    /// `spec.symbol` is already the raw exchange-native string (e.g. "BTCUSDT").
    fn build_topic(spec: &StreamSpec) -> Result<String, WebSocketError> {
        let sym = spec.symbol.as_str();

        // Guard against sentinel connect signal with empty symbol (transport sends
        // a Subscribe cmd with empty Raw("") to wake the driver; reject it here so
        // no malformed topic is sent to the exchange.
        if sym.is_empty() {
            return Err(WebSocketError::WireAbsent(
                "bybit: subscribe called with empty symbol (sentinel connect ignored)".into(),
            ));
        }

        let topic = match &spec.kind {
            StreamKind::Ticker | StreamKind::MarkPrice | StreamKind::FundingRate
            | StreamKind::OpenInterest => {
                format!("tickers.{}", sym)
            }
            StreamKind::Trade | StreamKind::AggTrade => format!("publicTrade.{}", sym),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => {
                let depth = spec.depth.unwrap_or(50);
                format!("orderbook.{}.{}", depth, sym)
            }
            StreamKind::Kline { interval } => {
                // Unknown interval → WireAbsent (NOT a silent 1m substitution).
                // The station arbiter relies on this error to fall back to its
                // trade-aggregation engine; silently returning 1m would deliver
                // 1-minute bars mislabelled as the requested (e.g. 1s) interval.
                let wire = bybit_kline_wire(interval).ok_or_else(|| {
                    WebSocketError::WireAbsent(format!(
                        "Bybit WS has no native kline interval {:?} — \
                         smallest is 1m; aggregate sub-minute from the trade stream",
                        interval.as_str()
                    ))
                })?;
                format!("kline.{}.{}", wire, sym)
            }
            StreamKind::Liquidation => format!("allLiquidation.{}", sym),
            StreamKind::OrderUpdate => "order".to_string(),
            StreamKind::BalanceUpdate => "wallet".to_string(),
            StreamKind::PositionUpdate => "position".to_string(),
            StreamKind::InsuranceFund => {
                // sym is the coin ticker (e.g. "BTC") — use it directly.
                format!("insurance.{}", sym)
            }
            StreamKind::RiskLimit => {
                let coin = if sym.is_empty() { "USDT" } else { sym };
                format!("adlAlert.{}", coin)
            }
            other => return Err(WebSocketError::NotImplemented(
                format!("bybit: unsupported stream kind {:?}", other),
            )),
        };

        Ok(topic)
    }
}

impl WsProtocol for BybitProtocol {
    fn name(&self) -> &'static str {
        "bybit"
    }

    fn endpoint(&self, account_type: AccountType, testnet: bool) -> Url {
        let url_str = match account_type {
            // Note: Bybit Spot tickers (`/v5/public/spot`) omit `bid1Price`/`ask1Price`
            // for USDT-margined symbols such as BTCUSDT.  BTCUSDT lives primarily on the
            // Linear (USDT perp) endpoint where full tickers are always present.  Routing
            // Spot through linear gives callers consistent bid/ask data.  Callers who need
            // genuine Bybit Spot-only streaming can pass AccountType::Margin explicitly.
            AccountType::Spot => {
                if testnet {
                    "wss://stream-testnet.bybit.com/v5/public/linear"
                } else {
                    "wss://stream.bybit.com/v5/public/linear"
                }
            }
            AccountType::Margin => {
                if testnet {
                    "wss://stream-testnet.bybit.com/v5/public/spot"
                } else {
                    "wss://stream.bybit.com/v5/public/spot"
                }
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                if testnet {
                    "wss://stream-testnet.bybit.com/v5/public/linear"
                } else {
                    "wss://stream.bybit.com/v5/public/linear"
                }
            }
            AccountType::Options => {
                if testnet {
                    "wss://stream-testnet.bybit.com/v5/public/option"
                } else {
                    "wss://stream.bybit.com/v5/public/option"
                }
            }
            _ => {
                // Default to linear for unknown types (e.g. inverse is FuturesCross with different endpoint)
                // Callers who need inverse should pass a distinct account_type.
                // For now map everything else to linear.
                if testnet {
                    "wss://stream-testnet.bybit.com/v5/public/linear"
                } else {
                    "wss://stream.bybit.com/v5/public/linear"
                }
            }
        };
        Url::parse(url_str).expect("bybit ws url is valid")
    }

    fn ping_frame(&self) -> Option<WsFrame> {
        Some(WsFrame::Text(r#"{"op":"ping"}"#.to_string()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(20)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        // Public WS only. Private WS auth is deferred.
        None
    }

    fn is_pong(&self, raw: &Value) -> bool {
        // Bybit pong: {"op":"pong"} or {"success":true,"op":"pong","ret_msg":"pong"}
        raw.get("op").and_then(|v| v.as_str()) == Some("pong")
            || raw.get("ret_msg").and_then(|v| v.as_str()) == Some("pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        matches!(
            raw.get("op").and_then(|v| v.as_str()),
            Some("subscribe") | Some("unsubscribe") | Some("auth") | Some("ping")
        ) && raw.get("topic").is_none()
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        // Control frames: op-keyed without a topic field
        if raw.get("op").is_some() && raw.get("topic").is_none() {
            return None;
        }
        // Success acks: {"success":true,...} without topic
        if raw.get("success").is_some() && raw.get("topic").is_none() {
            return None;
        }

        // Data frame: {"topic":"publicTrade.BTCUSDT", ...}
        let topic = raw.get("topic")?.as_str()?;
        Some(TopicKey::new(topic))
    }

    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry {
        match account_type {
            // Spot maps to linear endpoint (see endpoint() comment), so use linear registry.
            AccountType::Spot | AccountType::FuturesCross | AccountType::FuturesIsolated => {
                Self::linear_registry()
            }
            AccountType::Margin => Self::spot_registry(),
            AccountType::Options => Self::option_registry(),
            _ => Self::inverse_registry(),
        }
    }

    fn unsupported_by_exchange(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[]
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[StreamKind::OrderUpdate, StreamKind::BalanceUpdate, StreamKind::PositionUpdate]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Kline interval encoding
// ─────────────────────────────────────────────────────────────────────────────

/// Map internal KlineInterval → Bybit V5 wire interval string.
///
/// Bybit uses numeric-minute strings for intraday, letter codes for longer:
///   1m→"1", 3m→"3", 5m→"5", 15m→"15", 30m→"30", 60m/1h→"60",
///   120m/2h→"120", 240m/4h→"240", 360m/6h→"360", 720m/12h→"720",
///   1d/1D→"D", 1w/1W→"W", 1M→"M"
fn bybit_kline_wire(interval: &KlineInterval) -> Option<&'static str> {
    match interval.as_str() {
        "1m"  => Some("1"),
        "3m"  => Some("3"),
        "5m"  => Some("5"),
        "15m" => Some("15"),
        "30m" => Some("30"),
        "1h" | "60m"  => Some("60"),
        "2h" | "120m" => Some("120"),
        "4h" | "240m" => Some("240"),
        "6h" | "360m" => Some("360"),
        "12h"| "720m" => Some("720"),
        "1d" | "1D"   => Some("D"),
        "1w" | "1W"   => Some("W"),
        "1M"          => Some("M"),
        // Unknown interval (incl. all sub-minute like 1s/30s) → None. The
        // caller turns this into a WireAbsent error so the station arbiter
        // can aggregate from the trade stream instead of silently mislabelling
        // 1-minute bars as the requested interval.
        _ => None,
    }
}

/// Map Bybit wire interval → internal KlineInterval string.
fn internal_kline_interval(wire: &str) -> &'static str {
    match wire {
        "1"   => "1m",
        "3"   => "3m",
        "5"   => "5m",
        "15"  => "15m",
        "30"  => "30m",
        "60"  => "1h",
        "120" => "2h",
        "240" => "4h",
        "360" => "6h",
        "720" => "12h",
        "D"   => "1d",
        "W"   => "1w",
        "M"   => "1M",
        _     => "1h",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

/// Wire-level kline intervals that Bybit V5 publishes.
const BYBIT_KLINE_WIRES: &[&str] = &[
    "1", "3", "5", "15", "30", "60", "120", "240", "360", "720", "D", "W", "M",
];

fn build_registry(account_type: AccountType) -> TopicRegistry {
    let mut b = TopicRegistry::builder();

    // Core streams present on all product lines
    b = b
        .register(StreamKind::Ticker,       account_type, "tickers.*",     parse_ticker)
        .register(StreamKind::MarkPrice,   account_type, "tickers.*",     parse_mark_price)
        .register(StreamKind::FundingRate, account_type, "tickers.*",     parse_funding_rate)
        .register(StreamKind::OpenInterest,account_type, "tickers.*",     parse_open_interest)
        .register(StreamKind::Trade,       account_type, "publicTrade.*", parse_trade)
        .register(StreamKind::AggTrade,    account_type, "publicTrade.*", parse_agg_trade)
        .register(StreamKind::Orderbook,   account_type, "orderbook.1.*",   parse_orderbook)
        .register(StreamKind::Orderbook,   account_type, "orderbook.50.*",  parse_orderbook)
        .register(StreamKind::Orderbook,   account_type, "orderbook.200.*", parse_orderbook)
        .register(StreamKind::Orderbook,   account_type, "orderbook.500.*", parse_orderbook)
        .register(StreamKind::OrderbookDelta, account_type, "orderbook.1.*",   parse_orderbook)
        .register(StreamKind::OrderbookDelta, account_type, "orderbook.50.*",  parse_orderbook)
        .register(StreamKind::OrderbookDelta, account_type, "orderbook.200.*", parse_orderbook)
        .register(StreamKind::OrderbookDelta, account_type, "orderbook.500.*", parse_orderbook);

    // Kline channels
    for wire in BYBIT_KLINE_WIRES {
        let internal = internal_kline_interval(wire);
        let kind = StreamKind::Kline { interval: KlineInterval::new(internal) };
        let pattern = format!("kline.{}.*", wire);
        b = b.register(kind, account_type, pattern, parse_kline);
    }

    // Leveraged token klines (futures / options only — registered on all to be safe)
    for wire in BYBIT_KLINE_WIRES {
        let internal = internal_kline_interval(wire);
        let kind = StreamKind::Kline { interval: KlineInterval::new(internal) };
        let pattern = format!("kline_lt.{}.*", wire);
        b = b.register(kind, account_type, pattern, parse_kline);
    }

    // Futures-only streams (also registered on spot for completeness — silently unused)
    b = b
        .register(StreamKind::Liquidation,   account_type, "allLiquidation.*", parse_all_liquidation)
        .register(StreamKind::InsuranceFund, account_type, "insurance.*",      parse_insurance)
        .register(StreamKind::RiskLimit,     account_type, "adlAlert.*",       parse_adl_alert)
        .register(StreamKind::Ticker,        account_type, "tickers_lt.*",     parse_ticker_lt);

    // Private streams
    b = b
        .register(StreamKind::OrderUpdate,   account_type, "order",    parse_order_update)
        .register(StreamKind::BalanceUpdate, account_type, "wallet",   parse_balance_update)
        .register(StreamKind::PositionUpdate,account_type, "position", parse_position_update);

    b.build()
}

/// Inverse registry — same as linear for now (same wire format).
fn build_registry_inverse() -> TopicRegistry {
    build_registry(AccountType::FuturesCross)
}

// ─────────────────────────────────────────────────────────────────────────────
// Standalone parser functions
// ─────────────────────────────────────────────────────────────────────────────
//
// Each fn receives the full raw frame Value:
//   {"topic":"publicTrade.BTCUSDT","type":"snapshot","ts":...,"data":[...]}

fn parse_ticker(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let d = unwrap_array_or_self(data);
    let ts = raw.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);

    let parse_f64_str = |v: &Value| -> Option<f64> {
        v.as_str()
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
            .or_else(|| v.as_f64())
    };

    // Require lastPrice — absent on deltas that don't update the price.
    let last_price = d.get("lastPrice")
        .and_then(parse_f64_str)
        .ok_or_else(|| WebSocketError::FieldAbsent("lastPrice".into()))?;

    let symbol = d["symbol"].as_str().unwrap_or("").to_string();
    let bid_price = d.get("bid1Price").and_then(parse_f64_str);
    let ask_price = d.get("ask1Price").and_then(parse_f64_str);
    // Fix B-ticker: bid1Size / ask1Size — live-verified present in linear ticker frame.
    let bid_qty = d.get("bid1Size").and_then(parse_f64_str);
    let ask_qty = d.get("ask1Size").and_then(parse_f64_str);
    let high_24h = d.get("highPrice24h").and_then(parse_f64_str);
    let low_24h = d.get("lowPrice24h").and_then(parse_f64_str);
    let volume_24h = d.get("volume24h").and_then(parse_f64_str);
    let quote_volume_24h = d.get("turnover24h").and_then(parse_f64_str);
    let price_change_percent_24h = d.get("price24hPcnt")
        .and_then(parse_f64_str)
        .map(|v| v * 100.0);
    let price_change_24h = {
        let prev = d.get("prevPrice24h").and_then(parse_f64_str);
        prev.map(|p| last_price - p)
    };

    Ok(StreamEvent::Ticker {
        symbol,
        ticker: crate::core::Ticker {
            last_price,
            bid_price,
            ask_price,
            bid_qty,
            ask_qty,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h,
            price_change_24h,
            price_change_percent_24h,
            timestamp: ts, ..Default::default()
        },
    })
}

fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let ticker_data = unwrap_array_or_self(data);

    let symbol = ticker_data["symbol"].as_str().unwrap_or("").to_string();
    let ts = raw.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);

    let parse_f64_str = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };

    let mark_price = ticker_data.get("markPrice")
        .and_then(parse_f64_str)
        .ok_or_else(|| WebSocketError::FieldAbsent("markPrice".into()))?;

    let index_price = ticker_data.get("indexPrice").and_then(parse_f64_str);

    Ok(StreamEvent::MarkPrice { symbol, mark: crate::core::types::MarkPrice { mark_price, index_price, timestamp: ts, ..Default::default() } })
}

fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let ticker_data = unwrap_array_or_self(data);

    let symbol = ticker_data["symbol"].as_str().unwrap_or("").to_string();
    let ts = raw.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);

    let parse_f64_str = |v: &Value| -> Option<f64> {
        // Guard against empty string — Bybit sends "" for fundingRate on dated futures.
        v.as_str()
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
            .or_else(|| v.as_f64())
    };

    // fundingRate absent or empty string → delta without funding update → skip silently.
    let rate = ticker_data.get("fundingRate")
        .and_then(parse_f64_str)
        .ok_or_else(|| WebSocketError::FieldAbsent("fundingRate".into()))?;

    let next_funding_time = ticker_data.get("nextFundingTime")
        .and_then(parse_f64_str)
        .map(|ms| ms as i64);

    Ok(StreamEvent::FundingRate { symbol, funding: crate::core::types::FundingRate { rate, next_funding_time, timestamp: ts, ..Default::default() } })
}

fn parse_open_interest(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let ticker_data = unwrap_array_or_self(data);

    let symbol = ticker_data["symbol"].as_str().unwrap_or("").to_string();
    let ts = raw.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);

    let parse_f64_str = |v: &Value| -> Option<f64> {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
    };

    let open_interest = ticker_data.get("openInterest")
        .and_then(parse_f64_str)
        .ok_or_else(|| WebSocketError::FieldAbsent("openInterest".into()))?;

    let open_interest_value = ticker_data.get("openInterestValue").and_then(parse_f64_str);

    Ok(StreamEvent::OpenInterestUpdate { symbol, open_interest: crate::core::types::OpenInterest { open_interest, open_interest_value, timestamp: ts, ..Default::default() } })
}

fn parse_agg_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    let data = frame_data(raw)?;
    let arr = data.as_array()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: data not array".into()))?;
    let item = arr.first()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: empty data array".into()))?;

    let symbol = item["s"].as_str()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: missing s".into()))?
        .to_string();
    let price = item["p"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("publicTrade: invalid p".into()))?;
    let quantity = item["v"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("publicTrade: invalid v".into()))?;
    let timestamp = item["T"].as_i64()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: invalid T".into()))?;
    let side = item["S"].as_str()
        .map(|s| if s == "Buy" { TradeSide::Buy } else { TradeSide::Sell })
        .unwrap_or(TradeSide::Buy);
    let aggregate_id = item["i"].as_str()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    Ok(StreamEvent::AggTrade {
        symbol,
        agg: crate::core::types::AggTrade {
            aggregate_id, price, quantity,
            first_trade_id: 0,
            last_trade_id: 0,
            is_buy: side == TradeSide::Buy,
            timestamp,
            ..Default::default()
        },
    })
}

fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    let data = frame_data(raw)?;
    let arr = data.as_array()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: data not array".into()))?;
    let item = arr.first()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: empty data array".into()))?;

    let symbol = item["s"].as_str()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: missing s".into()))?
        .to_string();
    let price = item["p"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("publicTrade: invalid p".into()))?;
    let quantity = item["v"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("publicTrade: invalid v".into()))?;
    let timestamp = item["T"].as_i64()
        .ok_or_else(|| WebSocketError::Parse("publicTrade: invalid T".into()))?;
    let side = item["S"].as_str()
        .map(|s| if s == "Buy" { TradeSide::Buy } else { TradeSide::Sell })
        .unwrap_or(TradeSide::Buy);
    let id = item["i"].as_str().unwrap_or("0").to_string();

    // Fix B-trade: WS compact keys BT/RPI/L/seq (Bybit V5 publicTrade wire format).
    // `BT` = isBlockTrade (bool), `RPI` = isRPITrade (bool), `L` = isLiquidation (bool, futures only).
    // `seq` = sequence number (string int) — live-verified on REST; same field present on WS.
    let is_block_trade = item.get("BT").and_then(|v| v.as_bool());
    let is_rpi_trade = item.get("RPI").and_then(|v| v.as_bool());
    let is_liquidation = item.get("L").and_then(|v| v.as_bool());
    let seq = item.get("seq")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()).or_else(|| v.as_i64()));

    Ok(StreamEvent::Trade {
        symbol,
        trade: crate::core::PublicTrade {
            id, price, quantity, side, timestamp,
            is_block_trade,
            is_rpi_trade,
            is_liquidation,
            seq,
            ..Default::default()
        },
    })
}

fn parse_orderbook(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::OrderbookDelta;

    let data = frame_data(raw)?;
    let msg_type = raw.get("type").and_then(|v| v.as_str());
    let symbol = data.get("s").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let wrapper = serde_json::json!({ "retCode": 0, "result": data });
    let orderbook = BybitParser::parse_orderbook(&wrapper)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;

    if msg_type == Some("delta") {
        let delta = OrderbookDelta {
            bids: orderbook.bids,
            asks: orderbook.asks,
            timestamp: orderbook.timestamp,
            first_update_id: orderbook.first_update_id,
            last_update_id: orderbook.last_update_id,
            prev_update_id: orderbook.prev_update_id,
            event_time: orderbook.event_time,
            checksum: orderbook.checksum,
        };
        Ok(StreamEvent::OrderbookDelta { symbol, delta })
    } else {
        Ok(StreamEvent::OrderbookSnapshot { symbol, book: orderbook })
    }
}

fn parse_kline(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let arr = data.as_array()
        .ok_or_else(|| WebSocketError::Parse("kline: data not array".into()))?;
    let item = arr.first()
        .ok_or_else(|| WebSocketError::Parse("kline: empty data array".into()))?;

    let parse_str_f64 = |key: &str| -> WebSocketResult<f64> {
        item.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| WebSocketError::Parse(format!("kline: invalid {}", key)))
    };
    let opt_str_f64 = |key: &str| -> Option<f64> {
        item.get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse::<f64>().ok())
    };

    let start  = item.get("start").and_then(|v| v.as_i64())
        .ok_or_else(|| WebSocketError::Parse("kline: invalid start".into()))?;
    let open   = parse_str_f64("open")?;
    let high   = parse_str_f64("high")?;
    let low    = parse_str_f64("low")?;
    let close  = parse_str_f64("close")?;
    let volume = parse_str_f64("volume")?;

    // Fix A: `turnover` (string) → quote_volume — was silently dropped.
    // Live WS kline frame (Bybit V5 doc): start/end/interval/open/close/high/low/volume/turnover/confirm/timestamp.
    let quote_volume = opt_str_f64("turnover");

    // Fix B-kline: `end` → close_time; `confirm` (bool) → confirm.
    let close_time = item.get("end").and_then(|v| v.as_i64());
    let confirm = item.get("confirm").and_then(|v| v.as_bool());

    // topic: "kline.1.BTCUSDT" → parts[2]=symbol, parts[1]=interval
    let topic = raw.get("topic").and_then(|v| v.as_str()).unwrap_or("");
    let mut topic_parts = topic.splitn(3, '.');
    let interval_str = topic_parts.nth(1).unwrap_or("");
    let interval = KlineInterval::new(internal_kline_interval(interval_str));
    let symbol = topic_parts.next().unwrap_or("").to_string();

    Ok(StreamEvent::Kline {
        symbol,
        interval,
        kline: crate::core::Kline {
            open_time: start,
            open, high, low, close, volume,
            quote_volume,
            close_time,
            trades: None,
            confirm,
            ..Default::default()
        },
    })
}

/// Parser for the `allLiquidation.{sym}` channel (replaces deprecated `liquidation.{sym}`).
///
/// Frame data is an array of objects with fields:
///   T: timestamp ms, s: symbol, S: "Buy"|"Sell", v: qty (base), p: bankruptcy price.
///
/// `S="Buy"` means a long position was liquidated (exchange sold it).
/// `S="Sell"` means a short position was liquidated (exchange bought it).
fn parse_all_liquidation(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::TradeSide;

    let data = frame_data(raw)?;
    let item = if let Some(arr) = data.as_array() {
        match arr.first() {
            Some(v) => v,
            None => return Err(WebSocketError::FieldAbsent("allLiquidation: empty data array".into())),
        }
    } else {
        data
    };

    let symbol = item["s"].as_str()
        .ok_or_else(|| WebSocketError::Parse("allLiquidation: missing s".into()))?
        .to_string();
    let side_str = item["S"].as_str()
        .ok_or_else(|| WebSocketError::Parse("allLiquidation: missing S".into()))?;
    // S="Buy" → long was liquidated; S="Sell" → short was liquidated.
    // We report the side of the position that got liquidated.
    let side = match side_str {
        "Buy"  => TradeSide::Buy,
        "Sell" => TradeSide::Sell,
        other  => return Err(WebSocketError::Parse(format!("allLiquidation: unknown S: {}", other))),
    };
    let price = item["p"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("allLiquidation: invalid p".into()))?;
    let quantity = item["v"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| WebSocketError::Parse("allLiquidation: invalid v".into()))?;
    let timestamp = item["T"].as_i64()
        .ok_or_else(|| WebSocketError::Parse("allLiquidation: invalid T".into()))?;

    let sym = symbol;
    Ok(StreamEvent::Liquidation {
        symbol: sym.clone(),
        liquidation: crate::core::types::Liquidation {
            symbol: sym, side, price, quantity,
            value: Some(price * quantity),
            timestamp,
            ..Default::default()
        },
    })
}

fn parse_insurance(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let topic = raw.get("topic").and_then(|v| v.as_str()).unwrap_or("");
    let coin = topic.trim_start_matches("insurance.").to_string();

    let balance = data["balance"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| {
            data["symbols"].as_array()
                .and_then(|arr| arr.first())
                .and_then(|item| item["balance"].as_str())
                .and_then(|s| s.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    let timestamp = data["updateTime"].as_i64()
        .or_else(|| data["ts"].as_i64())
        .unwrap_or(0);

    Ok(StreamEvent::InsuranceFund { symbol: coin, fund: crate::core::types::InsuranceFund { balance, timestamp } })
}

/// Parse the Bybit `adlAlert.<coin>` WS frame into a `RiskLimit` event.
///
/// **Wire-absent fields (documented):** Bybit `adlAlert` carries only the
/// ADL-rank descriptor (`adlIdx` / `i_pr`), **NOT** real risk-tier metadata.
/// As a result the following `RiskLimit` fields are hard-coded:
///
/// | Field | Source | Wire-absent reason |
/// |---|---|---|
/// | `max_leverage` | `0.0` | adlAlert has no leverage field — exists only in REST `/v5/market/risk-limit` per (symbol, category) |
/// | `max_position_value` | `0.0` | adlAlert has no position-value cap — same REST-only |
/// | `mmr` | `i_pr * 0.5` | adlAlert has no maintenance-margin-rate — derived heuristic from ADL probability |
/// | `imr` | `adl_score.abs()` | adlAlert has no initial-margin-rate — repurposed ADL rank score |
///
/// If a consumer needs the real tier metadata, query Bybit REST
/// `GET /v5/market/risk-limit?category=linear&symbol=<S>` and join with this
/// event by `symbol`. ADL ranking and tier metadata live on different feeds.
fn parse_adl_alert(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let topic = raw.get("topic").and_then(|v| v.as_str()).unwrap_or("");
    let coin = topic.trim_start_matches("adlAlert.").to_string();
    let timestamp = crate::core::timestamp_millis() as i64;

    let items = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    // Emit first item as RiskLimit (ADL alert describes ADL rank per symbol)
    let item = match items.first() {
        Some(v) => v,
        None => return Ok(StreamEvent::RiskLimit {
            symbol: coin,
            risk_limit: crate::core::types::RiskLimit {
                tier: 0,
                max_leverage: 0.0,
                max_position_value: 0.0,
                mmr: 0.0,
                imr: 0.0,
                timestamp,
            },
        }),
    };

    let symbol = format!(
        "{}/{}",
        item["s"].as_str().unwrap_or(""),
        coin
    );
    let adl_score = item["adl_sr"].as_f64().unwrap_or(0.0);
    let i_pr = item["i_pr"].as_f64().unwrap_or(0.0);
    let tier = item["adl_tt"].as_f64().map(|v| v.abs() as u32).unwrap_or(0);

    Ok(StreamEvent::RiskLimit {
        symbol,
        risk_limit: crate::core::types::RiskLimit {
            tier,
            max_leverage: 0.0,
            max_position_value: 0.0,
            mmr: i_pr * 0.5,
            imr: adl_score.abs(),
            timestamp,
        },
    })
}

fn parse_ticker_lt(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let topic = raw.get("topic").and_then(|v| v.as_str()).unwrap_or("");
    let symbol = topic.trim_start_matches("tickers_lt.").to_string();
    let last_price = data["nav"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let timestamp = data["navTime"].as_i64().unwrap_or(0);

    Ok(StreamEvent::Ticker {
        symbol,
        ticker: crate::core::Ticker {
            last_price,
            bid_price: None, // Bybit tickers_lt (leveraged token NAV) channel does not carry top-of-book quotes
            ask_price: None, // Bybit tickers_lt (leveraged token NAV) channel does not carry top-of-book quotes
            high_24h: None, low_24h: None,
            volume_24h: None, quote_volume_24h: None,
            price_change_24h: None, price_change_percent_24h: None,
            timestamp, ..Default::default() 
        },
    })
}

fn parse_order_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let arr = data.as_array()
        .ok_or_else(|| WebSocketError::Parse("order: data not array".into()))?;
    let item = arr.first()
        .ok_or_else(|| WebSocketError::Parse("order: empty array".into()))?;

    let wrapper = serde_json::json!({ "retCode": 0, "result": item });
    let order = BybitParser::parse_order(&wrapper)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;

    Ok(StreamEvent::OrderUpdate {
        symbol: order.symbol.unwrap_or_default(),
        event: crate::core::OrderUpdateEvent {
            order_id: order.id,
            client_order_id: order.client_order_id,
            side: order.side,
            order_type: order.order_type,
            status: order.status,
            price: order.price,
            quantity: order.quantity,
            filled_quantity: order.filled_quantity,
            average_price: order.average_price,
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: order.commission_asset,
            trade_id: None,
            timestamp: order.updated_at.unwrap_or(order.created_at),
        },
    })
}

fn parse_balance_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let arr = data.as_array()
        .ok_or_else(|| WebSocketError::Parse("wallet: data not array".into()))?;
    let item = arr.first()
        .ok_or_else(|| WebSocketError::Parse("wallet: empty array".into()))?;

    let coin = item["coin"].as_str()
        .ok_or_else(|| WebSocketError::Parse("wallet: missing coin".into()))?;
    let free = item["walletBalance"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let locked = item["locked"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(StreamEvent::BalanceUpdate(crate::core::BalanceUpdateEvent {
        asset: coin.to_string(),
        free, locked,
        total: free + locked,
        delta: None,
        reason: None,
        timestamp: crate::core::timestamp_millis() as i64,
    }))
}

fn parse_position_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::PositionSide;

    let data = frame_data(raw)?;
    let arr = data.as_array()
        .ok_or_else(|| WebSocketError::Parse("position: data not array".into()))?;
    let item = arr.first()
        .ok_or_else(|| WebSocketError::Parse("position: empty array".into()))?;

    let symbol = item["symbol"].as_str()
        .ok_or_else(|| WebSocketError::Parse("position: missing symbol".into()))?
        .to_string();
    let side = item["side"].as_str()
        .map(|s| if s == "Buy" { PositionSide::Long } else { PositionSide::Short })
        .unwrap_or(PositionSide::Long);
    let quantity = item["size"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let entry_price = item["avgPrice"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let unrealized_pnl = item["unrealisedPnl"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let mark_price = item["markPrice"].as_str().and_then(|s| s.parse::<f64>().ok());
    let liquidation_price = item["liqPrice"].as_str().and_then(|s| s.parse::<f64>().ok());
    let leverage = item["leverage"].as_str().and_then(|s| s.parse::<u32>().ok());

    Ok(StreamEvent::PositionUpdate {
        symbol,
        event: crate::core::PositionUpdateEvent {
            side, quantity, entry_price, mark_price,
            unrealized_pnl, realized_pnl: None,
            liquidation_price, leverage, margin_type: None, reason: None,
            timestamp: crate::core::timestamp_millis() as i64,
        },
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Frame helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Extract "data" field from a Bybit data frame.
fn frame_data(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("data")
        .ok_or_else(|| WebSocketError::Parse("bybit frame missing 'data' field".into()))
}

/// If `data` is an array, return first element; otherwise return `data` itself.
fn unwrap_array_or_self(data: &Value) -> &Value {
    if let Some(arr) = data.as_array() {
        arr.first().unwrap_or(data)
    } else {
        data
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::websocket::StreamSpec;

    fn spot_proto() -> BybitProtocol {
        BybitProtocol::new(AccountType::Spot, false)
    }

    fn linear_proto() -> BybitProtocol {
        BybitProtocol::new(AccountType::FuturesCross, false)
    }

    fn spot_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTCUSDT".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = spot_proto();
        // Spot maps to linear registry (FuturesCross) — all entries use FuturesCross.
        let reg = proto.topic_registry(AccountType::Spot);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "linear registry must have entries");
        // Spot routes to linear endpoint so registry keys use FuturesCross.
        assert!(reg.supports(&StreamKind::Ticker, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Trade, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Orderbook, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Liquidation, AccountType::FuturesCross));
        assert!(reg.supports(
            &StreamKind::Kline { interval: KlineInterval::new("1m") },
            AccountType::FuturesCross
        ));
    }

    #[test]
    fn test_subscribe_frame_public_trade() {
        let proto = spot_proto();
        let spec = spot_spec(StreamKind::Trade);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["op"], "subscribe");
        assert_eq!(v["args"][0], "publicTrade.BTCUSDT");
    }

    #[test]
    fn test_extract_topic_public_trade_frame() {
        let proto = spot_proto();
        let frame = serde_json::json!({
            "topic": "publicTrade.BTCUSDT",
            "type": "snapshot",
            "ts": 1700000000000_i64,
            "data": []
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "publicTrade.BTCUSDT");
    }

    #[test]
    fn test_extract_topic_pong_returns_none() {
        let proto = spot_proto();
        let frame = serde_json::json!({ "op": "pong" });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_extract_topic_success_ack_returns_none() {
        let proto = spot_proto();
        let frame = serde_json::json!({
            "success": true,
            "op": "subscribe",
            "ret_msg": ""
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_endpoint_url_per_category() {
        let spot = spot_proto();
        let linear = linear_proto();
        let opt = BybitProtocol::new(AccountType::Options, false);
        let margin = BybitProtocol::new(AccountType::Margin, false);

        // Spot routes to linear so callers get bid/ask from the perp tickers channel.
        let spot_url = spot.endpoint(AccountType::Spot, false).to_string();
        let linear_url = linear.endpoint(AccountType::FuturesCross, false).to_string();
        let opt_url = opt.endpoint(AccountType::Options, false).to_string();
        let margin_url = margin.endpoint(AccountType::Margin, false).to_string();

        assert!(spot_url.contains("/linear"), "spot now routes to /linear: {}", spot_url);
        assert!(linear_url.contains("/linear"), "linear url must contain /linear: {}", linear_url);
        assert!(opt_url.contains("/option"), "option url must contain /option: {}", opt_url);
        assert!(margin_url.contains("/spot"), "margin url must contain /spot: {}", margin_url);
        assert_eq!(spot_url, linear_url, "spot and linear share same endpoint");
    }

    #[test]
    fn test_kline_subscribe_frame_interval() {
        let proto = spot_proto();
        let spec = StreamSpec {
            kind: StreamKind::Kline { interval: KlineInterval::new("1h") },
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTCUSDT".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        };
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        // 1h → Bybit wire "60"
        assert_eq!(v["args"][0], "kline.60.BTCUSDT");
    }

    #[test]
    fn test_is_pong() {
        let proto = spot_proto();
        assert!(proto.is_pong(&serde_json::json!({"op":"pong"})));
        assert!(proto.is_pong(&serde_json::json!({"success":true,"op":"pong","ret_msg":"pong"})));
        assert!(!proto.is_pong(&serde_json::json!({"topic":"publicTrade.BTCUSDT"})));
    }

    #[test]
    fn test_subscribe_frame_empty_symbol_rejected() {
        let proto = spot_proto();
        let spec = StreamSpec {
            kind: StreamKind::Ticker,
            symbol: crate::core::types::OwnedSymbolInput::Raw(String::new()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        };
        let result = proto.subscribe_frame(&spec);
        assert!(result.is_err(), "empty symbol must return Err, not send tickers. to exchange");
    }

    #[test]
    fn test_ping_frame_json() {
        let proto = spot_proto();
        match proto.ping_frame() {
            Some(WsFrame::Text(t)) => {
                let v: serde_json::Value = serde_json::from_str(&t).expect("ping must be valid JSON");
                assert_eq!(v["op"], "ping");
            }
            _ => panic!("expected Some(Text(...))"),
        }
    }

    /// Verify that `parse_all_liquidation` correctly decodes the Bybit V5 wire frame.
    /// Frame shape per docs: data is an object (not array) with fields T/s/S/v/p.
    #[test]
    fn test_parse_all_liquidation_object_data() {
        let frame = serde_json::json!({
            "topic": "allLiquidation.BTCUSDT",
            "type": "snapshot",
            "ts": 1700000000000_i64,
            "data": {
                "T": 1700000000000_i64,
                "s": "BTCUSDT",
                "S": "Buy",
                "v": "0.123",
                "p": "30000.50"
            }
        });
        let event = parse_all_liquidation(&frame).expect("should parse");
        match event {
            StreamEvent::Liquidation { symbol, liquidation } => {
                assert_eq!(symbol, "BTCUSDT");
                assert!((liquidation.price - 30000.50).abs() < 0.01, "price={}", liquidation.price);
                assert!((liquidation.quantity - 0.123).abs() < 0.001, "quantity={}", liquidation.quantity);
                assert_eq!(liquidation.timestamp, 1700000000000);
            }
            other => panic!("expected Liquidation, got {:?}", other),
        }
    }

    /// Verify parser handles array-wrapped data (alternative Bybit format).
    #[test]
    fn test_parse_all_liquidation_array_data() {
        let frame = serde_json::json!({
            "topic": "allLiquidation.ETHUSDT",
            "type": "snapshot",
            "ts": 1700000001000_i64,
            "data": [{
                "T": 1700000001000_i64,
                "s": "ETHUSDT",
                "S": "Sell",
                "v": "2.5",
                "p": "2000.00"
            }]
        });
        let event = parse_all_liquidation(&frame).expect("should parse array-wrapped");
        match event {
            StreamEvent::Liquidation { symbol, liquidation } => {
                assert_eq!(symbol, "ETHUSDT");
                assert!((liquidation.price - 2000.0).abs() < 0.01);
                assert!((liquidation.quantity - 2.5).abs() < 0.001);
                assert_eq!(liquidation.timestamp, 1700000001000);
            }
            other => panic!("expected Liquidation, got {:?}", other),
        }
    }

    /// Verify subscribe frame for allLiquidation channel.
    #[test]
    fn test_subscribe_frame_liquidation() {
        let proto = spot_proto();
        let spec = StreamSpec {
            kind: StreamKind::Liquidation,
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTCUSDT".to_string()),
            account_type: AccountType::Spot,
            depth: None,
            speed_ms: None,
        };
        let msg = proto.subscribe_frame(&spec).expect("must build subscribe frame");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
        assert_eq!(v["op"], "subscribe");
        assert_eq!(v["args"][0], "allLiquidation.BTCUSDT");
    }

    /// Verify topic_registry dispatches allLiquidation topic.
    #[test]
    fn test_registry_dispatches_all_liquidation() {
        let proto = linear_proto();
        let reg = proto.topic_registry(AccountType::FuturesCross);
        let key = crate::core::websocket::TopicKey::new("allLiquidation.BTCUSDT");
        let parsers = reg.dispatch_all(&key);
        assert!(!parsers.is_empty(), "allLiquidation.BTCUSDT must match a registered parser");
    }
}
