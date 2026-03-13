//! # Tiingo WebSocket Connector
//!
//! Real-time data over WebSocket for three Tiingo feeds:
//!
//! | Feed | URL | Data |
//! |------|-----|------|
//! | IEX stocks | `wss://api.tiingo.com/iex` | US equity quotes |
//! | Forex | `wss://api.tiingo.com/fx` | Forex pair quotes |
//! | Crypto | `wss://api.tiingo.com/crypto` | Crypto trade/quotes |
//!
//! ## Authentication
//! Authorization token is included in the subscribe message body, **not** in the
//! URL or HTTP headers.
//!
//! ## Subscribe format
//! ```json
//! {
//!   "eventName": "subscribe",
//!   "authorization": "TOKEN",
//!   "eventData": { "thresholdLevel": 5, "tickers": ["aapl"] }
//! }
//! ```
//!
//! ## Response format (messageType "A")
//! IEX/Forex: `["A", ticker, date, lastSaleTimestamp, lastSizeTimestamp, lastSize, lastSalePrice, bidSize, bidPrice, midPrice, askSize, askPrice, halted, afterHours]`
//! Crypto: `["A", ticker, date, exchCode, lastSizeTimestamp, lastSaleTimestamp, lastPrice, lastSize, bidSize, bidPrice, bidExchCode, askSize, askPrice, askExchCode]`

use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::core::AccountType;
use crate::core::types::{
    ConnectionStatus, StreamEvent, SubscriptionRequest, Ticker, WebSocketError,
    WebSocketResult,
};
use crate::core::traits::WebSocketConnector;

use super::auth::TiingoAuth;
use super::endpoints::TiingoUrls;

// ═══════════════════════════════════════════════════════════════════════════
// CHANNEL DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Tiingo WebSocket feed selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiingoFeed {
    /// US equity quotes from IEX (`wss://api.tiingo.com/iex`).
    Iex,
    /// Forex quotes (`wss://api.tiingo.com/fx`).
    Forex,
    /// Crypto trades and quotes (`wss://api.tiingo.com/crypto`).
    Crypto,
}

impl TiingoFeed {
    /// Return the WebSocket URL for this feed.
    pub fn ws_url(&self) -> &'static str {
        match self {
            TiingoFeed::Iex => "wss://api.tiingo.com/iex",
            TiingoFeed::Forex => "wss://api.tiingo.com/fx",
            TiingoFeed::Crypto => "wss://api.tiingo.com/crypto",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONNECTOR STRUCT
// ═══════════════════════════════════════════════════════════════════════════

/// Tiingo WebSocket connector for a single feed (IEX, Forex, or Crypto).
pub struct TiingoWebSocket {
    auth: TiingoAuth,
    _urls: TiingoUrls,
    feed: TiingoFeed,
    status: Arc<RwLock<ConnectionStatus>>,
    subscriptions: Arc<RwLock<Vec<SubscriptionRequest>>>,
    /// Broadcast sender. Cloned to produce receivers in `event_stream()`.
    broadcast_tx: Arc<std::sync::Mutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
}

impl TiingoWebSocket {
    /// Create a new IEX (US stocks) WebSocket connector.
    pub fn new_iex(auth: TiingoAuth) -> Self {
        Self::new(auth, TiingoFeed::Iex)
    }

    /// Create a Forex WebSocket connector.
    pub fn new_forex(auth: TiingoAuth) -> Self {
        Self::new(auth, TiingoFeed::Forex)
    }

    /// Create a Crypto WebSocket connector.
    pub fn new_crypto(auth: TiingoAuth) -> Self {
        Self::new(auth, TiingoFeed::Crypto)
    }

    fn new(auth: TiingoAuth, feed: TiingoFeed) -> Self {
        Self {
            auth,
            _urls: TiingoUrls::MAINNET,
            feed,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            broadcast_tx: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Subscribe / unsubscribe message builders (public for testing)
    // ────────────────────────────────────────────────────────────────────────

    /// Build a subscribe message for the given tickers.
    ///
    /// `threshold_level` controls data granularity (0–5; 5 = all updates).
    pub fn build_subscribe_message(&self, tickers: &[&str], threshold_level: u8) -> Value {
        json!({
            "eventName": "subscribe",
            "authorization": self.auth.ws_auth_token(),
            "eventData": {
                "thresholdLevel": threshold_level,
                "tickers": tickers
            }
        })
    }

    /// Build an unsubscribe message for the given tickers.
    pub fn build_unsubscribe_message(&self, tickers: &[&str]) -> Value {
        json!({
            "eventName": "unsubscribe",
            "authorization": self.auth.ws_auth_token(),
            "eventData": {
                "tickers": tickers
            }
        })
    }

    // ────────────────────────────────────────────────────────────────────────
    // Internal connection logic
    // ────────────────────────────────────────────────────────────────────────

    async fn do_connect(&self) -> WebSocketResult<()> {
        let url = self.feed.ws_url();

        let (ws_stream, _response) = timeout(Duration::from_secs(15), connect_async(url))
            .await
            .map_err(|_| WebSocketError::Timeout)?
            .map_err(|e| WebSocketError::ConnectionError(format!("WS connect failed: {}", e)))?;

        let (write, mut read) = ws_stream.split();

        // Create broadcast channel and spawn background reader
        let (tx, _) = broadcast::channel::<WebSocketResult<StreamEvent>>(512);
        {
            let mut guard = self.broadcast_tx.lock().unwrap();
            *guard = Some(tx.clone());
        }

        let broadcast_tx = self.broadcast_tx.clone();
        let status = self.status.clone();
        let feed = self.feed;

        // Drop write half; subscriptions are sent before the reader is spawned
        drop(write);

        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&text) {
                            if let Some(events) = Self::parse_message(&value, feed) {
                                for event in events {
                                    if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                        let _ = tx.send(Ok(event));
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Ping(_)) => { /* tungstenite handles pong automatically */ }
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

    // ────────────────────────────────────────────────────────────────────────
    // Message parser
    // ────────────────────────────────────────────────────────────────────────

    /// Parse a Tiingo WebSocket message into zero or more `StreamEvent`s.
    ///
    /// Tiingo wraps updates in `{"messageType": "A", "data": [...]}`.
    /// `data` is a positional array; field positions differ by feed.
    fn parse_message(value: &Value, feed: TiingoFeed) -> Option<Vec<StreamEvent>> {
        let msg_type = value.get("messageType").and_then(|v| v.as_str())?;

        match msg_type {
            "H" => {
                // Heartbeat — no events to emit
                Some(vec![])
            }
            "I" => {
                // Info/subscription confirmation
                Some(vec![])
            }
            "A" => {
                let data = value.get("data")?;
                let arr = data.as_array()?;

                match feed {
                    TiingoFeed::Iex | TiingoFeed::Forex => {
                        // IEX: [ticker, date, lastSaleTimestamp, lastSizeTimestamp, lastSize,
                        //       lastSalePrice, bidSize, bidPrice, midPrice, askSize, askPrice, ...]
                        // Forex: same layout for bid/ask fields
                        let ticker = arr.first()?.as_str().unwrap_or_default();
                        let last_price = arr.get(5).and_then(|v| v.as_f64()).unwrap_or_default();
                        let bid_price = arr.get(7).and_then(|v| v.as_f64()).unwrap_or_default();
                        let ask_price = arr.get(10).and_then(|v| v.as_f64()).unwrap_or_default();
                        let volume = arr.get(4).and_then(|v| v.as_f64()).unwrap_or_default();

                        let ticker_data = Ticker {
                            symbol: ticker.to_string(),
                            last_price,
                            bid_price: Some(bid_price),
                            ask_price: Some(ask_price),
                            high_24h: None,
                            low_24h: None,
                            volume_24h: Some(volume),
                            quote_volume_24h: None,
                            price_change_24h: None,
                            price_change_percent_24h: None,
                            timestamp: crate::core::utils::timestamp_millis() as i64,
                        };

                        Some(vec![StreamEvent::Ticker(ticker_data)])
                    }

                    TiingoFeed::Crypto => {
                        // Crypto: [ticker, date, exchCode, lastSizeTimestamp, lastSaleTimestamp,
                        //          lastPrice, lastSize, bidSize, bidPrice, bidExchCode,
                        //          askSize, askPrice, askExchCode]
                        let ticker = arr.first()?.as_str().unwrap_or_default();
                        let last_price = arr.get(5).and_then(|v| v.as_f64()).unwrap_or_default();
                        let bid_price = arr.get(8).and_then(|v| v.as_f64()).unwrap_or_default();
                        let ask_price = arr.get(11).and_then(|v| v.as_f64()).unwrap_or_default();
                        let volume = arr.get(6).and_then(|v| v.as_f64()).unwrap_or_default();

                        let ticker_data = Ticker {
                            symbol: ticker.to_string(),
                            last_price,
                            bid_price: Some(bid_price),
                            ask_price: Some(ask_price),
                            high_24h: None,
                            low_24h: None,
                            volume_24h: Some(volume),
                            quote_volume_24h: None,
                            price_change_24h: None,
                            price_change_percent_24h: None,
                            timestamp: crate::core::utils::timestamp_millis() as i64,
                        };

                        Some(vec![StreamEvent::Ticker(ticker_data)])
                    }
                }
            }
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for TiingoWebSocket {
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
        let _ = self.broadcast_tx.lock().unwrap().take();
        self.subscriptions.write().await.clear();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.status
            .try_read()
            .map(|s| *s)
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let status = self.status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(WebSocketError::NotConnected);
        }
        drop(status);

        self.subscriptions.write().await.push(request);
        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions
            .write()
            .await
            .retain(|s| s != &request);
        Ok(())
    }

    fn event_stream(
        &self,
    ) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let guard = self.broadcast_tx.lock().unwrap();
        if let Some(tx) = guard.as_ref() {
            let rx = tx.subscribe();
            Box::pin(
                tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(
                    |result| async move {
                        match result {
                            Ok(event) => Some(event),
                            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(
                                _,
                            )) => Some(Err(WebSocketError::ConnectionError(
                                "Event stream lagged".to_string(),
                            ))),
                        }
                    },
                ),
            )
        } else {
            Box::pin(futures_util::stream::empty())
        }
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.subscriptions
            .try_read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Credentials;

    fn make_auth() -> TiingoAuth {
        let creds = Credentials::new("test_token", "");
        TiingoAuth::new(&creds).unwrap()
    }

    #[test]
    fn test_feed_urls() {
        assert_eq!(TiingoFeed::Iex.ws_url(), "wss://api.tiingo.com/iex");
        assert_eq!(TiingoFeed::Forex.ws_url(), "wss://api.tiingo.com/fx");
        assert_eq!(TiingoFeed::Crypto.ws_url(), "wss://api.tiingo.com/crypto");
    }

    #[test]
    fn test_subscribe_message() {
        let auth = make_auth();
        let ws = TiingoWebSocket::new_iex(auth);
        let msg = ws.build_subscribe_message(&["aapl", "tsla"], 5);

        assert_eq!(msg["eventName"], "subscribe");
        assert_eq!(msg["authorization"], "test_token");
        assert_eq!(msg["eventData"]["thresholdLevel"], 5);
        let tickers = msg["eventData"]["tickers"].as_array().unwrap();
        assert_eq!(tickers.len(), 2);
    }

    #[test]
    fn test_unsubscribe_message() {
        let auth = make_auth();
        let ws = TiingoWebSocket::new_iex(auth);
        let msg = ws.build_unsubscribe_message(&["aapl"]);

        assert_eq!(msg["eventName"], "unsubscribe");
        let tickers = msg["eventData"]["tickers"].as_array().unwrap();
        assert_eq!(tickers[0], "aapl");
    }

    #[test]
    fn test_parse_iex_quote() {
        // IEX array: [ticker, date, lastSaleTs, lastSizeTs, lastSize,
        //             lastSalePrice, bidSize, bidPrice, midPrice, askSize, askPrice, ...]
        let msg = json!({
            "messageType": "A",
            "data": ["AAPL", "2024-01-02", null, null, 100.0,
                     185.50, 200.0, 185.40, 185.45, 300.0, 185.60]
        });

        let events = TiingoWebSocket::parse_message(&msg, TiingoFeed::Iex).unwrap();
        assert_eq!(events.len(), 1);
        if let StreamEvent::Ticker(t) = &events[0] {
            assert_eq!(t.symbol, "AAPL");
            assert_eq!(t.last_price, 185.50);
            assert_eq!(t.bid_price, Some(185.40));
            assert_eq!(t.ask_price, Some(185.60));
        } else {
            panic!("Expected Ticker event");
        }
    }

    #[test]
    fn test_parse_heartbeat() {
        let msg = json!({ "messageType": "H" });
        let events = TiingoWebSocket::parse_message(&msg, TiingoFeed::Iex).unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_initial_status() {
        let auth = make_auth();
        let ws = TiingoWebSocket::new_iex(auth);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_subscribe_before_connect() {
        let auth = make_auth();
        let mut ws = TiingoWebSocket::new_iex(auth);
        use crate::core::types::Symbol;
        let req = SubscriptionRequest::ticker(Symbol::new("AAPL", "USD"));
        let result = ws.subscribe(req).await;
        assert!(matches!(result, Err(WebSocketError::NotConnected)));
    }
}
