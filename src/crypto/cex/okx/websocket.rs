//! # OKX WebSocket Implementation
//!
//! WebSocket connector for OKX API v5.
//!
//! ## Features
//! - Public and private channels
//! - Text-based ping/pong (send "ping", receive "pong")
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Authentication
//! - Private channels require login via WebSocket message
//! - Signature: `timestamp + "GET" + "/users/self/verify"`
//!
//! ## Ping/Pong
//! - OKX uses text-based ping/pong (not WebSocket frames)
//! - Client sends text "ping" every 20 seconds
//! - Server responds with text "pong"
//!
//! ## Mutex starvation fix
//! - The WebSocket stream is split into a write half (sink) and a read half
//!   (stream) immediately after connecting.
//! - The ping task exclusively owns the sink.
//! - The message handler exclusively owns the read stream.
//! - Neither task can block the other by holding a shared lock during I/O.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Instant};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream,
};

use crate::core::{
    AccountType, ConnectionStatus, Credentials, ExchangeResult, OrderBook,
    StreamEvent, SubscriptionRequest, timestamp_iso8601,
};
use crate::core::types::OrderbookDelta;
use crate::core::traits::WebSocketConnector;
use crate::core::types::{WebSocketError, WebSocketResult, OrderbookCapabilities};

use super::auth::OkxAuth;
use super::endpoints::{format_symbol, OkxUrls};
use super::parser::OkxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WsStream, Message>;
type WsReader = futures_util::stream::SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// OKX WebSocket connector.
///
/// The underlying WebSocket stream is split into a write half (`ws_sink`) and a
/// read half (`ws_reader`) so that the ping task and the message handler each
/// hold only the half they need.  This eliminates the mutex-starvation problem
/// where `stream.next().await` would block the lock indefinitely between market
/// data messages, preventing the ping task from ever acquiring it.
pub struct OkxWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<OkxAuth>,
    /// URLs (mainnet/testnet)
    urls: OkxUrls,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender — behind StdMutex so event_stream() can subscribe
    /// without contending with the async message loop.
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Write half – used by `subscribe`, `unsubscribe`, `disconnect`, and the
    /// ping background task.
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    /// Read half – used exclusively by the message-handler background task.
    ws_reader: Arc<Mutex<Option<WsReader>>>,
    /// Timestamp of the most recently sent ping.
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
    /// Connected to private channel
    is_private: bool,
}

impl OkxWebSocket {
    /// Create new OKX WebSocket connector.
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            OkxUrls::TESTNET
        } else {
            OkxUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(OkxAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_sink: Arc::new(Mutex::new(None)),
            ws_reader: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
            is_private: false,
        })
    }

    /// Send the OKX WebSocket login message via `sink`.
    async fn send_login(&self, sink: &mut WsSink) -> WebSocketResult<()> {
        let auth = self.auth.as_ref().ok_or_else(|| {
            WebSocketError::Auth("Private channels require authentication".to_string())
        })?;

        let timestamp = timestamp_iso8601();
        let signature = auth.sign_websocket_login(&timestamp);

        let login_msg = json!({
            "op": "login",
            "args": [{
                "apiKey": auth.api_key(),
                "passphrase": auth.passphrase,
                "timestamp": timestamp,
                "sign": signature,
            }]
        });

        sink.send(Message::Text(login_msg.to_string()))
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        Ok(())
    }

    /// Start the ping background task.
    ///
    /// Acquires only the **sink** lock to send "ping" every 20 seconds.
    /// The read half is never touched here, so `start_message_handler` can
    /// block on `reader.next().await` without starving ping.
    fn start_ping_task(
        ws_sink: Arc<Mutex<Option<WsSink>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));

            loop {
                ticker.tick().await;

                let mut sink_guard = ws_sink.lock().await;
                if let Some(sink) = sink_guard.as_mut() {
                    // OKX uses text-based ping/pong, not WebSocket ping frames.
                    if sink.send(Message::Text("ping".to_string())).await.is_ok() {
                        *last_ping.lock().await = Instant::now();
                    } else {
                        // Connection lost; stop the task.
                        break;
                    }
                } else {
                    // Sink has been cleared (disconnect was called); stop.
                    break;
                }
            }
        });
    }

    /// Start the message-handler background task.
    ///
    /// Acquires only the **reader** lock.  Because `next().await` is called
    /// while the reader lock is held for the duration of each await, no other
    /// task competes for that lock — and the sink lock is never touched here.
    fn start_message_handler(
        ws_reader: Arc<Mutex<Option<WsReader>>>,
        broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            loop {
                // Poll the next message from the read half.
                let msg = {
                    let mut reader_guard = ws_reader.lock().await;
                    if let Some(reader) = reader_guard.as_mut() {
                        reader.next().await
                    } else {
                        break;
                    }
                };

                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Record round-trip time on pong response.
                        if text.trim() == "pong" {
                            let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                            *ws_ping_rtt_ms.lock().await = rtt;
                            continue;
                        }

                        // Parse JSON message.
                        if let Ok(value) = serde_json::from_str::<Value>(&text) {
                            // Handle event messages.
                            if let Some(event) = value.get("event").and_then(|e| e.as_str()) {
                                match event {
                                    "subscribe" | "unsubscribe" | "login" => {
                                        // Acknowledgment — nothing to broadcast.
                                        continue;
                                    }
                                    "error" => {
                                        let code = value
                                            .get("code")
                                            .and_then(|c| c.as_str())
                                            .unwrap_or("unknown");
                                        let msg_text = value
                                            .get("msg")
                                            .and_then(|m| m.as_str())
                                            .unwrap_or("Unknown error");
                                        let tx_guard = broadcast_tx.lock().unwrap();
                                        if let Some(ref tx) = *tx_guard {
                                            let _ = tx.send(Err(
                                                WebSocketError::ProtocolError(format!(
                                                    "{}: {}",
                                                    code, msg_text
                                                )),
                                            ));
                                        }
                                        continue;
                                    }
                                    _ => {}
                                }
                            }

                            // Handle data pushes.
                            if let Some(arg) = value.get("arg") {
                                if let Some(channel) =
                                    arg.get("channel").and_then(|c| c.as_str())
                                {
                                    // Extract top-level action ("snapshot" | "update")
                                    let action = value.get("action").and_then(|a| a.as_str());
                                    if let Some(data_arr) =
                                        value.get("data").and_then(|d| d.as_array())
                                    {
                                        for data in data_arr {
                                            let event =
                                                Self::parse_channel_data(channel, data, action);
                                            if let Some(ev) = event {
                                                let tx_guard = broadcast_tx.lock().unwrap();
                                                if let Some(ref tx) = *tx_guard {
                                                    let _ = tx.send(Ok(ev));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Some(Err(_)) | None => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {}
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            let _ = broadcast_tx.lock().unwrap().take();
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Parse channel data to [`StreamEvent`].
    ///
    /// `action` is taken from the top-level `"action"` field of the OKX push
    /// message.  OKX sets it to `"snapshot"` for the initial full book and
    /// `"update"` for incremental deltas.
    fn parse_channel_data(channel: &str, data: &Value, action: Option<&str>) -> Option<StreamEvent> {
        match channel {
            "tickers" => OkxParser::parse_ws_ticker(data)
                .ok()
                .map(StreamEvent::Ticker),
            "books" | "books5" | "books-l2-tbt" | "books50-l2-tbt" => {
                let (asks, bids) = OkxParser::parse_ws_orderbook(data).ok()?;
                let timestamp = OkxParser::get_i64(data, "ts").unwrap_or(0);

                // OKX sequences: seqId → first_update_id, prevSeqId → prev_update_id
                let seq_id = data.get("seqId").and_then(|v| v.as_u64());
                let prev_seq_id = data.get("prevSeqId").and_then(|v| v.as_u64());
                let checksum = data.get("checksum").and_then(|v| v.as_i64());

                if action == Some("snapshot") {
                    let orderbook = OrderBook {
                        asks,
                        bids,
                        timestamp,
                        sequence: None,
                        last_update_id: seq_id,
                        first_update_id: seq_id,
                        prev_update_id: prev_seq_id,
                        event_time: Some(timestamp),
                        transaction_time: None,
                        checksum,
                    };
                    Some(StreamEvent::OrderbookSnapshot(orderbook))
                } else {
                    // "update" or anything else → delta
                    let delta = OrderbookDelta {
                        asks,
                        bids,
                        timestamp,
                        first_update_id: seq_id,
                        last_update_id: seq_id,
                        prev_update_id: prev_seq_id,
                        event_time: Some(timestamp),
                        checksum,
                    };
                    Some(StreamEvent::OrderbookDelta(delta))
                }
            }
            "trades" => OkxParser::parse_ws_trade(data)
                .ok()
                .map(StreamEvent::Trade),
            "candle1m" | "candle5m" | "candle15m" | "candle30m" | "candle1H"
            | "candle4H" | "candle1D" => OkxParser::parse_ws_kline(data)
                .ok()
                .map(StreamEvent::Kline),
            "orders" => OkxParser::parse_ws_order_update(data)
                .ok()
                .map(StreamEvent::OrderUpdate),
            "account" => {
                if let Some(details) = data.get("details").and_then(|d| d.as_array()) {
                    for detail in details {
                        if let Ok(event) = OkxParser::parse_ws_balance_update(detail) {
                            return Some(StreamEvent::BalanceUpdate(event));
                        }
                    }
                }
                None
            }
            "positions" => OkxParser::parse_ws_position_update(data)
                .ok()
                .map(StreamEvent::PositionUpdate),
            _ => None,
        }
    }

    /// Returns the most recently measured WebSocket ping round-trip time in
    /// milliseconds.  Returns `0` until at least one pong has been received.
    pub fn ping_rtt_ms(&self) -> u64 {
        match self.ws_ping_rtt_ms.try_lock() {
            Ok(guard) => *guard,
            Err(_) => 0,
        }
    }

    /// Get a shared reference to the ping RTT value for external monitoring.
    ///
    /// The returned `Arc<Mutex<u64>>` is updated by the internal ping/pong
    /// handler each time a "pong" response is received from OKX.  Callers
    /// can cheaply poll the value (e.g. with `try_lock`) without blocking the
    /// WebSocket task.
    pub fn ping_rtt_handle(&self) -> Arc<Mutex<u64>> {
        self.ws_ping_rtt_ms.clone()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for OkxWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Determine URL (private vs public channel).
        let url = if self.auth.is_some() {
            self.is_private = true;
            self.urls.ws_url(true)
        } else {
            self.is_private = false;
            self.urls.ws_url(false)
        };

        // Establish the WebSocket connection.
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Split into independent read and write halves — this is the core fix.
        // With a unified stream, `next().await` in the message handler holds
        // the mutex for the entire duration between messages, starving the ping
        // task.  After splitting, each half has its own mutex and neither task
        // can block the other.
        let (mut sink, reader) = ws_stream.split();

        // Send login before storing halves (only the sink is needed here).
        if self.is_private {
            self.send_login(&mut sink).await?;
        }

        // Store both halves, replacing any previous connection.
        *self.ws_sink.lock().await = Some(sink);
        *self.ws_reader.lock().await = Some(reader);

        *self.status.lock().await = ConnectionStatus::Connected;

        // Create broadcast channel and store sender
        let (tx, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(tx);

        // Start background tasks — each holds only its own half.
        Self::start_ping_task(self.ws_sink.clone(), self.last_ping.clone());
        Self::start_message_handler(
            self.ws_reader.clone(),
            self.broadcast_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Send a close frame via the sink, then drop both halves.
        {
            let mut sink_guard = self.ws_sink.lock().await;
            if let Some(sink) = sink_guard.as_mut() {
                let _ = sink.send(Message::Close(None)).await;
            }
            *sink_guard = None;
        }
        *self.ws_reader.lock().await = None;
        *self.status.lock().await = ConnectionStatus::Disconnected;
        let _ = self.broadcast_tx.lock().unwrap().take();
        Ok(())
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channel = match &request.stream_type {
            crate::core::StreamType::Ticker => "tickers",
            crate::core::StreamType::Orderbook => {
                // OKX depth channels:
                //   books          → full 400-level snapshot+update
                //   books5         → top-5 levels
                //   books-l2-tbt   → 400-level tick-by-tick
                //   books50-l2-tbt → 50-level tick-by-tick
                match request.depth {
                    Some(5) => "books5",
                    Some(50) => "books50-l2-tbt",
                    _ => "books",
                }
            }
            crate::core::StreamType::OrderbookDelta => {
                match request.depth {
                    Some(50) => "books50-l2-tbt",
                    _ => "books-l2-tbt",
                }
            }
            crate::core::StreamType::Trade => "trades",
            crate::core::StreamType::Kline { interval } => match interval.as_str() {
                "1m" => "candle1m",
                "5m" => "candle5m",
                "15m" => "candle15m",
                "30m" => "candle30m",
                "1h" => "candle1H",
                "4h" => "candle4H",
                "1d" => "candle1D",
                _ => "candle1H",
            },
            crate::core::StreamType::MarkPrice => "mark-price",
            crate::core::StreamType::FundingRate => "funding-rate",
            crate::core::StreamType::OrderUpdate => "orders",
            crate::core::StreamType::BalanceUpdate => "account",
            crate::core::StreamType::PositionUpdate => "positions",
        };

        // For OKX the instId depends on account type.
        let account_type = request.account_type;
        let inst_id = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        let sub_msg = json!({
            "op": "subscribe",
            "args": [{
                "channel": channel,
                "instId": inst_id,
            }]
        });

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            self.subscriptions.lock().await.insert(request);
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channel = match &request.stream_type {
            crate::core::StreamType::Ticker => "tickers",
            crate::core::StreamType::Orderbook => "books",
            crate::core::StreamType::OrderbookDelta => "books",
            crate::core::StreamType::Trade => "trades",
            crate::core::StreamType::Kline { interval: _ } => "candle1H",
            crate::core::StreamType::MarkPrice => "mark-price",
            crate::core::StreamType::FundingRate => "funding-rate",
            crate::core::StreamType::OrderUpdate => "orders",
            crate::core::StreamType::BalanceUpdate => "account",
            crate::core::StreamType::PositionUpdate => "positions",
        };

        // Use the same account_type that was used in subscribe().
        let account_type = request.account_type;
        let inst_id = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        let unsub_msg = json!({
            "op": "unsubscribe",
            "args": [{
                "channel": channel,
                "instId": inst_id,
            }]
        });

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(unsub_msg.to_string()))
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            self.subscriptions.lock().await.remove(&request);
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    fn event_stream(
        &self,
    ) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send + 'static>> {
        let tx_guard = self.broadcast_tx.lock().unwrap();
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

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(guard) => *guard,
            // If the lock is held by a setter, assume we are still connected.
            Err(_) => ConnectionStatus::Connected,
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

    fn orderbook_capabilities(&self) -> OrderbookCapabilities {
        static DEPTHS: &[u32] = &[5, 50, 400];
        OrderbookCapabilities {
            ws_depths: DEPTHS,
            ws_default_depth: Some(50),
            rest_max_depth: Some(400),
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
        }
    }
}
