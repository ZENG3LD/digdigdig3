//! # Vertex WebSocket Implementation
//!
//! WebSocket connector for Vertex Protocol.
//!
//! ## Features
//! - Public and private channels
//! - EIP-712 authentication for private channels
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = VertexWebSocket::new(Some(credentials), false).await?;
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe_ticker(Symbol::new("BTC", "USDC")).await?;
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
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{mpsc, broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_millis,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;

use super::auth::VertexAuth;
use super::endpoints::VertexUrls;
use super::parser::VertexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing subscription message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    method: String,
    stream: StreamSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
}

/// Stream specification
#[derive(Debug, Clone, Serialize)]
struct StreamSpec {
    #[serde(rename = "type")]
    stream_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_id: Option<u32>,
}

/// Incoming message from Vertex
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    channel: Option<String>,
    data: Option<Value>,
    error: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VERTEX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Vertex WebSocket connector
pub struct VertexWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<VertexAuth>,
    /// URLs (mainnet/testnet)
    urls: VertexUrls,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event sender (internal - for message handler)
    event_tx: Arc<Mutex<Option<mpsc::UnboundedSender<WebSocketResult<StreamEvent>>>>>,
    /// Broadcast sender (for multiple consumers)
    broadcast_tx: Arc<broadcast::Sender<WebSocketResult<StreamEvent>>>,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Ping interval
    ping_interval: Duration,
}

impl VertexWebSocket {
    /// Create new Vertex WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            VertexUrls::TESTNET
        } else {
            VertexUrls::MAINNET
        };

        // Create auth if credentials provided
        let auth = credentials.as_ref().map(|creds| {
            let chain_id = if testnet { 421613 } else { 42161 };
            let verifying_contract = if testnet {
                "0x0000000000000000000000000000000000000000".to_string()
            } else {
                "0x0000000000000000000000000000000000000000".to_string()
            };

            VertexAuth::new(creds, chain_id, verifying_contract, None)
        }).transpose()?;

        // Create broadcast channel (capacity of 1000 events)
        let (broadcast_tx, _) = broadcast::channel(1000);

        Ok(Self {
            auth,
            urls,
            account_type: AccountType::Spot,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(Mutex::new(None)),
            broadcast_tx: Arc::new(broadcast_tx),
            ws_stream: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ping_interval: Duration::from_secs(30),
        })
    }

    /// Connect to WebSocket
    async fn connect_ws(&self, url: &str) -> ExchangeResult<WsStream> {
        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Authenticate WebSocket connection (for private channels)
    async fn authenticate(&self) -> ExchangeResult<()> {
        if let Some(auth) = &self.auth {
            let expiration_ms = (timestamp_millis() + 300000) as u64; // 5 min validity
            let (tx, signature) = auth.sign_ws_auth(expiration_ms).await?;

            let auth_msg = json!({
                "method": "authenticate",
                "tx": tx,
                "signature": signature,
            });

            let msg_json = serde_json::to_string(&auth_msg)
                .map_err(|e| ExchangeError::Parse(e.to_string()))?;

            let mut stream_guard = self.ws_stream.lock().await;
            if let Some(stream) = stream_guard.as_mut() {
                stream.send(Message::Text(msg_json)).await
                    .map_err(|e| ExchangeError::Network(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Start message handling task
    fn start_message_handler(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        status: Arc<Mutex<ConnectionStatus>>,
    ) {
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
                        if let Err(e) = Self::handle_message(&text, &event_tx).await {
                            let _ = event_tx.send(Err(e));
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        drop(stream_guard);
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Some(Err(e)) => {
                        drop(stream_guard);
                        let _ = event_tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        break;
                    }
                    None => {
                        drop(stream_guard);
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {
                        drop(stream_guard);
                    }
                }
            }
        });
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        text: &str,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
    ) -> WebSocketResult<()> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Handle different message types
        match msg.msg_type.as_deref() {
            Some("error") => {
                let error_msg = msg.error
                    .and_then(|e| e.get("message").and_then(|m| m.as_str()).map(String::from))
                    .unwrap_or_else(|| "Unknown error".to_string());
                return Err(WebSocketError::ProtocolError(error_msg));
            }
            Some("subscribed") | Some("authenticated") => {
                // Acknowledgment - ignore
                return Ok(());
            }
            Some("data") => {
                // Data message - parse and emit event
                if let Some(event) = Self::parse_data_message(&msg)? {
                    let _ = event_tx.send(Ok(event));
                }
            }
            _ => {
                // Unknown message type - ignore
            }
        }

        Ok(())
    }

    /// Parse data message to StreamEvent
    fn parse_data_message(msg: &IncomingMessage) -> WebSocketResult<Option<StreamEvent>> {
        let channel = msg.channel.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing channel".to_string()))?;

        let data = msg.data.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing data".to_string()))?;

        // Match by channel to determine event type
        match channel.as_str() {
            "book_depth" => {
                let orderbook = VertexParser::parse_ws_orderbook(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderbookSnapshot(orderbook)))
            }
            "trades" => {
                let trade = VertexParser::parse_ws_trade(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(trade)))
            }
            "book_ticker" => {
                let ticker = VertexParser::parse_ws_ticker(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            "fill" => {
                let fill = VertexParser::parse_ws_fill(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(fill)))
            }
            "order" => {
                let order_update = VertexParser::parse_ws_order_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderUpdate(order_update)))
            }
            "position" => {
                let position_update = VertexParser::parse_ws_position_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::PositionUpdate(position_update)))
            }
            _ => {
                // Unknown channel - ignore
                Ok(None)
            }
        }
    }

    /// Start ping task
    fn start_ping_task(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        ping_interval: Duration,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1000)).await;

                let last = *last_ping.lock().await;

                if last.elapsed() >= ping_interval {
                    let mut stream_guard = ws_stream.lock().await;
                    if let Some(stream) = stream_guard.as_mut() {
                        let ping = json!({
                            "method": "ping",
                        });

                        let msg_json = serde_json::to_string(&ping).expect("JSON serialization should never fail for valid struct");
                        if stream.send(Message::Text(msg_json)).await.is_ok() {
                            *last_ping.lock().await = Instant::now();
                        }
                    }
                }
            }
        });
    }

    /// Build subscription message
    fn build_subscribe_message(
        request: &SubscriptionRequest,
        _account_type: AccountType,
    ) -> WebSocketResult<SubscribeMessage> {
        let stream_type = match &request.stream_type {
            StreamType::Ticker => "book_ticker",
            StreamType::Trade => "trades",
            StreamType::Orderbook | StreamType::OrderbookDelta => "book_depth",
            StreamType::OrderUpdate => "order",
            StreamType::PositionUpdate => "position",
            _ => {
                return Err(WebSocketError::ProtocolError(
                    format!("Unsupported stream type: {:?}", request.stream_type)
                ));
            }
        };

        Ok(SubscribeMessage {
            method: "subscribe".to_string(),
            stream: StreamSpec {
                stream_type: stream_type.to_string(),
                product_id: None, // Will need product_id lookup in real implementation
            },
            id: Some(timestamp_millis()),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for VertexWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Determine URL based on whether we need private channels
        let url = if self.auth.is_some() {
            self.urls.websocket
        } else {
            self.urls.subscribe
        };

        // Connect WebSocket
        let ws_stream = self.connect_ws(url).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        *self.ws_stream.lock().await = Some(ws_stream);

        // Authenticate if we have credentials
        if self.auth.is_some() {
            self.authenticate().await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
        }

        *self.status.lock().await = ConnectionStatus::Connected;

        // Create event channel
        let (tx, mut rx) = mpsc::unbounded_channel();
        *self.event_tx.lock().await = Some(tx.clone());

        // Start message handler
        Self::start_message_handler(
            self.ws_stream.clone(),
            tx,
            self.status.clone(),
        );

        // Start forwarder task (mpsc -> broadcast)
        let broadcast_tx = self.broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Forward to broadcast channel (ignore if no receivers)
                let _ = broadcast_tx.send(event);
            }
        });

        // Start ping task
        Self::start_ping_task(
            self.ws_stream.clone(),
            self.ping_interval,
            self.last_ping.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_stream.lock().await = None;
        *self.event_tx.lock().await = None;
        self.subscriptions.lock().await.clear();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Use try_lock to avoid blocking
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let msg = Self::build_subscribe_message(&request, self.account_type)?;

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(stream_guard);

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let mut msg = Self::build_subscribe_message(&request, self.account_type)?;
        msg.method = "unsubscribe".to_string();

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(stream_guard);

        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        // Subscribe to broadcast channel
        let rx = self.broadcast_tx.subscribe();

        // Convert broadcast receiver to stream
        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
            match result {
                Ok(event) => Some(event),
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                    // Consumer was too slow, some events were dropped
                    // Return an error event
                    Some(Err(WebSocketError::ConnectionError("Event stream lagged behind".to_string())))
                }
            }
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        // Use try_lock to avoid blocking
        match self.subscriptions.try_lock() {
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}
