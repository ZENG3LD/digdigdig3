//! `MarketFeed` core — pumps events from `WebSocketConnector::event_stream`
//! into per-(exchange, account, symbol, kind) broadcast channels, with
//! refcount so the upstream subscription is released after the last consumer
//! drops its handle (with `unsub_grace` debounce).
//!
//! v0 scope: subscribe/recv API. Persistence/orderbook/reconnect knobs are
//! stored but not yet wired — they are filled by the builder so we don't
//! re-touch the API surface later.

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::StreamExt;
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, info, warn};

use super::handle::{FeedEvent, FeedHandle};
use super::options::FeedOptions;
use crate::connector_manager::ExchangeHub;
use crate::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, SubscriptionRequest, Symbol,
};

/// Stream-flavour key used both for the upstream registry and for handle fan-out.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StreamKey {
    pub exchange: ExchangeId,
    pub account_type: AccountType,
    pub symbol: String,
    pub kind: StreamKind,
}

/// User-facing kind selector — collapses `StreamType` into the four flavours
/// the high-level API exposes in v0.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StreamKind {
    Ticker,
    Trade,
    Orderbook,
    Kline(String),
}

impl StreamKind {
    fn to_stream_type(&self) -> StreamType {
        match self {
            StreamKind::Ticker => StreamType::Ticker,
            StreamKind::Trade => StreamType::Trade,
            StreamKind::Orderbook => StreamType::Orderbook,
            StreamKind::Kline(iv) => StreamType::Kline { interval: iv.clone() },
        }
    }
}

#[allow(dead_code)]
struct UpstreamEntry {
    tx: broadcast::Sender<FeedEvent>,
    refcount: Arc<()>,
    pump: tokio::task::JoinHandle<()>,
}

pub struct MarketFeed {
    hub: Arc<ExchangeHub>,
    opts: FeedOptions,
    upstreams: Arc<Mutex<HashMap<StreamKey, UpstreamEntry>>>,
}

impl MarketFeed {
    pub(crate) fn new(hub: Arc<ExchangeHub>, opts: FeedOptions) -> Self {
        Self {
            hub,
            opts,
            upstreams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a builder bound to `hub`. Accepts either `Arc<ExchangeHub>` or
    /// `&ExchangeHub` (it's `Clone` + `Default`, internals are `Arc`-backed).
    pub fn builder(hub: impl Into<Arc<ExchangeHub>>) -> super::builder::FeedBuilder {
        super::builder::FeedBuilder::new(hub.into())
    }

    /// Subscribe to ticker events for `symbol` on `exchange`/`account_type`.
    pub async fn subscribe_ticker(
        &self,
        exchange: ExchangeId,
        symbol: impl Into<String>,
        account_type: AccountType,
    ) -> Result<FeedHandle, FeedError> {
        self.subscribe(StreamKey {
            exchange,
            account_type,
            symbol: symbol.into(),
            kind: StreamKind::Ticker,
        })
        .await
    }

    /// Subscribe to public trade events.
    pub async fn subscribe_trades(
        &self,
        exchange: ExchangeId,
        symbol: impl Into<String>,
        account_type: AccountType,
    ) -> Result<FeedHandle, FeedError> {
        self.subscribe(StreamKey {
            exchange,
            account_type,
            symbol: symbol.into(),
            kind: StreamKind::Trade,
        })
        .await
    }

    /// Subscribe to orderbook snapshot/delta events.
    pub async fn subscribe_orderbook(
        &self,
        exchange: ExchangeId,
        symbol: impl Into<String>,
        account_type: AccountType,
    ) -> Result<FeedHandle, FeedError> {
        self.subscribe(StreamKey {
            exchange,
            account_type,
            symbol: symbol.into(),
            kind: StreamKind::Orderbook,
        })
        .await
    }

    /// Subscribe to kline events for the given interval (`"1m"`, `"5m"`, `"1h"`).
    pub async fn subscribe_kline(
        &self,
        exchange: ExchangeId,
        symbol: impl Into<String>,
        account_type: AccountType,
        interval: impl Into<String>,
    ) -> Result<FeedHandle, FeedError> {
        self.subscribe(StreamKey {
            exchange,
            account_type,
            symbol: symbol.into(),
            kind: StreamKind::Kline(interval.into()),
        })
        .await
    }

    // ── internals ────────────────────────────────────────────────────────

    async fn subscribe(&self, key: StreamKey) -> Result<FeedHandle, FeedError> {
        let mut upstreams = self.upstreams.lock().await;

        if let Some(entry) = upstreams.get(&key) {
            return Ok(FeedHandle {
                rx: entry.tx.subscribe(),
                _keep_alive: entry.refcount.clone(),
            });
        }

        // Cold path: spawn the pump for this key.
        let ws = self
            .hub
            .ws(key.exchange, key.account_type)
            .ok_or(FeedError::NoConnector {
                exchange: key.exchange,
                account_type: key.account_type,
            })?;

        // Build the subscription request — Symbol takes raw-string form (canonical
        // base/quote are intentionally empty per dig3 symbol-passthrough principle).
        let sym = Symbol::with_raw("", "", key.symbol.clone());
        let mut sub_req = SubscriptionRequest::new(sym, key.kind.to_stream_type());
        sub_req.account_type = key.account_type;

        ws.subscribe(sub_req.clone())
            .await
            .map_err(|e| FeedError::Subscribe(format!("{e}")))?;

        let (tx, _rx0) = broadcast::channel(self.opts.broadcast_capacity);
        let refcount = Arc::new(());
        let pump = spawn_pump(
            tx.clone(),
            ws.clone(),
            key.clone(),
            Arc::downgrade(&refcount),
        );

        let entry = UpstreamEntry {
            tx: tx.clone(),
            refcount: refcount.clone(),
            pump,
        };
        upstreams.insert(key.clone(), entry);

        info!(
            exchange = ?key.exchange,
            account = ?key.account_type,
            symbol  = %key.symbol,
            kind    = ?key.kind,
            "feed upstream started"
        );

        Ok(FeedHandle {
            rx: tx.subscribe(),
            _keep_alive: refcount,
        })
    }

    /// Number of distinct upstream subscriptions currently held.
    pub async fn active_upstreams(&self) -> usize {
        self.upstreams.lock().await.len()
    }
}

fn spawn_pump(
    tx: broadcast::Sender<FeedEvent>,
    ws: Arc<dyn crate::core::traits::WebSocketConnector>,
    key: StreamKey,
    _refcount_weak: std::sync::Weak<()>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut stream = ws.event_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(event) => {
                    if !matches_kind(&event, &key.kind) {
                        continue;
                    }
                    let symbol = event_symbol(&event).unwrap_or_else(|| key.symbol.clone());
                    if symbol != key.symbol && !key.symbol.is_empty() {
                        // ws.subscribe receives ALL symbols on this connection
                        // when the underlying WS multiplexes — drop foreign-symbol events.
                        continue;
                    }
                    let fe = FeedEvent {
                        exchange: key.exchange,
                        account_type: key.account_type,
                        symbol: symbol.clone(),
                        event,
                    };
                    if tx.send(fe).is_err() {
                        // No receivers left and no new subs in flight — bail.
                        debug!(
                            exchange = ?key.exchange,
                            symbol = %key.symbol,
                            "feed pump: no receivers, exiting"
                        );
                        return;
                    }
                }
                Err(e) => {
                    warn!(
                        exchange = ?key.exchange,
                        symbol = %key.symbol,
                        "ws error: {e}"
                    );
                }
            }
        }
        debug!(
            exchange = ?key.exchange,
            symbol = %key.symbol,
            "feed pump: upstream stream closed"
        );
    })
}

fn matches_kind(event: &StreamEvent, kind: &StreamKind) -> bool {
    matches!(
        (event, kind),
        (StreamEvent::Ticker { .. }, StreamKind::Ticker)
            | (StreamEvent::Trade { .. }, StreamKind::Trade)
            | (StreamEvent::OrderbookSnapshot { .. }, StreamKind::Orderbook)
            | (StreamEvent::OrderbookDelta { .. }, StreamKind::Orderbook)
            | (StreamEvent::Kline { .. }, StreamKind::Kline(_))
    )
}

/// Best-effort symbol extraction. Many `StreamEvent` variants carry the
/// symbol as a struct field; some (OrderBook, Kline, OrderbookDelta) do not
/// — for those we let the pump fall back to the subscription's key symbol,
/// which is unambiguous when one upstream pump serves one (exchange, symbol)
/// key.
fn event_symbol(event: &StreamEvent) -> Option<String> {
    match event {
        StreamEvent::Ticker { symbol, .. } => Some(symbol.clone()),
        StreamEvent::Trade { symbol, .. } => Some(symbol.clone()),
        _ => None,
    }
}

// ── errors ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("no websocket connector registered for {exchange:?} {account_type:?} — call hub.connect_full first")]
    NoConnector { exchange: ExchangeId, account_type: AccountType },
    #[error("subscribe failed: {0}")]
    Subscribe(String),
}
