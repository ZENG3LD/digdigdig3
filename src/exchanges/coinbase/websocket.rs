//! # Coinbase WebSocket Implementation
//!
//! WebSocket connector for Coinbase Advanced Trade API.
//!
//! ## Features
//! - Public and private channels
//! - No explicit ping/pong (handled by server)
//! - 5-second subscription deadline
//! - Sequence number tracking for orderbook sync
//!
//! ## Architecture
//! Uses split read/write halves to avoid mutex deadlock between the
//! message handler (reading) and subscribe (writing) operations.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;

use super::auth::CoinbaseAuth;
use super::endpoints::{CoinbaseUrls, format_symbol};
use super::parser::CoinbaseParser;

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsWriter = SplitSink<WsStream, Message>;
type WsReader = SplitStream<WsStream>;

// ===============================================================================
// WEBSOCKET MESSAGES
// ===============================================================================

/// Subscribe message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    product_ids: Vec<String>,
    channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    jwt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    granularity: Option<String>,
}

/// Coinbase WebSocket connector
pub struct CoinbaseWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<CoinbaseAuth>,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender (for multiple consumers)
    broadcast_tx: Arc<broadcast::Sender<WebSocketResult<StreamEvent>>>,
    /// WebSocket write half (for sending subscriptions)
    ws_writer: Arc<Mutex<Option<WsWriter>>>,
    /// Whether to use private endpoint
    use_private: bool,
    /// Last time a WS-level ping was sent (for RTT measurement)
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl CoinbaseWebSocket {
    /// Create new Coinbase WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
    ) -> ExchangeResult<Self> {
        let auth = if let Some(creds) = credentials {
            Some(CoinbaseAuth::new(&creds)
                .map_err(ExchangeError::Auth)?)
        } else {
            None
        };

        let use_private = auth.is_some();

        // Create broadcast channel (capacity of 1000 events)
        let (broadcast_tx, _) = broadcast::channel(1000);

        Ok(Self {
            auth,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(broadcast_tx),
            ws_writer: Arc::new(Mutex::new(None)),
            use_private,
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Connect to WebSocket, returns split read/write halves
    async fn connect_ws(&self) -> ExchangeResult<(WsReader, WsWriter)> {
        let ws_url = CoinbaseUrls::ws_url(self.use_private);

        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        let (write, read) = ws_stream.split();
        Ok((read, write))
    }

    /// Send subscribe message via the write half
    async fn send_subscribe_msg(
        writer: &mut WsWriter,
        auth: &Option<CoinbaseAuth>,
        use_private: bool,
        channel: &str,
        product_ids: Vec<String>,
        granularity: Option<&str>,
    ) -> ExchangeResult<()> {
        // Generate JWT if auth is available and using private endpoint
        let jwt = if let Some(auth) = auth {
            if use_private {
                let ws_host = "advanced-trade-ws-user.coinbase.com";
                Some(auth.build_websocket_jwt(ws_host)
                    .map_err(ExchangeError::Auth)?)
            } else {
                None
            }
        } else {
            None
        };

        let subscribe_msg = SubscribeMessage {
            msg_type: "subscribe".to_string(),
            product_ids,
            channel: channel.to_string(),
            jwt,
            granularity: granularity.map(|s| s.to_string()),
        };

        let msg_json = serde_json::to_string(&subscribe_msg)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize subscribe message: {}", e)))?;

        writer.send(Message::Text(msg_json)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send subscribe message: {}", e)))?;

        Ok(())
    }

    /// Spawn message handler for the read half (runs in background)
    fn start_message_handler(
        mut ws_read: WsReader,
        broadcast_tx: Arc<broadcast::Sender<WebSocketResult<StreamEvent>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg_result) = ws_read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if let Some(channel) = json.get("channel").and_then(|c| c.as_str()) {
                                let event = match channel {
                                    "ticker" | "ticker_batch" => {
                                        CoinbaseParser::parse_ws_ticker(&json)
                                            .ok()
                                            .map(StreamEvent::Ticker)
                                    },
                                    "level2" => {
                                        CoinbaseParser::parse_ws_orderbook(&json)
                                            .ok()
                                            .map(StreamEvent::OrderbookSnapshot)
                                    },
                                    "market_trades" => {
                                        CoinbaseParser::parse_ws_trades(&json)
                                            .ok()
                                            .map(StreamEvent::Trade)
                                    },
                                    "candles" => {
                                        CoinbaseParser::parse_ws_candles(&json)
                                            .ok()
                                            .map(StreamEvent::Kline)
                                    },
                                    _ => None,
                                };
                                if let Some(event) = event {
                                    let _ = broadcast_tx.send(Ok(event));
                                }
                            }
                        }
                    },
                    Ok(Message::Pong(_)) => {
                        // Record RTT for the WS-level ping sent by start_ping_task
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    },
                    Ok(Message::Close(_)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    },
                    Err(e) => {
                        let _ = broadcast_tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    },
                    _ => {}
                }
            }

            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Start ping task — sends WS-level pings every 5 seconds for RTT measurement.
    ///
    /// Coinbase handles keepalive server-side, but we still send WS pings so the
    /// `ping_rtt_handle()` value is populated.
    fn start_ping_task(
        ws_writer: Arc<Mutex<Option<WsWriter>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            // Skip the immediate first tick
            interval.tick().await;

            loop {
                interval.tick().await;

                let mut writer_guard = ws_writer.lock().await;
                if let Some(ref mut writer) = *writer_guard {
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
}

#[async_trait]
impl WebSocketConnector for CoinbaseWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        let (ws_read, ws_write) = self.connect_ws().await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        // Store write half for subscribe/unsubscribe
        *self.ws_writer.lock().await = Some(ws_write);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Spawn message handler with the read half (no mutex contention)
        Self::start_message_handler(
            ws_read,
            self.broadcast_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start ping task for RTT measurement (Coinbase handles keepalive server-side)
        Self::start_ping_task(
            self.ws_writer.clone(),
            self.last_ping.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            let _ = writer.close().await;
        }
        *self.status.lock().await = ConnectionStatus::Disconnected;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(
        &mut self,
        request: SubscriptionRequest,
    ) -> WebSocketResult<()> {
        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        // Determine account type from the request
        let account_type = if request.symbol.quote == "PERP" || request.symbol.base.ends_with("-PERP") {
            AccountType::FuturesCross
        } else {
            AccountType::Spot
        };

        let product_id = format_symbol(&request.symbol, account_type);

        let channel = match request.stream_type {
            StreamType::Ticker => "ticker",
            StreamType::Orderbook | StreamType::OrderbookDelta => "level2",
            StreamType::Trade => "market_trades",
            StreamType::Kline { .. } => "candles",
            _ => return Err(WebSocketError::ProtocolError(format!("Stream type {:?} not supported", request.stream_type))),
        };

        // For candles, we need to specify granularity
        let granularity = if let StreamType::Kline { ref interval } = request.stream_type {
            Some(super::endpoints::map_kline_interval(interval))
        } else {
            None
        };

        Self::send_subscribe_msg(writer, &self.auth, self.use_private, channel, vec![product_id], granularity).await
            .map_err(|e| WebSocketError::Subscription(e.to_string()))?;

        drop(writer_guard);

        // Track subscription
        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(
        &mut self,
        request: SubscriptionRequest,
    ) -> WebSocketResult<()> {
        self.subscriptions.lock().await.remove(&request);
        Ok(())
    }

    fn event_stream(&self) -> std::pin::Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.subscribe();
        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|r| async move {
            r.ok()
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
