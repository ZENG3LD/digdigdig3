//! # Crypto.com WebSocket Client
//!
//! WebSocket implementation for Crypto.com Exchange API v1.
//!
//! ## Features
//! - Public and private channels
//! - Automatic heartbeat handling
//! - Broadcast channel pattern for multiple consumers
//! - Ticker, orderbook, trade subscriptions
//! - Message parsing using CryptoComParser
//!
//! ## Critical Notes
//! - ALWAYS wait 1 second after connection before sending requests
//! - Respond to heartbeats to maintain connection
//! - Separate connections for user data and market data
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = CryptoComWebSocket::new(Some(auth), true);
//! ws.connect().await?;
//! ws.subscribe_ticker("BTC_USDT").await?;
//!
//! let mut stream = ws.event_stream();
//! while let Some(event) = stream.recv().await {
//!     println!("Event: {:?}", event);
//! }
//! ```

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, broadcast};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    ExchangeResult, ExchangeError, timestamp_millis,
    AccountType, ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;
use super::auth::CryptoComAuth;
use super::endpoints::{InstrumentType, format_symbol as fmt_symbol};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// ═══════════════════════════════════════════════════════════════════════════════
// MESSAGE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing WebSocket message
#[derive(Debug, Clone, Serialize)]
struct OutgoingMessage {
    id: i64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<SubscribeParams>,
}

#[derive(Debug, Clone, Serialize)]
struct SubscribeParams {
    channels: Vec<String>,
}

/// Incoming WebSocket message
#[derive(Debug, Clone, Deserialize)]
struct IncomingMessage {
    #[serde(default)]
    #[allow(dead_code)]
    id: Option<i64>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    code: Option<i64>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    result: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET EVENT
// ═══════════════════════════════════════════════════════════════════════════════

/// WebSocket event (simplified for testing)
#[derive(Debug, Clone)]
pub enum WsEvent {
    Ticker(Value),
    OrderBook(Value),
    Trade(Value),
    UserOrder(Value),
    UserBalance(Value),
    Heartbeat,
    SubscriptionSuccess(String),
    Error(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CLIENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Crypto.com WebSocket client
pub struct CryptoComWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<CryptoComAuth>,
    /// Is this a user stream (private) or market stream (public)
    is_user_stream: bool,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Broadcast sender for custom WsEvent (legacy interface)
    broadcast_tx: broadcast::Sender<WsEvent>,
    /// Broadcast sender for standard StreamEvent (trait interface, dropped on disconnect)
    stream_broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Active subscriptions (channel strings, legacy)
    subscriptions: Arc<Mutex<HashSet<String>>>,
    /// Active subscriptions (standard SubscriptionRequest, for trait)
    trait_subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Message ID counter
    message_id: Arc<Mutex<i64>>,
    /// Connection status
    is_connected: Arc<Mutex<bool>>,
    /// Current account type
    account_type: AccountType,
    /// Last time a WS-level ping was sent (for RTT measurement)
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl CryptoComWebSocket {
    /// Create new WebSocket client
    pub fn new(auth: Option<CryptoComAuth>, is_user_stream: bool) -> Self {
        let (tx, _) = broadcast::channel(1000);

        Self {
            auth,
            is_user_stream,
            ws_stream: Arc::new(Mutex::new(None)),
            broadcast_tx: tx,
            stream_broadcast_tx: Arc::new(StdMutex::new(None)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            trait_subscriptions: Arc::new(Mutex::new(HashSet::new())),
            message_id: Arc::new(Mutex::new(1)),
            is_connected: Arc::new(Mutex::new(false)),
            account_type: AccountType::Spot,
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        }
    }

    /// Get WebSocket URL
    fn get_ws_url(&self) -> &'static str {
        if self.is_user_stream {
            "wss://stream.crypto.com/exchange/v1/user"
        } else {
            "wss://stream.crypto.com/exchange/v1/market"
        }
    }

    /// Get next message ID
    async fn next_id(&self) -> i64 {
        let mut id = self.message_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    /// Connect to WebSocket
    pub async fn connect(&mut self) -> ExchangeResult<()> {
        let url = self.get_ws_url();

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        // CRITICAL: Wait 1 second before sending requests
        sleep(Duration::from_secs(1)).await;

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.is_connected.lock().await = true;

        // Create broadcast channel and store
        let (stream_sender, _) = broadcast::channel(1000);
        *self.stream_broadcast_tx.lock().unwrap() = Some(stream_sender);

        // Authenticate if user stream
        if self.is_user_stream {
            self.authenticate().await?;
        }

        // Start message handler
        self.start_message_handler();

        // Start heartbeat handler
        self.start_heartbeat_handler();

        // Start WS-level ping for RTT measurement
        self.start_ws_ping_task();

        Ok(())
    }

    /// Authenticate (for user streams)
    async fn authenticate(&self) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("User stream requires authentication".to_string()))?;

        let id = self.next_id().await;
        let nonce = timestamp_millis();

        let signature = auth.sign_ws_auth(id, nonce as i64);

        let msg = OutgoingMessage {
            id,
            method: "public/auth".to_string(),
            api_key: Some(auth.api_key().to_string()),
            sig: Some(signature),
            nonce: Some(nonce as i64),
            params: None,
        };

        self.send_message(&msg).await?;

        // Wait a bit for auth response
        sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    /// Send message to WebSocket
    async fn send_message(&self, msg: &OutgoingMessage) -> ExchangeResult<()> {
        let msg_json = serde_json::to_string(msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize message: {}", e)))?;

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard.as_mut()
            .ok_or_else(|| ExchangeError::Network("Not connected".to_string()))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    /// Start message handler task
    fn start_message_handler(&self) {
        let ws_stream = self.ws_stream.clone();
        let broadcast_tx = self.broadcast_tx.clone();
        let stream_broadcast_tx = self.stream_broadcast_tx.clone();
        let is_connected = self.is_connected.clone();
        let last_ping = self.last_ping.clone();
        let ws_ping_rtt_ms = self.ws_ping_rtt_ms.clone();

        tokio::spawn(async move {
            loop {
                let mut stream_guard = ws_stream.lock().await;
                let stream = match stream_guard.as_mut() {
                    Some(s) => s,
                    None => {
                        drop(stream_guard);
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                };

                match stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        drop(stream_guard);
                        if let Some(event) = Self::parse_message(&text) {
                            // Forward to standard StreamEvent broadcast
                            if let Some(stream_event) = Self::ws_event_to_stream_event(&event) {
                                if let Some(tx) = stream_broadcast_tx.lock().unwrap().as_ref() {
                                    let _ = tx.send(Ok(stream_event));
                                }
                            }
                            // Forward to legacy WsEvent broadcast
                            let _ = broadcast_tx.send(event);
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        drop(stream_guard);
                        // Record RTT for the WS-level ping sent by start_ws_ping_task
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Some(Ok(Message::Close(_))) => {
                        drop(stream_guard);
                        *is_connected.lock().await = false;
                        break;
                    }
                    Some(Err(e)) => {
                        drop(stream_guard);
                        if let Some(tx) = stream_broadcast_tx.lock().unwrap().as_ref() {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
                        let _ = broadcast_tx.send(WsEvent::Error(e.to_string()));
                        break;
                    }
                    None => {
                        drop(stream_guard);
                        *is_connected.lock().await = false;
                        break;
                    }
                    _ => {
                        drop(stream_guard);
                    }
                }
            }
            // Stream ended — drop broadcast sender
            let _ = stream_broadcast_tx.lock().unwrap().take();
        });
    }

    /// Convert custom WsEvent to standard StreamEvent
    fn ws_event_to_stream_event(event: &WsEvent) -> Option<StreamEvent> {
        match event {
            WsEvent::Ticker(data) => {
                let ticker = super::parser::CryptoComParser::parse_ws_ticker(data).ok()?;
                Some(StreamEvent::Ticker(ticker))
            }
            WsEvent::OrderBook(data) => {
                // Parse orderbook delta from raw data
                // Crypto.com sends incremental book updates with bids/asks arrays
                let bids = data.get("bids")
                    .and_then(|b| b.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|entry| {
                            let price = entry.get(0)?.as_str()?.parse::<f64>().ok()?;
                            let qty = entry.get(1)?.as_str()?.parse::<f64>().ok()?;
                            Some((price, qty))
                        }).collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let asks = data.get("asks")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|entry| {
                            let price = entry.get(0)?.as_str()?.parse::<f64>().ok()?;
                            let qty = entry.get(1)?.as_str()?.parse::<f64>().ok()?;
                            Some((price, qty))
                        }).collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let timestamp = data.get("t")
                    .and_then(|t| t.as_i64())
                    .unwrap_or(0);
                Some(StreamEvent::OrderbookDelta { bids, asks, timestamp })
            }
            WsEvent::Trade(data) => {
                let trade = super::parser::CryptoComParser::parse_ws_trade(data).ok()?;
                Some(StreamEvent::Trade(trade))
            }
            WsEvent::UserOrder(_) | WsEvent::UserBalance(_) => {
                // Private stream events - not parsed to StreamEvent yet
                None
            }
            WsEvent::Heartbeat | WsEvent::SubscriptionSuccess(_) | WsEvent::Error(_) => None,
        }
    }

    /// Parse incoming message
    fn parse_message(text: &str) -> Option<WsEvent> {
        // Try to parse as generic message first
        let msg: IncomingMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to parse message: {} - {}", e, text);
                return None;
            }
        };

        // Handle different message types
        match msg.method.as_deref() {
            Some("public/heartbeat") => Some(WsEvent::Heartbeat),
            Some("subscribe") => {
                // Both subscription confirmations AND data pushes arrive with method "subscribe".
                // Data pushes have a "data" array inside "result"; confirmations do not.
                if let Some(ref result) = msg.result {
                    if result.get("data").is_some() {
                        // This is a data push (ticker/book/trade update)
                        return Self::parse_data_message(result);
                    }
                    // Subscription confirmation (code == 0, no data field)
                    if msg.code == Some(0) {
                        if let Some(subscription) = result.get("subscription").and_then(|s| s.as_str()) {
                            return Some(WsEvent::SubscriptionSuccess(subscription.to_string()));
                        }
                    }
                }
                None
            }
            Some("public/auth") => {
                // Auth response
                if msg.code != Some(0) {
                    let error_msg = msg.message.unwrap_or_else(|| "Authentication failed".to_string());
                    Some(WsEvent::Error(error_msg))
                } else {
                    None // Auth success - no event needed
                }
            }
            None => {
                // No method field - this might be a data push
                if let Some(result) = msg.result {
                    return Self::parse_data_message(&result);
                }
                // Debug: print unknown message
                eprintln!("Unknown message format (no method, no result): {}", text);
                None
            }
            Some(method) => {
                // Unknown method with result might be data
                if let Some(result) = msg.result {
                    Self::parse_data_message(&result)
                } else {
                    eprintln!("Unknown method '{}': {}", method, text);
                    None
                }
            }
        }
    }

    /// Parse data message
    ///
    /// Crypto.com data pushes wrap the actual payload in a `data` array inside `result`.
    /// We extract the first element from that array so downstream parsers receive
    /// the individual data object (with fields like `i`, `b`, `k`, etc.) directly.
    fn parse_data_message(result: &Value) -> Option<WsEvent> {
        let channel = result.get("channel")?.as_str()?;

        // Extract the first element from the "data" array.
        // Fall back to the full result if "data" is missing (shouldn't happen for real pushes).
        let data = result
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .cloned()
            .unwrap_or_else(|| result.clone());

        match channel {
            "ticker" => Some(WsEvent::Ticker(data)),
            "book" => Some(WsEvent::OrderBook(data)),
            "trade" => Some(WsEvent::Trade(data)),
            "user.order" => Some(WsEvent::UserOrder(data)),
            "user.balance" => Some(WsEvent::UserBalance(data)),
            _ => None,
        }
    }

    /// Start WS-level ping task for RTT measurement (every 5 seconds).
    ///
    /// CryptoCom uses JSON-level heartbeats for keepalive; this task sends
    /// WS-level `Message::Ping` frames so the server responds with `Message::Pong`,
    /// allowing RTT measurement via `ping_rtt_handle()`.
    fn start_ws_ping_task(&self) {
        let ws_stream = self.ws_stream.clone();
        let last_ping = self.last_ping.clone();
        let is_connected = self.is_connected.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            // Skip immediate first tick
            interval.tick().await;

            loop {
                interval.tick().await;

                if !*is_connected.lock().await {
                    break;
                }

                let mut stream_guard = ws_stream.lock().await;
                if let Some(stream) = stream_guard.as_mut() {
                    *last_ping.lock().await = Instant::now();
                    if stream.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Start heartbeat handler task
    fn start_heartbeat_handler(&self) {
        let ws_stream = self.ws_stream.clone();
        let message_id = self.message_id.clone();
        let is_connected = self.is_connected.clone();
        let mut rx = self.broadcast_tx.subscribe();

        tokio::spawn(async move {
            loop {
                // Check if still connected
                if !*is_connected.lock().await {
                    break;
                }

                // Wait for heartbeat or timeout
                tokio::select! {
                    _ = sleep(Duration::from_secs(30)) => {
                        // Timeout - connection might be stale
                    }
                    event = rx.recv() => {
                        if let Ok(WsEvent::Heartbeat) = event {
                            // Respond to heartbeat
                            let id = {
                                let mut mid = message_id.lock().await;
                                let current = *mid;
                                *mid += 1;
                                current
                            };

                            let pong = OutgoingMessage {
                                id,
                                method: "public/respond-heartbeat".to_string(),
                                api_key: None,
                                sig: None,
                                nonce: None,
                                params: None,
                            };

                            if let Ok(msg_json) = serde_json::to_string(&pong) {
                                let mut stream_guard = ws_stream.lock().await;
                                if let Some(stream) = stream_guard.as_mut() {
                                    let _ = stream.send(Message::Text(msg_json)).await;
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// Subscribe to ticker channel
    pub async fn subscribe_ticker(&mut self, instrument_name: &str) -> ExchangeResult<()> {
        let channel = format!("ticker.{}", instrument_name);
        self.subscribe_channels(vec![channel]).await
    }

    /// Subscribe to orderbook channel
    pub async fn subscribe_orderbook(&mut self, instrument_name: &str, depth: u32) -> ExchangeResult<()> {
        let channel = format!("book.{}.{}", instrument_name, depth);
        self.subscribe_channels(vec![channel]).await
    }

    /// Subscribe to trade channel
    pub async fn subscribe_trades(&mut self, instrument_name: &str) -> ExchangeResult<()> {
        let channel = format!("trade.{}", instrument_name);
        self.subscribe_channels(vec![channel]).await
    }

    /// Subscribe to user order updates
    pub async fn subscribe_user_orders(&mut self, instrument_name: &str) -> ExchangeResult<()> {
        if !self.is_user_stream {
            return Err(ExchangeError::UnsupportedOperation(
                "User orders require user stream".to_string()
            ));
        }
        let channel = format!("user.order.{}", instrument_name);
        self.subscribe_channels(vec![channel]).await
    }

    /// Subscribe to user balance updates
    pub async fn subscribe_user_balance(&mut self) -> ExchangeResult<()> {
        if !self.is_user_stream {
            return Err(ExchangeError::UnsupportedOperation(
                "User balance requires user stream".to_string()
            ));
        }
        self.subscribe_channels(vec!["user.balance".to_string()]).await
    }

    /// Subscribe to channels
    async fn subscribe_channels(&mut self, channels: Vec<String>) -> ExchangeResult<()> {
        let id = self.next_id().await;
        let nonce = timestamp_millis();

        let msg = OutgoingMessage {
            id,
            method: "subscribe".to_string(),
            api_key: None,
            sig: None,
            nonce: Some(nonce as i64),
            params: Some(SubscribeParams { channels: channels.clone() }),
        };

        self.send_message(&msg).await?;

        // Add to subscriptions
        let mut subs = self.subscriptions.lock().await;
        for channel in channels {
            subs.insert(channel);
        }

        Ok(())
    }

    /// Unsubscribe from channels
    async fn unsubscribe_channels(&self, channels: Vec<String>) -> ExchangeResult<()> {
        let id = self.next_id().await;
        let nonce = timestamp_millis();

        let msg = OutgoingMessage {
            id,
            method: "unsubscribe".to_string(),
            api_key: None,
            sig: None,
            nonce: Some(nonce as i64),
            params: Some(SubscribeParams { channels: channels.clone() }),
        };

        self.send_message(&msg).await?;

        // Remove from subscriptions
        let mut subs = self.subscriptions.lock().await;
        for channel in &channels {
            subs.remove(channel);
        }

        Ok(())
    }

    /// Build channel name from SubscriptionRequest
    fn build_channel(request: &SubscriptionRequest, account_type: AccountType) -> Vec<String> {
        let instrument_type = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => InstrumentType::Perpetual,
            _ => InstrumentType::Spot,
        };
        let symbol_str = fmt_symbol(&request.symbol.base, &request.symbol.quote, instrument_type);

        match &request.stream_type {
            StreamType::Ticker => vec![format!("ticker.{}", symbol_str)],
            StreamType::Trade => vec![format!("trade.{}", symbol_str)],
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                vec![format!("book.{}.10", symbol_str)]
            }
            StreamType::Kline { interval } => vec![format!("candlestick.{}.{}", interval, symbol_str)],
            StreamType::OrderUpdate => vec![format!("user.order.{}", symbol_str)],
            StreamType::BalanceUpdate => vec!["user.balance".to_string()],
            _ => vec![],
        }
    }

    /// Get event stream (broadcast channel receiver)
    pub fn event_stream(&self) -> broadcast::Receiver<WsEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.lock().await
    }

    /// Disconnect
    pub async fn disconnect(&mut self) -> ExchangeResult<()> {
        *self.is_connected.lock().await = false;
        *self.ws_stream.lock().await = None;
        let _ = self.stream_broadcast_tx.lock().unwrap().take();
        self.subscriptions.lock().await.clear();
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for CryptoComWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        self.account_type = account_type;

        // Determine stream type based on account type
        // User stream requires private WS; otherwise use market WS
        let url = self.get_ws_url();

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| WebSocketError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        // CRITICAL: Wait 1 second before sending requests
        sleep(Duration::from_secs(1)).await;

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.is_connected.lock().await = true;

        // Create broadcast channel and store
        let (stream_sender, _) = broadcast::channel(1000);
        *self.stream_broadcast_tx.lock().unwrap() = Some(stream_sender);

        // Authenticate if user stream
        if self.is_user_stream {
            self.authenticate().await
                .map_err(|e| WebSocketError::Auth(e.to_string()))?;
        }

        // Start message handler
        self.start_message_handler();

        // Start heartbeat handler
        self.start_heartbeat_handler();

        // Start WS-level ping for RTT measurement
        self.start_ws_ping_task();

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.is_connected.lock().await = false;
        *self.ws_stream.lock().await = None;
        let _ = self.stream_broadcast_tx.lock().unwrap().take();
        self.subscriptions.lock().await.clear();
        self.trait_subscriptions.lock().await.clear();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Use try_lock to avoid blocking in a sync context
        match self.is_connected.try_lock() {
            Ok(connected) => {
                if *connected {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                }
            }
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channels = Self::build_channel(&request, self.account_type);
        if channels.is_empty() {
            return Err(WebSocketError::UnsupportedOperation(
                format!("Unsupported stream type: {:?}", request.stream_type),
            ));
        }

        self.subscribe_channels(channels).await
            .map_err(|e| WebSocketError::Subscription(e.to_string()))?;

        self.trait_subscriptions.lock().await.insert(request);
        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channels = Self::build_channel(&request, self.account_type);
        if channels.is_empty() {
            return Err(WebSocketError::UnsupportedOperation(
                format!("Unsupported stream type: {:?}", request.stream_type),
            ));
        }

        self.unsubscribe_channels(channels).await
            .map_err(|e| WebSocketError::Subscription(e.to_string()))?;

        self.trait_subscriptions.lock().await.remove(&request);
        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.stream_broadcast_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1);

        Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
                match result {
                    Ok(event) => Some(event),
                    Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                        Some(Err(WebSocketError::ReceiveError(
                            "Event stream lagged behind".to_string(),
                        )))
                    }
                }
            }),
        )
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.trait_subscriptions.try_lock() {
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS (kept for backward compatibility)
// ═══════════════════════════════════════════════════════════════════════════════

/// Wait 1 second after WebSocket connection (CRITICAL for Crypto.com)
async fn _wait_after_connection() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

/// Build authentication message for WebSocket
fn _build_auth_message(auth: &CryptoComAuth, id: i64, nonce: i64) -> serde_json::Value {
    let signature = auth.sign_ws_auth(id, nonce);

    serde_json::json!({
        "id": id,
        "method": "public/auth",
        "api_key": auth.api_key(),
        "sig": signature,
        "nonce": nonce
    })
}

/// Build heartbeat response message
fn _build_heartbeat_response(id: i64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "method": "public/respond-heartbeat"
    })
}

/// Build subscribe message
fn _build_subscribe_message(id: i64, channels: Vec<String>, nonce: i64) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "method": "subscribe",
        "params": {
            "channels": channels
        },
        "nonce": nonce
    })
}

