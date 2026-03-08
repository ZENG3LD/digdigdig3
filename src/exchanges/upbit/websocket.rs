//! # Upbit WebSocket Implementation
//!
//! WebSocket connector for Upbit.
//!
//! ## Features
//! - Public channels (ticker, orderbook, trade)
//! - Private channels (myAsset, myOrder)
//! - Ping/pong heartbeat (PING message every 30-60 seconds)
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = UpbitWebSocket::new(None, "sg").await?;
//! ws.connect().await?;
//! ws.subscribe_ticker(Symbol::new("BTC", "SGD")).await?;
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
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::SimpleRateLimiter;

use super::auth::UpbitAuth;
use super::endpoints::UpbitUrls;
use super::parser::UpbitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Global rate limiter for Upbit WebSocket connections
/// Upbit has strict rate limits: conservative approach with 10 connections per 10 seconds
static GLOBAL_WS_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

fn get_global_ws_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            // Very conservative: 5 connections per 10 seconds to avoid 429 errors
            // Upbit has strict rate limits
            SimpleRateLimiter::new(5, Duration::from_secs(10))
        ))
    }).clone()
}

/// Global rate limiter for subscriptions (separate from connections)
static SUBSCRIPTION_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

fn get_subscription_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    SUBSCRIPTION_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            // Conservative: 3 subscriptions per second
            SimpleRateLimiter::new(3, Duration::from_secs(1))
        ))
    }).clone()
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscription message format for Upbit
/// Format: [ticket_object, type_object(s), format_object?]
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct SubscriptionMessage {
    ticket: String,
    #[serde(rename = "type")]
    msg_type: String,
    codes: Vec<String>,
}

/// Incoming message from Upbit
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    code: Option<String>,
    status: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UPBIT WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Upbit WebSocket connector
pub struct UpbitWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<UpbitAuth>,
    /// URLs (region-specific)
    urls: UpbitUrls,
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
    /// Last ping time (updated when WS-frame ping is sent)
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl UpbitWebSocket {
    /// Create new Upbit WebSocket connector
    /// region: "sg" (Singapore), "id" (Indonesia), "th" (Thailand)
    pub async fn new(
        credentials: Option<Credentials>,
        region: &str,
    ) -> ExchangeResult<Self> {
        let urls = match region {
            "id" => UpbitUrls::INDONESIA,
            "th" => UpbitUrls::THAILAND,
            _ => UpbitUrls::SINGAPORE,
        };

        let auth = credentials
            .as_ref()
            .map(UpbitAuth::new)
            .transpose()?;

        // Create broadcast channel (capacity of 1000 events)
        let (broadcast_tx, _) = broadcast::channel(1000);

        Ok(Self {
            auth,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(Mutex::new(None)),
            broadcast_tx: Arc::new(broadcast_tx),
            ws_stream: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Wait for subscription rate limit if needed
    async fn subscription_rate_limit_wait() {
        loop {
            // Scope the lock to ensure it's dropped before await
            let can_subscribe = {
                let limiter = get_subscription_limiter();
                let mut guard = limiter.lock().expect("Mutex poisoned");
                guard.try_acquire()
            };

            if can_subscribe {
                return; // Successfully acquired, proceed
            }

            // Get wait time
            let wait_time = {
                let limiter = get_subscription_limiter();
                let guard = limiter.lock().expect("Mutex poisoned");
                guard.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                sleep(wait_time).await;
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    /// Subscribe to channels
    async fn send_subscription(
        &self,
        msg_type: &str,
        symbols: Vec<String>,
    ) -> WebSocketResult<()> {
        // Wait for rate limit
        Self::subscription_rate_limit_wait().await;

        let mut ws_lock = self.ws_stream.lock().await;
        let ws = ws_lock.as_mut()
            .ok_or(WebSocketError::NotConnected)?;

        // Upbit subscription format: [ticket, type, format?]
        let subscription = json!([
            {"ticket": "upbit-connector"},
            {
                "type": msg_type,
                "codes": symbols
            },
            {"format": "DEFAULT"}
        ]);

        ws.send(Message::Text(subscription.to_string())).await
            .map_err(|e| WebSocketError::SendError(e.to_string()))?;

        Ok(())
    }

    /// Send PING message to keep connection alive and measure RTT.
    ///
    /// Sends both a text "PING" (Upbit application-level keepalive) and a
    /// WS-frame `Message::Ping` (for transport-level RTT measurement).
    async fn send_ping(&self) -> WebSocketResult<()> {
        let mut ws_lock = self.ws_stream.lock().await;
        let ws = ws_lock.as_mut()
            .ok_or(WebSocketError::NotConnected)?;

        // Upbit accepts text "PING" message (application-level keepalive)
        ws.send(Message::Text("PING".to_string())).await
            .map_err(|e| WebSocketError::SendError(e.to_string()))?;

        // Also send a WS-frame ping for RTT measurement (server responds with Pong)
        *self.last_ping.lock().await = Instant::now();
        ws.send(Message::Ping(vec![])).await
            .map_err(|e| WebSocketError::SendError(e.to_string()))?;

        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(&self, text: &str) -> Option<StreamEvent> {
        // Try to parse as JSON
        let value: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return None,
        };

        // Check for status message (server ping response)
        if let Some(status) = value.get("status") {
            if status == "UP" {
                return None; // Ignore status messages
            }
        }

        // Parse based on message type
        let msg_type = value.get("type")
            .or_else(|| value.get("ty"))
            .and_then(|t| t.as_str())?;

        match msg_type {
            "ticker" => {
                UpbitParser::parse_ws_ticker(&value)
                    .ok()
                    .map(StreamEvent::Ticker)
            },
            "trade" => {
                UpbitParser::parse_ws_trade(&value)
                    .ok()
                    .map(StreamEvent::Trade)
            },
            "orderbook" => {
                UpbitParser::parse_ws_orderbook(&value)
                    .ok()
                    .map(StreamEvent::OrderbookSnapshot)
            },
            _ => None,
        }
    }

    /// Start message receiving loop
    async fn start_message_loop(&self) -> WebSocketResult<()> {
        let ws_stream = self.ws_stream.clone();
        let broadcast_tx = self.broadcast_tx.clone();
        let status = self.status.clone();
        let last_ping = self.last_ping.clone();
        let ws_ping_rtt_ms = self.ws_ping_rtt_ms.clone();
        let ws_clone = self.clone_for_loop();

        tokio::spawn(async move {
            loop {
                // Check if should ping (every 30 seconds)
                {
                    let last = last_ping.lock().await;
                    if last.elapsed() > Duration::from_secs(30) {
                        drop(last);
                        if let Err(e) = ws_clone.send_ping().await {
                            let _ = broadcast_tx.send(Err(e));
                            break;
                        }
                    }
                }

                // Read next message
                let msg = {
                    let mut ws_lock = ws_stream.lock().await;
                    match ws_lock.as_mut() {
                        Some(ws) => {
                            match tokio::time::timeout(Duration::from_millis(100), ws.next()).await {
                                Ok(Some(msg)) => msg,
                                Ok(None) => break,
                                Err(_) => continue, // Timeout, continue loop
                            }
                        },
                        None => break,
                    }
                };

                // Process message
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Some(event) = ws_clone.handle_message(&text).await {
                            let _ = broadcast_tx.send(Ok(event));
                        }
                    },
                    Ok(Message::Binary(data)) => {
                        // Upbit sends data as binary (compressed JSON)
                        // Try to decompress and parse
                        if let Ok(text) = String::from_utf8(data) {
                            if let Some(event) = ws_clone.handle_message(&text).await {
                                let _ = broadcast_tx.send(Ok(event));
                            }
                        }
                    },
                    Ok(Message::Pong(_)) => {
                        // Measure RTT from our last WS-frame ping.
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    },
                    Ok(Message::Close(_)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    },
                    Err(e) => {
                        let _ = broadcast_tx.send(Err(WebSocketError::ReceiveError(e.to_string())));
                        break;
                    },
                    _ => {},
                }
            }
        });

        Ok(())
    }

    /// Clone self for message loop (workaround for async move)
    fn clone_for_loop(&self) -> Self {
        Self {
            auth: self.auth.clone(),
            urls: self.urls.clone(),
            status: self.status.clone(),
            subscriptions: self.subscriptions.clone(),
            event_tx: self.event_tx.clone(),
            broadcast_tx: self.broadcast_tx.clone(),
            ws_stream: self.ws_stream.clone(),
            last_ping: self.last_ping.clone(),
            ws_ping_rtt_ms: self.ws_ping_rtt_ms.clone(),
        }
    }
}

#[async_trait]
impl WebSocketConnector for UpbitWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Rate limit WebSocket connections to avoid 429 errors
        let limiter = get_global_ws_limiter();
        loop {
            let can_connect = {
                let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
                limiter_guard.try_acquire()
            };

            if can_connect {
                break;
            }

            // Wait before retrying
            let wait_time = {
                let limiter_guard = limiter.lock().expect("Mutex poisoned");
                limiter_guard.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                sleep(wait_time).await;
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }

        let ws_url = if self.auth.is_some() {
            self.urls.ws_private_url()
        } else {
            self.urls.ws_url().to_string()
        };

        // For private WebSocket, need to add JWT token as header
        // Note: tokio-tungstenite doesn't support custom headers easily
        // This is a simplified implementation - full version would need custom headers
        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.status.lock().await = ConnectionStatus::Connected;
        *self.last_ping.lock().await = Instant::now();

        // Start message receiving loop
        self.start_message_loop().await?;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        let mut ws_lock = self.ws_stream.lock().await;
        if let Some(ws) = ws_lock.as_mut() {
            ws.close(None).await
                .map_err(|e: tokio_tungstenite::tungstenite::Error| WebSocketError::ConnectionError(e.to_string()))?;
        }
        *ws_lock = None;
        *self.status.lock().await = ConnectionStatus::Disconnected;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // We need to block on the async lock - use try_lock or return a default
        self.status.try_lock()
            .map(|s| *s)
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions.lock().await.insert(request.clone());

        // Upbit symbol format is QUOTE-BASE (e.g. "USDT-BTC").
        // The bridge's generic parse_symbol("usdt-btc") produces Symbol { base: "usdt", quote: "btc" }
        // where the first segment (the Upbit quote currency) is in `base` and the second
        // (the Upbit base currency) is in `quote`. To reconstruct the original Upbit code
        // we just concatenate them back in order: base-quote (both uppercased).
        // This is the inverse of what format_symbol does (which would reverse them again).
        let upbit_symbol = format!("{}-{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());

        match request.stream_type {
            StreamType::Ticker => {
                self.send_subscription("ticker", vec![upbit_symbol]).await?;
            },
            StreamType::Trade => {
                self.send_subscription("trade", vec![upbit_symbol]).await?;
            },
            StreamType::Orderbook => {
                self.send_subscription("orderbook", vec![upbit_symbol]).await?;
            },
            _ => {
                return Err(WebSocketError::UnsupportedOperation(format!("Unsupported stream type: {:?}", request.stream_type)));
            }
        }

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Upbit doesn't support unsubscribe - need to reconnect
        self.subscriptions.lock().await.remove(&request);
        Err(WebSocketError::UnsupportedOperation("Upbit doesn't support unsubscribe - reconnect required".to_string()))
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.subscribe();
        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|r| async move {
            r.ok()
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.subscriptions.try_lock()
            .map(|subs| subs.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}
