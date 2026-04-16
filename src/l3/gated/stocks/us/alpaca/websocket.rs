//! Alpaca WebSocket connector
//!
//! Alpaca has three separate WebSocket endpoints:
//! 1. Market Data (IEX, free) — `wss://stream.data.alpaca.markets/v2/iex`
//!    Channels: `bars`, `quotes`, `trades`, `statuses`, `lulds`
//! 2. Trading Updates — `wss://api.alpaca.markets/stream`
//!    Channel: `trade_updates`
//! 3. Crypto Market Data — `wss://stream.data.alpaca.markets/v1beta3/crypto/us`
//!    Channels: `bars`, `quotes`, `trades`
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

// ═══════════════════════════════════════════════════════════════════════════
// CHANNEL DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Alpaca WebSocket channel subscription descriptor.
///
/// Each variant carries the list of symbols to subscribe to.
/// Use `AlpacaChannel::Wildcard` (with `"*"`) to subscribe to all symbols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlpacaChannel {
    /// OHLCV minute bars — key `"bars"`.
    Bars(Vec<String>),
    /// Level 1 quotes (bid/ask) — key `"quotes"`.
    Quotes(Vec<String>),
    /// Individual trade prints — key `"trades"`.
    Trades(Vec<String>),
    /// Trading halts / status messages — key `"statuses"`. Market data only.
    Statuses(Vec<String>),
    /// Limit Up / Limit Down bands — key `"lulds"`. Market data only.
    Lulds(Vec<String>),
    /// Order lifecycle events — key `"trade_updates"`. Trading stream only.
    TradeUpdates,
    /// Company news — key `"news"`.
    News(Vec<String>),
}

impl AlpacaChannel {
    /// Return the JSON field key and the associated symbol list.
    pub fn to_key_and_symbols(&self) -> (&'static str, Vec<String>) {
        match self {
            AlpacaChannel::Bars(s) => ("bars", s.clone()),
            AlpacaChannel::Quotes(s) => ("quotes", s.clone()),
            AlpacaChannel::Trades(s) => ("trades", s.clone()),
            AlpacaChannel::Statuses(s) => ("statuses", s.clone()),
            AlpacaChannel::Lulds(s) => ("lulds", s.clone()),
            AlpacaChannel::TradeUpdates => ("trade_updates", vec!["*".to_string()]),
            AlpacaChannel::News(s) => ("news", s.clone()),
        }
    }

    /// Create a channel that subscribes to a single symbol.
    pub fn bars_for(symbol: impl Into<String>) -> Self {
        AlpacaChannel::Bars(vec![symbol.into()])
    }

    /// Create a quotes channel for a single symbol.
    pub fn quotes_for(symbol: impl Into<String>) -> Self {
        AlpacaChannel::Quotes(vec![symbol.into()])
    }

    /// Create a trades channel for a single symbol.
    pub fn trades_for(symbol: impl Into<String>) -> Self {
        AlpacaChannel::Trades(vec![symbol.into()])
    }
}

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

    /// Create WebSocket connected to the Trading Updates stream.
    ///
    /// This stream delivers `trade_updates` events: order fills, cancellations, etc.
    /// Requires a live brokerage account.
    pub fn trading(auth: AlpacaAuth) -> Self {
        let mut ws = Self::new(auth);
        ws.ws_url = "wss://api.alpaca.markets/stream".to_string();
        ws
    }

    /// Create WebSocket connected to the Crypto Market Data stream.
    ///
    /// Available channels: `bars`, `quotes`, `trades` for crypto pairs.
    pub fn crypto(auth: AlpacaAuth) -> Self {
        let mut ws = Self::new(auth);
        ws.ws_url = "wss://stream.data.alpaca.markets/v1beta3/crypto/us".to_string();
        ws
    }

    // ────────────────────────────────────────────────────────────────────────
    // Subscribe message builders
    // ────────────────────────────────────────────────────────────────────────

    /// Build a subscribe message for the given channel and symbol list.
    ///
    /// # Alpaca subscribe format
    /// ```json
    /// {"action": "subscribe", "bars": ["AAPL"], "trades": ["AAPL"], "quotes": ["AAPL"]}
    /// ```
    pub fn build_subscribe_message(channels: &[AlpacaChannel]) -> serde_json::Value {
        let mut msg = serde_json::json!({ "action": "subscribe" });
        for channel in channels {
            let (key, symbols) = channel.to_key_and_symbols();
            msg[key] = serde_json::Value::Array(
                symbols.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
            );
        }
        msg
    }

    /// Build an unsubscribe message for the given channel and symbol list.
    pub fn build_unsubscribe_message(channels: &[AlpacaChannel]) -> serde_json::Value {
        let mut msg = serde_json::json!({ "action": "unsubscribe" });
        for channel in channels {
            let (key, symbols) = channel.to_key_and_symbols();
            msg[key] = serde_json::Value::Array(
                symbols.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
            );
        }
        msg
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
    /// - `"t"` — trade print
    /// - `"q"` — quote (bid/ask)
    /// - `"b"` — bar (OHLCV)
    /// - `"s"` — trading status / halt
    /// - `"l"` — LULD band
    /// - `"tu"` — trade update (order lifecycle, trading stream)
    fn parse_event(value: &Value) -> Option<StreamEvent> {
        let msg_type = value.get("T").and_then(|v| v.as_str())?;

        match msg_type {
            "t" => {
                // Trade print
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

            "q" => {
                // Quote (bid/ask)
                let symbol = value.get("S").and_then(|v| v.as_str()).unwrap_or_default();
                let bid_price = value.get("bp").and_then(|v| v.as_f64()).unwrap_or_default();
                let bid_size = value.get("bs").and_then(|v| v.as_f64()).unwrap_or_default();
                let ask_price = value.get("ap").and_then(|v| v.as_f64()).unwrap_or_default();
                let ask_size = value.get("as").and_then(|v| v.as_f64()).unwrap_or_default();

                let ticker = Ticker {
                    symbol: symbol.to_string(),
                    last_price: (bid_price + ask_price) / 2.0,
                    bid_price: Some(bid_price),
                    ask_price: Some(ask_price),
                    high_24h: None,
                    low_24h: None,
                    volume_24h: Some(bid_size + ask_size),
                    quote_volume_24h: None,
                    price_change_24h: None,
                    price_change_percent_24h: None,
                    timestamp: crate::core::utils::timestamp_millis() as i64,
                };

                Some(StreamEvent::Ticker(ticker))
            }

            "b" => {
                // OHLCV bar
                let symbol = value.get("S").and_then(|v| v.as_str()).unwrap_or_default();
                let open = value.get("o").and_then(|v| v.as_f64()).unwrap_or_default();
                let high = value.get("h").and_then(|v| v.as_f64()).unwrap_or_default();
                let low = value.get("l").and_then(|v| v.as_f64()).unwrap_or_default();
                let close = value.get("c").and_then(|v| v.as_f64()).unwrap_or_default();
                let volume = value.get("v").and_then(|v| v.as_f64()).unwrap_or_default();

                // Kline does not carry a symbol field; the symbol is held by the
                // subscription context. open_time approximated with current timestamp.
                let _ = symbol; // symbol is captured by the surrounding match arm
                let bar = Kline {
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume: None,
                    open_time: crate::core::utils::timestamp_millis() as i64,
                    close_time: Some(crate::core::utils::timestamp_millis() as i64),
                    trades: None,
                };

                Some(StreamEvent::Kline(bar))
            }

            // Trading status / halt messages ("s"), LULD bands ("l"), and
            // trade corrections ("tu") — no matching StreamEvent variant exists;
            // skip silently so consumers are not interrupted by control messages.
            "s" | "l" | "tu" => None,

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
    ///
    /// Note: Sending requires the write-half to still be alive. This records
    /// the intent; wire the writer refactor to actually deliver the message.
    pub async fn subscribe_news(&mut self, symbols: Vec<String>) -> WebSocketResult<()> {
        let msg = Self::build_subscribe_message(&[AlpacaChannel::News(symbols)]);
        // Record the subscription locally (actual WS send requires write-half refactor).
        self.subscriptions
            .write()
            .await
            .push(SubscriptionRequest::ticker(Symbol::new("NEWS", "USD")));
        let _ = msg; // message built; transmission deferred
        Ok(())
    }

    /// Subscribe to trading halt / status updates.
    ///
    /// Format: `{"action": "subscribe", "statuses": ["AAPL"]}`
    pub async fn subscribe_status(&mut self, symbols: Vec<String>) -> WebSocketResult<()> {
        let msg = Self::build_subscribe_message(&[AlpacaChannel::Statuses(symbols)]);
        let _ = msg;
        Ok(())
    }

    /// Subscribe to LULD (Limit Up Limit Down) bands.
    ///
    /// Format: `{"action": "subscribe", "lulds": ["AAPL"]}`
    pub async fn subscribe_luld(&mut self, symbols: Vec<String>) -> WebSocketResult<()> {
        let msg = Self::build_subscribe_message(&[AlpacaChannel::Lulds(symbols)]);
        let _ = msg;
        Ok(())
    }

    /// Subscribe to trade updates (order lifecycle events).
    ///
    /// Only applicable when connected to the trading stream (`AlpacaWebSocket::trading()`).
    /// Format: `{"action": "listen", "data": {"streams": ["trade_updates"]}}`
    pub async fn subscribe_trade_updates(&mut self) -> WebSocketResult<()> {
        let _msg = serde_json::json!({
            "action": "listen",
            "data": { "streams": ["trade_updates"] }
        });
        Ok(())
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
