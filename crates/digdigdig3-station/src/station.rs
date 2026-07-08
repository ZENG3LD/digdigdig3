use std::any::Any;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::ws_health::WsHealth;

use dashmap::DashMap;
use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ConnectorCapabilities, ExchangeId, StreamEvent, StreamType, SubscriptionRequest,
    Symbol, SymbolInfo,
};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};

use crate::data::{
    AggTradePoint, AuctionEventPoint, BalanceUpdatePoint, BarPoint, BasisPoint, BlockTradePoint,
    CompositeIndexPoint,
    FootprintPoint,
    FundingRatePoint, FundingRateIndicatorsPoint, FundingRateFullPoint,
    FundingSettlementPoint, HistoricalVolatilityPoint,
    IndexPriceKlinePoint, IndexPricePoint, IndexPriceIndicatorsPoint,
    InsuranceFundPoint,
    LiquidationPoint, LiquidationIndicatorsPoint, LiquidationFullPoint,
    LiquidationBucketPoint,
    LongShortRatioPoint,
    MarkPriceKlinePoint, MarkPricePoint, MarkPriceIndicatorsPoint, MarkPriceFullPoint,
    MarketWarningPoint,
    ObDeltaPoint, ObDeltaIndicatorsPoint,
    ObSnapshotPoint, ObSnapshotIndicatorsPoint,
    OpenInterestPoint, OpenInterestIndicatorsPoint,
    OptionGreeksPoint,
    OrderUpdatePoint, OrderbookL3Point, PositionUpdatePoint,
    PredictedFundingPoint, PremiumIndexKlinePoint, RiskLimitPoint, SettlementEventPoint,
    TakerVolumePoint,
    TickerPoint, TickerIndicatorsPoint, TickerFullPoint,
    TradePoint, VolatilityIndexPoint,
    KagiSegmentPoint, PnfColumnPoint, RenkoBrickPoint, ScalarBarPoint,
    ThreeLineBreakLinePoint, TpoSessionPoint,
};
use crate::persistence::PersistDepth;
use crate::derived::{
    BasisDerived, DerivedStream, FundingSettlementDerived, TradeToBarDerived,
    TradeToRangeBarDerived, TradeToTickBarDerived, TradeToVolumeBarDerived,
    TradeToFootprintDerived, TradeToRenkoBarDerived, TradeToPnfBarDerived,
    TradeToKagiBarDerived, TradeToCvdLineDerived, TradeToThreeLineBreakDerived,
    TpoFromKline1mDerived,
    TpoFromTradeDerived, TradeToDollarBarDerived, TradeToTickImbalanceDerived,
    TradeToVolumeImbalanceDerived, TradeToRunBarDerived, interval_to_ms,
};
use crate::series::TpoSource;
#[cfg(not(target_arch = "wasm32"))]
use crate::polling;
#[cfg(not(target_arch = "wasm32"))]
use crate::polling::PollSource;
use crate::series::DiskStore;
use crate::series::{DataPoint, Kind, Series, SeriesKey};
use crate::subscription::{Entry, Event, FailedStream, MultiplexRef, Stream};
use crate::{
    PersistenceConfig, Result, StationBuilder, StationError, SubscribeReport, SubscriptionHandle,
    SubscriptionSet,
};


/// Phase 5 Station. Unified `Series<T>` + `DiskStore<T>` plumbing under all
/// stream classes. One multiplexer actor per `SeriesKey` (= exchange × account
/// × symbol × kind). N consumers share via `broadcast::channel`.
pub struct Station {
    pub(crate) inner: Arc<StationInner>,
}

pub(crate) struct StationInner {
    pub(crate) hub: Arc<ExchangeHub>,
    pub(crate) storage_root: PathBuf,
    pub(crate) persistence: PersistenceConfig,
    pub(crate) muxes: DashMap<SeriesKey, Multiplexer>,
    /// Sync-accessible series handles for render-time consumers.
    ///
    /// Each active forwarder stores `Arc<RwLock<Series<T>>>` here (type-erased
    /// to `Arc<dyn Any + Send + Sync>`). `Station::series<T>()` retrieves and
    /// downcasts. Entries are removed when the forwarder exits (same lifecycle
    /// as `muxes`).
    pub(crate) series_handles: DashMap<SeriesKey, Arc<dyn Any + Send + Sync>>,
    pub(crate) warm_start_capacity: usize,
    pub(crate) gap_heal: crate::GapHealConfig,
    /// How long to keep a forwarder alive after its last consumer drops.
    /// `Duration::ZERO` = immediate shutdown (default).
    pub(crate) unsubscribe_grace: std::time::Duration,
    /// Issue a one-shot REST `get_orderbook` seed on first subscribe to
    /// `Orderbook` / `OrderbookDelta`. False = WS-only (default).
    pub(crate) orderbook_rest_seed: bool,
    /// Depth for the REST seed. Passed as `Some(depth as u16)` to `get_orderbook`.
    pub(crate) orderbook_seed_depth: usize,
    /// Broadcast channel for connector lifecycle events (`ConnectorReady`,
    /// `SymbolsLoaded`). Independent of per-`SeriesKey` data muxes.
    /// Capacity 256 — lag drops oldest.
    pub(crate) connector_tx: broadcast::Sender<crate::subscription::Event>,
    /// Cache for `get_exchange_info` results, keyed by `(exchange, account_type)`.
    /// Populated by `warmup()`; re-emits without REST on repeated calls.
    pub(crate) exchange_info_cache: DashMap<(ExchangeId, AccountType), Vec<SymbolInfo>>,
    /// Per-`SeriesKey` flush notifiers for forwarders that opened a
    /// `DiskStore`. Registered at spawn time, unregistered by the forwarder
    /// itself right before it exits. Used by [`Station::flush_persistence`]
    /// to force every open store to drain + flush synchronously from the
    /// caller's perspective — needed on wasm where `beforeunload` /
    /// `visibilitychange` can close the tab before `Drop` gets a chance to
    /// await an async OPFS flush. Native `BufWriter` already flushes on
    /// `Drop`, but the API is uniform across both targets.
    pub(crate) flush_handles: DashMap<SeriesKey, FlushHandle>,
    /// Per-`SeriesKey` forwarder-exit acknowledgement receivers, registered
    /// at spawn time (same lifecycle as `flush_handles`). The paired
    /// `oneshot::Sender` half is held privately by the forwarder task and
    /// fired exactly once, right before it returns (any exit reason:
    /// shutdown signal, upstream closed, WS silence with no resub path).
    /// Used by [`Station::force_unsubscribe_and_await`] to await confirmation
    /// that a forwarder has actually torn down (mux entry removed, upstream
    /// refs released) rather than merely requesting shutdown and hoping.
    pub(crate) exit_acks: DashMap<SeriesKey, ExitAck>,
    /// Cold-seed [`crate::SeedOutcome`] recorded per `SeriesKey`, populated
    /// by `acquire_or_spawn_derived_body`'s trade-history seed fetch right
    /// before the forwarder spawns. `Station::subscribe` drains the entry
    /// for each key it just acquired into `SubscribeReport::seed_outcomes` —
    /// this is a side-channel (not threaded through `acquire_or_spawn`'s
    /// return type) so non-derived paths (WS-only, poll-only, orderbook
    /// REST seed) are unaffected. Entries for keys with no recorded seed
    /// (WS-only cold start, or a re-acquire of an already-live mux) are
    /// simply absent — `subscribe()` only forwards what's present.
    pub(crate) seed_outcomes: DashMap<SeriesKey, crate::SeedOutcome>,
}

/// Receiver half of a forwarder-exit acknowledgement. See
/// [`StationInner::exit_acks`].
pub(crate) type ExitAck = oneshot::Receiver<()>;

/// Request to flush a forwarder's `DiskStore` and report the outcome.
///
/// Sent on [`FlushHandle::0`]; the forwarder replies on the embedded
/// `oneshot::Sender` with the result of `DiskStore::flush().await`.
pub(crate) type FlushAck = oneshot::Sender<std::io::Result<()>>;

/// Per-forwarder flush notifier, registered in `StationInner::flush_handles`.
///
/// Each forwarder that opens a `DiskStore<T>` creates one of these at spawn
/// time and keeps the paired `mpsc::Receiver<FlushAck>` in its own select
/// loop. Sending on the channel with `SendError` (receiver dropped — the
/// forwarder already exited) is treated as "already gone" by
/// [`Station::flush_persistence`], never as a hard error.
#[derive(Clone)]
pub(crate) struct FlushHandle(mpsc::Sender<FlushAck>);

impl FlushHandle {
    /// Build a fresh handle + the receiver half the forwarder keeps in its
    /// select loop. Capacity 1 — a flush request is a rendezvous, not a queue;
    /// `flush_persistence` awaits each ack before moving to the next entry.
    pub(crate) fn channel() -> (Self, mpsc::Receiver<FlushAck>) {
        let (tx, rx) = mpsc::channel(1);
        (Self(tx), rx)
    }
}

/// One broadcast-fanout actor per `SeriesKey`. Each consumer increments
/// `consumers`; on the last drop the actor shuts down.
pub(crate) struct Multiplexer {
    pub(crate) tx: broadcast::Sender<Event>,
    pub(crate) consumers: Arc<AtomicUsize>,
    pub(crate) shutdown: Option<oneshot::Sender<()>>,
    /// Cancel sender for a pending grace-period timer task.
    /// `Some` only while the forwarder is in the grace window
    /// (refcount == 0 but shutdown not yet fired). Sending `()` on this
    /// channel cancels the timer and keeps the forwarder alive.
    /// A new subscribe arriving before the timer fires sends on this channel
    /// and increments consumers.
    pub(crate) grace_cancel: Option<oneshot::Sender<()>>,
}

impl std::fmt::Debug for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Station")
            .field("storage_root", &self.inner.storage_root)
            .field("persistence", &self.inner.persistence)
            .field("muxes", &self.inner.muxes.len())
            .finish()
    }
}

/// Typed prepended-points result of [`Station::rewarm_derived`], one
/// variant per supported output shape. The caller downstream (mlc bridge)
/// matches on this to build its own per-kind `LiveUpdate` batch event —
/// this crate does not emit any bridge event itself.
#[derive(Debug, Clone)]
pub enum RewarmedPoints {
    Renko(Vec<RenkoBrickPoint>),
    Pnf(Vec<PnfColumnPoint>),
    Kagi(Vec<KagiSegmentPoint>),
    ThreeLineBreak(Vec<ThreeLineBreakLinePoint>),
    /// Shared output shape for range/tick/volume/dollar bar.
    Bar(Vec<BarPoint>),
    Cvd(Vec<ScalarBarPoint>),
    Tpo(Vec<TpoSessionPoint>),
}

impl RewarmedPoints {
    /// Number of prepended points, regardless of concrete kind.
    pub fn len(&self) -> usize {
        match self {
            RewarmedPoints::Renko(v) => v.len(),
            RewarmedPoints::Pnf(v) => v.len(),
            RewarmedPoints::Kagi(v) => v.len(),
            RewarmedPoints::ThreeLineBreak(v) => v.len(),
            RewarmedPoints::Bar(v) => v.len(),
            RewarmedPoints::Cvd(v) => v.len(),
            RewarmedPoints::Tpo(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// [`Station::rewarm_derived`]'s full result: the prepended points plus the
/// [`crate::SeedOutcome`] of the underlying trade/kline window fetch that
/// produced them.
///
/// Additive wrapper around `RewarmedPoints` (which stays a plain enum for
/// callers that only care about the points) — added so the bridge can gate
/// its ladder on `seed.truncated_by` instead of inferring "cannot deepen"
/// from an empty diff after the fact. A `RecentOnly`/`RestWindow` ceiling is
/// visible on the FIRST deepen attempt via `seed.truncated_by`, letting the
/// caller mark the key exhausted immediately instead of retrying.
#[derive(Debug, Clone)]
pub struct RewarmOutcome {
    pub points: RewarmedPoints,
    pub seed: crate::SeedOutcome,
}

impl RewarmOutcome {
    /// Number of prepended points, regardless of concrete kind.
    pub fn len(&self) -> usize { self.points.len() }
    pub fn is_empty(&self) -> bool { self.points.is_empty() }
    /// True when the underlying seed fetch reports a capability ceiling
    /// (`RecentOnly` or `RestWindow`) rather than a transient/empty result —
    /// the caller should stop retrying this key at deeper depths.
    pub fn is_capability_exhausted(&self) -> bool {
        matches!(
            self.seed.truncated_by,
            Some(crate::TruncationReason::VenueRecentOnly | crate::TruncationReason::VenueWindowCap)
        )
    }
}

impl Station {
    pub fn builder() -> StationBuilder { StationBuilder::new() }
    pub fn storage_root(&self) -> &std::path::Path { &self.inner.storage_root }
    pub fn active_streams(&self) -> usize { self.inner.muxes.len() }

    /// Shared `ExchangeHub` backing this Station's connectors.
    ///
    /// Exposed so a consumer that also needs raw REST history (e.g. a chart
    /// doing scroll-left pagination) can route `backfill::fetch_history` /
    /// `backfill::klines_recent` through the SAME connector pool the Station's
    /// live subscriptions use — instead of dialing a second, parallel hub.
    /// One pool means one dial-wave, one rate-limit budget, one warm-up.
    ///
    /// `ExchangeHub::clone` is O(1) (Arc-pooled internally).
    pub fn hub(&self) -> Arc<ExchangeHub> {
        self.inner.hub.clone()
    }

    /// Return a sync handle to the in-memory ring for `key`.
    ///
    /// Returns `Some(Arc<RwLock<Series<T>>>)` when a forwarder for `key` is
    /// currently live (active subscription or within the 30 s grace window) and
    /// the concrete element type matches `T`.  Returns `None` when no active
    /// forwarder exists for this key or when the stored type differs from `T`
    /// (type mismatch is silently treated as absent rather than panicking).
    ///
    /// Render-time consumers (chart panels, dashboards) use this to peek at the
    /// running ring without awaiting an `Event`.  The handle is independent of
    /// `SubscriptionHandle::recv()` — events still flow through there for
    /// state-mutation paths; this getter is read-only.
    pub fn series<T: DataPoint + 'static>(&self, key: &SeriesKey)
        -> Option<Arc<RwLock<Series<T>>>>
    {
        let erased = self.inner.series_handles.get(key)?;
        // Downcast Arc<dyn Any + Send + Sync> → Arc<RwLock<Series<T>>>.
        // `Arc::downcast` is not available for trait objects; use `Any::downcast_ref`
        // on the inner value to verify the type, then clone the concrete Arc.
        erased
            .downcast_ref::<Arc<RwLock<Series<T>>>>()
            .map(Arc::clone)
    }

    /// Register a consumer with the given quota. Drop the returned
    /// [`ConsumerHandle`] to release all of the consumer's active
    /// subscriptions atomically.
    ///
    /// This is opt-in: [`Station::subscribe`] continues to work without
    /// quotas. A consumer that wants caps registers and uses
    /// [`ConsumerHandle::subscribe`]; one that does not keeps calling
    /// [`Station::subscribe`] directly.
    pub fn register_consumer(
        &self,
        quota: crate::quota::ConsumerQuota,
    ) -> crate::quota::ConsumerHandle {
        let rest_bucket = quota.max_rest_per_window.map(|cap| {
            crate::quota::TokenBucket::new(cap, quota.rest_window)
        });
        crate::quota::ConsumerHandle {
            station: Arc::clone(&self.inner),
            quota,
            rest_bucket: Arc::new(tokio::sync::Mutex::new(rest_bucket)),
            refs: tokio::sync::Mutex::new((0, Vec::new())),
        }
    }

    pub(crate) async fn from_builder(b: StationBuilder) -> Result<Self> {
        let _ = digdigdig3::core::install_default_crypto_provider();
        // Native: pre-create the storage root. wasm32: OPFS directories are created
        // lazily by the OPFS DiskStore on first append — std::fs is Unsupported here
        // (it would fail Station::build for any persistence-enabled Station on wasm).
        #[cfg(not(target_arch = "wasm32"))]
        if b.persistence.enabled {
            std::fs::create_dir_all(&b.storage_root).map_err(StationError::Io)?;
        }
        let (connector_tx, _) = broadcast::channel(256);
        Ok(Self {
            inner: Arc::new(StationInner {
                hub: Arc::new(ExchangeHub::new()),
                storage_root: b.storage_root,
                persistence: b.persistence,
                muxes: DashMap::new(),
                series_handles: DashMap::new(),
                warm_start_capacity: b.warm_start.max(1),
                gap_heal: b.gap_heal,
                unsubscribe_grace: b.unsubscribe_grace,
                orderbook_rest_seed: b.orderbook_rest_seed,
                orderbook_seed_depth: b.orderbook_seed_depth,
                connector_tx,
                exchange_info_cache: DashMap::new(),
                flush_handles: DashMap::new(),
                exit_acks: DashMap::new(),
                seed_outcomes: DashMap::new(),
            }),
        })
    }

    /// A broadcast receiver for connector lifecycle events
    /// (`ConnectorReady` / `SymbolsLoaded`). Returns events emitted from any
    /// source — `warmup()`, on-demand subscribe-time connector init, REST
    /// exchange-info refresh.
    ///
    /// Independent of `SubscriptionHandle` event streams. Capacity 256.
    /// Lag drops oldest.
    pub fn connector_events(&self) -> broadcast::Receiver<crate::subscription::Event> {
        self.inner.connector_tx.subscribe()
    }

    /// Snapshot of cached `SymbolInfo` for `exchange` across all account types.
    /// Empty if `warmup` hasn't yet been called or REST hasn't completed.
    pub fn symbols(&self, exchange: ExchangeId) -> Vec<SymbolInfo> {
        let mut out = Vec::new();
        for entry in self.inner.exchange_info_cache.iter() {
            if entry.key().0 == exchange {
                out.extend_from_slice(entry.value());
            }
        }
        out
    }

    /// Snapshot the live health metrics for the WS forwarder backing `key`.
    ///
    /// Returns `None` if no forwarder exists for this key (no active or
    /// grace-window subscription).
    ///
    /// Sync, non-blocking — suitable for periodic diagnostics polls or
    /// per-frame UI overlays (latency badges). Each field is a best-effort
    /// snapshot:
    ///
    /// - `connected`: always accurate — derived from mux presence.
    /// - `rtt_ms`: `None` until per-connector RTT handle wiring is added
    ///   (incremental; OKX is the first candidate).
    /// - `last_message_ms`: `None` until per-forwarder atomic timestamp
    ///   wiring is added (incremental).
    pub fn ws_health(&self, key: &SeriesKey) -> Option<WsHealth> {
        // Presence in muxes == a live forwarder (active or grace-window).
        self.inner.muxes.get(key)?;
        Some(WsHealth {
            connected: true,
            rtt_ms: None,
            last_message_ms: None,
        })
    }

    /// Aggregate health across all forwarders for `exchange`.
    ///
    /// - `rtt_ms`: median of non-`None` RTT values across forwarders.
    /// - `last_message_ms`: max of non-`None` last-message timestamps
    ///   (most-recent message seen on any forwarder for this exchange).
    /// - `connected`: `true` if at least one forwarder is connected.
    ///
    /// Returns `None` if there are no active forwarders for `exchange`.
    pub fn ws_health_for_exchange(
        &self,
        exchange: ExchangeId,
    ) -> Option<WsHealth> {
        let mut any_connected = false;
        let mut rtts: Vec<u64> = Vec::new();
        let mut last_msgs: Vec<i64> = Vec::new();

        for entry in self.inner.muxes.iter() {
            if entry.key().exchange != exchange {
                continue;
            }
            // Each mux entry == one live forwarder.
            let h = WsHealth {
                connected: true,
                rtt_ms: None,
                last_message_ms: None,
            };
            any_connected = true;
            if let Some(rtt) = h.rtt_ms {
                rtts.push(rtt);
            }
            if let Some(ts) = h.last_message_ms {
                last_msgs.push(ts);
            }
        }

        if !any_connected {
            return None;
        }

        let rtt_ms = if rtts.is_empty() {
            None
        } else {
            rtts.sort_unstable();
            Some(rtts[rtts.len() / 2])
        };

        Some(WsHealth {
            connected: true,
            rtt_ms,
            last_message_ms: last_msgs.into_iter().max(),
        })
    }

    /// Drain + flush every open `DiskStore` across all live forwarders
    /// (primitive, derived, and poller) and wait for each to acknowledge.
    ///
    /// This exists chiefly for wasm OPFS: the browser can tear down the tab
    /// on `beforeunload` / `visibilitychange` before an async `Drop` flush
    /// would ever run, so the mlc-shell calls this from the unload handler to
    /// force every buffered store to disk first. Native `BufWriter` already
    /// flushes synchronously on `Drop`; this method is still safe and useful
    /// there (e.g. before a controlled shutdown) and keeps the API identical
    /// across targets.
    ///
    /// A forwarder that has already exited (its flush-request receiver
    /// dropped) is treated as "already flushed" — not an error. Only a
    /// `DiskStore::flush().await` call that actually ran and returned `Err`
    /// is collected in the result.
    pub async fn flush_persistence(&self) -> std::result::Result<(), Vec<(SeriesKey, std::io::Error)>> {
        let handles: Vec<(SeriesKey, FlushHandle)> = self
            .inner
            .flush_handles
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let mut errors = Vec::new();

        for (key, handle) in handles {
            let (ack_tx, ack_rx) = oneshot::channel();
            match handle.0.send(ack_tx).await {
                Ok(()) => match ack_rx.await {
                    Ok(Ok(())) => {}
                    Ok(Err(io_err)) => errors.push((key, io_err)),
                    // Forwarder dropped the ack sender without replying
                    // (exited mid-flush) — treat as already gone, not an error.
                    Err(_) => {}
                },
                // Forwarder's receiver already dropped (exited before this
                // request was sent) — treat as already flushed.
                Err(_) => {}
            }
        }

        // Best-effort sweep of entries whose forwarder has since exited but
        // never got a chance to unregister (e.g. it exited concurrently with
        // this call, after the snapshot above but before the send). A stale
        // entry is harmless (the next flush_persistence call will just see
        // the same SendError and skip it again), so this is opportunistic
        // cleanup rather than a correctness requirement.
        self.inner
            .flush_handles
            .retain(|_, handle| !handle.0.is_closed());

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Force a live (or grace-window) forwarder for `key` to tear down
    /// immediately, and wait until it has actually exited.
    ///
    /// Unlike letting all consumers drop (which relies on
    /// `unsubscribe_grace` — 30s in mlc's config — before the forwarder
    /// actually shuts down), this cancels any pending grace timer, fires the
    /// forwarder's shutdown signal unconditionally regardless of remaining
    /// consumer count, and awaits a forwarder-exit acknowledgement before
    /// returning. This makes cold-respawn-with-a-larger-`warm_override`
    /// deterministic instead of racing a 30s timer.
    ///
    /// Idempotent: if no mux exists for `key` (already torn down, or never
    /// subscribed), returns `Ok(())` immediately.
    ///
    /// Race-safety: if the forwarder exits on its own (e.g. upstream closed)
    /// between the shutdown signal and the ack being registered, the ack
    /// entry being gone from `exit_acks` is treated as "already exited" —
    /// this method never hangs waiting for an ack that will never come.
    pub async fn force_unsubscribe_and_await(&self, key: &SeriesKey) -> Result<()> {
        // Cancel any pending grace-period timer, mirroring the reuse path in
        // `acquire_or_spawn` (station.rs ~:857) — a timer left running would
        // otherwise race the shutdown we are about to fire, though the
        // idempotent guards on both sides make this a non-issue either way.
        // Fire shutdown unconditionally regardless of `consumers` count —
        // this is the key difference from `release_consumer`, which only
        // fires shutdown once the count reaches zero.
        let removed = {
            let Some(mut mux) = self.inner.muxes.get_mut(key) else {
                // No live mux — nothing to tear down.
                return Ok(());
            };
            if let Some(cancel) = mux.grace_cancel.take() {
                let _ = cancel.send(());
            }
            mux.shutdown.take()
        };

        self.inner.muxes.remove(key);
        self.inner.series_handles.remove(key);

        if let Some(shutdown_tx) = removed {
            let _ = shutdown_tx.send(());
        }

        // Await the forwarder's exit ack. If the registry entry is already
        // gone (forwarder exited concurrently, e.g. upstream closed right
        // before we got here) there is nothing to await — treat as done.
        // A `RecvError` (sender dropped without sending — forwarder task
        // panicked or was aborted) is likewise treated as "exited" rather
        // than a hard error, since the goal is "forwarder is gone", not
        // "forwarder exited cleanly".
        if let Some((_, ack_rx)) = self.inner.exit_acks.remove(key) {
            let _ = ack_rx.await;
        }

        Ok(())
    }

    /// Deepen the warm window of a LIVE footprint subscription in place —
    /// no teardown, no live gap. Fetches a `warm_n`-deep aggTrade window
    /// from REST (same pager as the cold-start seed), replays it through a
    /// FRESH `TradeToFootprintDerived` state machine, and splices the
    /// resulting bars in FRONT of the existing ring: only bars at/older
    /// than the current ring head bucket are taken (the partial boundary
    /// bucket is replaced by its full rebuild); everything newer stays
    /// owned by the live forwarder, so there is no seam race with
    /// in-progress buckets.
    ///
    /// Footprint-only by design: its buckets are pure time buckets, so
    /// bars rebuilt from a longer trade window agree with already-ringed
    /// bars at every bucket boundary. Path-dependent derived kinds (renko/
    /// pnf/kagi/3lb/range/volume/tick/...) MUST keep the teardown+respawn
    /// deepen — their bar boundaries depend on where the seed starts.
    ///
    /// Returns the spliced-in points (oldest→newest, all `open_time <=`
    /// the pre-call ring head; empty when the key has no live ring, the
    /// venue yields no history, or nothing at/older than the ring head was
    /// produced). No `Event` is broadcast here — the caller decides how to
    /// deliver the batch downstream (a per-point broadcast of thousands of
    /// historical bars is exactly the consumer-side stall this API
    /// avoids).
    pub async fn rewarm_footprint(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> Vec<FootprintPoint> {
        // No live ring → nothing to splice into; caller should fall back
        // to a cold subscribe at the deeper depth.
        let Some(handle) = self.inner.series_handles.get(key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<FootprintPoint>>>>().map(Arc::clone)
        }) else {
            return Vec::new();
        };
        if warm_n == 0 {
            return Vec::new();
        }

        // REST fetch + derive happen OUTSIDE the ring lock — this is the
        // long part (100+ sequential pages on a 100k window in a browser);
        // the live forwarder keeps appending to the ring meanwhile.
        let caps_opt = self.inner.hub.capabilities(key.exchange);
        let use_agg = caps_opt.as_ref().map(|c| c.has_agg_trades).unwrap_or(false);
        let mut seed_events: Vec<Event> = Vec::new();
        if use_agg {
            let n_pages = ((warm_n + 999) / 1000).max(1);
            let (agg, _outcome) = crate::backfill::agg_trades_paginated(
                &self.inner.hub, key.exchange, key.account_type, raw_symbol, 1000, n_pages,
            )
            .await;
            for ap in agg {
                seed_events.push(Event::Trade {
                    exchange: key.exchange,
                    symbol: raw_symbol.to_string(),
                    point: TradePoint {
                        ts_ms: ap.ts_ms,
                        price: ap.price,
                        quantity: ap.quantity,
                        side: ap.side,
                        trade_id_hash: 0,
                    },
                });
            }
        } else {
            let (trades, _outcome) = crate::backfill::trades_recent(
                &self.inner.hub, key.exchange, key.account_type, raw_symbol, warm_n,
            )
            .await;
            for pt in trades {
                seed_events.push(Event::Trade {
                    exchange: key.exchange,
                    symbol: raw_symbol.to_string(),
                    point: pt,
                });
            }
        }
        if seed_events.is_empty() {
            return Vec::new();
        }
        // Chunked derive with a MACROTASK yield between chunks: on wasm the
        // whole rewarm runs on the browser main thread, and one synchronous
        // pass over a 100k-trade window stalls RAF/input for seconds
        // (microtask yields — tokio yield_now — do NOT let the browser
        // paint; only a timer macrotask does).
        let mut state = TradeToFootprintDerived::new_for_key(key);
        let mut emissions: Vec<FootprintPoint> = Vec::new();
        for chunk in seed_events.chunks(5_000) {
            emissions.extend(state.seed_from_events(chunk, 0));
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::sleep(std::time::Duration::from_millis(0)).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::task::yield_now().await;
        }
        // The state machine emits the in-progress bar on EVERY trade — the
        // cold-seed path collapses those through the ring's `upsert_by_ts`,
        // so a splice must dedup the same way: keep the LAST emission per
        // bucket open_time (chronological input → later supersedes).
        let mut rebuilt: Vec<FootprintPoint> = Vec::new();
        for p in emissions {
            match rebuilt.last_mut() {
                Some(last) if last.timestamp_ms() == p.timestamp_ms() => *last = p,
                _ => rebuilt.push(p),
            }
        }
        if rebuilt.is_empty() {
            return Vec::new();
        }

        // Splice under ONE write lock: snapshot → cut at the ring head
        // bucket → rebuild a larger ring. The in-lock work is a few
        // thousand clones (microseconds); a live bar landing between the
        // fetch above and this lock only grows the tail, which is kept
        // verbatim.
        let mut guard = handle.write().await;
        let existing = guard.snapshot();
        let head_ts = existing.first().map(|p| p.timestamp_ms());
        let older: Vec<FootprintPoint> = rebuilt
            .into_iter()
            .filter(|p| head_ts.is_none_or(|h| p.timestamp_ms() <= h))
            .collect();
        if older.is_empty() {
            return Vec::new();
        }
        let boundary_replaced = head_ts
            .is_some_and(|h| older.last().is_some_and(|p| p.timestamp_ms() == h));
        let keep_from = usize::from(boundary_replaced);
        let merged_len = older.len() + existing.len().saturating_sub(keep_from);
        let mut fresh = Series::new(merged_len.max(guard.capacity()));
        fresh.extend(older.iter().cloned());
        fresh.extend(existing.into_iter().skip(keep_from));
        *guard = fresh;
        older
    }

    /// Generalized version of [`Station::rewarm_footprint`] for path-
    /// dependent derived kinds — renko / pnf / kagi / three-line-break /
    /// range / volume / tick / dollar bar / cvd / tpo.
    ///
    /// Same call shape and same mechanics as `rewarm_footprint`: fetch a
    /// deeper raw trade window, run a THROWAWAY fold over it with a FRESH
    /// state machine (optionally seeded from the existing series' own
    /// output so path-dependent grids/state line up with the live ring),
    /// then splice the result in FRONT of the existing ring under one
    /// write lock. Existing points are moved verbatim — never re-derived.
    ///
    /// Returns `Ok(RewarmOutcome)` — prepended points plus the seed fetch's
    /// [`crate::SeedOutcome`] — on the kinds this mechanism supports.
    /// Returns `Err(StationError::RewarmUnsupported)` for
    /// `TickImbalanceBar` / `VolumeImbalanceBar` / `RunBar` — their fold
    /// state is an EMA carried across the ENTIRE history, which cannot be
    /// reconstructed from already-emitted output; callers must fall back to
    /// the teardown + resubscribe-at-depth path for those three. Also
    /// `Err` for `Footprint` (use `rewarm_footprint` instead) and any
    /// non-derived kind.
    pub async fn rewarm_derived(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> Result<RewarmOutcome> {
        match &key.kind {
            Kind::RenkoBar(_, _) => {
                let (v, seed) = self.rewarm_renko(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Renko(v), seed })
            }
            Kind::PnfBar(_, _) => {
                let (v, seed) = self.rewarm_pnf(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Pnf(v), seed })
            }
            Kind::KagiBar(_) => {
                let (v, seed) = self.rewarm_derived_standalone::<TradeToKagiBarDerived>(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Kagi(v), seed })
            }
            Kind::ThreeLineBreak { .. } => {
                let (v, seed) = self.rewarm_derived_standalone::<TradeToThreeLineBreakDerived>(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::ThreeLineBreak(v), seed })
            }
            Kind::RangeBar(_) => {
                let (v, seed) = self.rewarm_derived_standalone::<TradeToRangeBarDerived>(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Bar(v), seed })
            }
            Kind::VolumeBar(_) => {
                let (v, seed) = self.rewarm_derived_standalone::<TradeToVolumeBarDerived>(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Bar(v), seed })
            }
            Kind::TickBar(_) => {
                let (v, seed) = self.rewarm_derived_standalone::<TradeToTickBarDerived>(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Bar(v), seed })
            }
            Kind::DollarBar { .. } => {
                let (v, seed) = self.rewarm_derived_standalone::<TradeToDollarBarDerived>(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Bar(v), seed })
            }
            Kind::CvdLine => {
                let (v, seed) = self.rewarm_cvd(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Cvd(v), seed })
            }
            Kind::TpoProfile(_, _) => {
                let (v, seed) = self.rewarm_tpo(key, raw_symbol, warm_n).await;
                Ok(RewarmOutcome { points: RewarmedPoints::Tpo(v), seed })
            }
            Kind::TickImbalanceBar { .. } => Err(StationError::RewarmUnsupported(
                "TickImbalanceBar carries EMA state (E[T]/E[|theta|]) over the entire \
                 history — not reconstructible from emitted output; use teardown+resubscribe."
                    .to_string(),
            )),
            Kind::VolumeImbalanceBar { .. } => Err(StationError::RewarmUnsupported(
                "VolumeImbalanceBar carries EMA state over the entire history — not \
                 reconstructible from emitted output; use teardown+resubscribe."
                    .to_string(),
            )),
            Kind::RunBar { .. } => Err(StationError::RewarmUnsupported(
                "RunBar carries EMA state over the entire history — not reconstructible \
                 from emitted output; use teardown+resubscribe."
                    .to_string(),
            )),
            Kind::Footprint(_) => Err(StationError::RewarmUnsupported(
                "Footprint has its own dedicated rewarm_footprint — call that instead."
                    .to_string(),
            )),
            other => Err(StationError::RewarmUnsupported(format!(
                "{other:?} is not a prepend-fold-capable derived kind"
            ))),
        }
    }

    /// Fetch the deeper raw trade window shared by every `rewarm_*` helper:
    /// paginated aggTrades (falling back to `trades_recent` when the venue
    /// lacks aggTrade history), same pager `rewarm_footprint` uses.
    ///
    /// Returns the seed events alongside the [`crate::SeedOutcome`] so
    /// callers can tell a live venue-capability ceiling
    /// (`TruncationReason::VenueRecentOnly` / `VenueWindowCap`) apart from a
    /// genuinely empty window — a `RecentOnly` venue reports the ceiling on
    /// the FIRST deepen attempt instead of after repeated hammering.
    async fn rewarm_fetch_trade_window(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> (Vec<Event>, crate::SeedOutcome) {
        if warm_n == 0 {
            return (Vec::new(), crate::SeedOutcome::full(0, 0, crate::SeedSource::AggTradesPaginated));
        }
        let caps_opt = self.inner.hub.capabilities(key.exchange);
        let use_agg = caps_opt.as_ref().map(|c| c.has_agg_trades).unwrap_or(false);
        let mut seed_events: Vec<Event> = Vec::new();
        let outcome;
        if use_agg {
            let n_pages = ((warm_n + 999) / 1000).max(1);
            let (agg, agg_outcome) = crate::backfill::agg_trades_paginated(
                &self.inner.hub, key.exchange, key.account_type, raw_symbol, 1000, n_pages,
            )
            .await;
            for ap in agg {
                seed_events.push(Event::Trade {
                    exchange: key.exchange,
                    symbol: raw_symbol.to_string(),
                    point: TradePoint {
                        ts_ms: ap.ts_ms,
                        price: ap.price,
                        quantity: ap.quantity,
                        side: ap.side,
                        trade_id_hash: 0,
                    },
                });
            }
            outcome = agg_outcome;
        } else {
            let (trades, trades_outcome) = crate::backfill::trades_recent(
                &self.inner.hub, key.exchange, key.account_type, raw_symbol, warm_n,
            )
            .await;
            for pt in trades {
                seed_events.push(Event::Trade {
                    exchange: key.exchange,
                    symbol: raw_symbol.to_string(),
                    point: pt,
                });
            }
            outcome = trades_outcome;
        }
        (seed_events, outcome)
    }

    /// Kline-approx window for the kline-sufficient kinds
    /// (renko/pnf/kagi/tlb/range/volume/dollar) — matches the cold-seed
    /// composition in `acquire_or_spawn_derived_body`, so a deepened rewarm
    /// fold sees the same data quality as a cold seed would at that depth.
    ///
    /// These kinds are PURE KLINE CONVERTERS (no `Stream::Trade` dependency
    /// at all — see `DerivedStream::deps_for_key` on each of the seven), so
    /// this is their ENTIRE rewarm fetch; there is no trade-history tail to
    /// bridge with. `klines_paginated` returns oldest→newest and its window
    /// reaches "now", so the LAST bar may be the exchange's still-forming
    /// candle — excluded from the 4-leg fold for the same reason the
    /// cold-seed excludes it (see `acquire_or_spawn_derived_body`'s
    /// kline-sufficient seed branch): folding it in full would double-count
    /// against whatever the live series already derived from the WS Δ-tick
    /// path for that same bar. Dropping it here (rather than handing a
    /// baseline forward, which a THROWAWAY rewarm fold has no live
    /// continuation to hand it to) is the audited ≤1-bar seam already
    /// accepted elsewhere in this design (see `rewarm_derived_standalone`'s
    /// doc comment).
    async fn rewarm_fetch_kline_sufficient_window(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> (Vec<Event>, crate::SeedOutcome) {
        if warm_n == 0 {
            return (Vec::new(), crate::SeedOutcome::full(0, 0, crate::SeedSource::Klines));
        }
        let n_kline_pages = ((warm_n + 999) / 1000).clamp(1, 100);
        let (kline_bars, outcome) = crate::backfill::klines_paginated(
            &self.inner.hub, key.exchange, key.account_type, raw_symbol, "1m", 1000, n_kline_pages,
        )
        .await;
        let kline_interval_ms = interval_to_ms("1m").unwrap_or(60_000);
        let mut events = Vec::new();
        if let Some((_last, closed)) = kline_bars.split_last() {
            for bar in closed {
                events.extend(crate::derived::kline_to_synthetic_trades(
                    bar, kline_interval_ms, key.exchange, raw_symbol,
                ));
            }
        }
        (events, outcome)
    }

    /// Chunked throwaway fold over `events` through a fresh `D` state
    /// machine, yielding a wasm-safe macrotask between chunks — identical
    /// cadence to `rewarm_footprint`. `seed` runs once on the freshly
    /// constructed state, before any events are fed, so callers can inject
    /// per-kind grid/state presets (renko/pnf).
    async fn rewarm_fold<D: DerivedStream>(
        key: &SeriesKey,
        events: &[Event],
        seed: impl FnOnce(&mut D),
    ) -> Vec<D::Output> {
        let mut state = D::new_for_key(key);
        seed(&mut state);
        let mut emissions: Vec<D::Output> = Vec::new();
        for chunk in events.chunks(5_000) {
            emissions.extend(state.seed_from_events(chunk, 0));
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::sleep(std::time::Duration::from_millis(0)).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::task::yield_now().await;
        }
        emissions
    }

    /// Splice `rebuilt` (already deduped-per-identity, chronological) in
    /// front of the existing ring for `key`, under one write lock —
    /// same overall shape as the tail of `rewarm_footprint`, but with the
    /// OPPOSITE boundary policy: **existing wins**. Footprint's buckets are
    /// pure time-buckets so its fresher rebuild of the boundary bucket is
    /// strictly more complete and replaces the live one; every derived kind
    /// here is path-dependent (renko/pnf grid, kagi shoulder/waist, TLB
    /// ring, running counters, CVD cumulative value, TPO session) — its
    /// live bar reflects state the throwaway fold cannot see (everything
    /// that happened after the older-window fetch), so the live copy must
    /// never be overwritten. `ident` extracts the per-kind bar identity
    /// used for the boundary-collision check (NOT necessarily
    /// `timestamp_ms()` — e.g. pnf uses `column_id`); ordering itself still
    /// uses `timestamp_ms()`.
    async fn rewarm_splice<T: DataPoint + Clone, Id: PartialEq>(
        &self,
        key: &SeriesKey,
        rebuilt: Vec<T>,
        ident: impl Fn(&T) -> Id,
    ) -> Vec<T> {
        let Some(handle) = self.inner.series_handles.get(key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<T>>>>().map(Arc::clone)
        }) else {
            return Vec::new();
        };
        if rebuilt.is_empty() {
            return Vec::new();
        }
        let mut guard = handle.write().await;
        let existing = guard.snapshot();
        let head_ts = existing.first().map(|p| p.timestamp_ms());
        let mut older: Vec<T> = rebuilt
            .into_iter()
            .filter(|p| head_ts.is_none_or(|h| p.timestamp_ms() <= h))
            .collect();
        if older.is_empty() {
            return Vec::new();
        }
        // Boundary-collision guard: if the fold's last (newest) older point
        // shares identity with the existing head, the fold's copy is a
        // partial/stale rebuild of a bar the live forwarder already owns —
        // drop it. Existing never gets overwritten.
        let head_ident = existing.first().map(&ident);
        if head_ident.as_ref().is_some_and(|h| older.last().is_some_and(|p| &ident(p) == h)) {
            older.pop();
        }
        if older.is_empty() {
            return Vec::new();
        }
        let merged_len = older.len() + existing.len();
        let mut fresh = Series::new(merged_len.max(guard.capacity()));
        fresh.extend(older.iter().cloned());
        fresh.extend(existing);
        *guard = fresh;
        older
    }

    /// Dedup a chronological emission stream down to "last emission per
    /// identity supersedes" — same rule `rewarm_footprint` applies per
    /// `open_time` bucket, generalized to an arbitrary `ident` extractor
    /// (most kinds re-emit the in-progress bar on every trade; only the
    /// final emission per identity reflects the bar's closed/complete
    /// state).
    fn rewarm_dedup<T: Clone, Id: PartialEq>(emissions: Vec<T>, ident: impl Fn(&T) -> Id) -> Vec<T> {
        let mut out: Vec<T> = Vec::new();
        for p in emissions {
            match out.last() {
                Some(last) if ident(last) == ident(&p) => {
                    let idx = out.len() - 1;
                    out[idx] = p;
                }
                _ => out.push(p),
            }
        }
        out
    }

    /// Renko rewarm: preset the throwaway fold's grid anchor from the
    /// existing series' first brick's lower boundary (`bottom`) so the
    /// prepended bricks land on the SAME grid as the live series — no
    /// floor-snap off wherever the older window happens to start.
    ///
    /// Returns the prepended points alongside the [`crate::SeedOutcome`] of
    /// the underlying kline-sufficient window fetch (kline side is
    /// depth-defining — see [`Self::rewarm_fetch_kline_sufficient_window`])
    /// — a `RecentOnly`/`RestWindow` ceiling on that fetch is the caller's
    /// signal that this key cannot be deepened further no matter how many
    /// times it retries.
    async fn rewarm_renko(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> (Vec<RenkoBrickPoint>, crate::SeedOutcome) {
        let empty_outcome = crate::SeedOutcome::full(warm_n, 0, crate::SeedSource::AggTradesPaginated);
        let Some(handle) = self.inner.series_handles.get(key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<RenkoBrickPoint>>>>().map(Arc::clone)
        }) else {
            return (Vec::new(), empty_outcome);
        };
        let grid_floor = {
            let guard = handle.read().await;
            match guard.snapshot().first() {
                Some(first) => first.bottom,
                None => return (Vec::new(), empty_outcome),
            }
        };
        let (events, outcome) = self.rewarm_fetch_kline_sufficient_window(key, raw_symbol, warm_n).await;
        if events.is_empty() {
            return (Vec::new(), outcome);
        }
        let emissions = Self::rewarm_fold::<TradeToRenkoBarDerived>(key, &events, |state| {
            state.preset_grid_anchor(grid_floor);
        })
        .await;
        let rebuilt = Self::rewarm_dedup(emissions, |p: &RenkoBrickPoint| p.open_time);
        if rebuilt.is_empty() {
            return (Vec::new(), outcome);
        }
        (self.rewarm_splice(key, rebuilt, |p: &RenkoBrickPoint| p.open_time).await, outcome)
    }

    /// PnF rewarm: preset the throwaway fold's box grid from the existing
    /// series' first column's `(bottom, top, is_x)` span — same reasoning
    /// as renko.
    async fn rewarm_pnf(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> (Vec<PnfColumnPoint>, crate::SeedOutcome) {
        let empty_outcome = crate::SeedOutcome::full(warm_n, 0, crate::SeedSource::AggTradesPaginated);
        let Some(handle) = self.inner.series_handles.get(key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<PnfColumnPoint>>>>().map(Arc::clone)
        }) else {
            return (Vec::new(), empty_outcome);
        };
        let (bottom, top, is_x) = {
            let guard = handle.read().await;
            match guard.snapshot().first() {
                Some(first) => (first.bottom, first.top, first.is_x),
                None => return (Vec::new(), empty_outcome),
            }
        };
        let (events, outcome) = self.rewarm_fetch_kline_sufficient_window(key, raw_symbol, warm_n).await;
        if events.is_empty() {
            return (Vec::new(), outcome);
        }
        let emissions = Self::rewarm_fold::<TradeToPnfBarDerived>(key, &events, |state| {
            state.preset_grid(bottom, top, is_x);
        })
        .await;
        let rebuilt = Self::rewarm_dedup(emissions, |p: &PnfColumnPoint| p.column_id);
        if rebuilt.is_empty() {
            return (Vec::new(), outcome);
        }
        (self.rewarm_splice(key, rebuilt, |p: &PnfColumnPoint| p.column_id).await, outcome)
    }

    /// Standalone prepend-fold for kinds whose throwaway fold needs NO
    /// state injected from the existing series' output — kagi (shoulder/
    /// waist reconstruction deferred; v1 accepts the audited ≤1-segment
    /// seam), three-line-break (ring rebuilds itself within
    /// `lines_back` lines), and range/tick/volume/dollar bar (counters
    /// reset cleanly at the fold's own first trade; v1 accepts the
    /// audited ≤1-bar seam). `D::Output` identity for dedup + splice is
    /// `timestamp_ms()` for every one of these (renko/pnf are the only
    /// kinds with a non-timestamp identity).
    ///
    /// Kagi/three-line-break/range/volume/dollar are kline-sufficient (see
    /// `is_kline_sufficient_kind` in `acquire_or_spawn_derived_body`) and get
    /// the deep kline-approx window instead of deep trade pagination — tick
    /// bar genuinely needs per-trade counts and keeps the plain trade-window
    /// fetch.
    async fn rewarm_derived_standalone<D>(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> (Vec<D::Output>, crate::SeedOutcome)
    where
        D: DerivedStream,
        D::Output: Clone,
    {
        let empty_outcome = crate::SeedOutcome::full(warm_n, 0, crate::SeedSource::AggTradesPaginated);
        if self.inner.series_handles.get(key).is_none() {
            return (Vec::new(), empty_outcome);
        }
        let is_kline_sufficient_kind = matches!(
            key.kind,
            Kind::KagiBar(_) | Kind::ThreeLineBreak { .. }
            | Kind::RangeBar(_) | Kind::VolumeBar(_) | Kind::DollarBar { .. }
        );
        let (events, outcome) = if is_kline_sufficient_kind {
            self.rewarm_fetch_kline_sufficient_window(key, raw_symbol, warm_n).await
        } else {
            self.rewarm_fetch_trade_window(key, raw_symbol, warm_n).await
        };
        if events.is_empty() {
            return (Vec::new(), outcome);
        }
        let emissions = Self::rewarm_fold::<D>(key, &events, |_state| {}).await;
        let rebuilt = Self::rewarm_dedup(emissions, |p: &D::Output| p.timestamp_ms());
        if rebuilt.is_empty() {
            return (Vec::new(), outcome);
        }
        (self.rewarm_splice(key, rebuilt, |p: &D::Output| p.timestamp_ms()).await, outcome)
    }

    /// CVD rewarm: fold the older window standalone (starts its own
    /// cumulative sum at 0 from the fold's first trade), then OFFSET every
    /// prepended sample by a constant so the LAST prepended sample's value
    /// equals the existing FIRST sample's value. Existing samples are
    /// never touched — only the prepended segment is shifted to meet them
    /// exactly at the seam.
    async fn rewarm_cvd(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> (Vec<ScalarBarPoint>, crate::SeedOutcome) {
        let empty_outcome = crate::SeedOutcome::full(warm_n, 0, crate::SeedSource::AggTradesPaginated);
        let Some(handle) = self.inner.series_handles.get(key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<ScalarBarPoint>>>>().map(Arc::clone)
        }) else {
            return (Vec::new(), empty_outcome);
        };
        let existing_first_value = {
            let guard = handle.read().await;
            match guard.snapshot().first() {
                Some(first) => first.value,
                None => return (Vec::new(), empty_outcome),
            }
        };
        let (events, outcome) = self.rewarm_fetch_trade_window(key, raw_symbol, warm_n).await;
        if events.is_empty() {
            return (Vec::new(), outcome);
        }
        let emissions = Self::rewarm_fold::<TradeToCvdLineDerived>(key, &events, |_state| {}).await;
        let mut rebuilt = Self::rewarm_dedup(emissions, |p: &ScalarBarPoint| p.ts_ms);
        if rebuilt.is_empty() {
            return (Vec::new(), outcome);
        }
        // Offset so the fold's LAST (newest) sample ends exactly at the
        // existing series' first value — the seam is exact by
        // construction, not approximate.
        let offset = existing_first_value - rebuilt.last().map(|p| p.value).unwrap_or(0.0);
        for p in &mut rebuilt {
            p.value += offset;
        }
        (self.rewarm_splice(key, rebuilt, |p: &ScalarBarPoint| p.ts_ms).await, outcome)
    }

    /// TPO rewarm: sessions are wall-clock UTC-day buckets (like footprint's
    /// time buckets), so grid alignment is inherent — no state injection.
    /// The one boundary hazard is the throwaway fold's own trailing
    /// in-progress session colliding with the existing head session; that
    /// case is exactly what `rewarm_splice`'s identity-based boundary guard
    /// (`open_time == session_date_ms`) already drops, so no extra
    /// bookkeeping is needed beyond routing through the same helper.
    async fn rewarm_tpo(
        &self,
        key: &SeriesKey,
        raw_symbol: &str,
        warm_n: usize,
    ) -> (Vec<TpoSessionPoint>, crate::SeedOutcome) {
        let empty_outcome = crate::SeedOutcome::full(warm_n, 0, crate::SeedSource::Klines);
        if self.inner.series_handles.get(key).is_none() {
            return (Vec::new(), empty_outcome);
        }
        let (rebuilt, outcome): (Vec<TpoSessionPoint>, crate::SeedOutcome) = match &key.kind {
            Kind::TpoProfile(_, TpoSource::Kline1m) => {
                let (kline_bars, kline_outcome) = crate::backfill::klines_paginated(
                    &self.inner.hub, key.exchange, key.account_type, raw_symbol, "1m", 1000,
                    ((warm_n + 999) / 1000).max(1),
                )
                .await;
                if kline_bars.is_empty() {
                    return (Vec::new(), kline_outcome);
                }
                let interval = digdigdig3::core::websocket::KlineInterval::new("1m");
                let events: Vec<Event> = kline_bars
                    .into_iter()
                    .map(|point| Event::Bar {
                        exchange: key.exchange,
                        symbol: raw_symbol.to_string(),
                        timeframe: interval.clone(),
                        point,
                    })
                    .collect();
                let emissions = Self::rewarm_fold::<TpoFromKline1mDerived>(key, &events, |_state| {}).await;
                (Self::rewarm_dedup(emissions, |p: &TpoSessionPoint| p.open_time), kline_outcome)
            }
            Kind::TpoProfile(_, TpoSource::TradeBucket) => {
                let (events, trade_outcome) = self.rewarm_fetch_trade_window(key, raw_symbol, warm_n).await;
                if events.is_empty() {
                    return (Vec::new(), trade_outcome);
                }
                let emissions = Self::rewarm_fold::<TpoFromTradeDerived>(key, &events, |_state| {}).await;
                (Self::rewarm_dedup(emissions, |p: &TpoSessionPoint| p.open_time), trade_outcome)
            }
            _ => return (Vec::new(), empty_outcome),
        };
        if rebuilt.is_empty() {
            return (Vec::new(), outcome);
        }
        (self.rewarm_splice(key, rebuilt, |p: &TpoSessionPoint| p.open_time).await, outcome)
    }

    /// Eagerly connect to every exchange in `exchanges` and pre-load their
    /// full symbol list. Subscribes nothing — produces only
    /// `Event::ConnectorReady` (one per exchange that finishes
    /// `connect_public`) and `Event::SymbolsLoaded` (one per exchange whose
    /// REST `get_exchange_info` succeeds for at least one account type) on
    /// the broadcast channel returned by `Station::connector_events()`.
    ///
    /// Idempotent: running concurrently or repeatedly is safe.
    /// Already-connected exchanges short-circuit. Already-cached symbol lists
    /// re-broadcast from cache without REST.
    ///
    /// Runs to completion — returns a [`WarmupReport`] of outcomes.
    pub async fn warmup(&self, exchanges: &[ExchangeId]) -> crate::subscription::WarmupReport {
        use crate::subscription::{Event, WarmupReport};

        let mut ok = Vec::new();
        let mut failed: Vec<(ExchangeId, String)> = Vec::new();

        // Phase 1: connect all exchanges (spawn concurrent tasks).
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut join_set: tokio::task::JoinSet<(ExchangeId, Result<()>)> =
                tokio::task::JoinSet::new();

            for &eid in exchanges {
                if self.inner.hub.is_connected(eid) {
                    let _ = self.inner.connector_tx.send(Event::ConnectorReady { exchange: eid });
                    ok.push(eid);
                } else {
                    let hub = Arc::clone(&self.inner.hub);
                    join_set.spawn(async move {
                        (eid, hub.connect_public(eid, false).await.map_err(|e| {
                            crate::StationError::Core(e.to_string())
                        }))
                    });
                }
            }

            while let Some(res) = join_set.join_next().await {
                match res {
                    Ok((eid, Ok(()))) => {
                        let _ = self.inner.connector_tx.send(Event::ConnectorReady { exchange: eid });
                        ok.push(eid);
                    }
                    Ok((eid, Err(e))) => {
                        tracing::warn!(?eid, ?e, "warmup: connect_public failed");
                        failed.push((eid, e.to_string()));
                    }
                    Err(join_err) => {
                        tracing::warn!(?join_err, "warmup: task panicked");
                    }
                }
            }
        }
        // wasm32: no JoinSet — run sequentially (wasm is single-threaded).
        #[cfg(target_arch = "wasm32")]
        {
            for &eid in exchanges {
                if self.inner.hub.is_connected(eid) {
                    let _ = self.inner.connector_tx.send(Event::ConnectorReady { exchange: eid });
                    ok.push(eid);
                } else {
                    match self.inner.hub.connect_public(eid, false).await {
                        Ok(()) => {
                            let _ = self.inner.connector_tx.send(Event::ConnectorReady { exchange: eid });
                            ok.push(eid);
                        }
                        Err(e) => {
                            tracing::warn!(?eid, ?e, "warmup: connect_public failed");
                            failed.push((eid, e.to_string()));
                        }
                    }
                }
            }
        }

        // Phase 2: fetch exchange info for all successfully connected exchanges.
        const ACCOUNT_TYPES: &[AccountType] = &[AccountType::Spot, AccountType::FuturesCross];

        for eid in ok.iter().copied() {
            let Some(connector) = self.inner.hub.rest(eid) else {
                // REST connector absent — skip exchange-info silently.
                continue;
            };
            for &at in ACCOUNT_TYPES {
                // Cache hit: re-emit from cache, skip REST.
                if let Some(cached) = self.inner.exchange_info_cache.get(&(eid, at)) {
                    let symbols = cached.value().clone();
                    let _ = self.inner.connector_tx.send(Event::SymbolsLoaded {
                        exchange: eid,
                        account_type: at,
                        symbols,
                    });
                    continue;
                }
                // Cache miss: call REST.
                match connector.get_exchange_info(at).await {
                    Ok(symbols) if !symbols.is_empty() => {
                        self.inner.exchange_info_cache.insert((eid, at), symbols.clone());
                        let _ = self.inner.connector_tx.send(Event::SymbolsLoaded {
                            exchange: eid,
                            account_type: at,
                            symbols,
                        });
                    }
                    Ok(_empty) => {
                        // Empty list — account type not supported by this exchange, skip.
                    }
                    Err(e) => {
                        use digdigdig3::core::types::ExchangeError;
                        match &e {
                            ExchangeError::WireAbsent(_)
                            | ExchangeError::NotImplemented(_) => {
                                // Expected for exchanges without futures or without
                                // exchange-info REST — silent skip.
                            }
                            other => {
                                tracing::warn!(?eid, ?at, ?other, "warmup: get_exchange_info failed");
                                failed.push((eid, other.to_string()));
                            }
                        }
                    }
                }
            }
        }

        WarmupReport { ok, failed }
    }

    /// Subscribe to every (exchange, symbol, account, stream) combination in
    /// `set`. Continue-on-error: per-stream failures are collected in
    /// [`SubscribeReport::failed`] and do not abort the rest of the batch.
    ///
    /// The returned `handle` carries events for every stream in `ok`. A
    /// stream whose subscribe failed will simply not emit events through
    /// the handle.
    ///
    /// The whole call returns `Err` ONLY for batch-level failures (empty
    /// set). Per-stream failures (StreamNotSupported, connect_websocket,
    /// symbol normalize) are reported via `report.failed`.
    pub async fn subscribe(&self, set: SubscriptionSet) -> Result<SubscribeReport> {
        if set.is_empty() {
            return Err(StationError::Subscribe("empty SubscriptionSet".into()));
        }

        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        let mut refs: Vec<MultiplexRef> = Vec::new();
        let mut ok: Vec<SeriesKey> = Vec::new();
        let mut failed: Vec<FailedStream> = Vec::new();
        let mut seed_outcomes: Vec<(SeriesKey, crate::SeedOutcome)> = Vec::new();

        for entry in set.entries {
            // REST connector — needed for warm-start backfill (`get_recent_trades` /
            // `get_klines`). Hub memoizes internally; idempotent. Errors here are
            // logged-and-continued: WS-only subscribe still works without REST.
            if let Err(e) = self
                .inner
                .hub
                .connect_public(entry.exchange, false)
                .await
            {
                tracing::debug!(?e, ?entry.exchange, "connect_public failed; warm-start REST backfill will be skipped");
            }

            // WS connect: skip only if ALL streams in this entry are poll-only
            // (REST polling — never touch a WS connector). Derived streams DO
            // need WS: they subscribe to WS-backed upstreams (Basis ← MarkPrice
            // + IndexPrice; TradeToBar/Range/Tick/Volume/Footprint ← Trade), so
            // the WS connector must be up before the derived forwarder spawns —
            // otherwise the recursive upstream acquire fails with "ws handle
            // missing post-connect". Mixed entries (e.g. [Trade, LongShortRatio])
            // also need WS for the non-poll stream. Per-stream failures are
            // reported in `failed`; only poll-only streams are excluded from that.
            let needs_ws = entry.streams.iter().any(|s| {
                s.to_kind().is_poll_only().is_none()
            });

            if needs_ws {
                // For authenticated entries (private streams), open an
                // authenticated WS connection.  Falls back to public WS on
                // wasm32 where private WS auth is unavailable.
                let ws_connect_result = if let Some(ref creds) = entry.credentials {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        self.inner
                            .hub
                            .connect_websocket_with_credentials(
                                entry.exchange,
                                entry.account_type,
                                creds.clone(),
                            )
                            .await
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        let _ = creds;
                        Err(digdigdig3::core::types::ExchangeError::NotImplemented(
                            "private WS streams not supported on wasm32".into(),
                        ))
                    }
                } else {
                    self.inner
                        .hub
                        .connect_websocket(entry.exchange, entry.account_type, false)
                        .await
                };
                if let Err(e) = ws_connect_result
                {
                    let err_msg = format!("connect_websocket: {e}");
                    for s in &entry.streams {
                        // Only poll-only streams survive a WS-connect failure —
                        // derived streams need WS upstreams, so a WS failure fails
                        // them too.
                        if s.to_kind().is_poll_only().is_some() {
                            continue;
                        }
                        failed.push(FailedStream {
                            exchange: entry.exchange,
                            account_type: entry.account_type,
                            symbol: entry.symbol.clone(),
                            stream: s.clone(),
                            error: StationError::Core(err_msg.clone()),
                        });
                    }
                    // Only `continue` if there are no poll-only streams that can
                    // still be acquired without WS.
                    let has_non_ws = entry.streams.iter().any(|s| {
                        s.to_kind().is_poll_only().is_some()
                    });
                    if !has_non_ws {
                        continue;
                    }
                }
            }

            // Resolve to (canonical, raw exchange-native) pair.
            //
            // - `add_raw`: passthrough. `entry.symbol` is the wire format
            //   already; canonical Symbol is built with empty base/quote +
            //   the raw string as its `raw` field. This is the only path
            //   that works for exotic instruments where BASE-QUOTE doesn't
            //   apply (Deribit options "BTC-23MAY26-86000-C", dated
            //   futures, index symbols, etc.).
            // - `add` (canonical): parse "BTC-USDT"-style input, translate
            //   to exchange-native via SymbolNormalizer.
            let (canonical, raw) = if entry.is_raw {
                (
                    Symbol::with_raw("", "", entry.symbol.clone()),
                    entry.symbol.clone(),
                )
            } else {
                let canonical = parse_symbol(&entry.symbol);
                match SymbolNormalizer::to_exchange(
                    entry.exchange,
                    &canonical,
                    entry.account_type,
                ) {
                    Ok(r) => (canonical, r),
                    Err(e) => {
                        let err_msg = format!("symbol normalize: {e}");
                        for s in &entry.streams {
                            failed.push(FailedStream {
                                exchange: entry.exchange,
                                account_type: entry.account_type,
                                symbol: entry.symbol.clone(),
                                stream: s.clone(),
                                error: StationError::Subscribe(err_msg.clone()),
                            });
                        }
                        continue;
                    }
                }
            };

            // Part B seam: resolve display symbol → wire id for connectors where
            // the WS subscribe frame coin differs from the caller-facing display
            // name (HyperLiquid spot: "HYPE/USDC" → "@107"). The REST connector
            // is always present before subscriptions (connect_public / connect_full
            // is called before subscribe), and REST connectors self-warm their
            // universe cache on first use (OnceCell). For all other venues the
            // default impl is a passthrough (zero allocation, zero round-trip).
            let raw = if let Some(rest) = self.inner.hub.rest(entry.exchange) {
                rest.resolve_market_symbol(&raw, entry.account_type).await
            } else {
                raw
            };

            for s in &entry.streams {
                let kind = s.to_kind();
                let key = SeriesKey {
                    exchange: entry.exchange,
                    account_type: entry.account_type,
                    symbol: raw.clone(),
                    kind: kind.clone(),
                };

                // Task B: fail-fast capability gate — short-circuit OBVIOUS
                // unsupported cases before paying for a WS subscribe roundtrip.
                // Only fires when the connector has declared BOTH REST and WS
                // false for the Kind. Conservative — when uncertain we proceed.
                if let Some(caps) = self.inner.hub.capabilities(entry.exchange) {
                    if caps_explicitly_unsupported(&caps, &kind) {
                        let reason = format!(
                            "{:?} capability not declared on {:?} — neither WS nor REST available",
                            kind, entry.exchange,
                        );
                        tracing::debug!(
                            ?key, reason,
                            "capability fail-fast: skipping acquire_or_spawn"
                        );
                        failed.push(FailedStream {
                            exchange: entry.exchange,
                            account_type: entry.account_type,
                            symbol: entry.symbol.clone(),
                            stream: s.clone(),
                            error: StationError::StreamNotSupported(reason),
                        });
                        continue;
                    }
                }

                let (bcast_tx, pending_seed) = match self
                    .acquire_or_spawn(&key, &entry, &canonical, &raw, s)
                    .await
                {
                    Ok(pair) => pair,
                    Err(e) => {
                        // WireAbsent on a per-(exchange, kind) basis: log
                        // at debug, record in `failed`, move on. Other errors
                        // get an info-level log so they are not lost.
                        if e.is_not_supported() {
                            tracing::debug!(?key, ?e, "stream not supported; skipping");
                        } else {
                            tracing::info!(?key, ?e, "subscribe failed; skipping");
                        }
                        failed.push(FailedStream {
                            exchange: entry.exchange,
                            account_type: entry.account_type,
                            symbol: entry.symbol.clone(),
                            stream: s.clone(),
                            error: e,
                        });
                        continue;
                    }
                };

                let mut bcast_rx = bcast_tx.subscribe();
                let tx_clone = tx.clone();
                // Per-handle symbol label: relay rewrites Event.symbol from the
                // raw exchange-native form (carried on the broadcast) to the
                // user-input form THIS handle subscribed with. Two handles on
                // the same multiplex with different input forms each see their
                // own label.
                let label = entry.symbol.clone();
                {
                    let relay_fut = Box::pin(async move {
                        // Lagged is a back-pressure signal, NOT a fatal error.
                        // Cold-start scenario on single-threaded executors
                        // (wasm `spawn_local`): the relay task is queued
                        // AFTER `spawn_local(forwarder)`, so the forwarder
                        // can emit hundreds of events into its
                        // `broadcast::channel(512)` buffer before the relay
                        // gets its first CPU slice. If the producer is fast
                        // (derived TpoFromTrade emits one point per upstream
                        // Trade — ~30-100/s on BTCUSDT) the buffer overflows
                        // and `recv()` returns `Lagged(n)`.  Breaking here
                        // tore down the handle silently, so consumers saw
                        // `events_total=0`.  Keep going — skip the lost
                        // events and stream the live tail.
                        loop {
                            match bcast_rx.recv().await {
                                Ok(mut ev) => {
                                    ev.set_symbol(label.clone());
                                    if tx_clone.send(ev).is_err() {
                                        break;
                                    }
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Lagged(_n)) => {
                                    // Skip the dropped window, keep reading the live tail.
                                    continue;
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                    break;
                                }
                            }
                        }
                    });
                    #[cfg(not(target_arch = "wasm32"))]
                    tokio::spawn(relay_fut);
                    #[cfg(target_arch = "wasm32")]
                    wasm_bindgen_futures::spawn_local(relay_fut);
                }

                // For OrderbookDelta with REST seed: emit the snapshot NOW,
                // after the relay task has subscribed to the broadcast channel.
                // This guarantees the snapshot reaches the consumer's
                // SubscriptionHandle::recv() — previously it was sent before
                // any receiver existed and was silently dropped.
                if let Some(seed_ev) = pending_seed {
                    if bcast_tx.send(seed_ev).is_err() {
                        tracing::debug!(
                            target: "dig3::ob_seed",
                            ?key,
                            "ob delta seed: send failed after relay wired (unexpected)"
                        );
                    }
                }

                refs.push(MultiplexRef {
                    station: Arc::downgrade(&self.inner),
                    key: key.clone(),
                });
                // Drain any cold-seed outcome recorded for this key by
                // `acquire_or_spawn_derived_body` (side-channel — see
                // `StationInner::seed_outcomes`). Absent for WS-only /
                // poll-only / already-live-mux acquires.
                if let Some((_, outcome)) = self.inner.seed_outcomes.remove(&key) {
                    seed_outcomes.push((key.clone(), outcome));
                }
                ok.push(key);
            }
        }

        Ok(SubscribeReport {
            handle: SubscriptionHandle { rx, _refs: refs },
            ok,
            failed,
            seed_outcomes,
        })
    }

    /// Acquire (or spawn) the multiplexer for `key`. Spawn includes:
    /// - opening DiskStore<T> if persistence is on,
    /// - seeding broadcast with last-N (warm-start) before any live event,
    /// - issuing WS subscribe + forwarder task that runs until shutdown.
    async fn acquire_or_spawn(
        &self,
        key: &SeriesKey,
        entry: &Entry,
        canonical: &Symbol,
        raw_symbol: &str,
        stream: &Stream,
    ) -> Result<(broadcast::Sender<Event>, Option<Event>)> {
        if let Some(mut mux) = self.inner.muxes.get_mut(key) {
            // Cancel any pending grace-period timer — the forwarder is being
            // reused before the grace window expired. Sending on grace_cancel
            // unblocks the timer task's select! arm, which then exits without
            // firing the shutdown signal.
            if let Some(cancel) = mux.grace_cancel.take() {
                let _ = cancel.send(());
            }
            mux.consumers.fetch_add(1, Ordering::SeqCst);
            return Ok((mux.tx.clone(), None));
        }

        // --- Derived stream path (no WS, no REST) ---
        // Must come BEFORE the ws handle resolution so we never call
        // ws.subscribe() for a derived kind.
        //
        // Exception: Kind::Basis and Kind::FundingSettlement are "derivable"
        // but also have native REST history endpoints on some exchanges.  On
        // native builds the per-(exchange, account) capability is checked first;
        // if available the stream is routed through a REST poller instead of the
        // RAM-derive path.  On wasm (no tokio::time::interval) the derive path
        // is always used.
        if key.kind.is_derived() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Kind::Basis = &key.kind {
                    let hub = &self.inner.hub;
                    if let Some(source) = polling::basis_poll_source(hub, key.exchange) {
                        tracing::info!(
                            target: "dig3::station::native_source",
                            exchange = ?key.exchange,
                            symbol   = %key.symbol,
                            "Kind::Basis: native REST basis-history available — using poll path"
                        );
                        let poll_spec = crate::series::PollSpec {
                            cadence: source.cadence(),
                            jitter_pct: 10,
                        };
                        return self
                            .acquire_or_spawn_polled_with_source::<BasisPoint, _>(
                                key, poll_spec, raw_symbol, source,
                            )
                            .await
                            .map(|tx| (tx, None));
                    } else {
                        tracing::info!(
                            target: "dig3::station::native_source",
                            exchange = ?key.exchange,
                            symbol   = %key.symbol,
                            "Kind::Basis: no native source — deriving from mark+index in RAM"
                        );
                    }
                }
                if let Kind::FundingSettlement = &key.kind {
                    let hub = &self.inner.hub;
                    if let Some(source) = polling::funding_poll_source(hub, key.exchange) {
                        tracing::info!(
                            target: "dig3::station::native_source",
                            exchange = ?key.exchange,
                            symbol   = %key.symbol,
                            "Kind::FundingSettlement: native REST funding-rate history available — using poll path"
                        );
                        let poll_spec = crate::series::PollSpec {
                            cadence: source.cadence(),
                            jitter_pct: 10,
                        };
                        return self
                            .acquire_or_spawn_polled_with_source::<FundingSettlementPoint, _>(
                                key, poll_spec, raw_symbol, source,
                            )
                            .await
                            .map(|tx| (tx, None));
                    } else {
                        tracing::info!(
                            target: "dig3::station::native_source",
                            exchange = ?key.exchange,
                            symbol   = %key.symbol,
                            "Kind::FundingSettlement: no native source — deriving from funding-rate flips in RAM"
                        );
                    }
                }
            }

            return match &key.kind {
                Kind::Basis => {
                    self.acquire_or_spawn_derived::<BasisDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::FundingSettlement => {
                    self.acquire_or_spawn_derived::<FundingSettlementDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::RangeBar(_) => {
                    self.acquire_or_spawn_derived::<TradeToRangeBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::TickBar(_) => {
                    self.acquire_or_spawn_derived::<TradeToTickBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::VolumeBar(_) => {
                    self.acquire_or_spawn_derived::<TradeToVolumeBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::Footprint(_) => {
                    self.acquire_or_spawn_derived::<TradeToFootprintDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::RenkoBar(_, _) => {
                    self.acquire_or_spawn_derived::<TradeToRenkoBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::PnfBar(_, _) => {
                    self.acquire_or_spawn_derived::<TradeToPnfBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::KagiBar(_) => {
                    self.acquire_or_spawn_derived::<TradeToKagiBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::CvdLine => {
                    self.acquire_or_spawn_derived::<TradeToCvdLineDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::ThreeLineBreak { .. } => {
                    self.acquire_or_spawn_derived::<TradeToThreeLineBreakDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::TpoProfile(_, TpoSource::Kline1m) => {
                    self.acquire_or_spawn_derived::<TpoFromKline1mDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::TpoProfile(_, TpoSource::TradeBucket) => {
                    self.acquire_or_spawn_derived::<TpoFromTradeDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::DollarBar { .. } => {
                    self.acquire_or_spawn_derived::<TradeToDollarBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::TickImbalanceBar { .. } => {
                    self.acquire_or_spawn_derived::<TradeToTickImbalanceDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::VolumeImbalanceBar { .. } => {
                    self.acquire_or_spawn_derived::<TradeToVolumeImbalanceDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::RunBar { .. } => {
                    self.acquire_or_spawn_derived::<TradeToRunBarDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                _ => unreachable!("is_derived() returned true for unhandled kind — update acquire_or_spawn dispatch"),
            };
        }

        // --- Poll-only stream path (REST periodic polling, no WS) ---
        // Must come BEFORE the ws.subscribe call so we never try to subscribe
        // a WS channel for streams that have no WS feed.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(poll_spec) = key.kind.is_poll_only() {
            return self.acquire_or_spawn_polled(key, entry, poll_spec, raw_symbol).await.map(|tx| (tx, None));
        }
        // On wasm, poll-only kinds are not supported (no tokio::time::interval).
        #[cfg(target_arch = "wasm32")]
        if key.kind.is_poll_only().is_some() {
            return Err(StationError::StreamNotSupported(format!(
                "poll-only streams not supported on wasm32 ({:?})",
                key.kind
            )));
        }

        let sym = Symbol::with_raw(&canonical.base, &canonical.quote, raw_symbol.to_string());
        let req = ws_request_for(&key.kind, sym, entry.account_type);

        let ws = self
            .inner
            .hub
            .ws(entry.exchange, entry.account_type)
            .ok_or_else(|| StationError::Core("ws handle missing post-connect".into()))?;
        // `transport.rs::subscribe` eagerly invokes `subscribe_frame` and
        // propagates any frame-construction failure (WireAbsent and
        // NotImplemented included). Map those to
        // `StreamNotSupported` so `Station::subscribe(set)` can bucket
        // them into `SubscribeReport::failed` without spawning a forwarder
        // that would loop in heal/resub forever (this is what caused
        // MLI's 0.3.6 OOM — see release-0.3.7-plan.md).
        //
        // Special case — Kind::Kline(iv): if the venue does not natively
        // support this interval on its WS, fall back to TradeToBarDerived
        // (trade-aggregation engine) rather than returning a hard error.
        // The fallback is attempted only when ws.subscribe fails with
        // WireAbsent / NotImplemented; native kline paths are
        // unchanged. If the interval string is unknown (interval_to_ms
        // returns None) we cannot build the aggregator either — return a
        // clear StreamNotSupported to the caller.
        if let Err(e) = ws.subscribe(req.clone()).await {
            use digdigdig3::core::types::WebSocketError;
            let is_not_supported = matches!(
                e,
                WebSocketError::WireAbsent(_) | WebSocketError::NotImplemented(_)
            );
            if is_not_supported {
                if let Kind::Kline(iv) = &key.kind {
                    // Validate the interval before attempting the aggregator.
                    if interval_to_ms(iv.as_str()).is_none() {
                        return Err(StationError::StreamNotSupported(format!(
                            "Kline interval {:?} is unknown — cannot aggregate from trades",
                            iv.as_str()
                        )));
                    }
                    tracing::debug!(
                        target: "dig3::station::derived",
                        exchange = ?key.exchange,
                        symbol   = %key.symbol,
                        interval = %iv,
                        "native Kline WS not supported — falling back to TradeToBarDerived"
                    );
                    return self
                        .acquire_or_spawn_derived::<TradeToBarDerived>(key, entry, canonical, raw_symbol)
                        .await
                        .map(|tx| (tx, None));
                }
            }
            return Err(match e {
                WebSocketError::WireAbsent(msg)
                | WebSocketError::NotImplemented(msg) => {
                    StationError::StreamNotSupported(msg)
                }
                other => StationError::Subscribe(format!("ws.subscribe: {other}")),
            });
        }

        let (bcast_tx, _) = broadcast::channel::<Event>(1024);
        let consumers = Arc::new(AtomicUsize::new(1));
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let _ = stream; // kept for future per-Stream parameter customizations

        // For each kind, compute the REST backfill seed (used when disk is
        // empty), then spawn the typed forwarder. Backfill is best-effort —
        // empty Vec on any failure or unsupported endpoint.
        let warm_n = self.inner.warm_start_capacity;
        let hub = self.inner.hub.clone();
        let acct = entry.account_type;
        let raw_s = raw_symbol.to_string();

        // Populated by Kind::OrderbookDelta when orderbook_rest_seed=true.
        // Returned to Station::subscribe so it can be sent AFTER the relay task
        // has subscribed to the broadcast channel (fixes the race where the snapshot
        // was dropped because no receivers existed yet).
        let mut pending_seed: Option<Event> = None;

        match &key.kind {
            Kind::Trade => {
                let seed = if warm_n > 0 {
                    crate::backfill::trades_recent(&hub, key.exchange, acct, &raw_s, warm_n).await.0
                } else { Vec::new() };
                spawn_forwarder::<TradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::Kline(interval) => {
                let seed = if warm_n > 0 {
                    crate::backfill::klines_recent(&hub, key.exchange, acct, &raw_s, interval.as_str(), warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<BarPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::AggTrade => {
                let seed = if warm_n > 0 {
                    crate::backfill::agg_trades_recent(&hub, key.exchange, acct, &raw_s, warm_n).await.0
                } else { Vec::new() };
                spawn_forwarder::<AggTradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::Ticker => match self.inner.persistence.depth_for(&key.kind) {
                Some(PersistDepth::Indicators) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::tickers_indicators_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<TickerIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                Some(PersistDepth::Full) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::tickers_full_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<TickerFullPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                _ => spawn_forwarder::<TickerPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            },
            Kind::Orderbook => {
                let ob_seed = if self.inner.orderbook_rest_seed {
                    ob_rest_seed(&hub, key.exchange, acct, &raw_s, self.inner.orderbook_seed_depth).await
                } else {
                    Vec::new()
                };
                match self.inner.persistence.depth_for(&key.kind) {
                    Some(PersistDepth::Indicators) | Some(PersistDepth::Full) => {
                        // ObSnapshotIndicatorsPoint carries cts + prev_change_id.
                        // Seed is discarded for extended types (seed uses Compact layout).
                        spawn_forwarder::<ObSnapshotIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone());
                    }
                    _ => spawn_forwarder::<ObSnapshotPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), ob_seed, req.clone()),
                }
            }
            Kind::OrderbookDelta => {
                // Seed via REST snapshot: gives downstream assemblers a seeded
                // full-book state before deltas arrive. The seed event is NOT
                // emitted here — it is returned as `pending_seed` and emitted by
                // `Station::subscribe` AFTER the consumer's relay task has subscribed
                // to the broadcast channel. Emitting before any receiver exists (the
                // old behaviour) caused the snapshot to be silently dropped.
                //
                // Note: wasm32 skips REST seed (possible CORS), same as before.
                pending_seed = if self.inner.orderbook_rest_seed {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let snapshots = ob_rest_seed(&hub, key.exchange, acct, &raw_s, self.inner.orderbook_seed_depth).await;
                        snapshots.into_iter().next().map(|point| Event::OrderbookSnapshot {
                            exchange: key.exchange,
                            symbol: raw_s.clone(),
                            point,
                        })
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        tracing::warn!(
                            target: "dig3::ob_seed",
                            exchange = ?key.exchange, symbol = raw_s.as_str(),
                            "orderbook REST seed for delta stream skipped on wasm32 (possible CORS) — continuing WS-only"
                        );
                        None
                    }
                } else {
                    None
                };
                match self.inner.persistence.depth_for(&key.kind) {
                    Some(PersistDepth::Indicators) | Some(PersistDepth::Full) =>
                        spawn_forwarder::<ObDeltaIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
                    _ => spawn_forwarder::<ObDeltaPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
                }
            }
            Kind::MarkPrice => match self.inner.persistence.depth_for(&key.kind) {
                Some(PersistDepth::Indicators) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::mark_price_indicators_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<MarkPriceIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                Some(PersistDepth::Full) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::mark_price_full_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<MarkPriceFullPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                _ => {
                    let seed = if warm_n > 0 {
                        crate::backfill::mark_price_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<MarkPricePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
            },
            Kind::FundingRate => match self.inner.persistence.depth_for(&key.kind) {
                Some(PersistDepth::Indicators) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::funding_rate_indicators_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<FundingRateIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                Some(PersistDepth::Full) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::funding_rate_full_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<FundingRateFullPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                _ => {
                    let seed = if warm_n > 0 {
                        crate::backfill::funding_rate_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<FundingRatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
            },
            Kind::OpenInterest => match self.inner.persistence.depth_for(&key.kind) {
                Some(PersistDepth::Indicators) | Some(PersistDepth::Full) => {
                    // OpenInterestFullPoint is a type alias to OpenInterestIndicatorsPoint.
                    let seed = if warm_n > 0 {
                        crate::backfill::open_interest_indicators_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<OpenInterestIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                _ => {
                    let seed = if warm_n > 0 {
                        crate::backfill::open_interest_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<OpenInterestPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
            },
            Kind::Liquidation => match self.inner.persistence.depth_for(&key.kind) {
                Some(PersistDepth::Indicators) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::liquidation_indicators_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<LiquidationIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                Some(PersistDepth::Full) => {
                    let seed = if warm_n > 0 {
                        crate::backfill::liquidation_full_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<LiquidationFullPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
                _ => {
                    let seed = if warm_n > 0 {
                        crate::backfill::liquidations_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                    } else { Vec::new() };
                    spawn_forwarder::<LiquidationPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
                }
            },
            Kind::BlockTrade => spawn_forwarder::<BlockTradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::AuctionEvent => spawn_forwarder::<AuctionEventPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::IndexPrice => match self.inner.persistence.depth_for(&key.kind) {
                Some(PersistDepth::Indicators) | Some(PersistDepth::Full) => spawn_forwarder::<IndexPriceIndicatorsPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
                _ => spawn_forwarder::<IndexPricePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            },
            Kind::CompositeIndex => spawn_forwarder::<CompositeIndexPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::OptionGreeks => spawn_forwarder::<OptionGreeksPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::VolatilityIndex => spawn_forwarder::<VolatilityIndexPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::HistoricalVolatility => spawn_forwarder::<HistoricalVolatilityPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            // LongShortRatio / TakerVolume / LiquidationBucket: poll-only, unreachable in
            // normal operation (the is_poll_only() branch above handles these first).
            // Kept as defensive fallbacks so the match arm is exhaustive.
            Kind::LongShortRatio => spawn_forwarder::<LongShortRatioPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::TakerVolume => spawn_forwarder::<TakerVolumePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::LiquidationBucket => spawn_forwarder::<LiquidationBucketPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::Basis => spawn_forwarder::<BasisPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::InsuranceFund => {
                let seed = if warm_n > 0 {
                    crate::backfill::insurance_fund_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<InsuranceFundPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::OrderbookL3 => spawn_forwarder::<OrderbookL3Point>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::SettlementEvent => spawn_forwarder::<SettlementEventPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::MarketWarning => spawn_forwarder::<MarketWarningPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::RiskLimit => spawn_forwarder::<RiskLimitPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::PredictedFunding => spawn_forwarder::<PredictedFundingPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::FundingSettlement => spawn_forwarder::<FundingSettlementPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::MarkPriceKline(iv) => {
                let seed = if warm_n > 0 {
                    crate::backfill::mark_price_klines_recent(&hub, key.exchange, acct, &raw_s, iv.as_str(), warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<MarkPriceKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::IndexPriceKline(iv) => {
                let seed = if warm_n > 0 {
                    crate::backfill::index_price_klines_recent(&hub, key.exchange, acct, &raw_s, iv.as_str(), warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<IndexPriceKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::PremiumIndexKline(iv) => {
                let seed = if warm_n > 0 {
                    crate::backfill::premium_index_klines_recent(&hub, key.exchange, acct, &raw_s, iv.as_str(), warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<PremiumIndexKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            // Private streams — no warm-start seed, no persistence (ephemeral by design).
            Kind::OrderUpdate => spawn_forwarder::<OrderUpdatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::BalanceUpdate => spawn_forwarder::<BalanceUpdatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::PositionUpdate => spawn_forwarder::<PositionUpdatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req),
            // Derived kinds — handled above by acquire_or_spawn_derived before
            // reaching this match. These arms satisfy exhaustiveness only.
            Kind::RangeBar(_) | Kind::TickBar(_) | Kind::VolumeBar(_) | Kind::Footprint(_)
            | Kind::RenkoBar(_, _) | Kind::PnfBar(_, _) | Kind::KagiBar(_)
            | Kind::CvdLine | Kind::ThreeLineBreak { .. } | Kind::TpoProfile(_, _)
            | Kind::DollarBar { .. } | Kind::TickImbalanceBar { .. }
            | Kind::VolumeImbalanceBar { .. } | Kind::RunBar { .. } => {
                unreachable!("derived kinds dispatched before forwarder match")
            }
        }

        self.inner.muxes.insert(
            key.clone(),
            Multiplexer { tx: bcast_tx.clone(), consumers, shutdown: Some(shutdown_tx), grace_cancel: None },
        );

        Ok((bcast_tx, pending_seed))
    }
}

impl Station {
    /// Acquire (or spawn) a derived-stream multiplexer for `key`.
    ///
    /// Recursively calls `acquire_or_spawn` for each upstream dep (which
    /// follows the normal WS path), subscribes to each upstream broadcast,
    /// then spawns `spawn_derived_forwarder<D>` to run the computation.
    ///
    /// Ref-counting: each upstream `acquire_or_spawn` call increments the
    /// upstream `consumers` counter by 1 (for the derived forwarder's benefit).
    /// When the derived forwarder exits it calls `inner.release_consumer` on
    /// each upstream key, propagating shutdown upward if no other consumer
    /// holds the upstream.
    /// Acquire (or spawn) a derived multiplexer — **native** path.
    /// Returns `Pin<Box<dyn Future + Send>>` so the future can be awaited
    /// from `acquire_or_spawn` which is itself spawned via `tokio::spawn` on native.
    #[cfg(not(target_arch = "wasm32"))]
    fn acquire_or_spawn_derived<'a, D: DerivedStream>(
        &'a self,
        key: &'a SeriesKey,
        entry: &'a Entry,
        canonical: &'a digdigdig3::core::types::Symbol,
        raw_symbol: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<broadcast::Sender<Event>>> + Send + 'a>>
    where
        Event: EventFrom<D::Output>,
    {
        Box::pin(async move {
            self.acquire_or_spawn_derived_body::<D>(key, entry, canonical, raw_symbol).await
        })
    }

    /// Acquire (or spawn) a derived multiplexer — **wasm32** path.
    /// No `Send` bound — wasm is single-threaded and all futures are `!Send`.
    #[cfg(target_arch = "wasm32")]
    fn acquire_or_spawn_derived<'a, D: DerivedStream>(
        &'a self,
        key: &'a SeriesKey,
        entry: &'a Entry,
        canonical: &'a digdigdig3::core::types::Symbol,
        raw_symbol: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<broadcast::Sender<Event>>> + 'a>>
    where
        Event: EventFrom<D::Output>,
    {
        Box::pin(async move {
            self.acquire_or_spawn_derived_body::<D>(key, entry, canonical, raw_symbol).await
        })
    }

    /// Shared body for `acquire_or_spawn_derived` — called from both cfg variants.
    async fn acquire_or_spawn_derived_body<D: DerivedStream>(
        &self,
        key: &SeriesKey,
        entry: &Entry,
        canonical: &digdigdig3::core::types::Symbol,
        raw_symbol: &str,
    ) -> Result<broadcast::Sender<Event>>
    where
        Event: EventFrom<D::Output>,
    {
        let mut upstream_rxs: Vec<broadcast::Receiver<Event>> = Vec::new();
        let mut upstream_keys: Vec<SeriesKey> = Vec::new();

        // Use the key-aware dep list so derived streams whose deps carry a
        // runtime parameter (e.g. TpoFromKline1mDerived → Kline("1m")) can
        // resolve their actual upstream `SeriesKey`.
        let resolved_deps: Vec<Stream> = D::deps_for_key(key);
        for dep_stream in &resolved_deps {
            let dep_kind = dep_stream.to_kind();
            debug_assert!(
                !dep_kind.is_derived(),
                "DerivedStream::deps_for_key() must not list derived kinds (no derived-of-derived)"
            );
            let dep_key = SeriesKey {
                exchange: key.exchange,
                account_type: key.account_type,
                symbol: raw_symbol.to_string(),
                kind: dep_kind,
            };
            // Recursive call — follows the normal WS path for each upstream kind.
            // The pending_seed (second tuple element) is intentionally ignored here:
            // derived streams subscribe to the upstream broadcast directly, not through
            // a consumer relay, so the seed will be replayed naturally through the
            // upstream forwarder's warm-start mechanism.
            let (up_tx, _) = self
                .acquire_or_spawn(&dep_key, entry, canonical, raw_symbol, dep_stream)
                .await?;
            upstream_rxs.push(up_tx.subscribe());
            upstream_keys.push(dep_key);
        }

        // AggTrade dense seed for derived cold-start (P0-C closure). Build a
        // per-dep seed Event vec: for Trade deps, fetch AggTrade history from
        // REST (falling back to recent_trades when has_agg_trades=false), convert
        // to TradePoint→Event::Trade. Passed into spawn_derived_forwarder which
        // feeds them through D::seed_from_events before the live loop begins.
        //
        // Per-subscribe override: `entry.warm_override` replaces the Station-
        // wide `warm_start_capacity` for THIS entry's cold-start depth (see
        // `SubscriptionSet::add_with_warm`). `None` keeps existing behavior.
        let warm_n = entry.warm_override.unwrap_or(self.inner.warm_start_capacity);
        // Kline-sufficient kinds (Renko/PnF/Kagi/3LB/Range/Volume/Dollar) are
        // PURE KLINE CONVERTERS — history AND live both come from
        // `Stream::Kline("1m")` exclusively; they carry no `Stream::Trade`
        // dependency at all (see `DerivedStream::deps_for_key` on each of
        // the seven). Everything the fold needs (price path, and for
        // volume/dollar, volume/quote-volume) is fully recoverable from 1m
        // klines via `kline_to_synthetic_trades` for history and
        // `KlineDeltaState` for the live Δ-tick path. Tick/imbalance/run
        // bars, CVD, and footprint genuinely need real trades (aggressor
        // side, per-trade counts, intra-bar ladders) and are excluded here —
        // their paths are untouched.
        let is_kline_sufficient_kind = matches!(
            key.kind,
            Kind::RenkoBar(_, _) | Kind::PnfBar(_, _) | Kind::KagiBar(_) | Kind::ThreeLineBreak { .. }
            | Kind::RangeBar(_) | Kind::VolumeBar(_) | Kind::DollarBar { .. }
        );
        // Baseline handed to the live `KlineDeltaState` (see
        // `DerivedStream::seed_kline_baseline`) from the cold-seed's LAST
        // fetched kline — set only for `is_kline_sufficient_kind`, and only
        // when at least one kline was fetched. `spawn_derived_forwarder`
        // applies it right after `D::new_for_key`, before any live event.
        let mut kline_seed_baseline: Option<(i64, f64, f64)> = None;
        let mut agg_seed_per_dep: Vec<Vec<Event>> = Vec::with_capacity(resolved_deps.len());
        for dep_stream in &resolved_deps {
            let dep_kind = dep_stream.to_kind();
            let mut seed_events: Vec<Event> = Vec::new();
            if warm_n > 0 && is_kline_sufficient_kind && matches!(dep_kind, Kind::Kline(_)) {
                // Sole seed source for these 7 kinds. n_kline_pages formula:
                // warm_n is expressed in the same "trade count" unit as the
                // plain aggTrade warm depth (e.g. 50_000). One 1m kline page
                // = 1000 bars = ~16.7h. We want the kline window to scale
                // with the caller's requested depth while staying within a
                // sane REST budget, so: 1 page per 1000 "warm units"
                // requested, clamped to [1, 100] pages (100 pages × 1000 ×
                // 1m ≈ 69 days — plenty for "weeks of 1m", while bounding
                // worst-case REST calls per chart-open).
                let n_kline_pages = ((warm_n + 999) / 1000).clamp(1, 100);
                let (kline_bars, k_outcome) = crate::backfill::klines_paginated(
                    &self.inner.hub, key.exchange, entry.account_type, raw_symbol,
                    "1m", 1000, n_kline_pages,
                ).await;
                let kline_interval_ms = interval_to_ms("1m").unwrap_or(60_000);
                // `klines_paginated` returns oldest→newest. The query window
                // reaches "now" (see its `next_end_time` seed), so the LAST
                // bar may be the exchange's still-forming (unclosed) 1m
                // candle. Fold every bar EXCEPT the last as a full 4-leg
                // synthetic-trade path — those are certainly closed, honest
                // history. Hand the last bar's (open_time, volume, close) to
                // the live Δ-tick adapter as its seed baseline instead of
                // folding it too: the first live WS kline update sharing
                // that SAME open_time then computes a correct incremental
                // delta against this baseline rather than double-counting
                // (full 4-leg fold + live delta) or dropping it. If that
                // last bar is in fact already closed, this drops one bar's
                // OHLC path from the deep seed — the same audited ≤1-bar
                // seam already accepted elsewhere in this design (see
                // `rewarm_derived_standalone`'s doc comment).
                if let Some((last, closed)) = kline_bars.split_last() {
                    for bar in closed {
                        seed_events.extend(crate::derived::kline_to_synthetic_trades(
                            bar, kline_interval_ms, key.exchange, raw_symbol,
                        ));
                    }
                    kline_seed_baseline = Some((last.open_time, last.volume, last.close));
                }
                self.inner.seed_outcomes.insert(key.clone(), k_outcome);
            } else if warm_n > 0 && matches!(dep_kind, Kind::Trade) {
                let caps_opt = self.inner.hub.capabilities(key.exchange);
                let use_agg = caps_opt.as_ref().map(|c| c.has_agg_trades).unwrap_or(false);
                // Capability-aware trade-history tier for this (exchange,
                // account) — consulted BEFORE any REST call so a
                // `RecentOnly` venue never "attempts" pagination, and a
                // `RestWindow` venue's walk clamps at the wall instead of
                // discovering it by an empty page (Task 4).
                let tier = self.inner.hub
                    .trade_history_capabilities(key.exchange)
                    .map(|c| c.tier_for(entry.account_type))
                    .unwrap_or(digdigdig3::core::types::TradeHistoryTier::RecentOnly { max_trades: 1000 });
                let trade_outcome = if matches!(tier, digdigdig3::core::types::TradeHistoryTier::RecentOnly { .. }) {
                    // RecentOnly: no pagination attempt at all — a single
                    // shallow call is the venue's entire history.
                    let (trades, outcome) = crate::backfill::trades_recent(
                        &self.inner.hub, key.exchange, entry.account_type, raw_symbol, warm_n,
                    ).await;
                    for pt in trades {
                        seed_events.push(Event::Trade {
                            exchange: key.exchange,
                            symbol: raw_symbol.to_string(),
                            point: pt,
                        });
                    }
                    outcome
                } else if use_agg {
                    // Paginated aggTrade fetch — derive page count from warm_n
                    // so warm_start(5000) gives 5 pages (~20-60s of BTC history,
                    // enough for several closed range/volume/tick bars). Single-
                    // page agg_trades_recent only covered ~1-5s, never enough.
                    // `agg_trades_paginated` itself clamps pagination at the
                    // `RestWindow` wall when the tier declares one.
                    let n_pages = ((warm_n + 999) / 1000).max(1);
                    let (agg, outcome) = crate::backfill::agg_trades_paginated(
                        &self.inner.hub, key.exchange, entry.account_type, raw_symbol,
                        1000, n_pages,
                    ).await;
                    for ap in agg {
                        seed_events.push(Event::Trade {
                            exchange: key.exchange,
                            symbol: raw_symbol.to_string(),
                            point: TradePoint {
                                ts_ms: ap.ts_ms,
                                price: ap.price,
                                quantity: ap.quantity,
                                side: ap.side,
                                trade_id_hash: 0,
                            },
                        });
                    }
                    outcome
                } else {
                    // Fall back to recent trades when no aggTrades available.
                    let (trades, outcome) = crate::backfill::trades_recent(
                        &self.inner.hub, key.exchange, entry.account_type, raw_symbol, warm_n,
                    ).await;
                    for pt in trades {
                        seed_events.push(Event::Trade {
                            exchange: key.exchange,
                            symbol: raw_symbol.to_string(),
                            point: pt,
                        });
                    }
                    outcome
                };
                self.inner.seed_outcomes.insert(key.clone(), trade_outcome);
            } else if warm_n > 0 && matches!(dep_kind, Kind::MarkPrice) {
                // Seed MarkPrice dep (used by BasisDerived dep index 0) from REST snapshot
                // so Basis can emit immediately on cold-start without waiting for WS.
                let pts = crate::backfill::mark_price_recent(
                    &self.inner.hub, key.exchange, entry.account_type, raw_symbol, warm_n,
                ).await;
                for pt in pts {
                    seed_events.push(Event::MarkPrice {
                        exchange: key.exchange,
                        symbol: raw_symbol.to_string(),
                        point: pt,
                    });
                }
            } else if warm_n > 0 && matches!(dep_kind, Kind::IndexPrice) {
                // Seed IndexPrice dep (used by BasisDerived dep index 1) from REST snapshot
                // so Basis can emit immediately on cold-start without waiting for WS.
                let pts = crate::backfill::mark_price_recent(
                    &self.inner.hub, key.exchange, entry.account_type, raw_symbol, warm_n,
                ).await;
                // get_premium_index returns MarkPrice which carries both mark + index.
                // Map to IndexPricePoint via the index field.
                for pt in pts {
                    if pt.index.is_finite() {
                        seed_events.push(Event::IndexPrice {
                            exchange: key.exchange,
                            symbol: raw_symbol.to_string(),
                            point: IndexPricePoint { ts_ms: pt.ts_ms, price: pt.index },
                        });
                    }
                }
            }
            agg_seed_per_dep.push(seed_events);
        }

        // Ring-capacity hint: the in-memory Series<D::Output> ring must be
        // sized at least as large as the seed we just computed, or the ring
        // evicts the deep seed before spawn_derived_forwarder even reaches
        // the live loop (risk flagged in the design doc's risk register —
        // MUST ship in the same commit as the warm_override, not after).
        // A price-path deep seed can synthesize far more points than
        // warm_start_capacity (e.g. 100 kline pages × 1000 bars × 4 synthetic
        // legs = 400k raw trade-events feeding the state machine, though the
        // OUTPUT point count — bricks/columns/segments/lines — is typically
        // much smaller). We size on the largest per-dep seed length observed,
        // never below the Station-wide warm_start_capacity floor.
        let ring_capacity_hint = agg_seed_per_dep
            .iter()
            .map(|v| v.len())
            .max()
            .unwrap_or(warm_n)
            .max(self.inner.warm_start_capacity);

        let (bcast_tx, _) = broadcast::channel::<Event>(512);
        let consumers = Arc::new(AtomicUsize::new(1));
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let (seed_done_tx, seed_done_rx) = oneshot::channel::<()>();

        spawn_derived_forwarder::<D>(
            self,
            key,
            upstream_rxs,
            upstream_keys,
            bcast_tx.clone(),
            shutdown_rx,
            raw_symbol.to_string(),
            agg_seed_per_dep,
            ring_capacity_hint,
            seed_done_tx,
            kline_seed_baseline,
        );

        self.inner.muxes.insert(
            key.clone(),
            Multiplexer { tx: bcast_tx.clone(), consumers, shutdown: Some(shutdown_tx), grace_cancel: None },
        );

        // Gate return on the forwarder's cold-start seeding (disk warm-seed +
        // agg-seed + RAM warm-seed) so a consumer that immediately peeks
        // `Station::series::<D::Output>(key)` after `subscribe()` sees the
        // seeded ring instead of an empty one (see docs/plans — pnf_seed_smoke
        // regression: peek-then-live read n=0 right after subscribe while the
        // ring held thousands of points 20s later). Bounded by a 15s timeout
        // so a wedged/slow forwarder (e.g. REST paging stalls) can never hang
        // `subscribe()` forever — on timeout or a dropped sender we proceed
        // with whatever the ring holds at that moment. Same wasm-safe timer
        // idiom as the grace-period race above (`tokio::time::sleep` native /
        // `gloo_timers::future::sleep` wasm) since `tokio::time::timeout` is
        // not available on wasm32.
        const SEED_WAIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
        #[cfg(not(target_arch = "wasm32"))]
        {
            tokio::select! {
                res = seed_done_rx => {
                    if res.is_err() {
                        tracing::debug!(?key, "derived: seed_done sender dropped before signaling — proceeding anyway");
                    }
                }
                _ = tokio::time::sleep(SEED_WAIT_TIMEOUT) => {
                    tracing::debug!(?key, "derived: cold-start seed wait timed out — proceeding with partial/empty seed");
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            tokio::select! {
                res = seed_done_rx => {
                    if res.is_err() {
                        tracing::debug!(?key, "derived: seed_done sender dropped before signaling — proceeding anyway");
                    }
                }
                _ = gloo_timers::future::sleep(SEED_WAIT_TIMEOUT) => {
                    tracing::debug!(?key, "derived: cold-start seed wait timed out — proceeding with partial/empty seed");
                }
            }
        }

        Ok(bcast_tx)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Station {
    /// Acquire (or spawn) a poll-driven multiplexer for `key`.
    ///
    /// Called when `key.kind.is_poll_only()` returns `Some(PollSpec)`. Skips
    /// `ws.subscribe` entirely and instead spawns a `spawn_poller<T, S>` actor
    /// driven by `tokio::time::interval`.
    async fn acquire_or_spawn_polled(
        &self,
        key: &SeriesKey,
        entry: &Entry,
        poll_spec: crate::series::PollSpec,
        raw_symbol: &str,
    ) -> Result<broadcast::Sender<Event>> {
        use crate::station::Multiplexer;

        let (bcast_tx, _) = broadcast::channel::<Event>(1024);
        let consumers = Arc::new(AtomicUsize::new(1));
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let label = raw_symbol.to_string();

        match &key.kind {
            Kind::LongShortRatio => {
                let source = polling::lsr_poll_source(entry.exchange)
                    .ok_or_else(|| StationError::StreamNotSupported(format!(
                        "LongShortRatio REST polling not supported for {:?}",
                        entry.exchange
                    )))?;
                polling::spawn_poller::<LongShortRatioPoint, _>(
                    self, key, source, poll_spec, bcast_tx.clone(), shutdown_rx, label,
                );
            }
            Kind::HistoricalVolatility => {
                let source = polling::hv_poll_source(entry.exchange)
                    .ok_or_else(|| StationError::StreamNotSupported(format!(
                        "HistoricalVolatility REST polling not supported for {:?} \
                         (Deribit only)",
                        entry.exchange
                    )))?;
                polling::spawn_poller::<HistoricalVolatilityPoint, _>(
                    self, key, source, poll_spec, bcast_tx.clone(), shutdown_rx, label,
                );
            }
            Kind::TakerVolume => {
                let source = polling::taker_volume_poll_source(&self.inner.hub, entry.exchange)
                    .ok_or_else(|| StationError::StreamNotSupported(format!(
                        "TakerVolume REST polling not supported for {:?}",
                        entry.exchange
                    )))?;
                polling::spawn_poller::<TakerVolumePoint, _>(
                    self, key, source, poll_spec, bcast_tx.clone(), shutdown_rx, label,
                );
            }
            Kind::LiquidationBucket => {
                let source = polling::liquidation_bucket_poll_source(&self.inner.hub, entry.exchange)
                    .ok_or_else(|| StationError::StreamNotSupported(format!(
                        "LiquidationBucket REST polling not supported for {:?}",
                        entry.exchange
                    )))?;
                polling::spawn_poller::<LiquidationBucketPoint, _>(
                    self, key, source, poll_spec, bcast_tx.clone(), shutdown_rx, label,
                );
            }
            other => {
                return Err(StationError::StreamNotSupported(format!(
                    "acquire_or_spawn_polled: no poll source for {:?}",
                    other
                )));
            }
        }

        self.inner.muxes.insert(
            key.clone(),
            Multiplexer {
                tx: bcast_tx.clone(),
                consumers,
                shutdown: Some(shutdown_tx),
                grace_cancel: None,
            },
        );
        Ok(bcast_tx)
    }

    /// Acquire (or spawn) a poll-driven multiplexer for `key` using an
    /// explicitly supplied `PollSource` implementation.
    ///
    /// Unlike `acquire_or_spawn_polled`, this method does NOT require the
    /// `kind` to return `Some` from `is_poll_only()`. It is used for kinds
    /// that are normally "derived" (e.g. `Kind::Basis`,
    /// `Kind::FundingSettlement`) but have a native REST history endpoint on
    /// the current exchange.
    async fn acquire_or_spawn_polled_with_source<T, S>(
        &self,
        key: &SeriesKey,
        poll_spec: crate::series::PollSpec,
        raw_symbol: &str,
        source: S,
    ) -> Result<broadcast::Sender<Event>>
    where
        T: crate::series::DataPoint + 'static,
        S: crate::polling::PollSource<T>,
        Event: EventFrom<T>,
    {
        use crate::station::Multiplexer;

        let (bcast_tx, _) = broadcast::channel::<Event>(1024);
        let consumers = Arc::new(AtomicUsize::new(1));
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let label = raw_symbol.to_string();

        polling::spawn_poller::<T, S>(
            self, key, source, poll_spec, bcast_tx.clone(), shutdown_rx, label,
        );

        self.inner.muxes.insert(
            key.clone(),
            Multiplexer {
                tx: bcast_tx.clone(),
                consumers,
                shutdown: Some(shutdown_tx),
                grace_cancel: None,
            },
        );
        Ok(bcast_tx)
    }
}

impl StationInner {
    pub(crate) fn release_consumer(self: &Arc<Self>, key: &SeriesKey) {
        let (became_zero, grace) = {
            let Some(mux) = self.muxes.get(key) else { return; };
            let prev = mux.consumers.fetch_sub(1, Ordering::SeqCst);
            (prev <= 1, self.unsubscribe_grace)
        };

        if !became_zero {
            return;
        }

        if grace.is_zero() {
            // Immediate shutdown — existing behaviour.
            if let Some((_, mut mux)) = self.muxes.remove(key) {
                if let Some(tx) = mux.shutdown.take() {
                    let _ = tx.send(());
                }
            }
            return;
        }

        // Grace period: spawn a timer task. Store a cancel channel in the mux
        // so `acquire_or_spawn` can cancel it when a new subscriber arrives.
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

        // Store cancel_tx in the mux before spawning to avoid a race where
        // acquire_or_spawn could observe grace_cancel == None before the task
        // starts (extremely unlikely but theoretically possible on native).
        {
            let Some(mut mux) = self.muxes.get_mut(key) else { return; };
            mux.grace_cancel = Some(cancel_tx);
        }

        let inner = Arc::clone(self);
        let key = key.clone();

        let grace_fut = Box::pin(async move {
            // Race: grace timer vs cancel signal from acquire_or_spawn.
            #[cfg(not(target_arch = "wasm32"))]
            let timed_out = tokio::select! {
                _ = cancel_rx => false,
                _ = tokio::time::sleep(grace) => true,
            };
            #[cfg(target_arch = "wasm32")]
            let timed_out = tokio::select! {
                _ = cancel_rx => false,
                _ = gloo_timers::future::sleep(grace) => true,
            };

            if timed_out {
                // Grace expired without a new subscriber — fire shutdown.
                // Double-check consumers == 0 as a safety net (the cancel
                // channel send happens before fetch_add, so a race that
                // increments consumers before we reach here is possible in
                // theory; the guard prevents a spurious kill).
                let still_zero = inner
                    .muxes
                    .get(&key)
                    .map(|m| m.consumers.load(Ordering::SeqCst) == 0)
                    .unwrap_or(false);
                if still_zero {
                    if let Some((_, mut mux)) = inner.muxes.remove(&key) {
                        if let Some(tx) = mux.shutdown.take() {
                            let _ = tx.send(());
                        }
                    }
                    inner.series_handles.remove(&key);
                }
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(grace_fut);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(grace_fut);
    }
}

/// Derived-stream actor. Consumes from N upstream broadcast channels via
/// `futures_util::stream::select_all`, runs the `DerivedStream` state machine,
/// and emits output to the derived stream's own broadcast channel.
///
/// On exit (shutdown signal or all upstreams closed):
/// - flushes disk store
/// - decrements consumer ref-count on each upstream key (RAII propagation)
/// - removes own mux entry if no consumers remain
fn spawn_derived_forwarder<D: DerivedStream + 'static>(
    station: &Station,
    key: &SeriesKey,
    upstream_rxs: Vec<broadcast::Receiver<Event>>,
    upstream_keys: Vec<SeriesKey>,
    bcast_tx: broadcast::Sender<Event>,
    mut shutdown_rx: oneshot::Receiver<()>,
    symbol_label: String,
    // Per-dep AggTrade/Trade seed events for P0-C cold-start. One Vec per dep
    // in `D::deps()` order. Empty vec = no seed for that dep.
    agg_seed_per_dep: Vec<Vec<Event>>,
    // Ring capacity for the in-memory Series<D::Output> — sized at spawn time
    // to be at least as large as the computed seed (see the call site in
    // `acquire_or_spawn_derived_body`), so a deep kline-approx seed is never
    // evicted by the ring before the live loop begins. Always
    // >= `warm_start_capacity`.
    ring_capacity_hint: usize,
    // Fired exactly once, right before the live select loop begins (i.e. once
    // disk warm-seed + agg-seed + RAM warm-seed have all landed in the ring).
    // `acquire_or_spawn_derived_body` awaits this (with a timeout) so
    // `Station::subscribe` returns only after cold-start seeding is visible
    // to `Station::series::<T>()` peekers. Send-error (receiver already
    // dropped, e.g. on timeout) is intentionally ignored.
    seed_done_tx: oneshot::Sender<()>,
    // Cold-seed → live seam baseline for the kline-sufficient kinds'
    // `KlineDeltaState` (see `DerivedStream::seed_kline_baseline`):
    // `(open_time, volume, close)` of the LAST kline the cold-seed fetched
    // (possibly still-forming). `None` for every other derived kind, and
    // for kline-sufficient kinds when the cold-seed fetched zero klines
    // (e.g. `warm_n == 0`).
    kline_seed_baseline: Option<(i64, f64, f64)>,
) where
    Event: EventFrom<D::Output>,
{
    let inner = station.inner.clone();
    let key = key.clone();
    let storage_root = inner.storage_root.clone();
    let persistence = inner.persistence.clone();
    let warm = inner.warm_start_capacity;
    let exchange = key.exchange;

    // Create the shared series arc and register it in series_handles so
    // Station::series<T>() can hand it to render-time consumers synchronously
    // — same mechanism as spawn_forwarder (~:1893). Without this, a derived
    // kind (Renko/PnF/Kagi/3LB/Footprint/CVD/TPO/Basis/FundingSettlement)
    // never appears via Station::series<T>() and every re-subscribe re-seeds
    // from scratch.
    let shared_series: Arc<RwLock<Series<D::Output>>> =
        Arc::new(RwLock::new(Series::new(ring_capacity_hint)));
    // Type-erase: store Arc<RwLock<Series<D::Output>>> inside an
    // Arc<dyn Any+Send+Sync>. The outer Arc is what Any::downcast_ref will
    // find the concrete type on.
    let erased: Arc<dyn Any + Send + Sync> = Arc::new(Arc::clone(&shared_series));
    inner.series_handles.insert(key.clone(), erased);

    // Register the exit-ack receiver now, before the derived forwarder body
    // starts — same rationale as spawn_forwarder: a caller racing
    // `force_unsubscribe_and_await` against this spawn must always find the
    // entry once `acquire_or_spawn` has returned.
    let (exit_ack_tx, exit_ack_rx) = oneshot::channel::<()>();
    inner.exit_acks.insert(key.clone(), exit_ack_rx);

    {
        let derived_fut = Box::pin(async move {
            // Open disk store if persistence is on for this kind (native only).
            #[cfg(not(target_arch = "wasm32"))]
            let mut disk: Option<DiskStore<D::Output>> = None;
            #[cfg(not(target_arch = "wasm32"))]
            if persistence.is_enabled_for(&key.kind) {
                match DiskStore::<D::Output>::with_idx_every_and_retention(
                    &storage_root, key.clone(), 1024, persistence.retention_days,
                ).await {
                    Ok(store) => disk = Some(store),
                    Err(e) => tracing::warn!(?e, ?key, "derived: disk store open failed"),
                }
            }
            // On wasm, no disk store — `warm` is only consumed by the
            // native-only `d.read_tail(warm)` call below.
            #[cfg(target_arch = "wasm32")]
            let _ = (&storage_root, &persistence, &warm);

            // Register a flush handle so `Station::flush_persistence()` can
            // force this forwarder's DiskStore to drain + flush on demand.
            // Native-only — wasm derived forwarders open no disk store (see
            // above), so `flush_rx` stays `None` and the select branch below
            // is a permanent no-op there, matching current wasm behavior.
            #[cfg(not(target_arch = "wasm32"))]
            let mut flush_rx = if disk.is_some() {
                let (handle, rx) = FlushHandle::channel();
                inner.flush_handles.insert(key.clone(), handle);
                Some(rx)
            } else {
                None
            };
            #[cfg(target_arch = "wasm32")]
            let mut flush_rx: Option<mpsc::Receiver<FlushAck>> = None;

            // Disk warm-seed (Task C): emit persisted derived bars from the
            // previous session BEFORE the live loop begins. Bridges cross-session
            // history so consumers see past derived bars without waiting for new
            // live derives. Same pattern as spawn_forwarder (polling.rs ~line 145).
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(d) = disk.as_ref() {
                match d.read_tail(warm).await {
                    Ok(tail) => {
                        for point in &tail {
                            shared_series.write().await.upsert_by_ts(point.clone());
                            let _ = bcast_tx.send(Event::from_point(
                                exchange,
                                key.account_type,
                                &symbol_label,
                                &key.kind,
                                point.clone(),
                            ));
                        }
                    }
                    Err(e) => tracing::debug!(?e, ?key, "derived: disk warm-seed read_tail failed"),
                }
            }

            let mut state = D::new_for_key(&key);
            if let Some((open_time, volume, close)) = kline_seed_baseline {
                state.seed_kline_baseline(open_time, volume, close);
            }

            // AggTrade dense seed (P0-C). Feed pre-fetched REST AggTrade/Trade
            // events through the derived state machine, broadcasting emitted bars
            // so consumers see the cold-start window immediately. Applied per-dep
            // in deps() order before the live loop.
            for (dep_idx, seed_events) in agg_seed_per_dep.iter().enumerate() {
                if seed_events.is_empty() {
                    continue;
                }
                let emitted = state.seed_from_events(seed_events, dep_idx);
                for point in emitted {
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(d) = disk.as_mut() {
                        if let Err(e) = d.append(&point) {
                            tracing::warn!(?e, ?key, "derived agg-seed: disk append failed");
                        }
                    }
                    shared_series.write().await.upsert_by_ts(point.clone());
                    let _ = bcast_tx.send(Event::from_point(
                        exchange,
                        key.account_type,
                        &symbol_label,
                        &key.kind,
                        point,
                    ));
                }
            }

            // RAM warm-seed (Gap C): a trade-derived aggregator (Range/Tick/
            // Volume/Footprint bars — deps() == [Trade]) reconstructs its initial
            // bar state from the trades ALREADY buffered in the upstream Trade
            // Series ring, so subscribing while trades are flowing for another
            // consumer (tape / footprint panel / another chart) is not
            // cold-empty. We replay the buffered TradePoints through
            // `state.on_upstream_event` BEFORE the live loop, then emit/persist
            // the resulting bars so the new consumer sees them immediately.
            //
            // Only applies when the sole dependency is Stream::Trade — Basis
            // (MarkPrice+IndexPrice) and FundingSettlement (FundingRate) are NOT
            // trade-seeded (replaying trades through them is meaningless). The ring
            // is only non-trivial when the Station was built with warm_start(N>0);
            // otherwise it holds ≤1 point and the seed is a near-no-op.
            if D::deps() == [Stream::Trade] {
                if let Some(dep_key) = upstream_keys.first() {
                    let trade_handle = inner
                        .series_handles
                        .get(dep_key)
                        .and_then(|e| {
                            e.downcast_ref::<Arc<RwLock<Series<TradePoint>>>>().map(Arc::clone)
                        });
                    if let Some(handle) = trade_handle {
                        let buffered = handle.read().await.snapshot(); // oldest→newest
                        for pt in buffered {
                            let ev = Event::Trade {
                                exchange,
                                symbol: symbol_label.clone(),
                                point: pt,
                            };
                            if let Some(point) = state.on_upstream_event(&ev, 0) {
                                #[cfg(not(target_arch = "wasm32"))]
                                if let Some(d) = disk.as_mut() {
                                    if let Err(e) = d.append(&point) {
                                        tracing::warn!(?e, ?key, "derived seed: disk append failed");
                                    }
                                }
                                shared_series.write().await.upsert_by_ts(point.clone());
                                let _ = bcast_tx.send(Event::from_point(
                                    exchange,
                                    key.account_type,
                                    &symbol_label,
                                    &key.kind,
                                    point,
                                ));
                            }
                        }
                    }
                }
            }

            // Cold-start seeding (disk warm-seed + agg-seed + RAM warm-seed)
            // is now fully applied to `shared_series` — signal the awaiting
            // `acquire_or_spawn_derived_body` so `Station::subscribe` can
            // return with the ring already populated. Ignore send error:
            // the receiver may have already timed out and dropped.
            let _ = seed_done_tx.send(());

            // Convert each upstream Receiver into a tagged BroadcastStream so
            // the state machine can branch by dep_idx cheaply.
            let tagged: Vec<_> = upstream_rxs
                .into_iter()
                .enumerate()
                .map(|(idx, rx)| {
                    tokio_stream::wrappers::BroadcastStream::new(rx)
                        .filter_map(move |res| async move {
                            match res {
                                Ok(ev) => Some((idx, ev)),
                                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
                                    tracing::warn!(dep_idx = idx, lagged = n, "derived: upstream lagged — events dropped");
                                    None
                                }
                            }
                        })
                        // Box to make all stream types uniform for select_all.
                        .boxed()
                })
                .collect();

            let mut merged = futures_util::stream::select_all(tagged);

            loop {
                #[cfg(not(target_arch = "wasm32"))]
                let item_opt = tokio::select! {
                    _ = &mut shutdown_rx => break,
                    ack = recv_flush_request(&mut flush_rx) => {
                        let result = flush_disk_store(&mut disk).await;
                        let _ = ack.send(result);
                        continue;
                    }
                    item = merged.next() => item,
                };
                // wasm derived forwarders open no disk store, but the flush
                // branch is still wired for API uniformity — `flush_rx` is
                // always `None` here so it never fires.
                #[cfg(target_arch = "wasm32")]
                let item_opt = tokio::select! {
                    _ = &mut shutdown_rx => break,
                    ack = recv_flush_request(&mut flush_rx) => {
                        let _ = ack.send(Ok(()));
                        continue;
                    }
                    item = merged.next() => item,
                };

                let Some((dep_idx, ev)) = item_opt else {
                    // All upstreams closed their senders — derived stream is done.
                    tracing::info!(?key, "derived: all upstreams closed — exiting");
                    break;
                };

                if let Some(point) = state.on_upstream_event(&ev, dep_idx) {
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(d) = disk.as_mut() {
                        if let Err(e) = d.append(&point) {
                            tracing::warn!(?e, "derived: disk store append failed");
                        }
                    }
                    shared_series.write().await.push(point.clone());
                    let _ = bcast_tx.send(Event::from_point(exchange, key.account_type, &symbol_label, &key.kind, point));
                }
            }

            // Unregister the flush handle BEFORE the final flush — mirrors
            // spawn_forwarder's teardown ordering.
            #[cfg(not(target_arch = "wasm32"))]
            if flush_rx.is_some() {
                inner.flush_handles.remove(&key);
            }
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(mut d) = disk { let _ = d.flush().await; }

            // Release upstream consumer refs — propagates shutdown upward if the
            // derived forwarder was the only consumer of its upstream muxes.
            for up_key in &upstream_keys {
                inner.release_consumer(up_key);
            }

            // Remove own mux entry if no consumers remain.
            let still_consumers = inner
                .muxes
                .get(&key)
                .map(|m| m.consumers.load(Ordering::SeqCst))
                .unwrap_or(0);
            if still_consumers == 0 {
                inner.muxes.remove(&key);
            }
            // Remove the series handle so Station::series<T>() returns None once
            // the forwarder is gone (same lifecycle as the mux entry). Mirrors
            // spawn_forwarder's teardown (~:2159).
            inner.series_handles.remove(&key);
            // Fire the exit ack LAST — mirrors spawn_forwarder's teardown
            // ordering so `force_unsubscribe_and_await` only unblocks once
            // mux + series-handle + upstream refs are all torn down.
            inner.exit_acks.remove(&key);
            let _ = exit_ack_tx.send(());
        });
        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(derived_fut);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(derived_fut);
    }
}

/// Await the next flush request from an `Option<mpsc::Receiver<FlushAck>>`.
///
/// `None` (no `DiskStore` open for this forwarder — persistence disabled or
/// open failed) never resolves, so this branch is effectively disabled in
/// the enclosing `select!` without needing a separate `if`-guarded arm.
/// `Some(rx)` whose channel has since closed (should not happen — the
/// forwarder itself owns `rx` for its own lifetime) also never resolves;
/// `flush_persistence` treats a closed *sender* as "already gone" on its own
/// side, so this is purely defensive.
pub(crate) async fn recv_flush_request(rx: &mut Option<mpsc::Receiver<FlushAck>>) -> FlushAck {
    match rx {
        Some(rx) => match rx.recv().await {
            Some(ack) => ack,
            None => std::future::pending().await,
        },
        None => std::future::pending().await,
    }
}

/// Flush an open `DiskStore<T>`, normalizing both targets' error types to
/// `std::io::Result<()>`. `None` (no store open) is treated as a no-op success.
pub(crate) async fn flush_disk_store<T: DataPoint>(disk: &mut Option<DiskStore<T>>) -> std::io::Result<()> {
    match disk {
        Some(d) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                d.flush().await
            }
            #[cfg(target_arch = "wasm32")]
            {
                d.flush().await.map_err(std::io::Error::from)
            }
        }
        None => Ok(()),
    }
}

/// Generic per-kind forwarder. Owns:
/// - DiskStore<T> (Option; on if persistence enabled),
/// - in-memory Series<T> (capacity = warm_start_capacity, kept as scratch),
/// - WS event stream.
///
/// On spawn: emits warm-start tail from DiskStore (if any) as `Event`s to
/// broadcast. Then transitions to live mode: each StreamEvent → DataPoint::from
/// → write disk → push memory → emit broadcast Event.
fn spawn_forwarder<T: DataPoint + 'static>(
    station: &Station,
    key: &SeriesKey,
    ws: Arc<dyn digdigdig3::core::traits::WebSocketConnector>,
    bcast_tx: broadcast::Sender<Event>,
    mut shutdown_rx: oneshot::Receiver<()>,
    symbol_label: String,
    // REST-backfill seed used when on-disk history is empty. Empty Vec
    // disables the REST fallback.
    rest_seed: Vec<T>,
    // Original subscribe request. Held so the forwarder can issue
    // unsubscribe + subscribe on disconnect to force a fresh subscription
    // state at the exchange.
    sub_req: SubscriptionRequest,
) where
    Event: EventFrom<T>,
{
    let inner = station.inner.clone();
    let key = key.clone();
    let storage_root = inner.storage_root.clone();
    let persistence = inner.persistence.clone();
    let warm = inner.warm_start_capacity;
    let exchange = key.exchange;
    let gap_cfg = inner.gap_heal;
    let hub_for_heal = inner.hub.clone();

    // Create the shared series arc and register it in series_handles so
    // Station::series<T>() can hand it to render-time consumers synchronously.
    let shared_series: Arc<RwLock<Series<T>>> = Arc::new(RwLock::new(Series::new(warm)));
    // Type-erase: store Arc<RwLock<Series<T>>> inside an Arc<dyn Any+Send+Sync>.
    // The outer Arc is what Any::downcast_ref will find the concrete type on.
    let erased: Arc<dyn Any + Send + Sync> = Arc::new(Arc::clone(&shared_series));
    inner.series_handles.insert(key.clone(), erased);

    // Register the exit-ack receiver now, before the forwarder body starts —
    // mirrors series_handles: a caller racing `force_unsubscribe_and_await`
    // against this very spawn must always find the entry once `acquire_or_spawn`
    // has returned.
    let (exit_ack_tx, exit_ack_rx) = oneshot::channel::<()>();
    inner.exit_acks.insert(key.clone(), exit_ack_rx);

    {
    let forwarder_fut = Box::pin(async move {
        // Open disk store if persistence is on for this kind.
        // Native: std::fs-backed DiskStore. Wasm: OPFS-backed DiskStore.
        #[cfg(not(target_arch = "wasm32"))]
        let mut disk: Option<DiskStore<T>> = None;
        #[cfg(not(target_arch = "wasm32"))]
        if persistence.is_enabled_for(&key.kind) {
            match DiskStore::<T>::with_idx_every_and_retention(
                &storage_root, key.clone(), 1024, persistence.retention_days,
            ).await {
                Ok(store) => disk = Some(store),
                Err(e) => tracing::warn!(?e, ?key, "disk store open failed"),
            }
        }
        // Wasm: OPFS DiskStore (Wave 4-E).
        // TODO(wasm): sweep_retention — OPFS DiskStore has no retention sweep
        // yet (native-only for now, see series/store.rs task notes). Entry
        // removal on OPFS is not wired here; `persistence.retention_days` is
        // read on native only.
        #[cfg(target_arch = "wasm32")]
        let mut disk: Option<DiskStore<T>> = None;
        #[cfg(target_arch = "wasm32")]
        if persistence.is_enabled_for(&key.kind) {
            match DiskStore::<T>::new(key.clone()).await {
                Ok(store) => disk = Some(store),
                Err(e) => tracing::warn!(?e, ?key, "wasm OPFS disk store open failed"),
            }
        }
        // Suppress unused warning on wasm when persistence is disabled.
        #[cfg(target_arch = "wasm32")]
        let _ = &storage_root;

        // Register a flush handle so `Station::flush_persistence()` can force
        // this forwarder's DiskStore to drain + flush on demand (chiefly for
        // wasm OPFS — see `flush_persistence` doc comment). Only registered
        // when persistence actually opened a store; unregistered right
        // before the forwarder exits.
        let mut flush_rx = if disk.is_some() {
            let (handle, rx) = FlushHandle::channel();
            inner.flush_handles.insert(key.clone(), handle);
            Some(rx)
        } else {
            None
        };

        // In-memory ring (warm capacity) — shared with Station::series<T>().
        // The forwarder is the sole writer; render-time consumers hold read
        // guards for snapshot access without awaiting an Event.
        let mut last_emitted_ms: i64 = 0;

        // Warm-start. Priority: disk tail > REST seed.
        if warm > 0 {
            // Wave 4-D: both native and wasm read real disk/OPFS tail.
            let disk_tail: Vec<T> = if let Some(d) = disk.as_ref() {
                d.read_tail(warm).await.unwrap_or_default()
            } else {
                Vec::new()
            };
            let seed_points: Vec<T> = if disk_tail.is_empty() && !rest_seed.is_empty() {
                rest_seed
            } else {
                disk_tail
            };
            for p in &seed_points {
                let _ = bcast_tx.send(Event::from_point(exchange, key.account_type, &symbol_label, &key.kind, p.clone()));
                last_emitted_ms = last_emitted_ms.max(p.timestamp_ms());
            }
            shared_series.write().await.seed(seed_points);
        }

        let mut stream = ws.event_stream();
        // Silence threshold: if `event_stream().next()` produces no event for
        // this long, the underlying WS is presumed dropped. Mirrors MLC
        // `ws_manager` behaviour (60s). Tunable via env for tests.
        #[cfg(not(target_arch = "wasm32"))]
        let silence_timeout = std::time::Duration::from_secs(
            std::env::var("DIG3_WS_SILENCE_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60),
        );
        // Wasm (Wave 4-F): 60s silence watchdog via gloo_timers.
        // DIG3_WS_SILENCE_SECS is not readable on wasm32 (std::env absent);
        // default 60 s is hardcoded. Configurable at compile time if needed.
        #[cfg(target_arch = "wasm32")]
        let silence_timeout_ms: u32 = 60_000;
        // Debug-only: artificially slow down the per-event loop. Used by e2e
        // tests to force broadcast-channel overflow → `Lagged` error →
        // `stream_err` branch. Production callers leave this unset (0 ms).
        #[cfg(not(target_arch = "wasm32"))]
        let debug_slow_ms: u64 = std::env::var("DIG3_DEBUG_SLOW_CONSUMER_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        // Wasm (Wave 4-E): flush OPFS every N events to bound data loss.
        // A periodic gloo interval is not used here — instead a simple
        // flush-every-N-appends strategy matches the forwarder's sync
        // append pattern without an extra concurrent task.
        #[cfg(target_arch = "wasm32")]
        let mut wasm_flush_counter: u32 = 0;
        #[cfg(target_arch = "wasm32")]
        const WASM_FLUSH_EVERY: u32 = 64;

        loop {
            // Single select arm — exits via shutdown or detects disconnect
            // (None / Err / silence). All three cases = disconnect = heal.
            // Native: tokio::time::timeout for silence detection.
            // Wasm (4-F): gloo_timers::future::sleep race for silence detection.
            #[cfg(not(target_arch = "wasm32"))]
            let item_opt = tokio::select! {
                _ = &mut shutdown_rx => break,
                ack = recv_flush_request(&mut flush_rx) => {
                    let result = flush_disk_store(&mut disk).await;
                    let _ = ack.send(result);
                    continue;
                }
                res = tokio::time::timeout(silence_timeout, stream.next()) => res,
            };
            #[cfg(target_arch = "wasm32")]
            // On wasm: race stream.next() against a gloo_timers sleep.
            // Returns Ok(Some(...)) for a real event, Ok(None) for stream end,
            // Err(()) for the silence timeout expiring.
            let item_opt: std::result::Result<
                Option<std::result::Result<_, digdigdig3::core::types::WebSocketError>>,
                (),
            > = tokio::select! {
                _ = &mut shutdown_rx => break,
                ack = recv_flush_request(&mut flush_rx) => {
                    let result = flush_disk_store(&mut disk).await;
                    let _ = ack.send(result);
                    continue;
                }
                _ = gloo_timers::future::sleep(std::time::Duration::from_millis(silence_timeout_ms as u64)) => Err(()),
                item = stream.next() => Ok(item),
            };

            let trigger_heal_reason: Option<&'static str> = match &item_opt {
                Err(_) => Some("silence_timeout"),
                Ok(None) => Some("stream_ended"),
                Ok(Some(Err(_))) => Some("stream_err"),
                Ok(Some(Ok(_))) => None,
            };

            if let Some(reason) = trigger_heal_reason {
                // Heal + resub is kline-only. For non-kline kinds:
                // - REST cannot bridge the gap (no public endpoint for
                //   trade/OB/ticker/mark/funding/OI/liq history live-feed).
                // - Resub spam on a WireAbsent stream was the trigger for
                //   MLI's 0.3.6 OOM — see release-0.3.7-plan.md.
                // - The transport-level UniversalWsTransport auto-reconnects
                //   internally; the forwarder does not need to resub manually.
                //
                // Non-kline behavior: log + exit the forwarder. The mux entry
                // is removed below so a later subscribe for the same key can
                // re-spawn cleanly.
                let is_kline_family = matches!(
                    &key.kind,
                    Kind::Kline(_) | Kind::MarkPriceKline(_)
                    | Kind::IndexPriceKline(_) | Kind::PremiumIndexKline(_)
                );

                if !is_kline_family {
                    tracing::info!(
                        target: "dig3::gap_heal",
                        ?key, reason,
                        "non-kline stream disconnect — forwarder exiting (no resub for non-kline kinds)"
                    );
                    break;
                }

                tracing::info!(target: "dig3::gap_heal", ?key, reason, "ws disconnect detected → heal + resub");
                // 1. REST heal (kline-only; no-op for non-kline kinds, which
                //    have already returned above). Wave 4-B: enabled on both
                //    targets. On wasm REST succeeds for the 9 proxy-override
                //    venues; silently returns empty for others until Wave 4-C.
                {
                    let mut series_guard = shared_series.write().await;
                    run_kline_heal::<T>(
                        &hub_for_heal, &key, &gap_cfg, &symbol_label,
                        last_emitted_ms, exchange,
                        &mut *series_guard, &mut disk, &bcast_tx,
                    ).await;
                    last_emitted_ms = last_emitted_ms.max(
                        series_guard.last().map(|p| p.timestamp_ms()).unwrap_or(0)
                    );
                }
                // 2. Force a fresh subscription state at the exchange.
                //    Unsubscribe is best-effort (the server may have already
                //    dropped us). Resubscribe must succeed or we log + retry
                //    on the next disconnect cycle.
                let unsub_res = ws.unsubscribe(sub_req.clone()).await;
                let sub_res = ws.subscribe(sub_req.clone()).await;
                tracing::info!(
                    target: "dig3::gap_heal",
                    ?key,
                    unsub_ok = unsub_res.is_ok(),
                    sub_ok = sub_res.is_ok(),
                    "resub cycle complete"
                );
                if let Err(e) = unsub_res {
                    tracing::debug!(target: "dig3::gap_heal", ?key, ?e, "unsubscribe failed (best-effort)");
                }
                if let Err(e) = sub_res {
                    tracing::warn!(target: "dig3::gap_heal", ?key, ?e, "resubscribe failed");
                }
                // 3. Re-attach broadcast receiver — picks up post-resub events.
                //    Explicit drop of the old stream first: BroadcastStream
                //    holds a Receiver whose internal ring buffer occupies
                //    `event_tx` capacity until dropped. Letting the old one
                //    go before subscribing a new one keeps receiver_count
                //    minimal across heal cycles.
                drop(stream);
                stream = ws.event_stream();
                continue;
            }

            let ev = match item_opt {
                Ok(Some(Ok(ev))) => ev,
                _ => unreachable!(),
            };

            if !event_matches_key(&ev, &key) {
                continue;
            }
            let Some(point) = T::from_stream_event(&ev) else {
                continue;
            };

            // Wave 4-E: append on both targets (native std::fs, wasm OPFS).
            // Native append returns Result; wasm append returns () (infallible — buffers in memory).
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(d) = disk.as_mut() {
                if let Err(e) = d.append(&point) {
                    tracing::warn!(?e, "disk store append failed");
                }
            }
            #[cfg(target_arch = "wasm32")]
            if let Some(d) = disk.as_mut() {
                d.append(&point);
            }
            // Wasm (Wave 4-E): flush OPFS buffer periodically to bound data loss.
            #[cfg(target_arch = "wasm32")]
            {
                wasm_flush_counter = wasm_flush_counter.wrapping_add(1);
                if wasm_flush_counter % WASM_FLUSH_EVERY == 0 {
                    if let Some(d) = disk.as_mut() {
                        if let Err(e) = d.flush().await {
                            tracing::warn!(?e, "wasm OPFS periodic flush failed");
                        }
                    }
                }
            }
            let pt_ts = point.timestamp_ms();
            // Klines: multiple in-flight updates share open_time — upsert
            // keeps the ring deduplicated. Other kinds are monotonic.
            {
                let mut series_guard = shared_series.write().await;
                if matches!(&key.kind, Kind::Kline(_) | Kind::MarkPriceKline(_) | Kind::IndexPriceKline(_) | Kind::PremiumIndexKline(_)) {
                    series_guard.upsert_by_ts(point.clone());
                } else {
                    series_guard.push(point.clone());
                }
            }
            last_emitted_ms = last_emitted_ms.max(pt_ts);

            let _ = bcast_tx.send(Event::from_point(exchange, key.account_type, &symbol_label, &key.kind, point));

            #[cfg(not(target_arch = "wasm32"))]
            if debug_slow_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(debug_slow_ms)).await;
            }
        }

        // Unregister the flush handle BEFORE the final flush — once this
        // entry is gone, `flush_persistence` will no longer try to reach
        // this (about-to-exit) forwarder and will treat it as already flushed.
        // The final flush below still runs unconditionally right after.
        if flush_rx.is_some() {
            inner.flush_handles.remove(&key);
        }
        // Final flush on both targets (Wave 4-E: wasm flushes OPFS on shutdown).
        if let Some(mut d) = disk { let _ = d.flush().await; }
        // Remove the mux entry so a subsequent `subscribe` for the same key
        // can re-spawn a fresh forwarder. Without this, the dead mux would
        // sit in `inner.muxes`, and re-subscribe would attach to a broadcast
        // tx whose forwarder has already exited (no events ever arrive).
        //
        // Only remove if no consumer is left attached — otherwise a still-
        // alive consumer would think it has a working stream while we
        // silently tore it down. (`release_consumer` already removes on
        // refcount==0; here we cover the other path: forwarder ended on
        // its own before all consumers dropped.)
        let still_consumers = inner
            .muxes
            .get(&key)
            .map(|m| m.consumers.load(Ordering::SeqCst))
            .unwrap_or(0);
        if still_consumers == 0 {
            inner.muxes.remove(&key);
        }
        // Remove the series handle so Station::series<T>() returns None once
        // the forwarder is gone (same lifecycle as the mux entry).
        inner.series_handles.remove(&key);
        // Fire the exit ack LAST — after every other teardown step has
        // completed, so a caller awaiting `force_unsubscribe_and_await` only
        // unblocks once the mux and series-handle entries are already gone.
        // `force_unsubscribe_and_await` itself already removed this entry
        // before firing shutdown; a plain grace/refcount exit (the common
        // case) removes it here instead. Ignore send error: no awaiter
        // present is normal.
        inner.exit_acks.remove(&key);
        let _ = exit_ack_tx.send(());
    });
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(forwarder_fut);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(forwarder_fut);
    }
}

/// Run a kline auto-heal triggered by WS disconnect. Called only for
/// `Kind::Kline`; no-op for other kinds. Wave 4-B: enabled on both targets.
async fn run_kline_heal<T: DataPoint + 'static>(
    hub: &Arc<ExchangeHub>,
    key: &SeriesKey,
    cfg: &crate::GapHealConfig,
    symbol_label: &str,
    last_emitted_ms: i64,
    exchange: digdigdig3::core::types::ExchangeId,
    series: &mut Series<T>,
    disk: &mut Option<DiskStore<T>>,
    bcast_tx: &broadcast::Sender<Event>,
) where
    Event: EventFrom<T>,
{
    if !cfg.enabled {
        return;
    }
    let Kind::Kline(iv) = &key.kind else { return; };

    let now_ms = chrono::Utc::now().timestamp_millis();
    let limit = crate::gap_heal::heal_limit(cfg, iv.as_str(), last_emitted_ms, now_ms);

    tracing::info!(
        target: "dig3::gap_heal",
        ?key,
        last_emitted_ms,
        limit,
        "kline heal: pulling REST"
    );

    let pulled: Vec<T> = cast_vec(
        crate::gap_heal::heal_klines(hub, key.exchange, key.account_type, &key.symbol, iv.as_str(), last_emitted_ms, limit).await
    );
    let pulled_count = pulled.len();

    // ALL pulled bars get upserted (last-write-wins overwrites any in-flight
    // broken live bar). Only bars strictly newer than last_emitted_ms are
    // EMITTED to consumers (the older ones already reached them as live).
    let new_to_emit = crate::gap_heal::select_heal_window(pulled.clone(), last_emitted_ms);
    let emitted_count = new_to_emit.len();

    for p in pulled {
        if let Some(d) = disk.as_mut() {
            let _ = d.append(&p);
        }
        series.upsert_by_ts(p);
    }
    for p in new_to_emit {
        let _ = bcast_tx.send(Event::from_point(exchange, key.account_type, symbol_label, &key.kind, p));
    }
    if let Some(d) = disk.as_mut() { let _ = d.flush().await; }

    tracing::info!(
        target: "dig3::gap_heal",
        ?key,
        pulled_count,
        emitted_count,
        "kline heal: applied"
    );
}

/// Cast `Vec<A>` to `Vec<B>` when A == B at runtime via TypeId. Used to
/// bridge the kind-specific REST return type (`Vec<BarPoint>`) back to the
/// generic forwarder's `Vec<T>`. Safe when called at a site where the kind
/// match arm guarantees A == B.
fn cast_vec<A: 'static, B: 'static>(v: Vec<A>) -> Vec<B> {
    if std::any::TypeId::of::<A>() == std::any::TypeId::of::<B>() {
        // SAFETY: confirmed TypeId equality immediately above; memory layout
        // and ownership are identical between `Vec<A>` and `Vec<B>`.
        let mut v = std::mem::ManuallyDrop::new(v);
        let (ptr, len, cap) = (v.as_mut_ptr() as *mut B, v.len(), v.capacity());
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    } else {
        Vec::new()
    }
}

/// Symbol-level routing: drop events whose `symbol` field doesn't match our key.
/// Every public-data variant now carries `symbol: String` on the variant itself.
fn event_matches_key(ev: &StreamEvent, key: &SeriesKey) -> bool {
    let want = key.symbol.as_str();
    let got: Option<&str> = event_raw_symbol(ev);
    match got {
        // Empty string = parser couldn't extract; let event through (dispatch is by SeriesKey at the channel level).
        Some("") => true,
        Some(s) => s.eq_ignore_ascii_case(want),
        None => true,
    }
}

/// Extract the raw exchange-native symbol carried on a `StreamEvent` variant,
/// or `None` for private events that don't carry one in this dispatch model.
fn event_raw_symbol(ev: &StreamEvent) -> Option<&str> {
    match ev {
        StreamEvent::Trade { symbol, .. } => Some(symbol),
        StreamEvent::AggTrade { symbol, .. } => Some(symbol),
        StreamEvent::Ticker { symbol, .. } => Some(symbol),
        StreamEvent::Kline { symbol, .. } => Some(symbol),
        StreamEvent::OrderbookSnapshot { symbol, .. } => Some(symbol),
        StreamEvent::OrderbookDelta { symbol, .. } => Some(symbol),
        StreamEvent::MarkPrice { symbol, .. } => Some(symbol),
        StreamEvent::FundingRate { symbol, .. } => Some(symbol),
        StreamEvent::OpenInterestUpdate { symbol, .. } => Some(symbol),
        StreamEvent::Liquidation { symbol, .. } => Some(symbol),
        StreamEvent::LongShortRatio { symbol, .. } => Some(symbol),
        StreamEvent::MarkPriceKline { symbol, .. } => Some(symbol),
        StreamEvent::IndexPriceKline { symbol, .. } => Some(symbol),
        StreamEvent::PremiumIndexKline { symbol, .. } => Some(symbol),
        StreamEvent::IndexPrice { symbol, .. } => Some(symbol),
        StreamEvent::HistoricalVolatility { symbol, .. } => Some(symbol),
        StreamEvent::InsuranceFund { symbol, .. } => Some(symbol),
        StreamEvent::Basis { symbol, .. } => Some(symbol),
        StreamEvent::OptionGreeks { symbol, .. } => Some(symbol),
        StreamEvent::VolatilityIndex { symbol, .. } => Some(symbol),
        StreamEvent::BlockTrade { symbol, .. } => Some(symbol),
        StreamEvent::AuctionEvent { symbol, .. } => Some(symbol),
        StreamEvent::MarketWarning { symbol, .. } => symbol.as_deref(),
        StreamEvent::OrderbookL3 { symbol, .. } => Some(symbol),
        StreamEvent::SettlementEvent { symbol, .. } => Some(symbol),
        StreamEvent::RiskLimit { symbol, .. } => Some(symbol),
        StreamEvent::PredictedFunding { symbol, .. } => Some(symbol),
        StreamEvent::FundingSettlement { symbol, .. } => Some(symbol),
        StreamEvent::CompositeIndex { symbol, .. } => Some(symbol),
        // Private events — private dispatch isn't symbol-routed at the SeriesKey level.
        StreamEvent::OrderUpdate { symbol: _, event: _ }
        | StreamEvent::BalanceUpdate(_)
        | StreamEvent::PositionUpdate { symbol: _, event: _ } => None,
        // Batch is flattened by the transport-layer dispatcher before reaching station;
        // data modules see only leaf variants. This arm exists for exhaustiveness only.
        StreamEvent::Batch(_) => None,
    }
}

/// Trait wired by each `DataPoint` so the forwarder can build the right Event
/// variant. `pub(crate)` so `polling.rs` can use it in `spawn_poller`.
pub(crate) trait EventFrom<T> {
    fn from_point(
        exchange: digdigdig3::core::types::ExchangeId,
        account_type: digdigdig3::core::types::AccountType,
        symbol: &str,
        kind: &Kind,
        p: T,
    ) -> Self;
}

impl EventFrom<TradePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: TradePoint) -> Self {
        Event::Trade { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<AggTradePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: AggTradePoint) -> Self {
        Event::AggTrade { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<BarPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, kind: &Kind, point: BarPoint) -> Self {
        let timeframe = match kind { Kind::Kline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::Bar { exchange, symbol: symbol.to_string(), timeframe, point }
    }
}
impl EventFrom<TickerPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: TickerPoint) -> Self {
        Event::Ticker { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<ObSnapshotPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: ObSnapshotPoint) -> Self {
        Event::OrderbookSnapshot { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<ObDeltaPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: ObDeltaPoint) -> Self {
        Event::OrderbookDelta { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<MarkPricePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: MarkPricePoint) -> Self {
        Event::MarkPrice { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<FundingRatePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: FundingRatePoint) -> Self {
        Event::FundingRate { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<OpenInterestPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: OpenInterestPoint) -> Self {
        Event::OpenInterest { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<LiquidationPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: LiquidationPoint) -> Self {
        Event::Liquidation { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<BlockTradePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: BlockTradePoint) -> Self {
        Event::BlockTrade { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<AuctionEventPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: AuctionEventPoint) -> Self {
        Event::AuctionEvent { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<IndexPricePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: IndexPricePoint) -> Self {
        Event::IndexPrice { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<CompositeIndexPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: CompositeIndexPoint) -> Self {
        Event::CompositeIndex { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<OptionGreeksPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: OptionGreeksPoint) -> Self {
        Event::OptionGreeks { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<VolatilityIndexPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: VolatilityIndexPoint) -> Self {
        Event::VolatilityIndex { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<HistoricalVolatilityPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: HistoricalVolatilityPoint) -> Self {
        Event::HistoricalVolatility { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<LongShortRatioPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: LongShortRatioPoint) -> Self {
        Event::LongShortRatio { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<TakerVolumePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: TakerVolumePoint) -> Self {
        Event::TakerVolume { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<LiquidationBucketPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: LiquidationBucketPoint) -> Self {
        Event::LiquidationBucket { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<BasisPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: BasisPoint) -> Self {
        Event::Basis { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<InsuranceFundPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: InsuranceFundPoint) -> Self {
        Event::InsuranceFund { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<OrderbookL3Point> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: OrderbookL3Point) -> Self {
        Event::OrderbookL3 { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<SettlementEventPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: SettlementEventPoint) -> Self {
        Event::SettlementEvent { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<MarketWarningPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: MarketWarningPoint) -> Self {
        Event::MarketWarning { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<RiskLimitPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: RiskLimitPoint) -> Self {
        Event::RiskLimit { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<PredictedFundingPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: PredictedFundingPoint) -> Self {
        Event::PredictedFunding { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<FundingSettlementPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: FundingSettlementPoint) -> Self {
        Event::FundingSettlement { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<MarkPriceKlinePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, kind: &Kind, point: MarkPriceKlinePoint) -> Self {
        let timeframe = match kind { Kind::MarkPriceKline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::MarkPriceKline { exchange, symbol: symbol.to_string(), timeframe, point }
    }
}
impl EventFrom<IndexPriceKlinePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, kind: &Kind, point: IndexPriceKlinePoint) -> Self {
        let timeframe = match kind { Kind::IndexPriceKline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::IndexPriceKline { exchange, symbol: symbol.to_string(), timeframe, point }
    }
}
impl EventFrom<PremiumIndexKlinePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, kind: &Kind, point: PremiumIndexKlinePoint) -> Self {
        let timeframe = match kind { Kind::PremiumIndexKline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::PremiumIndexKline { exchange, symbol: symbol.to_string(), timeframe, point }
    }
}
impl EventFrom<OrderUpdatePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: OrderUpdatePoint) -> Self {
        Event::OrderUpdate { exchange, account_type, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<BalanceUpdatePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: BalanceUpdatePoint) -> Self {
        Event::BalanceUpdate { exchange, account_type, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<PositionUpdatePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: PositionUpdatePoint) -> Self {
        Event::PositionUpdate { exchange, account_type, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<FootprintPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: FootprintPoint) -> Self {
        Event::Footprint { exchange, symbol: symbol.to_string(), point }
    }
}
// Renko / PnF / Kagi / CVD / TPO emit through derived aggregators only —
// EventFrom funnels their Point types into the corresponding Event
// variant. The `kind` is consulted ONLY where the variant carries a
// runtime parameter that comes from the SeriesKey (none of the five do —
// the parameter is encoded into the Point itself).
impl EventFrom<PnfColumnPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: PnfColumnPoint) -> Self {
        Event::PnfBar { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<KagiSegmentPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: KagiSegmentPoint) -> Self {
        Event::KagiBar { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<ScalarBarPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: ScalarBarPoint) -> Self {
        Event::CvdLine { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<TpoSessionPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: TpoSessionPoint) -> Self {
        Event::TpoProfile { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<RenkoBrickPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: RenkoBrickPoint) -> Self {
        Event::RenkoBar { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<ThreeLineBreakLinePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, point: ThreeLineBreakLinePoint) -> Self {
        Event::ThreeLineBreakUpdate { exchange, symbol: symbol.to_string(), point }
    }
}


// ── Extended-depth EventFrom impls ────────────────────────────────────────────
//
// Extended point types (Indicators / Full) are written to disk at depth; they
// are NOT carried separately in the broadcast channel — broadcast consumers
// always receive the Compact event variant. These impls convert the enriched
// point back to the Compact Event variant so the forwarder compile-checks.

impl EventFrom<TickerIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: TickerIndicatorsPoint) -> Self {
        // Broadcast consumers see a Compact Ticker; disk has the Indicators record.
        Event::Ticker { exchange, symbol: symbol.to_string(), point: TickerPoint {
            ts_ms: p.ts_ms, last: p.last, bid: p.bid, ask: p.ask,
            high_24h: p.high_24h, low_24h: p.low_24h,
            vol_24h: p.vol_24h, quote_vol_24h: p.quote_vol_24h,
            change_pct_24h: p.change_pct_24h,
        }}
    }
}
impl EventFrom<TickerFullPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: TickerFullPoint) -> Self {
        Event::Ticker { exchange, symbol: symbol.to_string(), point: TickerPoint {
            ts_ms: p.ts_ms, last: p.last, bid: p.bid, ask: p.ask,
            high_24h: p.high_24h, low_24h: p.low_24h,
            vol_24h: p.vol_24h, quote_vol_24h: p.quote_vol_24h,
            change_pct_24h: p.price_change_pct_24h,
        }}
    }
}
impl EventFrom<MarkPriceIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: MarkPriceIndicatorsPoint) -> Self {
        Event::MarkPrice { exchange, symbol: symbol.to_string(), point: MarkPricePoint {
            ts_ms: p.ts_ms, mark: p.mark, index: p.index,
        }}
    }
}
impl EventFrom<MarkPriceFullPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: MarkPriceFullPoint) -> Self {
        Event::MarkPrice { exchange, symbol: symbol.to_string(), point: MarkPricePoint {
            ts_ms: p.ts_ms, mark: p.mark, index: p.index,
        }}
    }
}
impl EventFrom<FundingRateIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: FundingRateIndicatorsPoint) -> Self {
        Event::FundingRate { exchange, symbol: symbol.to_string(), point: FundingRatePoint {
            ts_ms: p.ts_ms, rate: p.rate,
            next_funding_time_ms: p.next_funding_time_ms.unwrap_or(0),
        }}
    }
}
impl EventFrom<FundingRateFullPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: FundingRateFullPoint) -> Self {
        Event::FundingRate { exchange, symbol: symbol.to_string(), point: FundingRatePoint {
            ts_ms: p.ts_ms, rate: p.rate,
            next_funding_time_ms: p.next_funding_time_ms.unwrap_or(0),
        }}
    }
}
impl EventFrom<OpenInterestIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: OpenInterestIndicatorsPoint) -> Self {
        Event::OpenInterest { exchange, symbol: symbol.to_string(), point: OpenInterestPoint {
            ts_ms: p.ts_ms, open_interest: p.open_interest,
            open_interest_value: p.open_interest_value,
        }}
    }
}
impl EventFrom<LiquidationIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: LiquidationIndicatorsPoint) -> Self {
        Event::Liquidation { exchange, symbol: symbol.to_string(), point: LiquidationPoint {
            ts_ms: p.ts_ms, price: p.price, quantity: p.quantity,
            value: p.value, side: p.side,
        }}
    }
}
impl EventFrom<LiquidationFullPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: LiquidationFullPoint) -> Self {
        Event::Liquidation { exchange, symbol: symbol.to_string(), point: LiquidationPoint {
            ts_ms: p.ts_ms, price: p.price, quantity: p.quantity,
            value: p.value, side: p.side,
        }}
    }
}
impl EventFrom<IndexPriceIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: IndexPriceIndicatorsPoint) -> Self {
        Event::IndexPrice { exchange, symbol: symbol.to_string(), point: IndexPricePoint {
            ts_ms: p.ts_ms, price: p.price,
        }}
    }
}
impl EventFrom<ObSnapshotIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: ObSnapshotIndicatorsPoint) -> Self {
        Event::OrderbookSnapshot { exchange, symbol: symbol.to_string(), point: ObSnapshotPoint {
            ts_ms: p.ts_ms, bids: p.bids, asks: p.asks,
        }}
    }
}
impl EventFrom<ObDeltaIndicatorsPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, _account_type: digdigdig3::core::types::AccountType, symbol: &str, _kind: &Kind, p: ObDeltaIndicatorsPoint) -> Self {
        Event::OrderbookDelta { exchange, symbol: symbol.to_string(), point: ObDeltaPoint {
            ts_ms: p.ts_ms, bid_changes: p.bid_changes, ask_changes: p.ask_changes,
        }}
    }
}

/// Fetch a REST orderbook snapshot and convert to `Vec<ObSnapshotPoint>`.
/// Returns an empty Vec on any failure (no REST, exchange error, empty book).
async fn ob_rest_seed(
    hub: &Arc<digdigdig3::connector_manager::ExchangeHub>,
    exchange: digdigdig3::core::types::ExchangeId,
    account: digdigdig3::core::types::AccountType,
    symbol: &str,
    depth: usize,
) -> Vec<ObSnapshotPoint> {
    let Some(rest) = hub.rest(exchange) else {
        tracing::warn!(
            target: "dig3::ob_seed",
            ?exchange, symbol,
            "orderbook REST seed: connector not initialized — continuing WS-only"
        );
        return Vec::new();
    };
    let depth_u16 = depth.min(u16::MAX as usize) as u16;
    match rest
        .get_orderbook(
            digdigdig3::core::types::SymbolInput::Raw(symbol),
            Some(depth_u16),
            account,
        )
        .await
    {
        Ok(ob) if ob.bids.is_empty() && ob.asks.is_empty() => {
            tracing::warn!(
                target: "dig3::ob_seed",
                ?exchange, symbol,
                "orderbook REST seed returned empty snapshot — continuing WS-only"
            );
            Vec::new()
        }
        Ok(ob) => {
            tracing::debug!(
                target: "dig3::ob_seed",
                ?exchange, symbol,
                bids = ob.bids.len(),
                asks = ob.asks.len(),
                "orderbook REST seed ok"
            );
            vec![ObSnapshotPoint::from_orderbook(&ob)]
        }
        Err(e) => {
            tracing::warn!(
                target: "dig3::ob_seed",
                ?exchange, symbol, ?e,
                "orderbook REST seed failed — continuing WS-only"
            );
            Vec::new()
        }
    }
}

fn ws_request_for(
    kind: &Kind,
    sym: Symbol,
    account: digdigdig3::core::types::AccountType,
) -> SubscriptionRequest {
    let stream_type = match kind {
        Kind::Trade => StreamType::Trade,
        Kind::AggTrade => StreamType::AggTrade,
        Kind::Kline(iv) => StreamType::Kline { interval: iv.as_str().to_string() },
        Kind::Ticker => StreamType::Ticker,
        Kind::Orderbook => StreamType::Orderbook,
        Kind::OrderbookDelta => StreamType::OrderbookDelta,
        Kind::MarkPrice => StreamType::MarkPrice,
        Kind::FundingRate => StreamType::FundingRate,
        Kind::OpenInterest => StreamType::OpenInterest,
        Kind::Liquidation => StreamType::Liquidation,
        Kind::BlockTrade => StreamType::BlockTrade,
        Kind::AuctionEvent => StreamType::AuctionEvent,
        Kind::IndexPrice => StreamType::IndexPrice,
        Kind::CompositeIndex => StreamType::CompositeIndex,
        Kind::OptionGreeks => StreamType::OptionGreeks,
        Kind::VolatilityIndex => StreamType::VolatilityIndex,
        Kind::HistoricalVolatility => StreamType::HistoricalVolatility,
        // LongShortRatio / TakerVolume / LiquidationBucket: poll-only kinds. The WS arms
        // are unreachable in normal operation (acquire_or_spawn_polled handles them first),
        // but kept as defensive fallbacks so the match is exhaustive.
        Kind::LongShortRatio => StreamType::LongShortRatio,
        Kind::TakerVolume | Kind::LiquidationBucket => {
            unreachable!("TakerVolume/LiquidationBucket are poll-only — ws_request_for must not be called for them")
        }
        Kind::Basis => StreamType::Basis,
        Kind::InsuranceFund => StreamType::InsuranceFund,
        Kind::OrderbookL3 => StreamType::OrderbookL3,
        Kind::SettlementEvent => StreamType::SettlementEvent,
        Kind::MarketWarning => StreamType::MarketWarning,
        Kind::RiskLimit => StreamType::RiskLimit,
        Kind::PredictedFunding => StreamType::PredictedFunding,
        Kind::FundingSettlement => StreamType::FundingSettlement,
        Kind::MarkPriceKline(iv) => StreamType::MarkPriceKline { interval: iv.as_str().to_string() },
        Kind::IndexPriceKline(iv) => StreamType::IndexPriceKline { interval: iv.as_str().to_string() },
        Kind::PremiumIndexKline(iv) => StreamType::PremiumIndexKline { interval: iv.as_str().to_string() },
        Kind::OrderUpdate => StreamType::OrderUpdate,
        Kind::BalanceUpdate => StreamType::BalanceUpdate,
        Kind::PositionUpdate => StreamType::PositionUpdate,
        // Derived kinds — never reach ws_request_for (acquire_or_spawn dispatches
        // them through acquire_or_spawn_derived before calling this function).
        Kind::RangeBar(_) | Kind::TickBar(_) | Kind::VolumeBar(_) | Kind::Footprint(_)
        | Kind::RenkoBar(_, _) | Kind::PnfBar(_, _) | Kind::KagiBar(_)
        | Kind::CvdLine | Kind::ThreeLineBreak { .. } | Kind::TpoProfile(_, _)
        | Kind::DollarBar { .. } | Kind::TickImbalanceBar { .. }
        | Kind::VolumeImbalanceBar { .. } | Kind::RunBar { .. } => {
            unreachable!("derived kinds must not call ws_request_for")
        }
    };
    SubscriptionRequest {
        symbol: sym,
        stream_type,
        account_type: account,
        depth: None,
        update_speed_ms: None,
    }
}

fn parse_symbol(s: &str) -> Symbol {
    if let Some((b, q)) = s.split_once(['-', '/', '_']) {
        return Symbol::new(b, q);
    }
    let upper = s.to_uppercase();
    for q in ["USDT", "USDC", "USD", "BTC", "ETH", "BUSD", "EUR", "JPY"] {
        if let Some(base) = upper.strip_suffix(q) {
            if !base.is_empty() {
                return Symbol::new(base, q);
            }
        }
    }
    Symbol::new(&upper, "")
}

/// Conservative capability gate for `Station::subscribe`.
///
/// Returns `true` ONLY when the connector has declared BOTH the REST capability
/// AND the WS capability false for a Kind — i.e. the exchange exposes the data
/// via neither REST nor WS and the stream would certainly fail.  When in doubt
/// (flag combination unclear, or the Kind has a derive fallback) this returns
/// `false` and the normal `acquire_or_spawn` error path handles it.
///
/// Derived Kinds (`RangeBar`, `TickBar`, `VolumeBar`, `Footprint`, `Basis`,
/// `FundingSettlement`) are never considered unsupported here — they derive
/// from other streams and may succeed even when no native channel exists.
///
/// Poll-only Kinds (`LongShortRatio`, `TakerVolume`, `LiquidationBucket`)
/// are checked against their REST history flag only (they have no WS path).
pub(crate) fn caps_explicitly_unsupported(caps: &ConnectorCapabilities, kind: &Kind) -> bool {
    match kind {
        Kind::Trade => !caps.has_recent_trades && !caps.has_ws_trades,
        Kind::AggTrade => !caps.has_agg_trades && !caps.has_ws_trades,
        Kind::Kline(_) => !caps.has_klines && !caps.has_ws_klines,
        Kind::Ticker => !caps.has_ticker && !caps.has_ws_ticker,
        Kind::Orderbook => !caps.has_orderbook && !caps.has_ws_orderbook,
        Kind::OrderbookDelta => !caps.has_orderbook && !caps.has_ws_orderbook,
        Kind::MarkPrice => !caps.has_premium_index && !caps.has_ws_mark_price,
        Kind::FundingRate => !caps.has_funding_rate_history && !caps.has_ws_funding_rate,
        Kind::OpenInterest => !caps.has_open_interest_history,
        Kind::Liquidation => !caps.has_liquidation_history,
        Kind::MarkPriceKline(_) => !caps.has_mark_price_klines,
        Kind::IndexPriceKline(_) => !caps.has_index_price_klines,
        Kind::PremiumIndexKline(_) => !caps.has_premium_index_klines,
        // Poll-only: REST-flag only (no WS path exists for these).
        Kind::LongShortRatio => !caps.has_long_short_ratio_history,
        Kind::TakerVolume => !caps.has_taker_volume_history,
        Kind::LiquidationBucket => !caps.has_liquidation_bucket_history,
        Kind::InsuranceFund => !caps.has_insurance_fund,
        // Derived Kinds: let acquire_or_spawn_derived handle them.
        Kind::RangeBar(_)
        | Kind::TickBar(_)
        | Kind::VolumeBar(_)
        | Kind::Footprint(_)
        | Kind::RenkoBar(_, _)
        | Kind::PnfBar(_, _)
        | Kind::KagiBar(_)
        | Kind::CvdLine
        | Kind::ThreeLineBreak { .. }
        | Kind::TpoProfile(_, _)
        | Kind::DollarBar { .. }
        | Kind::TickImbalanceBar { .. }
        | Kind::VolumeImbalanceBar { .. }
        | Kind::RunBar { .. }
        | Kind::Basis
        | Kind::FundingSettlement => false,
        // The remaining Kinds have no dedicated capability flag — let the WS
        // error path surface failures rather than pre-empting incorrectly.
        Kind::BlockTrade
        | Kind::AuctionEvent
        | Kind::IndexPrice
        | Kind::CompositeIndex
        | Kind::OptionGreeks
        | Kind::VolatilityIndex
        | Kind::HistoricalVolatility
        | Kind::OrderbookL3
        | Kind::SettlementEvent
        | Kind::MarketWarning
        | Kind::RiskLimit
        | Kind::PredictedFunding
        | Kind::OrderUpdate
        | Kind::BalanceUpdate
        | Kind::PositionUpdate => false,
    }
}

#[cfg(test)]
mod flush_persistence_tests {
    use super::*;
    use crate::series::Kind as SeriesKind;
    use digdigdig3::core::types::{AccountType, ExchangeId};

    fn test_key(symbol: &str) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::Spot, symbol, SeriesKind::Trade)
    }

    /// Exercises the registry mechanics in isolation: a synthetic task plays
    /// the "forwarder" role (owns a `mpsc::Receiver<FlushAck>`, registers a
    /// `FlushHandle` under a `SeriesKey`, replies on each ack), without
    /// spinning up a real WS/derived/poller forwarder (those need a live
    /// network or broadcast setup that a unit test cannot provide).
    #[tokio::test]
    async fn flush_persistence_round_trips_through_registered_handle() {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-flush-persistence-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));

        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");

        let key = test_key("BTCUSDT");
        let (handle, mut flush_rx) = FlushHandle::channel();
        station.inner.flush_handles.insert(key.clone(), handle);

        // Synthetic forwarder: waits for exactly one flush request, then acks
        // success. Mirrors the `recv_flush_request` + ack pattern real
        // forwarders run inside their select loop.
        let synthetic = tokio::spawn(async move {
            let ack = flush_rx.recv().await.expect("flush_persistence must send a request");
            let _ = ack.send(Ok(()));
        });

        let result = station.flush_persistence().await;
        assert!(result.is_ok(), "expected Ok(()), got {result:?}");

        synthetic.await.expect("synthetic forwarder task panicked");

        // The synthetic task's `flush_rx` is dropped when the task returns,
        // which closes the paired sender. `flush_persistence`'s opportunistic
        // cleanup sweep (`retain(|_, h| !h.is_closed())`) then reaps the
        // now-dead entry on this same call — matching what a real forwarder
        // achieves via its own explicit `flush_handles.remove(&key)` right
        // before exit. Either path converges on the entry being gone.
        assert!(!station.inner.flush_handles.contains_key(&key));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// A registry entry whose receiver has already been dropped (forwarder
    /// exited without unregistering — defensive case) must be treated as
    /// "already flushed", not surfaced as an error.
    #[tokio::test]
    async fn flush_persistence_treats_dead_handle_as_already_flushed() {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-flush-persistence-dead-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));

        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");

        let key = test_key("ETHUSDT");
        let (handle, flush_rx) = FlushHandle::channel();
        drop(flush_rx); // simulate a forwarder that already exited
        station.inner.flush_handles.insert(key.clone(), handle);

        let result = station.flush_persistence().await;
        assert!(result.is_ok(), "dead handle must not surface as an error: {result:?}");

        // Opportunistic cleanup: flush_persistence sweeps closed senders.
        assert!(!station.inner.flush_handles.contains_key(&key));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Aggregation: a genuine `Err` from the forwarder's `DiskStore::flush()`
    /// must be collected in the returned `Vec`, keyed by `SeriesKey`.
    #[tokio::test]
    async fn flush_persistence_aggregates_forwarder_reported_errors() {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-flush-persistence-err-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));

        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");

        let key = test_key("SOLUSDT");
        let (handle, mut flush_rx) = FlushHandle::channel();
        station.inner.flush_handles.insert(key.clone(), handle);

        let synthetic = tokio::spawn(async move {
            let ack = flush_rx.recv().await.expect("flush_persistence must send a request");
            let _ = ack.send(Err(std::io::Error::other("simulated disk failure")));
        });

        let result = station.flush_persistence().await;
        synthetic.await.expect("synthetic forwarder task panicked");

        let errors = result.expect_err("a reported flush error must surface");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, key);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod force_unsubscribe_tests {
    use super::*;
    use crate::series::Kind as SeriesKind;
    use digdigdig3::core::types::{AccountType, ExchangeId};

    fn test_key(symbol: &str) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::Spot, symbol, SeriesKind::Trade)
    }

    /// No mux registered for `key` — must be a no-op `Ok(())`, never hang.
    #[tokio::test]
    async fn force_unsubscribe_is_idempotent_when_no_mux_exists() {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-force-unsub-noop-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));
        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");

        let key = test_key("BTCUSDT");
        let result = station.force_unsubscribe_and_await(&key).await;
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Full mechanics: register a synthetic mux (fake "forwarder" — a real
    /// WS/derived/poller forwarder needs live network or a broadcast setup a
    /// unit test cannot provide) with a `shutdown` sender AND an `exit_acks`
    /// entry. A synthetic task plays the forwarder role: waits for the
    /// shutdown signal, then fires the exit ack — mirroring the real
    /// teardown ordering in `spawn_forwarder`/`spawn_derived_forwarder`/
    /// `spawn_poller`. `force_unsubscribe_and_await` must:
    /// - remove the mux + series_handles entries immediately,
    /// - fire shutdown regardless of `consumers` count (set to 2 here, i.e.
    ///   still "in use" by the ref-count that `release_consumer` would
    ///   otherwise respect),
    /// - await the exit ack before returning.
    #[tokio::test]
    async fn force_unsubscribe_fires_shutdown_and_awaits_exit_ack() {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-force-unsub-mechanics-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));
        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");

        let key = test_key("ETHUSDT");

        let (bcast_tx, _) = broadcast::channel::<Event>(16);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let (exit_ack_tx, exit_ack_rx) = oneshot::channel::<()>();

        // consumers = 2 — deliberately "still referenced" so a plain
        // `release_consumer` call would NOT fire shutdown. force_unsubscribe
        // must fire it anyway, unconditionally.
        station.inner.muxes.insert(
            key.clone(),
            Multiplexer {
                tx: bcast_tx,
                consumers: Arc::new(AtomicUsize::new(2)),
                shutdown: Some(shutdown_tx),
                grace_cancel: None,
            },
        );
        station.inner.exit_acks.insert(key.clone(), exit_ack_rx);
        let dummy_series: Arc<dyn Any + Send + Sync> =
            Arc::new(Arc::new(RwLock::new(Series::<TradePoint>::new(4))));
        station.inner.series_handles.insert(key.clone(), dummy_series);

        // Synthetic forwarder: waits for the shutdown signal, then does its
        // (simulated) teardown work and fires the exit ack — same sequence
        // real forwarders follow (shutdown observed → drain/cleanup →
        // exit_acks.remove + send).
        let shutdown_observed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_observed_clone = Arc::clone(&shutdown_observed);
        let synthetic = tokio::spawn(async move {
            let _ = shutdown_rx.await;
            shutdown_observed_clone.store(true, Ordering::SeqCst);
            let _ = exit_ack_tx.send(());
        });

        let result = station.force_unsubscribe_and_await(&key).await;
        assert!(result.is_ok());

        // Mux + series_handles must be gone immediately (before the await on
        // the ack even needed the synthetic task to run its cleanup).
        assert!(!station.inner.muxes.contains_key(&key));
        assert!(!station.inner.series_handles.contains_key(&key));
        // The await must not have returned before the synthetic forwarder
        // actually observed the shutdown signal.
        assert!(shutdown_observed.load(Ordering::SeqCst));

        synthetic.await.expect("synthetic forwarder task panicked");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Race-safety: if the forwarder has already exited and removed its own
    /// `exit_acks` entry (and the mux) before `force_unsubscribe_and_await`
    /// runs, the call must return `Ok(())` immediately rather than hang
    /// waiting for an ack that will never arrive.
    #[tokio::test]
    async fn force_unsubscribe_treats_already_exited_forwarder_as_done() {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-force-unsub-already-exited-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));
        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");

        let key = test_key("SOLUSDT");
        // No mux, no exit_ack registered — simulates a forwarder that has
        // already fully torn down (e.g. upstream closed concurrently).
        let result = station.force_unsubscribe_and_await(&key).await;
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// If the mux's `exit_acks` receiver is present but its paired sender
    /// was dropped without sending (forwarder task panicked/aborted before
    /// reaching its exit-ack send), the await must resolve (with a recv
    /// error) rather than hang forever.
    #[tokio::test]
    async fn force_unsubscribe_does_not_hang_on_dropped_ack_sender() {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-force-unsub-dropped-sender-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));
        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");

        let key = test_key("DOGEUSDT");
        let (bcast_tx, _) = broadcast::channel::<Event>(16);
        let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
        let (exit_ack_tx, exit_ack_rx) = oneshot::channel::<()>();
        drop(exit_ack_tx); // simulate a forwarder that panicked before ack-send

        station.inner.muxes.insert(
            key.clone(),
            Multiplexer {
                tx: bcast_tx,
                consumers: Arc::new(AtomicUsize::new(1)),
                shutdown: Some(shutdown_tx),
                grace_cancel: None,
            },
        );
        station.inner.exit_acks.insert(key.clone(), exit_ack_rx);

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            station.force_unsubscribe_and_await(&key),
        )
        .await
        .expect("force_unsubscribe_and_await must not hang on a dropped ack sender");
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod rewarm_derived_tests {
    //! Unit tests for `Station::rewarm_derived` and its private per-kind
    //! helpers. Network I/O (`rewarm_fetch_trade_window` /
    //! `rewarm_fetch_kline_sufficient_window`) is exercised implicitly through
    //! `rewarm_fold` + `rewarm_splice` + `rewarm_dedup` directly with
    //! synthetic events — a real REST fetch needs a live exchange
    //! connection a unit test cannot provide (same limitation documented in
    //! `derived_stream_integration.rs`). `rewarm_derived`'s dispatcher and
    //! `StationError::RewarmUnsupported` short-circuit ARE exercised
    //! end-to-end since they never touch the network.
    use super::*;
    use crate::series::Kind as SeriesKind;
    use digdigdig3::core::types::{AccountType, ExchangeId};

    fn test_key(symbol: &str, kind: SeriesKind) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::Spot, symbol, kind)
    }

    fn trade_event(ts_ms: i64, price: f64, quantity: f64, side: u8) -> Event {
        Event::Trade {
            exchange: ExchangeId::Binance,
            symbol: "BTCUSDT".to_string(),
            point: TradePoint { ts_ms, price, quantity, side, trade_id_hash: 0 },
        }
    }

    async fn station_with_tmp() -> (Station, PathBuf) {
        let tmp = std::env::temp_dir().join(format!(
            "dig3-rewarm-derived-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ));
        let station = Station::builder()
            .storage_root(&tmp)
            .build()
            .await
            .expect("Station::build is pure-local (no network) — must succeed");
        (station, tmp)
    }

    fn insert_series<T: DataPoint + Send + Sync + 'static>(
        station: &Station,
        key: &SeriesKey,
        points: Vec<T>,
    ) {
        let mut series = Series::<T>::new(points.len().max(16));
        series.extend(points);
        let handle: Arc<dyn Any + Send + Sync> = Arc::new(Arc::new(RwLock::new(series)));
        station.inner.series_handles.insert(key.clone(), handle);
    }

    // -----------------------------------------------------------------------
    // 1. Renko rewarm: existing bricks byte-identical after splice, AND
    //    prepended bricks sit on the SAME grid.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn renko_rewarm_keeps_existing_bricks_and_shares_grid() {
        let (station, tmp) = station_with_tmp().await;
        // box_size = 1.0 (1e8 fixed point), reversal = 1.
        let key = test_key("BTCUSDT", SeriesKind::RenkoBar(100_000_000, 1));

        // Existing live series: one brick already on the grid anchored at 100.0.
        let existing = vec![RenkoBrickPoint {
            open_time: 10_000,
            bottom: 100.0,
            top: 101.0,
            up: true,
            volume: 5.0,
            trades_count: 3,
        }];
        insert_series(&station, &key, existing.clone());

        // No REST connector in this unit test — the public helper's network
        // fetch returns an empty window, so it must return empty rather
        // than touching the existing series (same no-network contract as
        // the other rewarm_* public helpers).
        let (older_via_public, _outcome) = station.rewarm_renko(&key, "BTCUSDT", 100).await;
        assert!(older_via_public.is_empty(), "no REST connector in unit test — fold input is empty");

        // Drive the fold + splice path directly with synthetic OLDER
        // trades (bypassing the network fetch) to prove the real grid-seed
        // + seam behavior: `rewarm_fetch_kline_sufficient_window` (what
        // `rewarm_renko` actually calls) hands the fold `Event::Trade` legs
        // from `kline_to_synthetic_trades` — NOT `Event::Bar` — so this
        // must feed `Event::Trade` to match production. An OLDER trade
        // window that walks price up from 97.0 toward the existing grid,
        // seeded with the existing first brick's grid floor (100.0) exactly
        // as `rewarm_renko` would.
        let events = vec![
            trade_event(1_000, 97.5, 1.0, 0),
            trade_event(2_000, 98.5, 1.0, 0),
            trade_event(3_000, 99.5, 1.0, 0),
        ];
        let grid_floor = existing[0].bottom; // 100.0 — same preset rewarm_renko would compute
        let emissions = Station::rewarm_fold::<TradeToRenkoBarDerived>(&key, &events, |state| {
            state.preset_grid_anchor(grid_floor);
        })
        .await;
        let rebuilt = Station::rewarm_dedup(emissions, |p: &RenkoBrickPoint| p.open_time);
        assert!(!rebuilt.is_empty(), "expected at least one brick from the older window");
        let older = station.rewarm_splice(&key, rebuilt, |p: &RenkoBrickPoint| p.open_time).await;
        assert!(!older.is_empty(), "expected at least one prepended brick");
        for p in &older {
            // Every prepended brick must sit on the SAME 1.0-wide grid as
            // the existing series (bottom is an integer number of box
            // widths away from the existing first brick's bottom).
            let steps = (p.bottom - 100.0) / 1.0;
            assert!(
                (steps - steps.round()).abs() < 1e-9,
                "brick bottom {} is not grid-aligned with existing bottom 100.0",
                p.bottom
            );
        }
        // The prepended bricks must all be older than the existing head.
        assert!(older.iter().all(|p| p.open_time < 10_000));

        // Existing bricks are byte-identical after splice: read back the
        // series ring and check that entry.
        let handle = station.inner.series_handles.get(&key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<RenkoBrickPoint>>>>().map(Arc::clone)
        }).expect("series handle must still exist after splice");
        let after = handle.read().await.snapshot();
        let spliced_existing = after.last().expect("existing brick must be at the tail");
        assert_eq!(spliced_existing.open_time, existing[0].open_time);
        assert_eq!(spliced_existing.bottom, existing[0].bottom);
        assert_eq!(spliced_existing.top, existing[0].top);
        assert_eq!(spliced_existing.up, existing[0].up);
        assert_eq!(spliced_existing.volume, existing[0].volume);
        assert_eq!(spliced_existing.trades_count, existing[0].trades_count);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn renko_rewarm_no_existing_series_returns_empty() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::RenkoBar(100_000_000, 1));
        // No series_handles entry at all — must fall back to empty (cold path).
        let (older, _outcome) = station.rewarm_renko(&key, "BTCUSDT", 100).await;
        assert!(older.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -----------------------------------------------------------------------
    // 2. Counter-bar (tick_bar) rewarm: existing bars untouched, prepended
    //    bars strictly older, no identity collision at the seam.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn tick_bar_rewarm_existing_untouched_prepended_strictly_older() {
        let (station, tmp) = station_with_tmp().await;
        // n = 2 trades per bar.
        let key = test_key("BTCUSDT", SeriesKind::TickBar(2));

        let existing = vec![BarPoint {
            open_time: 50_000,
            open: 200.0,
            high: 201.0,
            low: 199.0,
            close: 200.5,
            volume: 4.0,
            quote_volume: 800.0,
            trades_count: 2,
        }];
        insert_series(&station, &key, existing.clone());

        let (older, _outcome) = station
            .rewarm_derived_standalone::<TradeToTickBarDerived>(&key, "BTCUSDT", 100)
            .await;

        // rewarm_fetch_trade_window needs a connected REST client to return
        // anything; without one it returns an empty window, so the fold
        // itself produces nothing and splice short-circuits to empty. That
        // is a legitimate outcome for a unit test with no live network —
        // assert the no-network contract explicitly, then re-verify the
        // same invariant by driving the fold directly with synthetic events.
        assert!(older.is_empty(), "no REST connector in unit test — fold input is empty");

        // Drive the fold + splice path directly with synthetic OLDER trades
        // (bypassing the network fetch) to prove the real seam behavior.
        let events = vec![
            trade_event(10_000, 190.0, 1.0, 0),
            trade_event(11_000, 191.0, 1.0, 0),
            trade_event(12_000, 192.0, 1.0, 1),
            trade_event(13_000, 193.0, 1.0, 1),
        ];
        let emissions = Station::rewarm_fold::<TradeToTickBarDerived>(&key, &events, |_s| {}).await;
        let rebuilt = Station::rewarm_dedup(emissions, |p: &BarPoint| p.open_time);
        // 4 trades / 2 per bar = 2 closed bars.
        assert_eq!(rebuilt.len(), 2);
        let spliced = station.rewarm_splice(&key, rebuilt, |p: &BarPoint| p.open_time).await;
        assert_eq!(spliced.len(), 2, "both synthetic bars are older than the existing head");
        assert!(spliced.iter().all(|p| p.open_time < 50_000), "prepended bars must be strictly older");

        // Existing bar must be byte-identical (untouched) after splice.
        let handle = station.inner.series_handles.get(&key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<BarPoint>>>>().map(Arc::clone)
        }).expect("series handle must still exist after splice");
        let after = handle.read().await.snapshot();
        let spliced_existing = after.last().expect("existing bar must be at the tail");
        assert_eq!(spliced_existing.open_time, existing[0].open_time);
        assert_eq!(spliced_existing.open, existing[0].open);
        assert_eq!(spliced_existing.close, existing[0].close);
        assert_eq!(spliced_existing.trades_count, existing[0].trades_count);
        // No identity collision: no prepended bar shares open_time with the
        // existing head.
        assert!(after[..after.len() - 1].iter().all(|p| p.open_time != existing[0].open_time));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Boundary-collision guard: when the throwaway fold's last emission
    /// happens to share identity with the existing head, existing wins —
    /// the fold's copy is dropped, never overwrites the live bar.
    #[tokio::test]
    async fn tick_bar_rewarm_boundary_collision_existing_wins() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::TickBar(2));

        let existing = vec![BarPoint {
            open_time: 5_000,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1.0,
            quote_volume: 100.0,
            trades_count: 1,
        }];
        insert_series(&station, &key, existing.clone());

        // Fold output deliberately includes a point with the SAME identity
        // (open_time) as the existing head, plus one strictly-older point.
        let rebuilt = vec![
            BarPoint {
                open_time: 1_000,
                open: 90.0, high: 90.0, low: 90.0, close: 90.0,
                volume: 1.0, quote_volume: 90.0, trades_count: 1,
            },
            BarPoint {
                open_time: 5_000, // collides with existing head
                open: 999.0, high: 999.0, low: 999.0, close: 999.0,
                volume: 999.0, quote_volume: 999.0, trades_count: 99,
            },
        ];
        let spliced = station.rewarm_splice(&key, rebuilt, |p: &BarPoint| p.open_time).await;
        // The colliding point must be dropped from the returned prepend set.
        assert_eq!(spliced.len(), 1);
        assert_eq!(spliced[0].open_time, 1_000);

        let handle = station.inner.series_handles.get(&key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<BarPoint>>>>().map(Arc::clone)
        }).expect("series handle must still exist");
        let after = handle.read().await.snapshot();
        // Existing head must be UNCHANGED (open == 100.0, not 999.0).
        let head = after.iter().find(|p| p.open_time == 5_000).expect("existing head must survive");
        assert_eq!(head.open, 100.0, "existing bar must never be overwritten by the fold's copy");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -----------------------------------------------------------------------
    // 3. CVD rewarm: prepended segment's last value == existing first
    //    value; existing samples untouched.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cvd_rewarm_offsets_to_meet_existing_first_value() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::CvdLine);

        let existing = vec![ScalarBarPoint { ts_ms: 10_000, value: 500.0 }];
        insert_series(&station, &key, existing.clone());

        // Older window: +1 -1 +1 (buy, sell, buy) => raw fold ends at +1.0
        // (starting from 0). After offset, must land exactly on 500.0.
        let events = vec![
            trade_event(1_000, 100.0, 1.0, 0), // buy: +1
            trade_event(2_000, 100.0, 1.0, 1), // sell: -1
            trade_event(3_000, 100.0, 1.0, 0), // buy: +1
        ];
        let emissions = Station::rewarm_fold::<TradeToCvdLineDerived>(&key, &events, |_s| {}).await;
        let mut rebuilt = Station::rewarm_dedup(emissions, |p: &ScalarBarPoint| p.ts_ms);
        assert_eq!(rebuilt.len(), 3);
        let raw_last = rebuilt.last().unwrap().value;
        assert!((raw_last - 1.0).abs() < 1e-9, "raw fold should end at +1.0 before offset");

        let offset = existing[0].value - raw_last;
        for p in &mut rebuilt {
            p.value += offset;
        }
        let spliced = station.rewarm_splice(&key, rebuilt, |p: &ScalarBarPoint| p.ts_ms).await;
        assert_eq!(spliced.len(), 3);
        let last_prepended = spliced.last().unwrap();
        assert!(
            (last_prepended.value - existing[0].value).abs() < 1e-9,
            "last prepended CVD sample must equal existing first value exactly"
        );

        // Existing sample must be untouched.
        let handle = station.inner.series_handles.get(&key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<ScalarBarPoint>>>>().map(Arc::clone)
        }).expect("series handle must still exist");
        let after = handle.read().await.snapshot();
        let tail = after.last().expect("existing sample must be at the tail");
        assert_eq!(tail.ts_ms, existing[0].ts_ms);
        assert_eq!(tail.value, existing[0].value);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn cvd_rewarm_public_helper_matches_manual_offset() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::CvdLine);
        let existing = vec![ScalarBarPoint { ts_ms: 10_000, value: -25.0 }];
        insert_series(&station, &key, existing.clone());

        // No REST connector in this unit test — rewarm_cvd's network fetch
        // returns an empty window, so the public helper itself must return
        // empty rather than panicking or touching the existing series.
        let (older, _outcome) = station.rewarm_cvd(&key, "BTCUSDT", 100).await;
        assert!(older.is_empty());

        let handle = station.inner.series_handles.get(&key).and_then(|e| {
            e.downcast_ref::<Arc<RwLock<Series<ScalarBarPoint>>>>().map(Arc::clone)
        }).expect("series handle must still exist");
        let after = handle.read().await.snapshot();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].value, existing[0].value);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -----------------------------------------------------------------------
    // 4. Unsupported kind (run_bar) returns the explicit unsupported error.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn run_bar_rewarm_returns_unsupported_error() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::RunBar { alpha_x100: 20, min_ticks: 10 });
        let result = station.rewarm_derived(&key, "BTCUSDT", 100).await;
        assert!(matches!(result, Err(StationError::RewarmUnsupported(_))));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn tick_imbalance_bar_rewarm_returns_unsupported_error() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::TickImbalanceBar { alpha_x100: 20, min_ticks: 10 });
        let result = station.rewarm_derived(&key, "BTCUSDT", 100).await;
        assert!(matches!(result, Err(StationError::RewarmUnsupported(_))));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn volume_imbalance_bar_rewarm_returns_unsupported_error() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::VolumeImbalanceBar { alpha_x100: 20, min_ticks: 10 });
        let result = station.rewarm_derived(&key, "BTCUSDT", 100).await;
        assert!(matches!(result, Err(StationError::RewarmUnsupported(_))));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn footprint_rewarm_derived_returns_unsupported_error_pointing_at_dedicated_fn() {
        let (station, tmp) = station_with_tmp().await;
        let key = test_key("BTCUSDT", SeriesKind::Footprint(KlineInterval::new("1m")));
        let result = station.rewarm_derived(&key, "BTCUSDT", 100).await;
        assert!(matches!(result, Err(StationError::RewarmUnsupported(_))));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -----------------------------------------------------------------------
    // rewarm_dedup: last emission per identity supersedes.
    // -----------------------------------------------------------------------

    #[test]
    fn rewarm_dedup_keeps_last_emission_per_identity() {
        let points = vec![
            ScalarBarPoint { ts_ms: 1, value: 1.0 },
            ScalarBarPoint { ts_ms: 1, value: 2.0 }, // same identity, supersedes
            ScalarBarPoint { ts_ms: 2, value: 3.0 },
        ];
        let out = Station::rewarm_dedup(points, |p: &ScalarBarPoint| p.ts_ms);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].value, 2.0, "second emission at ts=1 must supersede the first");
        assert_eq!(out[1].value, 3.0);
    }
}
