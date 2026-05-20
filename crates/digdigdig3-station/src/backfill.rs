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

use crate::data::{BarPoint, TradePoint};

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
