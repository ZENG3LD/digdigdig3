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
//!
//! ## Wasm note
//!
//! `PollSource<T>`, `LongShortRatioPoll`, and `DeribitHvPoll` are available on
//! wasm32. However, the concrete `impl PollSource` blocks and `spawn_poller`
//! are native-only because they rely on `tokio::time::interval` (full timer
//! feature) which is not available on wasm32. On wasm, Station returns
//! `StationError::StreamNotSupported` for poll-only kinds before reaching
//! `spawn_poller`.

use std::sync::Arc;
use std::time::Duration;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};

use crate::series::DataPoint;
use crate::Result;

// Items only needed on native (impl PollSource + spawn_poller).
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::Ordering;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::{broadcast, oneshot};
#[cfg(not(target_arch = "wasm32"))]
use crate::data::{
    BasisPoint, FundingSettlementPoint, HistoricalVolatilityPoint, LiquidationBucketPoint,
    LongShortRatioPoint, TakerVolumePoint,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::series::{DiskStore, PollSpec, SeriesKey};
#[cfg(not(target_arch = "wasm32"))]
use crate::subscription::Event;
#[cfg(not(target_arch = "wasm32"))]
use crate::StationError;
#[cfg(not(target_arch = "wasm32"))]
use crate::station::{Station, EventFrom};

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
///
/// # Wasm note
///
/// The native `spawn_poller` requires the returned future to be `Send`
/// (for `tokio::spawn`). Concrete implementations must therefore return `Send`
/// futures on native. On wasm `spawn_poller` is not compiled, so no `Send`
/// requirement is imposed — the REST future may be `!Send`.
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
// spawn_poller actor (native-only — tokio::time not available on wasm32)
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
///
/// Native-only: uses `tokio::time::interval` + `tokio::spawn`. On wasm,
/// `Station::acquire_or_spawn_polled` is `#[cfg(not(target_arch = "wasm32"))]`
/// so this function is never reachable from the wasm build.
#[cfg(not(target_arch = "wasm32"))]
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
                        .send(Event::from_point(exchange, key.account_type, &symbol_label, &key.kind, p.clone()));
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
                    bcast_tx.send(Event::from_point(exchange, key.account_type, &symbol_label, &key.kind, pt));
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
// LongShortRatioPoll (native-only)
// ─────────────────────────────────────────────────────────────────────────────
//
// Concrete `impl PollSource` requires `Send` futures from `hub.rest(...)`.
// On wasm, REST futures are `!Send` (browser fetch). Since `spawn_poller` is
// native-only, the struct + its impl are native-only too. Custom wasm poll
// sources can still implement the `PollSource` trait directly.

/// REST poll source for `Kind::LongShortRatio`.
///
/// Calls `get_long_short_ratio_history` on Binance / Bybit / OKX.
/// Normalises the `period` format divergence internally:
/// - Binance → `"5m"`
/// - Bybit   → `"5min"`
/// - OKX     → `"5m"`
#[cfg(not(target_arch = "wasm32"))]
pub struct LongShortRatioPoll {
    cadence: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
impl Default for LongShortRatioPoll {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
// DeribitHvPoll (native-only — same rationale as LongShortRatioPoll)
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll source for `Kind::HistoricalVolatility` on Deribit.
///
/// The `symbol` field of the `SeriesKey` is used as the `currency` parameter
/// (e.g. `"BTC"`, `"ETH"`). Use `SubscriptionSet::add_raw` with currency
/// strings directly.
#[cfg(not(target_arch = "wasm32"))]
pub struct DeribitHvPoll {
    cadence: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl DeribitHvPoll {
    pub fn new() -> Self {
        Self {
            cadence: Duration::from_secs(60 * 60),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for DeribitHvPoll {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
///
/// Native-only: called from `acquire_or_spawn_polled` which is itself native-only.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn lsr_poll_source(exchange: ExchangeId) -> Option<LongShortRatioPoll> {
    match exchange {
        ExchangeId::Binance | ExchangeId::Bybit | ExchangeId::OKX => {
            Some(LongShortRatioPoll::new())
        }
        _ => None,
    }
}

/// Returns `Some(DeribitHvPoll)` for Deribit only.
///
/// Native-only: called from `acquire_or_spawn_polled` which is itself native-only.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn hv_poll_source(exchange: ExchangeId) -> Option<DeribitHvPoll> {
    match exchange {
        ExchangeId::Deribit => Some(DeribitHvPoll::new()),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BasisHistoryPoll (native-only — same rationale as LongShortRatioPoll)
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll source for `Kind::Basis` on exchanges that expose a native basis
/// history endpoint (Binance, HTX, Bybit).
///
/// Calls `get_basis_history` with a 60-second cadence; fetches the last 500
/// buckets so the poller provides a built-in warm-start on the first tick.
#[cfg(not(target_arch = "wasm32"))]
pub struct BasisHistoryPoll {
    /// Exchange-native contract-type / period string (e.g. `"1h"`, `"5m"`).
    pub period: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl BasisHistoryPoll {
    pub fn new(period: impl Into<String>) -> Self {
        Self { period: period.into() }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PollSource<BasisPoint> for BasisHistoryPoll {
    fn poll(
        &self,
        hub: Arc<ExchangeHub>,
        exchange: ExchangeId,
        account_type: AccountType,
        symbol: String,
    ) -> impl std::future::Future<Output = Result<Vec<BasisPoint>>> + Send {
        let period = self.period.clone();
        async move {
            let connector = hub
                .rest(exchange)
                .ok_or_else(|| StationError::Core("REST connector missing for basis history poll".into()))?;
            let raw = connector
                .get_basis_history(
                    symbol.as_str().into(),
                    &period,
                    None,
                    None,
                    Some(500),
                    account_type,
                )
                .await
                .map_err(|e| StationError::Core(format!("poll basis history: {e}")))?;
            Ok(raw
                .into_iter()
                .map(|b| BasisPoint {
                    ts_ms: b.timestamp,
                    value: b.basis,
                    mark:  b.futures_price.unwrap_or(f64::NAN),
                    index: b.index_price.unwrap_or(f64::NAN),
                })
                .collect())
        }
    }

    fn cadence(&self) -> Duration {
        // Basis history buckets are typically 1 h wide; 60 s cadence keeps
        // the disk tail fresh with minimal REST cost.
        Duration::from_secs(60)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FundingHistoryPoll (native-only — same rationale as LongShortRatioPoll)
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll source for `Kind::FundingSettlement` on exchanges that expose a
/// native funding-rate history endpoint.
///
/// Calls `get_funding_rate_history` with a 5-minute cadence; funding cycles
/// are 1 h – 8 h, so 5 min keeps the tail fresh with low REST cost.
#[cfg(not(target_arch = "wasm32"))]
pub struct FundingHistoryPoll;

#[cfg(not(target_arch = "wasm32"))]
impl PollSource<FundingSettlementPoint> for FundingHistoryPoll {
    fn poll(
        &self,
        hub: Arc<ExchangeHub>,
        exchange: ExchangeId,
        account_type: AccountType,
        symbol: String,
    ) -> impl std::future::Future<Output = Result<Vec<FundingSettlementPoint>>> + Send {
        async move {
            let connector = hub
                .rest(exchange)
                .ok_or_else(|| StationError::Core("REST connector missing for funding history poll".into()))?;
            let raw = connector
                .get_funding_rate_history(
                    symbol.as_str().into(),
                    None,
                    None,
                    Some(500),
                    account_type,
                )
                .await
                .map_err(|e| StationError::Core(format!("poll funding history: {e}")))?;
            Ok(raw
                .into_iter()
                .map(|f| FundingSettlementPoint {
                    ts_ms: f.timestamp,
                    settled_rate: f.rate,
                    settlement_time: f.next_funding_time.unwrap_or(f.timestamp),
                })
                .collect())
        }
    }

    fn cadence(&self) -> Duration {
        Duration::from_secs(5 * 60)
    }
}

/// Returns `Some(BasisHistoryPoll)` for exchanges with a native basis-history
/// REST endpoint (`ConnectorCapabilities::has_basis_history`).
///
/// Native-only: called from `acquire_or_spawn_polled_native` which is itself
/// native-only.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn basis_poll_source(hub: &ExchangeHub, exchange: ExchangeId) -> Option<BasisHistoryPoll> {
    let caps = hub.capabilities(exchange)?;
    if caps.has_basis_history {
        Some(BasisHistoryPoll::new("1h"))
    } else {
        None
    }
}

/// Returns `Some(FundingHistoryPoll)` for exchanges with a native
/// funding-rate history REST endpoint
/// (`ConnectorCapabilities::has_funding_rate_history`).
///
/// Native-only: called from `acquire_or_spawn_polled_native` which is itself
/// native-only.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn funding_poll_source(hub: &ExchangeHub, exchange: ExchangeId) -> Option<FundingHistoryPoll> {
    let caps = hub.capabilities(exchange)?;
    if caps.has_funding_rate_history {
        Some(FundingHistoryPoll)
    } else {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TakerVolumePoll (native-only — same rationale as LongShortRatioPoll)
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll source for `Kind::TakerVolume`.
///
/// Calls `get_taker_volume_history` on exchanges that support it.
/// Fetches the last 500 5-minute buckets on each tick, deduplicating by
/// `timestamp_ms` inside the poller.
#[cfg(not(target_arch = "wasm32"))]
pub struct TakerVolumePoll {
    cadence: Duration,
    period: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl TakerVolumePoll {
    pub fn new(period: impl Into<String>) -> Self {
        Self {
            cadence: Duration::from_secs(5 * 60),
            period: period.into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PollSource<TakerVolumePoint> for TakerVolumePoll {
    fn poll(
        &self,
        hub: Arc<ExchangeHub>,
        exchange: ExchangeId,
        account_type: AccountType,
        symbol: String,
    ) -> impl std::future::Future<Output = Result<Vec<TakerVolumePoint>>> + Send {
        let period = self.period.clone();
        async move {
            let connector = hub
                .rest(exchange)
                .ok_or_else(|| StationError::Core("REST connector missing for taker_volume poll".into()))?;
            let raw = connector
                .get_taker_volume_history(
                    symbol.as_str().into(),
                    &period,
                    None,
                    None,
                    Some(500),
                    account_type,
                )
                .await
                .map_err(|e| StationError::Core(format!("poll taker_volume: {e}")))?;
            Ok(raw
                .into_iter()
                .map(|t| TakerVolumePoint {
                    ts_ms: t.timestamp,
                    buy_volume: t.buy_volume,
                    sell_volume: t.sell_volume,
                    buy_sell_ratio: t.buy_sell_ratio.unwrap_or(f64::NAN),
                    long_taker_size: t.long_taker_size.unwrap_or(f64::NAN),
                    short_taker_size: t.short_taker_size.unwrap_or(f64::NAN),
                })
                .collect())
        }
    }

    fn cadence(&self) -> Duration {
        self.cadence
    }
}

/// Returns `Some(TakerVolumePoll)` for exchanges with a taker-volume history
/// REST endpoint (`ConnectorCapabilities::has_taker_volume_history`).
///
/// Native-only: called from `acquire_or_spawn_polled` which is itself native-only.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn taker_volume_poll_source(hub: &ExchangeHub, exchange: ExchangeId) -> Option<TakerVolumePoll> {
    let caps = hub.capabilities(exchange)?;
    if caps.has_taker_volume_history {
        Some(TakerVolumePoll::new("5m"))
    } else {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LiquidationBucketPoll (native-only — same rationale as LongShortRatioPoll)
// ─────────────────────────────────────────────────────────────────────────────

/// REST poll source for `Kind::LiquidationBucket`.
///
/// Calls `get_liquidation_bucket_history` on exchanges that expose bucketed
/// liquidation aggregates (e.g. GateIO `contract_stats`). Fetches the last
/// 500 5-minute buckets on each tick.
#[cfg(not(target_arch = "wasm32"))]
pub struct LiquidationBucketPoll {
    cadence: Duration,
    period: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl LiquidationBucketPoll {
    pub fn new(period: impl Into<String>) -> Self {
        Self {
            cadence: Duration::from_secs(5 * 60),
            period: period.into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PollSource<LiquidationBucketPoint> for LiquidationBucketPoll {
    fn poll(
        &self,
        hub: Arc<ExchangeHub>,
        exchange: ExchangeId,
        account_type: AccountType,
        symbol: String,
    ) -> impl std::future::Future<Output = Result<Vec<LiquidationBucketPoint>>> + Send {
        let period = self.period.clone();
        async move {
            let connector = hub
                .rest(exchange)
                .ok_or_else(|| StationError::Core("REST connector missing for liquidation_bucket poll".into()))?;
            let raw = connector
                .get_liquidation_bucket_history(
                    symbol.as_str().into(),
                    &period,
                    None,
                    None,
                    Some(500),
                    account_type,
                )
                .await
                .map_err(|e| StationError::Core(format!("poll liquidation_bucket: {e}")))?;
            Ok(raw
                .into_iter()
                .map(|b| LiquidationBucketPoint {
                    ts_ms: b.timestamp,
                    long_liq_size: b.long_liq_size.unwrap_or(f64::NAN),
                    short_liq_size: b.short_liq_size.unwrap_or(f64::NAN),
                    long_liq_amount: b.long_liq_amount.unwrap_or(f64::NAN),
                    short_liq_amount: b.short_liq_amount.unwrap_or(f64::NAN),
                    long_liq_usd: b.long_liq_usd.unwrap_or(f64::NAN),
                    short_liq_usd: b.short_liq_usd.unwrap_or(f64::NAN),
                })
                .collect())
        }
    }

    fn cadence(&self) -> Duration {
        self.cadence
    }
}

/// Returns `Some(LiquidationBucketPoll)` for exchanges with a liquidation-bucket
/// history REST endpoint (`ConnectorCapabilities::has_liquidation_bucket_history`).
///
/// Native-only: called from `acquire_or_spawn_polled` which is itself native-only.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn liquidation_bucket_poll_source(hub: &ExchangeHub, exchange: ExchangeId) -> Option<LiquidationBucketPoll> {
    let caps = hub.capabilities(exchange)?;
    if caps.has_liquidation_bucket_history {
        Some(LiquidationBucketPoll::new("5m"))
    } else {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::series::Kind;

    // Wasm-safe tests — only use Kind (no native-only polling types).
    #[test]
    fn kind_lsr_poll_spec() {
        let spec = Kind::LongShortRatio.is_poll_only().unwrap();
        assert_eq!(spec.cadence, std::time::Duration::from_secs(300));
        assert_eq!(spec.jitter_pct, 10);
    }

    #[test]
    fn kind_hv_poll_spec() {
        let spec = Kind::HistoricalVolatility.is_poll_only().unwrap();
        assert_eq!(spec.cadence, std::time::Duration::from_secs(3600));
        assert_eq!(spec.jitter_pct, 5);
    }

    // Native-only tests — use concrete poll types and factory functions.
    #[cfg(not(target_arch = "wasm32"))]
    mod native {
        use super::super::*;

        #[test]
        fn lsr_poll_cadence() {
            assert_eq!(LongShortRatioPoll::new().cadence(), Duration::from_secs(300));
        }

        #[test]
        fn hv_poll_cadence() {
            assert_eq!(DeribitHvPoll::new().cadence(), Duration::from_secs(3600));
        }

        #[test]
        fn lsr_poll_source_allow_list() {
            assert!(lsr_poll_source(ExchangeId::Binance).is_some());
            assert!(lsr_poll_source(ExchangeId::Bybit).is_some());
            assert!(lsr_poll_source(ExchangeId::OKX).is_some());
            assert!(lsr_poll_source(ExchangeId::Deribit).is_none());
            assert!(lsr_poll_source(ExchangeId::Kraken).is_none());
        }

        #[test]
        fn hv_poll_source_allow_list() {
            assert!(hv_poll_source(ExchangeId::Deribit).is_some());
            assert!(hv_poll_source(ExchangeId::Binance).is_none());
            assert!(hv_poll_source(ExchangeId::Bybit).is_none());
            assert!(hv_poll_source(ExchangeId::OKX).is_none());
        }

        #[test]
        fn basis_history_poll_cadence() {
            assert_eq!(
                BasisHistoryPoll::new("1h").cadence(),
                Duration::from_secs(60)
            );
        }

        #[test]
        fn funding_history_poll_cadence() {
            assert_eq!(
                FundingHistoryPoll.cadence(),
                Duration::from_secs(300)
            );
        }

        #[test]
        fn lsr_period_for_exchange() {
            assert_eq!(LongShortRatioPoll::period_for(ExchangeId::Bybit), "5min");
            assert_eq!(LongShortRatioPoll::period_for(ExchangeId::Binance), "5m");
            assert_eq!(LongShortRatioPoll::period_for(ExchangeId::OKX), "5m");
        }

        #[test]
        fn taker_volume_poll_cadence() {
            assert_eq!(TakerVolumePoll::new("5m").cadence(), Duration::from_secs(300));
        }

        #[test]
        fn liquidation_bucket_poll_cadence() {
            assert_eq!(LiquidationBucketPoll::new("5m").cadence(), Duration::from_secs(300));
        }

        #[test]
        fn taker_volume_poll_source_allow_list() {
            // Factory returns None for ExchangeId values with no hub instance at unit-test time.
            // Smoke-test: verify the function is callable and returns Option.
            let _: fn(&ExchangeHub, ExchangeId) -> Option<TakerVolumePoll> = taker_volume_poll_source;
        }

        #[test]
        fn liquidation_bucket_poll_source_allow_list() {
            let _: fn(&ExchangeHub, ExchangeId) -> Option<LiquidationBucketPoll> = liquidation_bucket_poll_source;
        }
    }
}
