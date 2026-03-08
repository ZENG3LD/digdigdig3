//! # Gate.io WebSocket Implementation
//!
//! WebSocket connector for Gate.io V4 API.
//!
//! ## Features
//! - Public and private channels
//! - Ping/pong heartbeat (every 20 seconds)
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Architecture
//!
//! The WebSocket stream is split into independent read and write halves on connect.
//! The write half is stored behind a mutex for shared access by `subscribe`,
//! `unsubscribe`, and the ping task.  The read half is owned exclusively by the
//! message loop task — no mutex contention on reads, which eliminates the
//! "closed connection" deadlock that occurred when both the reader and the ping
//! task held the same mutex simultaneously.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = GateioWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
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
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_seconds,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::WeightRateLimiter;

use super::auth::GateioAuth;
use super::endpoints::{GateioUrls, format_symbol};
use super::parser::GateioParser;

// Global rate limiter for WebSocket connections (100 subscriptions per second)
// Shared across all Gate.io WebSocket instances to respect global rate limits
static WS_RATE_LIMITER: OnceLock<Arc<StdMutex<WeightRateLimiter>>> = OnceLock::new();

fn get_ws_rate_limiter() -> &'static Arc<StdMutex<WeightRateLimiter>> {
    WS_RATE_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            WeightRateLimiter::new(100, Duration::from_secs(1))
        ))
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by subscribe, unsubscribe, and ping task
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task
type WsReader = SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing message (subscribe/unsubscribe/ping)
#[derive(Debug, Clone, Serialize)]
struct OutgoingMessage {
    time: i64,
    channel: String,
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<AuthData>,
}

/// Authentication data for private channels
#[derive(Debug, Clone, Serialize)]
struct AuthData {
    method: String,
    #[serde(rename = "KEY")]
    key: String,
    #[serde(rename = "SIGN")]
    sign: String,
}

/// Incoming message from Gate.io
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    time: Option<i64>,
    time_ms: Option<i64>,
    channel: Option<String>,
    event: Option<String>,
    result: Option<Value>,
    error: Option<ErrorData>,
}

/// Error data in response
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct ErrorData {
    code: Option<i32>,
    message: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GATE.IO WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Gate.io WebSocket connector
pub struct GateioWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<GateioAuth>,
    /// Testnet mode
    _testnet: bool,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event broadcast sender — std::sync::Mutex so event_stream() can subscribe
    /// without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by subscribe, unsubscribe, and ping task.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Ping interval (20 seconds for Gate.io)
    ping_interval: Duration,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// URLs
    urls: GateioUrls,
    /// Most recent ping round-trip time in milliseconds (0 until first pong)
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl GateioWebSocket {
    /// Create new Gate.io WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let auth = if let Some(creds) = credentials {
            Some(GateioAuth::new(&creds)?)
        } else {
            None
        };

        let urls = if testnet {
            GateioUrls::TESTNET
        } else {
            GateioUrls::MAINNET
        };

        Ok(Self {
            auth,
            _testnet: testnet,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            ping_interval: Duration::from_secs(20), // Gate.io requires ping every 10-30 seconds
            last_ping: Arc::new(Mutex::new(Instant::now())),
            urls,
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Send a text message over the write half.
    ///
    /// Only locks `ws_writer` — no contention with the message loop task
    /// which owns the read half exclusively.
    async fn send_text(&self, text: String) -> WebSocketResult<()> {
        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;
        writer.send(Message::Text(text)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))
    }

    /// Start message read loop.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// The loop exits when the WebSocket connection closes or errors.
    fn start_message_loop(
        mut reader: WsReader,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        account_type: AccountType,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Detect pong before passing to handle_message — record RTT here
                        // (handle_message is sync and cannot await the tokio Mutex).
                        if text.contains(".pong") {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                if parsed.get("channel")
                                    .and_then(|c| c.as_str())
                                    .map(|c| c.ends_with(".pong"))
                                    .unwrap_or(false)
                                {
                                    let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                                    *ws_ping_rtt_ms.lock().await = rtt;
                                    // Still let handle_message consume it so it returns Ok(())
                                }
                            }
                        }
                        if let Err(e) = Self::handle_message(&text, &event_tx, account_type) {
                            let tx_guard = event_tx.lock().unwrap();
                            if let Some(ref tx) = *tx_guard {
                                let _ = tx.send(Err(e));
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Err(e) => {
                        let tx_guard = event_tx.lock().unwrap();
                        if let Some(ref tx) = *tx_guard {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
                        break;
                    }
                    _ => {}
                }
            }
            // Stream exhausted — connection closed
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Handle incoming WebSocket message
    fn handle_message(
        text: &str,
        event_tx: &Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        _account_type: AccountType,
    ) -> WebSocketResult<()> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Handle pong response
        if let Some(channel) = &msg.channel {
            if channel.ends_with(".pong") {
                // Pong response - ignore
                return Ok(());
            }
        }

        // Handle subscription confirmation
        if msg.event.as_deref() == Some("subscribe") {
            if let Some(error) = msg.error {
                return Err(WebSocketError::ProtocolError(
                    error.message.unwrap_or_else(|| "Subscription failed".to_string())
                ));
            }
            // Subscription successful
            return Ok(());
        }

        // Handle data messages
        if msg.event.as_deref() == Some("update") {
            if let (Some(channel), Some(result)) = (&msg.channel, &msg.result) {
                if let Some(event) = Self::parse_data_message(channel, result)? {
                    let tx_guard = event_tx.lock().unwrap();
                    if let Some(ref tx) = *tx_guard {
                        let _ = tx.send(Ok(event));
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse data message to StreamEvent
    fn parse_data_message(
        channel: &str,
        data: &Value,
    ) -> WebSocketResult<Option<StreamEvent>> {
        // Gate.io channels: spot.tickers, spot.trades, spot.order_book, spot.candlesticks
        // futures.tickers, futures.trades, futures.order_book_update
        // Private: spot.orders, spot.balances, futures.orders, futures.positions

        if channel.contains(".tickers") {
            // Ticker update
            let ticker = GateioParser::parse_ws_ticker(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Ticker(ticker)))
        } else if channel.contains(".trades") {
            // Trade update
            let trade = GateioParser::parse_ws_trade(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Trade(trade)))
        } else if channel.contains(".order_book") {
            // Orderbook update (snapshot)
            let orderbook = Self::parse_orderbook_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::OrderbookSnapshot(orderbook)))
        } else if channel.contains(".candlesticks") {
            // Kline update
            let kline = Self::parse_kline_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Kline(kline)))
        } else if channel.contains(".orders") {
            // Order update
            let event = GateioParser::parse_ws_order_update(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::OrderUpdate(event)))
        } else if channel.contains(".balances") {
            // Balance update
            let event = GateioParser::parse_ws_balance_update(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::BalanceUpdate(event)))
        } else if channel.contains(".positions") {
            // Position update
            let event = GateioParser::parse_ws_position_update(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::PositionUpdate(event)))
        } else {
            // Unknown channel - ignore
            Ok(None)
        }
    }

    /// Start ping task.
    ///
    /// Uses only `ws_writer` — no contention with the message loop task
    /// which owns the read half exclusively.
    ///
    /// The task exits naturally when the writer send fails (connection closed).
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        ping_interval: Duration,
        last_ping: Arc<Mutex<Instant>>,
        account_type: AccountType,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1000)).await;

                let last = *last_ping.lock().await;

                if last.elapsed() >= ping_interval {
                    // Determine ping channel based on account type
                    let ping_channel = match account_type {
                        AccountType::Spot | AccountType::Margin => "spot.ping",
                        AccountType::FuturesCross | AccountType::FuturesIsolated => "futures.ping",
                    };

                    let ping = json!({
                        "time": timestamp_seconds() as i64,
                        "channel": ping_channel
                    });

                    let msg_json = serde_json::to_string(&ping)
                        .expect("JSON serialization should never fail for valid struct");

                    let mut writer_guard = ws_writer.lock().await;
                    if let Some(ref mut writer) = *writer_guard {
                        if writer.send(Message::Text(msg_json)).await.is_ok() {
                            *last_ping.lock().await = Instant::now();
                        } else {
                            // Connection closed — exit ping task
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        });
    }

    /// Build channel string for subscription
    fn build_channel(request: &SubscriptionRequest, account_type: AccountType) -> String {
        let prefix = match account_type {
            AccountType::Spot | AccountType::Margin => "spot",
            AccountType::FuturesCross | AccountType::FuturesIsolated => "futures",
        };

        match &request.stream_type {
            StreamType::Ticker => format!("{}.tickers", prefix),
            StreamType::Trade => format!("{}.trades", prefix),
            StreamType::Orderbook | StreamType::OrderbookDelta => format!("{}.order_book", prefix),
            StreamType::Kline { .. } => format!("{}.candlesticks", prefix),
            StreamType::MarkPrice => format!("{}.tickers", prefix), // Gate.io includes mark price in ticker
            StreamType::FundingRate => format!("{}.tickers", prefix), // Gate.io includes funding rate in ticker
            StreamType::OrderUpdate => format!("{}.orders", prefix),
            StreamType::BalanceUpdate => format!("{}.balances", prefix),
            StreamType::PositionUpdate => format!("{}.positions", prefix),
        }
    }

    /// Build payload for subscription
    fn build_payload(request: &SubscriptionRequest, account_type: AccountType) -> Vec<String> {
        let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        match &request.stream_type {
            StreamType::Ticker => vec![symbol],
            StreamType::Trade => vec![symbol],
            StreamType::Orderbook | StreamType::OrderbookDelta => vec![symbol, "20".to_string(), "1000ms".to_string()],
            StreamType::Kline { interval } => vec![interval.to_string(), symbol],
            StreamType::MarkPrice => vec![symbol],
            StreamType::FundingRate => vec![symbol],
            StreamType::OrderUpdate => vec![symbol],
            StreamType::BalanceUpdate => vec![],
            StreamType::PositionUpdate => vec![symbol],
        }
    }

    /// Check if stream type requires private channel
    fn is_private(stream_type: &StreamType) -> bool {
        matches!(
            stream_type,
            StreamType::OrderUpdate | StreamType::BalanceUpdate | StreamType::PositionUpdate
        )
    }

    /// Wait for WebSocket rate limit if needed
    async fn ws_rate_limit_wait(weight: u32) {
        loop {
            // Scope the lock to ensure it's dropped before await
            let wait_time = {
                let limiter = get_ws_rate_limiter();
                let mut guard = limiter.lock().expect("Mutex poisoned");
                if guard.try_acquire(weight) {
                    return; // Successfully acquired, exit early
                }
                guard.time_until_ready(weight)
            }; // Lock is dropped here

            if wait_time > Duration::ZERO {
                sleep(wait_time).await;
            }
        }
    }

    /// Generate authentication signature for WebSocket subscription
    fn generate_auth_signature(
        auth: &GateioAuth,
        channel: &str,
        event: &str,
        timestamp: i64,
    ) -> ExchangeResult<(String, String)> {
        // Gate.io WebSocket auth signature format:
        // "channel={channel}&event={event}&time={timestamp}"
        let sign_str = format!("channel={}&event={}&time={}", channel, event, timestamp);

        let signature = auth.sign_ws(&sign_str)?;
        let api_key = auth.api_key().to_string();

        Ok((api_key, signature))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    fn parse_orderbook_ws(data: &Value) -> ExchangeResult<crate::core::OrderBook> {
        // Parse orderbook from WebSocket data
        // Gate.io format: { t, lastUpdateId, s, bids: [[price, size]], asks: [[price, size]] }

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = pair[0].as_str()?.parse::<f64>().ok()?;
                            let size = pair[1].as_str()?.parse::<f64>().ok()?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(crate::core::OrderBook {
            timestamp: data.get("t")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: data.get("lastUpdateId")
                .and_then(|s| s.as_i64())
                .map(|n| n.to_string()),
        })
    }

    fn parse_kline_ws(data: &Value) -> ExchangeResult<crate::core::Kline> {
        // Gate.io WebSocket kline format:
        // { t: "timestamp", v: "volume", c: "close", h: "high", l: "low", o: "open", n: "symbol", a: "quote_volume" }

        let open_time = data.get("t")
            .and_then(|t| t.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0) * 1000; // seconds to ms

        let parse_f64 = |key: &str| -> f64 {
            data.get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0)
        };

        Ok(crate::core::Kline {
            open_time,
            open: parse_f64("o"),
            high: parse_f64("h"),
            low: parse_f64("l"),
            close: parse_f64("c"),
            volume: parse_f64("v"),
            quote_volume: Some(parse_f64("a")),
            close_time: None,
            trades: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for GateioWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Connect WebSocket
        let ws_url = self.urls.ws_url(account_type);
        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        // Split into independent read and write halves.
        // The write half goes behind a mutex for shared use by subscribe/ping.
        // The read half is passed directly to the message loop — no mutex needed.
        let (write, read) = ws_stream.split();
        *self.ws_writer.lock().await = Some(write);

        // Create event broadcast channel
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Start message loop — reader is moved in, never shared via mutex.
        Self::start_message_loop(
            read,
            self.event_tx.clone(),
            self.status.clone(),
            account_type,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start ping task — uses ws_writer only.
        // Exits naturally when the connection drops.
        Self::start_ping_task(
            self.ws_writer.clone(),
            self.ping_interval,
            self.last_ping.clone(),
            account_type,
        );

        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Close the write half. The message loop task owns the read half and will
        // detect the close / stream termination naturally and exit on its own.
        // The ping task will fail on its next send attempt and also exit.
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            let _ = writer.close().await;
        }

        *self.status.lock().await = ConnectionStatus::Disconnected;
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
        // Wait for rate limit (weight 1 for subscriptions)
        Self::ws_rate_limit_wait(1).await;

        let channel = Self::build_channel(&request, self.account_type);
        let payload = Self::build_payload(&request, self.account_type);
        let timestamp = timestamp_seconds() as i64;

        // Build message
        let mut msg = OutgoingMessage {
            time: timestamp,
            channel: channel.clone(),
            event: "subscribe".to_string(),
            payload: if payload.is_empty() { None } else { Some(payload) },
            auth: None,
        };

        // Add authentication for private channels
        if Self::is_private(&request.stream_type) {
            let auth = self.auth.as_ref()
                .ok_or_else(|| WebSocketError::ConnectionError("Authentication required for private channels".to_string()))?;

            let (api_key, signature) = Self::generate_auth_signature(auth, &channel, "subscribe", timestamp)
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

            msg.auth = Some(AuthData {
                method: "api_key".to_string(),
                key: api_key,
                sign: signature,
            });
        }

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        // Send through write half — no contention with message loop read half
        self.send_text(msg_json).await?;

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Wait for rate limit (weight 1 for unsubscriptions)
        Self::ws_rate_limit_wait(1).await;

        let channel = Self::build_channel(&request, self.account_type);
        let payload = Self::build_payload(&request, self.account_type);

        let msg = OutgoingMessage {
            time: timestamp_seconds() as i64,
            channel,
            event: "unsubscribe".to_string(),
            payload: if payload.is_empty() { None } else { Some(payload) },
            auth: None,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        // Send through write half — no contention with message loop read half
        self.send_text(msg_json).await?;

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
