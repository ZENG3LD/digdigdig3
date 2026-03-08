//! # Bybit WebSocket Implementation
//!
//! WebSocket connector for Bybit V5 API.
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
//! `unsubscribe`, and the ping task. The read half is moved directly into the
//! message handler task — no mutex contention on reads, which eliminates the
//! starvation bug where the ping task could never acquire the shared mutex while
//! the read loop held it across `.next().await`.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = BybitWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
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
    Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_millis,
};
use crate::core::types::{WebSocketResult, WebSocketError, TradeSide};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::WeightRateLimiter;

use super::auth::BybitAuth;
use super::endpoints::{BybitUrls, format_symbol};
use super::parser::BybitParser;

// Global rate limiter for WebSocket connections (120 per second)
// Shared across all Bybit WebSocket instances to respect global rate limits
static WS_RATE_LIMITER: OnceLock<Arc<StdMutex<WeightRateLimiter>>> = OnceLock::new();

fn get_ws_rate_limiter() -> &'static Arc<StdMutex<WeightRateLimiter>> {
    WS_RATE_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            WeightRateLimiter::new(120, Duration::from_secs(1))
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
    op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
}

/// Ping message
#[derive(Debug, Clone, Serialize)]
struct PingMessage {
    op: String,
}

/// Auth message
#[derive(Debug, Clone, Serialize)]
struct AuthMessage {
    op: String,
    args: Vec<String>,
}

/// Incoming message from Bybit
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    op: Option<String>,
    success: Option<bool>,
    ret_msg: Option<String>,
    conn_id: Option<String>,
    topic: Option<String>,
    #[serde(rename = "type")]
    msg_type: Option<String>,
    ts: Option<i64>,
    data: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BYBIT WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bybit WebSocket connector
pub struct BybitWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<BybitAuth>,
    /// Testnet mode
    testnet: bool,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender — behind StdMutex so event_stream() can subscribe
    /// without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by subscribe, unsubscribe, and ping task.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Ping interval (20 seconds for Bybit)
    ping_interval: Duration,
    /// Last time a WS-level ping was sent (for RTT measurement)
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl BybitWebSocket {
    /// Create new Bybit WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let auth = credentials.map(|c| BybitAuth::new(&c));

        Ok(Self {
            auth,
            testnet,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            ping_interval: Duration::from_secs(20), // Bybit requires ping every 20 seconds
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Connect to WebSocket and return the raw stream
    async fn connect_ws(&self, account_type: AccountType, private: bool) -> ExchangeResult<WsStream> {
        let ws_url = if private {
            BybitUrls::ws_private_url(self.testnet)
        } else {
            BybitUrls::ws_url(account_type, self.testnet)
        };

        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Authenticate WebSocket connection (for private channels).
    ///
    /// Called before the read loop starts — uses `ws_writer` only so there is no
    /// contention with the (not-yet-started) read loop.
    async fn authenticate(&self) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required for private channels".to_string()))?;

        let (api_key, expires, signature) = auth.sign_websocket_auth();

        let auth_msg = AuthMessage {
            op: "auth".to_string(),
            args: vec![api_key, expires, signature],
        };

        let msg_json = serde_json::to_string(&auth_msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize auth message: {}", e)))?;

        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard.as_mut()
            .ok_or_else(|| ExchangeError::Network("WebSocket not connected".to_string()))?;
        writer.send(Message::Text(msg_json)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send auth message: {}", e)))?;

        Ok(())
    }

    /// Start message read loop.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// Runs until the WebSocket connection closes or errors.
    fn start_message_handler(
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
                        match Self::handle_message(&text, account_type) {
                            Ok(Some(event)) => {
                                let tx_guard = event_tx.lock().unwrap();
                                if let Some(ref tx) = *tx_guard {
                                    let _ = tx.send(Ok(event));
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let tx_guard = event_tx.lock().unwrap();
                                if let Some(ref tx) = *tx_guard {
                                    let _ = tx.send(Err(e));
                                }
                            }
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        // Record RTT for the WS-level ping sent by start_ping_task
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
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

    /// Handle incoming WebSocket message, returning the parsed event if any.
    fn handle_message(
        text: &str,
        account_type: AccountType,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Handle control messages
        match msg.op.as_deref() {
            Some("pong") => return Ok(None),
            Some("subscribe") => {
                if msg.success == Some(false) {
                    return Err(WebSocketError::ProtocolError(
                        msg.ret_msg.unwrap_or_else(|| "Subscription failed".to_string())
                    ));
                }
                return Ok(None);
            }
            Some("auth") => return Ok(None),
            _ => {}
        }

        // Data message
        if let Some(topic) = msg.topic {
            if let Some(data) = msg.data {
                return Self::parse_data_message(&topic, &data, account_type);
            }
        }

        Ok(None)
    }

    /// Parse data message to StreamEvent
    fn parse_data_message(
        topic: &str,
        data: &Value,
        _account_type: AccountType,
    ) -> WebSocketResult<Option<StreamEvent>> {
        if topic.starts_with("tickers.") {
            let ticker = Self::parse_ticker_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Ticker(ticker)))
        } else if topic.starts_with("publicTrade.") {
            let trade = Self::parse_trade_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Trade(trade)))
        } else if topic.starts_with("orderbook.") {
            let event = Self::parse_orderbook_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(event))
        } else if topic.starts_with("kline.") {
            let kline = Self::parse_kline_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::Kline(kline)))
        } else if topic == "order" {
            let event = Self::parse_order_update_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::OrderUpdate(event)))
        } else if topic == "wallet" {
            let event = Self::parse_balance_update_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::BalanceUpdate(event)))
        } else if topic == "position" {
            let event = Self::parse_position_update_ws(data)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::PositionUpdate(event)))
        } else {
            Ok(None)
        }
    }

    /// Start ping task.
    ///
    /// Uses only `ws_writer` — no contention with the reader half.
    /// Exits naturally when the writer send fails (connection closed).
    /// Sends both a JSON `{"op":"ping"}` (Bybit application-level keepalive)
    /// and a WS-level `Message::Ping` (for RTT measurement via Pong response).
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        ping_interval: Duration,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(ping_interval);
            // Skip the immediate first tick so we don't ping before the connection
            // is fully established.
            interval.tick().await;

            loop {
                interval.tick().await;

                let ping = PingMessage { op: "ping".to_string() };
                let msg_json = serde_json::to_string(&ping)
                    .expect("JSON serialization should never fail for valid struct");

                let mut writer_guard = ws_writer.lock().await;
                if let Some(ref mut writer) = *writer_guard {
                    // Send application-level JSON ping (Bybit keepalive)
                    if writer.send(Message::Text(msg_json)).await.is_err() {
                        break;
                    }
                    // Send WS-level ping for RTT measurement
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

    /// Send a text message through the write half
    async fn send_message(&self, msg_json: String) -> WebSocketResult<()> {
        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;
        writer.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))
    }

    /// Build topic string for subscription
    fn build_topic(request: &SubscriptionRequest, account_type: AccountType) -> String {
        match &request.stream_type {
            StreamType::Ticker => {
                let symbol = format_symbol(&request.symbol, account_type);
                format!("tickers.{}", symbol)
            }
            StreamType::Trade => {
                let symbol = format_symbol(&request.symbol, account_type);
                format!("publicTrade.{}", symbol)
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                let symbol = format_symbol(&request.symbol, account_type);
                format!("orderbook.50.{}", symbol) // 50 depth levels
            }
            StreamType::Kline { interval } => {
                let symbol = format_symbol(&request.symbol, account_type);
                format!("kline.{}.{}", interval, symbol)
            }
            StreamType::MarkPrice => {
                let symbol = format_symbol(&request.symbol, account_type);
                format!("tickers.{}", symbol) // Bybit includes mark price in ticker
            }
            StreamType::FundingRate => {
                let symbol = format_symbol(&request.symbol, account_type);
                format!("tickers.{}", symbol) // Bybit includes funding rate in ticker
            }
            StreamType::OrderUpdate => "order".to_string(),
            StreamType::BalanceUpdate => "wallet".to_string(),
            StreamType::PositionUpdate => "position".to_string(),
        }
    }

    /// Check if stream type requires private channel
    #[allow(dead_code)]
    fn is_private(stream_type: &StreamType) -> bool {
        matches!(
            stream_type,
            StreamType::OrderUpdate | StreamType::BalanceUpdate | StreamType::PositionUpdate
        )
    }

    /// Wait for WebSocket rate limit if needed
    async fn ws_rate_limit_wait(weight: u32) {
        loop {
            let wait_time = {
                let limiter = get_ws_rate_limiter();
                let mut guard = limiter.lock().expect("Mutex poisoned");
                if guard.try_acquire(weight) {
                    return;
                }
                guard.time_until_ready(weight)
            };

            if wait_time > Duration::ZERO {
                sleep(wait_time).await;
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    fn parse_ticker_ws(data: &Value) -> ExchangeResult<crate::core::Ticker> {
        if let Some(arr) = data.as_array() {
            if let Some(ticker_data) = arr.first() {
                let wrapper = json!({
                    "retCode": 0,
                    "result": {
                        "list": [ticker_data]
                    },
                    "time": timestamp_millis()
                });
                return BybitParser::parse_ticker(&wrapper);
            }
        }

        let wrapper = json!({
            "retCode": 0,
            "result": {
                "list": [data]
            },
            "time": timestamp_millis()
        });
        BybitParser::parse_ticker(&wrapper)
    }

    fn parse_trade_ws(data: &Value) -> ExchangeResult<crate::core::PublicTrade> {
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Trade data not an array".to_string()))?;

        let trade_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty trade array".to_string()))?;

        let symbol = trade_data.get("s")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?;

        let price = trade_data.get("p")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid price".to_string()))?;

        let quantity = trade_data.get("v")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid quantity".to_string()))?;

        let timestamp = trade_data.get("T")
            .and_then(|t| t.as_i64())
            .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))?;

        let side = trade_data.get("S")
            .and_then(|s| s.as_str())
            .map(|s| match s {
                "Buy" => TradeSide::Buy,
                "Sell" => TradeSide::Sell,
                _ => TradeSide::Buy,
            })
            .unwrap_or(TradeSide::Buy);

        let id = trade_data.get("i")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();

        Ok(crate::core::PublicTrade {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
            timestamp,
        })
    }

    fn parse_orderbook_ws(data: &Value) -> ExchangeResult<StreamEvent> {
        let wrapper = json!({
            "retCode": 0,
            "result": data,
        });

        let orderbook = BybitParser::parse_orderbook(&wrapper)?;
        Ok(StreamEvent::OrderbookSnapshot(orderbook))
    }

    fn parse_kline_ws(data: &Value) -> ExchangeResult<crate::core::Kline> {
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Kline data not an array".to_string()))?;

        let kline_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty kline array".to_string()))?;

        let start = kline_data.get("start")
            .and_then(|s| s.as_i64())
            .ok_or_else(|| ExchangeError::Parse("Invalid start time".to_string()))?;

        let open = kline_data.get("open")
            .and_then(|o| o.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid open".to_string()))?;

        let high = kline_data.get("high")
            .and_then(|h| h.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid high".to_string()))?;

        let low = kline_data.get("low")
            .and_then(|l| l.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid low".to_string()))?;

        let close = kline_data.get("close")
            .and_then(|c| c.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid close".to_string()))?;

        let volume = kline_data.get("volume")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid volume".to_string()))?;

        Ok(crate::core::Kline {
            open_time: start,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: None,
            close_time: None,
            trades: None,
        })
    }

    fn parse_order_update_ws(data: &Value) -> ExchangeResult<crate::core::OrderUpdateEvent> {
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Order data not an array".to_string()))?;

        let order_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty order array".to_string()))?;

        let wrapper = json!({
            "retCode": 0,
            "result": order_data,
        });

        let order = BybitParser::parse_order(&wrapper)?;

        Ok(crate::core::OrderUpdateEvent {
            order_id: order.id,
            client_order_id: order.client_order_id,
            symbol: order.symbol,
            side: order.side,
            order_type: order.order_type,
            status: order.status,
            price: order.price,
            quantity: order.quantity,
            filled_quantity: order.filled_quantity,
            average_price: order.average_price,
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: order.commission_asset,
            trade_id: None,
            timestamp: order.updated_at.unwrap_or(order.created_at),
        })
    }

    fn parse_balance_update_ws(data: &Value) -> ExchangeResult<crate::core::BalanceUpdateEvent> {
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Balance data not an array".to_string()))?;

        let balance_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty balance array".to_string()))?;

        let coin = balance_data.get("coin")
            .and_then(|c| c.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing coin".to_string()))?;

        let free = balance_data.get("walletBalance")
            .and_then(|b| b.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let locked = balance_data.get("locked")
            .and_then(|l| l.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let total = free + locked;

        Ok(crate::core::BalanceUpdateEvent {
            asset: coin.to_string(),
            free,
            locked,
            total,
            delta: None,
            reason: None,
            timestamp: timestamp_millis() as i64,
        })
    }

    fn parse_position_update_ws(data: &Value) -> ExchangeResult<crate::core::PositionUpdateEvent> {
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Position data not an array".to_string()))?;

        let pos_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty position array".to_string()))?;

        let symbol = pos_data.get("symbol")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?;

        let quantity = pos_data.get("size")
            .and_then(|s| s.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let side = pos_data.get("side")
            .and_then(|s| s.as_str())
            .map(|s| match s {
                "Buy" => crate::core::PositionSide::Long,
                "Sell" => crate::core::PositionSide::Short,
                _ => crate::core::PositionSide::Long,
            })
            .unwrap_or(crate::core::PositionSide::Long);

        let entry_price = pos_data.get("avgPrice")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let unrealized_pnl = pos_data.get("unrealisedPnl")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let mark_price = pos_data.get("markPrice")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let liquidation_price = pos_data.get("liqPrice")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let leverage = pos_data.get("leverage")
            .and_then(|l| l.as_str())
            .and_then(|s| s.parse::<u32>().ok());

        Ok(crate::core::PositionUpdateEvent {
            symbol: symbol.to_string(),
            side,
            quantity,
            entry_price,
            mark_price,
            unrealized_pnl,
            realized_pnl: None,
            liquidation_price,
            leverage,
            margin_type: None,
            reason: None,
            timestamp: timestamp_millis() as i64,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for BybitWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Public channels always use the public URL.
        let needs_private = false;

        // Connect and split into independent read/write halves.
        let ws_stream = self.connect_ws(account_type, needs_private).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        let (write, read) = ws_stream.split();
        *self.ws_writer.lock().await = Some(write);

        // Create event broadcast channel
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Authenticate if private (uses ws_writer, before read loop starts)
        if needs_private {
            self.authenticate().await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
        }

        // Start message loop — reader is moved in, never shared via mutex.
        Self::start_message_handler(
            read,
            self.event_tx.clone(),
            self.status.clone(),
            account_type,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start ping task — uses ws_writer only, no contention with reader.
        Self::start_ping_task(
            self.ws_writer.clone(),
            self.ping_interval,
            self.last_ping.clone(),
        );

        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Close the write half. The message loop owns the read half and will exit
        // naturally when it detects the close. The ping task will exit on next
        // failed send.
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            let _ = writer.close().await;
        }

        *self.status.lock().await = ConnectionStatus::Disconnected;
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
        Self::ws_rate_limit_wait(1).await;

        let topic = Self::build_topic(&request, self.account_type);

        let msg = OutgoingMessage {
            op: "subscribe".to_string(),
            args: Some(vec![topic]),
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        self.send_message(msg_json).await?;

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        Self::ws_rate_limit_wait(1).await;

        let topic = Self::build_topic(&request, self.account_type);

        let msg = OutgoingMessage {
            op: "unsubscribe".to_string(),
            args: Some(vec![topic]),
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        self.send_message(msg_json).await?;

        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
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
}
