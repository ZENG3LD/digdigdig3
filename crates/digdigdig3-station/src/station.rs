use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    StreamEvent, StreamType, SubscriptionRequest, Symbol,
};
use digdigdig3::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::data::{
    AggTradePoint, BarPoint, FundingRatePoint, LiquidationPoint, MarkPricePoint, ObSnapshotPoint,
    OpenInterestPoint, TickerPoint, TradePoint,
};
use crate::series::{DataPoint, DiskStore, Kind, Series, SeriesKey};
use crate::subscription::{Entry, Event, MultiplexRef, Stream};
use crate::{
    PersistenceConfig, Result, StationBuilder, StationError, SubscriptionHandle, SubscriptionSet,
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

    pub async fn subscribe(&self, set: SubscriptionSet) -> Result<SubscriptionHandle> {
        if set.is_empty() {
            return Err(StationError::Subscribe("empty SubscriptionSet".into()));
        }

        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        let mut refs: Vec<MultiplexRef> = Vec::new();

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
            self.inner
                .hub
                .connect_websocket(entry.exchange, entry.account_type, false)
                .await
                .map_err(|e| StationError::Core(format!("connect_websocket: {e}")))?;

            let canonical = parse_symbol(&entry.symbol);
            let raw = SymbolNormalizer::to_exchange(
                entry.exchange,
                &canonical,
                entry.account_type,
            )
            .map_err(|e| StationError::Subscribe(format!("symbol normalize: {e}")))?;

            for s in &entry.streams {
                let kind = s.to_kind();
                let key = SeriesKey {
                    exchange: entry.exchange,
                    account_type: entry.account_type,
                    symbol: raw.clone(),
                    kind: kind.clone(),
                };

                let bcast_tx = self.acquire_or_spawn(&key, &entry, &canonical, &raw, s).await?;

                let mut bcast_rx = bcast_tx.subscribe();
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    while let Ok(ev) = bcast_rx.recv().await {
                        if tx_clone.send(ev).is_err() {
                            break;
                        }
                    }
                });

                refs.push(MultiplexRef {
                    station: Arc::downgrade(&self.inner),
                    key,
                });
            }
        }

        Ok(SubscriptionHandle { rx, _refs: refs })
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
        ws.subscribe(req.clone())
            .await
            .map_err(|e| StationError::Subscribe(format!("ws.subscribe: {e}")))?;

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
                spawn_forwarder::<TradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), seed, req.clone());
            }
            Kind::Kline(interval) => {
                let seed = if warm_n > 0 {
                    crate::backfill::klines_recent(&hub, key.exchange, acct, &raw_s, interval, warm_n).await
                } else { Vec::new() };
                spawn_forwarder::<BarPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), seed, req.clone());
            }
            Kind::AggTrade => spawn_forwarder::<AggTradePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), Vec::new(), req.clone()),
            Kind::Ticker => spawn_forwarder::<TickerPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), Vec::new(), req.clone()),
            Kind::Orderbook => spawn_forwarder::<ObSnapshotPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), Vec::new(), req.clone()),
            Kind::MarkPrice => spawn_forwarder::<MarkPricePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), Vec::new(), req.clone()),
            Kind::FundingRate => spawn_forwarder::<FundingRatePoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), Vec::new(), req.clone()),
            Kind::OpenInterest => spawn_forwarder::<OpenInterestPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), Vec::new(), req.clone()),
            Kind::Liquidation => spawn_forwarder::<LiquidationPoint>(self, key, ws, bcast_tx.clone(), shutdown_rx, entry.symbol.clone(), Vec::new(), req),
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
                tracing::info!(target: "dig3::gap_heal", ?key, reason, "ws disconnect detected → heal + resub");
                // 1. REST heal (kline-only; no-op for other kinds).
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
            if matches!(&key.kind, Kind::Kline(_)) {
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
    let limit = crate::gap_heal::heal_limit(cfg, iv, last_emitted_ms, now_ms);

    tracing::info!(
        target: "dig3::gap_heal",
        ?key,
        last_emitted_ms,
        limit,
        "kline heal: pulling REST"
    );

    let pulled: Vec<T> = cast_vec(
        crate::gap_heal::heal_klines(hub, key.exchange, key.account_type, &key.symbol, iv, last_emitted_ms, limit).await
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
/// For events without a `symbol` field (OB), accept unconditionally — refine in
/// a later phase when per-symbol routing tightens (Phase 3+).
fn event_matches_key(ev: &StreamEvent, key: &SeriesKey) -> bool {
    let want = key.symbol.as_str();
    let got: Option<&str> = match ev {
        StreamEvent::Trade(t) => Some(&t.symbol),
        StreamEvent::AggTrade { symbol, .. } => Some(symbol),
        StreamEvent::Ticker(t) => Some(&t.symbol),
        StreamEvent::Kline(k) => k_symbol(k),
        StreamEvent::MarkPrice { symbol, .. } => Some(symbol),
        StreamEvent::FundingRate { symbol, .. } => Some(symbol),
        StreamEvent::OpenInterestUpdate { symbol, .. } => Some(symbol),
        StreamEvent::Liquidation { symbol, .. } => Some(symbol),
        StreamEvent::OrderbookSnapshot(_) | StreamEvent::OrderbookDelta(_) => None,
        _ => None,
    };
    match got {
        Some(s) => s.eq_ignore_ascii_case(want),
        None => true,
    }
}

fn k_symbol(_k: &digdigdig3::core::types::Kline) -> Option<&str> {
    // Kline struct lacks a `symbol` field; per-WS routing matches by topic upstream.
    None
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
        let timeframe = match kind { Kind::Kline(iv) => iv.clone(), _ => String::new() };
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

fn ws_request_for(
    kind: &Kind,
    sym: Symbol,
    account: digdigdig3::core::types::AccountType,
) -> SubscriptionRequest {
    let stream_type = match kind {
        Kind::Trade => StreamType::Trade,
        Kind::AggTrade => StreamType::AggTrade,
        Kind::Kline(iv) => StreamType::Kline { interval: iv.clone() },
        Kind::Ticker => StreamType::Ticker,
        Kind::Orderbook => StreamType::Orderbook,
        Kind::MarkPrice => StreamType::MarkPrice,
        Kind::FundingRate => StreamType::FundingRate,
        Kind::OpenInterest => StreamType::OpenInterest,
        Kind::Liquidation => StreamType::Liquidation,
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
