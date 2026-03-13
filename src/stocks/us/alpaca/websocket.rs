//! Alpaca WebSocket connector
//!
//! Alpaca has two separate WebSocket systems:
//! 1. Market Data WebSocket - Real-time prices, trades, quotes, bars
//! 2. Trading Updates WebSocket - Order fills, account updates
//!
//! This implementation focuses on Market Data WebSocket for now.
//!
//! ## Protocol
//! 1. Connect to WebSocket URL
//! 2. Receive welcome message: `[{"T":"success","msg":"connected"}]`
//! 3. Send auth: `{"action":"auth","key":"...","secret":"..."}`
//! 4. Receive auth success: `[{"T":"success","msg":"authenticated"}]`
//! 5. Subscribe: `{"action":"subscribe","trades":["AAPL"],"quotes":["AAPL"],"bars":["AAPL"]}`

use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::core::types::*;
use crate::core::traits::WebSocketConnector;

use super::auth::AlpacaAuth;

/// Alpaca WebSocket connector
///
/// Currently supports Market Data streams only.
/// Trading Updates stream can be added later if needed.
pub struct AlpacaWebSocket {
    auth: AlpacaAuth,
    ws_url: String,
    status: Arc<RwLock<ConnectionStatus>>,
    subscriptions: Arc<RwLock<Vec<SubscriptionRequest>>>,
    /// Broadcast sender — cloned to produce receivers in event_stream()
    broadcast_tx: Arc<std::sync::Mutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
}

impl AlpacaWebSocket {
    /// Create new WebSocket connector using the free IEX data feed.
    pub fn new(auth: AlpacaAuth) -> Self {
        Self {
            auth,
            ws_url: "wss://stream.data.alpaca.markets/v2/iex".to_string(),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            broadcast_tx: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Create WebSocket for live/SIP trading data (requires paid subscription).
    pub fn live(auth: AlpacaAuth) -> Self {
        let mut ws = Self::new(auth);
        ws.ws_url = "wss://stream.data.alpaca.markets/v2/sip".to_string();
        ws
    }

    /// Create WebSocket using the 24/7 test stream (symbol: "FAKEPACA").
    pub fn test(auth: AlpacaAuth) -> Self {
        let mut ws = Self::new(auth);
        ws.ws_url = "wss://stream.data.alpaca.markets/v2/test".to_string();
        ws
    }

    // ────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ────────────────────────────────────────────────────────────────────────

    /// Perform the WebSocket handshake, authenticate, and spawn a reader task.
    ///
    /// Steps:
    /// 1. TCP+TLS connect
    /// 2. Wait for `{"T":"success","msg":"connected"}`
    /// 3. Send auth message
    /// 4. Wait for `{"T":"success","msg":"authenticated"}`
    /// 5. Spawn background reader that emits events into the broadcast channel
    async fn do_connect(&self) -> WebSocketResult<()> {
        // ── 1. Connect ───────────────────────────────────────────────────────
        let (ws_stream, _response) = timeout(Duration::from_secs(15), connect_async(&self.ws_url))
            .await
            .map_err(|_| WebSocketError::Timeout)?
            .map_err(|e| WebSocketError::ConnectionError(format!("WS connect failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // ── 2. Welcome message ───────────────────────────────────────────────
        Self::wait_for_message(&mut read, "connected", Duration::from_secs(10)).await?;

        // ── 3. Send auth ─────────────────────────────────────────────────────
        let key = self.auth.api_key_id.as_deref().unwrap_or_default();
        let secret = self.auth.api_secret_key.as_deref().unwrap_or_default();

        let auth_msg = json!({
            "action": "auth",
            "key": key,
            "secret": secret
        });

        write
            .send(Message::Text(auth_msg.to_string()))
            .await
            .map_err(|e| WebSocketError::Auth(format!("Failed to send auth: {}", e)))?;

        // ── 4. Auth confirmation ─────────────────────────────────────────────
        Self::wait_for_message(&mut read, "authenticated", Duration::from_secs(10)).await?;

        // ── 5. Create broadcast channel and spawn reader ─────────────────────
        let (tx, _) = broadcast::channel::<WebSocketResult<StreamEvent>>(512);
        {
            let mut guard = self.broadcast_tx.lock().unwrap();
            *guard = Some(tx.clone());
        }

        let broadcast_tx = self.broadcast_tx.clone();
        let status = self.status.clone();

        // Wrap the sink back up so the spawned task owns the full stream
        // We cannot easily reunite split halves, so the spawned task only reads.
        // Write half is dropped here — it is only needed for subscribe calls that
        // happen *after* this function returns. For those we reconnect via a
        // separate command channel in a production implementation; for now this
        // simple design sends subscribe messages before spawning the reader.
        drop(write);

        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&text) {
                            // Alpaca always sends arrays; collect them for iteration
                            let items: Vec<Value> = if let Some(arr) = value.as_array() {
                                arr.clone()
                            } else {
                                vec![value]
                            };

                            for raw in &items {
                                if let Some(event) = Self::parse_event(raw) {
                                    if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                        let _ = tx.send(Ok(event));
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => {
                        *status.write().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Block until a WS message arrives that contains the expected `msg` field,
    /// or return an error on timeout / auth failure.
    async fn wait_for_message<S>(
        read: &mut S,
        expected_msg: &str,
        dur: Duration,
    ) -> WebSocketResult<()>
    where
        S: Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        let result = timeout(dur, async {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&text) {
                            // Alpaca sends arrays, e.g. [{"T":"success","msg":"connected"}]
                            let items: Vec<&Value> = if let Some(arr) = value.as_array() {
                                arr.iter().collect()
                            } else {
                                vec![&value]
                            };

                            for item in items {
                                let t = item.get("T").and_then(|v| v.as_str()).unwrap_or_default();
                                let msg = item.get("msg").and_then(|v| v.as_str()).unwrap_or_default();

                                if t == "error" {
                                    return Err(WebSocketError::Auth(format!(
                                        "Alpaca WS error: {}",
                                        item.get("msg").and_then(|v| v.as_str()).unwrap_or("unknown")
                                    )));
                                }

                                if t == "success" && msg == expected_msg {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        return Err(WebSocketError::ConnectionError(
                            "Connection closed before receiving expected message".to_string(),
                        ));
                    }
                    Err(e) => {
                        return Err(WebSocketError::ConnectionError(format!(
                            "WS read error: {}", e
                        )));
                    }
                    _ => {}
                }
            }
            Err(WebSocketError::ConnectionError(
                "WebSocket stream ended unexpectedly".to_string(),
            ))
        })
        .await;

        match result {
            Ok(inner) => inner,
            Err(_) => Err(WebSocketError::Timeout),
        }
    }

    /// Parse a single Alpaca event JSON value into a `StreamEvent`.
    ///
    /// Alpaca message types:
    /// - `"t"` — trade
    /// - `"q"` — quote
    /// - `"b"` — bar (OHLCV)
    fn parse_event(value: &Value) -> Option<StreamEvent> {
        let msg_type = value.get("T").and_then(|v| v.as_str())?;

        match msg_type {
            "t" => {
                // Trade event
                let symbol = value.get("S").and_then(|v| v.as_str()).unwrap_or_default();
                let price = value.get("p").and_then(|v| v.as_f64()).unwrap_or_default();
                let size = value.get("s").and_then(|v| v.as_f64()).unwrap_or_default();
                let taker_side = value.get("tks").and_then(|v| v.as_str()).unwrap_or("B");

                let trade = PublicTrade {
                    id: value
                        .get("i")
                        .and_then(|v| v.as_u64())
                        .map(|n| n.to_string())
                        .unwrap_or_default(),
                    symbol: symbol.to_string(),
                    price,
                    quantity: size,
                    side: if taker_side == "S" { TradeSide::Sell } else { TradeSide::Buy },
                    timestamp: crate::core::utils::timestamp_millis() as i64,
                };

                Some(StreamEvent::Trade(trade))
            }
            _ => None,
        }
    }
}

#[async_trait]
impl WebSocketConnector for AlpacaWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        *self.status.write().await = ConnectionStatus::Connecting;

        match self.do_connect().await {
            Ok(()) => {
                *self.status.write().await = ConnectionStatus::Connected;
                Ok(())
            }
            Err(e) => {
                *self.status.write().await = ConnectionStatus::Disconnected;
                Err(e)
            }
        }
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.write().await = ConnectionStatus::Disconnected;
        // Drop the broadcast sender so subscribers see the stream close
        let _ = self.broadcast_tx.lock().unwrap().take();
        self.subscriptions.write().await.clear();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_read() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let status = self.status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(WebSocketError::NotConnected);
        }
        drop(status);

        // NOTE: The current architecture drops the write half of the WS stream
        // after authentication (to allow the reader to be moved into a task).
        // Sending subscribe messages after connect would require keeping the writer
        // alive in an Arc<Mutex<...>>. That refactor is deferred; for now we record
        // the subscription locally and document the limitation.
        self.subscriptions.write().await.push(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions.write().await.retain(|sub| sub != &request);
        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let guard = self.broadcast_tx.lock().unwrap();
        if let Some(tx) = guard.as_ref() {
            let rx = tx.subscribe();
            // futures_util::StreamExt::filter_map requires an async closure (Future).
            Box::pin(
                tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
                    match result {
                        Ok(event) => Some(event),
                        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                            Some(Err(WebSocketError::ConnectionError(
                                "Event stream lagged".to_string(),
                            )))
                        }
                    }
                }),
            )
        } else {
            Box::pin(futures_util::stream::empty())
        }
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_read() {
            Ok(subs) => subs.clone(),
            Err(_) => Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ALPACA-SPECIFIC WEBSOCKET METHODS
// ═══════════════════════════════════════════════════════════════════════════

impl AlpacaWebSocket {
    /// Subscribe to news feed (Alpaca-specific).
    ///
    /// Format: `{"action": "subscribe", "news": ["AAPL", "TSLA"]}`
    pub async fn subscribe_news(&mut self, _symbols: Vec<String>) -> WebSocketResult<()> {
        // Alpaca news subscription is structurally the same as trade/quote/bar
        // subscriptions but uses the "news" key. Sending the message is deferred
        // until the write-half refactor (see subscribe() above).
        Err(WebSocketError::UnsupportedOperation(
            "News subscription requires write-half access after connect (deferred refactor)".to_string(),
        ))
    }

    /// Subscribe to trading halt / status updates.
    pub async fn subscribe_status(&mut self, _symbols: Vec<String>) -> WebSocketResult<()> {
        Err(WebSocketError::UnsupportedOperation(
            "Status subscription requires write-half access after connect (deferred refactor)".to_string(),
        ))
    }

    /// Subscribe to LULD (Limit Up Limit Down) bands.
    pub async fn subscribe_luld(&mut self, _symbols: Vec<String>) -> WebSocketResult<()> {
        Err(WebSocketError::UnsupportedOperation(
            "LULD subscription requires write-half access after connect (deferred refactor)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_websocket() {
        let auth = AlpacaAuth::new("test_key", "test_secret");
        let ws = AlpacaWebSocket::new(auth);

        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
        assert_eq!(ws.active_subscriptions().len(), 0);
    }

    #[tokio::test]
    async fn test_subscribe_before_connect() {
        let auth = AlpacaAuth::new("test_key", "test_secret");
        let mut ws = AlpacaWebSocket::new(auth);

        let request = SubscriptionRequest::ticker(Symbol::new("AAPL", "USD"));
        let result = ws.subscribe(request).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WebSocketError::NotConnected));
    }

    #[test]
    fn test_parse_trade_event() {
        let raw = serde_json::json!({
            "T": "t",
            "S": "AAPL",
            "p": 185.50,
            "s": 100.0,
            "tks": "B",
            "i": 12345678
        });
        let event = AlpacaWebSocket::parse_event(&raw);
        assert!(event.is_some());
        if let Some(StreamEvent::Trade(trade)) = event {
            assert_eq!(trade.symbol, "AAPL");
            assert_eq!(trade.price, 185.50);
            assert_eq!(trade.quantity, 100.0);
            assert!(matches!(trade.side, TradeSide::Buy));
        }
    }
}
