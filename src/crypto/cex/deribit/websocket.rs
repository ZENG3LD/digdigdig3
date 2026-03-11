//! # Deribit WebSocket Implementation
//!
//! WebSocket connector for Deribit using JSON-RPC 2.0 over WebSocket.
//!
//! ## Features
//! - Public and private channel subscriptions
//! - Automatic authentication (OAuth 2.0 over WebSocket)
//! - Heartbeat (test/ping every 30s)
//! - Broadcast channel pattern for event distribution
//! - JSON-RPC 2.0 message routing
//!
//! ## Architecture
//!
//! The WebSocket stream is split into independent read and write halves on connect.
//! The write half is stored behind a mutex for shared access by `send_request`,
//! `subscribe`, and the heartbeat task. The read half is owned exclusively by the
//! message loop task — no mutex contention on reads, which eliminates the deadlock
//! that occurred when both the reader and writer held the same mutex simultaneously.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = DeribitWebSocket::new(Some(credentials), false, AccountType::FuturesCross).await?;
//! ws.connect(AccountType::FuturesCross).await?;
//! ws.subscribe_ticker(Symbol::new("BTC", "USD")).await?;
//! ```

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use std::sync::Mutex as StdMutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeResult, ExchangeError,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;

use super::endpoints::DeribitUrls;
use super::auth::DeribitAuth;
use super::parser::DeribitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by send_request, subscribe, and heartbeat
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task
type WsReader = SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Deribit WebSocket connector
pub struct DeribitWebSocket {
    /// Authentication handler
    auth: Option<DeribitAuth>,
    /// URLs (mainnet/testnet)
    urls: DeribitUrls,
    /// Testnet mode
    _testnet: bool,
    /// Current account type
    _account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event broadcast sender — uses std::sync::Mutex so `subscribe()` can be called
    /// lock-free from `event_stream()` without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by send_request, subscribe, and heartbeat task.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Request ID counter
    request_id: Arc<Mutex<u64>>,
    /// Access token for authenticated requests
    access_token: Arc<Mutex<Option<String>>>,
    /// Last time a WS-level ping was sent (for RTT measurement)
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl DeribitWebSocket {
    /// Create new WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            DeribitUrls::TESTNET
        } else {
            DeribitUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(DeribitAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            _testnet: testnet,
            _account_type: account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(1)),
            access_token: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Get next request ID
    async fn next_id(&self) -> u64 {
        let mut id = self.request_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    /// Build JSON-RPC request
    fn build_request(&self, id: u64, method: &str, params: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        })
    }

    /// Send JSON-RPC request over WebSocket.
    ///
    /// Only locks `ws_writer` — the reader half is owned separately by the
    /// message loop task, so there is no deadlock risk here.
    async fn send_request(&self, method: &str, params: Value) -> ExchangeResult<u64> {
        let id = self.next_id().await;
        let request = self.build_request(id, method, params);

        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard.as_mut()
            .ok_or_else(|| ExchangeError::Network("WebSocket not connected".to_string()))?;

        let msg_text = serde_json::to_string(&request)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize request: {}", e)))?;

        writer.send(Message::Text(msg_text)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send message: {}", e)))?;

        Ok(id)
    }

    /// Authenticate via WebSocket
    async fn authenticate(&self) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("No credentials provided".to_string()))?;

        let params = auth.client_credentials_params();

        let params_json = serde_json::to_value(params)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize auth params: {}", e)))?;

        let _id = self.send_request("public/auth", params_json).await?;

        // The auth response (with access_token) is handled asynchronously in the message loop.

        Ok(())
    }

    /// Subscribe to channels
    async fn subscribe_channels(&self, channels: Vec<String>, is_private: bool) -> ExchangeResult<()> {
        let method = if is_private {
            "private/subscribe"
        } else {
            "public/subscribe"
        };

        let params = json!({
            "channels": channels
        });

        self.send_request(method, params).await?;
        Ok(())
    }

    /// Unsubscribe from channels
    async fn unsubscribe_channels(&self, channels: Vec<String>, is_private: bool) -> ExchangeResult<()> {
        let method = if is_private {
            "private/unsubscribe"
        } else {
            "public/unsubscribe"
        };

        let params = json!({
            "channels": channels
        });

        self.send_request(method, params).await?;
        Ok(())
    }

    /// Build channel name from subscription request
    fn build_channel_name(&self, request: &SubscriptionRequest) -> String {
        // Format symbol: BTC-PERPETUAL, ETH-PERPETUAL, etc.
        let instrument = if request.symbol.base.is_empty() {
            // Private channels don't need instrument
            String::new()
        } else {
            format!("{}-PERPETUAL", request.symbol.base.to_uppercase())
        };

        match &request.stream_type {
            StreamType::Ticker => format!("ticker.{}.100ms", instrument),
            StreamType::Trade => format!("trades.{}.100ms", instrument),
            StreamType::Orderbook => format!("book.{}.100ms", instrument),
            StreamType::OrderbookDelta => format!("book.{}.100ms", instrument),
            StreamType::Kline { interval } => {
                // Deribit uses chart.trades.{instrument}.{resolution}
                format!("chart.trades.{}.{}", instrument, interval)
            },
            StreamType::OrderUpdate => "user.orders.any.any.raw".to_string(),
            StreamType::BalanceUpdate => "user.portfolio.BTC".to_string(), // TODO: support multiple currencies
            StreamType::PositionUpdate => "user.changes.any.any.raw".to_string(),
            _ => String::new(),
        }
    }

    /// Check if subscription is private
    fn is_private_subscription(&self, request: &SubscriptionRequest) -> bool {
        matches!(
            request.stream_type,
            StreamType::OrderUpdate | StreamType::BalanceUpdate | StreamType::PositionUpdate
        )
    }

    /// Start message read loop.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// `ws_writer` is passed separately so the loop can send heartbeat replies
    /// without touching the reader.
    ///
    /// The loop runs until the WebSocket connection closes or errors naturally.
    /// There is no shutdown channel — the loop exits when the connection drops,
    /// which is the correct behaviour. Keeping a shutdown sender in the struct
    /// would cause the receiver to see a closed channel immediately when the
    /// struct is dropped (e.g. in bridge.rs after calling `event_stream()`),
    /// terminating the loop before any events are delivered.
    fn start_message_loop(
        mut reader: WsReader,
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        access_token: Arc<Mutex<Option<String>>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse JSON-RPC message
                        if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                            // Check if it's an auth response
                            if let Some(result) = parsed.get("result") {
                                if let Some(token) = result.get("access_token") {
                                    if let Some(token_str) = token.as_str() {
                                        let mut token_guard = access_token.lock().await;
                                        *token_guard = Some(token_str.to_string());
                                    }
                                }
                            }

                            // Check if it's a test_request heartbeat from server.
                            // Deribit sends: {"method": "heartbeat", "params": {"type": "test_request"}, "id": <N>}
                            // Client MUST reply with public/test echoing the same id, or Deribit
                            // closes the connection after ~10 seconds.
                            if let Some(method) = parsed.get("method") {
                                if method == "heartbeat" {
                                    let is_test_request = parsed
                                        .get("params")
                                        .and_then(|p| p.get("type"))
                                        .and_then(|t| t.as_str())
                                        == Some("test_request");

                                    if is_test_request {
                                        // Echo back the original id so Deribit accepts the reply.
                                        let original_id = parsed
                                            .get("id")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);

                                        let response = json!({
                                            "jsonrpc": "2.0",
                                            "id": original_id,
                                            "method": "public/test"
                                        });

                                        if let Ok(response_text) = serde_json::to_string(&response) {
                                            let mut writer_guard = ws_writer.lock().await;
                                            if let Some(ref mut writer) = *writer_guard {
                                                let _ = writer.send(Message::Text(response_text)).await;
                                            }
                                        }
                                    }
                                } else if method == "subscription" {
                                    // Parse and broadcast event
                                    if let Some(event) = Self::parse_event(&parsed) {
                                        let tx_guard = event_tx.lock().unwrap();
                                        if let Some(ref tx) = *tx_guard {
                                            let _ = tx.send(Ok(event));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        // Record RTT for the WS-level ping sent by start_ws_ping_task
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Ok(Message::Close(_)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Err(e) => {
                        let tx_guard = event_tx.lock().unwrap();
                        if let Some(ref tx) = *tx_guard {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(format!("WebSocket error: {}", e))));
                        }
                        break;
                    }
                    _ => {}
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            // from .next(). Without this, a clean close leaves the sender alive
            // and the bridge hangs forever instead of reconnecting.
            let _ = event_tx.lock().unwrap().take();
            // Stream exhausted — connection closed
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Parse event from JSON-RPC subscription notification
    fn parse_event(msg: &Value) -> Option<StreamEvent> {
        let params = msg.get("params")?;
        let channel = params.get("channel")?.as_str()?;
        let data = params.get("data")?;

        if channel.starts_with("ticker.") {
            DeribitParser::parse_ws_ticker(data).ok().map(StreamEvent::Ticker)
        } else if channel.starts_with("book.") {
            // parse_ws_orderbook already returns StreamEvent
            DeribitParser::parse_ws_orderbook(data).ok()
        } else if channel.starts_with("trades.") {
            DeribitParser::parse_ws_trade(data).ok().map(StreamEvent::Trade)
        } else if channel.starts_with("user.orders.") {
            DeribitParser::parse_ws_order_update(data).ok().map(StreamEvent::OrderUpdate)
        } else {
            // user.portfolio not yet implemented in parser
            None
        }
    }

    /// Start WS-level ping task for RTT measurement (every 5 seconds).
    ///
    /// Separate from the JSON heartbeat task — sends `Message::Ping` frames so
    /// the server responds with `Message::Pong`, allowing RTT measurement.
    fn start_ws_ping_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            // Skip immediate first tick
            interval.tick().await;

            loop {
                interval.tick().await;

                let mut writer_guard = ws_writer.lock().await;
                if let Some(ref mut writer) = *writer_guard {
                    *last_ping.lock().await = Instant::now();
                    if writer.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Start heartbeat task (send public/test every 30 seconds).
    ///
    /// Uses only `ws_writer` — no contention with the reader half.
    ///
    /// The task exits naturally when the writer send fails (connection closed).
    /// No shutdown channel is used — see `start_message_loop` for rationale.
    fn start_heartbeat_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        request_id: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            // Skip the immediate first tick so we don't send a heartbeat before
            // the connection is fully established.
            interval.tick().await;

            loop {
                interval.tick().await;

                let id = {
                    let mut id_guard = request_id.lock().await;
                    let current = *id_guard;
                    *id_guard += 1;
                    current
                };

                let test_msg = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "public/test"
                });

                if let Ok(msg_text) = serde_json::to_string(&test_msg) {
                    let mut writer_guard = ws_writer.lock().await;
                    if let Some(ref mut writer) = *writer_guard {
                        if writer.send(Message::Text(msg_text)).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for DeribitWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Update status
        *self.status.lock().await = ConnectionStatus::Connecting;

        // Connect to WebSocket
        let ws_url = self.urls.ws_url();
        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to connect: {}", e)))?;

        // Split into independent read and write halves.
        // The write half goes behind a mutex for shared use.
        // The read half is passed directly to the message loop — no mutex needed.
        let (write, read) = ws_stream.split();
        *self.ws_writer.lock().await = Some(write);

        // Create event broadcast channel
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Authenticate if credentials present.
        // Must happen after ws_writer is stored but before we start the read loop,
        // because authenticate() calls send_request() which uses ws_writer.
        if self.auth.is_some() {
            self.authenticate().await
                .map_err(|e| WebSocketError::Auth(format!("Authentication failed: {}", e)))?;
        }

        // Start message loop — reader is moved in, never shared via mutex.
        // The loop runs until the connection closes naturally; no shutdown channel
        // is needed or used.
        Self::start_message_loop(
            read,
            self.ws_writer.clone(),
            self.event_tx.clone(),
            self.status.clone(),
            self.access_token.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start WS-level ping task for RTT measurement
        Self::start_ws_ping_task(
            self.ws_writer.clone(),
            self.last_ping.clone(),
        );

        // Start heartbeat task — uses ws_writer only.
        // Exits naturally when the connection drops.
        Self::start_heartbeat_task(
            self.ws_writer.clone(),
            self.request_id.clone(),
        );

        // Update status
        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Close the write half. The message loop task owns the read half and will
        // detect the close frame / stream termination naturally and exit on its own.
        // The heartbeat task will fail on its next send attempt and also exit.
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            let _ = writer.close().await;
        }

        // Update status
        *self.status.lock().await = ConnectionStatus::Disconnected;

        // Clear subscriptions
        self.subscriptions.lock().await.clear();

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Note: We need to use blocking here since the trait method is not async
        // In production, we'd use a different pattern (e.g., Arc<AtomicU8>)
        match self.status.try_lock() {
            Ok(guard) => *guard,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Check connection
        if self.connection_status() != ConnectionStatus::Connected {
            return Err(WebSocketError::NotConnected);
        }

        // Build channel name
        let channel = self.build_channel_name(&request);

        if channel.is_empty() {
            return Err(WebSocketError::Subscription("Unsupported stream type".to_string()));
        }

        // Subscribe
        let is_private = self.is_private_subscription(&request);
        self.subscribe_channels(vec![channel], is_private).await
            .map_err(|e| WebSocketError::Subscription(format!("Subscribe failed: {}", e)))?;

        // Track subscription
        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Build channel name
        let channel = self.build_channel_name(&request);

        if channel.is_empty() {
            return Ok(());
        }

        // Unsubscribe
        let is_private = self.is_private_subscription(&request);
        self.unsubscribe_channels(vec![channel], is_private).await
            .map_err(|e| WebSocketError::Subscription(format!("Unsubscribe failed: {}", e)))?;

        // Remove from tracking
        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        // std::sync::Mutex::lock() never contends long — `send()` and `subscribe()` are
        // both instant operations.  This replaces the old tokio try_lock() which would
        // return an empty stream whenever the message loop held the lock (i.e. almost
        // always, at 100 ms ticker frequency).
        let tx_guard = self.event_tx.lock().unwrap();

        if let Some(ref tx) = *tx_guard {
            let rx = tx.subscribe();
            Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).map(|r| {
                r.map_err(|e| WebSocketError::ConnectionError(format!("Broadcast error: {}", e)))
                    .and_then(|x| x)
            }))
        } else {
            Box::pin(futures_util::stream::empty())
        }
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_lock() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}
