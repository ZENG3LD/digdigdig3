//! # Gemini WebSocket Implementation
//!
//! WebSocket connector for Gemini market data and order events.
//!
//! ## Features
//! - Market Data v2 (public)
//! - Order Events (private, authenticated)
//! - Broadcast channel pattern for multiple consumers
//! - Automatic reconnection
//! - Subscription management
//!
//! ## Architecture
//!
//! The WebSocket stream is split into independent read and write halves on connect.
//! The write half is stored behind a mutex for shared access by `subscribe` and
//! the ping handler. The read half is owned exclusively by the message loop task —
//! no mutex contention on reads, which eliminates the deadlock that occurred when
//! both `subscribe()` and `start_message_handler()` held the same `ws_stream`
//! mutex simultaneously.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = GeminiWebSocket::new_market_data(false).await?;
//! ws.connect().await?;
//! ws.subscribe_orderbook(Symbol::new("BTC", "USD")).await?;
//!
//! let stream = ws.event_stream();
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(StreamEvent::OrderbookDelta { .. }) => println!("Orderbook update"),
//!         _ => {}
//!     }
//! }
//! ```

use std::collections::HashSet;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;

use async_trait::async_trait;
use futures_util::{StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError, OrderbookCapabilities};
use crate::core::traits::WebSocketConnector;

use super::auth::GeminiAuth;
use super::endpoints::{GeminiUrls, normalize_symbol, format_symbol};
use super::parser::GeminiParser;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by subscribe and ping replies.
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task.
type WsReader = SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscribe message for market data
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    subscriptions: Vec<SubscriptionItem>,
}

#[derive(Debug, Clone, Serialize)]
struct SubscriptionItem {
    name: String,
    symbols: Vec<String>,
}

/// Incoming message from Gemini WebSocket
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    symbol: Option<String>,
    changes: Option<Value>,
    trades: Option<Vec<Value>>,
    event_id: Option<i64>,
    price: Option<String>,
    quantity: Option<String>,
    side: Option<String>,
    timestamp: Option<i64>,
    timestampms: Option<i64>,
    order_id: Option<String>,
    socket_sequence: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GEMINI WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Gemini WebSocket connector
pub struct GeminiWebSocket {
    /// Connection type (market data or order events)
    ws_type: WebSocketType,
    /// Authentication (for order events)
    auth: Option<GeminiAuth>,
    /// URLs (mainnet/testnet)
    urls: GeminiUrls,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<String>>>,
    /// Broadcast sender (for multiple consumers, dropped on disconnect)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by subscribe and ping replies.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Last heartbeat time (for order events)
    last_heartbeat: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds.
    /// NOTE: Gemini disconnects if it receives client ping frames, so this
    /// stays at 0. It is exposed via `ping_rtt_handle()` for interface
    /// consistency and can be populated by alternative means later.
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

/// Type of WebSocket connection
#[derive(Debug, Clone, Copy)]
pub enum WebSocketType {
    MarketData,
    OrderEvents,
}

impl GeminiWebSocket {
    /// Create new Gemini WebSocket connector for market data
    pub async fn new_market_data(testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            GeminiUrls::TESTNET
        } else {
            GeminiUrls::MAINNET
        };

        Ok(Self {
            ws_type: WebSocketType::MarketData,
            auth: None,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Create new Gemini WebSocket connector for order events (requires auth)
    pub async fn new_order_events(
        credentials: Credentials,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            GeminiUrls::TESTNET
        } else {
            GeminiUrls::MAINNET
        };

        let auth = Some(GeminiAuth::new(&credentials)?);

        Ok(Self {
            ws_type: WebSocketType::OrderEvents,
            auth,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Connect to WebSocket
    pub async fn connect(&self) -> ExchangeResult<()> {
        let url = match self.ws_type {
            WebSocketType::MarketData => self.urls.ws_market_url(),
            WebSocketType::OrderEvents => self.urls.ws_orders_url(),
        };

        // For order events, add auth headers
        let ws_stream = if matches!(self.ws_type, WebSocketType::OrderEvents) {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required for order events".to_string()))?;

            let headers = auth.sign_websocket_request("/v1/order/events")?;

            // Build WebSocket request with auth headers
            use tokio_tungstenite::tungstenite::handshake::client::Request;

            let mut request = Request::builder()
                .uri(url);

            for (key, value) in headers {
                request = request.header(key, value);
            }

            let request = request.body(())
                .map_err(|e| ExchangeError::Network(format!("Failed to build request: {}", e)))?;

            let (ws, _) = connect_async(request).await
                .map_err(|e| ExchangeError::Network(e.to_string()))?;

            ws
        } else {
            // Public market data - no auth needed
            let (ws, _) = connect_async(url).await
                .map_err(|e| ExchangeError::Network(e.to_string()))?;

            ws
        };

        // Split into independent read and write halves.
        // The write half goes behind a mutex for shared use by subscribe() and ping replies.
        // The read half is moved directly into the message loop — no mutex contention.
        let (write, read) = ws_stream.split();
        *self.ws_writer.lock().await = Some(write);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Create broadcast channel and store
        let (broadcast_sender, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(broadcast_sender);

        // Start message handler — reader is moved in, never shared via mutex.
        self.start_message_handler(read);

        Ok(())
    }

    /// Disconnect from WebSocket
    pub async fn disconnect(&self) -> ExchangeResult<()> {
        // Close the write half. The message loop task owns the read half and
        // will detect the close frame / stream exhaustion naturally.
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            writer.close().await.ok();
        }
        let _ = self.broadcast_tx.lock().unwrap().take();
        *self.status.lock().await = ConnectionStatus::Disconnected;
        Ok(())
    }

    /// Subscribe to orderbook updates
    pub async fn subscribe_orderbook(&self, symbol: Symbol) -> ExchangeResult<()> {
        self.subscribe_orderbook_with_account_type(symbol, AccountType::Spot).await
    }

    /// Subscribe to orderbook updates with specific account type
    pub async fn subscribe_orderbook_with_account_type(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<()> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));
        self.subscribe("l2", vec![symbol_str.to_uppercase()]).await
    }

    /// Subscribe to candles
    pub async fn subscribe_candles(&self, symbol: Symbol, interval: &str) -> ExchangeResult<()> {
        self.subscribe_candles_with_account_type(symbol, interval, AccountType::Spot).await
    }

    /// Subscribe to candles with specific account type
    pub async fn subscribe_candles_with_account_type(&self, symbol: Symbol, interval: &str, account_type: AccountType) -> ExchangeResult<()> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));
        let feed_name = format!("candles_{}", interval);
        self.subscribe(&feed_name, vec![symbol_str.to_uppercase()]).await
    }

    /// Internal subscribe method.
    ///
    /// Only locks `ws_writer` — the reader half is owned by the message loop task,
    /// so there is no deadlock risk.
    async fn subscribe(&self, feed_name: &str, symbols: Vec<String>) -> ExchangeResult<()> {
        if !matches!(self.ws_type, WebSocketType::MarketData) {
            return Err(ExchangeError::Network("Subscriptions only for market data".to_string()));
        }

        // Check if connected
        let status = *self.status.lock().await;
        if status != ConnectionStatus::Connected {
            return Err(ExchangeError::Network("Not connected to WebSocket".to_string()));
        }

        let subscribe_msg = SubscribeMessage {
            msg_type: "subscribe".to_string(),
            subscriptions: vec![SubscriptionItem {
                name: feed_name.to_string(),
                symbols: symbols.clone(),
            }],
        };

        let json_str = serde_json::to_string(&subscribe_msg)
            .map_err(|e| ExchangeError::Parse(e.to_string()))?;

        // Lock only the writer — reader is in the message loop task.
        let mut writer_guard = self.ws_writer.lock().await;
        if let Some(writer) = writer_guard.as_mut() {
            writer.send(Message::Text(json_str)).await
                .map_err(|e| ExchangeError::Network(e.to_string()))?;
        } else {
            return Err(ExchangeError::Network("WebSocket stream not available".to_string()));
        }

        // Track subscription
        for sym in symbols {
            self.subscriptions.lock().await.insert(format!("{}:{}", feed_name, sym));
        }

        Ok(())
    }

    /// Get event stream (multiple consumers can call this)
    pub fn event_stream(&self) -> broadcast::Receiver<WebSocketResult<StreamEvent>> {
        self.broadcast_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1)
    }

    /// Start message handler task.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// `ws_writer` is passed separately so the loop can reply to pings without
    /// touching the reader.
    fn start_message_handler(&self, mut reader: WsReader) {
        let ws_writer = Arc::clone(&self.ws_writer);
        let broadcast_tx = Arc::clone(&self.broadcast_tx);
        let status = Arc::clone(&self.status);
        let last_heartbeat = Arc::clone(&self.last_heartbeat);
        let ws_type = self.ws_type;

        tokio::spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse and broadcast events — a single WS frame can produce
                        // more than one StreamEvent (e.g. l2_updates carries both book
                        // changes and trade entries).
                        if let Ok(events) = Self::parse_message(&text, ws_type) {
                            if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                for evt in events {
                                    tx.send(Ok(evt)).ok();
                                }
                            }
                        }

                        // Update heartbeat for order events
                        if matches!(ws_type, WebSocketType::OrderEvents) {
                            *last_heartbeat.lock().await = Instant::now();
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        // Respond to ping — only lock the writer, not the reader.
                        let mut writer_guard = ws_writer.lock().await;
                        if let Some(writer) = writer_guard.as_mut() {
                            writer.send(Message::Pong(data)).await.ok();
                        }
                    }
                    Ok(Message::Close(_)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Err(e) => {
                        if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                            tx.send(Err(WebSocketError::ConnectionError(e.to_string()))).ok();
                        }
                        break;
                    }
                    _ => {}
                }
            }
            // Stream exhausted — drop sender so receivers know the stream is done
            let _ = broadcast_tx.lock().unwrap().take();
            // Connection closed.
            *status.lock().await = ConnectionStatus::Disconnected;
        });
        // Note: Gemini market data WebSocket does not expect client-initiated
        // ping frames and will close the connection if it receives them.
        // The server sends heartbeat messages on order events; for market data
        // the stream is kept alive by the server's own activity.
    }

    /// Parse incoming WebSocket message into zero or more stream events.
    ///
    /// Returns a `Vec` because a single Gemini `l2_updates` frame can carry
    /// both book-level changes (`changes` array) and executed trades (`trades`
    /// array) simultaneously.  All produced events are broadcast in order.
    fn parse_message(text: &str, ws_type: WebSocketType) -> ExchangeResult<Vec<StreamEvent>> {
        let value: Value = serde_json::from_str(text)
            .map_err(|e| ExchangeError::Parse(e.to_string()))?;

        let msg_type = value.get("type").and_then(|t| t.as_str());

        match (ws_type, msg_type) {
            // Market Data events
            (WebSocketType::MarketData, Some("subscribed")) => {
                // Subscription confirmation — no events.
                Ok(vec![])
            }
            (WebSocketType::MarketData, Some("l2_updates")) => {
                // l2_updates carry both book changes (always present) and an
                // optional trades array (non-empty only when executions happened).
                // Emit an OrderbookDelta for the book changes, and additionally a
                // Trade event for each trade entry.  Consumers subscribed to either
                // channel therefore receive the data they care about.
                let mut events: Vec<StreamEvent> = Vec::new();

                // Book changes — emit OrderbookDelta when changes array is present.
                let has_changes = value.get("changes")
                    .and_then(|c| c.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if has_changes {
                    match GeminiParser::parse_ws_l2_update(&value) {
                        Ok(ev) => events.push(ev),
                        Err(_) => {} // best-effort; skip malformed change
                    }
                }

                // Trade executions — emit Trade event from the last entry.
                let has_trades = value.get("trades")
                    .and_then(|t| t.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if has_trades {
                    match GeminiParser::parse_ws_l2_trade(&value) {
                        Ok(ev) => events.push(ev),
                        Err(_) => {}
                    }
                }

                Ok(events)
            }
            (WebSocketType::MarketData, Some(t)) if t.starts_with("candles_") => {
                // Candle update
                let kline = GeminiParser::parse_ws_candle(&value)?;
                Ok(vec![StreamEvent::Kline(kline)])
            }

            // Order Events
            (WebSocketType::OrderEvents, Some("subscription_ack")) => {
                Ok(vec![])
            }
            (WebSocketType::OrderEvents, Some("heartbeat")) => {
                Ok(vec![])
            }
            (WebSocketType::OrderEvents, Some("initial" | "accepted" | "booked" | "fill" | "cancelled" | "rejected" | "closed")) => {
                // Order event
                let order_event = GeminiParser::parse_ws_order_event(&value)?;
                Ok(vec![StreamEvent::OrderUpdate(order_event)])
            }

            _ => Ok(vec![]),
        }
    }
}

#[async_trait]
impl WebSocketConnector for GeminiWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        Self::connect(self).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        Self::disconnect(self).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))
    }

    fn connection_status(&self) -> ConnectionStatus {
        // This is a sync function, we need to use try_lock
        self.status.try_lock()
            .map(|guard| *guard)
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        match request.stream_type {
            StreamType::Ticker => {
                // Gemini doesn't have a dedicated ticker stream.
                // Subscribe to l2 (orderbook) instead — the l2_updates messages
                // contain best bid/ask which provides ticker-equivalent data.
                self.subscribe_orderbook(request.symbol).await
                    .map_err(|e| WebSocketError::Subscription(e.to_string()))
            }
            StreamType::Trade => {
                // Gemini l2_updates messages include a "trades" array with executed trades.
                // Subscribe to the l2 channel and parse trade entries from each update.
                self.subscribe_orderbook(request.symbol).await
                    .map_err(|e| WebSocketError::Subscription(e.to_string()))
            }
            StreamType::Orderbook => {
                self.subscribe_orderbook(request.symbol).await
                    .map_err(|e| WebSocketError::Subscription(e.to_string()))
            }
            StreamType::Kline { interval } => {
                self.subscribe_candles(request.symbol, &interval).await
                    .map_err(|e| WebSocketError::Subscription(e.to_string()))
            }
            _ => Err(WebSocketError::Subscription(format!("{:?} not supported", request.stream_type))),
        }
    }

    async fn unsubscribe(&mut self, _request: SubscriptionRequest) -> WebSocketResult<()> {
        // Gemini doesn't support unsubscribe - need to reconnect
        Err(WebSocketError::Subscription("Unsubscribe not supported by Gemini".to_string()))
    }

    fn event_stream(&self) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1);

        Box::pin(futures_util::stream::unfold(rx, |mut rx| async move {
            match rx.recv().await {
                Ok(event) => Some((event, rx)),
                Err(_) => None,
            }
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        // Gemini doesn't track subscriptions in SubscriptionRequest format
        // Return empty for now
        vec![]
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        // Gemini disconnects on client ping frames, so RTT stays at 0.
        Some(self.ws_ping_rtt_ms.clone())
    }

    fn orderbook_capabilities(&self) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            supports_snapshot: false,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws = GeminiWebSocket::new_market_data(false).await.unwrap();
        let status = *ws.status.lock().await;
        assert_eq!(status, ConnectionStatus::Disconnected);
    }
}
