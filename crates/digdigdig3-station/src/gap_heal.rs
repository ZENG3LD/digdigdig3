//! Auto-heal on WS disconnect — klines only.
//!
//! Model (mirrors `mylittlechart::live_data::ws_manager` + `chart-app::tick`
//! `ConnectorReady` handler):
//!
//! 1. The Station forwarder reads `ws.event_stream().next()` in a loop.
//! 2. When the underlying WS connection drops, the stream yields `Err(...)`
//!    (transport error) or `None` (stream closed). Both = "disconnect".
//! 3. On disconnect, IF the kind is `Kline(interval)`, the forwarder
//!    immediately calls REST `get_klines(limit=N)` where
//!    `N = max(default_limit, ceil(time_since_last_write / interval))`.
//! 4. Returned bars are pushed via `Series::upsert_by_ts` —
//!    last-write-wins by `open_time`. Any half-formed or corrupt live bar is
//!    overwritten with the canonical REST value.
//! 5. The disconnect itself triggers an internal reconnect inside
//!    `UniversalWsTransport`. The forwarder re-attaches `ws.event_stream()`
//!    after the heal and continues.
//!
//! Trade / OB / Ticker / Mark / Funding / OI / Liquidation are LIVE-ONLY:
//! a disconnect causes a gap that no public REST endpoint can bridge. The
//! forwarder still re-attaches the stream so future live events resume.

use std::sync::Arc;
use std::time::Duration;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use serde::{Deserialize, Serialize};

use crate::data::BarPoint;

/// Auto-heal configuration. Active only for `Kind::Kline` streams.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GapHealConfig {
    pub enabled: bool,
    /// Minimum number of bars to pull on every disconnect-driven heal,
    /// independent of how long the disconnect lasted. Default 300 — enough
    /// to overwrite any in-flight broken bar plus comfortable history.
    pub default_limit: usize,
    /// Hard cap on heal size to avoid runaway REST calls on multi-hour
    /// outages. Default 1000 (most exchanges' get_klines hard limit).
    pub max_limit: usize,
}

impl Default for GapHealConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_limit: parse_env_usize("DIG3_HEAL_DEFAULT_LIMIT").unwrap_or(300),
            max_limit: parse_env_usize("DIG3_HEAL_MAX_LIMIT").unwrap_or(1000),
        }
    }
}

fn parse_env_usize(key: &str) -> Option<usize> {
    std::env::var(key).ok().and_then(|s| s.parse().ok())
}

impl GapHealConfig {
    pub fn on() -> Self {
        Self { enabled: true, ..Self::default() }
    }
    pub fn default_limit(mut self, n: usize) -> Self { self.default_limit = n; self }
    pub fn max_limit(mut self, n: usize) -> Self { self.max_limit = n; self }
}

/// Compute heal size given the time since the last written bar and the kline
/// interval. Returns `max(default_limit, ceil(gap_ms / interval_ms))`, clipped
/// to `max_limit`. Used by the forwarder when sizing the REST `get_klines`
/// limit on disconnect.
pub fn heal_limit(
    cfg: &GapHealConfig,
    interval: &str,
    last_written_ms: i64,
    now_ms: i64,
) -> usize {
    let base = cfg.default_limit.max(1);
    let capped = base.min(cfg.max_limit.max(1));
    if last_written_ms <= 0 || now_ms <= last_written_ms {
        return capped;
    }
    let Some(d) = kline_interval_to_duration(interval) else { return capped };
    let interval_ms = d.as_millis().max(1) as u64;
    let gap_ms = (now_ms - last_written_ms) as u64;
    let need = ((gap_ms + interval_ms - 1) / interval_ms) as usize; // ceil
    need.max(base).min(cfg.max_limit.max(1))
}

/// Convert a kline interval string to a Duration. Supports canonical
/// Binance-style suffixes: s, m, h, d, w. Returns None for malformed input.
pub fn kline_interval_to_duration(s: &str) -> Option<Duration> {
    let (n_str, unit) = s.split_at(s.len().saturating_sub(1));
    let n: u64 = n_str.parse().ok()?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86400,
        "w" => n * 86400 * 7,
        _ => return None,
    };
    Some(Duration::from_secs(secs))
}

/// REST pull for kline auto-heal. Returns latest N bars (sorted by exchange,
/// generally oldest→newest). The forwarder upserts ALL of them and then emits
/// only the ones strictly newer than `last_emitted_ms` via [`select_heal_window`].
pub async fn heal_klines(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    _last_emitted_ms: i64,
    limit: usize,
) -> Vec<BarPoint> {
    crate::backfill::klines_recent(hub, exchange, account, symbol, interval, limit).await
}

/// Filter REST-returned bars to those strictly after `last_seen_ms`, sorted
/// ascending, deduped by open_time. Used post-REST to decide which bars to
/// emit to consumers as "new". (Bars at or before `last_seen_ms` are still
/// upserted into memory + disk for last-write-wins canonicalization, but the
/// consumer doesn't need a duplicate emit.)
pub fn select_heal_window<T: crate::series::DataPoint>(
    pulled: Vec<T>,
    last_seen_ms: i64,
) -> Vec<T> {
    let mut filtered: Vec<T> = pulled.into_iter().filter(|p| p.timestamp_ms() > last_seen_ms).collect();
    filtered.sort_by_key(|p| p.timestamp_ms());
    filtered.dedup_by_key(|p| p.timestamp_ms());
    filtered
}
