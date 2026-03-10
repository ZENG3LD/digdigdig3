//! # Bitfinex WebSocket Implementation
//!
//! WebSocket connector for Bitfinex API v2.
//!
//! ## Features
//! - Public and private channels
//! - Subscription management
//! - Message parsing to StreamEvent
//! - Automatic heartbeat handling
//! - WebSocket-level ping/pong response
//! - Application-level periodic ping for stale connection detection
//! - Info code handling (20051 reconnect, 20060/20061 maintenance)
//!
//! ## Bitfinex WebSocket Protocol
//! - Array-based messages (not JSON objects)
//! - Channel IDs for subscriptions
//! - Heartbeat messages every 15s: `[CHAN_ID, "hb"]`
//! - Event codes for errors and server status
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = BitfinexWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe_ticker(Symbol::new("BTC", "USD")).await?;
//!
//! let stream = ws.event_stream();
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(StreamEvent::Ticker(ticker)) => println!("{:?}", ticker),
//!         _ => {}
//!     }
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_millis,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::SimpleRateLimiter;

use super::auth::BitfinexAuth;
use super::endpoints::{BitfinexUrls, format_symbol};
use super::parser::BitfinexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET URLS
// ═══════════════════════════════════════════════════════════════════════════════

const WS_PUBLIC_URL: &str = "wss://api-pub.bitfinex.com/ws/2";
const WS_PRIVATE_URL: &str = "wss://api.bitfinex.com/ws/2";

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL RATE LIMITERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Global rate limiter for public WebSocket connections (20 per minute)
static GLOBAL_WS_PUBLIC_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

/// Global rate limiter for private WebSocket connections (5 per 15 seconds)
static GLOBAL_WS_PRIVATE_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

/// Get or initialize global public WebSocket rate limiter
fn get_global_ws_public_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_PUBLIC_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(SimpleRateLimiter::new(20, Duration::from_secs(60))))
    }).clone()
}

/// Get or initialize global private WebSocket rate limiter
fn get_global_ws_private_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_PRIVATE_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(SimpleRateLimiter::new(5, Duration::from_secs(15))))
    }).clone()
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing subscription message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    event: String,
    channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
}

/// Outgoing unsubscribe message
#[derive(Debug, Clone, Serialize)]
struct UnsubscribeMessage {
    event: String,
    #[serde(rename = "chanId")]
    chan_id: u64,
}

/// Authentication message
#[derive(Debug, Clone, Serialize)]
struct AuthMessage {
    event: String,
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(rename = "authSig")]
    auth_sig: String,
    #[serde(rename = "authPayload")]
    auth_payload: String,
    #[serde(rename = "authNonce")]
    auth_nonce: String,
}

/// Info message from Bitfinex
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct InfoMessage {
    event: String,
    version: Option<u32>,
    platform: Option<Value>,
}

/// Subscription response
#[derive(Debug, Clone, Deserialize)]
struct SubscriptionResponse {
    #[allow(dead_code)]
    event: String,
    #[serde(rename = "chanId")]
    chan_id: u64,
    channel: String,
    #[serde(rename = "symbol")]
    symbol: Option<String>,
    #[serde(rename = "key")]
    key: Option<String>,
}

/// Error message
#[derive(Debug, Clone, Deserialize)]
struct ErrorMessage {
    #[allow(dead_code)]
    event: String,
    msg: String,
    code: i32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CHANNEL TRACKING
// ═══════════════════════════════════════════════════════════════════════════════

/// Maps channel IDs to subscription requests
type ChannelMap = HashMap<u64, SubscriptionRequest>;

/// Pending subscriptions (waiting for channel ID assignment)
type PendingSubscriptions = HashMap<String, SubscriptionRequest>;

// ═══════════════════════════════════════════════════════════════════════════════
// BITFINEX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Bitfinex WebSocket connector
pub struct BitfinexWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<BitfinexAuth>,
    /// URLs (mainnet only for now)
    _urls: BitfinexUrls,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Channel ID mapping
    channels: Arc<Mutex<ChannelMap>>,
    /// Pending subscriptions (not yet assigned channel ID)
    pending_subs: Arc<Mutex<PendingSubscriptions>>,
    /// Event sender (internal - for message handler)
    event_tx: Arc<Mutex<Option<mpsc::UnboundedSender<WebSocketResult<StreamEvent>>>>>,
    /// Broadcast sender (for multiple consumers, dropped on disconnect)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket stream (used only during connect, before message handler takes it)
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Write command channel — used by subscribe/unsubscribe to send messages
    /// through the message handler task which owns the write half
    write_tx: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
    /// Is authenticated
    is_authenticated: Arc<Mutex<bool>>,
    /// Timestamp of the most recently sent application-level ping.
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl BitfinexWebSocket {
    /// Create new Bitfinex WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        _testnet: bool, // Bitfinex doesn't have testnet
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = BitfinexUrls::MAINNET;

        let auth = credentials
            .as_ref()
            .map(BitfinexAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            _urls: urls,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
            pending_subs: Arc::new(Mutex::new(HashMap::new())),
            event_tx: Arc::new(Mutex::new(None)),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_stream: Arc::new(Mutex::new(None)),
            write_tx: Arc::new(Mutex::new(None)),
            is_authenticated: Arc::new(Mutex::new(false)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Get WebSocket URL (public or private)
    fn ws_url(&self) -> &'static str {
        if self.auth.is_some() {
            WS_PRIVATE_URL
        } else {
            WS_PUBLIC_URL
        }
    }

    /// Connect to WebSocket
    async fn connect_ws(&self) -> ExchangeResult<WsStream> {
        let ws_url = self.ws_url();

        // Apply rate limiting based on connection type
        let limiter = if self.auth.is_some() {
            get_global_ws_private_limiter()
        } else {
            get_global_ws_public_limiter()
        };

        let wait_time = {
            let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
            if !limiter_guard.try_acquire() {
                limiter_guard.time_until_ready()
            } else {
                Duration::ZERO
            }
        };

        if !wait_time.is_zero() {
            tokio::time::sleep(wait_time).await;
            // Try again after waiting
            let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
            limiter_guard.try_acquire();
        }

        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Authenticate WebSocket connection
    async fn authenticate(&self, stream: &mut WsStream) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("No credentials provided".to_string()))?;

        let nonce = timestamp_millis().to_string();
        let auth_payload = format!("AUTH{}", nonce);

        // Sign authentication payload
        let signature = auth.sign_auth(&auth_payload);

        let auth_msg = AuthMessage {
            event: "auth".to_string(),
            api_key: auth.api_key().to_string(),
            auth_sig: signature,
            auth_payload,
            auth_nonce: nonce,
        };

        let msg_json = serde_json::to_string(&auth_msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize auth message: {}", e)))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send auth message: {}", e)))?;

        Ok(())
    }

    /// Start message handling task.
    ///
    /// Splits the WebSocket into read/write halves so we can:
    /// 1. Respond to WebSocket-level Ping frames with Pong
    /// 2. Send periodic application-level pings to detect stale connections
    /// 3. Process write commands from subscribe/unsubscribe without lock contention
    #[allow(clippy::too_many_arguments)]
    fn start_message_handler(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        mut write_rx: mpsc::UnboundedReceiver<Message>,
        status: Arc<Mutex<ConnectionStatus>>,
        channels: Arc<Mutex<ChannelMap>>,
        pending_subs: Arc<Mutex<PendingSubscriptions>>,
        is_authenticated: Arc<Mutex<bool>>,
        account_type: AccountType,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            // Take the stream out of the Arc<Mutex<Option<...>>> and split it
            let stream = {
                let mut guard = ws_stream.lock().await;
                match guard.take() {
                    Some(s) => s,
                    None => return,
                }
            };

            let (mut write, mut read) = stream.split();

            // Periodic ping interval (every 5 seconds for RTT measurement)
            let mut ping_timer = tokio::time::interval(Duration::from_secs(5));
            // Skip the first immediate tick
            ping_timer.tick().await;

            loop {
                tokio::select! {
                    // Incoming messages from exchange
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Err(e) = Self::handle_message(
                                    &text,
                                    &event_tx,
                                    &channels,
                                    &pending_subs,
                                    &is_authenticated,
                                    account_type,
                                    &last_ping,
                                    &ws_ping_rtt_ms,
                                ).await {
                                    let _ = event_tx.send(Err(e));
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                // Respond to WebSocket-level Ping with Pong
                                let _ = write.send(Message::Pong(data)).await;
                            }
                            Some(Ok(Message::Pong(_))) => {
                                // WS-frame pong — connection alive (Bitfinex ping/pong is JSON)
                            }
                            Some(Ok(Message::Close(_))) => {
                                *status.lock().await = ConnectionStatus::Disconnected;
                                break;
                            }
                            Some(Err(e)) => {
                                let _ = event_tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                                *status.lock().await = ConnectionStatus::Disconnected;
                                break;
                            }
                            None => {
                                *status.lock().await = ConnectionStatus::Disconnected;
                                break;
                            }
                            _ => {
                                // Binary or Frame — ignore
                            }
                        }
                    }
                    // Write commands from subscribe/unsubscribe
                    Some(msg) = write_rx.recv() => {
                        if write.send(msg).await.is_err() {
                            *status.lock().await = ConnectionStatus::Disconnected;
                            break;
                        }
                    }
                    // Periodic application-level ping for RTT measurement
                    _ = ping_timer.tick() => {
                        // Send Bitfinex-specific ping: {"event":"ping","cid":12345}
                        let cid = timestamp_millis();
                        let ping_msg = format!(r#"{{"event":"ping","cid":{}}}"#, cid);
                        // Record ping send time before sending
                        *last_ping.lock().await = Instant::now();
                        if write.send(Message::Text(ping_msg)).await.is_err() {
                            *status.lock().await = ConnectionStatus::Disconnected;
                            break;
                        }
                    }
                }
            }
        });
    }

    /// Handle incoming WebSocket message
    #[allow(clippy::too_many_arguments)]
    async fn handle_message(
        text: &str,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        channels: &Arc<Mutex<ChannelMap>>,
        pending_subs: &Arc<Mutex<PendingSubscriptions>>,
        is_authenticated: &Arc<Mutex<bool>>,
        account_type: AccountType,
        last_ping: &Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: &Arc<Mutex<u64>>,
    ) -> WebSocketResult<()> {
        // Try parsing as JSON value first
        let value: Value = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Bitfinex sends either objects or arrays
        if value.is_object() {
            // Event message (info, subscribed, error, auth, pong)
            Self::handle_event_message(&value, channels, pending_subs, is_authenticated, last_ping, ws_ping_rtt_ms).await?;
        } else if value.is_array() {
            // Data message [CHANNEL_ID, ...data]
            Self::handle_data_message(&value, event_tx, channels, account_type).await?;
        }

        Ok(())
    }

    /// Handle event messages (info, subscribed, error, auth, pong)
    async fn handle_event_message(
        value: &Value,
        channels: &Arc<Mutex<ChannelMap>>,
        pending_subs: &Arc<Mutex<PendingSubscriptions>>,
        is_authenticated: &Arc<Mutex<bool>>,
        last_ping: &Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: &Arc<Mutex<u64>>,
    ) -> WebSocketResult<()> {
        let event = value.get("event")
            .and_then(|e| e.as_str())
            .unwrap_or("");

        match event {
            "info" => {
                // Info message — check for server status codes
                // 20051: Server requests reconnection
                // 20060: Entering maintenance mode
                // 20061: Maintenance ended
                if let Some(code) = value.get("code").and_then(|c| c.as_u64()) {
                    match code {
                        20051 => {
                            return Err(WebSocketError::ConnectionError(
                                "Bitfinex requested reconnect (code 20051)".to_string()
                            ));
                        }
                        20060 => {
                            return Err(WebSocketError::ConnectionError(
                                "Bitfinex entering maintenance (code 20060)".to_string()
                            ));
                        }
                        20061 => {
                            return Err(WebSocketError::ConnectionError(
                                "Bitfinex maintenance ended, reconnect needed (code 20061)".to_string()
                            ));
                        }
                        _ => {}
                    }
                }
                Ok(())
            }
            "subscribed" => {
                // Subscription confirmed - store channel ID
                let sub_resp: SubscriptionResponse = serde_json::from_value(value.clone())
                    .map_err(|e| WebSocketError::Parse(format!("Failed to parse subscribed: {}", e)))?;

                let chan_id = sub_resp.chan_id;
                let channel = &sub_resp.channel;

                // Build key to lookup pending subscription
                let symbol_or_key = sub_resp.symbol.as_deref()
                    .or(sub_resp.key.as_deref())
                    .unwrap_or("");
                let key = format!("{}:{}", channel, symbol_or_key);

                // Find and remove matching pending subscription
                let mut pending = pending_subs.lock().await;
                if let Some(request) = pending.remove(&key) {
                    // Map channel ID to subscription request
                    channels.lock().await.insert(chan_id, request);
                }
                drop(pending);

                Ok(())
            }
            "unsubscribed" => {
                // Unsubscription confirmed
                Ok(())
            }
            "auth" => {
                // Authentication response
                let status = value.get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("FAILED");

                if status == "OK" {
                    *is_authenticated.lock().await = true;
                    Ok(())
                } else {
                    Err(WebSocketError::Auth("Authentication failed".to_string()))
                }
            }
            "pong" => {
                // Response to our application-level ping — measure RTT
                let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                *ws_ping_rtt_ms.lock().await = rtt;
                Ok(())
            }
            "error" => {
                // Error message
                let err_msg: ErrorMessage = serde_json::from_value(value.clone())
                    .map_err(|e| WebSocketError::Parse(format!("Failed to parse error: {}", e)))?;
                Err(WebSocketError::ProtocolError(format!("Code {}: {}", err_msg.code, err_msg.msg)))
            }
            _ => {
                // Unknown event
                Ok(())
            }
        }
    }

    /// Handle data messages [CHANNEL_ID, ...data]
    async fn handle_data_message(
        value: &Value,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        channels: &Arc<Mutex<ChannelMap>>,
        account_type: AccountType,
    ) -> WebSocketResult<()> {
        let arr = value.as_array()
            .ok_or_else(|| WebSocketError::Parse("Expected array".to_string()))?;

        if arr.is_empty() {
            return Ok(());
        }

        // First element is channel ID
        let chan_id = arr[0].as_u64()
            .ok_or_else(|| WebSocketError::Parse("Invalid channel ID".to_string()))?;

        // Heartbeat messages: [CHANNEL_ID, "hb"]
        if arr.len() == 2 && arr[1].as_str() == Some("hb") {
            return Ok(()); // Ignore heartbeats
        }

        // Get subscription type from channel map
        let channels_guard = channels.lock().await;
        let subscription = channels_guard.get(&chan_id);
        if let Some(sub) = subscription {
            if let Some(event) = Self::parse_channel_data(arr, sub, account_type)? {
                let _ = event_tx.send(Ok(event));
            }
        }
        drop(channels_guard);

        Ok(())
    }

    /// Parse channel data based on subscription type
    fn parse_channel_data(
        arr: &[Value],
        subscription: &SubscriptionRequest,
        _account_type: AccountType,
    ) -> WebSocketResult<Option<StreamEvent>> {
        if arr.len() < 2 {
            return Ok(None);
        }

        match &subscription.stream_type {
            StreamType::Ticker => {
                // Ticker: [CHANNEL_ID, [BID, BID_SIZE, ASK, ASK_SIZE, ...]]
                if let Some(data) = arr[1].as_array() {
                    let ticker = BitfinexParser::parse_ws_ticker(data)
                        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                    Ok(Some(StreamEvent::Ticker(ticker)))
                } else {
                    Ok(None)
                }
            }
            StreamType::Trade => {
                // Trades: [CHANNEL_ID, "te", [ID, MTS, AMOUNT, PRICE]]
                // or [CHANNEL_ID, [[ID, MTS, AMOUNT, PRICE], ...]]
                if arr.len() >= 3 && arr[1].as_str() == Some("te") {
                    if let Some(data) = arr[2].as_array() {
                        let trade = BitfinexParser::parse_ws_trade(data)
                            .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                        Ok(Some(StreamEvent::Trade(trade)))
                    } else {
                        Ok(None)
                    }
                } else if let Some(data) = arr[1].as_array() {
                    // Snapshot
                    if let Some(first) = data.first() {
                        if first.is_array() {
                            // Multiple trades
                            Ok(None) // Skip snapshots for now
                        } else {
                            let trade = BitfinexParser::parse_ws_trade(data)
                                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                            Ok(Some(StreamEvent::Trade(trade)))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                // Orderbook: [CHANNEL_ID, [[PRICE, COUNT, AMOUNT], ...]]
                if let Some(data) = arr[1].as_array() {
                    let event = BitfinexParser::parse_ws_orderbook_delta(data)
                        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                    Ok(Some(event))
                } else {
                    Ok(None)
                }
            }
            StreamType::Kline { .. } => {
                // Candles: [CHANNEL_ID, [MTS, OPEN, CLOSE, HIGH, LOW, VOLUME]]
                if let Some(data) = arr[1].as_array() {
                    let kline = BitfinexParser::parse_ws_kline(data)
                        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                    Ok(Some(StreamEvent::Kline(kline)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Build channel name for subscription
    fn build_channel(request: &SubscriptionRequest, account_type: AccountType) -> (String, Option<String>) {
        let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        match &request.stream_type {
            StreamType::Ticker => ("ticker".to_string(), Some(symbol)),
            StreamType::Trade => ("trades".to_string(), Some(symbol)),
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                ("book".to_string(), Some(symbol))
            }
            StreamType::Kline { interval } => {
                // Candles use "key" instead of "symbol"
                let key = format!("trade:{}:{}", interval, symbol);
                ("candles".to_string(), Some(key))
            }
            _ => ("".to_string(), None),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for BitfinexWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Connect WebSocket
        let mut ws_stream = self.connect_ws().await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Authenticate if credentials provided
        if self.auth.is_some() {
            self.authenticate(&mut ws_stream).await
                .map_err(|e| WebSocketError::Auth(e.to_string()))?;
        }

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Create event channel
        let (tx, mut rx) = mpsc::unbounded_channel();
        *self.event_tx.lock().await = Some(tx.clone());

        // Create write command channel (for subscribe/unsubscribe to send messages)
        let (write_cmd_tx, write_cmd_rx) = mpsc::unbounded_channel();
        *self.write_tx.lock().await = Some(write_cmd_tx);

        // Start message handler (takes ownership of ws_stream and write_cmd_rx)
        Self::start_message_handler(
            self.ws_stream.clone(),
            tx,
            write_cmd_rx,
            self.status.clone(),
            self.channels.clone(),
            self.pending_subs.clone(),
            self.is_authenticated.clone(),
            account_type,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Create broadcast channel and store
        let (broadcast_sender, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(broadcast_sender);

        // Start forwarder task (mpsc -> broadcast)
        let broadcast_tx = self.broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Forward to broadcast channel (ignore if no receivers)
                if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                    let _ = tx.send(event);
                }
            }
            // mpsc channel closed — drop broadcast sender
            let _ = broadcast_tx.lock().unwrap().take();
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_stream.lock().await = None;
        *self.event_tx.lock().await = None;
        *self.write_tx.lock().await = None; // Drop write channel, stopping the message handler
        let _ = self.broadcast_tx.lock().unwrap().take();
        self.subscriptions.lock().await.clear();
        self.channels.lock().await.clear();
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
        let (channel, symbol_or_key) = Self::build_channel(&request, self.account_type);

        if channel.is_empty() {
            return Err(WebSocketError::ProtocolError("Unsupported stream type".to_string()));
        }

        // Build key for pending subscription tracking
        let symbol_or_key_str = symbol_or_key.as_deref().unwrap_or("");
        let pending_key = format!("{}:{}", channel, symbol_or_key_str);

        // Store pending subscription before sending
        self.pending_subs.lock().await.insert(pending_key, request.clone());

        let msg = if channel == "candles" {
            SubscribeMessage {
                event: "subscribe".to_string(),
                channel,
                symbol: None,
                key: symbol_or_key,
            }
        } else {
            SubscribeMessage {
                event: "subscribe".to_string(),
                channel,
                symbol: symbol_or_key,
                key: None,
            }
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        // Send via write command channel (message handler owns the write half)
        let write_tx_guard = self.write_tx.lock().await;
        let tx = write_tx_guard.as_ref()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        tx.send(Message::Text(msg_json))
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(write_tx_guard);

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Find channel ID for this subscription
        let channels_guard = self.channels.lock().await;
        let chan_id = channels_guard.iter()
            .find(|(_, sub)| *sub == &request)
            .map(|(id, _)| *id);

        drop(channels_guard);

        if let Some(chan_id) = chan_id {
            let msg = UnsubscribeMessage {
                event: "unsubscribe".to_string(),
                chan_id,
            };

            let msg_json = serde_json::to_string(&msg)
                .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

            // Send via write command channel
            let write_tx_guard = self.write_tx.lock().await;
            let tx = write_tx_guard.as_ref()
                .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

            tx.send(Message::Text(msg_json))
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

            drop(write_tx_guard);

            self.subscriptions.lock().await.remove(&request);
            self.channels.lock().await.remove(&chan_id);
        }

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1);

        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
            match result {
                Ok(event) => Some(event),
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
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

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}
