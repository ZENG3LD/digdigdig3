//! # Bitget WebSocket Implementation
//!
//! WebSocket connector for Bitget Spot and Futures.
//!
//! ## Features
//! - Public and private channels
//! - Automatic ping/pong heartbeat (30s interval)
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = BitgetWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
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
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use std::sync::OnceLock;

use crate::core::{
    Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_millis, hmac_sha256, encode_base64,
};
use crate::core::types::{WebSocketResult, WebSocketError, OrderbookCapabilities, WsBookChannel, ChecksumInfo, ChecksumAlgorithm};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::SimpleRateLimiter;

use super::auth::BitgetAuth;
use super::endpoints::BitgetUrls;
use super::parser::BitgetParser;

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Global rate limiter for Bitget WebSocket connections
/// Shared across all instances to prevent rate limiting when tests run in parallel
/// Bitget allows 10 messages per second across all connections
static GLOBAL_WS_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

fn get_global_ws_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            // Conservative: 8 messages per second to stay under 10/sec limit
            SimpleRateLimiter::new(8, Duration::from_secs(1))
        ))
    }).clone()
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscription operation
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    op: String,
    args: Vec<SubscriptionArg>,
}

/// Subscription argument
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubscriptionArg {
    inst_type: String,
    channel: String,
    inst_id: String,
}

/// Login message (for private channels)
#[derive(Debug, Clone, Serialize)]
struct LoginMessage {
    op: String,
    args: Vec<LoginArg>,
}

/// Login arguments
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginArg {
    api_key: String,
    passphrase: String,
    timestamp: String,
    sign: String,
}

/// Incoming message from Bitget
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    #[serde(rename = "event")]
    event: Option<String>,
    #[serde(rename = "code")]
    code: Option<String>,
    #[serde(rename = "msg")]
    msg: Option<String>,
    #[serde(rename = "arg")]
    arg: Option<Value>,
    #[serde(rename = "action")]
    action: Option<String>,
    #[serde(rename = "data")]
    data: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BITGET WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Bitget WebSocket connector
pub struct BitgetWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<BitgetAuth>,
    /// URLs (mainnet/testnet)
    urls: BitgetUrls,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event broadcast sender — uses std::sync::Mutex so event_stream() can subscribe
    /// without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Is authenticated (for private channels)
    is_authenticated: Arc<Mutex<bool>>,
    /// Most recent ping round-trip time in milliseconds (0 until first pong)
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl BitgetWebSocket {
    /// Create new Bitget WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            BitgetUrls::TESTNET
        } else {
            BitgetUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(BitgetAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_stream: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            is_authenticated: Arc::new(Mutex::new(false)),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Get WebSocket URL for account type and privacy level
    fn get_ws_url(&self, is_private: bool) -> String {
        if is_private {
            self.urls.ws_private_url()
        } else {
            self.urls.ws_public_url()
        }
    }

    /// Generate WebSocket login signature
    fn generate_ws_signature(secret_key: &str, timestamp: i64) -> String {
        let prehash = format!("{}GET/user/verify", timestamp);
        encode_base64(&hmac_sha256(secret_key.as_bytes(), prehash.as_bytes()))
    }

    /// Connect to WebSocket
    async fn connect_ws(&self, url: &str) -> ExchangeResult<WsStream> {
        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Authenticate on private channel
    async fn authenticate(&self) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Private channels require authentication".to_string()))?;

        let timestamp = timestamp_millis() as i64;
        let signature = Self::generate_ws_signature(auth.api_secret(), timestamp);

        let login_msg = LoginMessage {
            op: "login".to_string(),
            args: vec![LoginArg {
                api_key: auth.api_key().to_string(),
                passphrase: auth.passphrase().to_string(),
                timestamp: timestamp.to_string(),
                sign: signature,
            }],
        };

        let msg_json = serde_json::to_string(&login_msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize login: {}", e)))?;

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard.as_mut()
            .ok_or_else(|| ExchangeError::Network("Not connected".to_string()))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send login: {}", e)))?;

        drop(stream_guard);

        // Wait for login response
        sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    /// Start message handling task
    fn start_message_handler(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
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

                        // Handle pong response — measure RTT
                        if text == "pong" {
                            let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                            *ws_ping_rtt_ms.lock().await = rtt;
                            continue;
                        }

                        // Clone the sender before .await to avoid holding the StdMutex guard
                        // across an await point.
                        let tx_clone = event_tx.lock().unwrap().clone();
                        if let Some(tx) = tx_clone {
                            if let Err(e) = Self::handle_message(&text, &tx, &is_authenticated, account_type).await {
                                let _ = tx.send(Err(e));
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        drop(stream_guard);
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Some(Err(e)) => {
                        drop(stream_guard);
                        if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
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
            // Drop the broadcast sender so all BroadcastStream receivers get None
            // from .next(). Without this, a clean close leaves the sender alive
            // and the bridge hangs forever instead of reconnecting.
            let _ = event_tx.lock().unwrap().take();
            // Stream exhausted — connection closed
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        text: &str,
        event_tx: &broadcast::Sender<WebSocketResult<StreamEvent>>,
        is_authenticated: &Arc<Mutex<bool>>,
        account_type: AccountType,
    ) -> WebSocketResult<()> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Handle event types
        if let Some(event) = msg.event.as_deref() {
            match event {
                "login" => {
                    // Check login success
                    if msg.code.as_deref() == Some("0") {
                        *is_authenticated.lock().await = true;
                        return Ok(());
                    } else {
                        let error_msg = msg.msg.unwrap_or_else(|| "Login failed".to_string());
                        return Err(WebSocketError::ProtocolError(error_msg));
                    }
                }
                "subscribe" => {
                    // Subscription acknowledgement
                    return Ok(());
                }
                "unsubscribe" => {
                    // Unsubscribe acknowledgement
                    return Ok(());
                }
                "error" => {
                    let error_msg = msg.msg.unwrap_or_else(|| "Unknown error".to_string());
                    return Err(WebSocketError::ProtocolError(error_msg));
                }
                _ => {}
            }
        }

        // Handle data messages - only if all required fields present
        if let (Some(action), Some(arg), Some(data)) = (&msg.action, &msg.arg, &msg.data) {
            // Only parse if we have valid data
            match Self::parse_data_message(action, arg, data, account_type) {
                Ok(Some(event)) => {
                    let _ = event_tx.send(Ok(event));
                }
                Ok(None) => {
                    // No event generated, that's fine
                }
                Err(_e) => {
                    // Silently skip unrecognised data messages — parse errors on
                    // individual market-data frames must never flood stderr.
                    // Uncomment the line below for local debugging only:
                    // eprintln!("[bitget ws] parse error: {}", _e);
                }
            }
        }

        Ok(())
    }

    /// Parse data message to StreamEvent
    fn parse_data_message(
        _action: &str,
        arg: &Value,
        data: &Value,
        _account_type: AccountType,
    ) -> WebSocketResult<Option<StreamEvent>> {
        // Extract channel from arg
        let channel = arg.get("channel")
            .and_then(|c| c.as_str())
            .ok_or_else(|| WebSocketError::Parse("Missing channel".to_string()))?;

        // Extract instId from arg as fallback symbol identifier
        let inst_id_fallback = arg.get("instId").and_then(|v| v.as_str());

        // Match by channel to determine event type
        match channel {
            "ticker" => {
                let ticker = BitgetParser::parse_ws_ticker(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            "trade" => {
                let trade = BitgetParser::parse_ws_trade(data, inst_id_fallback)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(trade)))
            }
            "books" | "books5" | "books15" => {
                let delta = BitgetParser::parse_ws_orderbook_delta(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(delta))
            }
            channel if channel.starts_with("candle") => {
                let kline = BitgetParser::parse_ws_kline(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Kline(kline)))
            }
            "orders" => {
                let event = BitgetParser::parse_ws_order_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderUpdate(event)))
            }
            "fill" => {
                // Bitget fill channel - could be mapped to trade or order update
                // For now, skip
                Ok(None)
            }
            "account" => {
                let event = BitgetParser::parse_ws_balance_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::BalanceUpdate(event)))
            }
            "positions" => {
                let event = BitgetParser::parse_ws_position_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::PositionUpdate(event)))
            }
            _ => {
                // Unknown channel - ignore
                Ok(None)
            }
        }
    }

    /// Start ping task (30 second interval as per Bitget spec)
    fn start_ping_task(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;

                // Rate limit before sending ping
                let limiter = get_global_ws_limiter();
                let wait_time = {
                    let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
                    if !limiter_guard.try_acquire() {
                        limiter_guard.time_until_ready()
                    } else {
                        Duration::ZERO
                    }
                };
                if !wait_time.is_zero() {
                    sleep(wait_time).await;
                    let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
                    limiter_guard.try_acquire();
                }

                let mut stream_guard = ws_stream.lock().await;
                if let Some(stream) = stream_guard.as_mut() {
                    if stream.send(Message::Text("ping".to_string())).await.is_ok() {
                        *last_ping.lock().await = Instant::now();
                    }
                }
                drop(stream_guard);
            }
        });
    }

    /// Build subscription args for request
    fn build_subscription_args(request: &SubscriptionRequest, account_type: AccountType) -> Vec<SubscriptionArg> {
        let inst_type = match account_type {
            AccountType::Spot | AccountType::Margin => "SPOT",
            AccountType::FuturesIsolated | AccountType::FuturesCross => "USDT-FUTURES",
            AccountType::Earn | AccountType::Lending
            | AccountType::Options | AccountType::Convert => "SPOT",
        };

        let (channel, inst_id) = match &request.stream_type {
            StreamType::Ticker => {
                let symbol = format!("{}{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());
                ("ticker", symbol)
            }
            StreamType::Trade => {
                let symbol = format!("{}{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());
                ("trade", symbol)
            }
            StreamType::Orderbook => {
                let symbol = format!("{}{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());
                ("books", symbol)
            }
            StreamType::OrderbookDelta => {
                let symbol = format!("{}{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());
                ("books15", symbol)
            }
            StreamType::Kline { interval } => {
                let symbol = format!("{}{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());
                let channel = format!("candle{}", interval);
                (Box::leak(channel.into_boxed_str()) as &str, symbol)
            }
            StreamType::MarkPrice => {
                let symbol = format!("{}{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());
                ("mark-price", symbol)
            }
            StreamType::FundingRate => {
                let symbol = format!("{}{}", request.symbol.base.to_uppercase(), request.symbol.quote.to_uppercase());
                ("funding-rate", symbol)
            }
            StreamType::OrderUpdate => {
                ("orders", "default".to_string())
            }
            StreamType::BalanceUpdate => {
                ("account", "default".to_string())
            }
            StreamType::PositionUpdate => {
                ("positions", "default".to_string())
            }
        };

        vec![SubscriptionArg {
            inst_type: inst_type.to_string(),
            channel: channel.to_string(),
            inst_id,
        }]
    }

    /// Check if stream type requires private channel
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
impl WebSocketConnector for BitgetWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Determine if we need private channel
        let needs_private = self.auth.is_some();

        // Get WebSocket URL
        let ws_url = self.get_ws_url(needs_private);

        // Connect WebSocket
        let ws_stream = self.connect_ws(&ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        *self.ws_stream.lock().await = Some(ws_stream);

        // If private channel, authenticate
        if needs_private {
            self.authenticate().await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
        }

        *self.status.lock().await = ConnectionStatus::Connected;

        // Create event channel (broadcast supports multiple receivers)
        let (tx, _rx) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Start message handler
        Self::start_message_handler(
            self.ws_stream.clone(),
            self.event_tx.clone(),
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

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_stream.lock().await = None;
        let _ = self.event_tx.lock().unwrap().take();
        *self.is_authenticated.lock().await = false;
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
        let is_private = Self::is_private(&request.stream_type);

        // Check authentication for private channels
        if is_private && !*self.is_authenticated.lock().await {
            return Err(WebSocketError::ProtocolError("Not authenticated for private channels".to_string()));
        }

        // Rate limit before sending subscription
        let limiter = get_global_ws_limiter();
        let wait_time = {
            let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
            if !limiter_guard.try_acquire() {
                limiter_guard.time_until_ready()
            } else {
                Duration::ZERO
            }
        };
        if !wait_time.is_zero() {
            sleep(wait_time).await;
            let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
            limiter_guard.try_acquire();
        }

        let args = Self::build_subscription_args(&request, self.account_type);

        let msg = SubscribeMessage {
            op: "subscribe".to_string(),
            args,
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
        // Rate limit before sending unsubscription
        let limiter = get_global_ws_limiter();
        let wait_time = {
            let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
            if !limiter_guard.try_acquire() {
                limiter_guard.time_until_ready()
            } else {
                Duration::ZERO
            }
        };
        if !wait_time.is_zero() {
            sleep(wait_time).await;
            let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
            limiter_guard.try_acquire();
        }

        let args = Self::build_subscription_args(&request, self.account_type);

        let msg = SubscribeMessage {
            op: "unsubscribe".to_string(),
            args,
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
        // std::sync::Mutex::lock() is instant here — no async contention.
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
        static BITGET_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("books1",  1,   100),
            WsBookChannel::snapshot("books5",  5,   150),
            WsBookChannel::snapshot("books15", 15,  150),
            WsBookChannel::delta("books",      None, Some(150)),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 5, 15],
            ws_default_depth: None,
            rest_max_depth: Some(150),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITGET_CHANNELS,
            checksum: Some(ChecksumInfo {
                algorithm: ChecksumAlgorithm::Crc32Interleaved,
                levels_per_side: 25,
                opt_in: false,
            }),
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}
