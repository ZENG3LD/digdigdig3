//! # Upbit WebSocket Implementation
//!
//! WebSocket connector for Upbit.
//!
//! ## Design
//! Uses a split sink/stream pattern: a background read task owns the `SplitStream`
//! half and a write channel (`mpsc::UnboundedSender<Message>`) allows `subscribe()`
//! to send frames without contending with reads. This eliminates the shared-mutex
//! bottleneck that caused snapshots to be missed.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;

use crate::core::{
    Credentials, AccountType, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError, OrderbookCapabilities};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::SimpleRateLimiter;

use super::auth::UpbitAuth;
use super::endpoints::UpbitUrls;
use super::parser::UpbitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Global rate limiter for Upbit WebSocket connections (5 per 10 seconds)
static GLOBAL_WS_LIMITER: OnceLock<Arc<StdMutex<SimpleRateLimiter>>> = OnceLock::new();

fn get_global_ws_limiter() -> Arc<StdMutex<SimpleRateLimiter>> {
    GLOBAL_WS_LIMITER.get_or_init(|| {
        Arc::new(StdMutex::new(SimpleRateLimiter::new(5, Duration::from_secs(10))))
    }).clone()
}

// ═══════════════════════════════════════════════════════════════════════════════
// UPBIT WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsWriter = SplitSink<WsStream, Message>;
type WsReader = SplitStream<WsStream>;

/// Upbit WebSocket connector
pub struct UpbitWebSocket {
    /// Authentication (None for public channels only)
    auth: Option<UpbitAuth>,
    /// URLs (region-specific)
    urls: UpbitUrls,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender (for multiple consumers)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Write command channel — subscribe/ping send here; background task owns WsWriter
    write_tx: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
    /// WebSocket writer (used by background write task)
    ws_writer: Arc<Mutex<Option<WsWriter>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// WebSocket ping round-trip time in milliseconds (0 = not measured yet).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl UpbitWebSocket {
    /// Create new Upbit WebSocket connector.
    /// region: "kr" (Korea / KRW markets), "sg" (Singapore), "id" (Indonesia), "th" (Thailand)
    pub async fn new(
        credentials: Option<Credentials>,
        region: &str,
    ) -> ExchangeResult<Self> {
        let urls = match region {
            "kr" | "korea" => UpbitUrls::KOREA,
            "id" => UpbitUrls::INDONESIA,
            "th" => UpbitUrls::THAILAND,
            _ => UpbitUrls::SINGAPORE,
        };

        let auth = credentials
            .as_ref()
            .map(UpbitAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            write_tx: Arc::new(Mutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Build Upbit subscription JSON payload.
    ///
    /// Format: `[{"ticket":"…"}, {"type":"…","codes":["…"],"is_only_realtime":true}, {"format":"DEFAULT"}]`
    ///
    /// `is_only_realtime: true` → skip initial snapshot; only real-time updates.
    /// This avoids the race where the snapshot is broadcast before the consumer
    /// has subscribed to the broadcast channel.
    fn build_sub_message(msg_type: &str, symbols: &[String]) -> Message {
        let payload = json!([
            {"ticket": "upbit-connector"},
            {
                "type": msg_type,
                "codes": symbols,
                "is_only_realtime": true
            },
            {"format": "DEFAULT"}
        ]);
        Message::Text(payload.to_string())
    }

    /// Parse a raw frame text (binary decoded or text) into a StreamEvent.
    fn parse_frame(text: &str) -> Option<StreamEvent> {
        let value: Value = serde_json::from_str(text).ok()?;

        // Server status ping response — ignore
        if let Some(status) = value.get("status") {
            if status == "UP" {
                return None;
            }
        }

        let msg_type = value.get("type")
            .or_else(|| value.get("ty"))
            .and_then(|t| t.as_str())?;

        match msg_type {
            "ticker" => {
                UpbitParser::parse_ws_ticker(&value).ok().map(StreamEvent::Ticker)
            }
            "trade" => {
                UpbitParser::parse_ws_trade(&value).ok().map(StreamEvent::Trade)
            }
            "orderbook" => {
                UpbitParser::parse_ws_orderbook(&value).ok().map(StreamEvent::OrderbookSnapshot)
            }
            _ => None,
        }
    }

    /// Start the background read task.
    ///
    /// Reads from `ws_reader` and broadcasts parsed events. Returns the write
    /// channel sender so callers can push frames without locking the reader.
    fn start_tasks(
        ws_writer: Arc<Mutex<Option<WsWriter>>>,
        ws_reader: WsReader,
        broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) -> mpsc::UnboundedSender<Message> {
        let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Message>();

        // Write task: drains write_rx and sends to WsWriter
        let status_w = status.clone();
        tokio::spawn(async move {
            while let Some(msg) = write_rx.recv().await {
                let mut guard = ws_writer.lock().await;
                if let Some(writer) = guard.as_mut() {
                    if writer.send(msg).await.is_err() {
                        *status_w.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        // Read task: reads from WsReader and broadcasts events
        let write_tx_clone = write_tx.clone();
        tokio::spawn(async move {
            let mut reader = ws_reader;
            while let Some(msg_result) = reader.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Some(event) = Self::parse_frame(&text) {
                            if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                let _ = tx.send(Ok(event));
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // Upbit DEFAULT format sends binary frames with raw UTF-8 JSON.
                        if let Ok(text) = String::from_utf8(data) {
                            if let Some(event) = Self::parse_frame(&text) {
                                if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                    let _ = tx.send(Ok(event));
                                }
                            }
                        }
                    }
                    Ok(Message::Ping(payload)) => {
                        let _ = write_tx_clone.send(Message::Pong(payload));
                    }
                    Ok(Message::Pong(_)) => {
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Ok(Message::Close(_)) | Err(_) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {}
                }
            }
            // Drop broadcast sender so all stream receivers see end-of-stream.
            let _ = broadcast_tx.lock().unwrap().take();
        });

        write_tx
    }
}

#[async_trait]
impl WebSocketConnector for UpbitWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        // Rate limit connections
        {
            let limiter = get_global_ws_limiter();
            loop {
                let can_connect = {
                    let mut g = limiter.lock().expect("limiter poisoned");
                    g.try_acquire()
                };
                if can_connect { break; }
                let wait = {
                    let g = limiter.lock().expect("limiter poisoned");
                    g.time_until_ready()
                };
                if wait > Duration::ZERO {
                    tokio::time::sleep(wait).await;
                } else {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }

        let ws_url = if self.auth.is_some() {
            self.urls.ws_private_url()
        } else {
            self.urls.ws_url().to_string()
        };

        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        let (writer, reader) = ws_stream.split();
        *self.ws_writer.lock().await = Some(writer);
        *self.status.lock().await = ConnectionStatus::Connected;
        *self.last_ping.lock().await = Instant::now();

        // Create broadcast channel
        let (tx, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(tx);

        // Start background tasks and store write channel
        let write_tx = Self::start_tasks(
            self.ws_writer.clone(),
            reader,
            self.broadcast_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );
        *self.write_tx.lock().await = Some(write_tx);

        Ok(())
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        *self.write_tx.lock().await = None;
        *self.ws_writer.lock().await = None;
        *self.status.lock().await = ConnectionStatus::Disconnected;
        let _ = self.broadcast_tx.lock().unwrap().take();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.status.try_lock()
            .map(|s| *s)
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions.lock().await.insert(request.clone());

        use crate::core::utils::symbol_normalizer::SymbolNormalizer;
        use crate::core::types::ExchangeId;
        let upbit_symbol = if let Some(raw) = request.symbol.raw() {
            raw.to_string()
        } else {
            SymbolNormalizer::to_exchange(ExchangeId::Upbit, &request.symbol, AccountType::Spot)
                .unwrap_or_else(|_| format!(
                    "{}-{}",
                    request.symbol.quote.to_uppercase(),
                    request.symbol.base.to_uppercase()
                ))
        };

        let write_tx_guard = self.write_tx.lock().await;
        let tx = write_tx_guard.as_ref().ok_or(WebSocketError::NotConnected)?;

        match request.stream_type {
            StreamType::Ticker => {
                tx.send(Self::build_sub_message("ticker", &[upbit_symbol.clone()]))
                    .map_err(|e| WebSocketError::SendError(e.to_string()))?;
                // Also subscribe orderbook so bid/ask can flow via synthetic Ticker.
                tx.send(Self::build_sub_message("orderbook", &[upbit_symbol]))
                    .map_err(|e| WebSocketError::SendError(e.to_string()))?;
            }
            StreamType::Trade => {
                tx.send(Self::build_sub_message("trade", &[upbit_symbol]))
                    .map_err(|e| WebSocketError::SendError(e.to_string()))?;
            }
            StreamType::Orderbook => {
                tx.send(Self::build_sub_message("orderbook", &[upbit_symbol]))
                    .map_err(|e| WebSocketError::SendError(e.to_string()))?;
            }
            _ => {
                return Err(WebSocketError::UnsupportedOperation(
                    format!("Upbit WS: unsupported stream type {:?}", request.stream_type)
                ));
            }
        }

        Ok(())
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions.lock().await.remove(&request);
        Err(WebSocketError::UnsupportedOperation(
            "Upbit does not support unsubscribe — reconnect required".to_string()
        ))
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let tx_guard = self.broadcast_tx.lock().unwrap();
        if let Some(ref tx) = *tx_guard {
            let rx = tx.subscribe();
            Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|r| async move {
                r.ok()
            }))
        } else {
            Box::pin(futures_util::stream::empty())
        }
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.subscriptions.try_lock()
            .map(|subs| subs.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[1, 5, 15, 30],
            ws_default_depth: Some(30),
            rest_max_depth: Some(30),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: false,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: &[],
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &[],
        }
    }
}
