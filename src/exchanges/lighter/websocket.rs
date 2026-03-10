//! # Lighter WebSocket Implementation
//!
//! WebSocket connector for Lighter DEX.
//!
//! ## Features
//! - Public channels (orderbook, trades, market stats)
//! - Authenticated channels (account updates - Phase 2)
//! - Ping/pong heartbeat
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Channels
//! - `order_book/{market_id}` - Order book updates (50ms batches)
//! - `trade/{market_id}` - Trade executions
//! - `market_stats/{market_id}` - Market statistics
//! - `account_all/{account_id}` - Account data (public)
//! - `account_market/{market_id}/{account_id}` - Market-specific account (auth required)

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::{mpsc, broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType, ExchangeResult,
    ConnectionStatus, StreamEvent, SubscriptionRequest,
    Ticker, PublicTrade, OrderBook,
};
use crate::core::types::TradeSide;
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;

use super::auth::LighterAuth;
use super::endpoints::{LighterUrls, symbol_to_market_id};

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET ID MAPPING
// ═══════════════════════════════════════════════════════════════════════════════

// symbol_to_market_id is imported from endpoints.rs (shared with connector.rs)

/// Build the correct Lighter WebSocket channel name for a given stream type and symbol.
fn build_channel(stream_type: &crate::core::types::StreamType, base: &str) -> Result<String, WebSocketError> {
    let market_id = symbol_to_market_id(base).ok_or_else(|| {
        WebSocketError::UnsupportedOperation(
            format!("Unknown Lighter market for base asset '{}'. Known: ETH(0), BTC(1), SOL(2), etc.", base)
        )
    })?;

    match stream_type {
        crate::core::types::StreamType::Ticker => Ok(format!("market_stats/{}", market_id)),
        crate::core::types::StreamType::Trade => Ok(format!("trade/{}", market_id)),
        crate::core::types::StreamType::Orderbook => Ok(format!("order_book/{}", market_id)),
        other => Err(WebSocketError::UnsupportedOperation(
            format!("Stream type {:?} not supported for Lighter WebSocket", other)
        )),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscription message.
///
/// Lighter uses `{"type": "subscribe", "channel": "order_book/0"}` format.
/// Previous code used `{"method": "subscribe", "params": {"channel": "..."}}` which
/// returned error 30001 "Invalid Type".
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<String>,
}

/// Ping message (used by send_ping for client-initiated keepalive)
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct PingMessage {
    #[serde(rename = "type")]
    msg_type: String,
}

/// Pong message
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct PongMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,
}

/// Incoming message from Lighter.
///
/// Lighter messages have varied structures:
/// - Connection: `{"session_id":"...","type":"connected"}`
/// - Data: `{"channel":"market_stats:0","type":"update/market_stats","market_stats":{...}}`
/// - Error: `{"error":{"code":30001,"message":"..."}}`
///
/// Data fields are nested inside sub-objects (e.g., `market_stats`, `order_book`, `trade`)
/// rather than at the top level. We parse as a generic Value and extract fields dynamically.
#[derive(Debug, Clone)]
struct IncomingMessage {
    /// The raw JSON value for flexible field access
    raw: Value,
}

impl IncomingMessage {
    fn from_value(v: Value) -> Self {
        Self { raw: v }
    }

    fn msg_type(&self) -> Option<&str> {
        self.raw.get("type").and_then(|v| v.as_str())
    }

    fn channel(&self) -> Option<&str> {
        self.raw.get("channel").and_then(|v| v.as_str())
    }

    fn error_message(&self) -> Option<String> {
        // Top-level message field
        if let Some(msg) = self.raw.get("message").and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
        // Nested error object: {"error":{"code":..., "message":"..."}}
        if let Some(err) = self.raw.get("error") {
            if let Some(msg) = err.get("message").and_then(|v| v.as_str()) {
                return Some(msg.to_string());
            }
        }
        None
    }

    /// Get the nested data object for a given channel type.
    /// e.g., for channel "market_stats:0", returns the "market_stats" sub-object.
    fn data_object(&self, key: &str) -> Option<&Value> {
        self.raw.get(key)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIGHTER WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Lighter WebSocket connector
#[allow(dead_code)]
pub struct LighterWebSocket {
    /// Authentication (optional, used in Phase 2 for authenticated channels)
    auth: Option<LighterAuth>,
    /// URLs (mainnet/testnet)
    urls: LighterUrls,
    /// Testnet mode
    testnet: bool,
    /// WebSocket connection (None if not connected)
    ws: Arc<Mutex<Option<WsStream>>>,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions (channel names)
    subscriptions: Arc<Mutex<HashSet<String>>>,
    /// Active subscription requests (full objects for tracking)
    subscription_requests: Arc<Mutex<Vec<SubscriptionRequest>>>,
    /// Internal event sender (message loop -> forwarder)
    event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
    /// Internal event receiver (forwarder reads from this)
    event_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<WebSocketResult<StreamEvent>>>>>,
    /// Broadcast sender — behind StdMutex so event_stream() can subscribe
    /// without contending with the async message loop.
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Ping interval (30 seconds recommended)
    ping_interval: Duration,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl LighterWebSocket {
    /// Create new WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            LighterUrls::TESTNET
        } else {
            LighterUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(LighterAuth::new)
            .transpose()?;

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            auth,
            urls,
            testnet,
            ws: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            subscription_requests: Arc::new(Mutex::new(Vec::new())),
            event_tx,
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ping_interval: Duration::from_secs(30),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Create public-only WebSocket connector
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CONNECTION MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Connect to WebSocket
    async fn connect_ws(&self) -> WebSocketResult<()> {
        let ws_url = self.urls.ws_url();

        // Connect
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Store connection
        *self.ws.lock().await = Some(ws_stream);
        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    /// Disconnect from WebSocket
    async fn disconnect_ws(&self) -> WebSocketResult<()> {
        if let Some(mut ws) = self.ws.lock().await.take() {
            let _ = ws.close(None).await;
        }
        *self.status.lock().await = ConnectionStatus::Disconnected;
        let _ = self.broadcast_tx.lock().unwrap().take();
        Ok(())
    }

    /// Subscribe to a channel
    async fn subscribe_channel(&self, channel: &str, auth: Option<String>) -> WebSocketResult<()> {
        let msg = SubscribeMessage {
            msg_type: "subscribe".to_string(),
            channel: channel.to_string(),
            auth,
        };

        let json_str = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;


        if let Some(ws) = self.ws.lock().await.as_mut() {
            ws.send(Message::Text(json_str))
                .await
                .map_err(|e| WebSocketError::SendError(e.to_string()))?;

            // Add to subscriptions
            self.subscriptions.lock().await.insert(channel.to_string());
        } else {
            return Err(WebSocketError::NotConnected);
        }

        Ok(())
    }

    /// Unsubscribe from a channel
    async fn unsubscribe_channel(&self, channel: &str) -> WebSocketResult<()> {
        let msg = json!({
            "type": "unsubscribe",
            "channel": channel
        });

        let json_str = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        if let Some(ws) = self.ws.lock().await.as_mut() {
            ws.send(Message::Text(json_str))
                .await
                .map_err(|e| WebSocketError::SendError(e.to_string()))?;

            // Remove from subscriptions
            self.subscriptions.lock().await.remove(channel);
        }

        Ok(())
    }

    /// Send ping to keep connection alive (used for periodic keepalive)
    #[allow(dead_code)]
    async fn send_ping(&self) -> WebSocketResult<()> {
        let msg = PingMessage {
            msg_type: "ping".to_string(),
        };

        let json_str = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        if let Some(ws) = self.ws.lock().await.as_mut() {
            ws.send(Message::Text(json_str))
                .await
                .map_err(|e| WebSocketError::SendError(e.to_string()))?;

            *self.last_ping.lock().await = Instant::now();
        }

        Ok(())
    }

    /// Start message handler loop
    async fn start_message_loop(&self) {
        let ws = self.ws.clone();
        let event_tx = self.event_tx.clone();
        let status = self.status.clone();
        let last_ping = self.last_ping.clone();
        let ws_ping_rtt_ms = self.ws_ping_rtt_ms.clone();

        tokio::spawn(async move {
            loop {
                // Check if connected
                let mut ws_guard = ws.lock().await;
                if ws_guard.is_none() {
                    break;
                }

                // Read next message
                if let Some(msg_result) = ws_guard.as_mut().expect("WebSocket is initialized").next().await {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            // Parse JSON once for all handling
                            let val = match serde_json::from_str::<Value>(&text) {
                                Ok(v) => v,
                                Err(_) => continue,
                            };

                            // Handle JSON ping from server: {"type":"ping","timestamp":...}
                            if val.get("type").and_then(|t| t.as_str()) == Some("ping") {
                                let ts = val.get("timestamp").and_then(|t| t.as_i64());
                                let pong = if let Some(ts) = ts {
                                    json!({"type": "pong", "timestamp": ts})
                                } else {
                                    json!({"type": "pong"})
                                };
                                if let Some(ws_inner) = ws_guard.as_mut() {
                                    let _ = ws_inner.send(Message::Text(pong.to_string())).await;
                                }
                                continue;
                            }

                            // Handle data message
                            let incoming = IncomingMessage::from_value(val);
                            Self::handle_message(incoming, &event_tx);
                        }
                        Ok(Message::Ping(data)) => {
                            // Respond with WebSocket-level pong
                            if let Some(ws_inner) = ws_guard.as_mut() {
                                let _ = ws_inner.send(Message::Pong(data)).await;
                            }
                        }
                        Ok(Message::Pong(_)) => {
                            // Record RTT for the WS-level ping sent by the ping task
                            let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                            *ws_ping_rtt_ms.lock().await = rtt;
                        }
                        Ok(Message::Close(_)) => {
                            *status.lock().await = ConnectionStatus::Disconnected;
                            break;
                        }
                        Err(_) => {
                            *status.lock().await = ConnectionStatus::Disconnected;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    /// Start WS-level ping task for RTT measurement.
    ///
    /// Sends `Message::Ping` every 5 seconds through the shared WS mutex.
    /// The message loop handles `Message::Pong` responses and records the RTT.
    fn start_ws_ping_task(&self) {
        let ws = self.ws.clone();
        let last_ping = self.last_ping.clone();
        let ping_interval = self.ping_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(ping_interval);
            // Skip immediate first tick
            interval.tick().await;

            loop {
                interval.tick().await;

                let mut ws_guard = ws.lock().await;
                if let Some(ws_inner) = ws_guard.as_mut() {
                    *last_ping.lock().await = Instant::now();
                    if ws_inner.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Start forwarder task (mpsc -> broadcast) so multiple consumers can use event_stream()
    fn start_forwarder(&self) {
        let broadcast_tx = self.broadcast_tx.clone();
        let event_rx = self.event_rx.clone();

        // Create broadcast channel and store sender
        let (tx, _) = broadcast::channel(1000);
        *broadcast_tx.lock().unwrap() = Some(tx);

        let broadcast_tx_inner = self.broadcast_tx.clone();
        tokio::spawn(async move {
            let mut rx = match event_rx.lock().await.take() {
                Some(rx) => rx,
                None => return,
            };
            while let Some(event) = rx.recv().await {
                let tx_guard = broadcast_tx_inner.lock().unwrap();
                if let Some(ref tx) = *tx_guard {
                    let _ = tx.send(event);
                }
            }
            // Drop the broadcast sender so consumers get None
            let _ = broadcast_tx_inner.lock().unwrap().take();
        });
    }

    /// Handle incoming message - parse into StreamEvent and send through channel.
    ///
    /// Note: JSON ping/pong is handled in the message loop before this is called.
    fn handle_message(
        msg: IncomingMessage,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
    ) {
        // Check for error messages (nested: {"error":{"code":...,"message":"..."}})
        if msg.raw.get("error").is_some() {
            let error_msg = msg.error_message().unwrap_or_else(|| "Unknown error".to_string());
            let _ = event_tx.send(Err(WebSocketError::ProtocolError(error_msg)));
            return;
        }

        // Handle special message types
        match msg.msg_type() {
            Some("pong") => return,
            Some("connected") => return,
            Some("error") => {
                let error_msg = msg.error_message().unwrap_or_else(|| "Unknown error".to_string());
                eprintln!("[lighter-ws] error from server: {}", error_msg);
                let _ = event_tx.send(Err(WebSocketError::ProtocolError(error_msg)));
                return;
            }
            None => return,
            _ => {}
        }

        let msg_type = msg.msg_type().unwrap_or("");
        let channel = msg.channel().unwrap_or("");

        match msg_type {
            // ── Order book update ────────────────────────────────────
            // Actual type from server: "update/order_book" (with underscore)
            "update/orderbook" | "update/order_book" => {
                if let Some(event) = Self::parse_orderbook(&msg, channel) {
                    let _ = event_tx.send(Ok(event));
                }
            }

            // ── Trade update ─────────────────────────────────────────
            "update/trade" => {
                if let Some(event) = Self::parse_trade(&msg, channel) {
                    let _ = event_tx.send(Ok(event));
                }
            }

            // ── Market stats (used as Ticker) ────────────────────────
            "update/market_stats" => {
                if let Some(event) = Self::parse_market_stats(&msg, channel) {
                    let _ = event_tx.send(Ok(event));
                }
            }

            // ── Subscription acknowledgements ─────────────────────────
            // ── Subscription acknowledgements and unknown types ──────
            _ => {}
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MESSAGE PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    // ═══════════════════════════════════════════════════════════════════════════
    // VALUE HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract market ID from channel string.
    /// Lighter uses colon separator: "order_book:0" -> "0", "market_stats:0" -> "0"
    fn extract_market_id(channel: &str) -> &str {
        channel.rsplit(':').next()
            .or_else(|| channel.rsplit('/').next())
            .unwrap_or(channel)
    }

    /// Get a string field from a JSON Value, parsing it as f64.
    fn val_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| {
            v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64())
        })
    }

    /// Get a string field from a JSON Value.
    fn val_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    /// Get an integer field from a JSON Value.
    fn val_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    /// Get a u64 field from a JSON Value.
    fn val_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    /// Get a bool field from a JSON Value.
    fn val_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }

    /// Parse price/size levels from a JSON array into (f64, f64) tuples.
    ///
    /// Lighter orderbook levels are objects: `[{"price":"2738.02","size":"15.40"}, ...]`
    /// Also supports legacy array format: `[["2738.02","15.40"], ...]`
    fn parse_levels(arr: &Value) -> Vec<(f64, f64)> {
        arr.as_array()
            .map(|levels| {
                levels.iter().filter_map(|entry| {
                    // Object format: {"price": "...", "size": "..."}
                    if let Some(obj) = entry.as_object() {
                        let price = obj.get("price")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<f64>().ok())?;
                        let size = obj.get("size")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<f64>().ok())?;
                        Some((price, size))
                    }
                    // Array format: ["price", "size"]
                    else if let Some(pair_arr) = entry.as_array() {
                        if pair_arr.len() >= 2 {
                            let price = pair_arr[0].as_str()?.parse::<f64>().ok()?;
                            let size = pair_arr[1].as_str()?.parse::<f64>().ok()?;
                            Some((price, size))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }).collect()
            })
            .unwrap_or_default()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MESSAGE PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order book update into StreamEvent::OrderbookSnapshot.
    ///
    /// Expected format:
    /// ```json
    /// {"channel":"order_book:0","type":"update/orderbook","order_book":{"asks":[...],"bids":[...],"offset":...,"nonce":...},"timestamp":...}
    /// ```
    fn parse_orderbook(msg: &IncomingMessage, _channel: &str) -> Option<StreamEvent> {
        // Try nested "order_book" object first, then fall back to top level
        let data = msg.data_object("order_book").unwrap_or(&msg.raw);

        let asks = data.get("asks").map(Self::parse_levels).unwrap_or_default();
        let bids = data.get("bids").map(Self::parse_levels).unwrap_or_default();

        if asks.is_empty() && bids.is_empty() {
            return None;
        }

        let timestamp = Self::val_i64(&msg.raw, "timestamp")
            .or_else(|| Self::val_i64(data, "timestamp"))
            .unwrap_or(0);
        let sequence = Self::val_i64(data, "nonce").map(|n| n.to_string());

        Some(StreamEvent::OrderbookSnapshot(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
        }))
    }

    /// Parse trade update into StreamEvent::Trade.
    ///
    /// Expected format:
    /// ```json
    /// {"channel":"trade:0","type":"update/trade","trade":{"trade_id":...,"price":"...","size":"...","side":"buy","is_maker_ask":true},"timestamp":...}
    /// ```
    fn parse_trade(msg: &IncomingMessage, channel: &str) -> Option<StreamEvent> {
        // Try nested "trade" object first, then fall back to top level
        let data = msg.data_object("trade").unwrap_or(&msg.raw);

        let price = Self::val_f64(data, "price")?;
        let quantity = Self::val_f64(data, "size")?;
        let timestamp = Self::val_i64(&msg.raw, "timestamp")
            .or_else(|| Self::val_i64(data, "timestamp"))
            .unwrap_or(0);
        let trade_id = Self::val_u64(data, "trade_id").unwrap_or(0);
        let market_id = Self::extract_market_id(channel);

        // Determine side from "side" field or "is_maker_ask"
        let side = if let Some(side_str) = Self::val_str(data, "side") {
            match side_str {
                "buy" => TradeSide::Buy,
                "sell" => TradeSide::Sell,
                _ => {
                    if Self::val_bool(data, "is_maker_ask").unwrap_or(false) {
                        TradeSide::Buy
                    } else {
                        TradeSide::Sell
                    }
                }
            }
        } else if Self::val_bool(data, "is_maker_ask").unwrap_or(false) {
            TradeSide::Buy
        } else {
            TradeSide::Sell
        };

        Some(StreamEvent::Trade(PublicTrade {
            id: trade_id.to_string(),
            symbol: market_id.to_string(),
            price,
            quantity,
            side,
            timestamp,
        }))
    }

    /// Parse market stats update into StreamEvent::Ticker.
    ///
    /// Actual Lighter format (from live data):
    /// ```json
    /// {"channel":"market_stats:0","type":"update/market_stats","market_stats":{
    ///   "symbol":"ETH","market_id":0,"index_price":"2736.94","mark_price":"2735.44",
    ///   "open_interest":"...","last_trade_price":"2735.41",
    ///   "current_funding_rate":"...","daily_volume":"...",
    ///   "daily_price_high":"...","daily_price_low":"...","daily_price_change":"..."
    /// }}
    /// ```
    fn parse_market_stats(msg: &IncomingMessage, channel: &str) -> Option<StreamEvent> {
        // Data is nested inside "market_stats" object
        let data = msg.data_object("market_stats").unwrap_or(&msg.raw);

        // Field names from actual Lighter API (different from research docs):
        // - "last_trade_price" (not "last_price")
        // - "daily_price_high" (not "daily_high")
        // - "daily_price_low" (not "daily_low")
        // - "daily_price_change" (not "daily_change")
        // - "daily_volume" (same)
        // - "current_funding_rate" (not "funding_rate")
        let last_price = Self::val_f64(data, "last_trade_price")
            .or_else(|| Self::val_f64(data, "last_price"))
            .or_else(|| Self::val_f64(data, "mark_price"))?;

        let market_id = Self::extract_market_id(channel);
        let symbol_name = Self::val_str(data, "symbol").unwrap_or(market_id);

        let high_24h = Self::val_f64(data, "daily_price_high")
            .or_else(|| Self::val_f64(data, "daily_high"));
        let low_24h = Self::val_f64(data, "daily_price_low")
            .or_else(|| Self::val_f64(data, "daily_low"));
        let volume_24h = Self::val_f64(data, "daily_volume")
            .or_else(|| Self::val_f64(data, "daily_base_token_volume"));
        let price_change_24h = Self::val_f64(data, "daily_price_change")
            .or_else(|| Self::val_f64(data, "daily_change"));

        let timestamp = Self::val_i64(&msg.raw, "timestamp")
            .or_else(|| Self::val_i64(data, "timestamp"))
            .unwrap_or(0);

        // Compute 24h price change percent from absolute change and last price.
        // Formula: pct = (change / open_price) * 100 where open_price = last - change.
        // Guard against division by zero or nonsensical values.
        let price_change_percent_24h = price_change_24h.and_then(|change| {
            let open = last_price - change;
            if open.abs() > 1e-10 {
                Some((change / open) * 100.0)
            } else {
                None
            }
        });

        Some(StreamEvent::Ticker(Ticker {
            symbol: symbol_name.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h,
            price_change_percent_24h,
            timestamp,
        }))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for LighterWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        self.connect_ws().await?;
        self.start_message_loop().await;
        self.start_forwarder();
        self.start_ws_ping_task();
        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        self.disconnect_ws().await
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Block on the async mutex to get the current status
        // This is safe because we're just reading a simple enum value
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Map StreamType + Symbol to Lighter channel name using numeric market IDs
        // Lighter channels: order_book/{market_id}, trade/{market_id}, market_stats/{market_id}
        let channel = build_channel(&request.stream_type, &request.symbol.base)?;


        // For private streams, would need auth token
        let auth = None; // Phase 1 - public only

        self.subscribe_channel(&channel, auth).await?;

        // Track the subscription request
        self.subscription_requests.lock().await.push(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Map StreamType + Symbol to Lighter channel name using numeric market IDs
        let channel = build_channel(&request.stream_type, &request.symbol.base)?;

        self.unsubscribe_channel(&channel).await?;

        // Remove from tracked subscriptions
        self.subscription_requests.lock().await.retain(|r| {
            r.symbol != request.symbol || r.stream_type != request.stream_type
        });

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

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        // Return cloned subscription requests
        match self.subscription_requests.try_lock() {
            Ok(subs) => subs.clone(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Lighter-specific subscriptions)
// ═══════════════════════════════════════════════════════════════════════════════

impl LighterWebSocket {
    /// Subscribe to order book for a market
    pub async fn subscribe_orderbook(&self, market_id: u16) -> WebSocketResult<()> {
        let channel = format!("order_book/{}", market_id);
        self.subscribe_channel(&channel, None).await
    }

    /// Subscribe to trades for a market
    pub async fn subscribe_trades(&self, market_id: u16) -> WebSocketResult<()> {
        let channel = format!("trade/{}", market_id);
        self.subscribe_channel(&channel, None).await
    }

    /// Subscribe to market stats
    pub async fn subscribe_market_stats(&self, market_id: u16) -> WebSocketResult<()> {
        let channel = format!("market_stats/{}", market_id);
        self.subscribe_channel(&channel, None).await
    }

    /// Subscribe to account data (public - no auth required)
    pub async fn subscribe_account(&self, account_id: u64) -> WebSocketResult<()> {
        let channel = format!("account_all/{}", account_id);
        self.subscribe_channel(&channel, None).await
    }

    /// Subscribe to blockchain height updates
    pub async fn subscribe_height(&self) -> WebSocketResult<()> {
        self.subscribe_channel("height", None).await
    }
}
