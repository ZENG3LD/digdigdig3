//! # Kraken WebSocket Implementation
//!
//! Complete WebSocket connector for Kraken v2 API.
//!
//! ## Features
//! - Public and private channels
//! - Ping/pong heartbeat (every 30 seconds)
//! - Token-based authentication for private channels
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Kraken WebSocket v2
//!
//! - Public URL: `wss://ws.kraken.com/v2`
//! - Private URL: `wss://ws-auth.kraken.com/v2`
//! - **IMPORTANT**: Symbol format is `BTC/USD` (uses BTC, NOT XBT!)
//!   - v2 WebSocket API uses `BTC/USD` format
//!   - v2 will REJECT `XBT/USD` with error "Currency pair not supported"
//!   - REST API still uses XBT for Bitcoin, but WebSocket v2 uses BTC
//! - Authentication: Token from REST API (expires in 15 minutes)
//!
//! ## Symbol Format Requirements
//!
//! When creating symbols for Kraken WebSocket v2:
//! - Use `Symbol::new("BTC", "USD")` - CORRECT ✓
//! - Do NOT use `Symbol::new("XBT", "USD")` - WRONG ❌ (will be rejected)
//!
//! The v2 WebSocket API requires the standard "BTC" symbol, unlike the REST API
//! which uses "XBT" for Bitcoin.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
    timestamp_millis,
};
use crate::core::types::{WebSocketResult, WebSocketError, TradeSide, OrderSide, OrderType, OrderStatus, OrderBookLevel, OrderbookDelta as OrderbookDeltaData, OrderbookCapabilities};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::WeightRateLimiter;

// Global rate limiter for WebSocket connections (150 per 10 minutes)
// Shared across all Kraken WebSocket instances to respect global rate limits
static WS_RATE_LIMITER: OnceLock<Arc<StdMutex<WeightRateLimiter>>> = OnceLock::new();

fn get_ws_rate_limiter() -> &'static Arc<StdMutex<WeightRateLimiter>> {
    WS_RATE_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            // 150 connections per 10 minutes = 0.25 per second
            WeightRateLimiter::new(1, Duration::from_secs(4))
        ))
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing subscribe/unsubscribe message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    method: String,
    params: SubscribeParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    req_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct SubscribeParams {
    channel: String,
    symbol: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depth: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_trigger: Option<String>,
}

/// Ping message
#[derive(Debug, Clone, Serialize)]
struct PingMessage {
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    req_id: Option<u64>,
}

/// Incoming message from Kraken
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    method: Option<String>,
    channel: Option<String>,
    #[serde(rename = "type")]
    msg_type: Option<String>,
    data: Option<Value>,
    success: Option<bool>,
    error: Option<String>,
    result: Option<Value>,
    req_id: Option<u64>,
    time_in: Option<String>,
    time_out: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// KRAKEN WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsWriter = SplitSink<WsStream, Message>;
type WsReader = SplitStream<WsStream>;

/// Kraken WebSocket connector
pub struct KrakenWebSocket {
    /// Authentication token (None for public channels only)
    token: Option<String>,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender — behind StdMutex so event_stream() can subscribe
    /// without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket writer (separate from reader to avoid lock contention)
    ws_writer: Arc<Mutex<Option<WsWriter>>>,
    /// Write command channel (to send messages without blocking reads)
    write_tx: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
    /// Ping interval (30 seconds for Kraken)
    ping_interval: Duration,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
    /// Request ID counter
    req_id_counter: Arc<Mutex<u64>>,
}

impl KrakenWebSocket {
    /// Create new Kraken WebSocket connector
    pub async fn new(
        token: Option<String>,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        Ok(Self {
            token,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            write_tx: Arc::new(Mutex::new(None)),
            ping_interval: Duration::from_secs(30), // Kraken recommends ping every 30 seconds
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
            req_id_counter: Arc::new(Mutex::new(1)),
        })
    }

    /// Connect to WebSocket
    async fn connect_ws(&self, private: bool) -> ExchangeResult<WsStream> {
        // Wait for rate limit
        Self::ws_rate_limit_wait(1).await;

        let ws_url = if private {
            "wss://ws-auth.kraken.com/v2"
        } else {
            "wss://ws.kraken.com/v2"
        };

        eprintln!("[KRAKEN WS] Connecting to {}", ws_url);

        let (ws_stream, response) = connect_async(ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        eprintln!("[KRAKEN WS] Connection successful, response status: {:?}", response.status());

        Ok(ws_stream)
    }

    /// Get next request ID
    async fn next_req_id(&self) -> u64 {
        let mut counter = self.req_id_counter.lock().await;
        let id = *counter;
        *counter += 1;
        id
    }

    /// Start message handling task with separate read/write tasks
    fn start_message_handler(
        ws_writer: Arc<Mutex<Option<WsWriter>>>,
        mut ws_reader: WsReader,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        _account_type: AccountType,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) -> tokio::sync::mpsc::UnboundedSender<Message> {
        // Create channel for write commands
        let (write_tx, mut write_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

        // Spawn write task
        let status_write = status.clone();
        tokio::spawn(async move {
            while let Some(msg) = write_rx.recv().await {
                let mut writer_guard = ws_writer.lock().await;
                if let Some(writer) = writer_guard.as_mut() {
                    match writer.send(msg).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("[KRAKEN WS] Write error: {}", e);
                            *status_write.lock().await = ConnectionStatus::Disconnected;
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        // Spawn read task
        let write_tx_clone = write_tx.clone();
        tokio::spawn(async move {
            while let Some(msg_result) = ws_reader.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        match Self::handle_message_broadcast(&text, &event_tx) {
                            Ok(()) => {}
                            Err(e) => {
                                let tx_guard = event_tx.lock().unwrap();
                                if let Some(ref tx) = *tx_guard {
                                    let _ = tx.send(Err(e));
                                }
                            }
                        }
                    }
                    Ok(Message::Ping(payload)) => {
                        let _ = write_tx_clone.send(Message::Pong(payload));
                    }
                    Ok(Message::Pong(_)) => {
                        // Record RTT for the WS-level ping sent by start_ping_task
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Ok(Message::Close(_frame)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Err(e) => {
                        eprintln!("[KRAKEN WS] WebSocket error: {}", e);
                        let tx_guard = event_tx.lock().unwrap();
                        if let Some(ref tx) = *tx_guard {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
                        break;
                    }
                    _ => {}
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            let _ = event_tx.lock().unwrap().take();
            *status.lock().await = ConnectionStatus::Disconnected;
        });

        write_tx
    }

    /// Handle incoming WebSocket message (broadcast sender variant)
    fn handle_message_broadcast(
        text: &str,
        event_tx: &Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    ) -> WebSocketResult<()> {
        // Parse and dispatch synchronously using the broadcast sender
        let msg: IncomingMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(_e) => return Ok(()),
        };

        match msg.method.as_deref() {
            Some("pong") => return Ok(()),
            Some("subscribe") | Some("unsubscribe") => {
                if msg.success == Some(false) {
                    let error_msg = msg.error.unwrap_or_else(|| "Subscription failed (no error message)".to_string());
                    return Err(WebSocketError::ProtocolError(error_msg));
                }
                if msg.success == Some(true) {
                    return Ok(());
                }
                return Err(WebSocketError::ProtocolError(
                    format!("Ambiguous subscription response (missing success field): {:?}", msg)
                ));
            }
            _ => {}
        }

        if let Some(channel) = msg.channel {
            if let Some(data) = msg.data {
                match Self::parse_data_message(&channel, &msg.msg_type, &data) {
                    Ok(Some(event)) => {
                        let tx_guard = event_tx.lock().unwrap();
                        if let Some(ref tx) = *tx_guard {
                            let _ = tx.send(Ok(event));
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("[KRAKEN WS] Parse error for channel '{}': {}", channel, e);
                    }
                }
            }
        }

        Ok(())
    }


    /// Parse data message to StreamEvent
    fn parse_data_message(
        channel: &str,
        msg_type: &Option<String>,
        data: &Value,
    ) -> WebSocketResult<Option<StreamEvent>> {
        // Kraken channels: ticker, book, trade, ohlc, executions, balances

        match channel {
            "ticker" => {
                // Ticker update
                let ticker = Self::parse_ticker_ws(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            "trade" => {
                // Trade update
                let trade = Self::parse_trade_ws(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(trade)))
            }
            "book" => {
                // Orderbook update
                let is_snapshot = msg_type.as_deref() == Some("snapshot");
                let event = Self::parse_orderbook_ws(data, is_snapshot)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(event))
            }
            "ohlc" => {
                // Kline update
                let kline = Self::parse_kline_ws(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Kline(kline)))
            }
            "executions" => {
                // Order/execution update
                let event = Self::parse_execution_ws(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(event))
            }
            "balances" => {
                // Balance update
                let event = Self::parse_balance_ws(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(event))
            }
            _ => {
                // Unknown channel - ignore
                Ok(None)
            }
        }
    }

    /// Start ping task.
    ///
    /// Sends both a JSON `{"method":"ping"}` (Kraken application keepalive) and
    /// a WS-level `Message::Ping` (for RTT measurement via Pong response).
    fn start_ping_task(
        write_tx: mpsc::UnboundedSender<Message>,
        ping_interval: Duration,
        last_ping: Arc<Mutex<Instant>>,
        req_id_counter: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1000)).await;

                let last = *last_ping.lock().await;

                if last.elapsed() >= ping_interval {
                    let req_id = {
                        let mut counter = req_id_counter.lock().await;
                        let id = *counter;
                        *counter += 1;
                        id
                    };

                    let ping = PingMessage {
                        method: "ping".to_string(),
                        req_id: Some(req_id),
                    };

                    let msg_json = serde_json::to_string(&ping).expect("JSON serialization should never fail for valid struct");
                    // Send application-level JSON ping (Kraken keepalive)
                    if write_tx.send(Message::Text(msg_json)).is_ok() {
                        // Send WS-level ping for RTT measurement
                        *last_ping.lock().await = Instant::now();
                        let _ = write_tx.send(Message::Ping(vec![]));
                    } else {
                        break;
                    }
                }
            }
        });
    }

    /// Build channel and symbol for subscription
    fn build_subscription_params(request: &SubscriptionRequest, token: Option<&str>) -> (String, Vec<String>, SubscribeParams) {
        let channel = match &request.stream_type {
            StreamType::Ticker => "ticker",
            StreamType::Trade => "trade",
            StreamType::Orderbook | StreamType::OrderbookDelta => "book",
            StreamType::Kline { .. } => "ohlc",
            StreamType::MarkPrice => "ticker", // Kraken includes mark price in ticker
            StreamType::FundingRate => "ticker", // Kraken includes funding rate in ticker
            StreamType::OrderUpdate => "executions",
            StreamType::BalanceUpdate => "balances",
            StreamType::PositionUpdate => "executions", // Position updates in executions channel
        };

        // Format symbol as BTC/USD (Kraken v2 WebSocket format)
        // CRITICAL: v2 WebSocket uses BTC, not XBT!
        // This directly uses symbol.base and symbol.quote without conversion.
        // Caller must ensure symbol.base is "BTC" not "XBT" for Bitcoin.
        let symbol_str = format!("{}/{}", request.symbol.base, request.symbol.quote);

        let mut params = SubscribeParams {
            channel: channel.to_string(),
            symbol: vec![symbol_str.clone()],
            token: token.map(String::from),
            depth: None,
            interval: None,
            snapshot: None,  // Don't send snapshot parameter - let Kraken use defaults
            event_trigger: None,  // Don't send event_trigger - let Kraken use defaults
        };

        // Add channel-specific parameters ONLY when required
        match &request.stream_type {
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                params.depth = Some(request.depth.unwrap_or(10) as u16);
            }
            StreamType::Kline { interval } => {
                // Parse interval string to minutes
                let minutes = match interval.as_str() {
                    "1m" => 1,
                    "5m" => 5,
                    "15m" => 15,
                    "30m" => 30,
                    "1h" => 60,
                    "4h" => 240,
                    "1d" => 1440,
                    "1w" => 10080,
                    _ => 1,
                };
                params.interval = Some(minutes);
            }
            _ => {}
        }

        (channel.to_string(), vec![symbol_str], params)
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

    // ═══════════════════════════════════════════════════════════════════════════
    // PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    fn parse_ticker_ws(data: &Value) -> ExchangeResult<crate::core::Ticker> {
        // Parse ticker from WebSocket data array
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Ticker data not an array".to_string()))?;

        let ticker_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty ticker array".to_string()))?;

        let symbol = ticker_data.get("symbol")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?;

        let last_price = ticker_data.get("last")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let bid_price = ticker_data.get("bid")
            .and_then(|v| v.as_f64());

        let ask_price = ticker_data.get("ask")
            .and_then(|v| v.as_f64());

        let high_24h = ticker_data.get("high")
            .and_then(|v| v.as_f64());

        let low_24h = ticker_data.get("low")
            .and_then(|v| v.as_f64());

        let volume_24h = ticker_data.get("volume")
            .and_then(|v| v.as_f64());

        let change_pct = ticker_data.get("change_pct")
            .and_then(|v| v.as_f64());

        Ok(crate::core::Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: change_pct,
            timestamp: timestamp_millis() as i64,
        })
    }

    fn parse_trade_ws(data: &Value) -> ExchangeResult<crate::core::PublicTrade> {
        // Parse first trade from array
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Trade data not an array".to_string()))?;

        let trade_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty trade array".to_string()))?;

        let symbol = trade_data.get("symbol")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?;

        let price = trade_data.get("price")
            .and_then(|p| p.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid price".to_string()))?;

        let quantity = trade_data.get("qty")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid quantity".to_string()))?;

        let timestamp_str = trade_data.get("timestamp")
            .and_then(|t| t.as_str())
            .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))?;

        // Parse ISO timestamp to milliseconds
        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(timestamp_millis() as i64);

        let side = trade_data.get("side")
            .and_then(|s| s.as_str())
            .map(|s| match s {
                "buy" => TradeSide::Buy,
                "sell" => TradeSide::Sell,
                _ => TradeSide::Buy,
            })
            .unwrap_or(TradeSide::Buy);

        let id = trade_data.get("trade_id")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "0".to_string());

        Ok(crate::core::PublicTrade {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
            timestamp,
        })
    }

    fn parse_orderbook_ws(data: &Value, is_snapshot: bool) -> ExchangeResult<StreamEvent> {
        // Parse orderbook from array
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Orderbook data not an array".to_string()))?;

        let book_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty orderbook array".to_string()))?;

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            book_data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let price = level.get("price")?.as_f64()?;
                            let qty = level.get("qty")?.as_f64()?;
                            Some(OrderBookLevel::new(price, qty))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let bids = parse_levels("bids");
        let asks = parse_levels("asks");

        if is_snapshot {
            Ok(StreamEvent::OrderbookSnapshot(crate::core::OrderBook {
                timestamp: timestamp_millis() as i64,
                bids,
                asks,
                sequence: None,
                last_update_id: None,
                first_update_id: None,
                prev_update_id: None,
                event_time: None,
                transaction_time: None,
                checksum: None,
            }))
        } else {
            Ok(StreamEvent::OrderbookDelta(OrderbookDeltaData {
                bids,
                asks,
                timestamp: timestamp_millis() as i64,
                first_update_id: None,
                last_update_id: None,
                prev_update_id: None,
                event_time: None,
                checksum: None,
            }))
        }
    }

    fn parse_kline_ws(data: &Value) -> ExchangeResult<crate::core::Kline> {
        // Parse kline from array
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Kline data not an array".to_string()))?;

        let kline_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty kline array".to_string()))?;

        let timestamp_str = kline_data.get("timestamp")
            .and_then(|t| t.as_str())
            .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))?;

        let open_time = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(timestamp_millis() as i64);

        let open = kline_data.get("open")
            .and_then(|o| o.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid open".to_string()))?;

        let high = kline_data.get("high")
            .and_then(|h| h.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid high".to_string()))?;

        let low = kline_data.get("low")
            .and_then(|l| l.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid low".to_string()))?;

        let close = kline_data.get("close")
            .and_then(|c| c.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid close".to_string()))?;

        let volume = kline_data.get("volume")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid volume".to_string()))?;

        let trades = kline_data.get("trades")
            .and_then(|t| t.as_u64());

        Ok(crate::core::Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: None,
            close_time: None,
            trades,
        })
    }

    fn parse_execution_ws(data: &Value) -> ExchangeResult<StreamEvent> {
        // Parse order/execution update from array
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Execution data not an array".to_string()))?;

        let exec_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty execution array".to_string()))?;

        // Check if it's a trade execution or order update
        if exec_data.get("exec_id").is_some() {
            // Trade execution - convert to order update with fill info
            return Self::parse_trade_execution(exec_data);
        }

        // Order update
        let order_id = exec_data.get("order_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing order_id".to_string()))?;

        let client_order_id = exec_data.get("order_userref")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string());

        let symbol = exec_data.get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?;

        let side = exec_data.get("side")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "sell" => OrderSide::Sell,
                _ => OrderSide::Buy,
            })
            .unwrap_or(OrderSide::Buy);

        let order_type = exec_data.get("order_type")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "market" => OrderType::Market,
                _ => OrderType::Limit { price: 0.0 },
            })
            .unwrap_or(OrderType::Limit { price: 0.0 });

        let status = exec_data.get("order_status")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "canceled" | "cancelled" => OrderStatus::Canceled,
                "filled" => OrderStatus::Filled,
                "open" => {
                    let filled = exec_data.get("filled_qty").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if filled > 0.0 {
                        OrderStatus::PartiallyFilled
                    } else {
                        OrderStatus::New
                    }
                }
                _ => OrderStatus::New,
            })
            .unwrap_or(OrderStatus::New);

        let price = exec_data.get("limit_price")
            .and_then(|v| v.as_f64());

        let quantity = exec_data.get("order_qty")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let filled_quantity = exec_data.get("filled_qty")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let average_price = exec_data.get("avg_price")
            .and_then(|v| v.as_f64());

        let commission_asset = exec_data.get("fee_currency")
            .and_then(|v| v.as_str())
            .map(String::from);

        let timestamp_str = exec_data.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing timestamp".to_string()))?;

        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(timestamp_millis() as i64);

        Ok(StreamEvent::OrderUpdate(crate::core::OrderUpdateEvent {
            order_id: order_id.to_string(),
            client_order_id,
            symbol: symbol.to_string(),
            side,
            order_type,
            status,
            price,
            quantity,
            filled_quantity,
            average_price,
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset,
            trade_id: None,
            timestamp,
        }))
    }

    fn parse_trade_execution(data: &Value) -> ExchangeResult<StreamEvent> {
        // Trade execution (partial fill or full fill)
        let order_id = data.get("order_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing order_id".to_string()))?;

        let symbol = data.get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?;

        let side = data.get("side")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "sell" => OrderSide::Sell,
                _ => OrderSide::Buy,
            })
            .unwrap_or(OrderSide::Buy);

        let last_qty = data.get("last_qty")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let last_price = data.get("last_price")
            .and_then(|v| v.as_f64());

        let fee = data.get("fee")
            .and_then(|v| v.as_f64());

        let trade_id = data.get("exec_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let timestamp_str = data.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing timestamp".to_string()))?;

        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(timestamp_millis() as i64);

        Ok(StreamEvent::OrderUpdate(crate::core::OrderUpdateEvent {
            order_id: order_id.to_string(),
            client_order_id: None,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Limit { price: 0.0 },
            status: OrderStatus::PartiallyFilled,
            price: last_price,
            quantity: last_qty,
            filled_quantity: last_qty,
            average_price: last_price,
            last_fill_price: last_price,
            last_fill_quantity: Some(last_qty),
            last_fill_commission: fee,
            commission_asset: None,
            trade_id,
            timestamp,
        }))
    }

    fn parse_balance_ws(data: &Value) -> ExchangeResult<StreamEvent> {
        // Parse balance update from array
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Balance data not an array".to_string()))?;

        let balance_data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty balance array".to_string()))?;

        let asset = balance_data.get("asset")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing asset".to_string()))?;

        let balance = balance_data.get("balance")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        Ok(StreamEvent::BalanceUpdate(crate::core::BalanceUpdateEvent {
            asset: asset.to_string(),
            free: balance,
            locked: 0.0,
            total: balance,
            delta: None,
            reason: None,
            timestamp: timestamp_millis() as i64,
        }))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for KrakenWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        eprintln!("[KRAKEN WS] Connecting...");
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Determine if we need private connection
        let needs_private = self.token.is_some();

        // Connect WebSocket
        let ws_stream = self.connect_ws(needs_private).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Split the stream into read and write halves
        let (ws_writer, ws_reader) = ws_stream.split();

        *self.ws_writer.lock().await = Some(ws_writer);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Reset ping timer to ensure first ping happens within ping_interval from now
        *self.last_ping.lock().await = Instant::now();

        // Create broadcast channel and store sender
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Start message handler (returns write channel)
        let write_tx = Self::start_message_handler(
            self.ws_writer.clone(),
            ws_reader,
            self.event_tx.clone(),
            self.status.clone(),
            account_type,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Store write channel
        *self.write_tx.lock().await = Some(write_tx.clone());

        // Start ping task
        Self::start_ping_task(
            write_tx.clone(),
            self.ping_interval,
            self.last_ping.clone(),
            self.req_id_counter.clone(),
        );

        // Send immediate ping to confirm keepalive works
        let initial_ping = PingMessage {
            method: "ping".to_string(),
            req_id: Some(self.next_req_id().await),
        };
        let ping_json = serde_json::to_string(&initial_ping)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;
        write_tx.send(Message::Text(ping_json))
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to send initial ping: {}", e)))?;

        // Wait a tiny bit for initial status message
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_writer.lock().await = None;
        let _ = self.event_tx.lock().unwrap().take();
        *self.write_tx.lock().await = None;
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
        // Check connection status first
        let status = *self.status.lock().await;
        if status != ConnectionStatus::Connected {
            eprintln!("[KRAKEN WS] Subscribe failed: not connected (status: {:?})", status);
            return Err(WebSocketError::ConnectionError(format!("Not connected (status: {:?})", status)));
        }

        // Wait for rate limit (weight 1 for subscriptions)
        Self::ws_rate_limit_wait(1).await;

        let token = if Self::is_private(&request.stream_type) {
            self.token.as_deref()
        } else {
            None
        };

        let (_channel, _symbols, params) = Self::build_subscription_params(&request, token);

        // Don't send req_id - keep it minimal for compatibility
        let msg = SubscribeMessage {
            method: "subscribe".to_string(),
            params,
            req_id: None,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        // Use write channel instead of directly accessing stream
        let write_tx_guard = self.write_tx.lock().await;
        let write_tx = write_tx_guard.as_ref()
            .ok_or_else(|| {
                eprintln!("[KRAKEN WS] Subscribe failed: write channel not initialized");
                WebSocketError::ConnectionError("Not connected (write channel None)".to_string())
            })?;

        write_tx.send(Message::Text(msg_json))
            .map_err(|e| {
                eprintln!("[KRAKEN WS] Failed to send subscription: {}", e);
                WebSocketError::ConnectionError(format!("Write channel send failed: {}", e))
            })?;

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Wait for rate limit (weight 1 for unsubscriptions)
        Self::ws_rate_limit_wait(1).await;

        let token = if Self::is_private(&request.stream_type) {
            self.token.as_deref()
        } else {
            None
        };

        let (_channel, _symbols, params) = Self::build_subscription_params(&request, token);

        // Don't send req_id - keep it minimal for compatibility
        let msg = SubscribeMessage {
            method: "unsubscribe".to_string(),
            params,
            req_id: None,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        // Use write channel
        let write_tx_guard = self.write_tx.lock().await;
        let write_tx = write_tx_guard.as_ref()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected (write channel None)".to_string()))?;

        write_tx.send(Message::Text(msg_json))
            .map_err(|e| WebSocketError::ConnectionError(format!("Write channel send failed: {}", e)))?;

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
        // Use try_lock to avoid blocking
        match self.subscriptions.try_lock() {
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }

    fn orderbook_capabilities(&self) -> OrderbookCapabilities {
        static DEPTHS: &[u32] = &[10, 25, 100, 500, 1000];
        OrderbookCapabilities {
            ws_depths: DEPTHS,
            ws_default_depth: Some(10),
            rest_max_depth: Some(500),
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws = KrakenWebSocket::new(None, AccountType::Spot).await.unwrap();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_req_id_increment() {
        let ws = KrakenWebSocket::new(None, AccountType::Spot).await.unwrap();
        let id1 = ws.next_req_id().await;
        let id2 = ws.next_req_id().await;
        assert_eq!(id2, id1 + 1);
    }
}
