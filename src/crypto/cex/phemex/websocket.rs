//! # Phemex WebSocket Implementation
//!
//! WebSocket connector for Phemex Spot and Contracts.
//!
//! ## Features
//! - Public and private channels
//! - Heartbeat (ping/pong every 5 seconds)
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Protocol
//! - Request: `{ "id": N, "method": "...", "params": [...] }`
//! - Response: `{ "id": N, "error": null, "result": {...} }`
//! - Server Push: `{ "type": "snapshot"|"incremental", "symbol": "...", ... }`
//!
//! ## CRITICAL Requirements
//! - Heartbeat MUST be sent every 5 seconds (recommended)
//! - Max 30 seconds without ping = disconnection
//! - Max 5 concurrent connections per user
//! - Max 20 subscriptions per connection

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, Message}, MaybeTlsStream, WebSocketStream};

use crate::core::types::{TradeSide, WebSocketError, WebSocketResult, OrderBookLevel, OrderbookDelta as OrderbookDeltaData, OrderbookCapabilities};
use crate::core::traits::WebSocketConnector;
use crate::core::{
    AccountType, ConnectionStatus, Credentials, ExchangeError, ExchangeResult, StreamEvent,
    StreamType, SubscriptionRequest, timestamp_millis,
};

use super::auth::PhemexAuth;
use super::endpoints::{PhemexUrls, format_symbol, unscale_price};

/// Heartbeat interval (5 seconds recommended by Phemex)
const HEARTBEAT_INTERVAL_SECS: u64 = 5;

/// Global request ID counter for Phemex WebSocket messages
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a unique request ID
fn next_request_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET STREAM TYPE
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// ═══════════════════════════════════════════════════════════════════════════════
// PHEMEX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Phemex WebSocket connector
///
/// Implements the `WebSocketConnector` trait for Phemex exchange.
///
/// ## Usage
/// ```ignore
/// let mut ws = PhemexWebSocket::new(Some(credentials), false).await?;
/// ws.connect(AccountType::FuturesCross).await?;
/// ws.subscribe(SubscriptionRequest::ticker(Symbol::new("BTC", "USD"))).await?;
///
/// let mut stream = ws.event_stream();
/// // use stream.next().await to receive events
/// ```
pub struct PhemexWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<PhemexAuth>,
    /// URLs (mainnet/testnet)
    urls: PhemexUrls,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender — behind StdMutex so event_stream() can subscribe
    /// without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
    /// Default price scale (used for Ep values; can be overridden per-symbol)
    price_scale: u8,
}

impl PhemexWebSocket {
    /// Create new WebSocket connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            PhemexUrls::TESTNET
        } else {
            PhemexUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(PhemexAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            account_type: AccountType::FuturesCross,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_stream: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
            // Default price scale for BTCUSD-type contracts.
            // For spot or other symbols this may differ; a production implementation
            // would fetch /public/products for exact scales.
            price_scale: 4,
        })
    }

    /// Connect to Phemex WebSocket endpoint
    async fn connect_ws(&self) -> ExchangeResult<WsStream> {
        let ws_url = self.urls.ws_url(self.account_type);

        let mut request = ws_url
            .into_client_request()
            .map_err(|e| ExchangeError::Network(format!("Request build failed: {}", e)))?;
        request.headers_mut().insert(
            "Origin",
            "https://phemex.com".parse().unwrap(),
        );

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Authenticate WebSocket connection (for private channels)
    async fn authenticate(auth: &PhemexAuth, stream: &mut WsStream) -> ExchangeResult<()> {
        let (api_key, expiry, signature) = auth.sign_websocket();

        let auth_msg = json!({
            "method": "user.auth",
            "params": ["API", api_key, signature, expiry],
            "id": next_request_id()
        });

        let msg_text = auth_msg.to_string();
        stream
            .send(Message::Text(msg_text))
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to send auth message: {}", e)))?;

        // Wait for auth response (with timeout)
        let response = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .map_err(|_| ExchangeError::Auth("Authentication timeout".to_string()))?;

        if let Some(Ok(Message::Text(text))) = response {
            let parsed: Value = serde_json::from_str(&text)
                .map_err(|e| ExchangeError::Parse(format!("Failed to parse auth response: {}", e)))?;

            // Phemex auth response: { "error": null, "id": N, "result": { "status": "success" } }
            if parsed.get("error").is_some_and(|e| !e.is_null()) {
                let err_msg = parsed
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Authentication failed");
                return Err(ExchangeError::Auth(err_msg.to_string()));
            }

            let status = parsed
                .get("result")
                .and_then(|r| r.get("status"))
                .and_then(|s| s.as_str());

            if status == Some("success") {
                Ok(())
            } else {
                Err(ExchangeError::Auth("Authentication failed: unexpected response".to_string()))
            }
        } else {
            Err(ExchangeError::Auth("Invalid auth response".to_string()))
        }
    }

    /// Start message handling task
    fn start_message_handler(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        price_scale: u8,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            loop {
                let mut stream_guard = ws_stream.lock().await;
                let stream = match stream_guard.as_mut() {
                    Some(s) => s,
                    None => {
                        drop(stream_guard);
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                };

                match stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        drop(stream_guard);
                        if let Err(e) = Self::handle_message(&text, &event_tx, price_scale).await {
                            if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                                let _ = tx.send(Err(e));
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        drop(stream_guard);
                        // Record RTT for the WS-level ping sent by start_ping_task
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Some(Ok(Message::Close(_))) => {
                        drop(stream_guard);
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Some(Err(e)) => {
                        drop(stream_guard);
                        if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
                        break;
                    }
                    None => {
                        drop(stream_guard);
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {
                        drop(stream_guard);
                    }
                }
            }
            let _ = event_tx.lock().unwrap().take();
        });
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        text: &str,
        event_tx: &Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        price_scale: u8,
    ) -> WebSocketResult<()> {
        let msg: Value = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Case 1: Response to a request (has "id" field)
        // Format: { "id": N, "error": null|{...}, "result": ... }
        if msg.get("id").is_some() {
            // Check for error
            if let Some(error) = msg.get("error") {
                if !error.is_null() {
                    let err_msg = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    let err_code = error
                        .get("code")
                        .and_then(|c| c.as_i64())
                        .unwrap_or(-1);
                    return Err(WebSocketError::ProtocolError(format!(
                        "Phemex error {}: {}",
                        err_code, err_msg
                    )));
                }
            }

            // Check if it's a pong response
            if msg.get("result").is_some_and(|r| r.as_str() == Some("pong")) {
                return Ok(());
            }

            // Subscription success or other response - ignore
            return Ok(());
        }

        // Case 2: Server push message (has data fields like "book", "trades", "kline", "market24h", "tick")
        // These messages have "symbol" and "type" fields
        if let Some(event) = Self::parse_push_message(&msg, price_scale)? {
            if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                let _ = tx.send(Ok(event));
            }
        }

        Ok(())
    }

    /// Parse a server push message into a StreamEvent
    fn parse_push_message(
        msg: &Value,
        price_scale: u8,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let symbol = msg
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        // Determine message type by which data field is present
        if msg.get("book").is_some() {
            return Self::parse_orderbook_push(msg, symbol, price_scale);
        }

        if msg.get("trades").is_some() {
            return Self::parse_trades_push(msg, symbol, price_scale);
        }

        if msg.get("kline").is_some() {
            return Self::parse_kline_push(msg, symbol, price_scale);
        }

        if msg.get("market24h").is_some() {
            return Self::parse_ticker_push(msg, symbol, price_scale);
        }

        if msg.get("tick").is_some() {
            return Self::parse_tick_push(msg, symbol, price_scale);
        }

        // AOP (Account-Order-Position) messages
        if msg.get("accounts").is_some()
            || msg.get("orders").is_some()
            || msg.get("positions").is_some()
        {
            return Self::parse_aop_push(msg);
        }

        // Unknown message - ignore
        Ok(None)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse orderbook push message
    fn parse_orderbook_push(
        msg: &Value,
        _symbol: &str,
        price_scale: u8,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let book = msg.get("book").ok_or_else(|| {
            WebSocketError::Parse("Missing 'book' field".to_string())
        })?;

        let msg_type = msg
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("snapshot");

        let timestamp = msg
            .get("timestamp")
            .and_then(|t| t.as_i64())
            .map(|ns| ns / 1_000_000) // nanoseconds to milliseconds
            .unwrap_or_else(|| timestamp_millis() as i64);

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            book.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 {
                                return None;
                            }
                            let price_ep = pair[0].as_i64()?;
                            let size = pair[1].as_f64().or_else(|| {
                                pair[1].as_i64().map(|i| i as f64)
                            })?;
                            let price = unscale_price(price_ep, price_scale);
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let bids = parse_levels("bids");
        let asks = parse_levels("asks");

        if msg_type == "snapshot" {
            Ok(Some(StreamEvent::OrderbookSnapshot(crate::core::OrderBook {
                timestamp,
                bids,
                asks,
                sequence: msg.get("sequence").and_then(|s| s.as_i64()).map(|n| n.to_string()),
                last_update_id: None,
                first_update_id: None,
                prev_update_id: None,
                event_time: None,
                transaction_time: None,
                checksum: None,
            })))
        } else {
            // incremental
            Ok(Some(StreamEvent::OrderbookDelta(OrderbookDeltaData {
                bids,
                asks,
                timestamp,
                first_update_id: None,
                last_update_id: None,
                prev_update_id: None,
                event_time: None,
                checksum: None,
            })))
        }
    }

    /// Parse trades push message
    fn parse_trades_push(
        msg: &Value,
        symbol: &str,
        price_scale: u8,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let trades = msg
            .get("trades")
            .and_then(|t| t.as_array())
            .ok_or_else(|| WebSocketError::Parse("Missing 'trades' array".to_string()))?;

        // Parse the most recent trade (last in array for incremental, first for snapshot)
        let trade_arr = trades
            .last()
            .and_then(|t| t.as_array())
            .ok_or_else(|| WebSocketError::Parse("Invalid trade format".to_string()))?;

        if trade_arr.len() < 4 {
            return Err(WebSocketError::Parse("Trade array too short".to_string()));
        }

        // Trade fields: [timestamp_ns, side, priceEp, size]
        let timestamp_ns = trade_arr[0].as_i64().unwrap_or(0);
        let timestamp_ms = timestamp_ns / 1_000_000;

        let side_str = trade_arr[1].as_str().unwrap_or("Buy");
        let side = match side_str {
            "Sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        let price_ep = trade_arr[2].as_i64().unwrap_or(0);
        let price = unscale_price(price_ep, price_scale);

        let quantity = trade_arr[3]
            .as_f64()
            .or_else(|| trade_arr[3].as_i64().map(|i| i as f64))
            .unwrap_or(0.0);

        let sequence = msg
            .get("sequence")
            .and_then(|s| s.as_i64())
            .unwrap_or(0);

        Ok(Some(StreamEvent::Trade(crate::core::PublicTrade {
            id: sequence.to_string(),
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
            timestamp: timestamp_ms,
        })))
    }

    /// Parse kline push message
    fn parse_kline_push(
        msg: &Value,
        _symbol: &str,
        price_scale: u8,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let klines = msg
            .get("kline")
            .and_then(|k| k.as_array())
            .ok_or_else(|| WebSocketError::Parse("Missing 'kline' array".to_string()))?;

        let kline_arr = klines
            .last()
            .and_then(|k| k.as_array())
            .ok_or_else(|| WebSocketError::Parse("Invalid kline format".to_string()))?;

        if kline_arr.len() < 8 {
            return Err(WebSocketError::Parse("Kline array too short".to_string()));
        }

        // Kline fields: [timestamp, interval, lastCloseEp, highEp, lowEp, openEp, volume, turnoverEv]
        let open_time = kline_arr[0].as_i64().unwrap_or(0) * 1000; // seconds to ms
        let close_ep = kline_arr[2].as_i64().unwrap_or(0);
        let high_ep = kline_arr[3].as_i64().unwrap_or(0);
        let low_ep = kline_arr[4].as_i64().unwrap_or(0);
        let open_ep = kline_arr[5].as_i64().unwrap_or(0);
        let volume = kline_arr[6]
            .as_f64()
            .or_else(|| kline_arr[6].as_i64().map(|i| i as f64))
            .unwrap_or(0.0);

        Ok(Some(StreamEvent::Kline(crate::core::Kline {
            open_time,
            open: unscale_price(open_ep, price_scale),
            high: unscale_price(high_ep, price_scale),
            low: unscale_price(low_ep, price_scale),
            close: unscale_price(close_ep, price_scale),
            volume,
            quote_volume: None,
            close_time: None,
            trades: None,
        })))
    }

    /// Parse 24h ticker (market24h) push message
    fn parse_ticker_push(
        msg: &Value,
        symbol: &str,
        price_scale: u8,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let market = msg
            .get("market24h")
            .ok_or_else(|| WebSocketError::Parse("Missing 'market24h' field".to_string()))?;

        let last_ep = market
            .get("lastEp")
            .or_else(|| market.get("close"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let open_ep = market.get("openEp").and_then(|v| v.as_i64());
        let high_ep = market.get("highEp").and_then(|v| v.as_i64());
        let low_ep = market.get("lowEp").and_then(|v| v.as_i64());
        let bid_ep = market.get("bidEp").and_then(|v| v.as_i64());
        let ask_ep = market.get("askEp").and_then(|v| v.as_i64());
        let volume = market
            .get("volume")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)));
        let turnover_ev = market.get("turnoverEv").and_then(|v| v.as_i64());

        let timestamp = market
            .get("timestamp")
            .and_then(|t| t.as_i64())
            .map(|ns| ns / 1_000_000)
            .unwrap_or_else(|| timestamp_millis() as i64);

        let last_price = unscale_price(last_ep, price_scale);
        let open_price = open_ep.map(|p| unscale_price(p, price_scale));

        Ok(Some(StreamEvent::Ticker(crate::core::Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: bid_ep.map(|p| unscale_price(p, price_scale)),
            ask_price: ask_ep.map(|p| unscale_price(p, price_scale)),
            high_24h: high_ep.map(|p| unscale_price(p, price_scale)),
            low_24h: low_ep.map(|p| unscale_price(p, price_scale)),
            volume_24h: volume,
            quote_volume_24h: turnover_ev.map(|v| v as f64 / 100_000_000.0),
            price_change_24h: open_price.map(|o| last_price - o),
            price_change_percent_24h: open_price.map(|o| {
                if o > 0.0 {
                    ((last_price - o) / o) * 100.0
                } else {
                    0.0
                }
            }),
            timestamp,
        })))
    }

    /// Parse tick (symbol price) push message
    fn parse_tick_push(
        msg: &Value,
        symbol: &str,
        price_scale: u8,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let tick = msg
            .get("tick")
            .ok_or_else(|| WebSocketError::Parse("Missing 'tick' field".to_string()))?;

        let last_ep = tick.get("last").and_then(|v| v.as_i64()).unwrap_or(0);
        let timestamp = tick
            .get("timestamp")
            .and_then(|t| t.as_i64())
            .map(|ns| ns / 1_000_000)
            .unwrap_or_else(|| timestamp_millis() as i64);

        let last_price = unscale_price(last_ep, price_scale);

        Ok(Some(StreamEvent::Ticker(crate::core::Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })))
    }

    /// Parse AOP (Account-Order-Position) push messages
    fn parse_aop_push(msg: &Value) -> WebSocketResult<Option<StreamEvent>> {
        // Prioritize order updates, then position updates, then account updates
        if let Some(orders) = msg.get("orders").and_then(|o| o.as_array()) {
            if let Some(order_data) = orders.first() {
                return Self::parse_order_update(order_data);
            }
        }

        if let Some(positions) = msg.get("positions").and_then(|p| p.as_array()) {
            if let Some(pos_data) = positions.first() {
                return Self::parse_position_update(pos_data);
            }
        }

        if let Some(accounts) = msg.get("accounts").and_then(|a| a.as_array()) {
            if let Some(acc_data) = accounts.first() {
                return Self::parse_balance_update(acc_data);
            }
        }

        Ok(None)
    }

    /// Parse order update from AOP message
    fn parse_order_update(data: &Value) -> WebSocketResult<Option<StreamEvent>> {
        let order_id = data
            .get("orderID")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let symbol = data
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let side = match data.get("side").and_then(|v| v.as_str()).unwrap_or("Buy") {
            "Sell" => crate::core::OrderSide::Sell,
            _ => crate::core::OrderSide::Buy,
        };

        let order_type = match data
            .get("ordType")
            .or_else(|| data.get("orderType"))
            .and_then(|v| v.as_str())
            .unwrap_or("Limit")
        {
            "Market" => crate::core::OrderType::Market,
            _ => crate::core::OrderType::Limit { price: 0.0 },
        };

        let status = match data.get("ordStatus").and_then(|v| v.as_str()).unwrap_or("New") {
            "New" | "Untriggered" => crate::core::OrderStatus::New,
            "PartiallyFilled" => crate::core::OrderStatus::PartiallyFilled,
            "Filled" => crate::core::OrderStatus::Filled,
            "Canceled" | "Cancelled" => crate::core::OrderStatus::Canceled,
            "Rejected" => crate::core::OrderStatus::Rejected,
            _ => crate::core::OrderStatus::New,
        };

        let quantity = data
            .get("orderQty")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
            .unwrap_or(0.0);

        let filled_quantity = data
            .get("cumQty")
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
            .unwrap_or(0.0);

        let timestamp = data
            .get("actionTimeNs")
            .or_else(|| data.get("createTimeNs"))
            .and_then(|t| t.as_i64())
            .map(|ns| ns / 1_000_000)
            .unwrap_or_else(|| timestamp_millis() as i64);

        Ok(Some(StreamEvent::OrderUpdate(crate::core::OrderUpdateEvent {
            order_id,
            client_order_id: data.get("clOrdID").and_then(|v| v.as_str()).map(String::from),
            symbol,
            side,
            order_type,
            status,
            price: data.get("priceEp").and_then(|v| v.as_i64()).map(|p| unscale_price(p, 4)),
            quantity,
            filled_quantity,
            average_price: data.get("avgPriceEp").and_then(|v| v.as_i64()).map(|p| unscale_price(p, 4)),
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: None,
            trade_id: None,
            timestamp,
        })))
    }

    /// Parse balance update from AOP message
    fn parse_balance_update(data: &Value) -> WebSocketResult<Option<StreamEvent>> {
        let asset = data
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("BTC")
            .to_string();

        let balance_ev = data.get("accountBalanceEv").and_then(|v| v.as_i64()).unwrap_or(0);
        let used_ev = data.get("totalUsedBalanceEv").and_then(|v| v.as_i64()).unwrap_or(0);

        // Default valueScale = 8 for BTC
        let total = balance_ev as f64 / 100_000_000.0;
        let used = used_ev as f64 / 100_000_000.0;

        Ok(Some(StreamEvent::BalanceUpdate(crate::core::BalanceUpdateEvent {
            asset,
            free: total - used,
            locked: used,
            total,
            delta: None,
            reason: None,
            timestamp: timestamp_millis() as i64,
        })))
    }

    /// Parse position update from AOP message
    fn parse_position_update(data: &Value) -> WebSocketResult<Option<StreamEvent>> {
        let symbol = data
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let size = data.get("size").and_then(|v| v.as_i64()).unwrap_or(0);

        let side = match data.get("side").and_then(|v| v.as_str()).unwrap_or("Buy") {
            "Sell" => crate::core::PositionSide::Short,
            _ => crate::core::PositionSide::Long,
        };

        let entry_price_ep = data.get("avgEntryPriceEp").and_then(|v| v.as_i64()).unwrap_or(0);
        let entry_price = unscale_price(entry_price_ep, 4);

        let mark_price = data
            .get("markPriceEp")
            .and_then(|v| v.as_i64())
            .map(|p| unscale_price(p, 4));

        let unrealized_pnl_ev = data.get("unrealisedPnlEv").and_then(|v| v.as_i64()).unwrap_or(0);
        let unrealized_pnl = unrealized_pnl_ev as f64 / 100_000_000.0;

        let liq_price = data
            .get("liquidationPriceEp")
            .and_then(|v| v.as_i64())
            .map(|p| unscale_price(p, 4));

        let leverage_er = data.get("leverageEr").and_then(|v| v.as_i64()).unwrap_or(0);

        Ok(Some(StreamEvent::PositionUpdate(
            crate::core::PositionUpdateEvent {
                symbol,
                side,
                quantity: size.unsigned_abs() as f64,
                entry_price,
                mark_price,
                unrealized_pnl,
                realized_pnl: None,
                liquidation_price: liq_price,
                leverage: Some((leverage_er.unsigned_abs() as f64 / 100_000_000.0) as u32),
                margin_type: None,
                reason: None,
                timestamp: timestamp_millis() as i64,
            },
        )))
    }

    /// Start ping task - sends heartbeat every 5 seconds.
    ///
    /// Sends both a JSON `{"method":"server.ping"}` (Phemex application keepalive)
    /// and a WS-level `Message::Ping` (for RTT measurement via Pong response).
    fn start_ping_task(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        let _ = ws_ping_rtt_ms; // stored on struct; pong handler updates it
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1000)).await;

                let last = *last_ping.lock().await;

                if last.elapsed() >= Duration::from_secs(HEARTBEAT_INTERVAL_SECS) {
                    let mut stream_guard = ws_stream.lock().await;
                    if let Some(stream) = stream_guard.as_mut() {
                        let ping = json!({
                            "id": next_request_id(),
                            "method": "server.ping",
                            "params": []
                        });

                        let msg_text = ping.to_string();
                        // Send application-level JSON ping (Phemex keepalive)
                        if stream.send(Message::Text(msg_text)).await.is_ok() {
                            // Send WS-level ping for RTT measurement
                            *last_ping.lock().await = Instant::now();
                            let _ = stream.send(Message::Ping(vec![])).await;
                        }
                    }
                }
            }
        });
    }

    /// Build Phemex subscription method and params for a given request
    fn build_subscribe_message(
        request: &SubscriptionRequest,
        account_type: AccountType,
    ) -> (String, Vec<Value>) {
        match &request.stream_type {
            StreamType::Ticker => {
                // Use market24h for ticker (broadcasts all symbols)
                ("market24h.subscribe".to_string(), vec![])
            }
            StreamType::Trade => {
                let symbol = format_symbol(
                    &request.symbol.base,
                    &request.symbol.quote,
                    account_type,
                );
                ("trade.subscribe".to_string(), vec![json!(symbol)])
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                let symbol = format_symbol(
                    &request.symbol.base,
                    &request.symbol.quote,
                    account_type,
                );
                ("orderbook.subscribe".to_string(), vec![json!(symbol)])
            }
            StreamType::Kline { interval } => {
                let symbol = format_symbol(
                    &request.symbol.base,
                    &request.symbol.quote,
                    account_type,
                );
                let resolution = super::endpoints::map_kline_interval(interval);
                (
                    "kline.subscribe".to_string(),
                    vec![json!(symbol), json!(resolution)],
                )
            }
            StreamType::MarkPrice | StreamType::FundingRate => {
                // Mark price and funding rate come through market24h
                ("market24h.subscribe".to_string(), vec![])
            }
            StreamType::OrderUpdate
            | StreamType::BalanceUpdate
            | StreamType::PositionUpdate => {
                // AOP (Account-Order-Position) - requires authentication
                ("aop.subscribe".to_string(), vec![])
            }
        }
    }

    /// Build unsubscribe message
    fn build_unsubscribe_method(stream_type: &StreamType) -> String {
        match stream_type {
            StreamType::Ticker => "market24h.unsubscribe".to_string(),
            StreamType::Trade => "trade.unsubscribe".to_string(),
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                "orderbook.unsubscribe".to_string()
            }
            StreamType::Kline { .. } => "kline.unsubscribe".to_string(),
            StreamType::MarkPrice | StreamType::FundingRate => {
                "market24h.unsubscribe".to_string()
            }
            StreamType::OrderUpdate
            | StreamType::BalanceUpdate
            | StreamType::PositionUpdate => "aop.unsubscribe".to_string(),
        }
    }

    /// Check if stream type requires private channel
    fn _is_private(stream_type: &StreamType) -> bool {
        matches!(
            stream_type,
            StreamType::OrderUpdate | StreamType::BalanceUpdate | StreamType::PositionUpdate
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for PhemexWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Connect WebSocket
        let mut ws_stream = self
            .connect_ws()
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Authenticate if we have credentials (needed for private channels)
        if let Some(ref auth) = self.auth {
            Self::authenticate(auth, &mut ws_stream)
                .await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
        }

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Create broadcast channel and store sender
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Start message handler
        Self::start_message_handler(
            self.ws_stream.clone(),
            self.event_tx.clone(),
            self.status.clone(),
            self.price_scale,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start ping task (every 5 seconds)
        Self::start_ping_task(self.ws_stream.clone(), self.last_ping.clone(), self.ws_ping_rtt_ms.clone());

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_stream.lock().await = None;
        let _ = self.event_tx.lock().unwrap().take();
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
        let (method, params) = Self::build_subscribe_message(&request, self.account_type);

        let msg = json!({
            "id": next_request_id(),
            "method": method,
            "params": params
        });

        let msg_text = msg.to_string();

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard
            .as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        stream
            .send(Message::Text(msg_text))
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(stream_guard);

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let method = Self::build_unsubscribe_method(&request.stream_type);

        let msg = json!({
            "id": next_request_id(),
            "method": method,
            "params": []
        });

        let msg_text = msg.to_string();

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard
            .as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        stream
            .send(Message::Text(msg_text))
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(stream_guard);

        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let tx_guard = self.event_tx.lock().unwrap();
        if let Some(ref tx) = *tx_guard {
            let rx = tx.subscribe();
            Box::pin(
                tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
                    match result {
                        Ok(event) => Some(event),
                        Err(
                            tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_),
                        ) => Some(Err(
                            WebSocketError::ConnectionError("Event stream lagged behind".to_string()),
                        )),
                    }
                }),
            )
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

    fn orderbook_capabilities(&self) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: Some(30),
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
        }
    }
}

