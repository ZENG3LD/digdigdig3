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
use digdigdig3::core::types::{
    AccountType, AggTrade, ExchangeId, FundingRate, InsuranceFund, Liquidation, MarkPrice,
    OpenInterest, SymbolInput, TradeHistoryTier, TradeSide,
};
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

// ═══════════════════════════════════════════════════════════════════════════════
// SEED OUTCOME — honest requested-vs-achieved reporting for trade/kline seeds
// ═══════════════════════════════════════════════════════════════════════════════

/// Where a trade/kline seed's points came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedSource {
    /// Paginated `get_agg_trades` walk (deep or window-capped).
    AggTradesPaginated,
    /// Single shallow `get_recent_trades` call (venue has no aggTrade
    /// pagination, or capability says `RecentOnly`).
    RecentTradesFallback,
    /// Synthetic trades derived from paginated 1m klines (price-path
    /// deep-seed path: renko/pnf/kagi/three-line-break).
    KlineSynthetic,
    /// Raw kline bars (no synthesis) — `klines_paginated` result used
    /// directly, not folded into synthetic trades.
    Klines,
}

/// Why a seed stopped short of `requested`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncationReason {
    /// Capability says `RecentOnly` — a single shallow call is the venue's
    /// entire history, no pagination was attempted.
    VenueRecentOnly,
    /// Capability says `RestWindow` and the cursor walk hit `max_back_ms`
    /// before reaching `requested` points.
    VenueWindowCap,
    /// Pagination ran until the venue returned an empty/partial page —
    /// genuinely exhausted history (or hit the ID/time origin), not a
    /// declared ceiling.
    VenueHistoryExhausted,
    /// A REST call failed mid-walk; the seed carries whatever was
    /// collected before the error.
    Error,
}

/// Honest requested-vs-achieved report for one backfill/rewarm seed call.
///
/// Every `backfill::*` helper that used to return a bare `Vec<T>` (silently
/// empty on error/truncation) now pairs its `Vec<T>` with one of these, so
/// callers (and ultimately the bridge/UI) can distinguish "got everything
/// asked for" from "venue capped us" from "call failed outright" instead of
/// treating all three as the same empty-looking result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedOutcome {
    /// Points the caller asked for (e.g. `warm_n`, or `page_size * n_pages`).
    pub requested: usize,
    /// Points actually returned.
    pub achieved: usize,
    /// Where the points came from.
    pub source: SeedSource,
    /// `None` = fully satisfied (`achieved >= requested` or the venue's
    /// deep pagination ran to genuine exhaustion with no ceiling involved).
    /// `Some(_)` = stopped short for a specific, nameable reason.
    pub truncated_by: Option<TruncationReason>,
}

impl SeedOutcome {
    /// Build a fully-satisfied outcome (no truncation).
    pub const fn full(requested: usize, achieved: usize, source: SeedSource) -> Self {
        Self { requested, achieved, source, truncated_by: None }
    }

    /// Build a truncated outcome.
    pub const fn truncated(
        requested: usize,
        achieved: usize,
        source: SeedSource,
        reason: TruncationReason,
    ) -> Self {
        Self { requested, achieved, source, truncated_by: Some(reason) }
    }

    /// True when the seed produced at least as many points as requested,
    /// or ended without a nameable truncation reason.
    pub fn is_satisfied(&self) -> bool {
        self.truncated_by.is_none() || self.achieved >= self.requested
    }
}

/// Pull up to `limit` recent trades from REST for (exchange, account, symbol).
/// Returns oldest→newest, paired with a [`SeedOutcome`] reporting
/// requested-vs-achieved. Empty vec + `TruncationReason::Error` on any REST
/// error or unsupported endpoint.
pub async fn trades_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> (Vec<TradePoint>, SeedOutcome) {
    let Some(rest) = hub.rest(exchange) else {
        return (
            Vec::new(),
            SeedOutcome::truncated(limit, 0, SeedSource::RecentTradesFallback, TruncationReason::Error),
        );
    };
    // Clamp the request to what the venue's RecentOnly tier (or the
    // conservative default) actually returns per call — no point asking
    // for more than the venue can ever give back in one shot.
    let cap = recent_only_cap(rest.trade_history_capabilities().tier_for(account));
    let effective_limit = limit.min(cap.unwrap_or(1000)).min(1000).max(1);
    let res = rest
        .get_recent_trades(SymbolInput::Raw(symbol), Some(effective_limit as u32), account)
        .await;
    match res {
        Ok(trades) => {
            let points: Vec<TradePoint> = trades.iter().map(TradePoint::from_public).collect();
            let achieved = points.len();
            let outcome = if cap.is_some_and(|c| limit > c) {
                SeedOutcome::truncated(limit, achieved, SeedSource::RecentTradesFallback, TruncationReason::VenueRecentOnly)
            } else {
                SeedOutcome::full(limit, achieved, SeedSource::RecentTradesFallback)
            };
            (points, outcome)
        }
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill trades failed");
            (
                Vec::new(),
                SeedOutcome::truncated(limit, 0, SeedSource::RecentTradesFallback, TruncationReason::Error),
            )
        }
    }
}

/// Maximum trades a single `RecentOnly` call can return, if the tier is
/// `RecentOnly`. `None` for tiers that support pagination (no single-call
/// ceiling relevant to this clamp).
fn recent_only_cap(tier: TradeHistoryTier) -> Option<usize> {
    match tier {
        TradeHistoryTier::RecentOnly { max_trades } => Some(max_trades as usize),
        _ => None,
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

/// Paginated kline backfill for deep warm-seeding of price-path-triggered
/// derived bars (Renko/PnF/Kagi/Three-Line-Break). Walks the `end_time_ms`
/// cursor backwards `n_pages` times to gather weeks of 1m history — a
/// single page (1000 bars of `1m`) covers under a day, not enough to
/// bootstrap a multi-week brick/column/segment chart.
///
/// Algorithm (mirrors `agg_trades_paginated`'s `from_id` cursor walk, using
/// `get_klines`'s `end_time` cursor instead — see `fetch_history` for the
/// same semantic on a single page):
/// 1. First page: request `page_size` bars ending at `now_ms` (exclusive
///    cursor) → newest `page_size` bars.
/// 2. For each subsequent page: `end_time_ms = min(open_time)_of_prev_page`
///    → returns `page_size` bars strictly earlier.
/// 3. Stop when the venue returns fewer than `page_size` bars (history
///    exhausted) or when `n_pages` have been collected.
///
/// Dedupes by `open_time` and returns oldest→newest, paired with a
/// [`SeedOutcome`]. Empty vec + `TruncationReason::Error` on any error or
/// unsupported endpoint (graceful degrade — caller falls back to today's
/// shallow aggTrade-only seed).
///
/// Consults `TradeHistoryCapabilities::kline_backpage` first: when the
/// connector's `get_klines` does not actually wire `end_time` through to
/// the REST call, a paginated walk here would just refetch the same page
/// `n_pages` times — so a false flag skips straight to a single page and
/// reports `TruncationReason::VenueWindowCap` (the honesty flag, not a
/// venue-side depth ceiling, is what stopped the walk).
pub async fn klines_paginated(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    page_size: usize,
    n_pages: usize,
) -> (Vec<BarPoint>, SeedOutcome) {
    use std::collections::BTreeMap;
    let requested = page_size.saturating_mul(n_pages);
    if n_pages == 0 || page_size == 0 {
        return (Vec::new(), SeedOutcome::full(0, 0, SeedSource::Klines));
    }
    let Some(rest) = hub.rest(exchange) else {
        return (Vec::new(), SeedOutcome::truncated(requested, 0, SeedSource::Klines, TruncationReason::Error));
    };
    let limit = page_size.min(1000).max(1) as u16;
    let kline_backpage = rest.trade_history_capabilities().kline_backpage;
    let effective_n_pages = if kline_backpage { n_pages } else { 1 };

    let mut by_open_time: BTreeMap<i64, BarPoint> = BTreeMap::new();
    let mut next_end_time: Option<i64> = Some(chrono::Utc::now().timestamp_millis());
    let mut pages_done = 0usize;
    let mut had_error = false;

    while pages_done < effective_n_pages {
        let res = rest
            .get_klines(SymbolInput::Raw(symbol), interval, Some(limit), account, next_end_time)
            .await;
        let page: Vec<BarPoint> = match res {
            Ok(bars) => bars.iter().map(BarPoint::from_kline).collect(),
            Err(e) => {
                tracing::debug!(
                    ?e,
                    exchange = ?exchange,
                    interval,
                    page = pages_done,
                    "klines_paginated page failed; stopping at partial result"
                );
                had_error = true;
                break;
            }
        };
        if page.is_empty() {
            break;
        }
        let min_open_time = page.iter().map(|p| p.open_time).min().unwrap_or(0);
        let was_partial = page.len() < limit as usize;
        for p in page {
            by_open_time.entry(p.open_time).or_insert(p);
        }
        pages_done += 1;
        if was_partial {
            // Venue exhausted its history window — no point in another call.
            break;
        }
        // Next page: bars strictly earlier than this page's oldest bar.
        next_end_time = Some(min_open_time);
    }

    let achieved = by_open_time.len();
    let outcome = if had_error {
        SeedOutcome::truncated(requested, achieved, SeedSource::Klines, TruncationReason::Error)
    } else if !kline_backpage && n_pages > 1 {
        SeedOutcome::truncated(requested, achieved, SeedSource::Klines, TruncationReason::VenueWindowCap)
    } else if pages_done < effective_n_pages {
        SeedOutcome::truncated(requested, achieved, SeedSource::Klines, TruncationReason::VenueHistoryExhausted)
    } else {
        SeedOutcome::full(requested, achieved, SeedSource::Klines)
    };

    (by_open_time.into_values().collect(), outcome)
}

/// Pull up to `limit` aggregated trades from REST for (exchange, account, symbol).
/// Returns oldest→newest, paired with a [`SeedOutcome`]. Empty on any error
/// or unsupported endpoint.
pub async fn agg_trades_recent(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    limit: usize,
) -> (Vec<AggTradePoint>, SeedOutcome) {
    let Some(rest) = hub.rest(exchange) else {
        return (Vec::new(), SeedOutcome::truncated(limit, 0, SeedSource::AggTradesPaginated, TruncationReason::Error));
    };
    let effective_limit = limit.min(1000).max(1) as u32;
    let res = rest
        .get_agg_trades(SymbolInput::Raw(symbol), Some(effective_limit), None, account)
        .await;
    match res {
        Ok(trades) => {
            let points: Vec<AggTradePoint> = trades.iter().map(agg_trade_point_from).collect();
            let achieved = points.len();
            let outcome = SeedOutcome::full(limit, achieved, SeedSource::AggTradesPaginated);
            (points, outcome)
        }
        Err(e) => {
            tracing::debug!(?e, exchange = ?exchange, "rest backfill agg_trades failed");
            (Vec::new(), SeedOutcome::truncated(limit, 0, SeedSource::AggTradesPaginated, TruncationReason::Error))
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
/// Dedupes by `aggregate_id` and returns oldest→newest, paired with a
/// [`SeedOutcome`]. On exchanges without paginated aggTrades support
/// (`NotImplemented` from `get_agg_trades`), returns the result of
/// `trades_recent` (single page recent trades) as a graceful fallback.
///
/// Consults `TradeHistoryCapabilities::tier_for(account)` BEFORE ever
/// calling `get_agg_trades`:
/// - `RecentOnly` → skip pagination entirely, go straight to the
///   `trades_recent` fallback clamped to `max_trades`. No "attempt" against
///   an endpoint the venue caps to a single shallow call — hammering it
///   with a from_id cursor buys nothing. `truncated_by =
///   TruncationReason::VenueRecentOnly`.
/// - `RestWindow` → page normally, but stop the walk once a page's oldest
///   trade crosses `now - max_back_ms` (the venue-side wall, e.g. Binance
///   futures aggTrades 24h) instead of discovering the wall by an empty
///   page. `truncated_by = TruncationReason::VenueWindowCap`.
/// - `RestDeep` / `FileDump` (FileDump has no REST path yet — treated like
///   RestDeep for this fn) → current behavior: page until `n_pages` are
///   collected or the venue returns a partial/empty page
///   (`TruncationReason::VenueHistoryExhausted`).
pub async fn agg_trades_paginated(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    page_size: usize,
    n_pages: usize,
) -> (Vec<AggTradePoint>, SeedOutcome) {
    use std::collections::BTreeMap;
    let requested = page_size.saturating_mul(n_pages);
    if n_pages == 0 || page_size == 0 {
        return (Vec::new(), SeedOutcome::full(0, 0, SeedSource::AggTradesPaginated));
    }
    let Some(rest) = hub.rest(exchange) else {
        return (Vec::new(), SeedOutcome::truncated(requested, 0, SeedSource::AggTradesPaginated, TruncationReason::Error));
    };
    let limit = page_size.min(1000).max(1) as u32;

    let tier = rest.trade_history_capabilities().tier_for(account);
    if let TradeHistoryTier::RecentOnly { max_trades } = tier {
        // Venue has no pagination cursor at all — a from_id walk against
        // this endpoint would just refetch the same fixed window forever.
        // Go straight to the shallow fallback.
        let cap = (max_trades as usize).min(limit as usize).max(1);
        let (trades, _inner_outcome) = trades_recent(hub, exchange, account, symbol, cap).await;
        let achieved = trades.len();
        let points: Vec<AggTradePoint> = trades
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
        return (
            points,
            SeedOutcome::truncated(requested, achieved, SeedSource::RecentTradesFallback, TruncationReason::VenueRecentOnly),
        );
    }
    let window_wall_ms: Option<i64> = match tier {
        TradeHistoryTier::RestWindow { max_back_ms, .. } if max_back_ms > 0 => {
            Some(chrono::Utc::now().timestamp_millis() - max_back_ms as i64)
        }
        _ => None,
    };

    let mut by_id: BTreeMap<u64, AggTradePoint> = BTreeMap::new();
    let mut next_from_id: Option<u64> = None;
    let mut pages_done = 0usize;
    let mut agg_fallback = false;
    let mut hit_window_wall = false;

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
        // Find the earliest aggregate_id / timestamp on this page (cursor
        // for next / wall check).
        let min_id = page.iter().map(|p| p.agg_id).min().unwrap_or(0);
        let min_ts = page.iter().map(|p| p.ts_ms).min().unwrap_or(i64::MAX);
        let was_partial = page.len() < limit as usize;
        for p in page {
            by_id.entry(p.agg_id).or_insert(p);
        }
        pages_done += 1;
        if let Some(wall) = window_wall_ms {
            if min_ts <= wall {
                // This page already crossed the venue-side lookback wall —
                // stop paging further, we're past the productive window.
                hit_window_wall = true;
                break;
            }
        }
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
        let (trades, _inner_outcome) = trades_recent(hub, exchange, account, symbol, limit as usize).await;
        let achieved = trades.len();
        let points: Vec<AggTradePoint> = trades
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
        return (
            points,
            SeedOutcome::truncated(requested, achieved, SeedSource::RecentTradesFallback, TruncationReason::Error),
        );
    }

    let achieved = by_id.len();
    let outcome = if hit_window_wall {
        SeedOutcome::truncated(requested, achieved, SeedSource::AggTradesPaginated, TruncationReason::VenueWindowCap)
    } else if pages_done < n_pages {
        SeedOutcome::truncated(requested, achieved, SeedSource::AggTradesPaginated, TruncationReason::VenueHistoryExhausted)
    } else {
        SeedOutcome::full(requested, achieved, SeedSource::AggTradesPaginated)
    };

    (by_id.into_values().collect(), outcome)
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

#[cfg(test)]
mod seed_outcome_tests {
    //! Unit tests for the pure `SeedOutcome`/capability-gating logic in this
    //! module. Live REST-dependent behavior (the actual paginated wall-clamp
    //! against a real venue) is covered by the `--ignored` e2e examples —
    //! these tests exercise the deterministic, no-network parts: outcome
    //! construction, `is_satisfied`, `recent_only_cap`, and the
    //! not-connected-hub error path every `backfill::*` fn shares.

    use super::*;
    use digdigdig3::core::types::HistoryCursor;
    use std::sync::Arc;

    // ── SeedOutcome: requested vs achieved reporting ──────────────────────

    #[test]
    fn seed_outcome_full_reports_requested_and_achieved() {
        let o = SeedOutcome::full(1000, 1000, SeedSource::AggTradesPaginated);
        assert_eq!(o.requested, 1000);
        assert_eq!(o.achieved, 1000);
        assert_eq!(o.source, SeedSource::AggTradesPaginated);
        assert!(o.truncated_by.is_none());
        assert!(o.is_satisfied());
    }

    #[test]
    fn seed_outcome_truncated_reports_short_achieved_and_reason() {
        let o = SeedOutcome::truncated(50_000, 60, SeedSource::RecentTradesFallback, TruncationReason::VenueRecentOnly);
        assert_eq!(o.requested, 50_000);
        assert_eq!(o.achieved, 60);
        assert_eq!(o.truncated_by, Some(TruncationReason::VenueRecentOnly));
        // Achieved << requested AND a truncation reason is set → not satisfied.
        assert!(!o.is_satisfied());
    }

    #[test]
    fn seed_outcome_truncated_but_fully_achieved_is_satisfied() {
        // A RestDeep walk that hit `VenueHistoryExhausted` exactly at
        // `requested` (e.g. n_pages consumed exactly) still counts as
        // satisfied — the reason names WHY the walk stopped, not that it
        // under-delivered.
        let o = SeedOutcome::truncated(1000, 1000, SeedSource::AggTradesPaginated, TruncationReason::VenueHistoryExhausted);
        assert!(o.is_satisfied());
    }

    // ── recent_only_cap: per-tier clamp lookup ────────────────────────────

    #[test]
    fn recent_only_cap_returns_max_trades_for_recent_only_tier() {
        let tier = TradeHistoryTier::RecentOnly { max_trades: 60 };
        assert_eq!(recent_only_cap(tier), Some(60));
    }

    #[test]
    fn recent_only_cap_is_none_for_paginating_tiers() {
        assert_eq!(recent_only_cap(TradeHistoryTier::RestDeep { cursor: HistoryCursor::FromId }), None);
        assert_eq!(
            recent_only_cap(TradeHistoryTier::RestWindow { cursor: HistoryCursor::FromId, max_back_ms: 86_400_000 }),
            None,
        );
        assert_eq!(
            recent_only_cap(TradeHistoryTier::FileDump { url_pattern: "https://example.test/%s.csv.gz", since_ms: 0 }),
            None,
        );
    }

    // ── tier_for: account-type dispatch (spot vs futures split) ──────────

    #[test]
    fn tier_for_dispatches_futures_account_types_to_futures_tier() {
        use digdigdig3::core::types::{AccountType, TradeHistoryCapabilities};
        let caps = TradeHistoryCapabilities {
            spot: TradeHistoryTier::RestDeep { cursor: HistoryCursor::FromId },
            futures: TradeHistoryTier::RestWindow { cursor: HistoryCursor::FromId, max_back_ms: 86_400_000 },
            kline_backpage: true,
        };
        assert_eq!(caps.tier_for(AccountType::FuturesCross), caps.futures);
        assert_eq!(caps.tier_for(AccountType::FuturesIsolated), caps.futures);
        assert_eq!(caps.tier_for(AccountType::Spot), caps.spot);
        // Non-spot/futures account types (Margin/Earn/Lending/Options/Convert)
        // fall back to the spot tier per the documented contract.
        assert_eq!(caps.tier_for(AccountType::Margin), caps.spot);
    }

    // ── Not-connected-hub path: every backfill fn reports Error honestly ──

    #[tokio::test]
    async fn trades_recent_on_unconnected_hub_reports_error_truncation() {
        let hub = Arc::new(ExchangeHub::new());
        let (points, outcome) = trades_recent(&hub, ExchangeId::Binance, AccountType::Spot, "BTCUSDT", 500).await;
        assert!(points.is_empty());
        assert_eq!(outcome.requested, 500);
        assert_eq!(outcome.achieved, 0);
        assert_eq!(outcome.truncated_by, Some(TruncationReason::Error));
    }

    #[tokio::test]
    async fn agg_trades_paginated_on_unconnected_hub_reports_error_truncation() {
        let hub = Arc::new(ExchangeHub::new());
        let (points, outcome) = agg_trades_paginated(&hub, ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT", 1000, 5).await;
        assert!(points.is_empty());
        assert_eq!(outcome.requested, 5000);
        assert_eq!(outcome.achieved, 0);
        assert_eq!(outcome.truncated_by, Some(TruncationReason::Error));
    }

    #[tokio::test]
    async fn klines_paginated_on_unconnected_hub_reports_error_truncation() {
        let hub = Arc::new(ExchangeHub::new());
        let (points, outcome) = klines_paginated(&hub, ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT", "1m", 1000, 3).await;
        assert!(points.is_empty());
        assert_eq!(outcome.requested, 3000);
        assert_eq!(outcome.achieved, 0);
        assert_eq!(outcome.truncated_by, Some(TruncationReason::Error));
    }

    #[tokio::test]
    async fn klines_paginated_zero_pages_is_full_empty_not_error() {
        // n_pages=0 short-circuits BEFORE the hub lookup — this is a
        // legitimate "nothing requested" outcome, not a failure.
        let hub = Arc::new(ExchangeHub::new());
        let (points, outcome) = klines_paginated(&hub, ExchangeId::Binance, AccountType::Spot, "BTCUSDT", "1m", 1000, 0).await;
        assert!(points.is_empty());
        assert_eq!(outcome.requested, 0);
        assert!(outcome.truncated_by.is_none());
    }
}
