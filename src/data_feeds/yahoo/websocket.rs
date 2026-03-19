//! Yahoo Finance WebSocket implementation
//!
//! Connects to `wss://streamer.finance.yahoo.com/` for real-time price streaming.
//!
//! ## Protocol
//! - Subscribe: send JSON `{"subscribe":["AAPL","BTC-USD"]}`
//! - Unsubscribe: send JSON `{"unsubscribe":["AAPL"]}`
//! - Responses: JSON envelopes with base64-encoded protobuf payload (Text frames):
//!   `{"type":"pricing","message":"<base64-protobuf>"}`
//! - No authentication required
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::aggregators::yahoo::YahooFinanceWebSocket;
//! use connectors_v5::core::types::{Symbol, AccountType, StreamType, SubscriptionRequest};
//! use connectors_v5::core::traits::{WebSocketConnector, WebSocketExt};
//! use futures_util::StreamExt;
//!
//! let mut ws = YahooFinanceWebSocket::new();
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe_ticker(Symbol::new("AAPL", "USD")).await?;
//!
//! let mut stream = ws.event_stream();
//! while let Some(Ok(event)) = stream.next().await {
//!     println!("{:?}", event);
//! }
//! ```

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};

use async_trait::async_trait;
use base64::Engine;
use futures_util::{Stream, StreamExt, SinkExt};
use serde_json::json;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::core::types::{
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest, Symbol, Ticker,
    WebSocketError, WebSocketResult,
};
use crate::core::traits::WebSocketConnector;

use super::endpoints::format_symbol;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Yahoo Finance WebSocket URL (version=2 sends JSON-wrapped base64 protobuf)
const WS_URL: &str = "wss://streamer.finance.yahoo.com/?version=2";

// ═══════════════════════════════════════════════════════════════════════════════
// PROTOBUF DECODER
// ═══════════════════════════════════════════════════════════════════════════════

/// Decoded PricingData from Yahoo Finance protobuf messages
#[derive(Debug, Clone, Default)]
struct PricingData {
    id: Option<String>,
    price: Option<f32>,
    time: Option<i64>,
    currency: Option<String>,
    exchange: Option<String>,
    change_percent: Option<f32>,
    day_high: Option<f32>,
    day_low: Option<f32>,
    day_open: Option<f32>,
    previous_close: Option<f32>,
    bid: Option<f32>,
    ask: Option<f32>,
    bid_size: Option<i64>,
    ask_size: Option<i64>,
    volume: Option<i64>,
    change: Option<f32>,
    short_name: Option<String>,
    market_state: Option<String>,
}

/// Protobuf wire types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    Fixed32 = 5,
}

impl WireType {
    fn from_u64(val: u64) -> Option<Self> {
        match val {
            0 => Some(Self::Varint),
            1 => Some(Self::Fixed64),
            2 => Some(Self::LengthDelimited),
            5 => Some(Self::Fixed32),
            _ => None,
        }
    }
}

/// Decode a varint from the buffer, returning (value, bytes_consumed)
fn decode_varint(buf: &[u8]) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0u32;

    for (i, &byte) in buf.iter().enumerate() {
        if shift >= 64 {
            return None; // Overflow protection
        }
        result |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            return Some((result, i + 1));
        }
    }
    None // Unterminated varint
}

/// Decode a signed varint (sint64) using ZigZag encoding
fn decode_sint64(val: u64) -> i64 {
    ((val >> 1) as i64) ^ -((val & 1) as i64)
}

/// Decode a float (f32) from 4 bytes little-endian
fn decode_f32(buf: &[u8]) -> Option<f32> {
    if buf.len() < 4 {
        return None;
    }
    let bytes: [u8; 4] = [buf[0], buf[1], buf[2], buf[3]];
    Some(f32::from_le_bytes(bytes))
}

/// Decode a double (f64) from 8 bytes little-endian
fn decode_f64(buf: &[u8]) -> Option<f64> {
    if buf.len() < 8 {
        return None;
    }
    let bytes: [u8; 8] = [buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]];
    Some(f64::from_le_bytes(bytes))
}

/// Parse protobuf binary data into PricingData
///
/// Uses the schema from Yahoo Finance research docs.
/// Field numbers match the PricingData protobuf schema documented in
/// `research/websocket_full.md`.
fn parse_protobuf(data: &[u8]) -> Option<PricingData> {
    let mut result = PricingData::default();
    let mut pos = 0;

    while pos < data.len() {
        // Read tag (field_number << 3 | wire_type)
        let (tag, consumed) = decode_varint(&data[pos..])?;
        pos += consumed;

        let field_number = tag >> 3;
        let wire_type = WireType::from_u64(tag & 0x07)?;

        match wire_type {
            WireType::Varint => {
                let (val, consumed) = decode_varint(&data[pos..])?;
                pos += consumed;

                match field_number {
                    3 => result.time = Some(decode_sint64(val)),
                    6 | 7 | 21 | 22 => { /* int32 fields we don't need */ }
                    15 | 16 => {
                        // bid_size (15), ask_size (16)
                        match field_number {
                            15 => result.bid_size = Some(val as i64),
                            16 => result.ask_size = Some(val as i64),
                            _ => {}
                        }
                    }
                    17 => result.volume = Some(val as i64),
                    _ => { /* skip unknown varint fields */ }
                }
            }
            WireType::Fixed64 => {
                if pos + 8 > data.len() {
                    return Some(result); // Truncated but return what we have
                }
                let _val = decode_f64(&data[pos..]);
                pos += 8;
                // No f64 fields we need for ticker currently
            }
            WireType::LengthDelimited => {
                let (len, consumed) = decode_varint(&data[pos..])?;
                pos += consumed;
                let len = len as usize;

                if pos + len > data.len() {
                    return Some(result); // Truncated
                }

                let field_data = &data[pos..pos + len];
                pos += len;

                match field_number {
                    1 => result.id = String::from_utf8(field_data.to_vec()).ok(),
                    4 => result.currency = String::from_utf8(field_data.to_vec()).ok(),
                    5 => result.exchange = String::from_utf8(field_data.to_vec()).ok(),
                    19 => result.short_name = String::from_utf8(field_data.to_vec()).ok(),
                    20 | 23 | 28 => { /* string fields we skip: exchange_name, tradeable, tz_name */ }
                    30 => result.market_state = String::from_utf8(field_data.to_vec()).ok(),
                    _ => { /* skip unknown length-delimited fields */ }
                }
            }
            WireType::Fixed32 => {
                if pos + 4 > data.len() {
                    return Some(result);
                }

                let val = decode_f32(&data[pos..]);
                pos += 4;

                match field_number {
                    2 => result.price = val,
                    8 => result.change_percent = val,
                    9 => result.day_high = val,
                    10 => result.day_low = val,
                    11 => result.day_open = val,
                    12 => result.previous_close = val,
                    13 => result.bid = val,
                    14 => result.ask = val,
                    18 => result.change = val,
                    24 | 25 | 26 | 29 | 32 | 33 | 36 | 37 => {
                        // Various float fields we don't need for Ticker
                    }
                    _ => { /* skip unknown fixed32 fields */ }
                }
            }
        }
    }

    // Only return if we at least got an id
    if result.id.is_some() {
        Some(result)
    } else {
        None
    }
}

/// Convert PricingData to StreamEvent::Ticker
fn pricing_data_to_ticker(data: &PricingData) -> StreamEvent {
    let symbol = data.id.clone().unwrap_or_default();

    StreamEvent::Ticker(Ticker {
        symbol,
        last_price: data.price.unwrap_or(0.0) as f64,
        bid_price: data.bid.map(|v| v as f64),
        ask_price: data.ask.map(|v| v as f64),
        high_24h: data.day_high.map(|v| v as f64),
        low_24h: data.day_low.map(|v| v as f64),
        volume_24h: data.volume.map(|v| v as f64),
        quote_volume_24h: None,
        price_change_24h: data.change.map(|v| v as f64),
        price_change_percent_24h: data.change_percent.map(|v| v as f64),
        timestamp: data.time.unwrap_or_else(|| chrono::Utc::now().timestamp()),
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// YAHOO FINANCE WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Yahoo Finance WebSocket connector for real-time price streaming
///
/// Supports:
/// - Ticker subscriptions (real-time price updates)
/// - Multiple symbols per connection
/// - No authentication required
/// - Protobuf-encoded binary messages (decoded automatically)
///
/// ## Architecture
///
/// The connector uses a command channel to avoid deadlocks between the message
/// reader loop and subscribe/unsubscribe calls. The spawned handler task owns
/// the WebSocket stream exclusively and receives outbound messages (subscribe,
/// unsubscribe) via an `mpsc` channel.
pub struct YahooFinanceWebSocket {
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Subscribed Yahoo symbols (raw format like "AAPL", "BTC-USD")
    yahoo_symbols: Arc<Mutex<HashSet<String>>>,
    /// Event broadcast sender — uses std::sync::Mutex so event_stream() can subscribe
    /// without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Channel for sending outbound WS messages (subscribe/unsubscribe JSON)
    /// from the caller to the handler task that owns the stream.
    cmd_tx: Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>,
}

impl YahooFinanceWebSocket {
    /// Create a new Yahoo Finance WebSocket connector
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            yahoo_symbols: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            cmd_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Convert a Symbol to Yahoo Finance format for WebSocket subscription
    fn symbol_to_yahoo(symbol: &Symbol) -> String {
        format_symbol(&symbol.base, &symbol.quote)
    }

    /// Try to decode a protobuf payload from raw bytes and emit a ticker event.
    /// Returns `true` if a valid PricingData was decoded and sent.
    fn try_decode_and_emit(
        data: &[u8],
        event_tx: &broadcast::Sender<WebSocketResult<StreamEvent>>,
    ) -> bool {
        if let Some(pricing_data) = parse_protobuf(data) {
            tracing::debug!(
                "Yahoo WS decoded protobuf: id={:?} price={:?} time={:?}",
                pricing_data.id,
                pricing_data.price,
                pricing_data.time,
            );
            let event = pricing_data_to_ticker(&pricing_data);
            let _ = event_tx.send(Ok(event));
            true
        } else {
            false
        }
    }

    /// Start the message handling loop.
    ///
    /// This task exclusively owns the WebSocket stream. It reads incoming
    /// messages and also polls the command channel for outbound messages
    /// to send (subscribe/unsubscribe). Using `tokio::select!` ensures
    /// neither direction blocks the other.
    fn start_message_handler(
        mut ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        mut cmd_rx: mpsc::UnboundedReceiver<String>,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
    ) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Incoming WS message
                    msg = ws_stream.next() => {
                        match msg {
                            Some(Ok(Message::Binary(data))) => {
                                tracing::trace!(
                                    "Yahoo WS binary message: {} bytes",
                                    data.len()
                                );
                                if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                                    Self::try_decode_and_emit(&data, tx);
                                }
                            }
                            Some(Ok(Message::Text(text))) => {
                                tracing::debug!(
                                    "Yahoo WS text message ({} chars): {}",
                                    text.len(),
                                    if text.len() > 200 { &text[..200] } else { &text }
                                );

                                // Yahoo WS v2 sends JSON envelopes:
                                //   {"type":"pricing","message":"<base64-protobuf>"}
                                // Extract the base64 payload from the JSON envelope,
                                // or fall back to treating the whole text as raw base64.
                                let b64_payload: Option<String> = serde_json::from_str::<serde_json::Value>(&text)
                                    .ok()
                                    .and_then(|v| v.get("message").and_then(|m| m.as_str().map(String::from)));

                                let b64_str = b64_payload.as_deref().unwrap_or(&text);

                                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(b64_str.as_bytes()) {
                                    let decoded_ok = if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                                        Self::try_decode_and_emit(&decoded, tx)
                                    } else {
                                        false
                                    };
                                    if !decoded_ok {
                                        tracing::debug!(
                                            "Yahoo WS: base64 decoded {} bytes but protobuf parse failed",
                                            decoded.len()
                                        );
                                    }
                                } else {
                                    // Not base64 at all -- might be a JSON error or info message
                                    tracing::debug!(
                                        "Yahoo WS: non-base64 text: {}",
                                        if text.len() > 200 { &text[..200] } else { &text }
                                    );
                                }
                            }
                            Some(Ok(Message::Ping(ping))) => {
                                tracing::trace!("Yahoo WS: received ping");
                                if let Err(e) = ws_stream.send(Message::Pong(ping)).await {
                                    tracing::warn!("Yahoo WS: failed to send pong: {}", e);
                                    if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                                        let _ = tx.send(Err(
                                            WebSocketError::ConnectionError(e.to_string()),
                                        ));
                                    }
                                    break;
                                }
                            }
                            Some(Ok(Message::Pong(_))) => {
                                tracing::trace!("Yahoo WS: received pong");
                            }
                            Some(Ok(Message::Close(frame))) => {
                                tracing::info!("Yahoo WS: server sent close frame: {:?}", frame);
                                *status.lock().await = ConnectionStatus::Disconnected;
                                break;
                            }
                            Some(Ok(Message::Frame(_))) => {
                                // Raw frame, ignore
                            }
                            Some(Err(e)) => {
                                tracing::warn!("Yahoo WS: read error: {}", e);
                                if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                                    let _ = tx.send(Err(
                                        WebSocketError::ConnectionError(e.to_string()),
                                    ));
                                }
                                *status.lock().await = ConnectionStatus::Disconnected;
                                break;
                            }
                            None => {
                                tracing::info!("Yahoo WS: stream ended");
                                *status.lock().await = ConnectionStatus::Disconnected;
                                break;
                            }
                        }
                    }
                    // Outbound command (subscribe/unsubscribe JSON text)
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(text) => {
                                tracing::debug!("Yahoo WS: sending command: {}", text);
                                if let Err(e) = ws_stream.send(Message::Text(text)).await {
                                    tracing::warn!("Yahoo WS: failed to send command: {}", e);
                                    if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                                        let _ = tx.send(Err(
                                            WebSocketError::SendError(e.to_string()),
                                        ));
                                    }
                                }
                            }
                            None => {
                                // Command channel closed -- disconnect requested
                                tracing::info!("Yahoo WS: command channel closed, shutting down");
                                let _ = ws_stream.close(None).await;
                                *status.lock().await = ConnectionStatus::Disconnected;
                                break;
                            }
                        }
                    }
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            // from .next(). Without this, a clean close leaves the sender alive
            // and the bridge hangs forever instead of reconnecting.
            let _ = event_tx.lock().unwrap().take();
        });
    }
}

impl Default for YahooFinanceWebSocket {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for YahooFinanceWebSocket {
    async fn connect(&mut self, _account_type: crate::core::types::AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;

        tracing::info!("Yahoo WS: connecting to {}", WS_URL);

        // Connect to Yahoo Finance WebSocket
        let (ws_stream, response) = connect_async(WS_URL)
            .await
            .map_err(|e| WebSocketError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        tracing::info!(
            "Yahoo WS: connected (HTTP status: {})",
            response.status()
        );

        *self.status.lock().await = ConnectionStatus::Connected;

        // Create event broadcast channel
        let (tx, _rx) = broadcast::channel(1024);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Create command channel for outbound messages
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        *self.cmd_tx.lock().await = Some(cmd_tx);

        // Start message handler -- it takes ownership of ws_stream
        Self::start_message_handler(
            ws_stream,
            cmd_rx,
            self.event_tx.clone(),
            self.status.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        tracing::info!("Yahoo WS: disconnecting");

        // Drop the command sender -- this signals the handler to shut down
        *self.cmd_tx.lock().await = None;

        // Give the handler a moment to close gracefully
        sleep(Duration::from_millis(100)).await;

        *self.status.lock().await = ConnectionStatus::Disconnected;
        let _ = self.event_tx.lock().unwrap().take();
        self.subscriptions.lock().await.clear();
        self.yahoo_symbols.lock().await.clear();

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Yahoo Finance WebSocket only supports ticker data
        match &request.stream_type {
            StreamType::Ticker => {}
            other => {
                return Err(WebSocketError::UnsupportedOperation(format!(
                    "Yahoo Finance WebSocket only supports Ticker streams, got {:?}",
                    other,
                )));
            }
        }

        let yahoo_symbol = Self::symbol_to_yahoo(&request.symbol);

        // Build subscribe JSON
        let msg = json!({ "subscribe": [yahoo_symbol] });
        let msg_text = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        tracing::info!("Yahoo WS: subscribing to {}", yahoo_symbol);

        // Send via command channel (non-blocking, no lock contention)
        let cmd_guard = self.cmd_tx.lock().await;
        let cmd_tx = cmd_guard
            .as_ref()
            .ok_or(WebSocketError::NotConnected)?;

        cmd_tx
            .send(msg_text)
            .map_err(|e| WebSocketError::SendError(e.to_string()))?;

        drop(cmd_guard);

        // Track subscription
        self.subscriptions.lock().await.insert(request);
        self.yahoo_symbols.lock().await.insert(yahoo_symbol);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let yahoo_symbol = Self::symbol_to_yahoo(&request.symbol);

        // Build unsubscribe JSON
        let msg = json!({ "unsubscribe": [yahoo_symbol] });
        let msg_text = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        tracing::info!("Yahoo WS: unsubscribing from {}", yahoo_symbol);

        // Send via command channel
        let cmd_guard = self.cmd_tx.lock().await;
        let cmd_tx = cmd_guard
            .as_ref()
            .ok_or(WebSocketError::NotConnected)?;

        cmd_tx
            .send(msg_text)
            .map_err(|e| WebSocketError::SendError(e.to_string()))?;

        drop(cmd_guard);

        // Remove subscription
        self.subscriptions.lock().await.remove(&request);
        self.yahoo_symbols.lock().await.remove(&yahoo_symbol);

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
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_varint_single_byte() {
        let buf = [0x08]; // varint = 8
        let (val, consumed) = decode_varint(&buf).unwrap();
        assert_eq!(val, 8);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_decode_varint_multi_byte() {
        let buf = [0xAC, 0x02]; // varint = 300
        let (val, consumed) = decode_varint(&buf).unwrap();
        assert_eq!(val, 300);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_decode_sint64() {
        assert_eq!(decode_sint64(0), 0);
        assert_eq!(decode_sint64(1), -1);
        assert_eq!(decode_sint64(2), 1);
        assert_eq!(decode_sint64(3), -2);
    }

    #[test]
    fn test_decode_f32() {
        // IEEE 754: 1.0f32 = 0x3F800000
        let bytes = 1.0f32.to_le_bytes();
        let val = decode_f32(&bytes).unwrap();
        assert_eq!(val, 1.0);
    }

    #[test]
    fn test_parse_protobuf_simple() {
        // Build a minimal protobuf message:
        // field 1 (id), wire type 2 (length-delimited) => tag = (1 << 3) | 2 = 0x0A
        // length = 4, data = "AAPL"
        // field 2 (price), wire type 5 (fixed32) => tag = (2 << 3) | 5 = 0x15
        // data = 150.25f32 little-endian
        let mut buf = Vec::new();

        // Field 1: id = "AAPL"
        buf.push(0x0A); // tag
        buf.push(0x04); // length
        buf.extend_from_slice(b"AAPL");

        // Field 2: price = 150.25
        buf.push(0x15); // tag
        buf.extend_from_slice(&150.25f32.to_le_bytes());

        let result = parse_protobuf(&buf).unwrap();
        assert_eq!(result.id.as_deref(), Some("AAPL"));
        assert!((result.price.unwrap() - 150.25).abs() < 0.01);
    }

    #[test]
    fn test_parse_protobuf_with_varint_fields() {
        let mut buf = Vec::new();

        // Field 1: id = "BTC-USD"
        buf.push(0x0A); // tag = (1 << 3) | 2
        buf.push(0x07); // length = 7
        buf.extend_from_slice(b"BTC-USD");

        // Field 2: price = 45000.0
        buf.push(0x15); // tag = (2 << 3) | 5
        buf.extend_from_slice(&45000.0f32.to_le_bytes());

        // Field 17: volume = 1000 (varint)
        // tag = (17 << 3) | 0 = 0x88, 0x01
        buf.push(0x88);
        buf.push(0x01);
        // varint 1000 = 0xE8, 0x07
        buf.push(0xE8);
        buf.push(0x07);

        let result = parse_protobuf(&buf).unwrap();
        assert_eq!(result.id.as_deref(), Some("BTC-USD"));
        assert!((result.price.unwrap() - 45000.0).abs() < 1.0);
        assert_eq!(result.volume, Some(1000));
    }

    #[test]
    fn test_pricing_data_to_ticker() {
        let data = PricingData {
            id: Some("AAPL".to_string()),
            price: Some(150.25),
            time: Some(1640995200),
            bid: Some(150.20),
            ask: Some(150.30),
            day_high: Some(151.50),
            day_low: Some(149.00),
            volume: Some(25_000_000),
            change: Some(1.75),
            change_percent: Some(1.18),
            ..Default::default()
        };

        let event = pricing_data_to_ticker(&data);
        match event {
            StreamEvent::Ticker(ticker) => {
                assert_eq!(ticker.symbol, "AAPL");
                assert!((ticker.last_price - 150.25).abs() < 0.01);
                assert_eq!(ticker.bid_price, Some(150.2000045776367)); // f32 -> f64 precision
                assert_eq!(ticker.volume_24h, Some(25_000_000.0));
            }
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_symbol_to_yahoo() {
        // Stock
        let symbol = Symbol::new("AAPL", "USD");
        assert_eq!(YahooFinanceWebSocket::symbol_to_yahoo(&symbol), "AAPL");

        // Crypto
        let symbol = Symbol::new("BTC", "USD");
        assert_eq!(YahooFinanceWebSocket::symbol_to_yahoo(&symbol), "BTC-USD");
    }

    #[test]
    fn test_websocket_creation() {
        let ws = YahooFinanceWebSocket::new();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
        assert!(ws.active_subscriptions().is_empty());
    }
}
