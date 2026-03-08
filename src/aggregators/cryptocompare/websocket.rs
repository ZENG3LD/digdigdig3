//! # CryptoCompare WebSocket Implementation
//!
//! WebSocket connector for CryptoCompare Streaming API v2.
//!
//! ## Features
//! - Public channels: Trade (0), Ticker/Current (2), Aggregate Ticker (5), OHLC (17)
//! - No API key required for basic access (limited rate)
//! - Automatic format selection based on API key availability
//! - Subscription management via SubAdd/SubRemove
//! - Message parsing to StreamEvent
//!
//! ## Dual Message Format Support
//!
//! The connector automatically selects the appropriate format:
//!
//! ### Streamer Format (No API Key)
//! - URL: `wss://streamer.cryptocompare.com/v2?format=streamer`
//! - Message format: Tilde-delimited strings (e.g., `5~CCCAGG~BTC~USD~1~78716.20~...`)
//! - Batched messages use pipe delimiter: `msg1|msg2|msg3`
//! - Works without authentication
//! - No bid/ask data in ticker (streamer format limitation)
//!
//! ### JSON Format (With API Key)
//! - URL: `wss://streamer.cryptocompare.com/v2?api_key={key}`
//! - Message format: JSON objects (e.g., `{"TYPE":"5","PRICE":78700,...}`)
//! - Includes bid/ask data in ticker
//! - Requires valid API key
//!
//! ## Channel Mapping
//!
//! CryptoCompare uses numeric channel types:
//! - `0~EXCHANGE~FSYM~TSYM` -> Trade
//! - `2~EXCHANGE~FSYM~TSYM` -> Ticker (exchange-specific)
//! - `5~CCCAGG~FSYM~TSYM` -> Aggregate Ticker (CCCAGG, volume-weighted)
//! - `17~EXCHANGE~FSYM~TSYM~INTERVAL` -> OHLC/Kline
//!
//! ## Protocol
//!
//! - Subscribe: `{"action":"SubAdd","subs":["5~CCCAGG~BTC~USD"]}`
//! - Unsubscribe: `{"action":"SubRemove","subs":["5~CCCAGG~BTC~USD"]}`
//! - Standard WebSocket ping/pong for keepalive

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::core::types::{
    Kline, PublicTrade, StreamType, Ticker, TradeSide, WebSocketError, WebSocketResult,
};
use crate::core::{AccountType, ConnectionStatus, StreamEvent, SubscriptionRequest};
use crate::core::traits::WebSocketConnector;

use super::auth::CryptoCompareAuth;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Default WebSocket URL for CryptoCompare streaming API v2
const WS_BASE_URL: &str = "wss://streamer.cryptocompare.com/v2";

/// Default exchange used for ticker/trade subscriptions when no specific exchange needed
const DEFAULT_EXCHANGE: &str = "CCCAGG";

// ═══════════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsWriter = SplitSink<WsStream, Message>;

// ═══════════════════════════════════════════════════════════════════════════════
// CRYPTOCOMPARE WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// CryptoCompare WebSocket connector
///
/// Connects to `wss://streamer.cryptocompare.com/v2` and provides
/// real-time market data (tickers, trades, OHLC) from CryptoCompare's
/// aggregated data streams.
///
/// Works without an API key for limited public access.
pub struct CryptoCompareWebSocket {
    /// Optional API key for enhanced rate limits
    auth: CryptoCompareAuth,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Internal event sender (message handler -> forwarder)
    event_tx: Arc<Mutex<Option<mpsc::UnboundedSender<WebSocketResult<StreamEvent>>>>>,
    /// Broadcast sender (for multiple consumers)
    broadcast_tx: Arc<broadcast::Sender<WebSocketResult<StreamEvent>>>,
    /// WebSocket write half
    ws_writer: Arc<Mutex<Option<WsWriter>>>,
    /// Whether to use streamer format (tilde-delimited) vs JSON format
    use_streamer_format: bool,
}

impl Default for CryptoCompareWebSocket {
    fn default() -> Self {
        Self::with_auth(CryptoCompareAuth::public())
    }
}

impl CryptoCompareWebSocket {
    /// Create new CryptoCompare WebSocket connector (public, no API key)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new CryptoCompare WebSocket connector with authentication
    pub fn with_auth(auth: CryptoCompareAuth) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        let use_streamer_format = auth.api_key.is_none();

        Self {
            auth,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(Mutex::new(None)),
            broadcast_tx: Arc::new(broadcast_tx),
            ws_writer: Arc::new(Mutex::new(None)),
            use_streamer_format,
        }
    }

    /// Build the WebSocket URL, including API key if available
    ///
    /// - With API key: use JSON format (default)
    /// - Without API key: use streamer format (tilde-delimited, no auth needed)
    fn ws_url(&self) -> String {
        match &self.auth.api_key {
            Some(key) => format!("{}?api_key={}", WS_BASE_URL, key),
            None => format!("{}?format=streamer", WS_BASE_URL),
        }
    }

    /// Build a CryptoCompare subscription string for a given request
    ///
    /// Returns the channel subscription string, e.g.:
    /// - Ticker: `5~CCCAGG~BTC~USD` (aggregate) or `2~Coinbase~BTC~USD` (exchange)
    /// - Trade: `0~CCCAGG~BTC~USD`
    /// - Kline: `17~CCCAGG~BTC~USD~1m`
    fn build_sub_string(request: &SubscriptionRequest) -> Result<String, WebSocketError> {
        let fsym = request.symbol.base.to_uppercase();
        let tsym = request.symbol.quote.to_uppercase();

        match &request.stream_type {
            StreamType::Ticker => {
                // Use aggregate ticker (channel 5) with CCCAGG exchange
                Ok(format!("5~{}~{}~{}", DEFAULT_EXCHANGE, fsym, tsym))
            }
            StreamType::Trade => {
                // Use trade channel (0) with CCCAGG aggregate
                Ok(format!("0~{}~{}~{}", DEFAULT_EXCHANGE, fsym, tsym))
            }
            StreamType::Kline { interval } => {
                // OHLC channel (17) with interval
                Ok(format!("17~{}~{}~{}~{}", DEFAULT_EXCHANGE, fsym, tsym, interval))
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                Err(WebSocketError::Subscription(
                    "CryptoCompare orderbook WebSocket requires paid tier".to_string(),
                ))
            }
            other => Err(WebSocketError::Subscription(format!(
                "Stream type {:?} not supported for CryptoCompare WebSocket",
                other
            ))),
        }
    }

    /// Send a subscribe or unsubscribe message
    async fn send_action(
        ws_writer: &Arc<Mutex<Option<WsWriter>>>,
        action: &str,
        subs: Vec<String>,
    ) -> WebSocketResult<()> {
        let msg = json!({
            "action": action,
            "subs": subs,
        });

        let json_str = msg.to_string();

        let mut writer_guard = ws_writer.lock().await;
        let writer = writer_guard
            .as_mut()
            .ok_or(WebSocketError::NotConnected)?;

        writer
            .send(Message::Text(json_str))
            .await
            .map_err(|e| WebSocketError::SendError(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    /// Start the message handler task that reads from the WebSocket
    fn start_message_handler(
        mut reader: futures_util::stream::SplitStream<WsStream>,
        ws_writer: Arc<Mutex<Option<WsWriter>>>,
        event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        status: Arc<Mutex<ConnectionStatus>>,
        use_streamer_format: bool,
    ) {
        tokio::spawn(async move {
            loop {
                match reader.next().await {
                    Some(Ok(Message::Text(text))) => {
                        Self::handle_message(&text, &event_tx, use_streamer_format);
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let mut writer_guard = ws_writer.lock().await;
                        if let Some(writer) = writer_guard.as_mut() {
                            let _ = writer.send(Message::Pong(data)).await;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Pong received, connection alive
                    }
                    Some(Ok(Message::Binary(_))) => {
                        // Not expected from CryptoCompare
                    }
                    Some(Ok(Message::Close(_))) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Raw frame, ignore
                    }
                    Some(Err(_e)) => {
                        let _ = event_tx.send(Err(WebSocketError::ConnectionError(
                            "WebSocket read error".to_string(),
                        )));
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    None => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                }
            }
        });
    }

    /// Start periodic ping task (every 30 seconds)
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsWriter>>>,
        status: Arc<Mutex<ConnectionStatus>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            interval.tick().await; // skip first immediate tick

            loop {
                interval.tick().await;

                let current_status = *status.lock().await;
                if current_status != ConnectionStatus::Connected {
                    break;
                }

                let mut writer_guard = ws_writer.lock().await;
                if let Some(writer) = writer_guard.as_mut() {
                    if writer.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Handle an incoming text message from CryptoCompare
    ///
    /// Supports two formats:
    /// - **JSON format** (with API key): `{"TYPE":"5","PRICE":78700,...}`
    /// - **Streamer format** (no key): `5~CCCAGG~BTC~USD~1~78700~...`
    ///
    /// Messages may be batched with `|` delimiter in streamer format.
    fn handle_message(
        text: &str,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        use_streamer_format: bool,
    ) {
        if use_streamer_format {
            Self::handle_streamer_message(text, event_tx);
        } else {
            Self::handle_json_message(text, event_tx);
        }
    }

    /// Handle streamer format message (tilde-delimited)
    ///
    /// Messages may be batched with pipe delimiter: `msg1|msg2|msg3`
    /// Each message is tilde-delimited: `TYPE~FIELD1~FIELD2~...`
    fn handle_streamer_message(
        text: &str,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
    ) {
        // Split on pipe for batched messages
        for msg in text.split('|') {
            let parts: Vec<&str> = msg.split('~').collect();

            match parts.first().copied() {
                Some("0") => {
                    // Trade
                    if let Some(event) = Self::parse_trade_streamer(&parts) {
                        let _ = event_tx.send(Ok(event));
                    }
                }
                Some("5") => {
                    // Aggregate ticker
                    if let Some(event) = Self::parse_ticker_streamer(&parts) {
                        let _ = event_tx.send(Ok(event));
                    }
                }
                Some("500") => {
                    // Error
                    let message = parts.get(1).unwrap_or(&"Unknown error");
                    let _ = event_tx.send(Err(WebSocketError::ProtocolError(
                        format!("CryptoCompare error: {}", message),
                    )));
                }
                Some("999") => {
                    // Heartbeat - ignore
                }
                Some("20") => {
                    // STREAMERWELCOME - ignore
                }
                Some("16") => {
                    // SUBSCRIBECOMPLETE - ignore
                }
                Some("3") => {
                    // LOADCOMPLETE - ignore
                }
                _ => {
                    // Unknown or system message - ignore
                }
            }
        }
    }

    /// Handle JSON format message (with API key)
    ///
    /// CryptoCompare sends JSON messages with a `TYPE` field indicating
    /// the channel type (0=Trade, 2=Current, 5=AggTicker, 17=OHLC, etc.)
    fn handle_json_message(
        text: &str,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
    ) {
        let json: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return, // Silently ignore unparseable messages
        };

        // CryptoCompare messages have a TYPE field
        let msg_type = match json.get("TYPE").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => {
                // Some messages use numeric TYPE
                match json.get("TYPE").and_then(|t| t.as_i64()) {
                    Some(n) => {
                        // Handle numeric types inline
                        match n {
                            0 => {
                                if let Some(event) = Self::parse_trade(&json) {
                                    let _ = event_tx.send(Ok(event));
                                }
                            }
                            2 | 5 => {
                                if let Some(event) = Self::parse_ticker(&json) {
                                    let _ = event_tx.send(Ok(event));
                                }
                            }
                            17 => {
                                if let Some(event) = Self::parse_ohlc(&json) {
                                    let _ = event_tx.send(Ok(event));
                                }
                            }
                            500 => {
                                // Error message from CryptoCompare
                                let message = json
                                    .get("MESSAGE")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown error");
                                let _ = event_tx.send(Err(WebSocketError::ProtocolError(
                                    format!("CryptoCompare error: {}", message),
                                )));
                            }
                            _ => {
                                // System or unknown channel, ignore
                            }
                        }
                        return;
                    }
                    None => return,
                }
            }
        };

        // Handle string-typed messages
        match msg_type {
            "0" => {
                if let Some(event) = Self::parse_trade(&json) {
                    let _ = event_tx.send(Ok(event));
                }
            }
            "2" | "5" => {
                if let Some(event) = Self::parse_ticker(&json) {
                    let _ = event_tx.send(Ok(event));
                }
            }
            "17" => {
                if let Some(event) = Self::parse_ohlc(&json) {
                    let _ = event_tx.send(Ok(event));
                }
            }
            "500" => {
                let message = json
                    .get("MESSAGE")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                let _ = event_tx.send(Err(WebSocketError::ProtocolError(format!(
                    "CryptoCompare error: {}",
                    message
                ))));
            }
            _ => {
                // System messages (8, 11, 20, 999, etc.) -- ignore
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse channel 0 (Trade) message to StreamEvent
    fn parse_trade(json: &Value) -> Option<StreamEvent> {
        let fsym = json.get("FSYM").and_then(|v| v.as_str())?;
        let tsym = json.get("TSYM").and_then(|v| v.as_str())?;
        let price = Self::extract_f64(json, "P")?;
        let quantity = Self::extract_f64(json, "Q").unwrap_or(0.0);
        let timestamp = json.get("TS").and_then(|v| v.as_i64()).unwrap_or(0);
        let trade_id = json
            .get("ID")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // CryptoCompare flags: bit 0x1 = buy, bit 0x2 = sell
        let flags = json
            .get("F")
            .and_then(|v| v.as_str())
            .and_then(|s| {
                if let Some(hex) = s.strip_prefix("0x") {
                    u64::from_str_radix(hex, 16).ok()
                } else {
                    s.parse::<u64>().ok()
                }
            })
            .unwrap_or(0);

        let side = if flags & 0x2 != 0 {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };

        Some(StreamEvent::Trade(PublicTrade {
            id: trade_id,
            symbol: format!("{}{}", fsym, tsym),
            price,
            quantity,
            side,
            timestamp: timestamp * 1000, // Convert seconds to milliseconds
        }))
    }

    /// Parse channel 2/5 (Ticker/Current) message to StreamEvent
    fn parse_ticker(json: &Value) -> Option<StreamEvent> {
        // Channel 5 uses FROMSYMBOL/TOSYMBOL, channel 2 uses FSYM/TSYM
        let fsym = json
            .get("FROMSYMBOL")
            .or_else(|| json.get("FSYM"))
            .and_then(|v| v.as_str())?;
        let tsym = json
            .get("TOSYMBOL")
            .or_else(|| json.get("TSYM"))
            .and_then(|v| v.as_str())?;

        let price = Self::extract_f64(json, "PRICE")?;
        let timestamp = json.get("LASTUPDATE").and_then(|v| v.as_i64()).unwrap_or(0);

        Some(StreamEvent::Ticker(Ticker {
            symbol: format!("{}{}", fsym, tsym),
            last_price: price,
            bid_price: Self::extract_f64(json, "BID"),
            ask_price: Self::extract_f64(json, "OFFER"),
            high_24h: Self::extract_f64(json, "HIGH24HOUR")
                .or_else(|| Self::extract_f64(json, "HIGHDAY")),
            low_24h: Self::extract_f64(json, "LOW24HOUR")
                .or_else(|| Self::extract_f64(json, "LOWDAY")),
            volume_24h: Self::extract_f64(json, "VOLUME24HOUR")
                .or_else(|| Self::extract_f64(json, "VOLUMEDAY")),
            quote_volume_24h: Self::extract_f64(json, "VOLUME24HOURTO")
                .or_else(|| Self::extract_f64(json, "VOLUMEDAYTO")),
            price_change_24h: {
                let open = Self::extract_f64(json, "OPEN24HOUR")
                    .or_else(|| Self::extract_f64(json, "OPENDAY"));
                open.map(|o| price - o)
            },
            price_change_percent_24h: {
                let open = Self::extract_f64(json, "OPEN24HOUR")
                    .or_else(|| Self::extract_f64(json, "OPENDAY"));
                open.filter(|&o| o > 0.0).map(|o| ((price - o) / o) * 100.0)
            },
            timestamp: timestamp * 1000, // Convert seconds to milliseconds
        }))
    }

    /// Parse channel 17 (OHLC) message to StreamEvent
    fn parse_ohlc(json: &Value) -> Option<StreamEvent> {
        let open = Self::extract_f64(json, "OPEN")?;
        let high = Self::extract_f64(json, "HIGH")?;
        let low = Self::extract_f64(json, "LOW")?;
        let close = Self::extract_f64(json, "CLOSE")?;
        let volume = Self::extract_f64(json, "VOLUME").unwrap_or(0.0);
        let timestamp = json.get("TS").and_then(|v| v.as_i64()).unwrap_or(0);

        Some(StreamEvent::Kline(Kline {
            open_time: timestamp * 1000,
            open,
            high,
            low,
            close,
            volume,
            close_time: None,
            quote_volume: Self::extract_f64(json, "VOLUMETO"),
            trades: None,
        }))
    }

    /// Extract f64 value from JSON, handling both numeric and string representations
    fn extract_f64(json: &Value, key: &str) -> Option<f64> {
        json.get(key).and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // STREAMER FORMAT PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse streamer format trade message (Type 0)
    ///
    /// Format: `0~EXCHANGE~FSYM~TSYM~FLAGS~ID~TIMESTAMP~QUANTITY~PRICE~TOTAL~...`
    ///
    /// Fields:
    /// - [0]=TYPE, [1]=EXCHANGE, [2]=FSYM, [3]=TSYM, [4]=FLAGS
    /// - [5]=ID, [6]=TIMESTAMP, [7]=QUANTITY, [8]=PRICE, [9]=TOTAL
    /// - FLAGS: 1=buy, 2=sell
    fn parse_trade_streamer(parts: &[&str]) -> Option<StreamEvent> {
        if parts.len() < 9 {
            return None;
        }

        let _exchange = parts.get(1)?; // Not used in current implementation
        let fsym = parts.get(2)?;
        let tsym = parts.get(3)?;
        let flags = parts.get(4)?.parse::<u32>().ok()?;
        let trade_id = parts.get(5)?.to_string();
        let timestamp = parts.get(6)?.parse::<i64>().ok()?;
        let quantity = parts.get(7)?.parse::<f64>().ok()?;
        let price = parts.get(8)?.parse::<f64>().ok()?;

        // FLAGS: 1=buy, 2=sell
        let side = if flags & 0x2 != 0 {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };

        Some(StreamEvent::Trade(PublicTrade {
            id: trade_id,
            symbol: format!("{}{}", fsym, tsym),
            price,
            quantity,
            side,
            timestamp: timestamp * 1000, // Convert seconds to milliseconds
        }))
    }

    /// Parse streamer format ticker message (Type 5)
    ///
    /// Format: `5~MARKET~FSYM~TSYM~FLAGS~PRICE~LASTUPDATE~...~VOLUME24H~VOLUME24HTO~OPEN24H~HIGH24H~LOW24H~...`
    ///
    /// Fields (at least 18 for basic ticker):
    /// - [0]=TYPE, [1]=MARKET, [2]=FSYM, [3]=TSYM, [4]=FLAGS
    /// - [5]=PRICE, [6]=LASTUPDATE
    /// - [11]=VOLUME24H, [12]=VOLUME24HTO
    /// - [15]=OPEN24H, [16]=HIGH24H, [17]=LOW24H
    ///
    /// Note: Not all fields are present in every update. The message may contain
    /// partial updates with only changed fields.
    fn parse_ticker_streamer(parts: &[&str]) -> Option<StreamEvent> {
        // Minimum fields needed: TYPE, MARKET, FSYM, TSYM, FLAGS, PRICE, LASTUPDATE
        if parts.len() < 7 {
            return None;
        }

        let fsym = parts.get(2)?;
        let tsym = parts.get(3)?;
        let price = parts.get(5)?.parse::<f64>().ok()?;
        let timestamp = parts.get(6)?.parse::<i64>().ok()?;

        // Optional fields - may not be present in partial updates
        let volume_24h = parts.get(11).and_then(|s| s.parse::<f64>().ok());
        let volume_24h_to = parts.get(12).and_then(|s| s.parse::<f64>().ok());
        let open_24h = parts.get(15).and_then(|s| s.parse::<f64>().ok());
        let high_24h = parts.get(16).and_then(|s| s.parse::<f64>().ok());
        let low_24h = parts.get(17).and_then(|s| s.parse::<f64>().ok());

        Some(StreamEvent::Ticker(Ticker {
            symbol: format!("{}{}", fsym, tsym),
            last_price: price,
            bid_price: None, // Not available in streamer format
            ask_price: None, // Not available in streamer format
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: volume_24h_to,
            price_change_24h: open_24h.map(|o| price - o),
            price_change_percent_24h: open_24h
                .filter(|&o| o > 0.0)
                .map(|o| ((price - o) / o) * 100.0),
            timestamp: timestamp * 1000, // Convert seconds to milliseconds
        }))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for CryptoCompareWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;

        let url = self.ws_url();
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to connect: {}", e)))?;

        let (writer, reader) = ws_stream.split();

        *self.ws_writer.lock().await = Some(writer);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Create event channel
        let (tx, mut rx) = mpsc::unbounded_channel();
        *self.event_tx.lock().await = Some(tx.clone());

        // Start message handler
        Self::start_message_handler(
            reader,
            self.ws_writer.clone(),
            tx,
            self.status.clone(),
            self.use_streamer_format,
        );

        // Start ping task (30s interval)
        Self::start_ping_task(self.ws_writer.clone(), self.status.clone());

        // Forward mpsc -> broadcast
        let broadcast_tx = self.broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let _ = broadcast_tx.send(event);
            }
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;

        // Close the writer to signal the handler task to stop
        let mut writer_guard = self.ws_writer.lock().await;
        if let Some(mut writer) = writer_guard.take() {
            let _ = writer.close().await;
        }

        *self.event_tx.lock().await = None;
        self.subscriptions.lock().await.clear();

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(guard) => *guard,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let sub_string = Self::build_sub_string(&request)?;

        Self::send_action(&self.ws_writer, "SubAdd", vec![sub_string]).await?;

        self.subscriptions.lock().await.insert(request);
        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let sub_string = Self::build_sub_string(&request)?;

        Self::send_action(&self.ws_writer, "SubRemove", vec![sub_string]).await?;

        self.subscriptions.lock().await.remove(&request);
        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.subscribe();
        Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(|res| async move { res.ok() }),
        )
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_lock() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Symbol;

    #[test]
    fn test_websocket_creation() {
        let ws = CryptoCompareWebSocket::new();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_ws_url_public() {
        let ws = CryptoCompareWebSocket::new();
        assert_eq!(ws.ws_url(), "wss://streamer.cryptocompare.com/v2?format=streamer");
    }

    #[test]
    fn test_ws_url_with_key() {
        let auth = CryptoCompareAuth::new("test_key_123");
        let ws = CryptoCompareWebSocket::with_auth(auth);
        assert_eq!(
            ws.ws_url(),
            "wss://streamer.cryptocompare.com/v2?api_key=test_key_123"
        );
    }

    #[test]
    fn test_build_sub_string_ticker() {
        let req = SubscriptionRequest::ticker(Symbol::new("BTC", "USD"));
        let sub = CryptoCompareWebSocket::build_sub_string(&req).unwrap();
        assert_eq!(sub, "5~CCCAGG~BTC~USD");
    }

    #[test]
    fn test_build_sub_string_trade() {
        let req = SubscriptionRequest::trade(Symbol::new("ETH", "USDT"));
        let sub = CryptoCompareWebSocket::build_sub_string(&req).unwrap();
        assert_eq!(sub, "0~CCCAGG~ETH~USDT");
    }

    #[test]
    fn test_build_sub_string_kline() {
        let req = SubscriptionRequest::kline(Symbol::new("BTC", "USD"), "1h");
        let sub = CryptoCompareWebSocket::build_sub_string(&req).unwrap();
        assert_eq!(sub, "17~CCCAGG~BTC~USD~1h");
    }

    #[test]
    fn test_build_sub_string_orderbook_fails() {
        let req = SubscriptionRequest::orderbook(Symbol::new("BTC", "USD"));
        let result = CryptoCompareWebSocket::build_sub_string(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ticker_channel5() {
        let json: Value = serde_json::from_str(
            r#"{
                "TYPE": "5",
                "FROMSYMBOL": "BTC",
                "TOSYMBOL": "USD",
                "PRICE": 84023.50,
                "LASTUPDATE": 1706280000,
                "HIGH24HOUR": 85000.0,
                "LOW24HOUR": 83000.0,
                "VOLUME24HOUR": 1500.5,
                "VOLUME24HOURTO": 126000000.0,
                "OPEN24HOUR": 83500.0
            }"#,
        )
        .unwrap();

        let event = CryptoCompareWebSocket::parse_ticker(&json);
        assert!(event.is_some());

        if let Some(StreamEvent::Ticker(ticker)) = event {
            assert_eq!(ticker.symbol, "BTCUSD");
            assert_eq!(ticker.last_price, 84023.50);
            assert_eq!(ticker.high_24h, Some(85000.0));
            assert_eq!(ticker.low_24h, Some(83000.0));
            assert_eq!(ticker.volume_24h, Some(1500.5));
        } else {
            panic!("Expected StreamEvent::Ticker");
        }
    }

    #[test]
    fn test_parse_trade_channel0() {
        let json: Value = serde_json::from_str(
            r#"{
                "TYPE": "0",
                "FSYM": "BTC",
                "TSYM": "USDT",
                "P": 45000.50,
                "Q": 0.5,
                "TS": 1706280000,
                "ID": "123456",
                "F": "0x1"
            }"#,
        )
        .unwrap();

        let event = CryptoCompareWebSocket::parse_trade(&json);
        assert!(event.is_some());

        if let Some(StreamEvent::Trade(trade)) = event {
            assert_eq!(trade.symbol, "BTCUSDT");
            assert_eq!(trade.price, 45000.50);
            assert_eq!(trade.quantity, 0.5);
            assert_eq!(trade.id, "123456");
        } else {
            panic!("Expected StreamEvent::Trade");
        }
    }

    #[test]
    fn test_parse_ohlc_channel17() {
        let json: Value = serde_json::from_str(
            r#"{
                "TYPE": "17",
                "OPEN": 45000.0,
                "HIGH": 45100.0,
                "LOW": 44950.0,
                "CLOSE": 45050.0,
                "VOLUME": 125.5,
                "VOLUMETO": 5650000.0,
                "TS": 1706280000
            }"#,
        )
        .unwrap();

        let event = CryptoCompareWebSocket::parse_ohlc(&json);
        assert!(event.is_some());

        if let Some(StreamEvent::Kline(kline)) = event {
            assert_eq!(kline.open, 45000.0);
            assert_eq!(kline.high, 45100.0);
            assert_eq!(kline.low, 44950.0);
            assert_eq!(kline.close, 45050.0);
            assert_eq!(kline.volume, 125.5);
            assert_eq!(kline.quote_volume, Some(5650000.0));
        } else {
            panic!("Expected StreamEvent::Kline");
        }
    }

    #[test]
    fn test_parse_trade_streamer() {
        // Format: 0~EXCHANGE~FSYM~TSYM~FLAGS~ID~TIMESTAMP~QUANTITY~PRICE~TOTAL
        let parts: Vec<&str> = vec![
            "0",
            "Coinbase",
            "BTC",
            "USD",
            "2",         // Sell flag
            "947952988",
            "1769917571",
            "0.00023",
            "78706.05",
            "18.1023915",
        ];

        let event = CryptoCompareWebSocket::parse_trade_streamer(&parts);
        assert!(event.is_some());

        if let Some(StreamEvent::Trade(trade)) = event {
            assert_eq!(trade.symbol, "BTCUSD");
            assert_eq!(trade.price, 78706.05);
            assert_eq!(trade.quantity, 0.00023);
            assert_eq!(trade.id, "947952988");
            assert_eq!(trade.side, TradeSide::Sell);
            assert_eq!(trade.timestamp, 1769917571000); // Converted to ms
        } else {
            panic!("Expected StreamEvent::Trade");
        }
    }

    #[test]
    fn test_parse_ticker_streamer() {
        // Format: 5~MARKET~FSYM~TSYM~FLAGS~PRICE~LASTUPDATE~...~VOL24H~VOL24HTO~OPEN24H~HIGH24H~LOW24H
        let parts: Vec<&str> = vec![
            "5",
            "CCCAGG",
            "BTC",
            "USD",
            "1",
            "78716.20", // [5] PRICE
            "1769917542", // [6] LASTUPDATE
            "78716.20", // [7] LASTVOLUME_PRICE
            "0.00023", // [8] LASTVOLUME
            "18.10", // [9] LASTVOLUMETO
            "947952988", // [10] LASTTRADEID
            "1500.5", // [11] VOLUME24H
            "118074300.0", // [12] VOLUME24HTO
            "1200.0", // [13] VOLUMEDAY
            "94459440.0", // [14] VOLUMEDAYTO
            "78000.0", // [15] OPEN24HOUR
            "79000.0", // [16] HIGH24HOUR
            "77500.0", // [17] LOW24HOUR
        ];

        let event = CryptoCompareWebSocket::parse_ticker_streamer(&parts);
        assert!(event.is_some());

        if let Some(StreamEvent::Ticker(ticker)) = event {
            assert_eq!(ticker.symbol, "BTCUSD");
            assert_eq!(ticker.last_price, 78716.20);
            assert_eq!(ticker.high_24h, Some(79000.0));
            assert_eq!(ticker.low_24h, Some(77500.0));
            assert_eq!(ticker.volume_24h, Some(1500.5));
            assert_eq!(ticker.quote_volume_24h, Some(118074300.0));
            // price_change = 78716.20 - 78000.0 = 716.20
            assert_eq!(ticker.price_change_24h, Some(716.20));
            assert!(ticker.bid_price.is_none()); // Not available in streamer format
            assert!(ticker.ask_price.is_none()); // Not available in streamer format
            assert_eq!(ticker.timestamp, 1769917542000); // Converted to ms
        } else {
            panic!("Expected StreamEvent::Ticker");
        }
    }

    #[test]
    fn test_parse_ticker_streamer_partial_update() {
        // Partial update with only first 7 fields
        let parts: Vec<&str> = vec![
            "5",
            "CCCAGG",
            "ETH",
            "USDT",
            "1",
            "2850.50", // PRICE
            "1769917600", // LASTUPDATE
        ];

        let event = CryptoCompareWebSocket::parse_ticker_streamer(&parts);
        assert!(event.is_some());

        if let Some(StreamEvent::Ticker(ticker)) = event {
            assert_eq!(ticker.symbol, "ETHUSDT");
            assert_eq!(ticker.last_price, 2850.50);
            // Optional fields should be None in partial updates
            assert!(ticker.high_24h.is_none());
            assert!(ticker.low_24h.is_none());
            assert!(ticker.volume_24h.is_none());
        } else {
            panic!("Expected StreamEvent::Ticker");
        }
    }
}
