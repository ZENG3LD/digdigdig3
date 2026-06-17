//! REST backfill for warm-start.
//!
//! When the disk store has fewer than `warm` records, the forwarder calls one
//! of these helpers to pull recent history from the exchange REST API. Each
//! helper returns oldest→newest, already deduped against `existing_count`
//! (so the caller can simply emit them to broadcast).
//!
//! Supported data-classes and their REST sources:
//! - `trades_recent`            — `get_recent_trades`
//! - `agg_trades_recent`        — `get_agg_trades`
//! - `klines_recent`            — `get_klines`
//! - `open_interest_recent`     — `get_open_interest_history`
//! - `mark_price_recent`        — `get_premium_index` (current snapshot)
//! - `funding_rate_recent`      — `get_funding_rate_history`
//! - `liquidations_recent`      — `get_liquidation_history`
//! - `insurance_fund_recent`    — `get_insurance_fund` (current snapshot)
//! - `mark_price_klines_recent` — `get_mark_price_klines`
//! - `index_price_klines_recent`   — `get_index_price_klines`
//! - `premium_index_klines_recent` — `get_premium_index_klines`
//!
//! All helpers return empty vec on any error or unsupported endpoint.

use std::sync::Arc;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, AggTrade, ExchangeId, FundingRate, InsuranceFund, Liquidation, MarkPrice, OpenInterest, SymbolInput, TradeSide};
use digdigdig3::core::websocket::KlineInterval;

use crate::data::{
    AggTradePoint, BarPoint, FundingRatePoint, FundingRateIndicatorsPoint, FundingRateFullPoint,
    IndexPriceKlinePoint, InsuranceFundPoint,
    LiquidationPoint, LiquidationIndicatorsPoint, LiquidationFullPoint,
    MarkPriceKlinePoint, MarkPricePoint, MarkPriceIndicatorsPoint, MarkPriceFullPoint,
    OpenInterestPoint, OpenInterestIndicatorsPoint,
    PremiumIndexKlinePoint, TickerIndicatorsPoint, TickerFullPoint, TradePoint,
};
use crate::error::{Result, StationError};

/// Pull up to `limit` recent trades from REST for (exchange, account, symbol).
/// Returns oldest→newest. Empty vec on any error or unsupported.
pub async fn trades_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<TradePoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_recent_trades(SymbolInput::Raw(symbol), Some(limit), account)
        .await;
    match res {
        Ok(trades) => trades.iter().map(TradePoint::from_public).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill trades failed");
            Vec::new()
        }
    }
}

/// Fetch up to `limit` historical bars ending at `end_time_ms` (exclusive)
/// for the given series. Used by chart UIs for scroll-left pagination past
/// the warm-start window.
///
/// Bypasses Station's persisted Series (pure REST through the shared
/// `ExchangeHub`). Caller decides what to do with the result — typically
/// merge into a local cache and re-render.
///
/// `end_time_ms` is exclusive: bars with `open_time >= end_time_ms` are
/// excluded (matches the existing dig3-core `get_klines` semantic).
///
/// `symbol` must be in raw exchange-native form — no `SymbolNormalizer`
/// is applied internally. The caller is responsible for normalization,
/// matching the `SubscriptionSet::add_raw` convention.
///
/// Returns the raw `BarPoint` Vec sorted oldest-first.
/// Returns `Ok(Vec::new())` if REST returns zero bars.
/// Returns `Err(StationError::Core(...))` if `exchange` is not connected
/// in `hub`.
pub async fn fetch_history(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    symbol: &str,
    account_type: AccountType,
    interval: &KlineInterval,
    end_time_ms: i64,
    limit: u16,
) -> Result<Vec<BarPoint>> {
    let rest = hub
        .rest(exchange)
        .ok_or_else(|| StationError::Core(format!("{exchange:?} not connected in hub")))?;
    let limit = limit.min(1000).max(1);
    let bars = rest
        .get_klines(
            SymbolInput::Raw(symbol),
            interval.as_str(),
            Some(limit),
            account_type,
            Some(end_time_ms),
        )
        .await
        .map_err(|e| StationError::Core(format!("get_klines failed: {e}")))?;
    let mut points: Vec<BarPoint> = bars.iter().map(BarPoint::from_kline).collect();
    points.sort_unstable_by_key(|p| p.open_time);
    Ok(points)
}

/// Pull up to `limit` klines (interval = `interval`) from REST for
/// (exchange, account, symbol). Returns oldest→newest. Empty on any error.
pub async fn klines_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    limit: usize,
) -> Vec<BarPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u16;
    let res = rest
        .get_klines(
            SymbolInput::Raw(symbol),
            interval,
            Some(limit),
            account,
            None,
        )
        .await;
    match res {
        Ok(bars) => bars.iter().map(BarPoint::from_kline).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, interval, "rest backfill klines failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` aggregated trades from REST for (exchange, account, symbol).
/// Returns oldest→newest. Empty on any error or unsupported endpoint.
pub async fn agg_trades_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<AggTradePoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_agg_trades(SymbolInput::Raw(symbol), Some(limit), None, account)
        .await;
    match res {
        Ok(trades) => trades.iter().map(agg_trade_point_from).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill agg_trades failed");
            Vec::new()
        }
    }
}

/// Paginated aggTrade backfill for deep warm-seeding of derived bars
/// (range/volume/tick/footprint). Walks `from_id` backwards `n_pages`
/// times to gather a real warm window — a single page (1000) covers
/// only ~1-5 seconds of BTC liquidity, which is not enough to derive
/// even one closed range bar at typical parameters.
///
/// Algorithm (Binance aggTrades / MEXC spot):
/// 1. First page: request `limit` trades, no cursor → newest 1000.
/// 2. For each subsequent page: `from_id = min_id_of_prev_page - limit`
///    → returns 1000 strictly earlier aggregates.
/// 3. Stop when the venue returns < limit results (history exhausted)
///    or when n_pages have been collected.
///
/// Dedupes by `aggregate_id` and returns oldest→newest. On exchanges
/// without paginated aggTrades support (`NotImplemented` from
/// `get_agg_trades`), returns the result of `trades_recent` (single
/// page recent trades) as a graceful fallback.
pub async fn agg_trades_paginated(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    page_size: usize,
    n_pages: usize,
) -> Vec<AggTradePoint> {
    use std::collections::BTreeMap;
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    if n_pages == 0 || page_size == 0 {
        return Vec::new();
    }
    let limit = page_size.min(1000).max(1) as u32;

    let mut by_id: BTreeMap<u64, AggTradePoint> = BTreeMap::new();
    let mut next_from_id: Option<u64> = None;
    let mut pages_done = 0usize;
    let mut agg_fallback = false;

    while pages_done < n_pages {
        let res = rest
            .get_agg_trades(SymbolInput::Raw(symbol), Some(limit), next_from_id, account)
            .await;
        let page: Vec<AggTradePoint> = match res {
            Ok(trades) => trades.iter().map(agg_trade_point_from).collect(),
            Err(e) => {
                if pages_done == 0 {
                    // First page failed → venue lacks aggTrades; fall back.
                    tracing::debug!(
                        ?e,
                        exchange = ?exchange,
                        "agg_trades_paginated first page failed; falling back to trades_recent"
                    );
                    agg_fallback = true;
                } else {
                    // Subsequent page failed → keep what we have.
                    tracing::debug!(
                        ?e,
                        exchange = ?exchange,
                        page = pages_done,
                        "agg_trades_paginated page failed; stopping at partial result"
                    );
                }
                break;
            }
        };
        if page.is_empty() {
            break;
        }
        // Find the earliest aggregate_id on this page (cursor for next).
        let min_id = page.iter().map(|p| p.agg_id).min().unwrap_or(0);
        let was_partial = page.len() < limit as usize;
        for p in page {
            by_id.entry(p.agg_id).or_insert(p);
        }
        pages_done += 1;
        if was_partial {
            // Venue exhausted its history window — no point in another call.
            break;
        }
        // Next page: trades strictly earlier than min_id.
        if min_id <= limit as u64 {
            // Reached the beginning of the venue history.
            break;
        }
        next_from_id = Some(min_id.saturating_sub(limit as u64));
    }

    if agg_fallback {
        // aggTrades not implemented for this venue. Single-shot
        // trades_recent through SymbolInput is the best we can do
        // without per-venue REST paths; wrap as AggTradePoint with
        // synthetic agg_ids (the bar derivation does not depend on
        // agg_id, only ts/price/qty/side).
        return trades_recent(hub, exchange, account, symbol, limit as usize)
            .await
            .into_iter()
            .enumerate()
            .map(|(i, t)| AggTradePoint {
                ts_ms: t.ts_ms,
                price: t.price,
                quantity: t.quantity,
                side: t.side,
                agg_id: i as u64,
            })
            .collect();
    }

    by_id.into_values().collect()
}

fn agg_trade_point_from(t: &AggTrade) -> AggTradePoint {
    let side = if t.is_buy { 0u8 } else { 1u8 };
    AggTradePoint {
        ts_ms: t.timestamp,
        price: t.price,
        quantity: t.quantity,
        side,
        agg_id: t.aggregate_id as u64,
    }
}

/// Pull up to `limit` open-interest history snapshots from REST for
/// (exchange, account, symbol). Returns oldest→newest. Empty on any error.
pub async fn open_interest_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<OpenInterestPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_open_interest_history(
            SymbolInput::Raw(symbol),
            "5m",
            None,
            None,
            Some(limit),
            account,
        )
        .await;
    match res {
        Ok(items) => items.iter().map(oi_point_from).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill open_interest failed");
            Vec::new()
        }
    }
}

fn oi_point_from(oi: &OpenInterest) -> OpenInterestPoint {
    OpenInterestPoint {
        ts_ms: oi.timestamp,
        open_interest: oi.open_interest,
        open_interest_value: oi.open_interest_value.unwrap_or(f64::NAN),
    }
}

/// Fetch the current mark-price snapshot from REST for (exchange, account, symbol).
/// Returns 0 or 1 element (single snapshot, not history). Empty on any error.
pub async fn mark_price_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    _limit: usize,
) -> Vec<MarkPricePoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let res = rest
        .get_premium_index(Some(SymbolInput::Raw(symbol)), account)
        .await;
    match res {
        Ok(items) => items.iter().map(mark_price_point_from).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill mark_price failed");
            Vec::new()
        }
    }
}

fn mark_price_point_from(mp: &MarkPrice) -> MarkPricePoint {
    MarkPricePoint {
        ts_ms: mp.timestamp,
        mark: mp.mark_price,
        index: mp.index_price.unwrap_or(f64::NAN),
    }
}

/// Pull up to `limit` historical funding rates from REST for (exchange, account, symbol).
/// Returns oldest→newest. Empty on any error.
pub async fn funding_rate_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<FundingRatePoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_funding_rate_history(SymbolInput::Raw(symbol), None, None, Some(limit), account)
        .await;
    match res {
        Ok(items) => items.iter().map(funding_rate_point_from).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill funding_rate failed");
            Vec::new()
        }
    }
}

fn funding_rate_point_from(fr: &FundingRate) -> FundingRatePoint {
    FundingRatePoint {
        ts_ms: fr.timestamp,
        rate: fr.rate,
        next_funding_time_ms: fr.next_funding_time.unwrap_or(0),
    }
}

/// Pull up to `limit` historical liquidation events from REST for
/// (exchange, account, symbol). Returns oldest→newest. Empty on any error.
pub async fn liquidations_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<LiquidationPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_liquidation_history(
            Some(SymbolInput::Raw(symbol)),
            None,
            None,
            Some(limit),
            account,
        )
        .await;
    match res {
        Ok(items) => items.iter().map(liquidation_point_from).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill liquidations failed");
            Vec::new()
        }
    }
}

fn liquidation_point_from(liq: &Liquidation) -> LiquidationPoint {
    let side = match liq.side {
        TradeSide::Buy => 0u8,
        TradeSide::Sell => 1u8,
    };
    LiquidationPoint {
        ts_ms: liq.timestamp,
        price: liq.price,
        quantity: liq.quantity,
        value: liq.value.unwrap_or(f64::NAN),
        side,
    }
}

/// Fetch the current insurance fund snapshot from REST for (exchange, account, symbol).
/// Returns 0 or 1 element. Empty on any error.
pub async fn insurance_fund_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    _limit: usize,
) -> Vec<InsuranceFundPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let res = rest
        .get_insurance_fund(Some(SymbolInput::Raw(symbol)), account)
        .await;
    match res {
        Ok(items) => items.iter().map(insurance_fund_point_from).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill insurance_fund failed");
            Vec::new()
        }
    }
}

fn insurance_fund_point_from(fund: &InsuranceFund) -> InsuranceFundPoint {
    InsuranceFundPoint {
        ts_ms: fund.timestamp,
        balance: fund.balance,
    }
}

/// Pull up to `limit` mark-price klines from REST for (exchange, account, symbol, interval).
/// Returns oldest→newest. Empty on any error.
pub async fn mark_price_klines_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    limit: usize,
) -> Vec<MarkPriceKlinePoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_mark_price_klines(SymbolInput::Raw(symbol), interval, Some(limit), account, None)
        .await;
    match res {
        Ok(bars) => bars.iter().map(|k| MarkPriceKlinePoint {
            open_time: k.open_time,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume,
            quote_volume: k.quote_volume.unwrap_or(f64::NAN),
            trades_count: k.trades.unwrap_or(0),
        }).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, interval, "rest backfill mark_price_klines failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` index-price klines from REST for (exchange, account, symbol, interval).
/// Returns oldest→newest. Empty on any error.
pub async fn index_price_klines_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    limit: usize,
) -> Vec<IndexPriceKlinePoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_index_price_klines(SymbolInput::Raw(symbol), interval, Some(limit), account, None)
        .await;
    match res {
        Ok(bars) => bars.iter().map(|k| IndexPriceKlinePoint {
            open_time: k.open_time,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume,
            quote_volume: k.quote_volume.unwrap_or(f64::NAN),
            trades_count: k.trades.unwrap_or(0),
        }).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, interval, "rest backfill index_price_klines failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` premium-index klines from REST for (exchange, account, symbol, interval).
/// Returns oldest→newest. Empty on any error.
pub async fn premium_index_klines_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    limit: usize,
) -> Vec<PremiumIndexKlinePoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_premium_index_klines(SymbolInput::Raw(symbol), interval, Some(limit), account, None)
        .await;
    match res {
        Ok(bars) => bars.iter().map(|k| PremiumIndexKlinePoint {
            open_time: k.open_time,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume,
            quote_volume: k.quote_volume.unwrap_or(f64::NAN),
            trades_count: k.trades.unwrap_or(0),
        }).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, interval, "rest backfill premium_index_klines failed");
            Vec::new()
        }
    }
}

// ─── Indicators / Full warm-seed helpers ──────────────────────────────────────
//
// Each helper mirrors its Compact sibling but maps to the enriched DataPoint
// type. Called by `station.rs` when `PersistDepth` is `Indicators` or `Full`.

/// Fetch the current ticker snapshot (Indicators depth) from REST.
/// Returns 0 or 1 element. Empty on any error or unsupported.
pub async fn tickers_indicators_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    _limit: usize,
) -> Vec<TickerIndicatorsPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let sym = SymbolInput::Raw(symbol);
    match rest.get_ticker(sym, account).await {
        Ok(t) => vec![TickerIndicatorsPoint::from_ticker(&t)],
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill ticker_indicators failed");
            Vec::new()
        }
    }
}

/// Fetch the current ticker snapshot (Full depth) from REST.
/// Returns 0 or 1 element. Empty on any error or unsupported.
pub async fn tickers_full_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    _limit: usize,
) -> Vec<TickerFullPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let sym = SymbolInput::Raw(symbol);
    match rest.get_ticker(sym, account).await {
        Ok(t) => vec![TickerFullPoint::from_ticker(&t)],
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill ticker_full failed");
            Vec::new()
        }
    }
}

/// Fetch the current mark-price snapshot (Indicators depth) from REST.
/// Returns 0 or 1+ elements. Empty on any error.
pub async fn mark_price_indicators_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    _limit: usize,
) -> Vec<MarkPriceIndicatorsPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    match rest.get_premium_index(Some(SymbolInput::Raw(symbol)), account).await {
        Ok(items) => items.iter().map(MarkPriceIndicatorsPoint::from_mark_price).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill mark_price_indicators failed");
            Vec::new()
        }
    }
}

/// Fetch the current mark-price snapshot (Full depth) from REST.
/// Returns 0 or 1+ elements. Empty on any error.
pub async fn mark_price_full_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    _limit: usize,
) -> Vec<MarkPriceFullPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    match rest.get_premium_index(Some(SymbolInput::Raw(symbol)), account).await {
        Ok(items) => items.iter().map(MarkPriceFullPoint::from_mark_price).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill mark_price_full failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` historical funding rates (Indicators depth) from REST.
/// Returns oldest→newest. Empty on any error.
pub async fn funding_rate_indicators_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<FundingRateIndicatorsPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    match rest.get_funding_rate_history(SymbolInput::Raw(symbol), None, None, Some(limit), account).await {
        Ok(items) => items.iter().map(FundingRateIndicatorsPoint::from_funding_rate).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill funding_rate_indicators failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` historical funding rates (Full depth) from REST.
/// Returns oldest→newest. Empty on any error.
pub async fn funding_rate_full_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<FundingRateFullPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    match rest.get_funding_rate_history(SymbolInput::Raw(symbol), None, None, Some(limit), account).await {
        Ok(items) => items.iter().map(FundingRateFullPoint::from_funding_rate).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill funding_rate_full failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` open-interest snapshots (Indicators depth) from REST.
/// Returns oldest→newest. Empty on any error.
pub async fn open_interest_indicators_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<OpenInterestIndicatorsPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    match rest.get_open_interest_history(
        SymbolInput::Raw(symbol),
        "5m",
        None,
        None,
        Some(limit),
        account,
    ).await {
        Ok(items) => items.iter().map(OpenInterestIndicatorsPoint::from_open_interest).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill open_interest_indicators failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` historical liquidation events (Indicators depth) from REST.
/// Returns oldest→newest. Empty on any error.
pub async fn liquidation_indicators_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<LiquidationIndicatorsPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    match rest.get_liquidation_history(
        Some(SymbolInput::Raw(symbol)),
        None,
        None,
        Some(limit),
        account,
    ).await {
        Ok(items) => items.iter().map(LiquidationIndicatorsPoint::from_liquidation).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill liquidation_indicators failed");
            Vec::new()
        }
    }
}

/// Pull up to `limit` historical liquidation events (Full depth) from REST.
/// Returns oldest→newest. Empty on any error.
pub async fn liquidation_full_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> Vec<LiquidationFullPoint> {
    let Some(rest) = hub.rest(exchange) else { return Vec::new(); };
    let limit = limit.min(1000).max(1) as u32;
    match rest.get_liquidation_history(
        Some(SymbolInput::Raw(symbol)),
        None,
        None,
        Some(limit),
        account,
    ).await {
        Ok(items) => items.iter().map(LiquidationFullPoint::from_liquidation).collect(),
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill liquidation_full failed");
            Vec::new()
        }
    }
}
