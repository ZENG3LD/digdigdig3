//! # Paradex WebSocket Implementation
//!
//! WebSocket connector for Paradex (JSON-RPC 2.0 protocol).
//!
//! ## Features
//! - Public and private channels
//! - JWT authentication (once per connection)
//! - Automatic ping/pong heartbeat (55s ping interval)
//! - Subscription management
//! - Message parsing to StreamEvent
//! - Broadcast channel for multiple consumers
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = ParadexWebSocket::new(Some(credentials), false).await?;
//! ws.connect().await?;
//! ws.subscribe_ticker(Symbol::new("BTC", "USD")).await?;
//!
//! let mut stream = ws.event_stream();
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
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;

use super::auth::ParadexAuth;
use super::endpoints::{ParadexUrls, format_symbol};
use super::parser::ParadexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES (JSON-RPC 2.0)
// ═══════════════════════════════════════════════════════════════════════════════

/// JSON-RPC 2.0 outgoing message
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

impl JsonRpcRequest {
    fn new(method: &str, params: Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }
}

/// JSON-RPC 2.0 incoming message
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct JsonRpcMessage {
    jsonrpc: Option<String>,
    method: Option<String>,
    params: Option<Value>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct JsonRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARADEX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Paradex WebSocket connector
pub struct ParadexWebSocket {
    /// Authentication (JWT-based)
    auth: Option<Arc<ParadexAuth>>,
    /// URLs (mainnet/testnet)
    urls: ParadexUrls,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender — behind StdMutex so event_stream() can subscribe
    /// without contending with the async message loop.
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Ping interval (55 seconds per Paradex docs)
    ping_interval: Duration,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
    /// Message ID counter
    msg_id: Arc<Mutex<u64>>,
    /// Authenticated flag
    authenticated: Arc<Mutex<bool>>,
}

impl ParadexWebSocket {
    /// Create new Paradex WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            ParadexUrls::TESTNET
        } else {
            ParadexUrls::MAINNET
        };

        let auth = credentials
            .map(|c| ParadexAuth::new(&c))
            .transpose()?
            .map(Arc::new);

        Ok(Self {
            auth,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_stream: Arc::new(Mutex::new(None)),
            ping_interval: Duration::from_secs(55),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
            msg_id: Arc::new(Mutex::new(1)),
            authenticated: Arc::new(Mutex::new(false)),
        })
    }

    /// Get next message ID
    async fn next_msg_id(&self) -> u64 {
        let mut id = self.msg_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    /// Send JSON-RPC message
    async fn send_json_rpc(&self, method: &str, params: Value) -> WebSocketResult<()> {
        let mut stream_lock = self.ws_stream.lock().await;
        if let Some(stream) = stream_lock.as_mut() {
            let id = self.next_msg_id().await;
            let request = JsonRpcRequest::new(method, params, id);
            let msg = serde_json::to_string(&request)
                .map_err(|e| WebSocketError::Parse(format!("Failed to serialize message: {}", e)))?;

            stream.send(Message::Text(msg)).await
                .map_err(|e| WebSocketError::SendError(e.to_string()))?;

            Ok(())
        } else {
            Err(WebSocketError::NotConnected)
        }
    }

    /// Authenticate with JWT token
    async fn authenticate(&self) -> WebSocketResult<()> {
        if let Some(auth) = &self.auth {
            let jwt_token = auth.get_jwt_token().await
                .map_err(|e| WebSocketError::Auth(e.to_string()))?;

            let params = json!({
                "jwt_token": jwt_token
            });

            self.send_json_rpc("authenticate", params).await?;

            // Mark as authenticated
            let mut auth_flag = self.authenticated.lock().await;
            *auth_flag = true;

            Ok(())
        } else {
            // No auth needed for public channels
            Ok(())
        }
    }

    /// Subscribe to a channel
    async fn subscribe_channel(&self, channel: &str) -> WebSocketResult<()> {
        let params = json!({
            "channel": channel
        });

        self.send_json_rpc("subscribe", params).await
    }

    /// Unsubscribe from a channel
    async fn unsubscribe_channel(&self, channel: &str) -> WebSocketResult<()> {
        let params = json!({
            "channel": channel
        });

        self.send_json_rpc("unsubscribe", params).await
    }

    /// Handle incoming message
    async fn handle_message(&self, text: &str) -> WebSocketResult<()> {
        // Resolve the subscribed ticker symbol so the parser can filter
        // `markets_summary` events that belong to a different market.
        let target_symbol: Option<String> = {
            let subs = self.subscriptions.lock().await;
            subs.iter()
                .find(|req| req.stream_type == crate::core::StreamType::Ticker)
                .map(|req| super::endpoints::format_symbol(
                    &req.symbol.base,
                    &req.symbol.quote,
                    crate::core::AccountType::FuturesCross,
                ))
        };

        // Try to parse as StreamEvent
        match ParadexParser::parse_ws_message(text, target_symbol.as_deref()) {
            Ok(event) => {
                // Broadcast to all consumers
                let tx_guard = self.broadcast_tx.lock().unwrap();
                if let Some(ref tx) = *tx_guard {
                    let _ = tx.send(Ok(event));
                }
                Ok(())
            }
            Err(_) => {
                // Filtered event (wrong market) or control message — silently ignore.
                Ok(())
            }
        }
    }

    /// Message loop (processes incoming messages)
    async fn message_loop(&self) {
        loop {
            // Check if still connected
            {
                let status = self.status.lock().await;
                if matches!(*status, ConnectionStatus::Disconnected) {
                    break;
                }
            }

            // Get message from stream
            let msg_opt = {
                let mut stream_lock = self.ws_stream.lock().await;
                if let Some(stream) = stream_lock.as_mut() {
                    stream.next().await
                } else {
                    break;
                }
            };

            match msg_opt {
                Some(Ok(Message::Text(text))) => {
                    if let Err(e) = self.handle_message(&text).await {
                        eprintln!("Error handling message: {}", e);
                    }
                }
                Some(Ok(Message::Ping(payload))) => {
                    // Respond with pong
                    let mut stream_lock = self.ws_stream.lock().await;
                    if let Some(stream) = stream_lock.as_mut() {
                        let _ = stream.send(Message::Pong(payload)).await;
                    }
                }
                Some(Ok(Message::Pong(_))) => {
                    // Record RTT for the WS-level ping sent by ping_loop
                    let rtt = self.last_ping.lock().await.elapsed().as_millis() as u64;
                    *self.ws_ping_rtt_ms.lock().await = rtt;
                }
                Some(Ok(Message::Close(_))) => {
                    // Connection closed
                    let mut status = self.status.lock().await;
                    *status = ConnectionStatus::Disconnected;
                    break;
                }
                Some(Err(e)) => {
                    eprintln!("WebSocket error: {}", e);
                    let mut status = self.status.lock().await;
                    *status = ConnectionStatus::Disconnected;
                    break;
                }
                None => {
                    // Stream ended
                    let mut status = self.status.lock().await;
                    *status = ConnectionStatus::Disconnected;
                    break;
                }
                _ => {}
            }
        }
    }

    /// Ping loop (sends ping every 55 seconds as per Paradex spec)
    async fn ping_loop(&self) {
        loop {
            sleep(self.ping_interval).await;

            // Check if still connected
            {
                let status = self.status.lock().await;
                if matches!(*status, ConnectionStatus::Disconnected) {
                    break;
                }
            }

            // Record send time BEFORE sending so RTT measurement is accurate
            *self.last_ping.lock().await = Instant::now();

            // Send ping (Paradex uses WebSocket-level ping, not JSON-RPC)
            let result = {
                let mut stream_lock = self.ws_stream.lock().await;
                if let Some(stream) = stream_lock.as_mut() {
                    stream.send(Message::Ping(vec![])).await
                } else {
                    break;
                }
            };

            if result.is_err() {
                eprintln!("Failed to send ping");
                let mut status = self.status.lock().await;
                *status = ConnectionStatus::Disconnected;
                break;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for ParadexWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Update status
        {
            let mut status = self.status.lock().await;
            *status = ConnectionStatus::Connecting;
        }

        // Connect to WebSocket
        let ws_url = self.urls.ws_url();
        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Store stream
        {
            let mut stream_lock = self.ws_stream.lock().await;
            *stream_lock = Some(ws_stream);
        }

        // Update status
        {
            let mut status = self.status.lock().await;
            *status = ConnectionStatus::Connected;
        }

        // Authenticate if we have credentials
        if self.auth.is_some() {
            self.authenticate().await?;
        }

        // Create broadcast channel and store sender
        let (tx, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(tx);

        // Start message loop
        let self_clone = Self {
            auth: self.auth.clone(),
            urls: self.urls.clone(),
            status: self.status.clone(),
            subscriptions: self.subscriptions.clone(),
            broadcast_tx: self.broadcast_tx.clone(),
            ws_stream: self.ws_stream.clone(),
            ping_interval: self.ping_interval,
            last_ping: self.last_ping.clone(),
            ws_ping_rtt_ms: self.ws_ping_rtt_ms.clone(),
            msg_id: self.msg_id.clone(),
            authenticated: self.authenticated.clone(),
        };
        tokio::spawn(async move {
            self_clone.message_loop().await;
            // Drop the broadcast sender so all BroadcastStream receivers get None
            let _ = self_clone.broadcast_tx.lock().unwrap().take();
        });

        // Start ping loop
        let self_clone2 = Self {
            auth: self.auth.clone(),
            urls: self.urls.clone(),
            status: self.status.clone(),
            subscriptions: self.subscriptions.clone(),
            broadcast_tx: self.broadcast_tx.clone(),
            ws_stream: self.ws_stream.clone(),
            ping_interval: self.ping_interval,
            last_ping: self.last_ping.clone(),
            ws_ping_rtt_ms: self.ws_ping_rtt_ms.clone(),
            msg_id: self.msg_id.clone(),
            authenticated: self.authenticated.clone(),
        };
        tokio::spawn(async move {
            self_clone2.ping_loop().await;
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Update status
        {
            let mut status = self.status.lock().await;
            *status = ConnectionStatus::Disconnected;
        }

        // Close WebSocket
        {
            let mut stream_lock = self.ws_stream.lock().await;
            if let Some(stream) = stream_lock.as_mut() {
                let _ = stream.close(None).await;
            }
            *stream_lock = None;
        }

        let _ = self.broadcast_tx.lock().unwrap().take();
        Ok(())
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Build channel name based on request
        let channel = match &request.stream_type {
            StreamType::Ticker => {
                "markets_summary".to_string() // Global ticker channel
            }
            StreamType::Orderbook => {
                let symbol_str = format_symbol(&request.symbol.base, &request.symbol.quote, AccountType::FuturesCross);
                format!("order_book.{}", symbol_str)
            }
            StreamType::Trade => {
                let symbol_str = format_symbol(&request.symbol.base, &request.symbol.quote, AccountType::FuturesCross);
                format!("trades.{}", symbol_str)
            }
            StreamType::OrderUpdate => "orders".to_string(),
            StreamType::BalanceUpdate => "account".to_string(),
            StreamType::PositionUpdate => "positions".to_string(),
            _ => return Err(WebSocketError::UnsupportedOperation(format!("Stream type {:?} not supported", request.stream_type))),
        };

        // Subscribe to channel
        self.subscribe_channel(&channel).await?;

        // Add to subscriptions
        {
            let mut subs = self.subscriptions.lock().await;
            subs.insert(request);
        }

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Build channel name (same logic as subscribe)
        let channel = match &request.stream_type {
            StreamType::Ticker => "markets_summary".to_string(),
            StreamType::Orderbook => {
                let symbol_str = format_symbol(&request.symbol.base, &request.symbol.quote, AccountType::FuturesCross);
                format!("order_book.{}", symbol_str)
            }
            StreamType::Trade => {
                let symbol_str = format_symbol(&request.symbol.base, &request.symbol.quote, AccountType::FuturesCross);
                format!("trades.{}", symbol_str)
            }
            StreamType::OrderUpdate => "orders".to_string(),
            StreamType::BalanceUpdate => "account".to_string(),
            StreamType::PositionUpdate => "positions".to_string(),
            _ => return Err(WebSocketError::UnsupportedOperation(format!("Stream type {:?} not supported", request.stream_type))),
        };

        // Unsubscribe from channel
        self.unsubscribe_channel(&channel).await?;

        // Remove from subscriptions
        {
            let mut subs = self.subscriptions.lock().await;
            subs.remove(&request);
        }

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
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
        // Use try_lock to avoid blocking in sync context
        if let Ok(status) = self.status.try_lock() {
            *status
        } else {
            ConnectionStatus::Disconnected
        }
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        if let Ok(subs) = self.subscriptions.try_lock() {
            subs.iter().cloned().collect()
        } else {
            vec![]
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws = ParadexWebSocket::new(None, true).await;
        assert!(ws.is_ok());
    }

    #[tokio::test]
    async fn test_msg_id_increment() {
        let ws = ParadexWebSocket::new(None, true).await.unwrap();
        let id1 = ws.next_msg_id().await;
        let id2 = ws.next_msg_id().await;
        assert_eq!(id2, id1 + 1);
    }
}
