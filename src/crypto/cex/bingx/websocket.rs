//! # BingX WebSocket Implementation
//!
//! WebSocket connector for BingX Spot and Swap markets.
//!
//! ## Features
//! - Public and private channels
//! - GZIP decompression for all messages
//! - Ping/pong heartbeat (server sends ping, client responds with pong)
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Architecture
//!
//! The WebSocket stream is split into independent read and write halves on connect.
//! The write half is stored behind a mutex for shared access by `subscribe`,
//! `unsubscribe`, and the message handler (for pong replies).
//! The read half is owned exclusively by the message loop task — no mutex
//! contention on reads, which eliminates the deadlock that occurred when the
//! reader and writer shared the same `Arc<Mutex<Option<WsStream>>>`.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = BingxWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
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
use std::io::Read;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use flate2::read::GzDecoder;
use futures_util::{SinkExt, Stream, StreamExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

use crate::core::{
    AccountType, ConnectionStatus, Credentials, ExchangeError, ExchangeResult, HttpClient,
    StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketError, WebSocketResult, OrderbookCapabilities};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::SimpleRateLimiter;
use std::sync::OnceLock;

/// Global rate limiter for BingX WebSocket connections
/// Shared across all instances to prevent 429 when tests run in parallel
static GLOBAL_WS_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

fn get_global_ws_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            // Very conservative: 5 connections per 10 seconds
            SimpleRateLimiter::new(5, Duration::from_secs(10))
        ))
    }).clone()
}

use super::auth::BingxAuth;
use super::endpoints::format_symbol;
use super::parser::BingxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by subscribe, unsubscribe, and pong replies
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task
type WsReader = SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscribe/Unsubscribe message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    id: String,
    #[serde(rename = "reqType")]
    req_type: String,
    #[serde(rename = "dataType")]
    data_type: String,
}

/// Ping message from server
#[derive(Debug, Clone, Deserialize)]
struct PingMessage {
    ping: i64,
}

/// Pong response to server
#[derive(Debug, Clone, Serialize)]
struct PongMessage {
    pong: i64,
}

/// Incoming message from BingX
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    #[serde(rename = "dataType")]
    data_type: Option<String>,
    data: Option<Value>,
    code: Option<i32>,
    msg: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// LISTEN KEY MANAGEMENT (for private channels)
// ═══════════════════════════════════════════════════════════════════════════════

/// Response from listen key endpoint
#[derive(Debug, Clone, Deserialize)]
struct ListenKeyResponse {
    code: i32,
    msg: Option<String>,
    data: Option<ListenKeyData>,
}

#[derive(Debug, Clone, Deserialize)]
struct ListenKeyData {
    #[serde(rename = "listenKey")]
    listen_key: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BINGX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// BingX WebSocket connector
pub struct BingxWebSocket {
    /// HTTP client for getting listen keys
    http: HttpClient,
    /// Authentication (None for public channels only)
    auth: Option<BingxAuth>,
    /// Base REST URL for listen key endpoints
    base_url: &'static str,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event broadcast sender — uses std::sync::Mutex so event_stream() can be
    /// called without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by subscribe, unsubscribe, and pong replies.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Listen key (for private channels)
    listen_key: Arc<Mutex<Option<String>>>,
    /// Rate limiter for WebSocket connection attempts
    connection_limiter: Arc<StdMutex<SimpleRateLimiter>>,
    /// Most recent ping round-trip time in milliseconds (0 until first pong)
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl BingxWebSocket {
    /// WebSocket URLs
    const WS_BASE_URL: &'static str = "wss://open-api-ws.bingx.com/market";

    /// Create new BingX WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        _testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let base_url = "https://open-api.bingx.com";
        let http = HttpClient::new(30_000)?;

        let auth = credentials.as_ref().map(BingxAuth::new).transpose()?;

        // Use global rate limiter shared across all instances
        let connection_limiter = get_global_ws_limiter();

        Ok(Self {
            http,
            auth,
            base_url,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            listen_key: Arc::new(Mutex::new(None)),
            connection_limiter,
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Wait for rate limit if necessary before connecting
    async fn rate_limit_wait(&self) {
        let wait_time = {
            let mut limiter = self.connection_limiter.lock().expect("Mutex poisoned");
            if !limiter.try_acquire() {
                limiter.time_until_ready()
            } else {
                Duration::ZERO
            }
        };

        if !wait_time.is_zero() {
            sleep(wait_time).await;
            // Try again after waiting
            let mut limiter = self.connection_limiter.lock().expect("Mutex poisoned");
            limiter.try_acquire();
        }
    }

    /// Get listen key for private channels
    async fn get_listen_key(&self, account_type: AccountType) -> ExchangeResult<String> {
        let auth = self
            .auth
            .as_ref()
            .ok_or_else(|| ExchangeError::Auth("Private channels require authentication".to_string()))?;

        let path = match account_type {
            AccountType::Spot | AccountType::Margin => "/openApi/spot/v1/user/listen-key",
            _ => "/openApi/swap/v2/user/listen-key",
        };

        let url = format!("{}{}", self.base_url, path);

        // Sign request
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-BX-APIKEY".to_string(), auth.api_key().to_string());

        let response = self.http.post(&url, &serde_json::json!({}), &headers).await?;

        // Parse response
        let resp: ListenKeyResponse = serde_json::from_value(response)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse listen key response: {}", e)))?;

        if resp.code != 0 {
            let msg = resp.msg.unwrap_or_else(|| "Failed to get listen key".to_string());
            return Err(ExchangeError::Api {
                code: resp.code,
                message: msg,
            });
        }

        let data = resp
            .data
            .ok_or_else(|| ExchangeError::Parse("Missing data in listen key response".to_string()))?;

        Ok(data.listen_key)
    }

    /// Connect to WebSocket, returning the split (write, read) halves
    async fn connect_ws(&self, listen_key: Option<&str>) -> ExchangeResult<(WsSink, WsReader)> {
        // Rate limit before attempting connection
        self.rate_limit_wait().await;

        let ws_url = if let Some(key) = listen_key {
            format!("{}?listenKey={}", Self::WS_BASE_URL, key)
        } else {
            Self::WS_BASE_URL.to_string()
        };

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream.split())
    }

    /// Start message handling task.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// `ws_writer` is passed separately so the loop can send pong replies without
    /// touching the reader lock (there is none).
    ///
    /// BingX sends a server-side ping roughly every 5 seconds.
    /// If no message of any kind arrives within READ_TIMEOUT we assume the TCP
    /// connection is silently dead and break out so the bridge retry logic can
    /// re-establish the stream.
    fn start_message_handler(
        mut reader: WsReader,
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        account_type: AccountType,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        const READ_TIMEOUT: Duration = Duration::from_secs(30);

        /// Extract a cloned Sender from the std::sync::Mutex so the guard is
        /// dropped before any `.await` point.  `broadcast::Sender` is Clone + Send.
        fn get_tx(
            event_tx: &StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>,
        ) -> Option<broadcast::Sender<WebSocketResult<StreamEvent>>> {
            event_tx.lock().unwrap().clone()
        }

        tokio::spawn(async move {
            loop {
                let next_msg = timeout(READ_TIMEOUT, reader.next()).await;

                match next_msg {
                    Err(_elapsed) => {
                        // No message received within READ_TIMEOUT — connection is stale.
                        // Grab a clone of the sender and drop the guard before .await.
                        if let Some(tx) = get_tx(&event_tx) {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(
                                "BingX WS read timeout — no message for 30s, reconnecting".to_string(),
                            )));
                        }
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Ok(Some(Ok(Message::Binary(data)))) => {
                        // Decompress GZIP data
                        match Self::decompress_message(&data) {
                            Ok(text) => {
                                // Check for ping message
                                if let Ok(ping) = serde_json::from_str::<PingMessage>(&text) {
                                    // Send pong via the write half — no deadlock risk
                                    let pong = PongMessage { pong: ping.ping };
                                    if let Ok(pong_json) = serde_json::to_string(&pong) {
                                        let mut writer_guard = ws_writer.lock().await;
                                        if let Some(ref mut writer) = *writer_guard {
                                            let _ = writer.send(Message::Text(pong_json)).await;
                                        }
                                    }
                                    *last_ping.lock().await = Instant::now();
                                    continue;
                                }

                                // Parse data message — clone sender, drop guard, then use clone.
                                if let Some(tx) = get_tx(&event_tx) {
                                    if let Err(e) = Self::handle_message(&text, &tx, account_type) {
                                        let _ = tx.send(Err(e));
                                    }
                                }
                            }
                            Err(e) => {
                                if let Some(tx) = get_tx(&event_tx) {
                                    let _ = tx.send(Err(WebSocketError::Parse(format!(
                                        "Failed to decompress message: {}",
                                        e
                                    ))));
                                }
                            }
                        }
                    }
                    Ok(Some(Ok(Message::Text(text)))) => {
                        // Check for ping message
                        if let Ok(ping) = serde_json::from_str::<PingMessage>(&text) {
                            let pong = PongMessage { pong: ping.ping };
                            if let Ok(pong_json) = serde_json::to_string(&pong) {
                                let mut writer_guard = ws_writer.lock().await;
                                if let Some(ref mut writer) = *writer_guard {
                                    let _ = writer.send(Message::Text(pong_json)).await;
                                }
                            }
                            *last_ping.lock().await = Instant::now();
                            continue;
                        }

                        // Parse data message — clone sender, drop guard, then use clone.
                        if let Some(tx) = get_tx(&event_tx) {
                            if let Err(e) = Self::handle_message(&text, &tx, account_type) {
                                let _ = tx.send(Err(e));
                            }
                        }
                    }
                    Ok(Some(Ok(Message::Pong(_)))) => {
                        // Response to our client-initiated WS Ping frame — measure RTT
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Ok(Some(Ok(Message::Close(_)))) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Ok(Some(Err(e))) => {
                        if let Some(tx) = get_tx(&event_tx) {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
                        break;
                    }
                    Ok(None) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Ok(Some(Ok(_))) => {
                        // Ignore other frame types (Ping, Frame)
                    }
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            // from .next(). Without this, a clean close leaves the sender alive
            // and the bridge hangs forever instead of reconnecting.
            let _ = event_tx.lock().unwrap().take();
            // Stream exhausted — connection closed
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Decompress GZIP message
    fn decompress_message(data: &[u8]) -> Result<String, std::io::Error> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Handle incoming WebSocket message
    fn handle_message(
        text: &str,
        event_tx: &broadcast::Sender<WebSocketResult<StreamEvent>>,
        account_type: AccountType,
    ) -> WebSocketResult<()> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Check for error response
        if let Some(code) = msg.code {
            if code != 0 {
                let error_msg = msg.msg.unwrap_or_else(|| "Unknown error".to_string());
                return Err(WebSocketError::ProtocolError(format!(
                    "Server error {}: {}",
                    code, error_msg
                )));
            }
        }

        // Parse data message
        if let Some(data_type) = msg.data_type {
            if let Some(data) = msg.data {
                if let Some(event) = Self::parse_data_message(&data_type, &data, account_type)? {
                    let _ = event_tx.send(Ok(event));
                }
            }
        }

        Ok(())
    }

    /// Parse data message to StreamEvent
    fn parse_data_message(
        data_type: &str,
        data: &Value,
        account_type: AccountType,
    ) -> WebSocketResult<Option<StreamEvent>> {
        // Parse based on stream type from dataType
        // Format: "SYMBOL@streamType" or "streamType" for private channels

        if data_type.ends_with("@ticker") {
            // Ticker stream
            let ticker = BingxParser::parse_ws_ticker(data, account_type)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Ticker(ticker)))
        } else if data_type.ends_with("@trade") {
            // Trade stream
            let trade = BingxParser::parse_ws_trade(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Trade(trade)))
        } else if data_type.ends_with("@depth") || data_type.ends_with("@depth20") {
            // Orderbook stream
            let event = BingxParser::parse_ws_orderbook(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(event))
        } else if data_type.contains("@kline_") {
            // Kline stream
            let kline = BingxParser::parse_ws_kline(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Kline(kline)))
        } else if data_type == "spot.executionReport" || data_type == "swap.order" {
            // Order update (private)
            let event = BingxParser::parse_ws_order_update(data, account_type)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::OrderUpdate(event)))
        } else if data_type == "spot.account" || data_type == "swap.account" {
            // Balance update (private)
            let event = BingxParser::parse_ws_balance_update(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::BalanceUpdate(event)))
        } else if data_type == "swap.position" {
            // Position update (private, futures only)
            let event = BingxParser::parse_ws_position_update(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::PositionUpdate(event)))
        } else {
            // Unknown stream type - ignore
            Ok(None)
        }
    }

    /// Build dataType string for subscription
    fn build_data_type(request: &SubscriptionRequest, _account_type: AccountType) -> String {
        match &request.stream_type {
            StreamType::Ticker => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, _account_type);
                format!("{}@ticker", symbol)
            }
            StreamType::Trade => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, _account_type);
                format!("{}@trade", symbol)
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, _account_type);
                format!("{}@depth", symbol)
            }
            StreamType::Kline { interval } => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, _account_type);
                // BingX uses format like "1min", "5min", "1hour", "1day"
                let bingx_interval = Self::map_kline_interval(interval);
                format!("{}@kline_{}", symbol, bingx_interval)
            }
            StreamType::OrderUpdate => {
                match _account_type {
                    AccountType::Spot | AccountType::Margin => "spot.executionReport".to_string(),
                    _ => "swap.order".to_string(),
                }
            }
            StreamType::BalanceUpdate => {
                match _account_type {
                    AccountType::Spot | AccountType::Margin => "spot.account".to_string(),
                    _ => "swap.account".to_string(),
                }
            }
            StreamType::PositionUpdate => "swap.position".to_string(),
            StreamType::MarkPrice => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, _account_type);
                format!("{}@markPrice", symbol)
            }
            StreamType::FundingRate => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, _account_type);
                format!("{}@fundingRate", symbol)
            }
        }
    }

    /// Map kline interval to BingX format
    fn map_kline_interval(interval: &str) -> &'static str {
        match interval {
            "1m" => "1min",
            "3m" => "3min",
            "5m" => "5min",
            "15m" => "15min",
            "30m" => "30min",
            "1h" => "1hour",
            "2h" => "2hour",
            "4h" => "4hour",
            "6h" => "6hour",
            "8h" => "8hour",
            "12h" => "12hour",
            "1d" => "1day",
            "3d" => "3day",
            "1w" => "1week",
            "1M" => "1month",
            _ => "1hour", // default
        }
    }

    /// Start client-initiated ping task to measure RTT via WS Ping/Pong frames.
    ///
    /// Sends a `Message::Ping(vec![])` every 5 seconds.  The `last_ping`
    /// timestamp is recorded when the frame is sent, and `ws_ping_rtt_ms` is
    /// updated in the message handler when the corresponding `Message::Pong`
    /// arrives.
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;

                let mut writer_guard = ws_writer.lock().await;
                if let Some(ref mut writer) = *writer_guard {
                    if writer.send(Message::Ping(vec![])).await.is_ok() {
                        *last_ping.lock().await = Instant::now();
                    } else {
                        // Connection closed — exit task
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Check if stream type requires private channel (listen key)
    fn _is_private(stream_type: &StreamType) -> bool {
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
impl WebSocketConnector for BingxWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Check if we need private channel (listen key)
        let listen_key = if self.auth.is_some() {
            let key = self
                .get_listen_key(account_type)
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            *self.listen_key.lock().await = Some(key.clone());
            Some(key)
        } else {
            None
        };

        // Connect WebSocket and split into independent read/write halves.
        // The write half goes behind a mutex for shared access.
        // The read half is passed directly to the message handler — no mutex needed.
        let (write, read) = self
            .connect_ws(listen_key.as_deref())
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        *self.ws_writer.lock().await = Some(write);

        // Create event broadcast channel
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        *self.status.lock().await = ConnectionStatus::Connected;

        // Start message handler — reader is moved in, never shared via mutex.
        Self::start_message_handler(
            read,
            self.ws_writer.clone(),
            self.event_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            account_type,
            self.ws_ping_rtt_ms.clone(),
        );

        // Start client-initiated ping task for RTT measurement
        Self::start_ping_task(
            self.ws_writer.clone(),
            self.last_ping.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Close the write half. The message loop task owns the read half and will
        // detect the close frame / stream termination naturally and exit on its own.
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            let _ = writer.close().await;
        }

        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.listen_key.lock().await = None;
        self.subscriptions.lock().await.clear();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let data_type = Self::build_data_type(&request, self.account_type);

        let msg = SubscribeMessage {
            id: Uuid::new_v4().to_string(),
            req_type: "sub".to_string(),
            data_type,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard
            .as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        writer
            .send(Message::Text(msg_json))
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(writer_guard);

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let data_type = Self::build_data_type(&request, self.account_type);

        let msg = SubscribeMessage {
            id: Uuid::new_v4().to_string(),
            req_type: "unsub".to_string(),
            data_type,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard
            .as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        writer
            .send(Message::Text(msg_json))
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(writer_guard);

        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        // std::sync::Mutex::lock() never contends long — send() is an instant operation.
        let tx_guard = self.event_tx.lock().unwrap();

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
        match self.subscriptions.try_lock() {
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[5, 10, 20],
            ws_default_depth: Some(20),
            rest_max_depth: None,
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[100],
            default_speed_ms: Some(100),
            ws_channels: &[],
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}
