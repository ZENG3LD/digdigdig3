//! REST backfill for warm-start.
//!
//! When the disk store has fewer than `warm` records, the forwarder calls one
//! of these helpers to pull recent history from the exchange REST API. Each
//! helper returns oldest→newest, already deduped against `existing_count`
//! (so the caller can simply emit them to broadcast).
//!
//! Only the data-classes that have a sensible REST history endpoint are
//! supported here: Trade (`get_recent_trades`), Kline (`get_klines`). Others
//! fall back to "wait for live" (no REST equivalent or it's account-gated).

use std::sync::Arc;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, SymbolInput};
use digdigdig3::core::websocket::KlineInterval;

use crate::data::{BarPoint, TradePoint};
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
