//! Gap-heal — proactive REST backfill when live WS deltas show a timestamp
//! jump larger than the configured threshold.
//!
//! Rationale: WebSocket transports already reconnect on their own with backoff,
//! and `event_stream()` continues silently after a reconnect. The forwarder
//! never sees a "reconnect event", but it CAN observe that the new live event
//! timestamp is N seconds past the last one — that's the gap signature.
//!
//! On gap detection the forwarder calls one of the kind-specific REST helpers
//! to pull `[last_seen_ts, current_event_ts]`, emits the recovered points to
//! consumers, and persists them to disk transparently. Consumer sees a
//! continuous stream.

use std::sync::Arc;
use std::time::Duration;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};

use crate::data::{BarPoint, TradePoint};

/// Default gap thresholds per kind. If a live event arrives whose
/// `timestamp_ms - last_seen_ms` exceeds the threshold below, gap-heal runs.
#[derive(Debug, Clone, Copy)]
pub struct GapHealConfig {
    pub enabled: bool,
    /// Trades: threshold beyond which we ask REST for missed prints (default 10s).
    pub trade_gap: Duration,
    /// Klines: threshold beyond N intervals (default 3 — i.e. 1m kline gap > 180s).
    pub kline_intervals: u32,
    /// Max records to pull per REST call (mirrors warm-start cap).
    pub max_records: usize,
}

impl Default for GapHealConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trade_gap: Duration::from_secs(10),
            kline_intervals: 3,
            max_records: 500,
        }
    }
}

impl GapHealConfig {
    pub fn on() -> Self {
        Self { enabled: true, ..Self::default() }
    }
    pub fn trade_gap(mut self, d: Duration) -> Self { self.trade_gap = d; self }
    pub fn kline_intervals(mut self, n: u32) -> Self { self.kline_intervals = n; self }
    pub fn max_records(mut self, n: usize) -> Self { self.max_records = n; self }
}

/// Pure decision: should we attempt gap-heal given (kind, last_seen_ms, now_ms)?
///
/// Returns false when:
/// - config is off,
/// - never-seen-yet (last_seen_ms == 0),
/// - now_ms <= last_seen_ms (clock skew or duplicate),
/// - kind has no REST history endpoint (anything other than Trade/Kline),
/// - Kline interval string is malformed.
///
/// Returns true when:
/// - Trade: gap > `cfg.trade_gap`,
/// - Kline(iv): gap > `parse(iv) * cfg.kline_intervals`.
pub fn should_heal(
    kind: &crate::series::Kind,
    last_seen_ms: i64,
    now_ms: i64,
    cfg: &GapHealConfig,
) -> bool {
    if !cfg.enabled || last_seen_ms <= 0 || now_ms <= last_seen_ms {
        return false;
    }
    let gap_ms = (now_ms - last_seen_ms) as u128;
    match kind {
        crate::series::Kind::Trade => gap_ms > cfg.trade_gap.as_millis(),
        crate::series::Kind::Kline(iv) => {
            let Some(d) = kline_interval_to_duration(iv) else { return false };
            gap_ms > (d.as_millis() * cfg.kline_intervals as u128)
        }
        _ => false,
    }
}

/// Filter recovered REST points to those that fill the gap `(last_seen_ms, now_ms]`
/// inclusive of `now_ms`-edge tolerance. Returns oldest→newest.
///
/// `pulled` is whatever REST returned, possibly unsorted, possibly containing
/// duplicates of already-seen points. We:
/// - drop anything with ts <= last_seen_ms (already delivered),
/// - sort ascending,
/// - dedup by exact ts (kline backfill can return overlapping points).
pub fn select_heal_window<T: crate::series::DataPoint>(
    pulled: Vec<T>,
    last_seen_ms: i64,
) -> Vec<T> {
    let mut filtered: Vec<T> = pulled.into_iter().filter(|p| p.timestamp_ms() > last_seen_ms).collect();
    filtered.sort_by_key(|p| p.timestamp_ms());
    // Dedup by ts (cheap & sufficient for trades+klines; trades may share ts).
    filtered.dedup_by_key(|p| p.timestamp_ms());
    filtered
}

/// Convert a kline interval string to a Duration. Supports the canonical
/// Binance-style suffixes: s (seconds), m (minutes), h (hours), d (days),
/// w (weeks). Returns None for malformed input.
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

/// REST pull for gap-heal — returns raw recent trades (oldest→newest as
/// returned by the exchange). The forwarder calls [`select_heal_window`] to
/// filter to the missing window.
pub async fn heal_trades(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    _since_ms: i64,
    max_records: usize,
) -> Vec<TradePoint> {
    crate::backfill::trades_recent(hub, exchange, account, symbol, max_records).await
}

/// REST pull for gap-heal — returns raw recent klines. Filter via
/// [`select_heal_window`].
pub async fn heal_klines(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    _since_open_time_ms: i64,
    max_records: usize,
) -> Vec<BarPoint> {
    crate::backfill::klines_recent(hub, exchange, account, symbol, interval, max_records).await
}
