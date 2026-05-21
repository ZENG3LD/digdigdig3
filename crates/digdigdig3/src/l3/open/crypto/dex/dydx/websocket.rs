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
use crate::core::types::{
    WebSocketResult, WebSocketError, OrderbookCapabilities,
    OrderUpdateEvent, PositionUpdateEvent,
    OrderSide, OrderType, OrderStatus, PositionSide,
    TradeSide,
};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::timestamp_millis;

use super::endpoints::{DydxUrls, normalize_symbol};
use super::parser::DydxParser;

// ─────────────────────────────────────────────────────────────────────────────
// Kline interval mapping
// ─────────────────────────────────────────────────────────────────────────────

/// Map a common interval string to dYdX v4 candle resolution format.
///
/// dYdX v4 `v4_candles` uses `{SYMBOL}/{RESOLUTION}` as the subscription id.
/// Resolutions: `1MIN`, `5MINS`, `15MINS`, `30MINS`, `1HOUR`, `4HOURS`, `1DAY`.
fn map_kline_interval_to_dydx(interval: &str) -> &'static str {
    match interval {
        "1m" | "1min" | "1MIN" => "1MIN",
        "5m" | "5min" | "5MINS" => "5MINS",
        "15m" | "15min" | "15MINS" => "15MINS",
        "30m" | "30min" | "30MINS" => "30MINS",
        "1h" | "1hour" | "1HOUR" | "60m" => "1HOUR",
        "4h" | "4hour" | "4HOURS" => "4HOURS",
        "1d" | "1day" | "1DAY" => "1DAY",
        _ => "1MIN", // default to 1 minute
    }
}

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
                    Ok(trade) => {
                        let symbol = data.get("trades")
                            .and_then(|t| t.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|item| item.get("market"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(target_ticker_symbol)
                            .to_string();
                        Some(Ok(StreamEvent::Trade { symbol, trade }))
                    }
                    Err(e) => Some(Err(WebSocketError::Parse(e.to_string()))),
                }
            }
            "v4_markets" => {
                // The v4_markets channel is global and contains ALL markets.
                // Pass the target symbol so the parser extracts only the subscribed market.
                match DydxParser::parse_ws_ticker(&data, target_ticker_symbol) {
                    Ok(ticker) => {
                        // Return only the primary Ticker event here.
                        // The caller (start_message_loop) handles multi-emit via parse_v4_markets_events.
                        let symbol = target_ticker_symbol.to_string();
                        Some(Ok(StreamEvent::Ticker { symbol, ticker }))
                    }
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
                // Parse each entry and emit OrderUpdate / PositionUpdate / Liquidation.
                // Multiple events may be emitted per message via the multi-emit path
                // in start_message_loop (parse_subaccount_events).
                // Return None here — start_message_loop handles the multi-emit directly.
                None
            }
            "v4_blockheight" => {
                // Block height + time. No matching StreamEvent variant — acknowledged silently.
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
    /// Parse a `v4_subaccounts` or `v4_parent_subaccounts` channel message into
    /// zero or more `StreamEvent`s.
    ///
    /// Emits per entry:
    /// - `contents.orders[]`           → `StreamEvent::OrderUpdate`
    /// - `contents.fills[]`            → `StreamEvent::OrderUpdate` (fill representation)
    /// - `contents.fills[]` where liquidity=="TAKER" and type contains "LIQUIDAT"
    ///                                 → additionally `StreamEvent::Liquidation`
    /// - `contents.perpetualPositions[]` → `StreamEvent::PositionUpdate`
    fn parse_subaccount_events(text: &str) -> Vec<StreamEvent> {
        let data: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let contents = match data.get("contents") {
            Some(c) => c,
            None => return vec![],
        };

        let now = timestamp_millis() as i64;
        let mut events: Vec<StreamEvent> = Vec::new();

        // ── orders ───────────────────────────────────────────────────────────
        if let Some(orders) = contents.get("orders").and_then(|o| o.as_array()) {
            for order in orders {
                let order_id = order.get("orderId")
                    .or_else(|| order.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let client_order_id = order.get("clientId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let symbol = order.get("market")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let side = match order.get("side").and_then(|v| v.as_str()) {
                    Some("BUY") => OrderSide::Buy,
                    _ => OrderSide::Sell,
                };

                let order_type = match order.get("type").and_then(|v| v.as_str()) {
                    Some("MARKET") => OrderType::Market,
                    _ => {
                        let p = order.get("price")
                            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                                .or_else(|| v.as_f64()))
                            .unwrap_or(0.0);
                        OrderType::Limit { price: p }
                    }
                };

                let status = match order.get("status").and_then(|v| v.as_str()) {
                    Some("OPEN") => OrderStatus::Open,
                    Some("FILLED") => OrderStatus::Filled,
                    Some("CANCELED") => OrderStatus::Canceled,
                    Some("BEST_EFFORT_CANCELED") => OrderStatus::Canceled,
                    Some("BEST_EFFORT_OPENED") => OrderStatus::Open,
                    _ => OrderStatus::New,
                };

                let price = order.get("price")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()));
                let quantity = order.get("size")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()))
                    .unwrap_or(0.0);

                let timestamp = order.get("updatedAt")
                    .or_else(|| order.get("createdAt"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.timestamp_millis())
                    .unwrap_or(now);

                events.push(StreamEvent::OrderUpdate {
                    symbol,
                    event: OrderUpdateEvent {
                        order_id,
                        client_order_id,
                        side,
                        order_type,
                        status,
                        price,
                        quantity,
                        filled_quantity: 0.0,
                        average_price: None,
                        last_fill_price: None,
                        last_fill_quantity: None,
                        last_fill_commission: None,
                        commission_asset: None,
                        trade_id: None,
                        timestamp,
                    },
                });
            }
        }

        // ── fills ─────────────────────────────────────────────────────────────
        if let Some(fills) = contents.get("fills").and_then(|f| f.as_array()) {
            for fill in fills {
                let order_id = fill.get("orderId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let fill_id = fill.get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let symbol = fill.get("market")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let side = match fill.get("side").and_then(|v| v.as_str()) {
                    Some("BUY") => OrderSide::Buy,
                    _ => OrderSide::Sell,
                };

                let price = fill.get("price")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()));
                let fill_qty = fill.get("size")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()))
                    .unwrap_or(0.0);
                let fee = fill.get("fee")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()));

                let timestamp = fill.get("createdAt")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.timestamp_millis())
                    .unwrap_or(now);

                let liquidity = fill.get("liquidity").and_then(|v| v.as_str()).unwrap_or("");

                // Represent fill as an OrderUpdate with filled_quantity set
                events.push(StreamEvent::OrderUpdate {
                    symbol: symbol.clone(),
                    event: OrderUpdateEvent {
                        order_id: order_id.clone(),
                        client_order_id: None,
                        side,
                        order_type: OrderType::Market,
                        status: OrderStatus::Filled,
                        price,
                        quantity: fill_qty,
                        filled_quantity: fill_qty,
                        average_price: price,
                        last_fill_price: price,
                        last_fill_quantity: Some(fill_qty),
                        last_fill_commission: fee,
                        commission_asset: Some("USDC".to_string()),
                        trade_id: fill_id,
                        timestamp,
                    },
                });

                // Detect liquidation: fill where liquidity field == "LIQUIDATED"
                // (verified from dYdX v4 Indexer API docs — not based on fill `type`).
                let is_liquidation = liquidity.eq_ignore_ascii_case("LIQUIDATED");

                if is_liquidation {
                    let liq_side = match fill.get("side").and_then(|v| v.as_str()) {
                        Some("BUY") => TradeSide::Buy,
                        _ => TradeSide::Sell,
                    };
                    events.push(StreamEvent::Liquidation {
                        symbol,
                        side: liq_side,
                        price: price.unwrap_or(0.0),
                        quantity: fill_qty,
                        timestamp,
                        value: price.map(|p| p * fill_qty),
                    });
                }
            }
        }

        // ── perpetualPositions ────────────────────────────────────────────────
        if let Some(positions) = contents.get("perpetualPositions").and_then(|p| p.as_array()) {
            for pos in positions {
                let symbol = pos.get("market")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let pos_side = match pos.get("side").and_then(|v| v.as_str()) {
                    Some("LONG") => PositionSide::Long,
                    Some("SHORT") => PositionSide::Short,
                    _ => PositionSide::Both,
                };

                let quantity = pos.get("size")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()))
                    .unwrap_or(0.0);
                let entry_price = pos.get("entryPrice")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()))
                    .unwrap_or(0.0);
                let unrealized_pnl = pos.get("unrealizedPnl")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()))
                    .unwrap_or(0.0);
                let realized_pnl = pos.get("realizedPnl")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()));

                let timestamp = pos.get("createdAtHeight")
                    .and_then(|v| v.as_str())
                    .and_then(|_| None) // block height, not a timestamp — use now
                    .unwrap_or(now);

                events.push(StreamEvent::PositionUpdate {
                    symbol,
                    event: PositionUpdateEvent {
                        side: pos_side,
                        quantity,
                        entry_price,
                        mark_price: None,
                        unrealized_pnl,
                        realized_pnl,
                        liquidation_price: None,
                        leverage: None,
                        margin_type: None,
                        reason: None,
                        timestamp,
                    },
                });
            }
        }

        events
    }

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
                        let (ticker_sym, wants_funding, wants_mark): (String, bool, bool) = {
                            let subs = subscriptions.lock().await;
                            let mut sym = String::new();
                            let mut funding = false;
                            let mut mark = false;
                            for req in subs.iter() {
                                match req.stream_type {
                                    StreamType::Ticker => {
                                        sym = super::endpoints::normalize_symbol(&req.symbol.to_string());
                                    }
                                    StreamType::FundingRate => funding = true,
                                    StreamType::MarkPrice => mark = true,
                                    _ => {}
                                }
                            }
                            (sym, funding, mark)
                        };

                        // Multi-emit for v4_markets: ONLY when caller actually
                        // subscribed to FundingRate or MarkPrice. dYdX pushes
                        // `nextFundingRate` + `oraclePrice` on every v4_markets
                        // frame; if the consumer only asked for Ticker, those
                        // extra events look like wrong-topic routing (the
                        // FundingRate frame arrives before/instead of Ticker
                        // in some windows). Emit them only when requested.
                        if !ticker_sym.is_empty() && text.contains("v4_markets") && (wants_funding || wants_mark) {
                            let extra = Self::parse_v4_markets_extra(&text, &ticker_sym, wants_funding, wants_mark);
                            if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                for event in extra {
                                    let _ = tx.send(Ok(event));
                                }
                            }
                        }

                        // Multi-emit for v4_subaccounts / v4_parent_subaccounts:
                        // OrderUpdate, PositionUpdate, Liquidation.
                        if text.contains("v4_subaccounts") || text.contains("v4_parent_subaccounts") {
                            let sub_events = Self::parse_subaccount_events(&text);
                            if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                for event in sub_events {
                                    let _ = tx.send(Ok(event));
                                }
                            }
                            // parse_channel_data returns None for these channels — skip handle_message.
                        } else if let Some(event) = Self::handle_message(&text, &ticker_sym) {
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

    /// Extract FundingRate and IndexPrice events from a raw `v4_markets` message.
    ///
    /// `v4_markets` pushes `nextFundingRate` and `oraclePrice` per market.
    /// The Ticker is already emitted by `handle_message`; this method emits
    /// the additional derivative events so consumers get the full picture.
    fn parse_v4_markets_extra(
        text: &str,
        target_symbol: &str,
        emit_funding: bool,
        emit_mark: bool,
    ) -> Vec<StreamEvent> {
        let data: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let contents = match data.get("contents") {
            Some(c) => c,
            None => return vec![],
        };

        // Support both snapshot (contents.markets.{SYM}) and delta (contents IS the map)
        let markets = contents.get("markets")
            .and_then(|m| m.as_object())
            .or_else(|| contents.as_object());

        let market = match markets.and_then(|m| m.get(target_symbol)) {
            Some(m) => m,
            None => return vec![],
        };

        let get_f64_str = |key: &str| -> Option<f64> {
            market.get(key).and_then(|v| {
                v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
        };

        let now = crate::core::utils::timestamp_millis() as i64;
        let mut events = Vec::with_capacity(2);

        if emit_funding {
            if let Some(rate) = get_f64_str("nextFundingRate") {
                events.push(StreamEvent::FundingRate {
                    symbol: target_symbol.to_string(),
                    rate,
                    next_funding_time: None,
                    timestamp: now,
                });
            }
        }

        if emit_mark {
            if let Some(idx_px) = get_f64_str("oraclePrice") {
                events.push(StreamEvent::IndexPrice {
                    symbol: target_symbol.to_string(),
                    price: idx_px,
                    timestamp: now,
                });
            }
        }

        events
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
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
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

    async fn disconnect(&self) -> WebSocketResult<()> {
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

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let symbol_str = request.symbol.to_string();
        let norm = normalize_symbol(&symbol_str);

        // Insert subscription BEFORE sending the subscribe frame so that the message
        // loop can resolve ticker_sym before the server's snapshot ACK arrives.
        // v4_markets snapshots arrive within milliseconds of the subscribe frame —
        // without pre-insertion the symbol lookup races and the snapshot is silently dropped.
        self.subscriptions.lock().await.insert(request.clone());

        let send_result = match &request.stream_type {
            StreamType::Ticker => {
                self.send_subscribe_no_id("v4_markets").await
            }
            StreamType::Orderbook => {
                self.send_subscribe_with_id("v4_orderbook", &norm).await
            }
            StreamType::Trade => {
                self.send_subscribe_with_id("v4_trades", &norm).await
            }
            StreamType::Kline { interval } => {
                // dYdX v4_candles id format: "{SYMBOL}/{RESOLUTION}" e.g. "BTC-USD/1MIN"
                let resolution = map_kline_interval_to_dydx(interval);
                let id = format!("{}/{}", norm, resolution);
                self.send_subscribe_with_id("v4_candles", &id).await
            }
            other => {
                // Unsupported — remove the speculatively inserted entry and bail.
                self.subscriptions.lock().await.remove(&request);
                return Err(WebSocketError::ProtocolError(
                    format!("Stream type {:?} not supported", other)
                ));
            }
        };

        if let Err(e) = send_result {
            // Wire send failed — roll back the speculative insertion.
            self.subscriptions.lock().await.remove(&request);
            return Err(e);
        }

        Ok(())
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let symbol_str = request.symbol.to_string();
        let norm = normalize_symbol(&symbol_str);

        match &request.stream_type {
            StreamType::Ticker => {
                self.send_unsubscribe_no_id("v4_markets").await?;
            }
            StreamType::Orderbook => {
                self.send_unsubscribe_with_id("v4_orderbook", &norm).await?;
            }
            StreamType::Trade => {
                self.send_unsubscribe_with_id("v4_trades", &norm).await?;
            }
            StreamType::Kline { interval } => {
                let resolution = map_kline_interval_to_dydx(interval);
                let id = format!("{}/{}", norm, resolution);
                self.send_unsubscribe_with_id("v4_candles", &id).await?;
            }
            _ => {}
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

    /// dYdX v4 orderbook capabilities.
    ///
    /// Single channel `v4_orderbook`: snapshot on subscribe, then incremental deltas.
    /// Depth is server-controlled (up to 100 levels per side). No client depth param.
    /// No checksum. Carries `message_id` (connection-level sequence) for gap detection.
    /// No prev-sequence in individual messages — gap detection relies on connection counter.
    /// Perpetuals only (account_type is irrelevant, but we match for consistency).
    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: &[],
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
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
