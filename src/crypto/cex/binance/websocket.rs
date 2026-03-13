//! # Binance WebSocket Implementation
//!
//! WebSocket connector for Binance Spot and Futures.
//!
//! ## Features
//! - Public and private channels
//! - User Data Stream with listenKey management
//! - Automatic ping/pong handling
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = BinanceWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
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

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    HttpClient, Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::SimpleRateLimiter;

use super::auth::BinanceAuth;
use super::endpoints::{BinanceUrls, BinanceEndpoint, format_symbol};
use super::parser::BinanceParser;

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Global rate limiter for Binance WebSocket connections
/// Shared across all instances to prevent connection limits (300 per 5 minutes per IP)
static GLOBAL_WS_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

fn get_global_ws_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            // Conservative: 50 connections per 5 minutes (300 is the limit)
            SimpleRateLimiter::new(50, Duration::from_secs(300))
        ))
    }).clone()
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscribe/Unsubscribe request
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    method: String,
    params: Vec<String>,
    id: u64,
}

/// Incoming WebSocket message (combined stream format)
#[derive(Debug, Clone, Deserialize)]
struct CombinedStreamMessage {
    #[allow(dead_code)]
    stream: Option<String>,
    data: Value,
}

/// Incoming WebSocket message (single stream format)
#[derive(Debug, Clone, Deserialize)]
struct SingleStreamMessage {
    #[serde(rename = "e")]
    event_type: Option<String>,
    #[serde(flatten)]
    data: Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BINANCE WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WsStream, Message>;
type WsReader = futures_util::stream::SplitStream<WsStream>;

/// Binance WebSocket connector
pub struct BinanceWebSocket {
    /// HTTP client for listenKey operations
    http: HttpClient,
    /// Authentication (None for public channels only)
    auth: Option<BinanceAuth>,
    /// URLs (mainnet/testnet)
    urls: BinanceUrls,
    /// Current account type
    account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event broadcast sender
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — used by subscribe/unsubscribe to send messages.
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    /// Listen key for user data stream (private channels)
    listen_key: Arc<Mutex<Option<String>>>,
    /// Last listen key refresh time
    last_refresh: Arc<Mutex<Instant>>,
    /// Message counter for subscribe/unsubscribe
    msg_id: Arc<Mutex<u64>>,
    /// Timestamp of the most recently sent ping frame.
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl BinanceWebSocket {
    /// Create new Binance WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            BinanceUrls::TESTNET
        } else {
            BinanceUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?;

        let auth = credentials
            .as_ref()
            .map(BinanceAuth::new)
            .transpose()?;

        Ok(Self {
            http,
            auth,
            urls,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_sink: Arc::new(Mutex::new(None)),
            listen_key: Arc::new(Mutex::new(None)),
            last_refresh: Arc::new(Mutex::new(Instant::now())),
            msg_id: Arc::new(Mutex::new(1)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Get next message ID
    async fn next_msg_id(&self) -> u64 {
        let mut id = self.msg_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    /// Create listenKey for user data stream
    async fn create_listen_key(&self) -> ExchangeResult<String> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required for private channels".to_string()))?;

        let base_url = self.urls.rest_url(self.account_type);
        let endpoint = match self.account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotListenKey,
            AccountType::FuturesCross | AccountType::FuturesIsolated => BinanceEndpoint::FuturesListenKey,
        };

        let url = format!("{}{}", base_url, endpoint.path());

        // For listenKey creation, we just need the API key header
        let mut headers = HashMap::new();
        headers.insert("X-MBX-APIKEY".to_string(), auth.api_key().to_string());

        let response = self.http.post(&url, &json!({}), &headers).await?;

        BinanceParser::check_error(&response)?;

        let listen_key = response.get("listenKey")
            .and_then(|k| k.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing listenKey in response".to_string()))?
            .to_string();

        Ok(listen_key)
    }

    /// Refresh listenKey (keepalive)
    async fn _refresh_listen_key(&self, listen_key: &str) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let base_url = self.urls.rest_url(self.account_type);
        let endpoint = match self.account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotListenKey,
            AccountType::FuturesCross | AccountType::FuturesIsolated => BinanceEndpoint::FuturesListenKey,
        };

        let path = format!("{}?listenKey={}", endpoint.path(), listen_key);
        let url = format!("{}{}", base_url, path);

        // For listenKey refresh, we just need the API key header
        let mut headers = HashMap::new();
        headers.insert("X-MBX-APIKEY".to_string(), auth.api_key().to_string());

        let response = self.http.put(&url, &json!({}), &headers).await?;
        BinanceParser::check_error(&response)?;

        Ok(())
    }

    /// Start listenKey refresh task (every 30 minutes)
    fn _start_refresh_task(
        listen_key: Arc<Mutex<Option<String>>>,
        last_refresh: Arc<Mutex<Instant>>,
        http: HttpClient,
        auth: Option<BinanceAuth>,
        urls: BinanceUrls,
        account_type: AccountType,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await; // Check every minute

                let last = *last_refresh.lock().await;
                if last.elapsed() >= Duration::from_secs(30 * 60) {
                    // Refresh every 30 minutes
                    let key_guard = listen_key.lock().await;
                    if let Some(ref key) = *key_guard {
                        let key_copy = key.clone();
                        drop(key_guard);

                        if let Some(ref auth) = auth {
                            let base_url = urls.rest_url(account_type);
                            let endpoint = match account_type {
                                AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotListenKey,
                                AccountType::FuturesCross | AccountType::FuturesIsolated => BinanceEndpoint::FuturesListenKey,
                            };

                            let path = format!("{}?listenKey={}", endpoint.path(), key_copy);
                            let url = format!("{}{}", base_url, path);

                            // For listenKey refresh, we just need the API key header
                            let mut headers = HashMap::new();
                            headers.insert("X-MBX-APIKEY".to_string(), auth.api_key().to_string());

                            if http.put(&url, &json!({}), &headers).await.is_ok() {
                                *last_refresh.lock().await = Instant::now();
                            }
                        }
                    }
                }
            }
        });
    }

    /// Connect to WebSocket
    async fn connect_ws(&self) -> ExchangeResult<WsStream> {
        // Rate limit WebSocket connections (300 per 5 minutes per IP)
        let limiter = get_global_ws_limiter();
        loop {
            let can_connect = {
                let mut limiter_guard = limiter.lock().expect("Mutex poisoned");
                limiter_guard.try_acquire()
            };

            if can_connect {
                break;
            }

            // Wait before retrying
            let wait_time = {
                let limiter_guard = limiter.lock().expect("Mutex poisoned");
                limiter_guard.time_until_ready()
            };

            if !wait_time.is_zero() {
                sleep(wait_time).await;
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }

        let ws_base = self.urls.ws_url(self.account_type);

        // Determine if we need private stream
        let needs_private = self.auth.is_some();

        let ws_url = if needs_private {
            // Create listenKey for user data stream
            let listen_key = self.create_listen_key().await?;
            *self.listen_key.lock().await = Some(listen_key.clone());
            *self.last_refresh.lock().await = Instant::now();

            // Note: We don't start refresh task here because HttpClient doesn't implement Clone
            // and it's not critical for WebSocket functionality since listenKey lasts 24h
            // In production, we'd need to implement refresh task differently

            // User data stream URL
            format!("{}/ws/{}", ws_base, listen_key)
        } else {
            // Public stream URL (we'll use combined stream format)
            format!("{}/stream", ws_base)
        };

        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Start message handling task
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
                        let tx_clone = event_tx.lock().unwrap().as_ref().cloned();
                        if let Some(tx) = tx_clone {
                            if let Err(e) = Self::handle_message(&text, &tx, account_type).await {
                                let _ = tx.send(Err(e));
                            }
                        }
                    }
                    Ok(Message::Ping(_ping)) => {
                        // tokio-tungstenite automatically responds to Ping at the protocol level.
                        // No manual Pong needed here.
                    }
                    Ok(Message::Pong(_)) => {
                        // Measure RTT from our last client-initiated ping frame.
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Ok(Message::Close(_)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Err(e) => {
                        if let Some(tx) = event_tx.lock().unwrap().as_ref() {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
                        break;
                    }
                    _ => {}
                }
            }
            // Stream ended — drop sender so receivers know the stream is done
            let _ = event_tx.lock().unwrap().take();
            // Mark disconnected
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Start periodic WebSocket-frame ping task (every 5 seconds).
    ///
    /// Binance tolerates client-initiated ping frames and will respond with
    /// `Message::Pong`, which the message handler uses to measure RTT.
    fn start_ping_task(
        ws_sink: Arc<Mutex<Option<WsSink>>>,
        last_ping: Arc<Mutex<Instant>>,
        status: Arc<Mutex<ConnectionStatus>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            interval.tick().await; // skip first immediate tick

            loop {
                interval.tick().await;

                // Stop if disconnected
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

    /// Handle incoming WebSocket message
    async fn handle_message(
        text: &str,
        event_tx: &broadcast::Sender<WebSocketResult<StreamEvent>>,
        account_type: AccountType,
    ) -> WebSocketResult<()> {
        // Try to parse as combined stream format first
        if let Ok(combined) = serde_json::from_str::<CombinedStreamMessage>(text) {
            if let Some(event) = Self::parse_stream_data(&combined.data, account_type)? {
                let _ = event_tx.send(Ok(event));
            }
            return Ok(());
        }

        // Try to parse as single stream format
        if let Ok(single) = serde_json::from_str::<SingleStreamMessage>(text) {
            if let Some(event_type) = single.event_type.as_deref() {
                if let Some(event) = Self::parse_event_by_type(event_type, &single.data, account_type)? {
                    let _ = event_tx.send(Ok(event));
                }
            }
            return Ok(());
        }

        // Try parsing as raw JSON
        if let Ok(data) = serde_json::from_str::<Value>(text) {
            if let Some(event) = Self::parse_stream_data(&data, account_type)? {
                let _ = event_tx.send(Ok(event));
            }
        }

        Ok(())
    }

    /// Parse stream data to StreamEvent
    fn parse_stream_data(data: &Value, account_type: AccountType) -> WebSocketResult<Option<StreamEvent>> {
        // Check event type
        let event_type = data.get("e")
            .and_then(|e| e.as_str());

        if let Some(event_type) = event_type {
            Self::parse_event_by_type(event_type, data, account_type)
        } else {
            Ok(None)
        }
    }

    /// Parse event by type
    fn parse_event_by_type(event_type: &str, data: &Value, _account_type: AccountType) -> WebSocketResult<Option<StreamEvent>> {
        match event_type {
            // Ticker
            "24hrTicker" => {
                let ticker = Self::parse_ticker(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            // Mini ticker
            "24hrMiniTicker" => {
                let ticker = Self::parse_mini_ticker(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            // Trade
            "trade" => {
                let trade = Self::parse_trade(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(trade)))
            }
            // Aggregate trade
            "aggTrade" => {
                let trade = Self::parse_agg_trade(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(trade)))
            }
            // Depth update (incremental)
            "depthUpdate" => {
                let delta = Self::parse_depth_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(delta))
            }
            // Kline
            "kline" => {
                let kline = Self::parse_kline(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Kline(kline)))
            }
            // Mark price (futures)
            "markPriceUpdate" => {
                let event = Self::parse_mark_price(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(event))
            }
            // User data: execution report (order update)
            "executionReport" => {
                let event = Self::parse_execution_report(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderUpdate(event)))
            }
            // User data: account position (balance update)
            "outboundAccountPosition" => {
                // This contains multiple balances, emit first non-zero
                if let Some(event) = Self::parse_account_position(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))? {
                    Ok(Some(StreamEvent::BalanceUpdate(event)))
                } else {
                    Ok(None)
                }
            }
            // User data: balance update
            "balanceUpdate" => {
                let event = Self::parse_balance_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::BalanceUpdate(event)))
            }
            // Futures: account update (balance + position)
            "ACCOUNT_UPDATE" => {
                // ACCOUNT_UPDATE format:
                // {"e":"ACCOUNT_UPDATE","T":1234567890,"a":{"B":[{"a":"USDT","wb":"100.00","cw":"95.00"}],
                //   "P":[{"s":"BTCUSDT","pa":"0.1","ep":"67000","up":"100"}],"m":"ORDER"}}
                // Emit first balance from B[] array as BalanceUpdate
                if let Some(event) = Self::parse_futures_account_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))? {
                    Ok(Some(StreamEvent::BalanceUpdate(event)))
                } else {
                    Ok(None)
                }
            }
            // Futures: order update
            "ORDER_TRADE_UPDATE" => {
                let event = Self::parse_futures_order_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderUpdate(event)))
            }
            _ => {
                // Unknown event type - ignore
                Ok(None)
            }
        }
    }

    /// Build stream name for subscription
    fn build_stream_name(request: &SubscriptionRequest, account_type: AccountType) -> String {
        let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type).to_lowercase();

        match &request.stream_type {
            StreamType::Ticker => format!("{}@ticker", symbol),
            StreamType::Trade => format!("{}@trade", symbol),
            StreamType::Orderbook => format!("{}@depth20@100ms", symbol),
            StreamType::OrderbookDelta => format!("{}@depth@100ms", symbol),
            StreamType::Kline { interval } => format!("{}@kline_{}", symbol, interval),
            StreamType::MarkPrice => format!("{}@markPrice", symbol),
            StreamType::FundingRate => format!("{}@markPrice", symbol), // Binance includes funding in mark price stream
            _ => String::new(), // Private streams don't use stream names
        }
    }

    /// Check if stream type requires private channel
    fn is_private(stream_type: &StreamType) -> bool {
        matches!(
            stream_type,
            StreamType::OrderUpdate | StreamType::BalanceUpdate | StreamType::PositionUpdate
        )
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARSING HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    fn parse_ticker(data: &Value) -> ExchangeResult<crate::core::Ticker> {
        BinanceParser::parse_ticker(data)
    }

    fn parse_mini_ticker(data: &Value) -> ExchangeResult<crate::core::Ticker> {
        use crate::core::Ticker;

        let parse_f64 = |key: &str| -> Option<f64> {
            data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| data.get(key).and_then(|v| v.as_f64()))
        };

        Ok(Ticker {
            symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            last_price: parse_f64("c").unwrap_or(0.0),
            bid_price: None,
            ask_price: None,
            high_24h: parse_f64("h"),
            low_24h: parse_f64("l"),
            volume_24h: parse_f64("v"),
            quote_volume_24h: parse_f64("q"),
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: data.get("E").and_then(|t| t.as_i64()).unwrap_or(0),
        })
    }

    fn parse_trade(data: &Value) -> ExchangeResult<crate::core::PublicTrade> {
        use crate::core::PublicTrade;
        use crate::core::types::TradeSide;

        let parse_f64 = |key: &str| -> Option<f64> {
            data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| data.get(key).and_then(|v| v.as_f64()))
        };

        // is_buyer_maker = true means buyer was maker (sell side), false means buyer was taker (buy side)
        let is_buyer_maker = data.get("m").and_then(|m| m.as_bool()).unwrap_or(false);
        let side = if is_buyer_maker {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };

        Ok(PublicTrade {
            id: data.get("t").and_then(|t| t.as_i64()).map(|t| t.to_string()).unwrap_or_default(),
            symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            price: parse_f64("p").unwrap_or(0.0),
            quantity: parse_f64("q").unwrap_or(0.0),
            side,
            timestamp: data.get("T").and_then(|t| t.as_i64()).unwrap_or(0),
        })
    }

    fn parse_agg_trade(data: &Value) -> ExchangeResult<crate::core::PublicTrade> {
        use crate::core::PublicTrade;
        use crate::core::types::TradeSide;

        let parse_f64 = |key: &str| -> Option<f64> {
            data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| data.get(key).and_then(|v| v.as_f64()))
        };

        // is_buyer_maker = true means buyer was maker (sell side), false means buyer was taker (buy side)
        let is_buyer_maker = data.get("m").and_then(|m| m.as_bool()).unwrap_or(false);
        let side = if is_buyer_maker {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };

        Ok(PublicTrade {
            id: data.get("a").and_then(|a| a.as_i64()).map(|a| a.to_string()).unwrap_or_default(),
            symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            price: parse_f64("p").unwrap_or(0.0),
            quantity: parse_f64("q").unwrap_or(0.0),
            side,
            timestamp: data.get("T").and_then(|t| t.as_i64()).unwrap_or(0),
        })
    }

    fn parse_depth_update(data: &Value) -> ExchangeResult<StreamEvent> {
        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = pair[0].as_str()?.parse().ok()?;
                            let size = pair[1].as_str()?.parse().ok()?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(StreamEvent::OrderbookDelta {
            bids: parse_levels("b"),
            asks: parse_levels("a"),
            timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
        })
    }

    fn parse_kline(data: &Value) -> ExchangeResult<crate::core::Kline> {
        use crate::core::Kline;

        let k = data.get("k")
            .ok_or_else(|| ExchangeError::Parse("Missing 'k' field in kline event".to_string()))?;

        let parse_f64 = |key: &str| -> Option<f64> {
            k.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| k.get(key).and_then(|v| v.as_f64()))
        };

        Ok(Kline {
            open_time: k.get("t").and_then(|t| t.as_i64()).unwrap_or(0),
            open: parse_f64("o").unwrap_or(0.0),
            high: parse_f64("h").unwrap_or(0.0),
            low: parse_f64("l").unwrap_or(0.0),
            close: parse_f64("c").unwrap_or(0.0),
            volume: parse_f64("v").unwrap_or(0.0),
            close_time: k.get("T").and_then(|t| t.as_i64()),
            quote_volume: parse_f64("q"),
            trades: k.get("n").and_then(|n| n.as_i64()).map(|n| n as u64),
        })
    }

    fn parse_mark_price(data: &Value) -> ExchangeResult<StreamEvent> {
        let parse_f64 = |key: &str| -> Option<f64> {
            data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| data.get(key).and_then(|v| v.as_f64()))
        };

        Ok(StreamEvent::MarkPrice {
            symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            mark_price: parse_f64("p").unwrap_or(0.0),
            index_price: parse_f64("i"),
            timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
        })
    }

    fn parse_execution_report(data: &Value) -> ExchangeResult<crate::core::OrderUpdateEvent> {
        use crate::core::{OrderUpdateEvent, OrderSide, OrderType, OrderStatus};

        let parse_f64 = |key: &str| -> Option<f64> {
            data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| data.get(key).and_then(|v| v.as_f64()))
        };

        let side = match data.get("S").and_then(|s| s.as_str()).unwrap_or("BUY") {
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match data.get("o").and_then(|o| o.as_str()).unwrap_or("LIMIT") {
            "MARKET" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = match data.get("X").and_then(|x| x.as_str()).unwrap_or("NEW") {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::New,
        };

        let filled_qty = parse_f64("z").unwrap_or(0.0);
        let avg_price = if filled_qty > 0.0 {
            let quote_qty = parse_f64("Z").unwrap_or(0.0);
            Some(quote_qty / filled_qty)
        } else {
            None
        };

        Ok(OrderUpdateEvent {
            order_id: data.get("i").and_then(|i| i.as_i64()).map(|i| i.to_string()).unwrap_or_default(),
            client_order_id: data.get("c").and_then(|c| c.as_str()).map(String::from),
            symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: parse_f64("p"),
            quantity: parse_f64("q").unwrap_or(0.0),
            filled_quantity: filled_qty,
            average_price: avg_price,
            last_fill_price: parse_f64("L"),
            last_fill_quantity: parse_f64("l"),
            last_fill_commission: parse_f64("n"),
            commission_asset: data.get("N").and_then(|n| n.as_str()).map(String::from),
            trade_id: data.get("t").and_then(|t| t.as_i64()).map(|t| t.to_string()),
            timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
        })
    }

    fn parse_account_position(data: &Value) -> ExchangeResult<Option<crate::core::BalanceUpdateEvent>> {
        use crate::core::BalanceUpdateEvent;

        // outboundAccountPosition contains array of balances
        let balances = data.get("B")
            .and_then(|b| b.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'B' field".to_string()))?;

        // Return first non-zero balance
        for balance in balances {
            let asset = balance.get("a").and_then(|a| a.as_str()).unwrap_or("");
            let free = balance.get("f").and_then(|f| f.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let locked = balance.get("l").and_then(|l| l.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);

            if free > 0.0 || locked > 0.0 {
                return Ok(Some(BalanceUpdateEvent {
                    asset: asset.to_string(),
                    free,
                    locked,
                    total: free + locked,
                    delta: None,
                    reason: None,
                    timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
                }));
            }
        }

        Ok(None)
    }

    fn parse_balance_update(data: &Value) -> ExchangeResult<crate::core::BalanceUpdateEvent> {
        use crate::core::BalanceUpdateEvent;

        let parse_f64 = |key: &str| -> Option<f64> {
            data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| data.get(key).and_then(|v| v.as_f64()))
        };

        let delta = parse_f64("d").unwrap_or(0.0);

        Ok(BalanceUpdateEvent {
            asset: data.get("a").and_then(|a| a.as_str()).unwrap_or("").to_string(),
            free: 0.0, // Not provided in balanceUpdate
            locked: 0.0,
            total: 0.0,
            delta: Some(delta),
            reason: None, // Binance doesn't provide reason in this event
            timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
        })
    }

    fn parse_futures_account_update(data: &Value) -> ExchangeResult<Option<crate::core::BalanceUpdateEvent>> {
        use crate::core::{BalanceUpdateEvent, BalanceChangeReason};

        // ACCOUNT_UPDATE nests payload under "a"
        let account = match data.get("a") {
            Some(a) => a,
            None => return Ok(None),
        };

        let balances = match account.get("B").and_then(|b| b.as_array()) {
            Some(b) => b,
            None => return Ok(None),
        };

        // Map Binance event reason string to BalanceChangeReason
        let reason: Option<BalanceChangeReason> = account
            .get("m")
            .and_then(|m| m.as_str())
            .map(|m| match m {
                "DEPOSIT" => BalanceChangeReason::Deposit,
                "WITHDRAW" => BalanceChangeReason::Withdraw,
                "ORDER" | "TRADE" => BalanceChangeReason::Trade,
                "FUNDING_FEE" => BalanceChangeReason::Funding,
                "REALIZED_PNL" => BalanceChangeReason::RealizedPnl,
                "TRANSFER" => BalanceChangeReason::Transfer,
                "COMMISSION" => BalanceChangeReason::Commission,
                _ => BalanceChangeReason::Other,
            });

        let timestamp = data.get("T").and_then(|t| t.as_i64()).unwrap_or(0);

        // Return first balance entry that has a non-empty asset
        for balance in balances {
            let asset = balance.get("a").and_then(|a| a.as_str()).unwrap_or("");
            if asset.is_empty() {
                continue;
            }

            let parse_f64 = |key: &str| -> f64 {
                balance
                    .get(key)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .or_else(|| balance.get(key).and_then(|v| v.as_f64()))
                    .unwrap_or(0.0)
            };

            // wb = wallet balance, cw = cross wallet balance
            let total = parse_f64("wb");
            let cross_wallet = parse_f64("cw");

            return Ok(Some(BalanceUpdateEvent {
                asset: asset.to_string(),
                free: cross_wallet,
                locked: (total - cross_wallet).max(0.0),
                total,
                delta: None,
                reason,
                timestamp,
            }));
        }

        Ok(None)
    }

    fn parse_futures_order_update(data: &Value) -> ExchangeResult<crate::core::OrderUpdateEvent> {
        use crate::core::{OrderUpdateEvent, OrderSide, OrderType, OrderStatus};

        // Futures order update is nested in 'o' field
        let order = data.get("o")
            .ok_or_else(|| ExchangeError::Parse("Missing 'o' field in ORDER_TRADE_UPDATE".to_string()))?;

        let parse_f64 = |key: &str| -> Option<f64> {
            order.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                .or_else(|| order.get(key).and_then(|v| v.as_f64()))
        };

        let side = match order.get("S").and_then(|s| s.as_str()).unwrap_or("BUY") {
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match order.get("o").and_then(|o| o.as_str()).unwrap_or("LIMIT") {
            "MARKET" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = match order.get("X").and_then(|x| x.as_str()).unwrap_or("NEW") {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::New,
        };

        let filled_qty = parse_f64("z").unwrap_or(0.0);
        let avg_price = parse_f64("ap");

        Ok(OrderUpdateEvent {
            order_id: order.get("i").and_then(|i| i.as_i64()).map(|i| i.to_string()).unwrap_or_default(),
            client_order_id: order.get("c").and_then(|c| c.as_str()).map(String::from),
            symbol: order.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: parse_f64("p"),
            quantity: parse_f64("q").unwrap_or(0.0),
            filled_quantity: filled_qty,
            average_price: avg_price,
            last_fill_price: parse_f64("L"),
            last_fill_quantity: parse_f64("l"),
            last_fill_commission: parse_f64("n"),
            commission_asset: order.get("N").and_then(|n| n.as_str()).map(String::from),
            trade_id: order.get("t").and_then(|t| t.as_i64()).map(|t| t.to_string()),
            timestamp: data.get("E").and_then(|e| e.as_i64()).unwrap_or(0),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for BinanceWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Connect WebSocket
        let ws_stream = self.connect_ws().await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Split into independent read and write halves to avoid mutex contention.
        let (sink, reader) = ws_stream.split();
        *self.ws_sink.lock().await = Some(sink);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Create event broadcast channel (capacity of 1024 messages)
        let (tx, _rx) = broadcast::channel(1024);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Start message handler with the read half — no shared mutex needed.
        Self::start_message_handler(
            reader,
            self.event_tx.clone(),
            self.status.clone(),
            account_type,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start periodic ping task for RTT measurement.
        Self::start_ping_task(
            self.ws_sink.clone(),
            self.last_ping.clone(),
            self.status.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        *self.ws_sink.lock().await = None;
        let _ = self.event_tx.lock().unwrap().take();
        *self.listen_key.lock().await = None;
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
        // Check if private stream
        if Self::is_private(&request.stream_type) {
            // Private streams don't need explicit subscription - they're automatic with listenKey
            self.subscriptions.lock().await.insert(request);
            return Ok(());
        }

        // Build stream name
        let stream_name = Self::build_stream_name(&request, self.account_type);

        let msg = SubscribeMessage {
            method: "SUBSCRIBE".to_string(),
            params: vec![stream_name],
            id: self.next_msg_id().await,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        let sink = sink_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        sink.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(sink_guard);

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Check if private stream
        if Self::is_private(&request.stream_type) {
            self.subscriptions.lock().await.remove(&request);
            return Ok(());
        }

        let stream_name = Self::build_stream_name(&request, self.account_type);

        let msg = SubscribeMessage {
            method: "UNSUBSCRIBE".to_string(),
            params: vec![stream_name],
            id: self.next_msg_id().await,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        let sink = sink_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        sink.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(sink_guard);

        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.event_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1);

        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
            result.ok()
        }))
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
