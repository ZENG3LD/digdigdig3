//! # HTX WebSocket Implementation
//!
//! WebSocket connector for HTX API.
//!
//! ## Features
//! - Public and private channels
//! - GZIP decompression (required for all messages!)
//! - Ping/pong heartbeat (v1: every 5s, v2: every 20s)
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Critical Implementation Details
//! - **ALL WebSocket messages are GZIP compressed** - must decompress before parsing
//! - Ping/pong is also compressed
//! - V1 (market data): ping every 5s, respond within 2 pings
//! - V2 (private): ping every 20s
//!
//! ## Architecture
//!
//! The WebSocket stream is split into independent read and write halves at connect
//! time. The write half is stored behind a mutex for shared access by pong replies
//! and subscribe/unsubscribe calls. The read half is owned exclusively by the
//! message loop task — no mutex contention on reads, eliminating the deadlock that
//! occurred when the old pattern held `ws_stream.lock()` across `.next().await`.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = HtxWebSocket::new(None, false, AccountType::Spot).await?;
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
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use flate2::read::GzDecoder;
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_millis,
};
use crate::core::types::{WebSocketResult, WebSocketError, Ticker, OrderBook};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::WeightRateLimiter;

use super::auth::HtxAuth;
use super::endpoints::{HtxUrls, format_symbol};
use super::parser::HtxParser;

// Global rate limiter for WebSocket connections (100 connections per IP)
static _WS_RATE_LIMITER: OnceLock<Arc<StdMutex<WeightRateLimiter>>> = OnceLock::new();

fn _get_ws_rate_limiter() -> &'static Arc<StdMutex<WeightRateLimiter>> {
    _WS_RATE_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            WeightRateLimiter::new(100, Duration::from_secs(1))
        ))
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by pong replies, subscribe, and unsubscribe
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task
type WsReader = SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Decompress GZIP message
///
/// HTX sends all WebSocket messages as GZIP compressed binary data
fn decompress_message(data: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscription message
#[derive(Debug, Clone, Serialize)]
struct SubMessage {
    sub: String,
    id: String,
}

/// Unsubscription message
#[derive(Debug, Clone, Serialize)]
struct UnsubMessage {
    unsub: String,
    id: String,
}

/// Pong message (v1)
#[derive(Debug, Clone, Serialize)]
struct PongMessage {
    pong: i64,
}

/// Pong message (v2)
#[derive(Debug, Clone, Serialize)]
struct PongMessageV2 {
    action: String,
    data: PongDataV2,
}

#[derive(Debug, Clone, Serialize)]
struct PongDataV2 {
    ts: i64,
}

/// Auth message (v2)
#[derive(Debug, Clone, Serialize)]
struct AuthMessage {
    action: String,
    ch: String,
    params: AuthParams,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthParams {
    auth_type: String,
    access_key: String,
    signature_method: String,
    signature_version: String,
    timestamp: String,
    signature: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HTX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// HTX WebSocket connector
pub struct HtxWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<HtxAuth>,
    /// Testnet mode
    testnet: bool,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender (for multiple consumers)
    broadcast_tx: Arc<broadcast::Sender<WebSocketResult<StreamEvent>>>,
    /// WebSocket write half — shared by pong replies, subscribe, and unsubscribe.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Message ID counter
    msg_id_counter: Arc<StdMutex<u64>>,
    /// Timestamp of the most recently sent WS-frame ping.
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl HtxWebSocket {
    /// Create new HTX WebSocket connector
    pub fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let auth = credentials.map(|c| HtxAuth::new(&c));

        // Create broadcast channel (capacity of 1000 events)
        let (broadcast_tx, _) = broadcast::channel(1000);

        Ok(Self {
            auth,
            testnet,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(broadcast_tx),
            ws_writer: Arc::new(Mutex::new(None)),
            msg_id_counter: Arc::new(StdMutex::new(0)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Get next message ID
    fn next_msg_id(&self) -> String {
        let mut counter = self.msg_id_counter.lock().expect("Mutex poisoned");
        *counter += 1;
        format!("id{}", *counter)
    }

    /// Connect to WebSocket, returning the raw stream
    async fn connect_ws(&self, private: bool) -> ExchangeResult<WsStream> {
        let ws_url = if private {
            HtxUrls::ws_account_url(self.testnet)
        } else {
            // Route to correct WebSocket URL based on account type
            match self.account_type {
                AccountType::FuturesCross | AccountType::FuturesIsolated => {
                    HtxUrls::ws_linear_swap_url(self.testnet)
                }
                _ => {
                    HtxUrls::ws_market_url(self.testnet)
                }
            }
        };

        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Authenticate WebSocket connection (for private channels, v2).
    ///
    /// Takes a mutable reference to the whole (unsplit) stream so we can do a
    /// synchronous request/response before handing read/write halves to their
    /// respective owners.
    async fn authenticate(&self, stream: &mut WsStream) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required for private channels".to_string()))?;

        let (api_key, timestamp, sig_method, sig_version, signature) = auth.sign_websocket_auth("api.huobi.pro");

        let auth_msg = AuthMessage {
            action: "req".to_string(),
            ch: "auth".to_string(),
            params: AuthParams {
                auth_type: "api".to_string(),
                access_key: api_key,
                signature_method: sig_method,
                signature_version: sig_version,
                timestamp,
                signature,
            },
        };

        let msg_json = serde_json::to_string(&auth_msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize auth message: {}", e)))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send auth message: {}", e)))?;

        // Wait for auth response (compressed!)
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            stream.next()
        ).await
            .map_err(|_| ExchangeError::Auth("Authentication timeout".to_string()))?;

        if let Some(Ok(Message::Binary(data))) = response {
            let decompressed = decompress_message(&data)
                .map_err(|e| ExchangeError::Parse(format!("Failed to decompress auth response: {}", e)))?;

            let json: Value = serde_json::from_str(&decompressed)
                .map_err(|e| ExchangeError::Parse(format!("Failed to parse auth response: {}", e)))?;

            if json["action"] == "req" && json["code"] == 200 {
                return Ok(());
            }

            return Err(ExchangeError::Auth(format!("Authentication failed: {:?}", json)));
        }

        Err(ExchangeError::Auth("Invalid auth response".to_string()))
    }

    /// Parse ticker from WebSocket data
    fn parse_ticker_from_ws_data(data: &Value, channel: &str) -> ExchangeResult<Ticker> {
        // Extract symbol from channel: "market.btcusdt.ticker" or "market.btcusdt.detail.merged"
        let parts: Vec<&str> = channel.split('.').collect();
        let symbol = if parts.len() >= 2 {
            parts[1].to_uppercase()
        } else {
            "UNKNOWN".to_string()
        };

        let last_price = data["close"].as_f64()
            .ok_or_else(|| ExchangeError::Parse("Invalid close price".into()))?;

        let bid_price = data["bid"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64());

        let ask_price = data["ask"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64());

        Ok(Ticker {
            symbol,
            last_price,
            bid_price,
            ask_price,
            high_24h: data["high"].as_f64(),
            low_24h: data["low"].as_f64(),
            volume_24h: data["amount"].as_f64(),
            quote_volume_24h: data["vol"].as_f64(),
            price_change_24h: {
                let close = data["close"].as_f64();
                let open = data["open"].as_f64();
                match (close, open) {
                    (Some(c), Some(o)) => Some(c - o),
                    _ => None,
                }
            },
            price_change_percent_24h: {
                let close = data["close"].as_f64();
                let open = data["open"].as_f64();
                match (close, open) {
                    (Some(c), Some(o)) if o != 0.0 => Some(((c - o) / o) * 100.0),
                    _ => None,
                }
            },
            timestamp: timestamp_millis() as i64,
        })
    }

    /// Parse orderbook from WebSocket data
    fn parse_orderbook_from_ws_data(data: &Value) -> ExchangeResult<OrderBook> {
        let bids = data["bids"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing bids".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_f64()?;
                let size = arr.get(1)?.as_f64()?;
                Some((price, size))
            })
            .collect();

        let asks = data["asks"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing asks".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_f64()?;
                let size = arr.get(1)?.as_f64()?;
                Some((price, size))
            })
            .collect();

        let timestamp = data["ts"].as_i64().unwrap_or_else(|| timestamp_millis() as i64);
        let sequence = data["version"].as_i64().map(|v| v.to_string());

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
        })
    }

    /// Send a raw text message over the write half.
    async fn _send_text(&self, text: String) -> ExchangeResult<()> {
        let mut writer_guard = self.ws_writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            writer.send(Message::Text(text)).await
                .map_err(|e| ExchangeError::Network(format!("Failed to send message: {}", e)))?;
        }
        Ok(())
    }

    /// Send pong (v1)
    async fn _send_pong(&self, ping_ts: i64) -> ExchangeResult<()> {
        let pong = PongMessage { pong: ping_ts };
        let msg_json = serde_json::to_string(&pong)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize pong: {}", e)))?;
        self._send_text(msg_json).await
    }

    /// Send pong (v2)
    async fn _send_pong_v2(&self, ping_ts: i64) -> ExchangeResult<()> {
        let pong = PongMessageV2 {
            action: "pong".to_string(),
            data: PongDataV2 { ts: ping_ts },
        };
        let msg_json = serde_json::to_string(&pong)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize pong: {}", e)))?;
        self._send_text(msg_json).await
    }

    /// Subscribe to channel (internal)
    async fn subscribe_channel(&self, channel: &str) -> ExchangeResult<()> {
        let sub_msg = SubMessage {
            sub: channel.to_string(),
            id: self.next_msg_id(),
        };

        let msg_json = serde_json::to_string(&sub_msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize sub message: {}", e)))?;

        let mut writer_guard = self.ws_writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            writer.send(Message::Text(msg_json)).await
                .map_err(|e| ExchangeError::Network(format!("Failed to send subscribe: {}", e)))?;
        } else {
            return Err(ExchangeError::Network("Not connected to WebSocket".to_string()));
        }

        Ok(())
    }

    /// Unsubscribe from channel (internal)
    async fn unsubscribe_channel(&self, channel: &str) -> ExchangeResult<()> {
        let unsub_msg = UnsubMessage {
            unsub: channel.to_string(),
            id: self.next_msg_id(),
        };

        let msg_json = serde_json::to_string(&unsub_msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize unsub message: {}", e)))?;

        let mut writer_guard = self.ws_writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            writer.send(Message::Text(msg_json)).await
                .map_err(|e| ExchangeError::Network(format!("Failed to send unsubscribe: {}", e)))?;
        } else {
            return Err(ExchangeError::Network("Not connected to WebSocket".to_string()));
        }

        Ok(())
    }

    /// Start message read loop.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// `ws_writer` is passed separately so the loop can send pong replies without
    /// touching the reader.
    ///
    /// Exits when the stream yields `None` (connection closed), a Close frame
    /// arrives, or an unrecoverable error occurs.
    /// Start periodic WS-frame ping task (every 5 seconds) for RTT measurement.
    ///
    /// HTX primarily communicates heartbeats via compressed JSON ping/pong.
    /// Alongside those, we send a WS-level `Message::Ping` frame every 5 seconds.
    /// HTX responds with a `Message::Pong` at the transport level which the
    /// message loop uses to compute RTT.
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            interval.tick().await; // skip first immediate tick

            loop {
                interval.tick().await;

                if *status.lock().await != ConnectionStatus::Connected {
                    break;
                }

                let mut writer_guard = ws_writer.lock().await;
                if let Some(writer) = writer_guard.as_mut() {
                    *last_ping.lock().await = Instant::now();
                    if writer.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    fn start_message_loop(
        mut reader: WsReader,
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        broadcast_tx: Arc<broadcast::Sender<WebSocketResult<StreamEvent>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Binary(data)) => {
                        // Decompress GZIP message
                        let decompressed = match decompress_message(&data) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("HTX WS: failed to decompress message: {}", e);
                                continue;
                            }
                        };

                        let json: Value = match serde_json::from_str(&decompressed) {
                            Ok(v) => v,
                            Err(e) => {
                                eprintln!("HTX WS: failed to parse JSON: {}", e);
                                continue;
                            }
                        };

                        // Handle ping (v1) — respond via the write half only
                        if let Some(ping_ts) = HtxParser::is_ws_ping(&json) {
                            let pong = PongMessage { pong: ping_ts };
                            if let Ok(msg_json) = serde_json::to_string(&pong) {
                                let mut writer_guard = ws_writer.lock().await;
                                if let Some(ref mut writer) = *writer_guard {
                                    let _ = writer.send(Message::Text(msg_json)).await;
                                }
                            }
                            continue;
                        }

                        // Handle ping (v2) — respond via the write half only
                        if let Some(ping_ts) = HtxParser::is_ws_v2_ping(&json) {
                            let pong = PongMessageV2 {
                                action: "pong".to_string(),
                                data: PongDataV2 { ts: ping_ts },
                            };
                            if let Ok(msg_json) = serde_json::to_string(&pong) {
                                let mut writer_guard = ws_writer.lock().await;
                                if let Some(ref mut writer) = *writer_guard {
                                    let _ = writer.send(Message::Text(msg_json)).await;
                                }
                            }
                            continue;
                        }

                        // Subscription confirmation — no event to emit
                        if json["status"] == "ok" && json.get("subbed").is_some() {
                            continue;
                        }

                        // Parse and emit data events
                        if let Ok((channel, data)) = HtxParser::parse_ws_message(&json) {
                            if channel.contains(".ticker") || channel.contains(".detail") {
                                if let Ok(ticker) = Self::parse_ticker_from_ws_data(data, &channel) {
                                    let _ = broadcast_tx.send(Ok(StreamEvent::Ticker(ticker)));
                                }
                            } else if channel.contains(".depth.") {
                                if let Ok(orderbook) = Self::parse_orderbook_from_ws_data(data) {
                                    let _ = broadcast_tx.send(Ok(StreamEvent::OrderbookSnapshot(orderbook)));
                                }
                            }
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        // Measure RTT from our last client-initiated WS ping frame.
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Ok(Message::Close(_)) => {
                        // Server closed the connection
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Err(e) => {
                        eprintln!("HTX WS: connection error: {}", e);
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {
                        // Text or other message types — ignore
                    }
                }
            }
            // Stream exhausted (None) — connection closed
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for HtxWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Determine if we need private or public endpoint
        let private = self.auth.is_some();

        let mut stream = self.connect_ws(private).await
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        // Authenticate (before split) if private
        if private {
            self.authenticate(&mut stream).await
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
        }

        // Split stream into independent read and write halves.
        // The write half goes behind a mutex for shared use (pong, subscribe, unsubscribe).
        // The read half is passed directly to the message loop — no mutex needed.
        let (write, read) = stream.split();
        *self.ws_writer.lock().await = Some(write);

        // Update status
        *self.status.lock().await = ConnectionStatus::Connected;

        // Start message loop — reader is moved in, never shared via mutex.
        // The loop exits naturally when the stream yields None (connection closed),
        // a Close frame arrives, or a network error occurs.
        Self::start_message_loop(
            read,
            self.ws_writer.clone(),
            self.broadcast_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start periodic WS-frame ping task for RTT measurement.
        Self::start_ping_task(
            self.ws_writer.clone(),
            self.status.clone(),
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

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Connecting,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let symbol_str = format_symbol(&request.symbol, self.account_type);

        let channel = match &request.stream_type {
            StreamType::Ticker => format!("market.{}.ticker", symbol_str),
            StreamType::Orderbook => format!("market.{}.depth.step0", symbol_str),
            StreamType::Trade => format!("market.{}.trade.detail", symbol_str),
            StreamType::Kline { interval } => format!("market.{}.kline.{}", symbol_str, interval),
            _ => return Err(WebSocketError::Subscription("Unsupported stream type".to_string())),
        };

        self.subscribe_channel(&channel).await
            .map_err(|e| WebSocketError::Subscription(e.to_string()))?;

        let mut subs = self.subscriptions.lock().await;
        subs.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let symbol_str = format_symbol(&request.symbol, self.account_type);

        let channel = match &request.stream_type {
            StreamType::Ticker => format!("market.{}.ticker", symbol_str),
            StreamType::Orderbook => format!("market.{}.depth.step0", symbol_str),
            StreamType::Trade => format!("market.{}.trade.detail", symbol_str),
            StreamType::Kline { interval } => format!("market.{}.kline.{}", symbol_str, interval),
            _ => return Err(WebSocketError::Subscription("Unsupported stream type".to_string())),
        };

        self.unsubscribe_channel(&channel).await
            .map_err(|e| WebSocketError::Subscription(e.to_string()))?;

        let mut subs = self.subscriptions.lock().await;
        subs.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.subscribe();
        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|r| async move {
            r.ok()
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_lock() {
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => vec![],
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }
}
