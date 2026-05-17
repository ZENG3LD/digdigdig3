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
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, StreamEvent, TradeSide, WebSocketError, WebSocketResult,
};
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
    fn build_frame(method: &str, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        let coin = spec.symbol.as_str();
        let subscription = match &spec.kind {
            // Empty coin → allMids (global mid-price snapshot)
            StreamKind::Ticker if coin.is_empty() => json!({ "type": "allMids", "dex": "" }),
            StreamKind::Ticker => json!({ "type": "activeAssetCtx", "coin": coin }),
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
            StreamKind::Liquidation => json!({ "type": "liquidations" }),
            StreamKind::BalanceUpdate => json!({ "type": "clearinghouseState", "user": coin }),
            StreamKind::PositionUpdate => json!({ "type": "clearinghouseState", "user": coin }),
            StreamKind::OrderUpdate => json!({ "type": "orderUpdates", "user": coin }),
            other => {
                return Err(WebSocketError::UnsupportedOperation(format!(
                    "hyperliquid: unsupported stream kind {:?}",
                    other
                )))
            }
        };
        let frame = json!({ "method": method, "subscription": subscription });
        Ok(Message::Text(frame.to_string()))
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

    fn ping_frame(&self) -> Option<Message> {
        Some(Message::Text(json!({ "method": "ping" }).to_string()))
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("subscribe", spec)
    }

    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<Message, WebSocketError> {
        Self::build_frame("unsubscribe", spec)
    }

    fn auth_frame(&self, _credentials: &Credentials) -> Option<Result<Message, WebSocketError>> {
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
        .register(StreamKind::Ticker, at, "activeAssetCtx", parse_active_asset_ctx)
        .register(StreamKind::Trade, at, "trades", parse_trades)
        .register(StreamKind::Orderbook, at, "l2Book", parse_l2_book)
        .register(StreamKind::OrderbookDelta, at, "l2Book", parse_l2_book)
        .register(StreamKind::FundingRate, at, "activeAssetCtx", parse_funding_from_ctx)
        .register(StreamKind::MarkPrice, at, "activeAssetCtx", parse_mark_price_from_ctx)
        .register(StreamKind::OpenInterest, at, "activeAssetCtx", parse_open_interest_from_ctx)
        .register(StreamKind::IndexPrice, at, "activeAssetCtx", parse_index_price_from_ctx)
        .register(StreamKind::Liquidation, at, "liquidations", parse_liquidation)
        // Notifications (public)
        .register(StreamKind::MarketWarning, at, "notifications", parse_notification)
        // BBO (best bid/offer)
        .register(StreamKind::Orderbook, at, "bbo", parse_bbo)
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
        return Ok(StreamEvent::Ticker(crate::core::Ticker {
            symbol: symbol.clone(),
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: now,
        }));
    }

    Err(WebSocketError::Parse("allMids: empty mids object".into()))
}

fn parse_active_asset_ctx(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    parse_active_asset_ctx_inner(data)
}

fn parse_active_asset_ctx_inner(data: &Value) -> WebSocketResult<StreamEvent> {
    let coin = data
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx: missing 'coin'".into()))?;

    let ctx = data
        .get("ctx")
        .ok_or_else(|| WebSocketError::Parse("activeAssetCtx: missing 'ctx'".into()))?;

    let mark_px = parse_f64_field(ctx, "markPx").unwrap_or(0.0);
    let mid_px = parse_f64_field(ctx, "midPx");
    let prev_day_px = parse_f64_field(ctx, "prevDayPx");
    let volume_24h = parse_f64_field(ctx, "dayNtlVlm");

    let last_price = mid_px.unwrap_or(mark_px);
    let now = crate::core::utils::timestamp_millis() as i64;

    let (price_change_24h, price_change_percent_24h) = match prev_day_px {
        Some(prev) if prev > 0.0 => {
            let change = last_price - prev;
            (Some(change), Some((change / prev) * 100.0))
        }
        _ => (None, None),
    };

    Ok(StreamEvent::Ticker(crate::core::Ticker {
        symbol: coin.to_string(),
        last_price,
        bid_price: None,
        ask_price: None,
        high_24h: None,
        low_24h: None,
        volume_24h,
        quote_volume_24h: None,
        price_change_24h,
        price_change_percent_24h,
        timestamp: now,
    }))
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
        rate,
        next_funding_time: None,
        timestamp: now,
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
        mark_price,
        index_price,
        timestamp: now,
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
        open_interest,
        open_interest_value: None,
        timestamp: now,
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
        price,
        timestamp: now,
    })
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

    parsed
        .into_iter()
        .next()
        .map(StreamEvent::Trade)
        .ok_or_else(|| WebSocketError::Parse("trades: no trade parsed".into()))
}

fn parse_l2_book(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let ob = HyperliquidParser::parse_orderbook(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    Ok(StreamEvent::OrderbookSnapshot(ob))
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
    let last_price = bid_price
        .zip(ask_price)
        .map(|(b, a)| (b + a) / 2.0)
        .or(bid_price)
        .or(ask_price)
        .unwrap_or(0.0);

    let now = crate::core::utils::timestamp_millis() as i64;
    Ok(StreamEvent::Ticker(crate::core::Ticker {
        symbol: coin.to_string(),
        last_price,
        bid_price,
        ask_price,
        high_24h: None,
        low_24h: None,
        volume_24h: None,
        quote_volume_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: now,
    }))
}

fn parse_candle(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;
    let klines = HyperliquidParser::parse_klines(data)
        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
    klines
        .into_iter()
        .next()
        .map(StreamEvent::Kline)
        .ok_or_else(|| WebSocketError::Parse("candle: no kline parsed".into()))
}

fn parse_liquidation(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    let symbol = data
        .get("coin")
        .and_then(|c| c.as_str())
        .ok_or_else(|| WebSocketError::Parse("liquidation: missing 'coin'".into()))?
        .to_string();

    let side_str = data
        .get("side")
        .and_then(|s| s.as_str())
        .unwrap_or("A");

    let price = parse_f64_field(data, "px")
        .ok_or_else(|| WebSocketError::Parse("liquidation: missing 'px'".into()))?;

    let quantity = parse_f64_field(data, "sz").unwrap_or(0.0);

    let timestamp = data
        .get("time")
        .or_else(|| data.get("ts"))
        .and_then(|t| t.as_i64())
        .unwrap_or(0);

    // "B"/"Buy" = buy-side forced order → short liquidated
    // "A"/"Sell" = sell-side forced order → long liquidated
    let side = match side_str {
        "B" | "Buy" => TradeSide::Sell,
        _ => TradeSide::Buy,
    };

    Ok(StreamEvent::Liquidation {
        symbol,
        side,
        price,
        quantity,
        value: Some(price * quantity),
        timestamp,
    })
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

        return Ok(StreamEvent::PositionUpdate(crate::core::PositionUpdateEvent {
            symbol: coin.to_string(),
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
        }));
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
        rate,
        next_funding_time: None,
        timestamp,
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
        return Ok(StreamEvent::Ticker(crate::core::Ticker {
            symbol: symbol.clone(),
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: now,
        }));
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

    // Extract basic fields
    let symbol = order_obj
        .get("order")
        .and_then(|o| o.get("coin"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let now = crate::core::utils::timestamp_millis() as i64;

    // Emit as a BalanceUpdate placeholder — full OrderUpdate parsing would require
    // mapping HL order status strings to OrderUpdateEvent; defer to connector layer.
    // Use BalanceUpdate with zero delta as a notification sentinel.
    Ok(StreamEvent::BalanceUpdate(crate::core::BalanceUpdateEvent {
        asset: symbol,
        free: 0.0,
        locked: 0.0,
        total: 0.0,
        delta: None,
        reason: None,
        timestamp: now,
    }))
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
        symbol: String::new(),
        warning_kind: "notification".to_string(),
        message: msg,
        timestamp: now,
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
            symbol: "BTC".to_string(),
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
        assert!(reg.supports(&StreamKind::Liquidation, AccountType::FuturesCross));
    }

    #[test]
    fn test_subscribe_frame_trades_uses_coin() {
        let proto = HyperliquidProtocol::new(false);
        let spec = futures_spec(StreamKind::Trade);
        let msg = proto.subscribe_frame(&spec).expect("subscribe_frame must succeed");
        let text = match msg {
            Message::Text(t) => t,
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
}
