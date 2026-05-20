use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, OrderBook, OrderBookLevel, StreamEvent, SubscriptionRequest, Symbol,
};
use digdigdig3::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::orderbook::OrderBookTracker;
use crate::persistence::TradeWriter;
use crate::subscription::{Entry, Event, MultiplexRef, StreamKey, StreamKind};
use crate::{
    PersistenceConfig, Result, StationBuilder, StationError, SubscriptionHandle, SubscriptionSet,
};

/// Phase 2 Station. Owns an `ExchangeHub` and a `DashMap<StreamKey, Multiplexer>`
/// of shared per-(exchange, account, symbol, kind) actors. Each multiplexer
/// runs one WS subscription on the exchange, and fans events out to N consumers
/// via `broadcast::channel`. When the last consumer drops, the multiplexer
/// shuts down.
pub struct Station {
    pub(crate) inner: Arc<StationInner>,
}

pub(crate) struct StationInner {
    pub(crate) hub: Arc<ExchangeHub>,
    pub(crate) storage_root: PathBuf,
    pub(crate) persistence: PersistenceConfig,
    pub(crate) muxes: DashMap<StreamKey, Multiplexer>,
}

pub(crate) struct Multiplexer {
    pub(crate) tx: broadcast::Sender<StreamEvent>,
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
    pub fn builder() -> StationBuilder {
        StationBuilder::new()
    }

    pub fn storage_root(&self) -> &std::path::Path {
        &self.inner.storage_root
    }

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
            }),
        })
    }

    /// Number of currently-live multiplexer actors. For tests + `dig3 inspect`.
    pub fn active_streams(&self) -> usize {
        self.inner.muxes.len()
    }

    /// Connect each entry's WS, share multiplexer if (exchange, account, symbol,
    /// kind) already running, otherwise spawn a new one. Returns a single
    /// `SubscriptionHandle` that merges all requested streams.
    pub async fn subscribe(&self, set: SubscriptionSet) -> Result<SubscriptionHandle> {
        if set.is_empty() {
            return Err(StationError::Subscribe("empty SubscriptionSet".into()));
        }

        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        let mut refs: Vec<MultiplexRef> = Vec::new();

        for entry in set.entries {
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
                let Some(kind) = StreamKind::from_stream(s) else {
                    tracing::warn!(stream = ?s, "stream kind not yet supported by Station phase 2");
                    continue;
                };
                let key = StreamKey {
                    exchange: entry.exchange,
                    account_type: entry.account_type,
                    symbol_raw: raw.clone(),
                    kind: kind.clone(),
                };

                let bcast_tx = self.acquire_or_spawn(&key, &entry, &canonical, &raw).await?;

                let mut bcast_rx = bcast_tx.subscribe();
                let tx_clone = tx.clone();
                let exchange = entry.exchange;
                let symbol_label = entry.symbol.clone();
                let kind_clone = kind.clone();
                tokio::spawn(async move {
                    while let Ok(ev) = bcast_rx.recv().await {
                        let out = match (&kind_clone, ev) {
                            (StreamKind::Trade, StreamEvent::Trade(t)) => Event::Trade {
                                exchange,
                                symbol: symbol_label.clone(),
                                price: t.price,
                                quantity: t.quantity,
                                side: format!("{:?}", t.side),
                                timestamp: t.timestamp,
                            },
                            (StreamKind::Orderbook, StreamEvent::OrderbookSnapshot(ob)) => {
                                ob_event_from(exchange, &symbol_label, &ob)
                            }
                            _ => continue,
                        };
                        if tx_clone.send(out).is_err() {
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

    /// Get the broadcast sender for a `StreamKey`, spawning the multiplexer
    /// actor if not yet running. Increments the consumer ref count.
    async fn acquire_or_spawn(
        &self,
        key: &StreamKey,
        entry: &Entry,
        canonical: &Symbol,
        raw_symbol: &str,
    ) -> Result<broadcast::Sender<StreamEvent>> {
        if let Some(mux) = self.inner.muxes.get(key) {
            mux.consumers.fetch_add(1, Ordering::SeqCst);
            return Ok(mux.tx.clone());
        }

        let sym = Symbol::with_raw(&canonical.base, &canonical.quote, raw_symbol.to_string());
        let req = match key.kind {
            StreamKind::Trade => SubscriptionRequest::trade_for(sym, entry.account_type),
            StreamKind::Orderbook => SubscriptionRequest {
                symbol: sym,
                stream_type: digdigdig3::core::types::StreamType::Orderbook,
                account_type: entry.account_type,
                depth: None,
                update_speed_ms: None,
            },
        };

        let ws = self
            .inner
            .hub
            .ws(entry.exchange, entry.account_type)
            .ok_or_else(|| StationError::Core("ws handle missing post-connect".into()))?;
        ws.subscribe(req)
            .await
            .map_err(|e| StationError::Subscribe(format!("ws.subscribe: {e}")))?;

        let (bcast_tx, _) = broadcast::channel::<StreamEvent>(1024);
        let consumers = Arc::new(AtomicUsize::new(1));
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let mut stream = ws.event_stream();
        let bcast_for_task = bcast_tx.clone();
        let key_for_task = key.clone();
        let symbol_for_task = entry.symbol.clone();
        let exchange = entry.exchange;
        let storage_root = self.inner.storage_root.clone();
        let persist = self.inner.persistence.clone();
        let account_label = account_key_str(entry.account_type).to_string();
        let symbol_raw_for_task = raw_symbol.to_string();

        tokio::spawn(async move {
            let mut ob_tracker: Option<OrderBookTracker> = match key_for_task.kind {
                StreamKind::Orderbook => Some(OrderBookTracker::new(symbol_for_task.clone())),
                _ => None,
            };
            let mut trade_writer: Option<TradeWriter> = match key_for_task.kind {
                StreamKind::Trade if persist.enabled && persist.trades => {
                    match TradeWriter::new(
                        &storage_root,
                        &format!("{:?}", exchange).to_lowercase(),
                        &account_label,
                        &symbol_raw_for_task,
                    ) {
                        Ok(w) => Some(w),
                        Err(e) => {
                            tracing::warn!(?e, ?exchange, "trade writer open failed");
                            None
                        }
                    }
                }
                _ => None,
            };

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    item = stream.next() => {
                        let Some(item) = item else { break };
                        let ev = match item {
                            Ok(ev) => ev,
                            Err(e) => {
                                tracing::warn!(?e, "ws event_stream yielded err");
                                continue;
                            }
                        };

                        // Topic filter: route only events that belong to OUR
                        // (key.symbol_raw, key.kind). Other consumers on the same
                        // shared WS connection have their own muxes.
                        let route = match (&key_for_task.kind, &ev) {
                            (StreamKind::Trade, StreamEvent::Trade(t)) => {
                                eq_symbol(&t.symbol, &key_for_task.symbol_raw)
                            }
                            (StreamKind::Orderbook, StreamEvent::OrderbookSnapshot(_ob)) => {
                                // OrderBook struct lacks an inherent `symbol`. The
                                // event_stream from a per-(exchange, account) WS
                                // doesn't distinguish here in Phase 2; refine in
                                // step 7+ when symbol-routing tightens. For now
                                // we accept every snapshot.
                                true
                            }
                            (StreamKind::Orderbook, StreamEvent::OrderbookDelta(_)) => true,
                            _ => false,
                        };
                        if !route {
                            continue;
                        }

                        // Side-effects + downstream broadcast.
                        match (&key_for_task.kind, ev) {
                            (StreamKind::Trade, StreamEvent::Trade(t)) => {
                                if let Some(w) = trade_writer.as_mut() {
                                    if let Err(e) = w.append(t.timestamp, t.price, t.quantity, t.side, &t.id) {
                                        tracing::warn!(?e, "trade writer append failed");
                                    }
                                }
                                let _ = bcast_for_task.send(StreamEvent::Trade(t));
                            }
                            (StreamKind::Orderbook, StreamEvent::OrderbookSnapshot(ob)) => {
                                if let Some(tracker) = ob_tracker.as_mut() {
                                    if let Err(e) = tracker.apply_snapshot(&ob) {
                                        tracing::warn!(?e, "ob tracker snapshot apply failed");
                                    }
                                    let merged = build_ob_from_tracker(tracker);
                                    let _ = bcast_for_task.send(StreamEvent::OrderbookSnapshot(merged));
                                }
                            }
                            (StreamKind::Orderbook, StreamEvent::OrderbookDelta(d)) => {
                                if let Some(tracker) = ob_tracker.as_mut() {
                                    if let Err(e) = tracker.apply_delta(&d) {
                                        tracing::warn!(?e, "ob tracker delta apply failed");
                                    }
                                    let merged = build_ob_from_tracker(tracker);
                                    let _ = bcast_for_task.send(StreamEvent::OrderbookSnapshot(merged));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        self.inner.muxes.insert(
            key.clone(),
            Multiplexer {
                tx: bcast_tx.clone(),
                consumers,
                shutdown: Some(shutdown_tx),
            },
        );

        Ok(bcast_tx)
    }
}

impl StationInner {
    /// Decrement the consumer count for `key`; if it hits zero, shut down the
    /// multiplexer and remove the entry.
    pub(crate) fn release_consumer(self: &Arc<Self>, key: &StreamKey) {
        let should_remove = {
            let Some(mux) = self.muxes.get(key) else {
                return;
            };
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

fn eq_symbol(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn ob_event_from(exchange: ExchangeId, label: &str, ob: &OrderBook) -> Event {
    let bids = ob.bids.iter().map(|l| (l.price, l.size)).collect();
    let asks = ob.asks.iter().map(|l| (l.price, l.size)).collect();
    Event::OrderbookSnapshot {
        exchange,
        symbol: label.to_string(),
        bids,
        asks,
        timestamp: ob.timestamp,
    }
}

fn build_ob_from_tracker(t: &OrderBookTracker) -> OrderBook {
    let bids = t
        .top_bids(50)
        .into_iter()
        .map(|(p, q)| OrderBookLevel {
            price: dec_to_f64(p),
            size: dec_to_f64(q),
            order_count: None,
        })
        .collect();
    let asks = t
        .top_asks(50)
        .into_iter()
        .map(|(p, q)| OrderBookLevel {
            price: dec_to_f64(p),
            size: dec_to_f64(q),
            order_count: None,
        })
        .collect();
    OrderBook {
        bids,
        asks,
        timestamp: t.last_timestamp_ms(),
        sequence: None,
        last_update_id: t.last_update_id(),
        first_update_id: None,
        prev_update_id: None,
        event_time: None,
        transaction_time: None,
        checksum: None,
    }
}

fn dec_to_f64(d: rust_decimal::Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(0.0)
}

fn account_key_str(a: AccountType) -> &'static str {
    match a {
        AccountType::Spot => "spot",
        AccountType::Margin => "margin",
        AccountType::FuturesCross => "futures_cross",
        AccountType::FuturesIsolated => "futures_isolated",
        AccountType::Earn => "earn",
        AccountType::Lending => "lending",
        AccountType::Options => "options",
        AccountType::Convert => "convert",
    }
}
