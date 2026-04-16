//! Polymarket CLOB WebSocket Client
//!
//! Real-time market data streaming from the CLOB WebSocket endpoint.
//!
//! ## URL
//!
//! `wss://ws-subscriptions-clob.polymarket.com/ws/market`
//!
//! ## Message flow
//!
//! 1. Connect to WS URL
//! 2. Send subscription message: `{"type": "market", "assets_ids": ["TOKEN_ID_1", ...]}`
//! 3. Receive events: `book`, `price_change`, `last_trade_price`, `tick_size_changed`, etc.
//! 4. Send "PING" string every 10 seconds to keep alive
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::data_feeds::prediction::polymarket::ClobWebSocket;
//!
//! let token_ids = vec!["TOKEN_ID_1".to_string()];
//! let mut ws = ClobWebSocket::new(token_ids);
//!
//! ws.connect().await?;
//!
//! while let Ok(Some(event)) = ws.recv().await {
//!     match event {
//!         WsEvent::Book(snapshot) => {
//!             println!("Book: {} bids, {} asks", snapshot.bids.len(), snapshot.asks.len());
//!         }
//!         WsEvent::LastTradePrice(trade) => {
//!             println!("Trade: {} @ {}", trade.size.as_deref().unwrap_or("?"), trade.price);
//!         }
//!         _ => {}
//!     }
//! }
//! ```

use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};

use super::parser::{
    WsBookSnapshot, WsLastTradePrice, WsPriceChange, WsTickSizeChange,
    WsBestBidAsk,
};

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

/// WebSocket market channel URL
const WS_MARKET_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// WebSocket user channel URL (authenticated)
const _WS_USER_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/user";

/// Ping interval (10 seconds)
const PING_INTERVAL_SECS: u64 = 10;

/// Initial reconnection delay
const INITIAL_BACKOFF_SECS: u64 = 1;

/// Maximum reconnection delay
const MAX_BACKOFF_SECS: u64 = 60;

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// WebSocket error
#[derive(Debug)]
pub enum WsError {
    Connection(String),
    Send(String),
    Receive(String),
    Parse(String),
    Disconnected,
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(s) => write!(f, "Connection error: {}", s),
            Self::Send(s) => write!(f, "Send error: {}", s),
            Self::Receive(s) => write!(f, "Receive error: {}", s),
            Self::Parse(s) => write!(f, "Parse error: {}", s),
            Self::Disconnected => write!(f, "WebSocket disconnected"),
        }
    }
}

impl std::error::Error for WsError {}

// ═══════════════════════════════════════════════════════════════════════════
// EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Reconnection information
#[derive(Debug, Clone)]
pub struct WsReconnectInfo {
    pub reconnection_count: u64,
    pub markets_resubscribed: usize,
}

/// Unknown or unhandled event
#[derive(Debug, Clone)]
pub struct WsUnknownEvent {
    pub raw: String,
}

/// Parsed WebSocket event from Polymarket CLOB
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// Full order book snapshot
    Book(WsBookSnapshot),
    /// Incremental price level update
    PriceChange(WsPriceChange),
    /// Last trade price
    LastTradePrice(WsLastTradePrice),
    /// Tick size changed
    TickSizeChange(WsTickSizeChange),
    /// Best bid/ask update
    BestBidAsk(WsBestBidAsk),
    /// Pong response to PING
    Pong,
    /// Reconnected after disconnect
    Reconnected(WsReconnectInfo),
    /// Unknown/unhandled event type (gracefully handled)
    Unknown(WsUnknownEvent),
}

// ═══════════════════════════════════════════════════════════════════════════
// WEBSOCKET CLIENT
// ═══════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Polymarket CLOB WebSocket client for market data
pub struct ClobWebSocket {
    /// Token IDs subscribed to
    token_ids: Vec<String>,
    /// Enable feature-flagged events (best_bid_ask)
    enable_features: bool,
    /// Active WebSocket connection
    ws: Option<WsStream>,
    /// Last ping time
    last_ping: Instant,
    /// Exponential backoff delay for reconnections
    backoff_delay: Duration,
    /// Total reconnection count since creation
    reconnection_count: u64,
}

impl ClobWebSocket {
    /// Create a new WebSocket client
    ///
    /// # Arguments
    ///
    /// * `token_ids` — Token IDs to subscribe to on connect
    /// * `enable_features` — Enable best_bid_ask and other feature-flagged events
    pub fn new(token_ids: Vec<String>, enable_features: bool) -> Self {
        Self {
            token_ids,
            enable_features,
            ws: None,
            last_ping: Instant::now(),
            backoff_delay: Duration::from_secs(INITIAL_BACKOFF_SECS),
            reconnection_count: 0,
        }
    }

    /// Connect to the WebSocket server and subscribe to tracked token IDs
    pub async fn connect(&mut self) -> Result<(), WsError> {
        let (ws_stream, _) = connect_async(WS_MARKET_URL)
            .await
            .map_err(|e| WsError::Connection(e.to_string()))?;

        self.ws = Some(ws_stream);
        self.send_subscription().await?;
        self.backoff_delay = Duration::from_secs(INITIAL_BACKOFF_SECS);
        self.last_ping = Instant::now();

        Ok(())
    }

    /// Reconnect with exponential backoff
    pub async fn reconnect(&mut self) -> Result<WsReconnectInfo, WsError> {
        sleep(self.backoff_delay).await;

        let result = self.connect().await;
        if let Err(e) = result {
            self.backoff_delay = std::cmp::min(
                self.backoff_delay * 2,
                Duration::from_secs(MAX_BACKOFF_SECS),
            );
            return Err(e);
        }

        self.reconnection_count += 1;
        let markets_resubscribed = self.token_ids.len();

        tracing::info!(
            reconnection_count = self.reconnection_count,
            markets_resubscribed,
            "Polymarket WebSocket reconnected"
        );

        Ok(WsReconnectInfo {
            reconnection_count: self.reconnection_count,
            markets_resubscribed,
        })
    }

    /// Send initial subscription message
    async fn send_subscription(&mut self) -> Result<(), WsError> {
        let mut sub_value = serde_json::json!({
            "type": "market",
            "assets_ids": self.token_ids
        });

        if self.enable_features {
            if let Some(obj) = sub_value.as_object_mut() {
                obj.insert(
                    "custom_feature_enabled".to_string(),
                    Value::Bool(true),
                );
            }
        }

        let msg = serde_json::to_string(&sub_value)
            .map_err(|e| WsError::Send(e.to_string()))?;

        self.send_text(msg).await
    }

    /// Subscribe to additional token IDs
    pub async fn subscribe(&mut self, token_ids: Vec<String>) -> Result<(), WsError> {
        let sub_value = serde_json::json!({
            "assets_ids": token_ids,
            "operation": "subscribe"
        });

        let msg = serde_json::to_string(&sub_value)
            .map_err(|e| WsError::Send(e.to_string()))?;

        self.send_text(msg).await?;
        self.token_ids.extend(token_ids);
        Ok(())
    }

    /// Unsubscribe from token IDs
    pub async fn unsubscribe(&mut self, token_ids: &[String]) -> Result<(), WsError> {
        let sub_value = serde_json::json!({
            "assets_ids": token_ids,
            "operation": "unsubscribe"
        });

        let msg = serde_json::to_string(&sub_value)
            .map_err(|e| WsError::Send(e.to_string()))?;

        self.send_text(msg).await?;
        self.token_ids.retain(|id| !token_ids.contains(id));
        Ok(())
    }

    /// Send a text message over the WebSocket
    async fn send_text(&mut self, text: String) -> Result<(), WsError> {
        let ws = self.ws.as_mut().ok_or(WsError::Disconnected)?;
        ws.send(Message::Text(text))
            .await
            .map_err(|e| WsError::Send(e.to_string()))
    }

    /// Send ping keepalive
    async fn send_ping(&mut self) -> Result<(), WsError> {
        self.send_text("PING".to_string()).await?;
        self.last_ping = Instant::now();
        Ok(())
    }

    /// Receive next event
    ///
    /// Handles automatic ping/pong keepalive. Returns `Ok(None)` on graceful close.
    pub async fn recv(&mut self) -> Result<Option<WsEvent>, WsError> {
        loop {
            // Send ping if interval elapsed
            if self.last_ping.elapsed() >= Duration::from_secs(PING_INTERVAL_SECS) {
                self.send_ping().await?;
            }

            let ws = self.ws.as_mut().ok_or(WsError::Disconnected)?;

            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    if text == "PONG" {
                        return Ok(Some(WsEvent::Pong));
                    }
                    return parse_event(&text).map(Some);
                }
                Some(Ok(Message::Close(_))) => return Ok(None),
                Some(Err(e)) => return Err(WsError::Receive(e.to_string())),
                None => return Ok(None),
                _ => continue, // Binary, Ping, Pong, Frame — ignore
            }
        }
    }

    /// Run the WebSocket loop, sending events to a channel
    ///
    /// Runs indefinitely, handling reconnections automatically.
    pub async fn start(
        &mut self,
        tx: tokio::sync::mpsc::Sender<WsEvent>,
    ) -> Result<(), WsError> {
        loop {
            if self.ws.is_none() {
                self.connect().await?;
            }

            match self.recv().await {
                Ok(Some(event)) => {
                    let _ = tx.send(event).await;
                }
                Ok(None) => {
                    // Graceful close — reconnect
                    self.ws = None;
                    match self.reconnect().await {
                        Ok(info) => {
                            let _ = tx.send(WsEvent::Reconnected(info)).await;
                        }
                        Err(e) => {
                            tracing::warn!("Polymarket WS reconnect failed: {}", e);
                            // Continue loop — will retry
                        }
                    }
                }
                Err(WsError::Disconnected) => {
                    self.ws = None;
                    match self.reconnect().await {
                        Ok(info) => {
                            let _ = tx.send(WsEvent::Reconnected(info)).await;
                        }
                        Err(e) => {
                            tracing::warn!("Polymarket WS reconnect failed: {}", e);
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Close the WebSocket connection
    pub async fn close(&mut self) {
        if let Some(mut ws) = self.ws.take() {
            let _ = ws.close(None).await;
        }
    }

    /// Whether the WebSocket is currently connected
    pub fn is_connected(&self) -> bool {
        self.ws.is_some()
    }

    /// Get total reconnection count
    pub fn reconnection_count(&self) -> u64 {
        self.reconnection_count
    }

    /// Get currently subscribed token IDs
    pub fn subscribed_tokens(&self) -> &[String] {
        &self.token_ids
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EVENT PARSING
// ═══════════════════════════════════════════════════════════════════════════

/// Parse a WebSocket message JSON string into a WsEvent
pub fn parse_event(json: &str) -> Result<WsEvent, WsError> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| WsError::Parse(e.to_string()))?;

    let event_type = match value.get("event_type").and_then(|v| v.as_str()) {
        Some(et) => et,
        None => {
            // Some messages don't have event_type (acks, heartbeats)
            tracing::debug!("Polymarket WS: message without event_type: {}", json);
            return Ok(WsEvent::Unknown(WsUnknownEvent { raw: json.to_string() }));
        }
    };

    match event_type {
        "book" => {
            let mut snapshot: WsBookSnapshot = serde_json::from_value(value.clone())
                .map_err(|e| WsError::Parse(format!("book parse: {}", e)))?;
            // Normalize prices (.48 -> 0.48)
            for level in &mut snapshot.bids {
                normalize_price_in_place(&mut level.price);
            }
            for level in &mut snapshot.asks {
                normalize_price_in_place(&mut level.price);
            }
            Ok(WsEvent::Book(snapshot))
        }
        "price_change" => {
            let mut change: WsPriceChange = serde_json::from_value(value)
                .map_err(|e| WsError::Parse(format!("price_change parse: {}", e)))?;
            for level in &mut change.changes {
                normalize_price_in_place(&mut level.price);
            }
            Ok(WsEvent::PriceChange(change))
        }
        "last_trade_price" => {
            let trade: WsLastTradePrice = serde_json::from_value(value)
                .map_err(|e| WsError::Parse(format!("last_trade_price parse: {}", e)))?;
            Ok(WsEvent::LastTradePrice(trade))
        }
        "tick_size_change" => {
            let change: WsTickSizeChange = serde_json::from_value(value)
                .map_err(|e| WsError::Parse(format!("tick_size_change parse: {}", e)))?;
            Ok(WsEvent::TickSizeChange(change))
        }
        "best_bid_ask" => {
            let bba: WsBestBidAsk = serde_json::from_value(value)
                .map_err(|e| WsError::Parse(format!("best_bid_ask parse: {}", e)))?;
            Ok(WsEvent::BestBidAsk(bba))
        }
        _ => {
            tracing::debug!(
                "Polymarket WS: unhandled event_type '{}': {}",
                event_type,
                json
            );
            Ok(WsEvent::Unknown(WsUnknownEvent { raw: json.to_string() }))
        }
    }
}

/// Normalize a price string (`.48` → `0.48`)
fn normalize_price_in_place(price: &mut String) {
    if price.starts_with('.') {
        *price = format!("0{}", price);
    }
}

/// Normalize a price string (returns new string)
pub fn normalize_price(price: &str) -> String {
    if price.starts_with('.') {
        format!("0{}", price)
    } else {
        price.to_string()
    }
}
