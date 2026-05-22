use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    StreamEvent, StreamType, SubscriptionRequest, Symbol,
};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::data::{
    AggTradePoint, AuctionEventPoint, BarPoint, BasisPoint, BlockTradePoint, CompositeIndexPoint,
    FundingRatePoint, FundingSettlementPoint, HistoricalVolatilityPoint, IndexPriceKlinePoint,
    IndexPricePoint, InsuranceFundPoint, LiquidationPoint, MarkPriceKlinePoint, MarkPricePoint,
    MarketWarningPoint, ObSnapshotPoint, OpenInterestPoint, OptionGreeksPoint, OrderbookL3Point,
    PredictedFundingPoint, PremiumIndexKlinePoint, RiskLimitPoint, SettlementEventPoint,
    TickerPoint, TradePoint, VolatilityIndexPoint,
};
use crate::series::{DataPoint, DiskStore, Kind, Series, SeriesKey};
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
    pub(crate) warm_start_capacity: usize,
    pub(crate) gap_heal: crate::GapHealConfig,
}

/// One broadcast-fanout actor per `SeriesKey`. Each consumer increments
/// `consumers`; on the last drop the actor shuts down.
pub(crate) struct Multiplexer {
    pub(crate) tx: broadcast::Sender<Event>,
    pub(crate) consumers: Arc<AtomicUsize>,
    pub(crate) shutdown: Option<oneshot::Sender<()>>,
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

    pub(crate) async fn from_builder(b: StationBuilder) -> Result<Self> {
        let _ = digdigdig3::core::install_default_crypto_provider();
        if b.persistence.enabled {
            std::fs::create_dir_all(&b.storage_root).map_err(StationError::Io)?;
        }
        Ok(Self {
            inner: Arc::new(StationInner {
                hub: Arc::new(ExchangeHub::new()),
                storage_root: b.storage_root,
                persistence: b.persistence,
                muxes: DashMap::new(),
                warm_start_capacity: b.warm_start.max(1),
                gap_heal: b.gap_heal,
            }),
        })
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

            // WS connect: per-entry failure aborts every stream in this entry
            // (no point trying to subscribe through a connector that wouldn't
            // come up). Push one FailedStream per requested stream so the
            // consumer sees the full picture.
            if let Err(e) = self
                .inner
                .hub
                .connect_websocket(entry.exchange, entry.account_type, false)
                .await
            {
                let err_msg = format!("connect_websocket: {e}");
                for s in &entry.streams {
                    failed.push(FailedStream {
                        exchange: entry.exchange,
                        account_type: entry.account_type,
                        symbol: entry.symbol.clone(),
                        stream: s.clone(),
                        error: StationError::Core(err_msg.clone()),
                    });
                }
                continue;
            }

            let canonical = parse_symbol(&entry.symbol);
            let raw = match SymbolNormalizer::to_exchange(
                entry.exchange,
                &canonical,
                entry.account_type,
            ) {
                Ok(r) => r,
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
            };

            for s in &entry.streams {
                let kind = s.to_kind();
                let key = SeriesKey {
                    exchange: entry.exchange,
                    account_type: entry.account_type,
                    symbol: raw.clone(),
                    kind: kind.clone(),
                };

                let bcast_tx = match self
                    .acquire_or_spawn(&key, &entry, &canonical, &raw, s)
                    .await
                {
                    Ok(tx) => tx,
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
                tokio::spawn(async move {
                    while let Ok(mut ev) = bcast_rx.recv().await {
                        ev.set_symbol(label.clone());
                        if tx_clone.send(ev).is_err() {
                            break;
                        }
                    }
                });

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
    ) -> Result<broadcast::Sender<Event>> {
        if let Some(mux) = self.inner.muxes.get(key) {
            mux.consumers.fetch_add(1, Ordering::SeqCst);
            return Ok(mux.tx.clone());
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
        if let Err(e) = ws.subscribe(req.clone()).await {
            use digdigdig3::core::types::WebSocketError;
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
            Kind::Orderbook => spawn_forwarder::<ObSnapshotPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
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
            Kind::Basis => spawn_forwarder::<BasisPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::InsuranceFund => spawn_forwarder::<InsuranceFundPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::OrderbookL3 => spawn_forwarder::<OrderbookL3Point>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::SettlementEvent => spawn_forwarder::<SettlementEventPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::AuctionEvent => spawn_forwarder::<AuctionEventPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::MarketWarning => spawn_forwarder::<MarketWarningPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::RiskLimit => spawn_forwarder::<RiskLimitPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::PredictedFunding => spawn_forwarder::<PredictedFundingPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::FundingSettlement => spawn_forwarder::<FundingSettlementPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::MarkPriceKline(_) => spawn_forwarder::<MarkPriceKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::IndexPriceKline(_) => spawn_forwarder::<IndexPriceKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req.clone()),
            Kind::PremiumIndexKline(_) => spawn_forwarder::<PremiumIndexKlinePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, key.symbol.clone(), Vec::new(), req),
        }

        self.inner.muxes.insert(
            key.clone(),
            Multiplexer { tx: bcast_tx.clone(), consumers, shutdown: Some(shutdown_tx) },
        );

        Ok(bcast_tx)
    }
}

impl StationInner {
    pub(crate) fn release_consumer(self: &Arc<Self>, key: &SeriesKey) {
        let should_remove = {
            let Some(mux) = self.muxes.get(key) else { return; };
            let prev = mux.consumers.fetch_sub(1, Ordering::SeqCst);
            prev <= 1
        };
        if should_remove {
            if let Some((_, mut mux)) = self.muxes.remove(key) {
                if let Some(tx) = mux.shutdown.take() {
                    let _ = tx.send(());
                }
            }
        }
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

    tokio::spawn(async move {
        // Open disk store if persistence is on for this kind.
        let mut disk: Option<DiskStore<T>> = None;
        if persistence.is_enabled_for(&key.kind) {
            match DiskStore::<T>::new(&storage_root, key.clone()) {
                Ok(store) => disk = Some(store),
                Err(e) => tracing::warn!(?e, ?key, "disk store open failed"),
            }
        }

        // In-memory ring (warm capacity).
        let mut series = Series::<T>::new(warm);
        // Newest open_time emitted so far. Used to size disconnect heal and to
        // skip already-delivered bars after REST returns overlapping window.
        let mut last_emitted_ms: i64 = 0;

        // Warm-start. Priority: disk tail > REST seed.
        if warm > 0 {
            let disk_tail: Vec<T> = disk
                .as_ref()
                .and_then(|d| d.read_tail(warm).ok())
                .unwrap_or_default();
            let seed_points: Vec<T> = if disk_tail.is_empty() && !rest_seed.is_empty() {
                rest_seed
            } else {
                disk_tail
            };
            for p in &seed_points {
                let _ = bcast_tx.send(Event::from_point(exchange, &symbol_label, &key.kind, p.clone()));
                last_emitted_ms = last_emitted_ms.max(p.timestamp_ms());
            }
            series.seed(seed_points);
        }

        let mut stream = ws.event_stream();
        // Silence threshold: if `event_stream().next()` produces no event for
        // this long, the underlying WS is presumed dropped. Mirrors MLC
        // `ws_manager` behaviour (60s). Tunable via env for tests.
        let silence_timeout = std::time::Duration::from_secs(
            std::env::var("DIG3_WS_SILENCE_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60),
        );
        // Debug-only: artificially slow down the per-event loop. Used by e2e
        // tests to force broadcast-channel overflow → `Lagged` error →
        // `stream_err` branch. Production callers leave this unset (0 ms).
        let debug_slow_ms: u64 = std::env::var("DIG3_DEBUG_SLOW_CONSUMER_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        loop {
            // Single tokio::select! arm — exits via the shutdown branch or
            // detects (None / Err / silence). All three = disconnect = heal.
            let item_opt = tokio::select! {
                _ = &mut shutdown_rx => break,
                res = tokio::time::timeout(silence_timeout, stream.next()) => res,
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
                //    have already returned above).
                run_kline_heal::<T>(
                    &hub_for_heal, &key, &gap_cfg, &symbol_label,
                    last_emitted_ms, exchange,
                    &mut series, &mut disk, &bcast_tx,
                ).await;
                last_emitted_ms = last_emitted_ms.max(
                    series.last().map(|p| p.timestamp_ms()).unwrap_or(0)
                );
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

            if let Some(d) = disk.as_mut() {
                if let Err(e) = d.append(&point) {
                    tracing::warn!(?e, "disk store append failed");
                }
            }
            let pt_ts = point.timestamp_ms();
            // Klines: multiple in-flight updates share open_time — upsert
            // keeps the ring deduplicated. Other kinds are monotonic.
            if matches!(&key.kind, Kind::Kline(_) | Kind::MarkPriceKline(_) | Kind::IndexPriceKline(_) | Kind::PremiumIndexKline(_)) {
                series.upsert_by_ts(point.clone());
            } else {
                series.push(point.clone());
            }
            last_emitted_ms = last_emitted_ms.max(pt_ts);

            let _ = bcast_tx.send(Event::from_point(exchange, &symbol_label, &key.kind, point));

            if debug_slow_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(debug_slow_ms)).await;
            }
        }

        if let Some(mut d) = disk { let _ = d.flush(); }
        let _ = series; // dropped
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
    });
}

/// Run a kline auto-heal triggered by WS disconnect. Called only for
/// `Kind::Kline`; no-op for other kinds.
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
        let _ = bcast_tx.send(Event::from_point(exchange, symbol_label, &key.kind, p));
    }
    if let Some(d) = disk.as_mut() { let _ = d.flush(); }

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
/// variant. Implemented below for all 9 types.
trait EventFrom<T> {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, kind: &Kind, p: T) -> Self;
}

impl EventFrom<TradePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: TradePoint) -> Self {
        Event::Trade { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<AggTradePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: AggTradePoint) -> Self {
        Event::AggTrade { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<BarPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, kind: &Kind, point: BarPoint) -> Self {
        let timeframe = match kind { Kind::Kline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::Bar { exchange, symbol: symbol.to_string(), timeframe, point }
    }
}
impl EventFrom<TickerPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: TickerPoint) -> Self {
        Event::Ticker { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<ObSnapshotPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: ObSnapshotPoint) -> Self {
        Event::OrderbookSnapshot { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<MarkPricePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: MarkPricePoint) -> Self {
        Event::MarkPrice { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<FundingRatePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: FundingRatePoint) -> Self {
        Event::FundingRate { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<OpenInterestPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: OpenInterestPoint) -> Self {
        Event::OpenInterest { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<LiquidationPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: LiquidationPoint) -> Self {
        Event::Liquidation { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<BlockTradePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: BlockTradePoint) -> Self {
        Event::BlockTrade { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<IndexPricePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: IndexPricePoint) -> Self {
        Event::IndexPrice { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<CompositeIndexPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: CompositeIndexPoint) -> Self {
        Event::CompositeIndex { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<OptionGreeksPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: OptionGreeksPoint) -> Self {
        Event::OptionGreeks { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<VolatilityIndexPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: VolatilityIndexPoint) -> Self {
        Event::VolatilityIndex { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<HistoricalVolatilityPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: HistoricalVolatilityPoint) -> Self {
        Event::HistoricalVolatility { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<BasisPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: BasisPoint) -> Self {
        Event::Basis { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<InsuranceFundPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: InsuranceFundPoint) -> Self {
        Event::InsuranceFund { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<OrderbookL3Point> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: OrderbookL3Point) -> Self {
        Event::OrderbookL3 { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<SettlementEventPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: SettlementEventPoint) -> Self {
        Event::SettlementEvent { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<AuctionEventPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: AuctionEventPoint) -> Self {
        Event::AuctionEvent { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<MarketWarningPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: MarketWarningPoint) -> Self {
        Event::MarketWarning { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<RiskLimitPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: RiskLimitPoint) -> Self {
        Event::RiskLimit { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<PredictedFundingPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: PredictedFundingPoint) -> Self {
        Event::PredictedFunding { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<FundingSettlementPoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, _kind: &Kind, point: FundingSettlementPoint) -> Self {
        Event::FundingSettlement { exchange, symbol: symbol.to_string(), point }
    }
}
impl EventFrom<MarkPriceKlinePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, kind: &Kind, point: MarkPriceKlinePoint) -> Self {
        let timeframe = match kind { Kind::MarkPriceKline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::MarkPriceKline { exchange, symbol: symbol.to_string(), timeframe, point }
    }
}
impl EventFrom<IndexPriceKlinePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, kind: &Kind, point: IndexPriceKlinePoint) -> Self {
        let timeframe = match kind { Kind::IndexPriceKline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::IndexPriceKline { exchange, symbol: symbol.to_string(), timeframe, point }
    }
}
impl EventFrom<PremiumIndexKlinePoint> for Event {
    fn from_point(exchange: digdigdig3::core::types::ExchangeId, symbol: &str, kind: &Kind, point: PremiumIndexKlinePoint) -> Self {
        let timeframe = match kind { Kind::PremiumIndexKline(iv) => iv.clone(), _ => KlineInterval::new("") };
        Event::PremiumIndexKline { exchange, symbol: symbol.to_string(), timeframe, point }
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
        Kind::Basis => StreamType::Basis,
        Kind::InsuranceFund => StreamType::InsuranceFund,
        Kind::OrderbookL3 => StreamType::OrderbookL3,
        Kind::SettlementEvent => StreamType::SettlementEvent,
        Kind::AuctionEvent => StreamType::AuctionEvent,
        Kind::MarketWarning => StreamType::MarketWarning,
        Kind::RiskLimit => StreamType::RiskLimit,
        Kind::PredictedFunding => StreamType::PredictedFunding,
        Kind::FundingSettlement => StreamType::FundingSettlement,
        Kind::MarkPriceKline(iv) => StreamType::MarkPriceKline { interval: iv.as_str().to_string() },
        Kind::IndexPriceKline(iv) => StreamType::IndexPriceKline { interval: iv.as_str().to_string() },
        Kind::PremiumIndexKline(iv) => StreamType::PremiumIndexKline { interval: iv.as_str().to_string() },
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
