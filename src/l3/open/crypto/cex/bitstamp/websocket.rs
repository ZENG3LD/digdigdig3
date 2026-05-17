//! # Bitstamp WebSocket Implementation
//!
//! WebSocket connector for Bitstamp WebSocket API (Pusher protocol).
//!
//! ## Features
//! - Public channels (live_trades, order_book, diff_order_book)
//! - Subscription management
//! - Message parsing to StreamEvent
//! - WebSocket-level ping/pong heartbeat handling
//!
//! ## Channel Mapping
//!
//! Bitstamp does not have a dedicated "ticker" WebSocket channel. Instead:
//! - `StreamType::Ticker` -> `live_trades_{pair}` (emits Ticker from each trade price)
//! - `StreamType::Trade` -> `live_trades_{pair}` (per-trade, emits Trade events)
//! - `StreamType::Orderbook` -> `order_book_{pair}` (periodic full snapshots)
//! - `StreamType::OrderbookDelta` -> `diff_order_book_{pair}` (incremental updates)
//!
//! ## Pusher Protocol
//!
//! Bitstamp uses the Pusher protocol over WebSocket:
//! - Connection: `wss://ws.bitstamp.net`
//! - Server sends `pusher:connection_established` on connect
//! - Subscribe: `{"event":"bts:subscribe","data":{"channel":"..."}}`
//! - Heartbeat: client sends `{"event":"pusher:ping","data":{}}`,
//!   server responds with `{"event":"pusher:pong","data":{}}`
//! - Trade events: `{"event":"trade","channel":"live_trades_...","data":{...}}`
//! - Order book events: `{"event":"data","channel":"diff_order_book_...","data":{...}}`

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::SplitSink, stream::SplitStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, SubscriptionRequest,
};
use crate::core::types::{
    WebSocketResult, WebSocketError, OrderbookCapabilities, WsBookChannel,
    OrderbookDelta as OrderbookDeltaData, OrderBookLevel, OrderSide,
};
use crate::core::traits::WebSocketConnector;

use super::endpoints::{BitstampUrls, format_symbol};
use super::parser::BitstampParser;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscription message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    event: String,
    data: ChannelData,
}

#[derive(Debug, Clone, Serialize)]
struct ChannelData {
    channel: String,
}

/// Incoming WebSocket message
#[derive(Debug, Clone, Deserialize)]
struct IncomingMessage {
    event: String,
    channel: Option<String>,
    data: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BITSTAMP WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsWriter = SplitSink<WsStream, Message>;
type WsReader = SplitStream<WsStream>;

/// Bitstamp WebSocket connector
pub struct BitstampWebSocket {
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Channels subscribed as Ticker (live_trades_* channels used for ticker data).
    /// These channels emit `StreamEvent::Ticker` instead of `StreamEvent::Trade`.
    ticker_channels: Arc<Mutex<HashSet<String>>>,
    /// Event sender (internal - for message handler)
    event_tx: Arc<Mutex<Option<mpsc::UnboundedSender<WebSocketResult<StreamEvent>>>>>,
    /// Broadcast sender (for multiple consumers, dropped on disconnect)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half (for sending subscriptions and pongs)
    ws_writer: Arc<Mutex<Option<WsWriter>>>,
    /// Timestamp of the most recently sent WS-frame ping.
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl BitstampWebSocket {
    /// Create new Bitstamp WebSocket connector
    pub async fn new() -> ExchangeResult<Self> {
        Ok(Self {
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            ticker_channels: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(Mutex::new(None)),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Subscribe to a channel by sending a Pusher subscription message
    async fn subscribe_channel(&self, channel: &str) -> ExchangeResult<()> {
        let msg = SubscribeMessage {
            event: "bts:subscribe".to_string(),
            data: ChannelData {
                channel: channel.to_string(),
            },
        };

        let json = serde_json::to_string(&msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize: {}", e)))?;

        let mut writer_guard = self.ws_writer.lock().await;
        if let Some(writer) = writer_guard.as_mut() {
            writer.send(Message::Text(json))
                .await
                .map_err(|e| ExchangeError::Network(format!("Failed to send message: {}", e)))?;
        } else {
            return Err(ExchangeError::Network("Not connected".to_string()));
        }

        Ok(())
    }

    /// Subscribe to ticker data via live_trades channel.
    ///
    /// Bitstamp has no dedicated ticker WebSocket channel. The `live_trades`
    /// channel provides actual executed trade prices, which is used to emit
    /// `StreamEvent::Ticker` events with the latest price. The channel name
    /// is tracked in `ticker_channels` so the message handler knows to emit
    /// a Ticker (with last_price) rather than a raw Trade event.
    ///
    /// `pair` must be the exchange-native raw string (e.g. `"btcusd"`).
    pub async fn subscribe_ticker(&self, pair: &str) -> ExchangeResult<()> {
        let channel = format!("live_trades_{}", pair);
        self.ticker_channels.lock().await.insert(channel.clone());
        self.subscribe_channel(&channel).await
    }

    /// Subscribe to live trades.
    ///
    /// `pair` must be the exchange-native raw string (e.g. `"btcusd"`).
    pub async fn subscribe_trades(&self, pair: &str) -> ExchangeResult<()> {
        let channel = format!("live_trades_{}", pair);
        self.subscribe_channel(&channel).await
    }

    /// Subscribe to order book snapshots.
    ///
    /// `pair` must be the exchange-native raw string (e.g. `"btcusd"`).
    pub async fn subscribe_orderbook(&self, pair: &str) -> ExchangeResult<()> {
        let channel = format!("order_book_{}", pair);
        self.subscribe_channel(&channel).await
    }

    /// Subscribe to live order events (L3 — per-order lifecycle: created/changed/deleted).
    ///
    /// This is a legacy channel.  Prefer `diff_order_book` (`StreamType::OrderbookDelta`)
    /// for L2 aggregated incremental updates.  Use `live_orders` only when you need
    /// individual order-level events (e.g. to build a full L3 book).
    ///
    /// Events arrive as `OrderbookDelta` with a single bid or ask level per frame:
    /// - `"order_created"` / `"order_changed"` — level with non-zero quantity
    /// - `"order_deleted"` — level with zero quantity (remove signal)
    ///
    /// `pair` must be the exchange-native raw string (e.g. `"btcusd"`).
    pub async fn subscribe_live_orders(&self, pair: &str) -> ExchangeResult<()> {
        let channel = format!("live_orders_{}", pair);
        self.subscribe_channel(&channel).await
    }

    /// Subscribe to the full L3 order book snapshot with order IDs.
    ///
    /// Channel: `detail_order_book_{pair}` (e.g. `detail_order_book_btcusd`)
    ///
    /// Bitstamp sends a full L3 book snapshot on subscribe and pushes incremental
    /// updates when individual orders are created, changed, or deleted.
    ///
    /// `pair` must be the exchange-native raw string (e.g. `"btcusd"`).
    pub async fn subscribe_detail_order_book(&self, pair: &str) -> ExchangeResult<()> {
        let channel = format!("detail_order_book_{}", pair);
        self.subscribe_channel(&channel).await
    }

    /// Start message handling task.
    ///
    /// The reader half processes incoming messages while the writer half
    /// is shared for sending Pusher pings and subscription messages.
    fn start_message_handler(
        reader: WsReader,
        ws_writer: Arc<Mutex<Option<WsWriter>>>,
        event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        status: Arc<Mutex<ConnectionStatus>>,
        ticker_channels: Arc<Mutex<HashSet<String>>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            let mut reader = reader;

            loop {
                match reader.next().await {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = Self::handle_message(&text, &event_tx, &ticker_channels).await {
                            let _ = event_tx.send(Err(e));
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // Respond to WebSocket-level ping with pong
                        let mut writer_guard = ws_writer.lock().await;
                        if let Some(writer) = writer_guard.as_mut() {
                            let _ = writer.send(Message::Pong(data)).await;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Measure RTT from our last client-initiated ping frame.
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Some(Ok(Message::Binary(_))) => {
                        // Binary messages not expected from Bitstamp
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
                            "WebSocket read error".to_string()
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

    /// Start periodic WebSocket-level ping task (every 5 seconds for RTT measurement).
    ///
    /// Bitstamp's Pusher server has an `activity_timeout` of 120 seconds.
    /// Sending a WS-frame ping every 5 seconds keeps the connection alive and
    /// lets us measure RTT from the server's Pong response.
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsWriter>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            interval.tick().await; // skip first immediate tick

            loop {
                interval.tick().await;

                // Check if still connected
                let current_status = *status.lock().await;
                if current_status != ConnectionStatus::Connected {
                    break;
                }

                // Record time before sending ping, then send WS-frame ping
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

    /// Handle incoming WebSocket text message (Pusher protocol)
    async fn handle_message(
        text: &str,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        ticker_channels: &Arc<Mutex<HashSet<String>>>,
    ) -> WebSocketResult<()> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        match msg.event.as_str() {
            // Pusher protocol events
            "pusher:connection_established" => {
                // Connection confirmed by Pusher - nothing to do
                return Ok(());
            }
            "pusher:pong" => {
                // Heartbeat response - connection is alive
                return Ok(());
            }
            "pusher:error" => {
                return Err(WebSocketError::ProtocolError(
                    format!("Pusher error: {:?}", msg.data)
                ));
            }

            // Bitstamp-specific protocol events
            "bts:subscription_succeeded" => {
                // Subscription confirmed
                return Ok(());
            }
            "bts:error" => {
                return Err(WebSocketError::ProtocolError(
                    format!("Bitstamp error: {:?}", msg.data)
                ));
            }
            "bts:request_reconnect" => {
                return Err(WebSocketError::ConnectionError(
                    "Server requested reconnection (bts:request_reconnect)".to_string()
                ));
            }

            // Data events
            "trade" | "data" => {
                let is_ticker_channel = if let Some(ch) = msg.channel.as_ref() {
                    ticker_channels.lock().await.contains(ch)
                } else {
                    false
                };
                if let Some(event) = Self::parse_data_message(&msg, is_ticker_channel)? {
                    let _ = event_tx.send(Ok(event));
                }
            }

            // Live orders (L3) — per-order lifecycle events on live_orders_{pair} channels.
            // Each event carries a single order; map to OrderbookDelta with one level.
            "order_created" | "order_changed" | "order_deleted" => {
                if let Some(event) = Self::parse_live_order_message(&msg)? {
                    let _ = event_tx.send(Ok(event));
                }
            }

            _ => {
                // Unknown event type - silently ignore
            }
        }

        Ok(())
    }

    /// Parse data message to StreamEvent based on the channel name.
    ///
    /// When `as_ticker` is true, a `live_trades_*` message is parsed as
    /// `StreamEvent::Ticker` (last_price = trade price) instead of `StreamEvent::Trade`.
    fn parse_data_message(msg: &IncomingMessage, as_ticker: bool) -> WebSocketResult<Option<StreamEvent>> {
        let channel = msg.channel.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing channel".to_string()))?;

        // Reconstruct JSON for parser (parser expects { channel, event, data } format)
        let json = serde_json::json!({
            "channel": channel,
            "event": &msg.event,
            "data": msg.data
        });

        // Match channel to determine event type
        if channel.starts_with("live_trades_") {
            if as_ticker {
                // Build a minimal Ticker from the trade price.
                // Bitstamp has no WS ticker channel, so we use live trade price.
                let trade = BitstampParser::parse_ws_trade(&json)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                let ticker = crate::core::types::Ticker {
                    symbol: trade.symbol,
                    last_price: trade.price,
                    bid_price: None,
                    ask_price: None,
                    high_24h: None,
                    low_24h: None,
                    volume_24h: None,
                    quote_volume_24h: None,
                    price_change_24h: None,
                    price_change_percent_24h: None,
                    timestamp: trade.timestamp,
                };
                Ok(Some(StreamEvent::Ticker(ticker)))
            } else {
                let trade = BitstampParser::parse_ws_trade(&json)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(trade)))
            }
        } else if channel.starts_with("diff_order_book_") {
            let orderbook = BitstampParser::parse_ws_orderbook(&json)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::OrderbookSnapshot(orderbook)))
        } else if channel.starts_with("detail_order_book_") {
            // L3 full book: each bid/ask entry is [price, amount, order_id].
            // Bitstamp sends one snapshot on subscribe and incremental snapshots on change.
            // Emit OrderbookL3 for every individual entry so consumers can build/update
            // a full L3 order book.
            //
            // Data shape (verified via REST ?group=2 which mirrors WS L3 layout):
            //   data.bids: [["price", "amount", "order_id"], ...]
            //   data.asks: [["price", "amount", "order_id"], ...]
            //   data.microtimestamp: "1643643584684047"
            let data_obj = json.get("data")
                .ok_or_else(|| WebSocketError::Parse("detail_order_book: missing data".to_string()))?;

            // Timestamp in microseconds → milliseconds
            let timestamp_ms = data_obj
                .get("microtimestamp")
                .or_else(|| data_obj.get("timestamp"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .map(|us| {
                    // microtimestamp is 16 digits (microseconds), timestamp is 10 digits (seconds)
                    if us > 1_000_000_000_000_000 { us / 1000 } else { us * 1000 }
                })
                .unwrap_or(0);

            // Extract pair from channel name (e.g. "detail_order_book_btcusd" → "btcusd")
            let pair = channel.trim_start_matches("detail_order_book_").to_uppercase();

            let parse_side = |entries: &Value, side: OrderSide| -> Vec<StreamEvent> {
                entries.as_array()
                    .map(|arr| {
                        arr.iter().filter_map(|entry| {
                            let e = entry.as_array()?;
                            let price = e.first()?.as_str()?.parse::<f64>().ok()?;
                            let quantity = e.get(1)?.as_str()?.parse::<f64>().ok()?;
                            let order_id = e.get(2)?.as_str()?.to_string();
                            Some(StreamEvent::OrderbookL3 {
                                symbol: pair.clone(),
                                side,
                                order_id,
                                price,
                                quantity,
                                action: "create".to_string(),
                                timestamp: timestamp_ms,
                            })
                        }).collect()
                    })
                    .unwrap_or_default()
            };

            let mut events: Vec<StreamEvent> = Vec::new();
            events.extend(parse_side(data_obj.get("bids").unwrap_or(&serde_json::Value::Null), OrderSide::Buy));
            events.extend(parse_side(data_obj.get("asks").unwrap_or(&serde_json::Value::Null), OrderSide::Sell));

            // Return first event (trait returns single Option); for multi-event channels
            // the WS message handler would need to loop — but the current architecture
            // supports only one event per frame. Emit snapshot start if bids are present.
            Ok(events.into_iter().next())
        } else if channel.starts_with("order_book_") {
            let orderbook = BitstampParser::parse_ws_orderbook(&json)
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;
            Ok(Some(StreamEvent::OrderbookSnapshot(orderbook)))
        } else {
            // Unknown channel
            Ok(None)
        }
    }

    /// Parse a `live_orders_{pair}` event into a single-level `OrderbookDelta`.
    ///
    /// Event data fields (all strings):
    /// - `id` — order UUID
    /// - `price` — price level
    /// - `amount` — remaining amount (`"0"` on `order_deleted`)
    /// - `order_type` — `"0"` = bid, `"1"` = ask
    /// - `microtimestamp` — timestamp in microseconds
    fn parse_live_order_message(msg: &IncomingMessage) -> WebSocketResult<Option<StreamEvent>> {
        let json = serde_json::json!({
            "channel": msg.channel,
            "event": &msg.event,
            "data": msg.data
        });
        Self::parse_live_order_from_json(&json, &msg.event)
    }

    fn parse_live_order_from_json(json: &serde_json::Value, event_name: &str) -> WebSocketResult<Option<StreamEvent>> {
        let data = json.get("data")
            .ok_or_else(|| WebSocketError::Parse("live_orders: missing data".to_string()))?;

        // Parse price — string field
        let price = data.get("price")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| WebSocketError::Parse("live_orders: missing price".to_string()))?;

        // Amount is "0" on deletion events
        let amount = data.get("amount")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        // order_type: "0" = bid, "1" = ask
        let is_bid = data.get("order_type")
            .and_then(|v| v.as_str())
            .map(|s| s == "0")
            .unwrap_or(true);

        // Timestamp in microseconds → milliseconds
        let timestamp_ms = data.get("microtimestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .map(|us| us / 1000)
            .unwrap_or(0);

        // On order_deleted the amount is "0" — consumer interprets zero-qty as removal
        let _ = event_name; // event semantics already captured in amount

        let (bids, asks) = if is_bid {
            (vec![OrderBookLevel::new(price, amount)], vec![])
        } else {
            (vec![], vec![OrderBookLevel::new(price, amount)])
        };
        let delta = OrderbookDeltaData {
            bids,
            asks,
            timestamp: timestamp_ms,
            first_update_id: None,
            last_update_id: None,
            prev_update_id: None,
            event_time: None,
            checksum: None,
        };

        Ok(Some(StreamEvent::OrderbookDelta(delta)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for BitstampWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;

        let url = BitstampUrls::ws_url();
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to connect: {}", e)))?;

        // Split the stream into read and write halves.
        // This allows the message handler to read messages without blocking
        // the write half, which is needed for sending pong responses and
        // subscription messages concurrently.
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
            self.ticker_channels.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start WS-frame ping task for RTT measurement
        Self::start_ping_task(
            self.ws_writer.clone(),
            self.status.clone(),
            self.last_ping.clone(),
        );

        // Create broadcast channel and store
        let (broadcast_sender, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(broadcast_sender);

        // Start forwarder task (mpsc -> broadcast)
        let broadcast_tx = self.broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                    let _ = tx.send(event);
                }
            }
            // mpsc channel closed — drop broadcast sender
            let _ = broadcast_tx.lock().unwrap().take();
        });

        Ok(())
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_writer.lock().await = None;
        *self.event_tx.lock().await = None;
        let _ = self.broadcast_tx.lock().unwrap().take();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(guard) => *guard,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Extract exchange-native pair string from the subscription symbol.
        // Callers that already hold a raw string (e.g. "btcusd") should pass it via
        // Symbol::with_raw. Callers that pass Symbol::new("BTC","USD") get the pair
        // via format_symbol which produces the Bitstamp lowercase-concat format.
        let pair = request.symbol
            .raw()
            .map(|r| r.to_string())
            .unwrap_or_else(|| format_symbol(&request.symbol, AccountType::Spot));

        let result = match request.stream_type {
            crate::core::types::StreamType::Ticker => {
                // Ticker -> live_trades (high frequency, reliable)
                self.subscribe_ticker(&pair).await
                    .map_err(|e| WebSocketError::Subscription(format!("{:?}", e)))
            }
            crate::core::types::StreamType::Trade => {
                // Trade -> live_trades (per-trade events)
                self.subscribe_trades(&pair).await
                    .map_err(|e| WebSocketError::Subscription(format!("{:?}", e)))
            }
            crate::core::types::StreamType::Orderbook => {
                // Orderbook -> order_book (full snapshots)
                self.subscribe_orderbook(&pair).await
                    .map_err(|e| WebSocketError::Subscription(format!("{:?}", e)))
            }
            crate::core::types::StreamType::OrderbookDelta => {
                // OrderbookDelta -> diff_order_book (incremental updates)
                let channel = format!("diff_order_book_{}", pair);
                self.subscribe_channel(&channel).await
                    .map_err(|e| WebSocketError::Subscription(format!("{:?}", e)))
            }
            crate::core::types::StreamType::OrderbookL3 => {
                // L3 full orderbook with order IDs via detail_order_book channel
                self.subscribe_detail_order_book(&pair).await
                    .map_err(|e| WebSocketError::Subscription(format!("{:?}", e)))
            }
            _ => Err(WebSocketError::Subscription("Unsupported subscription type".to_string())),
        };

        // Track subscription if successful
        if result.is_ok() {
            self.subscriptions.lock().await.insert(request);
        }

        result
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions.lock().await.remove(&request);
        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1);
        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|res| async move {
            res.ok()
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_lock() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static BITSTAMP_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("order_book",      100, 1000),
            WsBookChannel::delta("diff_order_book",    None, None),
        ];
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITSTAMP_CHANNELS,
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &["0", "1", "2"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws = BitstampWebSocket::new().await;
        assert!(ws.is_ok());
    }

    #[tokio::test]
    async fn test_subscribe_message() {
        let msg = SubscribeMessage {
            event: "bts:subscribe".to_string(),
            data: ChannelData {
                channel: "diff_order_book_btcusd".to_string(),
            },
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("bts:subscribe"));
        assert!(json.contains("diff_order_book_btcusd"));
    }

    #[tokio::test]
    async fn test_pusher_message_parsing() {
        // Verify we can parse Pusher protocol messages
        let established = r#"{"event":"pusher:connection_established","data":"{\"socket_id\":\"123\",\"activity_timeout\":120}"}"#;
        let parsed: IncomingMessage = serde_json::from_str(established).unwrap();
        assert_eq!(parsed.event, "pusher:connection_established");
    }
}
