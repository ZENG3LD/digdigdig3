//! Polling subscription layer for `digdigdig3-station`.
//!
//! A *polling subscription* is a Station-internal actor that periodically calls
//! a REST endpoint and emits events through the same broadcast pipeline as WS
//! forwarders. Consumers see no difference — they call `handle.recv().await`
//! and receive interleaved `Event` values regardless of the underlying source.
//!
//! Two concrete [`PollSource`] impls ship here:
//!
//! - [`LongShortRatioPoll`] — calls `get_long_short_ratio_history` on Binance /
//!   Bybit / OKX every 5 minutes, normalising the `period` format divergence
//!   (`"5m"` vs `"5min"`) internally.
//! - [`DeribitHvPoll`] — calls `get_historical_volatility` on Deribit every
//!   hour.
//!
//! The public entry point for the station dispatch loop is [`is_poll_only`] +
//! [`spawn_poller`]; they are `pub(crate)` and called from `station.rs`.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use tokio::sync::{broadcast, oneshot};

use crate::data::{HistoricalVolatilityPoint, LongShortRatioPoint};
use crate::series::{DataPoint, DiskStore, PollSpec, SeriesKey};
use crate::station::{Station, EventFrom};
use crate::subscription::Event;
use crate::{Result, StationError};

// ─────────────────────────────────────────────────────────────────────────────
// PollSource trait
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll contract for one `(kind, exchange, symbol)` combination.
///
/// `poll` is called on every interval tick and returns all records the exchange
/// has available (not just the newest one), allowing the caller to dedup by
/// `timestamp_ms` and emit only genuinely new points.
///
/// The trait uses stable AFIT (available since Rust 1.75). No `async_trait`
/// macro is needed.
pub trait PollSource<T: DataPoint>: Send + Sync + 'static {
    /// Fetch recent data points from the exchange.
    ///
    /// Implementations should request the last ~500 buckets with no
    /// `start_time` filter. This gives the poller a built-in warm-start on
    /// the first tick without a separate backfill path.
    ///
    /// Return `Err(String)` on any REST failure. The caller logs + retries on
    /// the next tick without exiting the actor.
    fn poll(
        &self,
        hub: Arc<ExchangeHub>,
        exchange: ExchangeId,
        account_type: AccountType,
        symbol: String,
    ) -> impl std::future::Future<Output = Result<Vec<T>>> + Send;

    /// Polling cadence — taken from [`PollSpec`] at construction time.
    fn cadence(&self) -> Duration;
}

// ─────────────────────────────────────────────────────────────────────────────
// spawn_poller actor
// ─────────────────────────────────────────────────────────────────────────────

/// Spawn a poll actor for `key`. Mirrors `spawn_forwarder` in structure but is
/// driven by a `tokio::time::interval` instead of a WS event stream.
///
/// On each tick:
/// 1. Calls `source.poll(...)`.
/// 2. For each returned point with `timestamp_ms > last_emitted_ms`: appends to
///    disk, emits on `bcast_tx`.
/// 3. On consecutive REST errors ≥ 10: logs "poller degraded", keeps retrying.
/// 4. On shutdown signal: flushes disk, removes mux entry if no consumers remain.
pub(crate) fn spawn_poller<T, S>(
    station: &Station,
    key: &SeriesKey,
    source: S,
    poll_spec: PollSpec,
    bcast_tx: broadcast::Sender<Event>,
    shutdown_rx: oneshot::Receiver<()>,
    symbol_label: String,
) where
    T: DataPoint + 'static,
    S: PollSource<T>,
    Event: EventFrom<T>,
{
    let inner = station.inner.clone();
    let key = key.clone();
    let storage_root = inner.storage_root.clone();
    let persistence = inner.persistence.clone();
    let exchange = key.exchange;
    let hub = inner.hub.clone();
    let account_type = key.account_type;
    let raw_symbol = key.symbol.clone();

    tokio::spawn(async move {
        // Open disk store if persistence is enabled for this kind.
        let mut disk: Option<DiskStore<T>> = None;
        if persistence.is_enabled_for(&key.kind) {
            match DiskStore::<T>::new(&storage_root, key.clone()).await {
                Ok(store) => disk = Some(store),
                Err(e) => tracing::warn!(?e, ?key, "poll: disk store open failed"),
            }
        }

        // last_emitted_ms: dedup fence. Points at or below this ts are skipped.
        let mut last_emitted_ms: i64 = 0;

        // Warm-start: emit disk tail before the first live poll tick.
        if let Some(d) = disk.as_ref() {
            if let Ok(tail) = d.read_tail(500).await {
                for p in &tail {
                    let _ = bcast_tx
                        .send(Event::from_point(exchange, &symbol_label, &key.kind, p.clone()));
                    last_emitted_ms = last_emitted_ms.max(p.timestamp_ms());
                }
            }
        }

        // First-tick jitter: sleep a deterministic pseudo-random offset so that
        // N symbols × M exchanges don't all fire at the same wall-clock second.
        // Uses no `rand` crate — symbol bytes are a sufficient seed.
        {
            let jitter_max_ms = (poll_spec.cadence.as_millis() as u64)
                .saturating_mul(poll_spec.jitter_pct as u64)
                / 100;
            if jitter_max_ms > 0 {
                let seed = key
                    .symbol
                    .as_bytes()
                    .iter()
                    .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
                // Map seed into [0, jitter_max_ms].
                let sleep_ms = seed % jitter_max_ms.max(1);
                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
            }
        }

        let mut interval = tokio::time::interval(source.cadence());
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut consecutive_errors: u32 = 0;
        const DEGRADE_THRESHOLD: u32 = 10;

        let mut shutdown_rx = shutdown_rx;

        loop {
            tokio::select! {
                biased;
                _ = &mut shutdown_rx => break,
                _ = interval.tick() => {}
            }

            let pts = match source.poll(hub.clone(), exchange, account_type, raw_symbol.clone()).await {
                Ok(v) => {
                    consecutive_errors = 0;
                    v
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors == 1 || consecutive_errors == DEGRADE_THRESHOLD {
                        tracing::warn!(
                            target: "dig3::poll",
                            ?key,
                            consecutive_errors,
                            error = %e,
                            "poller REST error{}",
                            if consecutive_errors >= DEGRADE_THRESHOLD { " — poller degraded" } else { "" }
                        );
                    }
                    // Keep retrying — never exit the actor on REST error.
                    continue;
                }
            };

            // Dedup + emit. Only points strictly newer than last_emitted_ms.
            for pt in pts {
                if pt.timestamp_ms() <= last_emitted_ms {
                    continue; // already delivered
                }
                if let Some(d) = disk.as_mut() {
                    if let Err(e) = d.append(&pt) {
                        tracing::warn!(?e, "poll: disk append failed");
                    }
                }
                last_emitted_ms = pt.timestamp_ms();
                let _ =
                    bcast_tx.send(Event::from_point(exchange, &symbol_label, &key.kind, pt));
            }
        }

        // Flush disk on graceful shutdown.
        if let Some(mut d) = disk {
            let _ = d.flush().await;
        }

        // Mux cleanup — same pattern as spawn_forwarder.
        let still_consumers = inner
            .muxes
            .get(&key)
            .map(|m| m.consumers.load(Ordering::SeqCst))
            .unwrap_or(0);
        if still_consumers == 0 {
            inner.muxes.remove(&key);
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// LongShortRatioPoll
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll source for `Kind::LongShortRatio`.
///
/// Calls `get_long_short_ratio_history` on Binance / Bybit / OKX.
/// Normalises the `period` format divergence internally:
/// - Binance → `"5m"`
/// - Bybit   → `"5min"`
/// - OKX     → `"5m"`
pub struct LongShortRatioPoll {
    cadence: Duration,
}

impl LongShortRatioPoll {
    pub fn new() -> Self {
        Self {
            cadence: Duration::from_secs(5 * 60),
        }
    }

    /// Exchange-native period string for the 5-minute bucket.
    fn period_for(exchange: ExchangeId) -> &'static str {
        match exchange {
            ExchangeId::Bybit => "5min",
            _ => "5m", // Binance, OKX, and all others
        }
    }
}

impl Default for LongShortRatioPoll {
    fn default() -> Self {
        Self::new()
    }
}

impl PollSource<LongShortRatioPoint> for LongShortRatioPoll {
    fn poll(
        &self,
        hub: Arc<ExchangeHub>,
        exchange: ExchangeId,
        account_type: AccountType,
        symbol: String,
    ) -> impl std::future::Future<Output = Result<Vec<LongShortRatioPoint>>> + Send {
        let period = Self::period_for(exchange);
        async move {
            let connector = hub
                .rest(exchange)
                .ok_or_else(|| StationError::Core("REST connector missing for LSR poll".into()))?;
            let raw = connector
                .get_long_short_ratio_history(
                    symbol.as_str().into(),
                    period,
                    None,
                    None,
                    Some(500),
                    account_type,
                )
                .await
                .map_err(|e| StationError::Core(format!("poll LSR: {e}")))?;
            Ok(raw
                .into_iter()
                .map(|r| LongShortRatioPoint {
                    ts_ms: r.timestamp,
                    ratio: r.ratio.unwrap_or_else(|| {
                        if r.short_ratio > 0.0 {
                            r.long_ratio / r.short_ratio
                        } else {
                            1.0
                        }
                    }),
                    long_pct: r.long_ratio,
                    short_pct: r.short_ratio,
                })
                .collect())
        }
    }

    fn cadence(&self) -> Duration {
        self.cadence
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DeribitHvPoll
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll source for `Kind::HistoricalVolatility` on Deribit.
///
/// The `symbol` field of the `SeriesKey` is used as the `currency` parameter
/// (e.g. `"BTC"`, `"ETH"`). Use `SubscriptionSet::add_raw` with currency
/// strings directly.
pub struct DeribitHvPoll {
    cadence: Duration,
}

impl DeribitHvPoll {
    pub fn new() -> Self {
        Self {
            cadence: Duration::from_secs(60 * 60),
        }
    }
}

impl Default for DeribitHvPoll {
    fn default() -> Self {
        Self::new()
    }
}

impl PollSource<HistoricalVolatilityPoint> for DeribitHvPoll {
    fn poll(
        &self,
        hub: Arc<ExchangeHub>,
        _exchange: ExchangeId,
        _account_type: AccountType,
        symbol: String, // used as `currency`
    ) -> impl std::future::Future<Output = Result<Vec<HistoricalVolatilityPoint>>> + Send {
        async move {
            let connector = hub
                .rest(ExchangeId::Deribit)
                .ok_or_else(|| StationError::Core("Deribit REST connector missing for HV poll".into()))?;
            let raw = connector
                .get_historical_volatility(&symbol)
                .await
                .map_err(|e| StationError::Core(format!("poll HV: {e}")))?;
            Ok(raw
                .into_iter()
                .map(|h| HistoricalVolatilityPoint {
                    ts_ms: h.timestamp,
                    volatility: h.volatility,
                })
                .collect())
        }
    }

    fn cadence(&self) -> Duration {
        self.cadence
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Factory helpers (used in station.rs acquire_or_spawn_polled)
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `Some(LongShortRatioPoll)` for exchanges that support LSR REST.
/// Returns `None` for exchanges that don't, which causes `acquire_or_spawn`
/// to return `StationError::StreamNotSupported`.
pub(crate) fn lsr_poll_source(exchange: ExchangeId) -> Option<LongShortRatioPoll> {
    match exchange {
        ExchangeId::Binance | ExchangeId::Bybit | ExchangeId::OKX => {
            Some(LongShortRatioPoll::new())
        }
        _ => None,
    }
}

/// Returns `Some(DeribitHvPoll)` for Deribit only.
pub(crate) fn hv_poll_source(exchange: ExchangeId) -> Option<DeribitHvPoll> {
    match exchange {
        ExchangeId::Deribit => Some(DeribitHvPoll::new()),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::Kind;

    #[test]
    fn lsr_poll_source_allow_list() {
        // Supported exchanges return Some(...)
        assert!(lsr_poll_source(ExchangeId::Binance).is_some());
        assert!(lsr_poll_source(ExchangeId::Bybit).is_some());
        assert!(lsr_poll_source(ExchangeId::OKX).is_some());
        // Unsupported exchanges return None
        assert!(lsr_poll_source(ExchangeId::Deribit).is_none());
        assert!(lsr_poll_source(ExchangeId::Kraken).is_none());
    }

    #[test]
    fn hv_poll_source_allow_list() {
        // Only Deribit supported
        assert!(hv_poll_source(ExchangeId::Deribit).is_some());
        assert!(hv_poll_source(ExchangeId::Binance).is_none());
        assert!(hv_poll_source(ExchangeId::Bybit).is_none());
        assert!(hv_poll_source(ExchangeId::OKX).is_none());
    }

    #[test]
    fn lsr_period_for_exchange() {
        assert_eq!(LongShortRatioPoll::period_for(ExchangeId::Bybit), "5min");
        assert_eq!(LongShortRatioPoll::period_for(ExchangeId::Binance), "5m");
        assert_eq!(LongShortRatioPoll::period_for(ExchangeId::OKX), "5m");
    }

    #[test]
    fn lsr_poll_cadence() {
        assert_eq!(LongShortRatioPoll::new().cadence(), Duration::from_secs(300));
    }

    #[test]
    fn hv_poll_cadence() {
        assert_eq!(DeribitHvPoll::new().cadence(), Duration::from_secs(3600));
    }

    #[test]
    fn kind_lsr_poll_spec() {
        let spec = Kind::LongShortRatio.is_poll_only().unwrap();
        assert_eq!(spec.cadence, Duration::from_secs(300));
        assert_eq!(spec.jitter_pct, 10);
    }

    #[test]
    fn kind_hv_poll_spec() {
        let spec = Kind::HistoricalVolatility.is_poll_only().unwrap();
        assert_eq!(spec.cadence, Duration::from_secs(3600));
        assert_eq!(spec.jitter_pct, 5);
    }
}
