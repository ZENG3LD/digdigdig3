//! # MEXC WebSocket Implementation
//!
//! WebSocket connector for MEXC Spot API using **Protobuf** encoding.
//!
//! ## Endpoint
//!
//! Uses `wss://wbs-api.mexc.com/ws` (the only active endpoint since Aug 2025).
//! The old `wss://wbs.mexc.com/ws` is deprecated and returns errors.
//!
//! ## Protocol
//!
//! - **Subscription/control messages**: JSON text frames
//! - **Market data pushes**: Binary frames with Protobuf encoding
//! - All channel names must use the `.pb` suffix (e.g., `spot@public.miniTicker.v3.api.pb@BTCUSDT@UTC+0`)
//! - Without `.pb` suffix, subscriptions are rejected with "Blocked!" error
//! - Protobuf schema: <https://github.com/mexcdevelop/websocket-proto>
//!
//! ## Channel Naming
//!
//! | Channel | Format |
//! |---------|--------|
//! | Mini Ticker | `spot@public.miniTicker.v3.api.pb@{SYMBOL}@UTC+0` |
//! | Aggre Deals | `spot@public.aggre.deals.v3.api.pb@100ms@{SYMBOL}` |
//! | Aggre Depth | `spot@public.aggre.depth.v3.api.pb@100ms@{SYMBOL}` |
//! | Book Ticker | `spot@public.bookTicker.v3.api.pb@{SYMBOL}` |
//! | Kline | `spot@public.kline.v3.api.pb@{SYMBOL}@{INTERVAL}` |
//!
//! ## Ping/Pong
//!
//! - Send: `{"method":"PING"}`
//! - Receive: `{"id":0,"code":0,"msg":"PONG"}`
//! - Server disconnects after 60s without ping
//!
//! ## Architecture
//!
//! The WebSocket stream is split into independent read and write halves on connect.
//! The write half is stored behind a mutex for shared access by `send_message` and
//! the ping task. The read half is owned exclusively by the message loop task —
//! no mutex contention on reads, which eliminates the deadlock that occurred when
//! both the reader (holding the full-stream mutex across `.next().await`) and the
//! writer (ping / subscribe) tried to acquire the same mutex simultaneously.
//!
//! ## Geo-Blocking Note
//!
//! The old non-`.pb` channels return "Blocked!" on the new endpoint. This is NOT
//! geo-blocking -- it is channel format deprecation. The `.pb` channels work from
//! Netherlands and most regions.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = MexcWebSocket::new(None).await?;
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe_ticker(Symbol::new("BTC", "USDT")).await?;
//!
//! let stream = ws.event_stream();
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(StreamEvent::Ticker(ticker)) => println!("{:?}", ticker),
//!         _ => {}
//!     }
//! }
//! ```

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;

use super::endpoints::{MexcUrls, MexcWsChannels, format_symbol};
use super::parser::MexcParser;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by send_message and the ping task
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task
type WsReader = SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// MEXC WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// MEXC WebSocket connector
pub struct MexcWebSocket {
    /// Authentication (None for public channels only)
    _auth: Option<()>, // MEXC doesn't use WebSocket auth for public channels
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event broadcast sender — std::sync::Mutex so event_stream() never contends
    /// with the async message loop (which only uses send(), an instant operation).
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by send_message and ping task.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Ping interval (20 seconds recommended)
    ping_interval: Duration,
    /// Last time a WS-level ping was sent (for RTT measurement)
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl MexcWebSocket {
    /// Create new MEXC WebSocket connector
    pub async fn new(_credentials: Option<Credentials>) -> ExchangeResult<Self> {
        Ok(Self {
            _auth: None, // MEXC doesn't use auth for public WebSocket
            account_type: AccountType::Spot, // Default
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            ping_interval: Duration::from_secs(20), // MEXC recommends ping every 10-20 seconds
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Send a raw JSON message over the write half.
    ///
    /// Only locks `ws_writer` — the reader half is owned separately by the
    /// message loop task, so there is no deadlock risk here.
    async fn send_message(&self, msg: &Value) -> ExchangeResult<()> {
        let msg_json = serde_json::to_string(msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize message: {}", e)))?;

        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard.as_mut()
            .ok_or_else(|| ExchangeError::Network("WebSocket not connected".to_string()))?;

        writer.send(Message::Text(msg_json)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    /// Subscribe to ticker stream (miniTicker, protobuf format).
    pub async fn subscribe_ticker(&self, symbol: Symbol) -> ExchangeResult<()> {
        let symbol_str = format_symbol(&symbol, self.account_type);
        let stream_name = MexcWsChannels::mini_ticker(&symbol_str);

        let msg = json!({
            "method": "SUBSCRIPTION",
            "params": [stream_name]
        });

        self.send_message(&msg).await?;

        // Add to subscriptions
        let mut subs = self.subscriptions.lock().await;
        subs.insert(SubscriptionRequest {
            stream_type: StreamType::Ticker,
            symbol: symbol.clone(),
            account_type: crate::core::AccountType::default(),
            depth: None,
            update_speed_ms: None,
        });

        Ok(())
    }

    /// Subscribe to trades stream (aggregated deals, protobuf format).
    pub async fn subscribe_trades(&self, symbol: Symbol) -> ExchangeResult<()> {
        let symbol_str = format_symbol(&symbol, self.account_type);
        let stream_name = MexcWsChannels::aggre_deals(&symbol_str);

        let msg = json!({
            "method": "SUBSCRIPTION",
            "params": [stream_name]
        });

        self.send_message(&msg).await?;

        // Add to subscriptions
        let mut subs = self.subscriptions.lock().await;
        subs.insert(SubscriptionRequest {
            stream_type: StreamType::Trade,
            symbol: symbol.clone(),
            account_type: crate::core::AccountType::default(),
            depth: None,
            update_speed_ms: None,
        });

        Ok(())
    }

    /// Start the message read loop.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// `ws_writer` is passed separately so the loop can send ping replies without
    /// touching the reader.
    ///
    /// The loop runs until the WebSocket connection closes or errors naturally.
    fn start_message_loop(
        mut reader: WsReader,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<Value>(&text) {
                            Ok(json) => {
                                // Check for PONG response: {"id":0,"code":0,"msg":"PONG"}
                                if json.get("msg").and_then(|m| m.as_str()) == Some("PONG") {
                                    continue;
                                }
                                // Also handle lowercase pong (legacy compat)
                                if json.get("msg").and_then(|m| m.as_str()) == Some("pong") {
                                    continue;
                                }

                                // Subscription confirmation: {"id":0,"code":0,"msg":"spot@public..."}
                                if json.get("code").and_then(|c| c.as_i64()) == Some(0) {
                                    if let Some(msg_str) = json.get("msg").and_then(|m| m.as_str()) {
                                        if msg_str.starts_with("spot@") {
                                            // Successful subscription confirmation - skip
                                            continue;
                                        }
                                    }
                                }

                                // Error detection: "Blocked!" means the channel format
                                // is wrong (non-.pb channels are rejected on new endpoint).
                                if let Some(msg_str) = json.get("msg").and_then(|m| m.as_str()) {
                                    if msg_str.contains("Blocked") || msg_str.contains("Not Subscribed successfully") {
                                        let tx_guard = event_tx.lock().unwrap();
                                        if let Some(ref tx) = *tx_guard {
                                            let _ = tx.send(Err(WebSocketError::Subscription(msg_str.to_string())));
                                        }
                                        continue;
                                    }
                                }

                                // Other JSON text messages — try to parse as a stream event
                                if let Ok((channel, data)) = MexcParser::parse_ws_message(&json) {
                                    let event_result = if channel.contains("deals") || channel.contains("bookTicker") || channel.contains("depth") {
                                        MexcParser::parse_ws_ticker(data).map(StreamEvent::Ticker)
                                    } else {
                                        Err(ExchangeError::Parse(format!("Unknown channel: {}", channel)))
                                    };

                                    if let Ok(event) = event_result {
                                        let tx_guard = event_tx.lock().unwrap();
                                        if let Some(ref tx) = *tx_guard {
                                            let _ = tx.send(Ok(event));
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let tx_guard = event_tx.lock().unwrap();
                                if let Some(ref tx) = *tx_guard {
                                    let _ = tx.send(Err(WebSocketError::Parse(e.to_string())));
                                }
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // MEXC sends all market data as protobuf binary frames.
                        match MexcParser::parse_protobuf_message(&data) {
                            Ok((_channel, event)) => {
                                let tx_guard = event_tx.lock().unwrap();
                                if let Some(ref tx) = *tx_guard {
                                    let _ = tx.send(Ok(event));
                                }
                            }
                            Err(e) => {
                                // Only log unexpected parse errors, not unknown channels
                                let err_msg = e.to_string();
                                if !err_msg.contains("Unsupported protobuf channel") {
                                    let tx_guard = event_tx.lock().unwrap();
                                    if let Some(ref tx) = *tx_guard {
                                        let _ = tx.send(Err(WebSocketError::Parse(err_msg)));
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        // Record RTT for the WS-level ping sent by start_ping_task
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
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
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

    /// Start the ping task.
    ///
    /// Uses only `ws_writer` — no contention with the reader half.
    /// The task exits naturally when the writer send fails (connection closed).
    /// Sends both a JSON `{"method":"PING"}` (MEXC application keepalive) and
    /// a WS-level `Message::Ping` (for RTT measurement via Pong response).
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        ping_interval: Duration,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(ping_interval);
            // Skip the immediate first tick
            interval.tick().await;

            loop {
                interval.tick().await;

                let ping_msg = json!({"method": "PING"});
                if let Ok(ping_text) = serde_json::to_string(&ping_msg) {
                    let mut writer_guard = ws_writer.lock().await;
                    if let Some(ref mut writer) = *writer_guard {
                        // Send application-level JSON ping (MEXC keepalive)
                        if writer.send(Message::Text(ping_text)).await.is_err() {
                            break;
                        }
                        // Send WS-level ping for RTT measurement
                        *last_ping.lock().await = Instant::now();
                        if writer.send(Message::Ping(vec![])).await.is_err() {
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
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for MexcWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        self.account_type = account_type;

        *self.status.lock().await = ConnectionStatus::Connecting;

        // Establish the connection
        let ws_url = MexcUrls::ws_url();
        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        // Split into independent read and write halves.
        // The write half goes behind a mutex for shared use by send_message and ping task.
        // The read half is passed directly to the message loop — no mutex needed.
        let (write, read) = ws_stream.split();
        *self.ws_writer.lock().await = Some(write);

        // Create the broadcast channel
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Start message loop — reader is moved in, never shared via mutex.
        Self::start_message_loop(
            read,
            self.event_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start ping task — uses ws_writer only.
        Self::start_ping_task(
            self.ws_writer.clone(),
            self.ping_interval,
            self.last_ping.clone(),
        );

        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Close the write half. The message loop task owns the read half and will
        // detect the close frame / stream termination naturally and exit on its own.
        // The ping task will fail on its next send attempt and also exit.
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            let _ = writer.close().await;
        }

        *self.status.lock().await = ConnectionStatus::Disconnected;
        self.subscriptions.lock().await.clear();

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.status.try_lock()
            .map(|guard| *guard)
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        match request.stream_type {
            StreamType::Ticker => {
                self.subscribe_ticker(request.symbol.clone()).await
                    .map_err(|e| WebSocketError::Subscription(e.to_string()))?;
            }
            StreamType::Trade => {
                self.subscribe_trades(request.symbol.clone()).await
                    .map_err(|e| WebSocketError::Subscription(e.to_string()))?;
            }
            _ => {
                return Err(WebSocketError::Subscription(
                    format!("Unsupported stream type: {:?}", request.stream_type)
                ));
            }
        }
        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Build the same channel name that was used during subscribe, then send UNSUBSCRIPTION
        let symbol_str = format_symbol(&request.symbol, self.account_type);

        let channel = match &request.stream_type {
            StreamType::Ticker => MexcWsChannels::mini_ticker(&symbol_str),
            StreamType::Trade => MexcWsChannels::aggre_deals(&symbol_str),
            StreamType::Orderbook | StreamType::OrderbookDelta => MexcWsChannels::aggre_depth(&symbol_str),
            StreamType::Kline { interval } => MexcWsChannels::kline(&symbol_str, interval),
            _ => {
                // Unknown or unsupported stream type — remove from local tracking only
                self.subscriptions.lock().await.remove(&request);
                return Ok(());
            }
        };

        let msg = json!({
            "method": "UNSUBSCRIPTION",
            "params": [channel]
        });

        self.send_message(&msg).await
            .map_err(|e| WebSocketError::Subscription(format!("Unsubscribe failed: {}", e)))?;

        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        // std::sync::Mutex::lock() never contends long — `send()` inside the loop
        // is an instant operation. This avoids the old tokio try_lock() which would
        // return an empty stream whenever the message loop held the lock.
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
        self.subscriptions.try_lock()
            .map(|guard| guard.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}
