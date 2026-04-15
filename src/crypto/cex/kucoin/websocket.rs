//! # KuCoin WebSocket Implementation
//!
//! WebSocket connector for KuCoin Spot and Futures.
//!
//! ## Features
//! - Public and private channels
//! - Automatic token management (24h expiry)
//! - Ping/pong heartbeat
//! - Subscription management
//! - Message parsing to StreamEvent
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = KuCoinWebSocket::new(Some(credentials), false, AccountType::Spot).await?;
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
    timestamp_millis,
};
use crate::core::types::{WebSocketResult, WebSocketError, OrderbookCapabilities};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::WeightRateLimiter;

use super::auth::KuCoinAuth;
use super::endpoints::{KuCoinUrls, KuCoinEndpoint, format_symbol};
use super::parser::KuCoinParser;

// Global rate limiter for WebSocket connections (4000 weight per 30 seconds)
// Shared across all KuCoin WebSocket instances to respect global rate limits
static WS_RATE_LIMITER: OnceLock<Arc<StdMutex<WeightRateLimiter>>> = OnceLock::new();

fn get_ws_rate_limiter() -> &'static Arc<StdMutex<WeightRateLimiter>> {
    WS_RATE_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(
            WeightRateLimiter::new(4000, Duration::from_secs(30))
        ))
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET TOKEN RESPONSE
// ═══════════════════════════════════════════════════════════════════════════════

/// Response from bullet-public/bullet-private endpoints
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct TokenResponse {
    code: String,
    data: TokenData,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct TokenData {
    token: String,
    #[serde(rename = "instanceServers")]
    instance_servers: Vec<InstanceServer>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct InstanceServer {
    endpoint: String,
    #[serde(rename = "pingInterval")]
    ping_interval: u64,
    #[serde(rename = "pingTimeout")]
    ping_timeout: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing message (subscribe/unsubscribe/ping)
#[derive(Debug, Clone, Serialize)]
struct OutgoingMessage {
    id: String,
    #[serde(rename = "type")]
    msg_type: String,
    topic: String,
    #[serde(rename = "privateChannel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    private_channel: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<bool>,
}

/// Ping message
#[derive(Debug, Clone, Serialize)]
struct PingMessage {
    id: String,
    #[serde(rename = "type")]
    msg_type: String,
}

/// Incoming message from KuCoin
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct IncomingMessage {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "pingInterval")]
    ping_interval: Option<u64>,
    #[serde(rename = "pingTimeout")]
    ping_timeout: Option<u64>,
    topic: Option<String>,
    subject: Option<String>,
    data: Option<Value>,
    id: Option<String>,
    code: Option<String>,
    message: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// KUCOIN WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// KuCoin WebSocket connector
pub struct KuCoinWebSocket {
    /// HTTP client for getting tokens
    http: HttpClient,
    /// Authentication (None for public channels only)
    auth: Option<KuCoinAuth>,
    /// URLs (mainnet/testnet)
    urls: KuCoinUrls,
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
    /// Ping interval (milliseconds)
    ping_interval: Arc<Mutex<Duration>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Token and endpoint
    token_info: Arc<Mutex<Option<(String, String)>>>,
    /// Most recent ping round-trip time in milliseconds (0 until first pong)
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl KuCoinWebSocket {
    /// Create new KuCoin WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            KuCoinUrls::TESTNET
        } else {
            KuCoinUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?;

        let mut auth = credentials
            .as_ref()
            .map(KuCoinAuth::new)
            .transpose()?;

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = urls.rest_url(AccountType::Spot);
            let url = format!("{}/api/v1/timestamp", base_url);
            if let Ok(response) = http.get(&url, &std::collections::HashMap::new()).await {
                if let Some(data) = response.get("data").and_then(|d| d.as_i64()) {
                    if let Some(ref mut a) = auth {
                        a.sync_time(data);
                    }
                }
            }
        }

        Ok(Self {
            http,
            auth,
            urls,
            account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_stream: Arc::new(Mutex::new(None)),
            ping_interval: Arc::new(Mutex::new(Duration::from_millis(18000))),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            token_info: Arc::new(Mutex::new(None)),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Get WebSocket token (public or private)
    async fn get_token(&self, private: bool) -> ExchangeResult<(String, String, Duration)> {
        let base_url = self.urls.rest_url(self.account_type);
        let endpoint = if private {
            KuCoinEndpoint::WsPrivateToken
        } else {
            KuCoinEndpoint::WsPublicToken
        };

        let url = format!("{}{}", base_url, endpoint.path());

        let response = if private {
            // Private token requires authentication
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Private channels require authentication".to_string()))?;

            // Sign with empty JSON object as body
            let body = json!({});
            let headers = auth.sign_request("POST", endpoint.path(), &body.to_string());
            self.http.post(&url, &body, &headers).await?
        } else {
            // Public token - no auth needed
            self.http.post(&url, &json!({}), &HashMap::new()).await?
        };

        // Check response code
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("500000");

        if code != "200000" {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Failed to get WebSocket token");
            return Err(ExchangeError::Api {
                code: code.parse().unwrap_or(-1),
                message: msg.to_string(),
            });
        }

        // Parse token response
        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing data field in token response".to_string()))?;

        let token = data.get("token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing token in response".to_string()))?
            .to_string();

        let servers = data.get("instanceServers")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing instanceServers".to_string()))?;

        let server = servers.first()
            .ok_or_else(|| ExchangeError::Parse("No instance servers available".to_string()))?;

        let endpoint = server.get("endpoint")
            .and_then(|e| e.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing endpoint".to_string()))?
            .to_string();

        let ping_interval = server.get("pingInterval")
            .and_then(|p| p.as_u64())
            .unwrap_or(18000);

        Ok((token, endpoint, Duration::from_millis(ping_interval)))
    }

    /// Connect to WebSocket with token
    async fn connect_ws(&self, token: &str, endpoint: &str) -> ExchangeResult<WsStream> {
        let ws_url = format!("{}/?token={}", endpoint, token);

        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Start message handling task
    fn start_message_handler(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        account_type: AccountType,
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
                        match Self::handle_message_broadcast(&text, account_type, &last_ping, &ws_ping_rtt_ms).await {
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
                    Some(Ok(Message::Close(_))) => {
                        drop(stream_guard);
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Some(Err(e)) => {
                        drop(stream_guard);
                        let tx_guard = event_tx.lock().unwrap();
                        if let Some(ref tx) = *tx_guard {
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
            // Drop the broadcast sender so all BroadcastStream receivers get None
            let _ = event_tx.lock().unwrap().take();
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Handle incoming WebSocket message (returns event for broadcast dispatch)
    async fn handle_message_broadcast(
        text: &str,
        account_type: AccountType,
        last_ping: &Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: &Arc<Mutex<u64>>,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Handle different message types
        match msg.msg_type.as_deref() {
            Some("welcome") => return Ok(None),
            Some("pong") => {
                let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                *ws_ping_rtt_ms.lock().await = rtt;
                return Ok(None);
            }
            Some("ack") => return Ok(None),
            Some("error") => {
                let error_msg = msg.message.unwrap_or_else(|| "Unknown error".to_string());
                return Err(WebSocketError::ProtocolError(error_msg));
            }
            Some("message") => {
                return Self::parse_data_message(&msg, account_type);
            }
            _ => {}
        }

        Ok(None)
    }

    /// Parse data message to StreamEvent
    fn parse_data_message(
        msg: &IncomingMessage,
        account_type: AccountType,
    ) -> WebSocketResult<Option<StreamEvent>> {
        let _topic = msg.topic.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing topic".to_string()))?;

        let subject = msg.subject.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing subject".to_string()))?;

        let data = msg.data.as_ref()
            .ok_or_else(|| WebSocketError::Parse("Missing data".to_string()))?;

        // Match by subject to determine event type
        match subject.as_str() {
            // Spot ticker (bid/ask/last only — no 24h stats)
            "trade.ticker" => {
                let ticker = Self::parse_spot_ticker(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            // Spot snapshot ticker (full 24h stats: high/low/vol/changeRate)
            "trade.snapshot" => {
                let ticker = Self::parse_spot_snapshot(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            // Futures ticker
            "tickerV2" => {
                let ticker = Self::parse_futures_ticker(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Ticker(ticker)))
            }
            // Match execution (trades)
            "trade.l3match" | "match" => {
                let trade = Self::parse_trade(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Trade(trade)))
            }
            // Level2 orderbook
            "trade.l2update" | "level2" => {
                let delta = Self::parse_orderbook_delta(data, account_type)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(delta))
            }
            // Klines/Candles
            "trade.candles.update" => {
                let kline = Self::parse_kline(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::Kline(kline)))
            }
            // Mark price
            "mark.index.price" => {
                let event = Self::parse_mark_price(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(event))
            }
            // Funding rate
            "funding.rate" => {
                let event = Self::parse_funding_rate_ws(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(event))
            }
            // Private: Order updates
            "orderChange" => {
                let event = Self::parse_order_update(data, account_type)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::OrderUpdate(event)))
            }
            // Private: Balance updates
            "account.balance" | "walletBalance.change" => {
                let event = Self::parse_balance_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::BalanceUpdate(event)))
            }
            // Private: Position updates
            "position.change" | "position.settlement" => {
                let event = Self::parse_position_update(data)
                    .map_err(|e| WebSocketError::Parse(e.to_string()))?;
                Ok(Some(StreamEvent::PositionUpdate(event)))
            }
            _ => {
                // Unknown subject - ignore
                Ok(None)
            }
        }
    }

    /// Start ping task
    fn start_ping_task(
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        ping_interval: Arc<Mutex<Duration>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1000)).await;

                let interval = *ping_interval.lock().await;
                let last = *last_ping.lock().await;

                if last.elapsed() >= interval {
                    let mut stream_guard = ws_stream.lock().await;
                    if let Some(stream) = stream_guard.as_mut() {
                        let ping = PingMessage {
                            id: timestamp_millis().to_string(),
                            msg_type: "ping".to_string(),
                        };

                        let msg_json = serde_json::to_string(&ping).expect("JSON serialization should never fail for valid struct");
                        if stream.send(Message::Text(msg_json)).await.is_ok() {
                            *last_ping.lock().await = Instant::now();
                        }
                    }
                }
            }
        });
    }

    /// Build topic string for subscription
    fn build_topic(request: &SubscriptionRequest, account_type: AccountType) -> String {
        match &request.stream_type {
            StreamType::Ticker => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                match account_type {
                    AccountType::Spot | AccountType::Margin => format!("/market/ticker:{}", symbol),
                    _ => format!("/contractMarket/tickerV2:{}", symbol),
                }
            }
            StreamType::Trade => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                match account_type {
                    AccountType::Spot | AccountType::Margin => format!("/market/match:{}", symbol),
                    _ => format!("/contractMarket/execution:{}", symbol),
                }
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                match account_type {
                    AccountType::Spot | AccountType::Margin => format!("/market/level2:{}", symbol),
                    _ => format!("/contractMarket/level2:{}", symbol),
                }
            }
            StreamType::Kline { interval } => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        format!("/market/candles:{}_{}", symbol, interval)
                    }
                    _ => {
                        // Futures doesn't have kline WebSocket channel - use REST API instead
                        format!("/market/candles:{}_{}", symbol, interval)
                    }
                }
            }
            StreamType::MarkPrice => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                format!("/contract/instrument:{}", symbol)
            }
            StreamType::FundingRate => {
                let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type);
                format!("/contract/instrument:{}", symbol)
            }
            StreamType::OrderUpdate => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => "/spotMarket/tradeOrdersV2".to_string(),
                    _ => "/contractMarket/tradeOrders".to_string(),
                }
            }
            StreamType::BalanceUpdate => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => "/account/balance".to_string(),
                    _ => "/contractAccount/wallet".to_string(),
                }
            }
            StreamType::PositionUpdate => {
                "/contract/positionAll".to_string()
            }
        }
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
    // PARSING HELPERS (stub implementations - use KuCoinParser where possible)
    // ═══════════════════════════════════════════════════════════════════════════

    fn parse_spot_ticker(data: &Value) -> ExchangeResult<crate::core::Ticker> {
        KuCoinParser::parse_ws_ticker(data)
    }

    /// Parse snapshot ticker — data is `{ sequence: "...", data: { symbol, changeRate, ... } }`
    fn parse_spot_snapshot(data: &Value) -> ExchangeResult<crate::core::Ticker> {
        // Snapshot wraps the actual stats one level deeper under "data"
        let inner = data.get("data").unwrap_or(data);
        KuCoinParser::parse_ws_snapshot_ticker(inner)
    }

    fn parse_futures_ticker(data: &Value) -> ExchangeResult<crate::core::Ticker> {
        KuCoinParser::parse_ws_ticker(data)
    }

    fn parse_trade(data: &Value) -> ExchangeResult<crate::core::PublicTrade> {
        KuCoinParser::parse_ws_trade(data)
    }

    fn parse_orderbook_delta(data: &Value, _account_type: AccountType) -> ExchangeResult<StreamEvent> {
        KuCoinParser::parse_ws_orderbook_delta(data)
    }

    fn parse_kline(data: &Value) -> ExchangeResult<crate::core::Kline> {
        KuCoinParser::parse_ws_kline(data)
    }

    fn parse_mark_price(data: &Value) -> ExchangeResult<StreamEvent> {
        KuCoinParser::parse_ws_mark_price(data)
    }

    fn parse_funding_rate_ws(data: &Value) -> ExchangeResult<StreamEvent> {
        KuCoinParser::parse_ws_funding_rate(data)
    }

    fn parse_order_update(data: &Value, _account_type: AccountType) -> ExchangeResult<crate::core::OrderUpdateEvent> {
        KuCoinParser::parse_ws_order_update(data)
    }

    fn parse_balance_update(data: &Value) -> ExchangeResult<crate::core::BalanceUpdateEvent> {
        KuCoinParser::parse_ws_balance_update(data)
    }

    fn parse_position_update(data: &Value) -> ExchangeResult<crate::core::PositionUpdateEvent> {
        KuCoinParser::parse_ws_position_update(data)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for KuCoinWebSocket {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;
        self.account_type = account_type;

        // Determine if we need private token
        let needs_private = self.auth.is_some();

        // Get token
        let (token, endpoint, ping_interval) = self.get_token(needs_private).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Store token info
        *self.token_info.lock().await = Some((token.clone(), endpoint.clone()));
        *self.ping_interval.lock().await = ping_interval;

        // Connect WebSocket
        let ws_stream = self.connect_ws(&token, &endpoint).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

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
            account_type,
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start ping task
        Self::start_ping_task(
            self.ws_stream.clone(),
            self.ping_interval.clone(),
            self.last_ping.clone(),
        );

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
        // Use try_lock to avoid blocking
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Wait for rate limit (weight 1 for subscriptions)
        Self::ws_rate_limit_wait(1).await;

        let topic = Self::build_topic(&request, self.account_type);
        let is_private = Self::is_private(&request.stream_type);

        let msg = OutgoingMessage {
            id: timestamp_millis().to_string(),
            msg_type: "subscribe".to_string(),
            topic,
            private_channel: if is_private { Some(true) } else { None },
            response: Some(true),
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        {
            let mut stream_guard = self.ws_stream.lock().await;
            let stream = stream_guard.as_mut()
                .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;
            stream.send(Message::Text(msg_json)).await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
        }

        // For Spot ticker, also subscribe to snapshot channel for 24h stats.
        // /market/snapshot:{symbol} sends changeRate, changePrice, high, low, vol, volValue.
        if matches!(request.stream_type, StreamType::Ticker)
            && matches!(self.account_type, AccountType::Spot | AccountType::Margin)
        {
            Self::ws_rate_limit_wait(1).await;
            let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, self.account_type);
            let snapshot_topic = format!("/market/snapshot:{}", symbol);
            let snapshot_msg = OutgoingMessage {
                id: timestamp_millis().to_string(),
                msg_type: "subscribe".to_string(),
                topic: snapshot_topic,
                private_channel: None,
                response: Some(true),
            };
            let snapshot_json = serde_json::to_string(&snapshot_msg)
                .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;
            let mut stream_guard = self.ws_stream.lock().await;
            let stream = stream_guard.as_mut()
                .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;
            stream.send(Message::Text(snapshot_json)).await
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
        }

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Wait for rate limit (weight 1 for unsubscriptions)
        Self::ws_rate_limit_wait(1).await;

        let topic = Self::build_topic(&request, self.account_type);
        let is_private = Self::is_private(&request.stream_type);

        let msg = OutgoingMessage {
            id: timestamp_millis().to_string(),
            msg_type: "unsubscribe".to_string(),
            topic,
            private_channel: if is_private { Some(true) } else { None },
            response: Some(true),
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut stream_guard = self.ws_stream.lock().await;
        let stream = stream_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        stream.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(stream_guard);

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
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: Some(100),
            supports_snapshot: false,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
        }
    }
}
