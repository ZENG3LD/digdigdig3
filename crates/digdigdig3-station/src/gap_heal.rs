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

/// REST helper: pull recent trades after `since_ms` (best-effort — exchange
/// REST `get_recent_trades` returns the most recent N, not a timestamp window,
/// so the caller filters by `ts > since_ms`).
pub async fn heal_trades(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    since_ms: i64,
    max_records: usize,
) -> Vec<TradePoint> {
    let pulled = crate::backfill::trades_recent(hub, exchange, account, symbol, max_records).await;
    pulled.into_iter().filter(|p| p.ts_ms > since_ms).collect()
}

/// REST helper: pull klines covering `since_open_time_ms .. now`. Filters out
/// bars we've already seen.
pub async fn heal_klines(
    hub: &Arc<ExchangeHub>,
    exchange: ExchangeId,
    account: AccountType,
    symbol: &str,
    interval: &str,
    since_open_time_ms: i64,
    max_records: usize,
) -> Vec<BarPoint> {
    let pulled =
        crate::backfill::klines_recent(hub, exchange, account, symbol, interval, max_records).await;
    pulled.into_iter().filter(|p| p.open_time > since_open_time_ms).collect()
}
