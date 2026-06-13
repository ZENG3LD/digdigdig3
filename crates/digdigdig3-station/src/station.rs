use std::any::Any;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::ws_health::WsHealth;

use dashmap::DashMap;
use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, SubscriptionRequest, Symbol, SymbolInfo,
};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};

use crate::data::{
    AggTradePoint, BalanceUpdatePoint, BarPoint, BasisPoint, BlockTradePoint, CompositeIndexPoint,
    FundingRatePoint, FundingSettlementPoint, HistoricalVolatilityPoint, IndexPriceKlinePoint,
    IndexPricePoint, InsuranceFundPoint, LiquidationPoint, LongShortRatioPoint,
    MarkPriceKlinePoint, MarkPricePoint,
    MarketWarningPoint, ObDeltaPoint, ObSnapshotPoint, OpenInterestPoint, OptionGreeksPoint,
    OrderUpdatePoint, OrderbookL3Point, PositionUpdatePoint,
    PredictedFundingPoint, PremiumIndexKlinePoint, RiskLimitPoint, SettlementEventPoint,
    TickerPoint, TradePoint, VolatilityIndexPoint,
};
use crate::derived::{BasisDerived, DerivedStream, FundingSettlementDerived, TradeToBarDerived, interval_to_ms};
#[cfg(not(target_arch = "wasm32"))]
use crate::polling;
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
                            ExchangeError::NotSupported(_)
                            | ExchangeError::UnsupportedOperation(_) => {
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

            // WS connect: skip if ALL streams in this entry are derived or
            // poll-only (they never touch a WS connector). For mixed entries —
            // e.g. [Stream::Trade, Stream::LongShortRatio] — the WS connect is
            // still needed for the Trade stream. Per-stream failures are reported
            // in `failed`; derived and poll-only streams are excluded from that.
            let needs_ws = entry.streams.iter().any(|s| {
                let kind = s.to_kind();
                !kind.is_derived() && kind.is_poll_only().is_none()
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
                        Err(digdigdig3::core::types::ExchangeError::UnsupportedOperation(
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
                        // Exclude derived and poll-only streams from the WS-connect
                        // failure list — they don't use WS.
                        let kind = s.to_kind();
                        if kind.is_derived() || kind.is_poll_only().is_some() {
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
                    // Only `continue` if there are no derived/poll-only streams
                    // that can still be acquired without WS.
                    let has_non_ws = entry.streams.iter().any(|s| {
                        let kind = s.to_kind();
                        kind.is_derived() || kind.is_poll_only().is_some()
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

                let (bcast_tx, pending_seed) = match self
                    .acquire_or_spawn(&key, &entry, &canonical, &raw, s)
                    .await
                {
                    Ok(pair) => pair,
                    Err(e) => {
                        // NotSupported on a per-(exchange, kind) basis: log
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
                        while let Ok(mut ev) = bcast_rx.recv().await {
                            ev.set_symbol(label.clone());
                            if tx_clone.send(ev).is_err() {
                                break;
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
                ok.push(key);
            }
        }

        Ok(SubscribeReport {
            handle: SubscriptionHandle { rx, _refs: refs },
            ok,
            failed,
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
        if key.kind.is_derived() {
            return match &key.kind {
                Kind::Basis => {
                    self.acquire_or_spawn_derived::<BasisDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                Kind::FundingSettlement => {
                    self.acquire_or_spawn_derived::<FundingSettlementDerived>(key, entry, canonical, raw_symbol).await.map(|tx| (tx, None))
                }
                _ => unreachable!("is_derived() returned true for non-derived kind"),
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
        // propagates any frame-construction failure (NotSupported and
        // UnsupportedOperation included). Map those to
        // `StreamNotSupported` so `Station::subscribe(set)` can bucket
        // them into `SubscribeReport::failed` without spawning a forwarder
        // that would loop in heal/resub forever (this is what caused
        // MLI's 0.3.6 OOM — see release-0.3.7-plan.md).
        //
        // Special case — Kind::Kline(iv): if the venue does not natively
        // support this interval on its WS, fall back to TradeToBarDerived
        // (trade-aggregation engine) rather than returning a hard error.
        // The fallback is attempted only when ws.subscribe fails with
        // NotSupported / UnsupportedOperation; native kline paths are
        // unchanged. If the interval string is unknown (interval_to_ms
        // returns None) we cannot build the aggregator either — return a
        // clear StreamNotSupported to the caller.
        if let Err(e) = ws.subscribe(req.clone()).await {
            use digdigdig3::core::types::WebSocketError;
            let is_not_supported = matches!(
                e,
                WebSocketError::NotSupported(_) | WebSocketError::UnsupportedOperation(_)
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
                WebSocketError::NotSupported(msg)
                | WebSocketError::UnsupportedOperation(msg) => {
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
                    crate::backfill::trades_recent(&hub, key.exchange, acct, &raw_s, warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<TradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::Kline(interval) => {
                let seed = if warm_n > 0 {
                    crate::backfill::klines_recent(&hub, key.exchange, acct, &raw_s, interval.as_str(), warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<BarPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), seed, req.clone());
            }
            Kind::AggTrade => spawn_forwarder::<AggTradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::Ticker => spawn_forwarder::<TickerPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::Orderbook => {
                let ob_seed = if self.inner.orderbook_rest_seed {
                    ob_rest_seed(&hub, key.exchange, acct, &raw_s, self.inner.orderbook_seed_depth).await
                } else {
                    Vec::new()
                };
                spawn_forwarder::<ObSnapshotPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), ob_seed, req.clone());
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
                spawn_forwarder::<ObDeltaPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone());
            }
            Kind::MarkPrice => spawn_forwarder::<MarkPricePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::FundingRate => spawn_forwarder::<FundingRatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::OpenInterest => spawn_forwarder::<OpenInterestPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::Liquidation => spawn_forwarder::<LiquidationPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::BlockTrade => spawn_forwarder::<BlockTradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::IndexPrice => spawn_forwarder::<IndexPricePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::CompositeIndex => spawn_forwarder::<CompositeIndexPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::OptionGreeks => spawn_forwarder::<OptionGreeksPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::VolatilityIndex => spawn_forwarder::<VolatilityIndexPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::HistoricalVolatility => spawn_forwarder::<HistoricalVolatilityPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            // LongShortRatio: poll-only, unreachable in normal operation (the
            // is_poll_only() branch above handles this first). Kept as defensive
            // fallback so the match arm is exhaustive.
            Kind::LongShortRatio => spawn_forwarder::<LongShortRatioPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::Basis => spawn_forwarder::<BasisPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::InsuranceFund => spawn_forwarder::<InsuranceFundPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::OrderbookL3 => spawn_forwarder::<OrderbookL3Point>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::SettlementEvent => spawn_forwarder::<SettlementEventPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::MarketWarning => spawn_forwarder::<MarketWarningPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::RiskLimit => spawn_forwarder::<RiskLimitPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::PredictedFunding => spawn_forwarder::<PredictedFundingPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::FundingSettlement => spawn_forwarder::<FundingSettlementPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::MarkPriceKline(_) => spawn_forwarder::<MarkPriceKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::IndexPriceKline(_) => spawn_forwarder::<IndexPriceKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::PremiumIndexKline(_) => spawn_forwarder::<PremiumIndexKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            // Private streams — no warm-start seed, no persistence (ephemeral by design).
            Kind::OrderUpdate => spawn_forwarder::<OrderUpdatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::BalanceUpdate => spawn_forwarder::<BalanceUpdatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::PositionUpdate => spawn_forwarder::<PositionUpdatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req),
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

        for dep_stream in D::deps() {
            let dep_kind = dep_stream.to_kind();
            debug_assert!(
                !dep_kind.is_derived(),
                "DerivedStream::deps() must not list derived kinds (no derived-of-derived)"
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

        let (bcast_tx, _) = broadcast::channel::<Event>(512);
        let consumers = Arc::new(AtomicUsize::new(1));
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        spawn_derived_forwarder::<D>(
            self,
            key,
            upstream_rxs,
            upstream_keys,
            bcast_tx.clone(),
            shutdown_rx,
            raw_symbol.to_string(),
        );

        self.inner.muxes.insert(
            key.clone(),
            Multiplexer { tx: bcast_tx.clone(), consumers, shutdown: Some(shutdown_tx), grace_cancel: None },
        );

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
) where
    Event: EventFrom<D::Output>,
{
    let inner = station.inner.clone();
    let key = key.clone();
    let storage_root = inner.storage_root.clone();
    let persistence = inner.persistence.clone();
    let warm = inner.warm_start_capacity;
    let exchange = key.exchange;

    {
        let derived_fut = Box::pin(async move {
            // Open disk store if persistence is on for this kind (native only).
            #[cfg(not(target_arch = "wasm32"))]
            let mut disk: Option<DiskStore<D::Output>> = None;
            #[cfg(not(target_arch = "wasm32"))]
            if persistence.is_enabled_for(&key.kind) {
                match DiskStore::<D::Output>::new(&storage_root, key.clone()).await {
                    Ok(store) => disk = Some(store),
                    Err(e) => tracing::warn!(?e, ?key, "derived: disk store open failed"),
                }
            }
            // On wasm, no disk store.
            #[cfg(target_arch = "wasm32")]
            let _ = (&storage_root, &persistence);

            let mut series = Series::<D::Output>::new(warm);

            // Derived streams start with no warm-start / backfill — see spec §11.
            let mut state = D::new_for_key(&key);

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
                let item_opt = tokio::select! {
                    _ = &mut shutdown_rx => break,
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
                    series.push(point.clone());
                    let _ = bcast_tx.send(Event::from_point(exchange, key.account_type, &symbol_label, &key.kind, point));
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(mut d) = disk { let _ = d.flush().await; }
            let _ = series;

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
        });
        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(derived_fut);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(derived_fut);
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

    {
    let forwarder_fut = Box::pin(async move {
        // Open disk store if persistence is on for this kind.
        // Native: std::fs-backed DiskStore. Wasm: OPFS-backed DiskStore.
        #[cfg(not(target_arch = "wasm32"))]
        let mut disk: Option<DiskStore<T>> = None;
        #[cfg(not(target_arch = "wasm32"))]
        if persistence.is_enabled_for(&key.kind) {
            match DiskStore::<T>::new(&storage_root, key.clone()).await {
                Ok(store) => disk = Some(store),
                Err(e) => tracing::warn!(?e, ?key, "disk store open failed"),
            }
        }
        // Wasm: OPFS DiskStore (Wave 4-E).
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
                // - Resub spam on a NotSupported stream was the trigger for
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
        Kind::IndexPrice => StreamType::IndexPrice,
        Kind::CompositeIndex => StreamType::CompositeIndex,
        Kind::OptionGreeks => StreamType::OptionGreeks,
        Kind::VolatilityIndex => StreamType::VolatilityIndex,
        Kind::HistoricalVolatility => StreamType::HistoricalVolatility,
        // LongShortRatio: poll-only kind. The WS arm is unreachable in normal
        // operation (acquire_or_spawn_polled handles it first), but kept as a
        // defensive fallback so the match is exhaustive.
        Kind::LongShortRatio => StreamType::LongShortRatio,
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
