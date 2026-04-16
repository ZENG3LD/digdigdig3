//! # dYdX v4 WebSocket Implementation
//!
//! WebSocket connector for dYdX v4 Indexer real-time data feeds.
//!
//! ## Features
//! - Public channels (orderbook, trades, markets, candles)
//! - Broadcast channel pattern for multiple consumers
//! - Automatic reconnection
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Available Channels
//! - `v4_orderbook` - Order book updates (batched L2 updates, requires market id)
//! - `v4_trades` - Trade executions (requires market id)
//! - `v4_markets` - Market info updates (global, no id)
//! - `v4_candles` - OHLC candle data (requires market id)
//! - `v4_subaccounts` - Private subaccount updates: orders, fills, positions (requires subaccount id)
//! - `v4_parent_subaccounts` - Parent subaccount updates (requires parent subaccount id)
//! - `v4_blockheight` - Chain block height updates (global, no id)

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    AccountType, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;

use super::endpoints::{DydxUrls, normalize_symbol};
use super::parser::DydxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscribe message for channels that require an id (orderbook, trades, candles)
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessageWithId {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    batched: Option<bool>,
}

/// Subscribe message for channels without id (v4_markets, v4_block_height)
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessageNoId {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
}

/// Unsubscribe message for channels with id
#[derive(Debug, Clone, Serialize)]
struct UnsubscribeMessageWithId {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
    id: String,
}

/// Unsubscribe message for channels without id
#[derive(Debug, Clone, Serialize)]
struct UnsubscribeMessageNoId {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
}

/// Incoming WebSocket message
#[derive(Debug, Clone, Deserialize, Serialize)]
struct IncomingMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(rename = "connection_id")]
    connection_id: Option<String>,
    channel: Option<String>,
    id: Option<String>,
    #[serde(rename = "message_id")]
    message_id: Option<u64>,
    contents: Option<Value>,
    version: Option<String>,
    message: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DYDX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;

/// dYdX v4 WebSocket connector
pub struct DydxWebSocket {
    /// WebSocket URL
    url: String,
    /// Current account type (dYdX only futures)
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender (for multiple consumers, dropped on disconnect)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Write half of WebSocket (for sending subscribe/unsubscribe messages)
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    /// Last message time
    last_message: Arc<Mutex<Instant>>,
    /// Timestamp of the most recently sent WS-frame ping.
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl DydxWebSocket {
    /// Create new dYdX WebSocket connector
    pub async fn new(
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            DydxUrls::TESTNET
        } else {
            DydxUrls::MAINNET
        };

        let url = urls.indexer_ws.to_string();

        Ok(Self {
            url,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_sink: Arc::new(Mutex::new(None)),
            last_message: Arc::new(Mutex::new(Instant::now())),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Subscribe to a channel with an id (orderbook, trades, candles)
    async fn send_subscribe_with_id(&self, channel: &str, id: &str) -> WebSocketResult<()> {
        let message = SubscribeMessageWithId {
            msg_type: "subscribe".to_string(),
            channel: channel.to_string(),
            id: id.to_string(),
            batched: Some(false),
        };

        let json = serde_json::to_string(&message)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(json)).await
                .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;
        }

        Ok(())
    }

    /// Subscribe to a channel without id (v4_markets)
    async fn send_subscribe_no_id(&self, channel: &str) -> WebSocketResult<()> {
        let message = SubscribeMessageNoId {
            msg_type: "subscribe".to_string(),
            channel: channel.to_string(),
        };

        let json = serde_json::to_string(&message)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(json)).await
                .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;
        }

        Ok(())
    }

    /// Unsubscribe from a channel with id
    async fn send_unsubscribe_with_id(&self, channel: &str, id: &str) -> WebSocketResult<()> {
        let message = UnsubscribeMessageWithId {
            msg_type: "unsubscribe".to_string(),
            channel: channel.to_string(),
            id: id.to_string(),
        };

        let json = serde_json::to_string(&message)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(json)).await
                .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;
        }

        Ok(())
    }

    /// Unsubscribe from a channel without id (v4_markets)
    async fn send_unsubscribe_no_id(&self, channel: &str) -> WebSocketResult<()> {
        let message = UnsubscribeMessageNoId {
            msg_type: "unsubscribe".to_string(),
            channel: channel.to_string(),
        };

        let json = serde_json::to_string(&message)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(json)).await
                .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;
        }

        Ok(())
    }

    /// Handle incoming WebSocket message.
    ///
    /// `target_ticker_symbol` — the dYdX market identifier for the active ticker
    /// subscription (e.g. `"BTC-USD"`).  Required so that the global `v4_markets`
    /// snapshot/update is filtered to only the subscribed market.
    fn handle_message(text: &str, target_ticker_symbol: &str) -> Option<WebSocketResult<StreamEvent>> {
        let msg: IncomingMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[dYdX WS] Parse error: {}", e);
                return Some(Err(WebSocketError::Parse(format!("Failed to parse message: {}", e))));
            }
        };

        match msg.msg_type.as_str() {
            "connected" => {
                // Initial connection message
                None
            }
            "subscribed" => {
                // Subscription confirmed - may include initial snapshot data
                // Check if there's initial data in contents
                if msg.contents.is_some() {
                    if let Some(channel) = &msg.channel {
                        return Self::parse_channel_data(channel, &msg, target_ticker_symbol);
                    }
                }
                None
            }
            "unsubscribed" => {
                // Unsubscription confirmed
                None
            }
            "channel_data" => {
                // Parse channel data based on channel type
                if let Some(channel) = &msg.channel {
                    Self::parse_channel_data(channel, &msg, target_ticker_symbol)
                } else {
                    None
                }
            }
            "error" => {
                let error_msg = msg.message.unwrap_or_else(|| "Unknown error".to_string());
                Some(Err(WebSocketError::ProtocolError(error_msg)))
            }
            _ => None,
        }
    }

    /// Parse channel-specific data.
    ///
    /// `target_ticker_symbol` is forwarded to the `v4_markets` parser so it can extract
    /// only the subscribed market from the global snapshot/update map.
    fn parse_channel_data(channel: &str, msg: &IncomingMessage, target_ticker_symbol: &str) -> Option<WebSocketResult<StreamEvent>> {
        let data = serde_json::to_value(msg).ok()?;

        match channel {
            "v4_orderbook" => {
                match DydxParser::parse_ws_orderbook_delta(&data) {
                    Ok(event) => Some(Ok(event)),
                    Err(e) => Some(Err(WebSocketError::Parse(e.to_string()))),
                }
            }
            "v4_trades" => {
                match DydxParser::parse_ws_trade(&data) {
                    Ok(trade) => Some(Ok(StreamEvent::Trade(trade))),
                    Err(e) => Some(Err(WebSocketError::Parse(e.to_string()))),
                }
            }
            "v4_markets" => {
                // The v4_markets channel is global and contains ALL markets.
                // Pass the target symbol so the parser extracts only the subscribed market.
                match DydxParser::parse_ws_ticker(&data, target_ticker_symbol) {
                    Ok(ticker) => Some(Ok(StreamEvent::Ticker(ticker))),
                    Err(_) => None, // Symbol not present in this update — skip silently.
                }
            }
            "v4_candles" => {
                // Candles are per-market/resolution. Parse as Kline and emit to subscribers.
                match DydxParser::parse_ws_candle(&data) {
                    Ok(event) => Some(Ok(event)),
                    Err(e) => Some(Err(WebSocketError::Parse(e.to_string()))),
                }
            }
            "v4_subaccounts" | "v4_parent_subaccounts" => {
                // Private channels: orders, fills, positions for a subaccount.
                // These carry complex nested objects (fills, perpetualPositions, orders).
                // Parse as OrderUpdate / PositionUpdate once private-stream types are
                // added to DydxParser.  Until then acknowledge without emitting.
                let _ = &data;
                None
            }
            "v4_blockheight" => {
                // Block height updates: {"height":"12345678","time":"..."}
                // No matching StreamEvent variant yet — acknowledged silently.
                let _ = &data;
                None
            }
            _ => None,
        }
    }

    /// Start message receiving loop using the read half of the WebSocket.
    /// The read half is owned by this task, so no mutex contention with the
    /// write half used for subscribe/unsubscribe.
    ///
    /// `subscriptions` is used to resolve the active ticker symbol for the global
    /// `v4_markets` channel so that only data for the subscribed market is forwarded.
    /// Start periodic WS-frame ping task (every 5 seconds) for RTT measurement.
    fn start_ping_task(
        ws_sink: Arc<Mutex<Option<WsSink>>>,
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

                let mut sink_guard = ws_sink.lock().await;
                if let Some(sink) = sink_guard.as_mut() {
                    *last_ping.lock().await = Instant::now();
                    if sink.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    fn start_message_loop(
        mut ws_read: futures_util::stream::SplitStream<WsStream>,
        broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_message: Arc<Mutex<Instant>>,
        subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            loop {
                // Check if still connected
                {
                    let s = status.lock().await;
                    if *s == ConnectionStatus::Disconnected {
                        break;
                    }
                }

                match ws_read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        *last_message.lock().await = Instant::now();

                        // Resolve the active ticker symbol for v4_markets filtering.
                        // Most messages are not v4_markets, so this is a fast read.
                        let ticker_sym: String = {
                            let subs = subscriptions.lock().await;
                            subs.iter()
                                .find(|req| req.stream_type == StreamType::Ticker)
                                .map(|req| super::endpoints::normalize_symbol(&req.symbol.to_string()))
                                .unwrap_or_default()
                        };

                        if let Some(event) = Self::handle_message(&text, &ticker_sym) {
                            if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                let _ = tx.send(event);
                            }
                        }
                    }
                    Some(Ok(Message::Ping(_))) => {
                        // Pong is handled automatically by tungstenite
                        *last_message.lock().await = Instant::now();
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Measure RTT from our last client-initiated ping frame.
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Some(Ok(Message::Close(_))) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                            let _ = tx.send(Err(WebSocketError::NotConnected));
                        }
                        break;
                    }
                    Some(Err(e)) => {
                        if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                            let _ = tx.send(Err(WebSocketError::ProtocolError(e.to_string())));
                        }
                    }
                    None => {
                        // Stream ended
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {}
                }
            }
            // Stream ended — drop sender so receivers know the stream is done
            let _ = broadcast_tx.lock().unwrap().take();
        });
    }

    /// Check if a channel requires an id parameter
    fn channel_requires_id(channel: &str) -> bool {
        // v4_markets and v4_blockheight do not require an id
        !matches!(channel, "v4_markets" | "v4_blockheight")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (dYdX-specific channel subscriptions)
// ═══════════════════════════════════════════════════════════════════════════════

impl DydxWebSocket {
    /// Subscribe to the global `v4_markets` channel (market info updates, no id required).
    pub async fn subscribe_markets(&self) -> WebSocketResult<()> {
        self.send_subscribe_no_id("v4_markets").await
    }

    /// Unsubscribe from the `v4_markets` channel.
    pub async fn unsubscribe_markets(&self) -> WebSocketResult<()> {
        self.send_unsubscribe_no_id("v4_markets").await
    }

    /// Subscribe to the `v4_orderbook` channel for the given market (e.g. `"BTC-USD"`).
    pub async fn subscribe_orderbook(&self, market: &str) -> WebSocketResult<()> {
        self.send_subscribe_with_id("v4_orderbook", &normalize_symbol(market)).await
    }

    /// Unsubscribe from the `v4_orderbook` channel for the given market.
    pub async fn unsubscribe_orderbook(&self, market: &str) -> WebSocketResult<()> {
        self.send_unsubscribe_with_id("v4_orderbook", &normalize_symbol(market)).await
    }

    /// Subscribe to the `v4_trades` channel for the given market (e.g. `"ETH-USD"`).
    pub async fn subscribe_trades(&self, market: &str) -> WebSocketResult<()> {
        self.send_subscribe_with_id("v4_trades", &normalize_symbol(market)).await
    }

    /// Unsubscribe from the `v4_trades` channel for the given market.
    pub async fn unsubscribe_trades(&self, market: &str) -> WebSocketResult<()> {
        self.send_unsubscribe_with_id("v4_trades", &normalize_symbol(market)).await
    }

    /// Subscribe to the `v4_candles` channel for the given market (e.g. `"BTC-USD"`).
    pub async fn subscribe_candles(&self, market: &str) -> WebSocketResult<()> {
        self.send_subscribe_with_id("v4_candles", &normalize_symbol(market)).await
    }

    /// Unsubscribe from the `v4_candles` channel for the given market.
    pub async fn unsubscribe_candles(&self, market: &str) -> WebSocketResult<()> {
        self.send_unsubscribe_with_id("v4_candles", &normalize_symbol(market)).await
    }

    /// Subscribe to `v4_subaccounts` — private stream for orders, fills, and positions
    /// belonging to a subaccount.
    ///
    /// The `id` is the subaccount identifier string returned by the dYdX Indexer API,
    /// e.g. `"dydx1abc...xyz/0"` (address + subaccount number separated by `/`).
    pub async fn subscribe_subaccount(&self, subaccount_id: &str) -> WebSocketResult<()> {
        self.send_subscribe_with_id("v4_subaccounts", subaccount_id).await
    }

    /// Unsubscribe from `v4_subaccounts` for the given subaccount id.
    pub async fn unsubscribe_subaccount(&self, subaccount_id: &str) -> WebSocketResult<()> {
        self.send_unsubscribe_with_id("v4_subaccounts", subaccount_id).await
    }

    /// Subscribe to `v4_parent_subaccounts` — parent subaccount updates (aggregates
    /// child subaccount positions and orders).
    ///
    /// The `id` format matches the Indexer convention: `"<address>/<parent_subaccount_number>"`.
    pub async fn subscribe_parent_subaccount(&self, parent_subaccount_id: &str) -> WebSocketResult<()> {
        self.send_subscribe_with_id("v4_parent_subaccounts", parent_subaccount_id).await
    }

    /// Unsubscribe from `v4_parent_subaccounts` for the given parent subaccount id.
    pub async fn unsubscribe_parent_subaccount(&self, parent_subaccount_id: &str) -> WebSocketResult<()> {
        self.send_unsubscribe_with_id("v4_parent_subaccounts", parent_subaccount_id).await
    }

    /// Subscribe to `v4_blockheight` — chain block height updates (global, no id required).
    pub async fn subscribe_blockheight(&self) -> WebSocketResult<()> {
        self.send_subscribe_no_id("v4_blockheight").await
    }

    /// Unsubscribe from `v4_blockheight`.
    pub async fn unsubscribe_blockheight(&self) -> WebSocketResult<()> {
        self.send_unsubscribe_no_id("v4_blockheight").await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for DydxWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.url).await
            .map_err(|e| WebSocketError::ConnectionError(format!("Connection failed: {}", e)))?;

        // Split into read and write halves to avoid mutex contention.
        // The read half is owned by the message loop task.
        // The write half is stored for sending subscribe/unsubscribe messages.
        let (ws_sink, ws_read) = ws_stream.split();

        *self.ws_sink.lock().await = Some(ws_sink);
        *self.status.lock().await = ConnectionStatus::Connected;
        *self.last_message.lock().await = Instant::now();

        // Create broadcast channel and store
        let (broadcast_sender, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(broadcast_sender);

        // Start message receiving loop with the read half
        Self::start_message_loop(
            ws_read,
            Arc::clone(&self.broadcast_tx),
            Arc::clone(&self.status),
            Arc::clone(&self.last_message),
            Arc::clone(&self.subscriptions),
            Arc::clone(&self.last_ping),
            Arc::clone(&self.ws_ping_rtt_ms),
        );

        // Start periodic ping task for RTT measurement.
        Self::start_ping_task(
            Arc::clone(&self.ws_sink),
            Arc::clone(&self.status),
            Arc::clone(&self.last_ping),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            let _ = sink.close().await;
        }
        *sink_guard = None;
        drop(sink_guard);

        let _ = self.broadcast_tx.lock().unwrap().take();

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // This is synchronous, so we need to use try_lock or block
        match self.status.try_lock() {
            Ok(guard) => *guard,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channel = match &request.stream_type {
            StreamType::Ticker => "v4_markets",
            StreamType::Orderbook => "v4_orderbook",
            StreamType::Trade => "v4_trades",
            StreamType::Kline { .. } => "v4_candles",
            _ => {
                return Err(WebSocketError::ProtocolError(
                    format!("Stream type {:?} not supported", request.stream_type)
                ));
            }
        };

        if Self::channel_requires_id(channel) {
            let symbol_str = request.symbol.to_string();
            self.send_subscribe_with_id(channel, &normalize_symbol(&symbol_str)).await?;
        } else {
            // v4_markets does not take an id parameter
            self.send_subscribe_no_id(channel).await?;
        }

        self.subscriptions.lock().await.insert(request);
        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channel = match &request.stream_type {
            StreamType::Ticker => "v4_markets",
            StreamType::Orderbook => "v4_orderbook",
            StreamType::Trade => "v4_trades",
            StreamType::Kline { .. } => "v4_candles",
            _ => return Ok(()),
        };

        if Self::channel_requires_id(channel) {
            let symbol_str = request.symbol.to_string();
            self.send_unsubscribe_with_id(channel, &normalize_symbol(&symbol_str)).await?;
        } else {
            self.send_unsubscribe_no_id(channel).await?;
        }

        self.subscriptions.lock().await.remove(&request);
        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1);
        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
            result.ok()
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_lock() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => vec![],
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(Arc::clone(&self.ws_ping_rtt_ms))
    }
}

// Clone implementation for Arc wrapping
impl Clone for DydxWebSocket {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            account_type: self.account_type,
            status: Arc::clone(&self.status),
            subscriptions: Arc::clone(&self.subscriptions),
            broadcast_tx: Arc::clone(&self.broadcast_tx),
            ws_sink: Arc::clone(&self.ws_sink),
            last_message: Arc::clone(&self.last_message),
            last_ping: Arc::clone(&self.last_ping),
            ws_ping_rtt_ms: Arc::clone(&self.ws_ping_rtt_ms),
        }
    }
}
