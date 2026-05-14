//! # OKX WebSocket Implementation
//!
//! WebSocket connector for OKX API v5.
//!
//! ## Features
//! - Public and private channels
//! - Text-based ping/pong (send "ping", receive "pong")
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Authentication
//! - Private channels require login via WebSocket message
//! - Signature: `timestamp + "GET" + "/users/self/verify"`
//!
//! ## Ping/Pong
//! - OKX uses text-based ping/pong (not WebSocket frames)
//! - Client sends text "ping" every 20 seconds
//! - Server responds with text "pong"
//!
//! ## Mutex starvation fix
//! - The WebSocket stream is split into a write half (sink) and a read half
//!   (stream) immediately after connecting.
//! - The ping task exclusively owns the sink.
//! - The message handler exclusively owns the read stream.
//! - Neither task can block the other by holding a shared lock during I/O.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Instant};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream,
};

use crate::core::{
    AccountType, ConnectionStatus, Credentials, ExchangeResult, OrderBook,
    StreamEvent, SubscriptionRequest, timestamp_iso8601,
};
use crate::core::types::OrderbookDelta;
use crate::core::traits::WebSocketConnector;
use crate::core::types::{WebSocketError, WebSocketResult, OrderbookCapabilities, WsBookChannel, ChecksumInfo, ChecksumAlgorithm};

use super::auth::OkxAuth;
use super::endpoints::{format_symbol, OkxUrls};
use super::parser::OkxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WsStream, Message>;
type WsReader = futures_util::stream::SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// OKX WebSocket connector.
///
/// The underlying WebSocket stream is split into a write half (`ws_sink`) and a
/// read half (`ws_reader`) so that the ping task and the message handler each
/// hold only the half they need.  This eliminates the mutex-starvation problem
/// where `stream.next().await` would block the lock indefinitely between market
/// data messages, preventing the ping task from ever acquiring it.
pub struct OkxWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<OkxAuth>,
    /// URLs (mainnet/testnet)
    urls: OkxUrls,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender — behind StdMutex so event_stream() can subscribe
    /// without contending with the async message loop.
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Write half – used by `subscribe`, `unsubscribe`, `disconnect`, and the
    /// ping background task.
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    /// Read half – used exclusively by the message-handler background task.
    ws_reader: Arc<Mutex<Option<WsReader>>>,
    /// Timestamp of the most recently sent ping.
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
    /// Connected to private channel
    is_private: bool,
}

impl OkxWebSocket {
    /// Create new OKX WebSocket connector.
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            OkxUrls::TESTNET
        } else {
            OkxUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(OkxAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_sink: Arc::new(Mutex::new(None)),
            ws_reader: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
            is_private: false,
        })
    }

    /// Send the OKX WebSocket login message via `sink`.
    async fn send_login(&self, sink: &mut WsSink) -> WebSocketResult<()> {
        let auth = self.auth.as_ref().ok_or_else(|| {
            WebSocketError::Auth("Private channels require authentication".to_string())
        })?;

        let timestamp = timestamp_iso8601();
        let signature = auth.sign_websocket_login(&timestamp);

        let login_msg = json!({
            "op": "login",
            "args": [{
                "apiKey": auth.api_key(),
                "passphrase": auth.passphrase,
                "timestamp": timestamp,
                "sign": signature,
            }]
        });

        sink.send(Message::Text(login_msg.to_string()))
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        Ok(())
    }

    /// Start the ping background task.
    ///
    /// Acquires only the **sink** lock to send "ping" every 20 seconds.
    /// The read half is never touched here, so `start_message_handler` can
    /// block on `reader.next().await` without starving ping.
    fn start_ping_task(
        ws_sink: Arc<Mutex<Option<WsSink>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));

            loop {
                ticker.tick().await;

                let mut sink_guard = ws_sink.lock().await;
                if let Some(sink) = sink_guard.as_mut() {
                    // OKX uses text-based ping/pong, not WebSocket ping frames.
                    if sink.send(Message::Text("ping".to_string())).await.is_ok() {
                        *last_ping.lock().await = Instant::now();
                    } else {
                        // Connection lost; stop the task.
                        break;
                    }
                } else {
                    // Sink has been cleared (disconnect was called); stop.
                    break;
                }
            }
        });
    }

    /// Start the message-handler background task.
    ///
    /// Acquires only the **reader** lock.  Because `next().await` is called
    /// while the reader lock is held for the duration of each await, no other
    /// task competes for that lock — and the sink lock is never touched here.
    fn start_message_handler(
        ws_reader: Arc<Mutex<Option<WsReader>>>,
        broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            loop {
                // Poll the next message from the read half.
                let msg = {
                    let mut reader_guard = ws_reader.lock().await;
                    if let Some(reader) = reader_guard.as_mut() {
                        reader.next().await
                    } else {
                        break;
                    }
                };

                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Record round-trip time on pong response.
                        if text.trim() == "pong" {
                            let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                            *ws_ping_rtt_ms.lock().await = rtt;
                            continue;
                        }

                        // Parse JSON message.
                        if let Ok(value) = serde_json::from_str::<Value>(&text) {
                            // Handle event messages.
                            if let Some(event) = value.get("event").and_then(|e| e.as_str()) {
                                match event {
                                    "subscribe" | "unsubscribe" | "login" => {
                                        // Acknowledgment — nothing to broadcast.
                                        continue;
                                    }
                                    "error" => {
                                        let code = value
                                            .get("code")
                                            .and_then(|c| c.as_str())
                                            .unwrap_or("unknown");
                                        let msg_text = value
                                            .get("msg")
                                            .and_then(|m| m.as_str())
                                            .unwrap_or("Unknown error");
                                        let tx_guard = broadcast_tx.lock().unwrap();
                                        if let Some(ref tx) = *tx_guard {
                                            let _ = tx.send(Err(
                                                WebSocketError::ProtocolError(format!(
                                                    "{}: {}",
                                                    code, msg_text
                                                )),
                                            ));
                                        }
                                        continue;
                                    }
                                    _ => {}
                                }
                            }

                            // Handle data pushes.
                            if let Some(arg) = value.get("arg") {
                                if let Some(channel) =
                                    arg.get("channel").and_then(|c| c.as_str())
                                {
                                    // Extract top-level action ("snapshot" | "update")
                                    let action = value.get("action").and_then(|a| a.as_str());
                                    // Extract instId from arg for channels where data
                                    // array items don't carry the symbol (e.g. candles).
                                    let arg_inst_id = arg.get("instId")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    if let Some(data_arr) =
                                        value.get("data").and_then(|d| d.as_array())
                                    {
                                        for data in data_arr {
                                            let events = Self::parse_channel_data(
                                                channel, data, action, arg_inst_id,
                                            );
                                            for ev in events {
                                                let tx_guard = broadcast_tx.lock().unwrap();
                                                if let Some(ref tx) = *tx_guard {
                                                    let _ = tx.send(Ok(ev));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Some(Err(_)) | None => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {}
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            let _ = broadcast_tx.lock().unwrap().take();
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Parse channel data to 0-N [`StreamEvent`]s.
    ///
    /// `action` is taken from the top-level `"action"` field of the OKX push
    /// message.  OKX sets it to `"snapshot"` for the initial full book and
    /// `"update"` for incremental deltas.
    ///
    /// `arg_inst_id` is the `instId` from the subscription `arg` object; used
    /// by candle channels where the data array does not contain the symbol.
    ///
    /// Most channels return exactly one event. The `tickers` channel returns
    /// the primary Ticker plus any supplementary FundingRate/MarkPrice/
    /// OpenInterestUpdate events when those fields are present (linear/inverse).
    fn parse_channel_data(
        channel: &str,
        data: &Value,
        action: Option<&str>,
        arg_inst_id: &str,
    ) -> Vec<StreamEvent> {
        let parse_f64_field = |v: &Value| -> Option<f64> {
            v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64())
        };

        match channel {
            "tickers" => {
                let mut events = Vec::new();
                if let Ok(ticker) = OkxParser::parse_ws_ticker(data) {
                    let symbol = ticker.symbol.clone();
                    let ts = ticker.timestamp;
                    events.push(StreamEvent::Ticker(ticker));

                    // Supplementary events from linear/inverse SWAP/FUTURES tickers
                    if let Some(rate) = data.get("fundingRate").and_then(|v| parse_f64_field(v)) {
                        let next_funding_time = data.get("nextFundingTime")
                            .and_then(|v| parse_f64_field(v))
                            .map(|ms| ms as i64);
                        events.push(StreamEvent::FundingRate {
                            symbol: symbol.clone(),
                            rate,
                            next_funding_time,
                            timestamp: ts,
                        });
                    }
                    if let Some(mark_price) = data.get("markPx").and_then(|v| parse_f64_field(v)) {
                        let index_price = data.get("indexPx").and_then(|v| parse_f64_field(v));
                        events.push(StreamEvent::MarkPrice {
                            symbol: symbol.clone(),
                            mark_price,
                            index_price,
                            timestamp: ts,
                        });
                    }
                    if let Some(open_interest) = data.get("openInterest").and_then(|v| parse_f64_field(v)) {
                        let open_interest_value = data.get("openInterestValue")
                            .and_then(|v| parse_f64_field(v));
                        events.push(StreamEvent::OpenInterestUpdate {
                            symbol,
                            open_interest,
                            open_interest_value,
                            timestamp: ts,
                        });
                    }
                }
                events
            }
            "books" | "books5" | "books-l2-tbt" | "books50-l2-tbt" => {
                let Ok((asks, bids)) = OkxParser::parse_ws_orderbook(data) else { return vec![] };
                let timestamp = OkxParser::get_i64(data, "ts").unwrap_or(0);

                // OKX sequences: seqId → first_update_id, prevSeqId → prev_update_id
                let seq_id = data.get("seqId").and_then(|v| v.as_u64());
                let prev_seq_id = data.get("prevSeqId").and_then(|v| v.as_u64());
                let checksum = data.get("checksum").and_then(|v| v.as_i64());

                if action == Some("snapshot") {
                    let orderbook = OrderBook {
                        asks,
                        bids,
                        timestamp,
                        sequence: None,
                        last_update_id: seq_id,
                        first_update_id: seq_id,
                        prev_update_id: prev_seq_id,
                        event_time: Some(timestamp),
                        transaction_time: None,
                        checksum,
                    };
                    vec![StreamEvent::OrderbookSnapshot(orderbook)]
                } else {
                    // "update" or anything else → delta
                    let delta = OrderbookDelta {
                        asks,
                        bids,
                        timestamp,
                        first_update_id: seq_id,
                        last_update_id: seq_id,
                        prev_update_id: prev_seq_id,
                        event_time: Some(timestamp),
                        checksum,
                    };
                    vec![StreamEvent::OrderbookDelta(delta)]
                }
            }
            "trades" => OkxParser::parse_ws_trade(data)
                .ok()
                .map(StreamEvent::Trade)
                .into_iter()
                .collect(),
            "candle1m" | "candle5m" | "candle15m" | "candle30m" | "candle1H"
            | "candle4H" | "candle1D" => OkxParser::parse_ws_kline(data)
                .ok()
                .map(StreamEvent::Kline)
                .into_iter()
                .collect(),
            "orders" => OkxParser::parse_ws_order_update(data)
                .ok()
                .map(StreamEvent::OrderUpdate)
                .into_iter()
                .collect(),
            "account" => {
                if let Some(details) = data.get("details").and_then(|d| d.as_array()) {
                    for detail in details {
                        if let Ok(event) = OkxParser::parse_ws_balance_update(detail) {
                            return vec![StreamEvent::BalanceUpdate(event)];
                        }
                    }
                }
                vec![]
            }
            "positions" => OkxParser::parse_ws_position_update(data)
                .ok()
                .map(StreamEvent::PositionUpdate)
                .into_iter()
                .collect(),
            "mark-price" => {
                // OKX WS mark-price: { markPx, instId, ts }
                let symbol = data.get("instId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let Some(mark_price) = data.get("markPx").and_then(|v| parse_f64_field(v)) else {
                    return vec![];
                };
                let timestamp = data.get("ts")
                    .and_then(|v| parse_f64_field(v))
                    .map(|ms| ms as i64)
                    .unwrap_or(0);
                vec![StreamEvent::MarkPrice { symbol, mark_price, index_price: None, timestamp }]
            }
            "funding-rate" => {
                // OKX WS funding-rate: { fundingRate, instId, fundingTime, nextFundingTime }
                let symbol = data.get("instId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let Some(rate) = data.get("fundingRate").and_then(|v| parse_f64_field(v)) else {
                    return vec![];
                };
                let next_funding_time = data.get("nextFundingTime")
                    .and_then(|v| parse_f64_field(v))
                    .map(|ms| ms as i64);
                let timestamp = data.get("fundingTime")
                    .and_then(|v| parse_f64_field(v))
                    .map(|ms| ms as i64)
                    .unwrap_or(0);
                vec![StreamEvent::FundingRate { symbol, rate, next_funding_time, timestamp }]
            }
            "liquidation-orders" => {
                // OKX WS liquidation-orders: { instId, details: [{side, sz, fillPx/bkPx, ts}] }
                use crate::core::types::TradeSide;
                let symbol = data.get("instId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let Some(details) = data.get("details").and_then(|d| d.as_array()) else {
                    return vec![];
                };
                let Some(detail) = details.first() else { return vec![] };
                let side_str = detail.get("side").and_then(|s| s.as_str()).unwrap_or("buy");
                let Some(price) = detail.get("fillPx")
                    .or_else(|| detail.get("bkPx"))
                    .and_then(|v| parse_f64_field(v)) else { return vec![] };
                let Some(quantity) = detail.get("sz").and_then(|v| parse_f64_field(v)) else {
                    return vec![];
                };
                let timestamp: i64 = detail.get("ts")
                    .and_then(|v| parse_f64_field(v))
                    .map(|ms| ms as i64)
                    .unwrap_or(0);
                // "buy" = long being liquidated; "sell" = short being liquidated
                let side = match side_str {
                    "buy" => TradeSide::Buy,
                    _ => TradeSide::Sell,
                };
                vec![StreamEvent::Liquidation {
                    symbol,
                    side,
                    price,
                    quantity,
                    timestamp,
                    value: Some(price * quantity),
                }]
            }
            "index-tickers" => {
                // OKX WS index-tickers: { instId, idxPx, ts }
                // Maps to StreamType::IndexPrice
                let symbol = data.get("instId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let Some(price) = data.get("idxPx").and_then(|v| parse_f64_field(v)) else {
                    return vec![];
                };
                let timestamp = data.get("ts")
                    .and_then(|v| parse_f64_field(v))
                    .map(|ms| ms as i64)
                    .unwrap_or(0);
                vec![StreamEvent::IndexPrice { symbol, price, timestamp }]
            }
            ch if ch.starts_with("mark-price-candle") => {
                // OKX WS mark-price-candle<interval>: data is [ ts, o, h, l, c, confirm ]
                // Maps to StreamType::MarkPriceKline { interval }
                let Ok(kline) = OkxParser::parse_ws_price_candle(data) else { return vec![] };
                // Recover interval from channel name: "mark-price-candle1m" → "1m"
                let interval = ch.trim_start_matches("mark-price-candle").to_string();
                // Symbol comes from the subscription arg, not the data array.
                vec![StreamEvent::MarkPriceKline {
                    symbol: arg_inst_id.to_string(),
                    interval,
                    kline,
                }]
            }
            ch if ch.starts_with("index-candle") => {
                // OKX WS index-candle<interval>: data is [ ts, o, h, l, c, confirm ]
                // Maps to StreamType::IndexPriceKline { interval }
                let Ok(kline) = OkxParser::parse_ws_price_candle(data) else { return vec![] };
                let interval = ch.trim_start_matches("index-candle").to_string();
                vec![StreamEvent::IndexPriceKline {
                    symbol: arg_inst_id.to_string(),
                    interval,
                    kline,
                }]
            }
            ch if ch.starts_with("funding-rate-candle") => {
                // TODO: OKX funding-rate-candle<interval> — funding rate values over time.
                // No direct StreamEvent variant fits (FundingRate is a scalar, not OHLC).
                // Defer: leave unhandled until a FundingRateKline variant is added.
                let _ = ch;
                vec![]
            }
            "estimated-price" => {
                // OKX option settlement estimated price — no matching StreamEvent variant.
                vec![]
            }
            "price-limit" => {
                // OKX price-limit pushes upper/lower price bounds — no matching StreamEvent variant.
                vec![]
            }
            "opt-summary" => {
                // OKX opt-summary: option Greeks (delta, gamma, theta, vega).
                // No StreamEvent variant for option Greeks yet.
                vec![]
            }
            _ => vec![],
        }
    }

    /// Returns the most recently measured WebSocket ping round-trip time in
    /// milliseconds.  Returns `0` until at least one pong has been received.
    pub fn ping_rtt_ms(&self) -> u64 {
        match self.ws_ping_rtt_ms.try_lock() {
            Ok(guard) => *guard,
            Err(_) => 0,
        }
    }

    /// Get a shared reference to the ping RTT value for external monitoring.
    ///
    /// The returned `Arc<Mutex<u64>>` is updated by the internal ping/pong
    /// handler each time a "pong" response is received from OKX.  Callers
    /// can cheaply poll the value (e.g. with `try_lock`) without blocking the
    /// WebSocket task.
    pub fn ping_rtt_handle(&self) -> Arc<Mutex<u64>> {
        self.ws_ping_rtt_ms.clone()
    }

    /// Send a subscribe message for a dynamically-named channel (e.g. mark-price-candle1m).
    async fn subscribe_dynamic_channel(
        &mut self,
        channel: String,
        request: SubscriptionRequest,
    ) -> WebSocketResult<()> {
        let account_type = request.account_type;
        let inst_id = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        let sub_msg = json!({
            "op": "subscribe",
            "args": [{ "channel": channel, "instId": inst_id }]
        });

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            self.subscriptions.lock().await.insert(request);
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    /// Send an unsubscribe message for a dynamically-named channel.
    async fn unsubscribe_dynamic_channel(
        &mut self,
        channel: String,
        request: SubscriptionRequest,
    ) -> WebSocketResult<()> {
        let account_type = request.account_type;
        let inst_id = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        let unsub_msg = json!({
            "op": "unsubscribe",
            "args": [{ "channel": channel, "instId": inst_id }]
        });

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(unsub_msg.to_string()))
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            self.subscriptions.lock().await.remove(&request);
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for OkxWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Determine URL (private vs public channel).
        let url = if self.auth.is_some() {
            self.is_private = true;
            self.urls.ws_url(true)
        } else {
            self.is_private = false;
            self.urls.ws_url(false)
        };

        // Establish the WebSocket connection.
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Split into independent read and write halves — this is the core fix.
        // With a unified stream, `next().await` in the message handler holds
        // the mutex for the entire duration between messages, starving the ping
        // task.  After splitting, each half has its own mutex and neither task
        // can block the other.
        let (mut sink, reader) = ws_stream.split();

        // Send login before storing halves (only the sink is needed here).
        if self.is_private {
            self.send_login(&mut sink).await?;
        }

        // Store both halves, replacing any previous connection.
        *self.ws_sink.lock().await = Some(sink);
        *self.ws_reader.lock().await = Some(reader);

        *self.status.lock().await = ConnectionStatus::Connected;

        // Create broadcast channel and store sender
        let (tx, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(tx);

        // Start background tasks — each holds only its own half.
        Self::start_ping_task(self.ws_sink.clone(), self.last_ping.clone());
        Self::start_message_handler(
            self.ws_reader.clone(),
            self.broadcast_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Send a close frame via the sink, then drop both halves.
        {
            let mut sink_guard = self.ws_sink.lock().await;
            if let Some(sink) = sink_guard.as_mut() {
                let _ = sink.send(Message::Close(None)).await;
            }
            *sink_guard = None;
        }
        *self.ws_reader.lock().await = None;
        *self.status.lock().await = ConnectionStatus::Disconnected;
        let _ = self.broadcast_tx.lock().unwrap().take();
        Ok(())
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channel = match &request.stream_type {
            crate::core::StreamType::Ticker => "tickers",
            crate::core::StreamType::Orderbook => {
                // OKX depth channels:
                //   books          → full 400-level snapshot+update
                //   books5         → top-5 levels
                //   books-l2-tbt   → 400-level tick-by-tick
                //   books50-l2-tbt → 50-level tick-by-tick
                match request.depth {
                    Some(5) => "books5",
                    Some(50) => "books50-l2-tbt",
                    _ => "books",
                }
            }
            crate::core::StreamType::OrderbookDelta => {
                match request.depth {
                    Some(50) => "books50-l2-tbt",
                    _ => "books-l2-tbt",
                }
            }
            crate::core::StreamType::Trade => "trades",
            crate::core::StreamType::Kline { interval } => match interval.as_str() {
                "1m" => "candle1m",
                "5m" => "candle5m",
                "15m" => "candle15m",
                "30m" => "candle30m",
                "1h" => "candle1H",
                "4h" => "candle4H",
                "1d" => "candle1D",
                _ => "candle1H",
            },
            crate::core::StreamType::MarkPrice => "mark-price",
            crate::core::StreamType::FundingRate => "funding-rate",
            crate::core::StreamType::Liquidation => "liquidation-orders",
            crate::core::StreamType::OrderUpdate => "orders",
            crate::core::StreamType::BalanceUpdate => "account",
            crate::core::StreamType::PositionUpdate => "positions",
            crate::core::StreamType::IndexPrice => "index-tickers",
            crate::core::StreamType::MarkPriceKline { interval } => {
                // Map internal interval to OKX mark-price-candle<interval> channel.
                // OKX intervals: 1m, 3m, 5m, 15m, 30m, 1H, 2H, 4H, 6H, 12H, 1D, etc.
                let okx_interval = super::endpoints::map_kline_interval(interval.as_str());
                // Return a leaked string; channel string lifetime is 'static for the match arm.
                // Since we can't easily return a dynamic &'static str here, use a known set.
                return self.subscribe_dynamic_channel(
                    format!("mark-price-candle{}", okx_interval),
                    request,
                ).await;
            }
            crate::core::StreamType::IndexPriceKline { interval } => {
                let okx_interval = super::endpoints::map_kline_interval(interval.as_str());
                return self.subscribe_dynamic_channel(
                    format!("index-candle{}", okx_interval),
                    request,
                ).await;
            }
            _ => "",
        };

        // For OKX the instId depends on account type.
        let account_type = request.account_type;
        let inst_id = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        let sub_msg = json!({
            "op": "subscribe",
            "args": [{
                "channel": channel,
                "instId": inst_id,
            }]
        });

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            self.subscriptions.lock().await.insert(request);
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let channel = match &request.stream_type {
            crate::core::StreamType::Ticker => "tickers",
            crate::core::StreamType::Orderbook => "books",
            crate::core::StreamType::OrderbookDelta => "books",
            crate::core::StreamType::Trade => "trades",
            crate::core::StreamType::Kline { interval: _ } => "candle1H",
            crate::core::StreamType::MarkPrice => "mark-price",
            crate::core::StreamType::FundingRate => "funding-rate",
            crate::core::StreamType::Liquidation => "liquidation-orders",
            crate::core::StreamType::OrderUpdate => "orders",
            crate::core::StreamType::BalanceUpdate => "account",
            crate::core::StreamType::PositionUpdate => "positions",
            crate::core::StreamType::IndexPrice => "index-tickers",
            crate::core::StreamType::MarkPriceKline { interval } => {
                let okx_interval = super::endpoints::map_kline_interval(interval.as_str());
                return self.unsubscribe_dynamic_channel(
                    format!("mark-price-candle{}", okx_interval),
                    request,
                ).await;
            }
            crate::core::StreamType::IndexPriceKline { interval } => {
                let okx_interval = super::endpoints::map_kline_interval(interval.as_str());
                return self.unsubscribe_dynamic_channel(
                    format!("index-candle{}", okx_interval),
                    request,
                ).await;
            }
            _ => "",
        };

        // Use the same account_type that was used in subscribe().
        let account_type = request.account_type;
        let inst_id = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);

        let unsub_msg = json!({
            "op": "unsubscribe",
            "args": [{
                "channel": channel,
                "instId": inst_id,
            }]
        });

        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(sink) = sink_guard.as_mut() {
            sink.send(Message::Text(unsub_msg.to_string()))
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
            self.subscriptions.lock().await.remove(&request);
            Ok(())
        } else {
            Err(WebSocketError::ConnectionError("Not connected".to_string()))
        }
    }

    fn event_stream(
        &self,
    ) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send + 'static>> {
        let tx_guard = self.broadcast_tx.lock().unwrap();
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

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(guard) => *guard,
            // If the lock is held by a setter, assume we are still connected.
            Err(_) => ConnectionStatus::Connected,
        }
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
        static OKX_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("bbo-tbt",        1,   10),
            WsBookChannel::snapshot("books5",         5,   100),
            WsBookChannel::delta("books",             Some(400), Some(100)),
            WsBookChannel::delta("books50-l2-tbt",    Some(50),  Some(10)).with_auth_tier(),
            WsBookChannel::delta("books-l2-tbt",      Some(400), Some(10)).with_auth_tier(),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 5, 50, 400],
            ws_default_depth: Some(400),
            rest_max_depth: Some(400),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[10, 100],
            default_speed_ms: Some(100),
            ws_channels: OKX_CHANNELS,
            checksum: Some(ChecksumInfo {
                algorithm: ChecksumAlgorithm::Crc32Interleaved,
                levels_per_side: 25,
                opt_in: false,
            }),
            has_sequence: true,
            has_prev_sequence: true,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}
