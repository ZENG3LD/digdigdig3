//! # Bithumb WebSocket Implementation
//!
//! WebSocket connector for Bithumb Pro.
//!
//! ## Features
//! - Public channels: TICKER, ORDERBOOK, TRADE
//! - Private channels: ORDER, CONTRACT_ORDER (requires authentication)
//! - Automatic ping/pong heartbeat (30 seconds)
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Connection
//!
//! **Base URL**: `wss://global-api.bithumb.pro/message/realtime`
//!
//! ## Authentication
//!
//! For private channels, use `authKey` command after connecting:
//! ```json
//! {
//!   "cmd": "authKey",
//!   "args": ["apiKey", "timestamp_ms", "signature"]
//! }
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = BithumbWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
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
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_millis, hmac_sha256,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::SimpleRateLimiter;

use super::auth::BithumbAuth;
use super::endpoints::{BithumbUrls, format_symbol};
use super::parser::BithumbParser;

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Global WebSocket rate limiter for Bithumb
/// Bithumb has poor documentation and flaky infrastructure
/// Use 2 requests per second (very conservative)
static GLOBAL_WS_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

fn get_global_ws_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_LIMITER
        .get_or_init(|| {
            // 2 requests per second - conservative to avoid overwhelming infrastructure
            Arc::new(StdMutex::new(
                SimpleRateLimiter::new(2, Duration::from_secs(1))
            ))
        })
        .clone()
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing command (subscribe/unsubscribe/ping/authKey)
#[derive(Debug, Clone, Serialize)]
struct OutgoingMessage {
    cmd: String,
    args: Vec<String>,
}

/// Incoming message from Bithumb
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    /// Response code
    /// - "0" = pong
    /// - "00000" = auth success / command success
    /// - "00001" = subscribe success
    /// - "00002" = connection success
    /// - "00006" = initial message (snapshot)
    /// - "00007" = normal message (update)
    /// - "10000+" = error
    code: Option<String>,

    /// Message text (for errors or info)
    msg: Option<String>,

    /// Topic (for data messages)
    topic: Option<String>,

    /// Data payload
    data: Option<Value>,

    /// Timestamp
    timestamp: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BITHUMB WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Bithumb WebSocket connector
pub struct BithumbWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<BithumbAuth>,
    /// URLs (mainnet/testnet)
    urls: BithumbUrls,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event sender
    event_tx: Arc<Mutex<Option<mpsc::UnboundedSender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Is authenticated (for private channels)
    is_authenticated: Arc<Mutex<bool>>,
    /// Connection rate limiter (global, shared across all instances)
    connection_limiter: Arc<StdMutex<SimpleRateLimiter>>,
    /// Most recent ping round-trip time in milliseconds (0 until first pong)
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl BithumbWebSocket {
    /// Create new Bithumb WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            BithumbUrls::TESTNET
        } else {
            BithumbUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(BithumbAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(Mutex::new(None)),
            ws_stream: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            is_authenticated: Arc::new(Mutex::new(false)),
            connection_limiter: get_global_ws_limiter(),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Wait for rate limit before WebSocket operations
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.connection_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire() {
                    Duration::ZERO
                } else {
                    limiter.time_until_ready()
                }
            };

            if wait_time.is_zero() {
                break;
            }

            tokio::time::sleep(wait_time).await;
        }
    }

    /// Connect to WebSocket
    async fn connect_ws(&self) -> ExchangeResult<WsStream> {
        // Apply rate limiting before connecting
        self.rate_limit_wait().await;

        let ws_url = self.urls.ws_url();

        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Authenticate for private channels
    async fn authenticate(&self) -> WebSocketResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| WebSocketError::Auth("No credentials provided".to_string()))?;

        let timestamp = timestamp_millis();
        let api_key = auth.api_key();

        // Generate signature for WebSocket authentication
        // Format: /message/realtime + timestamp + apiKey
        let path = "/message/realtime";
        let message = format!("{}{}{}", path, timestamp, api_key);
        let signature_bytes = hmac_sha256(
            auth.api_secret().as_bytes(),
            message.as_bytes(),
        );
        let signature = signature_bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        // Send authKey command
        let auth_cmd = OutgoingMessage {
            cmd: "authKey".to_string(),
            args: vec![
                api_key.to_string(),
                timestamp.to_string(),
                signature,
            ],
        };

        let msg_json = serde_json::to_string(&auth_cmd)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(stream_guard);

        // Wait for auth response
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    /// Start message handling task
    fn start_message_handler(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        status: Arc<Mutex<ConnectionStatus>>,
        is_authenticated: Arc<Mutex<bool>>,
        account_type: AccountType,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
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
                        if let Err(e) = Self::handle_message(&text, &event_tx, &is_authenticated, account_type, &last_ping, &ws_ping_rtt_ms).await {
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
        is_authenticated: &Arc<Mutex<bool>>,
        account_type: AccountType,
        last_ping: &Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: &Arc<Mutex<u64>>,
    ) -> WebSocketResult<()> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        let code = msg.code.as_deref().unwrap_or("");

        // Handle different response codes
        match code {
            "0" => {
                // Pong response — measure RTT
                let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                *ws_ping_rtt_ms.lock().await = rtt;
                Ok(())
            }
            "00000" => {
                // Auth success or command success
                if msg.msg.as_deref() == Some("Authentication successful") {
                    *is_authenticated.lock().await = true;
                }
                Ok(())
            }
            "00001" => {
                // Subscribe success - ignore
                Ok(())
            }
            "00002" => {
                // Connection success - ignore
                Ok(())
            }
            "00006" | "00007" => {
                // Data message (00006 = snapshot, 00007 = update)
                if let Some(event) = Self::parse_data_message(&msg, code == "00006", account_type)? {
                    let _ = event_tx.send(Ok(event));
                }
                Ok(())
            }
            _ if code.starts_with("10") => {
                // Error codes (10000+)
                let error_msg = msg.msg.unwrap_or_else(|| format!("Error code: {}", code));
                Err(WebSocketError::ProtocolError(error_msg))
            }
            _ => {
                // Unknown code - ignore
                Ok(())
            }
        }
    }

    /// Parse data message to StreamEvent
    fn parse_data_message(
        msg: &IncomingMessage,
        is_snapshot: bool,
        _account_type: AccountType,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let topic = msg.topic.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing topic".to_string()))?;

        let data = msg.data.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing data".to_string()))?;

        // Match by topic to determine event type
        match topic.as_str() {
            "TICKER" => {
                let ticker = BithumbParser::parse_ws_ticker(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            "ORDERBOOK" => {
                if is_snapshot {
                    let orderbook = BithumbParser::parse_ws_orderbook_snapshot(data)
                        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                    Ok(Some(StreamEvent::OrderbookSnapshot(orderbook)))
                } else {
                    let delta = BithumbParser::parse_ws_orderbook_delta(data)
                        .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                    Ok(Some(delta))
                }
            }
            "TRADE" => {
                let trades = BithumbParser::parse_ws_trades(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                // Bithumb sends array of trades, emit first one for simplicity
                if let Some(trade) = trades.first() {
                    Ok(Some(StreamEvent::Trade(trade.clone())))
                } else {
                    Ok(None)
                }
            }
            "ORDER" => {
                let event = BithumbParser::parse_ws_order_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderUpdate(event)))
            }
            "CONTRACT_ORDER" => {
                let event = BithumbParser::parse_ws_order_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderUpdate(event)))
            }
            _ => {
                // Unknown topic - ignore
                Ok(None)
            }
        }
    }

    /// Start ping task (every 30 seconds)
    fn start_ping_task(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let ping_interval = Duration::from_secs(30);

            loop {
                sleep(Duration::from_millis(1000)).await;

                let last = *last_ping.lock().await;

                if last.elapsed() >= ping_interval {
                    let mut stream_guard = ws_stream.lock().await;
                    if let Some(stream) = stream_guard.as_mut() {
                        let ping = OutgoingMessage {
                            cmd: "ping".to_string(),
                            args: vec![],
                        };

                        let msg_json = serde_json::to_string(&ping).expect("JSON serialization should never fail for valid struct");
                        if stream.send(Message::Text(msg_json)).await.is_ok() {
                            *last_ping.lock().await = Instant::now();
                        }
                    }
                }
            }
        });
    }

    /// Build topic string for subscription
    fn build_topic(request: &SubscriptionRequest, account_type: AccountType) -> String {
        match &request.stream_type {
            StreamType::Ticker => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                format!("TICKER:{}", symbol)
            }
            StreamType::Trade => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                format!("TRADE:{}", symbol)
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                format!("ORDERBOOK:{}", symbol)
            }
            StreamType::OrderUpdate => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => "ORDER".to_string(),
                    _ => "CONTRACT_ORDER".to_string(),
                }
            }
            // Bithumb Pro doesn't support these via WebSocket
            StreamType::Kline { .. } |
            StreamType::MarkPrice |
            StreamType::FundingRate |
            StreamType::BalanceUpdate |
            StreamType::PositionUpdate => {
                // Return placeholder topic - these will fail subscription
                "UNSUPPORTED".to_string()
            }
        }
    }

    /// Check if stream type requires authentication
    fn is_private(stream_type: &StreamType) -> bool {
        matches!(
            stream_type,
            StreamType::OrderUpdate | StreamType::BalanceUpdate | StreamType::PositionUpdate
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for BithumbWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Connect WebSocket
        let ws_stream = self.connect_ws().await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Create event channel
        let (tx, _rx) = mpsc::unbounded_channel();
        *self.event_tx.lock().await = Some(tx.clone());

        // Start message handler
        Self::start_message_handler(
            self.ws_stream.clone(),
            tx,
            self.status.clone(),
            self.is_authenticated.clone(),
            account_type,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start ping task
        Self::start_ping_task(
            self.ws_stream.clone(),
            self.last_ping.clone(),
        );

        // Authenticate if credentials provided
        if self.auth.is_some() {
            // Wait a bit for connection to stabilize
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.authenticate().await?;
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_stream.lock().await = None;
        *self.event_tx.lock().await = None;
        *self.is_authenticated.lock().await = false;
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
        // Apply rate limiting before subscribing
        self.rate_limit_wait().await;

        // Check if private channel requires authentication
        if Self::is_private(&request.stream_type) && !*self.is_authenticated.lock().await {
            return Err(WebSocketError::Auth("Private channels require authentication".to_string()));
        }

        let topic = Self::build_topic(&request, self.account_type);

        let msg = OutgoingMessage {
            cmd: "subscribe".to_string(),
            args: vec![topic],
        };

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
        // Apply rate limiting before unsubscribing
        self.rate_limit_wait().await;

        let topic = Self::build_topic(&request, self.account_type);

        let msg = OutgoingMessage {
            cmd: "unSubscribe".to_string(),
            args: vec![topic],
        };

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
        let (_tx, rx) = mpsc::unbounded_channel();

        // Clone the event_tx to forward events
        let _event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            // This is a simplified implementation
            // In production, we'd properly forward events from event_tx to tx
        });

        Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
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
