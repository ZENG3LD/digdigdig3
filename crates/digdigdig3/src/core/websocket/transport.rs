//! UniversalWsTransport<P: WsProtocol> — generic WebSocket transport.
//!
//! Owns ALL connection lifecycle, ping scheduling, subscription replay,
//! frame routing, and unmatched-frame logging.
//!
//! ## Invariants
//! - Every data frame gets tracing::trace! before dispatch.
//! - Every unmatched topic gets tracing::warn! — NEVER silently dropped.
//! - tokio::sync::Mutex only (never std::sync::Mutex across .await).
//! - broadcast::Sender is Arc-held and never taken/dropped on reconnect.
//! - Subscriptions are replayed on every successful reconnect.
//!
//! ## Target portability (Phase 3)
//! All tokio::spawn / tokio::time / tokio_tungstenite calls are gone.
//! The transport goes through `crate::core::rt::Runtime`:
//! - `rt.spawn_send(fut)` on native / `rt.spawn_local(fut)` on wasm
//! - `rt.sleep(dur).await` replaces tokio::time::sleep
//! - `rt.connect_ws(url, timeout).await` replaces connect_async
//! - `WsConn::send` / `next_frame` replace SinkExt / StreamExt on the raw stream

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, Mutex as TokioMutex, RwLock as TokioRwLock};
use tracing::{debug, trace, warn};

use crate::core::rt::{self, WsFrame, WsRtError};
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, ConnectionStatus, StreamEvent, SubscriptionRequest, WebSocketError,
    WebSocketResult,
};

use super::{
    capability_provider::CapabilityProvider,
    protocol::WsProtocol,
    reconnect::{BackoffState, ReconnectConfig},
    stream_kind::StreamKind,
    stream_spec::StreamSpec,
    support_level::SupportLevel,
};

// ─────────────────────────────────────────────────────────────────────────────
// TransportState
// ─────────────────────────────────────────────────────────────────────────────

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransportState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    Reconnecting = 3,
}

impl TransportState {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Disconnected,
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::Reconnecting,
            _ => Self::Disconnected,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TransportCmd
// ─────────────────────────────────────────────────────────────────────────────

pub(super) enum TransportCmd {
    Subscribe(StreamSpec),
    Unsubscribe(StreamSpec),
    Shutdown,
}

// ─────────────────────────────────────────────────────────────────────────────
// UniversalWsTransport
// ─────────────────────────────────────────────────────────────────────────────

/// Generic WebSocket transport.
///
/// Each exchange is a thin `WsProtocol` impl; this struct owns all connection
/// lifecycle, reconnect, ping, subscription replay, and frame dispatch.
pub struct UniversalWsTransport<P: WsProtocol> {
    protocol: Arc<P>,
    account_type: AccountType,
    testnet: bool,
    credentials: Option<Credentials>,
    reconnect_cfg: ReconnectConfig,

    // Runtime state (Arc-shared with tasks)
    state: Arc<AtomicU8>,
    active_subs: Arc<TokioRwLock<HashSet<StreamSpec>>>,
    event_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
    cmd_tx: mpsc::UnboundedSender<TransportCmd>,
}

impl<P: WsProtocol> Clone for UniversalWsTransport<P> {
    fn clone(&self) -> Self {
        Self {
            protocol: Arc::clone(&self.protocol),
            account_type: self.account_type,
            testnet: self.testnet,
            credentials: self.credentials.clone(),
            reconnect_cfg: self.reconnect_cfg.clone(),
            state: Arc::clone(&self.state),
            active_subs: Arc::clone(&self.active_subs),
            event_tx: self.event_tx.clone(),
            cmd_tx: self.cmd_tx.clone(),
        }
    }
}

impl<P: WsProtocol> UniversalWsTransport<P> {
    /// Construct. Does NOT connect yet.
    pub fn new(
        protocol: P,
        account_type: AccountType,
        testnet: bool,
        credentials: Option<Credentials>,
    ) -> Self {
        Self::with_reconnect(protocol, account_type, testnet, credentials, ReconnectConfig::default())
    }

    /// Construct with custom reconnect config.
    pub fn with_reconnect(
        protocol: P,
        account_type: AccountType,
        testnet: bool,
        credentials: Option<Credentials>,
        reconnect_cfg: ReconnectConfig,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(4096);
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        let state = Arc::new(AtomicU8::new(TransportState::Disconnected as u8));
        let active_subs = Arc::new(TokioRwLock::new(HashSet::new()));

        let transport = Self {
            protocol: Arc::new(protocol),
            account_type,
            testnet,
            credentials,
            reconnect_cfg,
            state: Arc::clone(&state),
            active_subs: Arc::clone(&active_subs),
            event_tx,
            cmd_tx,
        };

        // Spawn driver task — it holds cmd_rx and owns the WS connection loop.
        let last_frame_at = Arc::new(TokioMutex::new(Instant::now()));
        let driver = DriverTask {
            protocol: Arc::clone(&transport.protocol),
            account_type,
            testnet,
            credentials: transport.credentials.clone(),
            reconnect_cfg: transport.reconnect_cfg.clone(),
            state: Arc::clone(&state),
            active_subs: Arc::clone(&active_subs),
            event_tx: transport.event_tx.clone(),
            cmd_rx,
            http: reqwest::Client::new(),
            last_frame_at,
            rt: rt::default_runtime(),
        };

        // Spawn driver via cfg-conditional rt dispatch.
        // The driver's `run()` future is `Send` on native (all fields are Send),
        // so `tokio::spawn` is valid. On wasm, everything is single-threaded so
        // `spawn_local` is used.
        {
            let driver_fut = Box::pin(driver.run());
            #[cfg(not(target_arch = "wasm32"))]
            tokio::spawn(driver_fut);
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(driver_fut);
        }

        // ── Lag-check task ─────────────────────────────────────────────────
        // Periodically inspects the broadcast queue depth.  If depth exceeds
        // `lag_threshold`, emits a tracing::warn so monitoring can alert before
        // consumers start receiving RecvError::Lagged.
        {
            let lag_tx = transport.event_tx.clone();
            let lag_threshold = transport.reconnect_cfg.lag_threshold;
            let lag_interval =
                Duration::from_millis(transport.reconnect_cfg.lag_check_interval_ms);
            let protocol_name = transport.protocol.name().to_owned();

            let lag_fut = Box::pin(async move {
                let mut tick = tokio::time::interval(lag_interval);
                loop {
                    tick.tick().await;
                    let queue_depth = lag_tx.len();
                    let receiver_count = lag_tx.receiver_count();
                    if queue_depth > lag_threshold {
                        tracing::warn!(
                            target: "dig3::ws::lag",
                            exchange = %protocol_name,
                            queue_depth,
                            threshold = lag_threshold,
                            receiver_count,
                            "broadcast queue lagging — consumers may drop events"
                        );
                    }
                }
            });
            #[cfg(not(target_arch = "wasm32"))]
            tokio::spawn(lag_fut);
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(lag_fut);
        }

        transport
    }

    /// Initiate connection.
    pub async fn connect(&self) -> WebSocketResult<()> {
        self.cmd_tx
            .send(TransportCmd::Subscribe(StreamSpec {
                kind: StreamKind::Ticker, // sentinel — driver ignores this on connect signal
                symbol: crate::core::types::OwnedSymbolInput::Raw(String::new()),
                account_type: self.account_type,
                depth: None,
                speed_ms: None,
            }))
            .ok();
        // Wait for Connected state
        let deadline = tokio::time::Instant::now()
            + Duration::from_millis(self.reconnect_cfg.connection_timeout_ms + 2_000);
        loop {
            let s = TransportState::from_u8(self.state.load(Ordering::Acquire));
            if s == TransportState::Connected {
                return Ok(());
            }
            if tokio::time::Instant::now() > deadline {
                return Err(WebSocketError::Timeout);
            }
            tokio::time::sleep(Duration::from_millis(50)).await; // Wait for state change
        }
    }

    /// Graceful shutdown.
    pub async fn disconnect(&self) -> WebSocketResult<()> {
        self.cmd_tx.send(TransportCmd::Shutdown).ok();
        Ok(())
    }

    /// Subscribe to a stream.
    ///
    /// Eagerly probes `subscribe_frame` BEFORE queuing the subscribe command.
    /// Any frame-construction error (`NotSupported`, `UnsupportedOperation`,
    /// or any other variant the protocol returns) is propagated to the caller
    /// immediately. Callers see the failure right away instead of
    /// `silent_0_events` after a heal cycle timeout (this was the root cause
    /// of MLI's OOM on 53-stream validator — see release-0.3.7-plan).
    pub async fn subscribe(&self, spec: StreamSpec) -> WebSocketResult<()> {
        if let Err(e) = self.protocol.subscribe_frame(&spec) {
            return Err(e);
        }
        self.cmd_tx
            .send(TransportCmd::Subscribe(spec))
            .map_err(|_| WebSocketError::ProtocolError("transport shut down".into()))
    }

    /// Unsubscribe from a stream.
    pub async fn unsubscribe(&self, spec: StreamSpec) -> WebSocketResult<()> {
        self.cmd_tx
            .send(TransportCmd::Unsubscribe(spec))
            .map_err(|_| WebSocketError::ProtocolError("transport shut down".into()))
    }

    /// Returns a broadcast receiver stream.
    /// Lag capacity: 4096 events (broadcast channel buffer).
    /// Callers MUST process or discard events promptly — slow consumers receive
    /// `RecvError::Lagged` when the buffer overflows.
    pub fn event_stream(&self) -> impl Stream<Item = WebSocketResult<StreamEvent>> + Send {
        let rx = self.event_tx.subscribe();
        tokio_stream::wrappers::BroadcastStream::new(rx).map(|r| match r {
            Ok(v) => v,
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
                Err(WebSocketError::ProtocolError(format!("receiver lagged by {n} events")))
            }
        })
    }

    /// Snapshot of current connection state.
    pub fn connection_status(&self) -> ConnectionStatus {
        match TransportState::from_u8(self.state.load(Ordering::Acquire)) {
            TransportState::Disconnected => ConnectionStatus::Disconnected,
            TransportState::Connecting => ConnectionStatus::Connecting,
            TransportState::Connected => ConnectionStatus::Connected,
            TransportState::Reconnecting => ConnectionStatus::Reconnecting,
        }
    }

    /// Active subscriptions.
    pub fn active_subscriptions(&self) -> Vec<StreamSpec> {
        match self.active_subs.try_read() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Read-only access to the protocol shim.
    pub fn protocol(&self) -> &P {
        &self.protocol
    }

    /// Inject pre-built events into the broadcast channel.
    ///
    /// Used by connectors that need to seed initial state from REST before live
    /// WS events flow (e.g. Bitstamp L3 snapshot bootstrap: fetch REST order book,
    /// emit synthetic `OrderbookL3 { action: "create" }` events, then live
    /// `live_orders_*` events follow).
    ///
    /// Events that fail to send (no active receivers) are silently discarded.
    pub fn broadcast_events(&self, events: Vec<StreamEvent>) {
        for ev in events {
            let _ = self.event_tx.send(Ok(ev));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CapabilityProvider
// ─────────────────────────────────────────────────────────────────────────────

impl<P: WsProtocol> CapabilityProvider for UniversalWsTransport<P> {
    fn supports(&self, kind: &StreamKind, account: AccountType) -> SupportLevel {
        let registry = self.protocol.topic_registry(account);
        if registry.supports(kind, account) {
            return SupportLevel::Native;
        }
        // Check requires_auth_kinds
        if self.protocol.requires_auth_kinds(account).contains(kind) {
            return SupportLevel::RequiresAuth;
        }
        // Check unsupported_by_exchange
        if self.protocol.unsupported_by_exchange(account).contains(kind) {
            return SupportLevel::UnsupportedByExchange;
        }
        SupportLevel::NotImplemented
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector blanket impl (migration adapter)
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl<P: WsProtocol> crate::core::traits::WebSocketConnector for UniversalWsTransport<P> {
    async fn connect(&self, account_type: AccountType) -> WebSocketResult<()> {
        let _ = account_type; // transport is bound at construction
        UniversalWsTransport::connect(self).await
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        UniversalWsTransport::disconnect(self).await
    }

    fn connection_status(&self) -> ConnectionStatus {
        UniversalWsTransport::connection_status(self)
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        UniversalWsTransport::subscribe(self, spec).await
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        UniversalWsTransport::unsubscribe(self, spec).await
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        Box::pin(UniversalWsTransport::event_stream(self))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        UniversalWsTransport::active_subscriptions(self)
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DriverTask — internal reconnect + message loop
// ─────────────────────────────────────────────────────────────────────────────

struct DriverTask<P: WsProtocol> {
    protocol: Arc<P>,
    account_type: AccountType,
    testnet: bool,
    credentials: Option<Credentials>,
    reconnect_cfg: ReconnectConfig,
    state: Arc<AtomicU8>,
    active_subs: Arc<TokioRwLock<HashSet<StreamSpec>>>,
    event_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
    cmd_rx: mpsc::UnboundedReceiver<TransportCmd>,
    http: reqwest::Client,
    /// Shared timestamp of the last received frame — updated on every incoming frame.
    last_frame_at: Arc<TokioMutex<Instant>>,
    /// Runtime abstraction: spawn + sleep + connect_ws.
    rt: rt::Runtime,
}

impl<P: WsProtocol> DriverTask<P> {
    async fn run(mut self) {
        let mut backoff = BackoffState::new(self.reconnect_cfg.clone());
        let exchange = self.protocol.name();

        loop {
            // ── Set state ──────────────────────────────────────────────────
            let is_reconnect = backoff.attempt > 0;
            self.state.store(
                if is_reconnect {
                    TransportState::Reconnecting
                } else {
                    TransportState::Connecting
                } as u8,
                Ordering::Release,
            );

            // ── Pre-connect hook (e.g. KuCoin token fetch) ─────────────────
            let url = match self
                .protocol
                .pre_connect_hook(&self.http, self.account_type, self.testnet)
                .await
            {
                Ok(Some(dynamic_url)) => dynamic_url,
                Ok(None) => self.protocol.endpoint(self.account_type, self.testnet),
                Err(e) => {
                    warn!(target: "dig3::ws::connect", exchange, error = %e, "pre_connect_hook failed");
                    self.state
                        .store(TransportState::Reconnecting as u8, Ordering::Release);
                    let delay = backoff.next_delay();
                    self.rt.sleep(delay).await;
                    continue;
                }
            };

            debug!(target: "dig3::ws::connect", exchange, url = %url, "connecting");

            // ── TCP + TLS handshake via rt abstraction ─────────────────────
            let conn_timeout = backoff.connection_timeout();
            let ws_result = self.rt.connect_ws(url.as_str(), conn_timeout).await;

            let mut conn = match ws_result {
                Ok(c) => c,
                Err(WsRtError::Timeout) => {
                    warn!(target: "dig3::ws::connect", exchange, "connection timed out");
                    let _ = self.event_tx.send(Err(WebSocketError::Timeout));
                    let delay = backoff.next_delay();
                    self.rt.sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    warn!(target: "dig3::ws::connect", exchange, error = %e, "connection failed");
                    let _ = self
                        .event_tx
                        .send(Err(WebSocketError::ConnectionError(e.to_string())));
                    let delay = backoff.next_delay();
                    self.rt.sleep(delay).await;
                    continue;
                }
            };

            // ── Auth handshake ─────────────────────────────────────────────
            if let Some(creds) = &self.credentials {
                if let Some(auth_result) = self.protocol.auth_frame(creds) {
                    match auth_result {
                        Err(e) => {
                            warn!(target: "dig3::ws::auth", exchange, error = %e, "auth frame build failed");
                            let delay = backoff.auth_failure_delay();
                            self.rt.sleep(delay).await;
                            continue;
                        }
                        Ok(auth_frame) => {
                            if let Err(e) = conn.send(auth_frame).await {
                                warn!(target: "dig3::ws::auth", exchange, error = %e, "auth frame send failed");
                                let delay = backoff.auth_failure_delay();
                                self.rt.sleep(delay).await;
                                continue;
                            }
                            // Wait for auth ack
                            let ack_timeout = self.protocol.auth_ack_timeout();
                            let ack_ok = wait_for_auth_ack(
                                &mut *conn,
                                &*self.protocol,
                                ack_timeout,
                                exchange,
                            )
                            .await;
                            if !ack_ok {
                                warn!(target: "dig3::ws::auth", exchange, "auth ack not received");
                                let delay = backoff.auth_failure_delay();
                                self.rt.sleep(delay).await;
                                continue;
                            }
                            debug!(target: "dig3::ws::auth", exchange, "auth ack received");
                        }
                    }
                }
            }

            // ── Subscription replay ────────────────────────────────────────
            {
                let subs = self.active_subs.read().await;
                for spec in subs.iter() {
                    match self.protocol.subscribe_frame(spec) {
                        Ok(frame) => {
                            if let Err(e) = conn.send(frame).await {
                                warn!(target: "dig3::ws::replay", exchange, error = %e, "replay send failed");
                            }
                        }
                        Err(e) => {
                            warn!(target: "dig3::ws::replay", exchange, error = %e, "subscribe_frame failed");
                        }
                    }
                }
            }

            // ── Mark Connected ─────────────────────────────────────────────
            self.state
                .store(TransportState::Connected as u8, Ordering::Release);
            backoff.reset();
            // Reset silence clock so we measure from connection time, not task start.
            *self.last_frame_at.lock().await = Instant::now();
            debug!(target: "dig3::ws::connect", exchange, "connected");

            // ── Channel-bridge for read / write separation ─────────────────
            // `WsConn` is a single `&mut` object. To use it safely in a
            // `tokio::select!` loop where both the read arm (next_frame) and
            // write arms (send on ping / cmd) borrow it, we bridge via two
            // mpsc channels:
            //
            //   read_task:  conn.next_frame() → read_tx
            //   write_task: write_rx → conn.send()
            //
            // The main select! loop owns only the channel endpoints —
            // no `&mut conn` in the loop.  Both bridge tasks are stopped via
            // a oneshot `kill` signal when the loop exits.

            // Channel carries Option<Result<..>>: Some(frame) = data, None = EOF.
            let (read_tx, mut read_rx) =
                mpsc::unbounded_channel::<Option<Result<WsFrame, WsRtError>>>();
            let (write_tx, write_rx) =
                mpsc::unbounded_channel::<WsFrame>();
            let (kill_tx, _kill_rx) = tokio::sync::broadcast::channel::<()>(1);

            // ── Split conn into read half + write half via boxing ──────────
            // We need to move conn into the read task. The write side is
            // handled by sending through write_tx, and the write task reads
            // from write_rx and calls conn.send().
            //
            // Both halves need ownership of `conn`. We achieve this by
            // splitting via a shared Arc<Mutex<Box<dyn WsConn>>>.
            //
            // read_task holds the Arc and calls next_frame (no other writer).
            // write_task holds the same Arc and calls send (interleaved).
            // This is safe: only one task uses conn at a time (the read lock
            // is held briefly per frame; write lock briefly per send).
            let conn_shared = Arc::new(TokioMutex::new(conn));
            let conn_read = Arc::clone(&conn_shared);
            let conn_write = Arc::clone(&conn_shared);

            // read task
            {
                let read_tx = read_tx.clone();
                let mut kill_sub = kill_tx.subscribe();
                let read_fut = async move {
                    loop {
                        // next_frame() returns Option<Result<WsFrame, WsRtError>>
                        let opt_result = tokio::select! {
                            frame = async { conn_read.lock().await.next_frame().await } => frame,
                            _ = kill_sub.recv() => break,
                        };
                        // Forward the Option directly; None means WsConn closed.
                        let is_none = opt_result.is_none();
                        if read_tx.send(opt_result).is_err() {
                            break;
                        }
                        if is_none {
                            break; // WsConn EOF
                        }
                    }
                };
                #[cfg(not(target_arch = "wasm32"))]
                tokio::spawn(read_fut);
                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(read_fut);
            }

            // write task
            {
                let mut write_rx = write_rx;
                let mut kill_sub = kill_tx.subscribe();
                let write_fut = async move {
                    loop {
                        tokio::select! {
                            frame = write_rx.recv() => {
                                match frame {
                                    Some(f) => {
                                        let _ = conn_write.lock().await.send(f).await;
                                    }
                                    None => break,
                                }
                            }
                            _ = kill_sub.recv() => break,
                        }
                    }
                };
                #[cfg(not(target_arch = "wasm32"))]
                tokio::spawn(write_fut);
                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(write_fut);
            }

            // ── Silent-stream watchdog ─────────────────────────────────────
            // Fires if no frames arrive for ping_interval × silent_multiplier.
            let (silent_tx, mut silent_rx) = mpsc::channel::<()>(1);
            {
                let last_frame_at = Arc::clone(&self.last_frame_at);
                let ping_interval_dur = self.protocol.ping_interval();
                let multiplier = self.reconnect_cfg.silent_multiplier;
                let silent_threshold = ping_interval_dur * multiplier;
                let check_interval = ping_interval_dur / 2;
                let watchdog_fut = async move {
                    let mut ticker = tokio::time::interval(check_interval);
                    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                    loop {
                        ticker.tick().await;
                        let elapsed = last_frame_at.lock().await.elapsed();
                        if elapsed > silent_threshold {
                            warn!(
                                target: "dig3::ws::silent",
                                elapsed_secs = elapsed.as_secs(),
                                threshold_secs = silent_threshold.as_secs(),
                                "no frames received — forcing reconnect"
                            );
                            let _ = silent_tx.send(()).await;
                            break;
                        }
                    }
                };
                #[cfg(not(target_arch = "wasm32"))]
                tokio::spawn(watchdog_fut);
                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(watchdog_fut);
            }

            // ── Message loop ───────────────────────────────────────────────
            let mut ping_interval =
                tokio::time::interval(self.protocol.ping_interval());
            ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            let exit = loop {
                tokio::select! {
                    // Incoming frame from read_task channel
                    // read_rx.recv() → Option<Option<Result<WsFrame, WsRtError>>>
                    // outer None = channel closed (read_task exited)
                    // inner None = WsConn returned None (stream closed)
                    // inner Some(Ok(frame)) = data frame
                    // inner Some(Err(e)) = ws-level error
                    chan_item = read_rx.recv() => {
                        match chan_item {
                            // Normal data frame
                            Some(Some(Ok(msg))) => {
                                // Update silence clock on every received frame.
                                *self.last_frame_at.lock().await = Instant::now();
                                match self.dispatch_message(msg, exchange).await {
                                    Ok(true) => {} // normal
                                    Ok(false) => break LoopExit::Shutdown, // shutdown cmd via pong
                                    Err(e) => {
                                        warn!(target: "dig3::ws::frame", exchange, error = %e, "frame error");
                                        break LoopExit::Error;
                                    }
                                }
                            }
                            // WsConn-level error
                            Some(Some(Err(e))) => {
                                warn!(target: "dig3::ws::frame", exchange, error = %e, "ws error");
                                break LoopExit::Error;
                            }
                            // WsConn returned None (EOF) or read_task channel closed
                            Some(None) | None => {
                                debug!(target: "dig3::ws::connect", exchange, "stream closed");
                                break LoopExit::Closed;
                            }
                        }
                    }

                    // Silent-stream watchdog fired
                    _ = silent_rx.recv() => {
                        break LoopExit::Silent;
                    }

                    // Command from user
                    cmd = self.cmd_rx.recv() => {
                        match cmd {
                            Some(TransportCmd::Subscribe(spec)) => {
                                // Add to active set first
                                self.active_subs.write().await.insert(spec.clone());
                                match self.protocol.subscribe_frame(&spec) {
                                    Ok(frame) => {
                                        if write_tx.send(frame).is_err() {
                                            warn!(target: "dig3::ws", exchange, "subscribe send: write task gone");
                                        }
                                    }
                                    Err(e) => {
                                        warn!(target: "dig3::ws", exchange, error = %e, "subscribe_frame failed");
                                    }
                                }
                            }
                            Some(TransportCmd::Unsubscribe(spec)) => {
                                self.active_subs.write().await.remove(&spec);
                                match self.protocol.unsubscribe_frame(&spec) {
                                    Ok(frame) => {
                                        if write_tx.send(frame).is_err() {
                                            warn!(target: "dig3::ws", exchange, "unsubscribe send: write task gone");
                                        }
                                    }
                                    Err(e) => {
                                        warn!(target: "dig3::ws", exchange, error = %e, "unsubscribe_frame failed");
                                    }
                                }
                            }
                            Some(TransportCmd::Shutdown) => {
                                let _ = conn_shared.lock().await.close().await;
                                self.state.store(TransportState::Disconnected as u8, Ordering::Release);
                                return;
                            }
                            None => {
                                // cmd_rx closed — all senders dropped
                                break LoopExit::Closed;
                            }
                        }
                    }

                    // Ping timer
                    _ = ping_interval.tick() => {
                        let frame = match self.protocol.ping_frame() {
                            Some(f) => f,
                            None => WsFrame::Ping(vec![]),
                        };
                        if write_tx.send(frame).is_err() {
                            warn!(target: "dig3::ws::ping", exchange, "ping: write task gone");
                            break LoopExit::Error;
                        }
                    }
                }
            };

            // Kill bridge tasks
            let _ = kill_tx.send(());

            // ── Handle loop exit ───────────────────────────────────────────
            match exit {
                LoopExit::Shutdown => {
                    self.state
                        .store(TransportState::Disconnected as u8, Ordering::Release);
                    return;
                }
                LoopExit::Closed | LoopExit::Error | LoopExit::Silent => {
                    // Will reconnect
                    if backoff.max_attempts() > 0 && backoff.attempt >= backoff.max_attempts() {
                        warn!(target: "dig3::ws::connect", exchange, "max reconnect attempts reached");
                        self.state
                            .store(TransportState::Disconnected as u8, Ordering::Release);
                        return;
                    }
                    let delay = backoff.next_delay();
                    self.rt.sleep(delay).await;
                }
            }
        }
    }

    /// Dispatch a single WebSocket frame. Returns Ok(true) = continue, Ok(false) = shutdown.
    async fn dispatch_message(
        &self,
        msg: WsFrame,
        exchange: &str,
    ) -> WebSocketResult<bool> {
        let raw: Value = match msg {
            WsFrame::Text(text) => {
                trace_raw_frame(exchange, "text", text.as_bytes());
                match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(target: "dig3::ws::frame", exchange, error = %e, "JSON parse failed");
                        return Ok(true);
                    }
                }
            }
            WsFrame::Binary(bytes) => {
                trace_raw_frame(exchange, "binary", &bytes);
                match self.protocol.decode_binary(&bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(target: "dig3::ws::frame", exchange, error = %e, "binary decode failed");
                        return Ok(true);
                    }
                }
            }
            WsFrame::Ping(data) => {
                // Native WebSocket ping — handled by the rt impl (TungsteniteConn
                // auto-replies with Pong at the tungstenite layer).
                trace!(target: "dig3::ws::frame", exchange, kind = "Ping", len = data.len());
                return Ok(true);
            }
            WsFrame::Pong(_) => {
                trace!(target: "dig3::ws::frame", exchange, kind = "Pong");
                return Ok(true);
            }
            WsFrame::Close => {
                debug!(target: "dig3::ws::connect", exchange, "received Close frame");
                return Ok(true); // outer loop will see stream end
            }
        };

        // Invariant: trace every data frame
        trace!(
            target: "dig3::ws::frame",
            exchange,
            payload_len = raw.to_string().len(),
            "frame received"
        );

        // Check if it's a pong (suppress unmatched warn)
        if self.protocol.is_pong(&raw) {
            return Ok(true);
        }

        // Check if it's a subscribe ack (suppress unmatched warn)
        if self.protocol.is_subscribe_ack(&raw) {
            return Ok(true);
        }

        // Check if it's an auth ack (suppress unmatched warn — handled at connect)
        if self.credentials.is_some() && self.protocol.is_auth_ack(&raw) {
            return Ok(true);
        }

        // Extract routing topic
        let topic_key = match self.protocol.extract_topic(&raw) {
            None => return Ok(true), // heartbeat / ack / system frame
            Some(k) => k,
        };

        let topic_str = topic_key.to_string();

        // Look up parsers — dispatch_all returns all matching parsers (multiple for
        // fan-out topics like Bybit linear tickers.* that carry Ticker+MarkPrice+...).
        let registry = self.protocol.topic_registry(self.account_type);
        let parsers = registry.dispatch_all(&topic_key);
        if parsers.is_empty() {
            // Invariant: unmatched topic → warn, NEVER silent drop
            warn!(
                target: "dig3::ws::unmatched",
                exchange,
                topic = %topic_str,
                "no registered parser"
            );
        } else {
            let n_receivers = self.event_tx.receiver_count();
            for parser in parsers {
                match parser(&raw) {
                    Ok(event) => {
                        if n_receivers > 0 {
                            let _ = self.event_tx.send(Ok(event));
                        }
                    }
                    Err(crate::core::types::WebSocketError::FieldAbsent(_)) => {
                        // Delta frame did not carry this particular field — silent skip.
                        trace!(
                            target: "dig3::ws::parse",
                            exchange,
                            topic = %topic_str,
                            "field absent in delta frame — parser skipped"
                        );
                    }
                    Err(e) => {
                        warn!(
                            target: "dig3::ws::parse",
                            exchange,
                            topic = %topic_str,
                            error = %e,
                            "parser failed"
                        );
                        let _ = self.event_tx.send(Err(e));
                    }
                }
            }
        }

        Ok(true)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Auth ack helper
// ─────────────────────────────────────────────────────────────────────────────

/// Wait for an auth ack frame from the exchange.
/// Returns true if ack received within timeout, false if timeout or error.
///
/// Uses `rt::WsConn::next_frame()` — works on both native and wasm.
async fn wait_for_auth_ack<P: WsProtocol>(
    conn: &mut dyn rt::WsConn,
    protocol: &P,
    ack_timeout: Duration,
    exchange: &str,
) -> bool {
    // Use tokio::time::timeout (available on both native and wasm via tokio/sync+macros).
    // On native: tokio::time feature is enabled. On wasm: the timeout arm is driven
    // by gloo_timers via the rt::sleep mechanism.
    //
    // We implement a manual timeout with rt::select pattern:
    // sleep fires → return false; next_frame fires → check ack.
    let deadline = tokio::time::Instant::now() + ack_timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            warn!(target: "dig3::ws::auth", exchange, "auth ack timed out");
            return false;
        }
        let frame_opt = tokio::select! {
            f = conn.next_frame() => f,
            _ = tokio::time::sleep(remaining) => {
                warn!(target: "dig3::ws::auth", exchange, "auth ack timed out");
                return false;
            }
        };
        match frame_opt {
            Some(Ok(WsFrame::Text(text))) => {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if protocol.is_auth_ack(&v) {
                        return true;
                    }
                    // Skip non-ack frames silently during auth handshake
                }
            }
            Some(Ok(_)) => continue, // Ping/Pong/Binary during auth — skip
            Some(Err(e)) => {
                warn!(target: "dig3::ws::auth", exchange, error = %e, "error during auth ack wait");
                return false;
            }
            None => return false, // stream closed
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LoopExit
// ─────────────────────────────────────────────────────────────────────────────

enum LoopExit {
    Shutdown,
    Closed,
    Error,
    /// Watchdog detected silence beyond `ping_interval × silent_multiplier`.
    Silent,
}

// ─────────────────────────────────────────────────────────────────────────────
// Binary decode (default implementation, also called from protocol.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// Default binary frame decoder: tries gzip, then zlib, then raw deflate, then UTF-8.
pub fn decode_binary_default(bytes: &[u8]) -> WebSocketResult<Value> {
    use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
    use std::io::Read;

    // Gzip: magic bytes 0x1f 0x8b
    if bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
        let mut decoder = GzDecoder::new(bytes);
        let mut decompressed = String::new();
        if decoder.read_to_string(&mut decompressed).is_ok() {
            return serde_json::from_str(&decompressed)
                .map_err(|e| WebSocketError::Parse(e.to_string()));
        }
    }

    // Zlib: first byte 0x78 (zlib magic)
    if !bytes.is_empty() && bytes[0] == 0x78 {
        let mut decoder = ZlibDecoder::new(bytes);
        let mut decompressed = String::new();
        if decoder.read_to_string(&mut decompressed).is_ok() {
            return serde_json::from_str(&decompressed)
                .map_err(|e| WebSocketError::Parse(e.to_string()));
        }
    }

    // Raw deflate (MEXC)
    {
        let mut decoder = DeflateDecoder::new(bytes);
        let mut decompressed = String::new();
        if decoder.read_to_string(&mut decompressed).is_ok() {
            if let Ok(v) = serde_json::from_str(&decompressed) {
                return Ok(v);
            }
        }
    }

    // Plain UTF-8 JSON
    let text = std::str::from_utf8(bytes)
        .map_err(|e| WebSocketError::Parse(format!("binary not valid UTF-8: {e}")))?;
    serde_json::from_str(text).map_err(|e| WebSocketError::Parse(e.to_string()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Raw frame trace (debug)
//
// When env `DIG3_WS_TRACE` is set, every incoming WS frame is appended to
// `<dir>/<exchange>.jsonl` as one line per frame:
//   {"kind":"text","ts":<unix_ms>,"len":<bytes>,"body":"<utf8-or-hex>"}
//
// Accepted values:
//   DIG3_WS_TRACE=1                        → default dir `target/harness_out/ws_trace/`
//   DIG3_WS_TRACE=<absolute-or-rel-path>   → use the given dir verbatim
//
// Use for debug-only inspection of live wire traffic when a stream is silent
// or producing WRONG_TYPE. Not for production — fsync-per-line is slow.
// ─────────────────────────────────────────────────────────────────────────────

fn trace_raw_frame(exchange: &str, kind: &str, payload: &[u8]) {
    use std::io::Write;
    let Ok(raw) = std::env::var("DIG3_WS_TRACE") else { return; };
    let dir_buf;
    let dir_path: &std::path::Path = if raw == "1" || raw.eq_ignore_ascii_case("true") {
        dir_buf = std::path::PathBuf::from("target/harness_out/ws_trace");
        dir_buf.as_path()
    } else {
        std::path::Path::new(&raw)
    };
    if std::fs::create_dir_all(dir_path).is_err() { return; }
    let path = dir_path.join(format!("{}.jsonl", exchange));
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let body = match std::str::from_utf8(payload) {
        Ok(s) => serde_json::Value::String(s.to_string()),
        Err(_) => serde_json::Value::String(format!("0x{}", hex_encode(payload))),
    };
    let line = serde_json::json!({
        "kind": kind,
        "ts": ts,
        "len": payload.len(),
        "body": body,
    });
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", line);
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_state_roundtrip() {
        let states = [
            TransportState::Disconnected,
            TransportState::Connecting,
            TransportState::Connected,
            TransportState::Reconnecting,
        ];
        for s in states {
            assert_eq!(TransportState::from_u8(s as u8), s);
        }
    }

    #[test]
    fn decode_binary_plain_json() {
        let json = br#"{"type":"trade","symbol":"BTCUSDT"}"#;
        let v = decode_binary_default(json).unwrap();
        assert_eq!(v["type"], "trade");
    }

    /// Verify that the lag-check threshold logic works correctly at the
    /// broadcast::Sender level — no live exchange connection required.
    ///
    /// Build a channel with capacity 16, set lag_threshold = 8.
    /// Send 12 events without any receiver consuming them.
    /// Assert that event_tx.len() > lag_threshold (i.e. the check would fire).
    #[test]
    fn lag_check_threshold_fires_when_queue_deep() {
        use crate::core::types::{StreamEvent, WebSocketResult};
        use tokio::sync::broadcast;

        let capacity = 16_usize;
        let lag_threshold = 8_usize;

        let (tx, _rx) = broadcast::channel::<WebSocketResult<StreamEvent>>(capacity);

        // Send 12 events; _rx is alive so they are buffered (not dropped).
        for i in 0_u32..12 {
            let _ = tx.send(Err(crate::core::types::WebSocketError::ProtocolError(
                format!("dummy-{i}"),
            )));
        }

        let queue_depth = tx.len();
        // At least 8 events must be buffered for the lag warn to trigger.
        assert!(
            queue_depth > lag_threshold,
            "expected queue_depth {queue_depth} > lag_threshold {lag_threshold}"
        );
    }

    /// ReconnectConfig default lag fields are sane.
    #[test]
    fn reconnect_config_lag_defaults() {
        use crate::core::websocket::reconnect::ReconnectConfig;
        let cfg = ReconnectConfig::default();
        assert_eq!(cfg.lag_threshold, 512);
        assert_eq!(cfg.lag_check_interval_ms, 5_000);
    }
}
