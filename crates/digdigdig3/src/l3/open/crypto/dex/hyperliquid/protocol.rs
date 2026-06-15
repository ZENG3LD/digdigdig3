//! HyperliquidProtocol — WsProtocol implementation for Hyperliquid DEX.
//!
//! Hyperliquid is a perp-only DEX; no spot/futures split at the WS level.
//! One registry covers all account types.
//!
//! Frame shape:
//!   `{"channel":"trades","data":[...]}`
//! Topic = `channel` field.
//!
//! Ping: `{"method":"ping"}` → server replies `{"channel":"pong"}`.

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, OrderSide, OrderStatus, OrderType, StreamEvent,
    WebSocketError, WebSocketResult,
};
use crate::core::OrderUpdateEvent;
use crate::core::websocket::{
    KlineInterval, StreamKind, StreamSpec, TopicKey, TopicRegistry, WsProtocol,
};

use super::parser::HyperliquidParser;

// ─────────────────────────────────────────────────────────────────────────────
// Registry cache — single registry (perp-only, no account type split)
// ─────────────────────────────────────────────────────────────────────────────

static REGISTRY: OnceLock<TopicRegistry> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// HyperliquidProtocol
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative Hyperliquid WS protocol shim.
pub struct HyperliquidProtocol {
    _testnet: bool,
}

impl HyperliquidProtocol {
    pub fn new(testnet: bool) -> Self {
        Self { _testnet: testnet }
    }

    fn registry() -> &'static TopicRegistry {
        REGISTRY.get_or_init(build_registry)
    }

    /// Build subscribe/unsubscribe JSON frame.
    fn build_frame(method: &str, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        let coin = spec.symbol.as_str();
        let subscription = match &spec.kind {
            // Empty coin → allMids (global mid-price snapshot)
            StreamKind::Ticker if coin.is_empty() => json!({ "type": "allMids", "dex": "" }),
            // Per-coin ticker: use bbo channel for real-time bid/ask top-of-book
            StreamKind::Ticker => json!({ "type": "bbo", "coin": coin }),
            StreamKind::Trade => json!({ "type": "trades", "coin": coin }),
            StreamKind::Orderbook | StreamKind::OrderbookDelta => json!({
                "type": "l2Book",
                "coin": coin,
                "nSigFigs": null,
                "mantissa": null,
            }),
            StreamKind::Kline { interval } => json!({
                "type": "candle",
                "coin": coin,
                "interval": interval.as_str(),
            }),
            StreamKind::MarkPrice => json!({ "type": "activeAssetCtx", "coin": coin }),
            StreamKind::FundingRate => json!({ "type": "activeAssetCtx", "coin": coin }),
            StreamKind::OpenInterest => json!({ "type": "activeAssetCtx", "coin": coin }),
            StreamKind::IndexPrice => json!({ "type": "activeAssetCtx", "coin": coin }),
            StreamKind::Liquidation => {
                return Err(WebSocketError::NotSupported(
                    "HyperLiquid liquidations WS feed is user-specific (requires wallet address) — \
                     not available as a public anonymous stream".to_string(),
                ));
            }
            StreamKind::AggTrade => {
                return Err(WebSocketError::NotSupported(
                    "HyperLiquid has no aggregated trade WS channel — \
                     subscribe to StreamKind::Trade for trades per coin".to_string(),
                ));
            }
            StreamKind::BalanceUpdate => json!({ "type": "clearinghouseState", "user": coin }),
            StreamKind::PositionUpdate => json!({ "type": "clearinghouseState", "user": coin }),
            StreamKind::OrderUpdate => json!({ "type": "orderUpdates", "user": coin }),
            StreamKind::MarketWarning => {
                return Err(WebSocketError::NotSupported(
                    "HyperLiquid does not expose a market-warning / notification WS channel — \
                     status updates are delivered out-of-band via Discord / status page".to_string(),
                ));
            }
            other => {
                return Err(WebSocketError::UnsupportedOperation(format!(
                    "hyperliquid: unsupported stream kind {:?}",
                    other
                )))
            }
        };
        let frame = json!({ "method": method, "subscription": subscription });
        Ok(WsFrame::Text(frame.to_string()))
    }
}

impl WsProtocol for HyperliquidProtocol {
    fn name(&self) -> &'static str {
        "hyperliquid"
    }

    fn endpoint(&self, _account_type: AccountType, testnet: bool) -> Url {
        let url = if testnet {
            "wss://api.hyperliquid-testnet.xyz/ws"
        } else {
            "wss://api.hyperliquid.xyz/ws"
        };
        Url::parse(url).expect("hyperliquid ws url is valid")
    }

    fn ping_frame(&self) -> Option<WsFrame> {
        Some(WsFrame::Text(json!({ "method": "ping" }).to_string()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>> {
        None
    }

    fn is_pong(&self, raw: &Value) -> bool {
        raw.get("channel").and_then(|v| v.as_str()) == Some("pong")
    }

    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        // Hyperliquid sends {"channel":"subscriptionResponse",...} as sub ack
        raw.get("channel").and_then(|v| v.as_str()) == Some("subscriptionResponse")
    }

    fn extract_topic(&self, raw: &Value) -> Option<TopicKey> {
        let channel = raw.get("channel").and_then(|v| v.as_str())?;
        // Filter pong + sub acks
        match channel {
            "pong" | "subscriptionResponse" => None,
            other => Some(TopicKey::new(other)),
        }
    }

    fn topic_registry(&self, _account_type: AccountType) -> &TopicRegistry {
        Self::registry()
    }

    fn requires_auth_kinds(&self, _account_type: AccountType) -> &'static [StreamKind] {
        &[
            StreamKind::BalanceUpdate,
            StreamKind::PositionUpdate,
            StreamKind::OrderUpdate,
        ]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_registry() -> TopicRegistry {
    // Hyperliquid is perp-only — use FuturesCross as the canonical AccountType.
    let at = AccountType::FuturesCross;

    let mut b = TopicRegistry::builder()
        // Public market channels
        .register(StreamKind::Ticker, at, "allMids", parse_all_mids)
        .register(StreamKind::Trade, at, "trades", parse_trades)
        .register(StreamKind::Orderbook, at, "l2Book", parse_l2_book)
        .register(StreamKind::OrderbookDelta, at, "l2Book", parse_l2_book)
        .register(StreamKind::FundingRate, at, "activeAssetCtx", parse_funding_from_ctx)
        .register(StreamKind::MarkPrice, at, "activeAssetCtx", parse_mark_price_from_ctx)
        .register(StreamKind::OpenInterest, at, "activeAssetCtx", parse_open_interest_from_ctx)
        .register(StreamKind::IndexPrice, at, "activeAssetCtx", parse_index_price_from_ctx)
        .register(StreamKind::Ticker, at, "activeAssetCtx", parse_ticker_from_ctx)
        // Liquidation: user-specific (requires wallet address) — not a public feed.
        // Removed from registry; subscribe_frame returns NotSupported.
        // Notifications (public)
        .register(StreamKind::MarketWarning, at, "notifications", parse_notification)
        // BBO (best bid/offer) → emits Ticker with bid_price + ask_price
        .register(StreamKind::Ticker, at, "bbo", parse_bbo)
        // User/private channels (auth-gated but we register parsers)
        .register(StreamKind::BalanceUpdate, at, "clearinghouseState", parse_clearinghouse)
        .register(StreamKind::PositionUpdate, at, "clearinghouseState", parse_clearinghouse_position)
        .register(StreamKind::BalanceUpdate, at, "userNonFundingLedgerUpdates", parse_non_funding_ledger)
        .register(StreamKind::OrderUpdate, at, "orderUpdates", parse_order_update)
        .register(StreamKind::BalanceUpdate, at, "userFundings", parse_user_fundings)
        .register(StreamKind::BalanceUpdate, at, "webData2", parse_web_data2);

    // Kline — single "candle" channel regardless of interval
    for interval in HYPERLIQUID_KLINE_INTERVALS {
        b = b.register(
            StreamKind::Kline {
                interval: KlineInterval::new(*interval),
            },
            at,
            "candle",
            parse_candle,
        );
    }

    b.build()
}

/// Kline intervals supported by Hyperliquid candle channel.
const HYPERLIQUID_KLINE_INTERVALS: &[&str] = &[
    "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "3d", "1w",
];

// ─────────────────────────────────────────────────────────────────────────────
// Parser functions (ParserFn = fn(&Value) -> WebSocketResult<StreamEvent>)
//
// Each receives the full frame: {"channel":"...","data":[...]}
// ─────────────────────────────────────────────────────────────────────────────

fn parse_all_mids(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    // { "mids": { "BTC": "50123.45", ... } }
    let mids = data
        .get("mids")
        .and_then(|m| m.as_object())
        .ok_or_else(|| WebSocketError::Parse("allMids: missing 'mids' object".into()))?;

    let now = crate::core::utils::timestamp_millis() as i64;

    // Return the first entry as a Ticker (broadcast channel fans out all coins).
    // In practice callers subscribe allMids at most once; they get N Ticker events
    // via the dispatch loop in the transport — but ParserFn returns one event.
    // Emit the first coin found.
    if let Some((symbol, price_val)) = mids.iter().next() {
        let price = parse_f64_val(price_val)
            .ok_or_else(|| WebSocketError::Parse("allMids: invalid price".into()))?;
        let ticker = crate::core::Ticker {
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: now, ..Default::default() 
        };
        return Ok(StreamEvent::Ticker { symbol: symbol.clone(), ticker });
    }

    Err(WebSocketError::Parse("allMids: empty mids object".into()))
}

fn parse_funding_from_ctx(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let coin = data
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/funding: missing 'coin'".into()))?;
    let ctx = data
        .get("ctx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/funding: missing 'ctx'".into()))?;
    let rate = parse_f64_field(ctx, "funding")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/funding: missing 'funding'".into()))?;
    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::FundingRate {
        symbol: coin.to_string(),
        funding: crate::core::types::FundingRate {
            rate,
            next_funding_time: None,
            timestamp: now,
            ..Default::default()
        },
    })
}

fn parse_mark_price_from_ctx(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let coin = data
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/mark: missing 'coin'".into()))?;
    let ctx = data
        .get("ctx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/mark: missing 'ctx'".into()))?;
    let mark_price = parse_f64_field(ctx, "markPx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/mark: missing 'markPx'".into()))?;
    let index_price = parse_f64_field(ctx, "oraclePx");
    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::MarkPrice {
        symbol: coin.to_string(),
        mark: crate::core::types::MarkPrice {
            mark_price,
            index_price,
            timestamp: now,
            ..Default::default()
        },
    })
}

fn parse_open_interest_from_ctx(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let coin = data
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/oi: missing 'coin'".into()))?;
    let ctx = data
        .get("ctx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/oi: missing 'ctx'".into()))?;
    let open_interest = parse_f64_field(ctx, "openInterest")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/oi: missing 'openInterest'".into()))?;
    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::OpenInterestUpdate {
        symbol: coin.to_string(),
        open_interest: crate::core::types::OpenInterest {
            open_interest,
            open_interest_value: None,
            timestamp: now,
            ..Default::default()
        },
    })
}

fn parse_index_price_from_ctx(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let coin = data
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/idx: missing 'coin'".into()))?;
    let ctx = data
        .get("ctx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/idx: missing 'ctx'".into()))?;
    let price = parse_f64_field(ctx, "oraclePx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/idx: missing 'oraclePx'".into()))?;
    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::IndexPrice {
        symbol: coin.to_string(),
        index_price: crate::core::types::IndexPrice { price, timestamp: now, ..Default::default() },
    })
}

fn parse_ticker_from_ctx(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let coin = data
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/ticker: missing 'coin'".into()))?;
    let ctx = data
        .get("ctx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx/ticker: missing 'ctx'".into()))?;
    let mid_px = parse_f64_field(ctx, "midPx")
        .ok_or_else(|| WebSocketError::FieldAbsent("midPx".into()))?;
    let volume_24h = parse_f64_field(ctx, "dayBaseVlm");
    let quote_volume_24h = parse_f64_field(ctx, "dayNtlVlm");
    let mark_px = parse_f64_field(ctx, "markPx");
    let prev_day_px = parse_f64_field(ctx, "prevDayPx");
    let price_change_24h = match (mark_px, prev_day_px) {
        (Some(mark), Some(prev)) if prev > 0.0 => Some((mark - prev) / prev),
        _ => None,
    };
    let now = crate::core::utils::timestamp_millis() as i64;
    let symbol = coin.to_string();
    let ticker = crate::core::Ticker {
        last_price: mid_px,
        bid_price: None,
        ask_price: None,
        high_24h: None,
        low_24h: None,
        volume_24h,
        quote_volume_24h,
        price_change_24h,
        price_change_percent_24h: price_change_24h.map(|c| c * 100.0),
        timestamp: now, ..Default::default() 
    };
    Ok(StreamEvent::Ticker { symbol, ticker })
}

fn parse_trades(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let trades = data
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("trades: expected array".into()))?;

    let trade_data = trades
        .first()
        .ok_or_else(|| WebSocketError::Parse("trades: empty array".into()))?;

    let parsed = HyperliquidParser::parse_recent_trades(&serde_json::json!([trade_data]))
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;

    let trade = parsed
        .into_iter()
        .next()
        .ok_or_else(|| WebSocketError::Parse("trades: no trade parsed".into()))?;
    let symbol = trade_data.get("coin").and_then(|c| c.as_str()).unwrap_or("").to_string();
    Ok(StreamEvent::Trade { symbol, trade })
}

fn parse_l2_book(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    // HyperLiquid l2Book data carries "coin" field
    let ob_symbol = data.get("coin").and_then(|c| c.as_str()).unwrap_or("").to_string();
    let book = HyperliquidParser::parse_orderbook(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderbookSnapshot { symbol: ob_symbol, book })
}

fn parse_bbo(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    let coin = match data.get("coin").and_then(|c| c.as_str()) {
        Some(s) => s,
        None => return Err(WebSocketError::Parse("bbo: missing 'coin'".into())),
    };

    let bbo_arr = data
        .get("bbo")
        .and_then(|b| b.as_array())
        .ok_or_else(|| WebSocketError::Parse("bbo: missing 'bbo' array".into()))?;

    let bid_price = bbo_arr.first().and_then(|l| l.get("px")).and_then(parse_f64_val);
    let ask_price = bbo_arr.get(1).and_then(|l| l.get("px")).and_then(parse_f64_val);

    // Skip frames where both sides are absent (initial snapshot before data arrives)
    let last_price = match (bid_price, ask_price) {
        (None, None) => {
            return Err(WebSocketError::Parse(
                "bbo: both bid and ask are absent — skipping empty frame".into(),
            ))
        }
        (Some(b), Some(a)) => (b + a) / 2.0,
        (Some(b), None) => b,
        (None, Some(a)) => a,
    };

    let now = crate::core::utils::timestamp_millis() as i64;
    let symbol = coin.to_string();
    let ticker = crate::core::Ticker {
        last_price,
        bid_price,
        ask_price,
        high_24h: None,
        low_24h: None,
        volume_24h: None,
        quote_volume_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: now, ..Default::default() 
    };
    Ok(StreamEvent::Ticker { symbol, ticker })
}

fn parse_candle(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    // HyperLiquid candle data carries "coin" and "interval" fields.
    // The WS `candle` channel delivers a SINGLE candle object in `data`
    // (fields t/T/s/i/o/c/h/l/v/n), whereas REST `candleSnapshot` returns an
    // ARRAY. `parse_klines` only accepts an array, so a bare object made
    // `as_array()` return None → every live candle was dropped (the chart's
    // countdown then froze on the last REST bar). Wrap a single object in a
    // one-element array before handing it to the shared parser.
    // The single candle object uses "s" (symbol) and "i" (interval) — the
    // same field names parse_klines reads. ("coin"/"interval" were the old
    // assumed names; they never matched the real candle object, but it didn't
    // show because the event was dropped before reaching here.) Fall back to
    // the array element when data is already an array (REST-shaped frame).
    let candle_obj = if data.is_array() {
        data.get(0).unwrap_or(data)
    } else {
        data
    };
    let kl_symbol = candle_obj.get("s").and_then(|c| c.as_str())
        .or_else(|| data.get("coin").and_then(|c| c.as_str()))
        .unwrap_or("")
        .to_string();
    let kl_interval = KlineInterval::new(
        candle_obj.get("i").and_then(|i| i.as_str())
            .or_else(|| data.get("interval").and_then(|i| i.as_str()))
            .unwrap_or(""),
    );
    let as_array = if data.is_array() {
        std::borrow::Cow::Borrowed(data)
    } else {
        std::borrow::Cow::Owned(Value::Array(vec![data.clone()]))
    };
    let klines = HyperliquidParser::parse_klines(&as_array)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    let kline = klines
        .into_iter()
        .next()
        .ok_or_else(|| WebSocketError::Parse("candle: no kline parsed".into()))?;
    Ok(StreamEvent::Kline { symbol: kl_symbol, interval: kl_interval, kline })
}

fn parse_clearinghouse(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let now = crate::core::utils::timestamp_millis() as i64;

    let summary = data
        .get("marginSummary")
        .ok_or_else(|| WebSocketError::Parse("clearinghouseState: missing 'marginSummary'".into()))?;

    let account_value = parse_f64_field(summary, "accountValue").unwrap_or(0.0);
    let margin_used = parse_f64_field(summary, "totalMarginUsed").unwrap_or(0.0);

    Ok(StreamEvent::BalanceUpdate(crate::core::BalanceUpdateEvent {
        asset: "USDC".to_string(),
        free: (account_value - margin_used).max(0.0),
        locked: margin_used,
        total: account_value,
        delta: None,
        reason: None,
        timestamp: now,
    }))
}

fn parse_clearinghouse_position(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let now = crate::core::utils::timestamp_millis() as i64;

    let positions = data
        .get("assetPositions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| WebSocketError::Parse("clearinghouseState/pos: missing 'assetPositions'".into()))?;

    // Return first non-zero position
    for entry in positions {
        let pos = entry.get("position").unwrap_or(entry);
        let coin = match pos.get("coin").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };
        let size_str = pos.get("szi").and_then(|v| v.as_str()).unwrap_or("0");
        let size: f64 = size_str.parse().unwrap_or(0.0);
        if size == 0.0 {
            continue;
        }

        let entry_price = parse_f64_field(pos, "entryPx").unwrap_or(0.0);
        let unrealized_pnl = parse_f64_field(pos, "unrealizedPnl").unwrap_or(0.0);
        let side = if size > 0.0 {
            crate::core::PositionSide::Long
        } else {
            crate::core::PositionSide::Short
        };

        return Ok(StreamEvent::PositionUpdate {
            symbol: coin.to_string(),
            event: crate::core::PositionUpdateEvent {
                side,
                quantity: size.abs(),
                entry_price,
                mark_price: None,
                unrealized_pnl,
                realized_pnl: None,
                liquidation_price: parse_f64_field(pos, "liquidationPx"),
                leverage: pos
                    .get("leverage")
                    .and_then(|v| v.get("value"))
                    .and_then(|v| v.as_f64())
                    .map(|v| v as u32),
                margin_type: None,
                reason: None,
                timestamp: now,
            },
        });
    }

    Err(WebSocketError::Parse(
        "clearinghouseState/pos: no non-zero position found".into(),
    ))
}

fn parse_non_funding_ledger(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let entries = data
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("userNonFundingLedgerUpdates: expected array".into()))?;

    let entry = entries
        .first()
        .ok_or_else(|| WebSocketError::Parse("userNonFundingLedgerUpdates: empty array".into()))?;

    let timestamp = entry
        .get("time")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| crate::core::utils::timestamp_millis() as i64);

    let delta_obj = entry
        .get("delta")
        .ok_or_else(|| WebSocketError::Parse("userNonFundingLedgerUpdates: missing 'delta'".into()))?;

    let delta_type = delta_obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let amount = parse_f64_field(delta_obj, "usdc")
        .or_else(|| parse_f64_field(delta_obj, "usdcValue"))
        .unwrap_or(0.0);

    let reason = Some(match delta_type {
        "deposit" => crate::core::BalanceChangeReason::Deposit,
        "withdrawal" | "withdraw" => crate::core::BalanceChangeReason::Withdraw,
        _ => crate::core::BalanceChangeReason::Other,
    });

    Ok(StreamEvent::BalanceUpdate(crate::core::BalanceUpdateEvent {
        asset: "USDC".to_string(),
        free: amount,
        locked: 0.0,
        total: amount,
        delta: Some(amount),
        reason,
        timestamp,
    }))
}

fn parse_user_fundings(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let fundings = data
        .get("fundings")
        .and_then(|f| f.as_array())
        .ok_or_else(|| WebSocketError::Parse("userFundings: missing 'fundings' array".into()))?;

    let entry = fundings
        .first()
        .ok_or_else(|| WebSocketError::Parse("userFundings: empty fundings array".into()))?;

    let coin = entry
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("userFundings: missing 'coin'".into()))?;

    let rate = parse_f64_field(entry, "fundingRate")
        .or_else(|| parse_f64_field(entry, "funding"))
        .ok_or_else(|| WebSocketError::Parse("userFundings: missing rate".into()))?;

    let timestamp = entry
        .get("time")
        .and_then(|t| t.as_i64())
        .unwrap_or_else(|| crate::core::utils::timestamp_millis() as i64);

    Ok(StreamEvent::FundingRate {
        symbol: coin.to_string(),
        funding: crate::core::types::FundingRate {
            rate,
            next_funding_time: None,
            timestamp,
            ..Default::default()
        },
    })
}

fn parse_web_data2(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    // Extract mids sub-object and emit a Ticker for the first coin
    let mids = data
        .get("mids")
        .and_then(|m| m.as_object())
        .ok_or_else(|| WebSocketError::Parse("webData2: missing 'mids'".into()))?;

    let now = crate::core::utils::timestamp_millis() as i64;
    if let Some((symbol, price_val)) = mids.iter().next() {
        let price = parse_f64_val(price_val)
            .ok_or_else(|| WebSocketError::Parse("webData2: invalid price".into()))?;
        let ticker = crate::core::Ticker {
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: now, ..Default::default() 
        };
        return Ok(StreamEvent::Ticker { symbol: symbol.clone(), ticker });
    }
    Err(WebSocketError::Parse("webData2: no mids entry".into()))
}

fn parse_order_update(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    // orderUpdates data is an array of order status objects
    let orders = data
        .as_array()
        .ok_or_else(|| WebSocketError::Parse("orderUpdates: expected array".into()))?;

    let order_obj = orders
        .first()
        .ok_or_else(|| WebSocketError::Parse("orderUpdates: empty array".into()))?;

    let inner = order_obj
        .get("order")
        .ok_or_else(|| WebSocketError::Parse("orderUpdates: missing 'order' field".into()))?;

    let symbol = inner
        .get("coin")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let order_id = inner
        .get("oid")
        .and_then(|v| v.as_u64())
        .map(|id| id.to_string())
        .unwrap_or_default();

    let client_order_id = inner
        .get("cloid")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty() && *s != "null")
        .map(|s| s.to_string());

    let side = match inner.get("side").and_then(|v| v.as_str()) {
        Some("B") => OrderSide::Buy,
        _ => OrderSide::Sell,
    };

    let price = inner
        .get("limitPx")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|&p| p > 0.0);

    let quantity = inner
        .get("sz")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let orig_sz = inner
        .get("origSz")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    // filled = origSz - remaining sz
    let filled_quantity = (orig_sz - quantity).max(0.0);

    let status_str = order_obj
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("open");

    let status = match status_str {
        "open" | "triggered" => OrderStatus::Open,
        "filled" => OrderStatus::Filled,
        "canceled" | "marginCanceled" => OrderStatus::Canceled,
        "rejected" => OrderStatus::Rejected,
        _ => OrderStatus::Open,
    };

    let timestamp = order_obj
        .get("statusTimestamp")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| crate::core::utils::timestamp_millis() as i64);

    Ok(StreamEvent::OrderUpdate {
        symbol,
        event: OrderUpdateEvent {
            order_id,
            client_order_id,
            side,
            order_type: OrderType::Limit { price: price.unwrap_or(0.0) },
            status,
            price,
            quantity: orig_sz,
            filled_quantity,
            average_price: None,
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: None,
            trade_id: None,
            timestamp,
        },
    })
}

fn parse_notification(raw: &Value) -> WebSocketResult<StreamEvent> {
    // notifications channel: {"channel":"notifications","data":{"notification":"..."}}
    // Map to MarketWarning
    let data = frame_data(raw)?;
    let msg = data
        .get("notification")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::MarketWarning {
        symbol: None,
        warning: crate::core::types::MarketWarning {
            symbol: String::new(),
            warning_kind: "notification".to_string(),
            message: msg,
            timestamp: now,
        },
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Frame helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Extract `data` field from a Hyperliquid frame.
fn frame_data(raw: &Value) -> WebSocketResult<&Value> {
    raw.get("data")
        .ok_or_else(|| WebSocketError::Parse("hyperliquid frame missing 'data' field".into()))
}

/// Parse f64 from a Value (string or number).
fn parse_f64_val(val: &Value) -> Option<f64> {
    val.as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| val.as_f64())
}

/// Parse f64 from a named field in a Value.
fn parse_f64_field(obj: &Value, key: &str) -> Option<f64> {
    obj.get(key).and_then(parse_f64_val)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::websocket::StreamSpec;

    fn futures_spec(kind: StreamKind) -> StreamSpec {
        StreamSpec {
            kind,
            symbol: crate::core::types::OwnedSymbolInput::Raw("BTC".to_string()),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        }
    }

    #[test]
    fn test_topic_registry_non_empty() {
        let proto = HyperliquidProtocol::new(false);
        let reg = proto.topic_registry(AccountType::FuturesCross);
        let keys: Vec<_> = reg.native_pairs().collect();
        assert!(!keys.is_empty(), "registry must have entries");
        assert!(reg.supports(&StreamKind::Trade, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Orderbook, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::Ticker, AccountType::FuturesCross));
        assert!(reg.supports(&StreamKind::FundingRate, AccountType::FuturesCross));
        // Liquidation is user-specific (requires wallet) — removed from public registry.
        assert!(!reg.supports(&StreamKind::Liquidation, AccountType::FuturesCross));
    }

    #[test]
    fn test_subscribe_frame_trades_uses_coin() {
        let proto = HyperliquidProtocol::new(false);
        let spec = futures_spec(StreamKind::Trade);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["method"], "subscribe");
        let sub = &v["subscription"];
        assert_eq!(sub["type"], "trades");
        assert_eq!(sub["coin"], "BTC");
    }

    #[test]
    fn test_extract_topic_trades_frame() {
        let proto = HyperliquidProtocol::new(false);
        let frame = serde_json::json!({
            "channel": "trades",
            "data": [{"coin":"BTC","side":"B","px":"50000","sz":"0.1","time":1000,"tid":1}]
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "trades");
    }

    #[test]
    fn test_extract_topic_pong_returns_none() {
        let proto = HyperliquidProtocol::new(false);
        let frame = serde_json::json!({ "channel": "pong" });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_extract_topic_subscribe_ack_returns_none() {
        let proto = HyperliquidProtocol::new(false);
        let frame = serde_json::json!({
            "channel": "subscriptionResponse",
            "data": { "method": "subscribe" }
        });
        assert!(proto.extract_topic(&frame).is_none());
    }

    #[test]
    fn test_subscribe_frame_ticker_uses_bbo() {
        let proto = HyperliquidProtocol::new(false);
        let spec = futures_spec(StreamKind::Ticker);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(v["method"], "subscribe");
        let sub = &v["subscription"];
        assert_eq!(sub["type"], "bbo");
        assert_eq!(sub["coin"], "BTC");
    }

    #[test]
    fn test_parse_bbo_emits_ticker_with_bid_ask() {
        let frame = serde_json::json!({
            "channel": "bbo",
            "data": {
                "coin": "BTC",
                "time": 1716100000000i64,
                "bbo": [
                    {"px": "67100.0", "sz": "0.45", "n": 3},
                    {"px": "67110.0", "sz": "0.30", "n": 2}
                ]
            }
        });
        let event = parse_bbo(&frame).expect("parse_bbo must succeed");
        match event {
            crate::core::types::StreamEvent::Ticker { ticker: t, symbol, .. } => {
                assert_eq!(symbol, "BTC");
                assert!((t.bid_price.unwrap() - 67100.0).abs() < f64::EPSILON);
                assert!((t.ask_price.unwrap() - 67110.0).abs() < f64::EPSILON);
                assert!((t.last_price - 67105.0).abs() < f64::EPSILON);
            }
            other => panic!("expected Ticker, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_bbo_empty_returns_error() {
        // Initial frame may arrive with null/missing bid and ask — must be skipped
        let frame = serde_json::json!({
            "channel": "bbo",
            "data": {
                "coin": "BTC",
                "time": 1716100000000i64,
                "bbo": [null, null]
            }
        });
        let result = parse_bbo(&frame);
        assert!(result.is_err(), "empty bbo frame must return error, not zero ticker");
    }

    #[test]
    fn test_ticker_subscribe_frame_uses_bbo() {
        // subscribe_frame for Ticker must request bbo (bid/ask channel).
        // activeAssetCtx is also registered as a Ticker fan-out (24h stats),
        // but the subscribe frame still routes to bbo.
        let proto = HyperliquidProtocol::new(false);
        let reg = proto.topic_registry(AccountType::FuturesCross);
        assert!(reg.supports(&StreamKind::Ticker, AccountType::FuturesCross));
        let spec = futures_spec(StreamKind::Ticker);
        let proto2 = HyperliquidProtocol::new(false);
        let msg = proto2.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            WsFrame::Text(t) => t,
            _ => panic!("expected text frame"),
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        let sub = &v["subscription"];
        assert_eq!(sub["type"], "bbo", "Ticker subscribe must use bbo");
    }

    #[test]
    fn test_parse_ticker_from_ctx_emits_24h_stats() {
        let frame = serde_json::json!({
            "channel": "activeAssetCtx",
            "data": {
                "coin": "ETH",
                "ctx": {
                    "midPx": "3200.0",
                    "markPx": "3205.0",
                    "prevDayPx": "3100.0",
                    "dayBaseVlm": "12500.5",
                    "dayNtlVlm": "40000000.0",
                    "funding": "0.0001",
                    "openInterest": "50000.0",
                    "oraclePx": "3201.0"
                }
            }
        });
        let event = parse_ticker_from_ctx(&frame).expect("parse_ticker_from_ctx must succeed");
        match event {
            crate::core::types::StreamEvent::Ticker { symbol, ticker: t, .. } => {
                assert_eq!(symbol, "ETH");
                assert!((t.last_price - 3200.0).abs() < f64::EPSILON);
                assert!(t.bid_price.is_none());
                assert!(t.ask_price.is_none());
                assert!((t.volume_24h.unwrap() - 12500.5).abs() < f64::EPSILON);
                assert!((t.quote_volume_24h.unwrap() - 40000000.0).abs() < 0.01);
                // price_change_24h = (3205 - 3100) / 3100 ≈ 0.033871
                let pct = t.price_change_24h.unwrap();
                assert!((pct - (3205.0 - 3100.0) / 3100.0).abs() < 1e-9);
            }
            other => panic!("expected Ticker, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ticker_from_ctx_missing_midpx_returns_error() {
        let frame = serde_json::json!({
            "channel": "activeAssetCtx",
            "data": {
                "coin": "BTC",
                "ctx": {
                    "markPx": "67000.0",
                    "prevDayPx": "65000.0"
                }
            }
        });
        let result = parse_ticker_from_ctx(&frame);
        assert!(result.is_err(), "missing midPx must return error");
    }

    #[test]
    fn test_extract_topic_bbo_frame() {
        let proto = HyperliquidProtocol::new(false);
        let frame = serde_json::json!({
            "channel": "bbo",
            "data": {"coin":"BTC","time":1716100000000i64,"bbo":[{"px":"67100.0","sz":"0.45","n":3},{"px":"67110.0","sz":"0.30","n":2}]}
        });
        let topic = proto.extract_topic(&frame).expect("should extract topic");
        assert_eq!(topic.as_str(), "bbo");
    }

    #[test]
    fn test_parse_candle_single_object_frame() {
        // Regression: the WS `candle` channel delivers a SINGLE candle object
        // in `data` (not the REST array). parse_candle must wrap it before
        // parse_klines — a bare object previously made parse_klines bail
        // ("Expected array of candles") so no live candle ever reached the
        // chart, freezing the bar countdown at 00:00.
        let frame = serde_json::json!({
            "channel": "candle",
            "data": {
                "t": 1704067200000i64,
                "T": 1704074400000i64,
                "s": "MUUSDC",
                "i": "2h",
                "o": "980.0",
                "c": "985.4",
                "h": "990.0",
                "l": "975.0",
                "v": "123.45",
                "n": 42
            }
        });
        let ev = parse_candle(&frame).expect("single-object candle must parse");
        match ev {
            StreamEvent::Kline { symbol, interval, kline } => {
                assert_eq!(symbol, "MUUSDC");
                assert_eq!(interval.as_str(), "2h");
                assert_eq!(kline.open_time, 1704067200000);
                assert_eq!(kline.close, 985.4);
                assert_eq!(kline.high, 990.0);
            }
            _ => panic!("expected Kline event"),
        }
    }
}
