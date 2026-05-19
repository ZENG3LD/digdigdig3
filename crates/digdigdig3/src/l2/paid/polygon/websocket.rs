//! # Polygon.io WebSocket Implementation
//!
//! WebSocket connector for Polygon.io real-time and delayed data.
//!
//! ## WebSocket Feeds (separate connections)
//!
//! | Feed | URL | Channels |
//! |------|-----|----------|
//! | Stocks | `wss://socket.polygon.io/stocks` | `T.`, `Q.`, `A.`, `AM.` |
//! | Options | `wss://socket.polygon.io/options` | `T.`, `Q.`, `A.`, `AM.` |
//! | Forex | `wss://socket.polygon.io/forex` | `C.`, `CA.` |
//! | Crypto | `wss://socket.polygon.io/crypto` | `XT.`, `XQ.`, `XA.`, `XAS.` |
//!
//! ## Channel Prefixes (Stocks/Options)
//! - `T.AAPL` — trades
//! - `Q.AAPL` — quotes
//! - `A.AAPL` — second aggregates
//! - `AM.AAPL` — minute aggregates
//!
//! ## Protocol
//! 1. Connect → receive `[{"ev":"status","status":"connected"}]`
//! 2. Send auth: `{"action":"auth","params":"API_KEY"}`
//! 3. Receive `[{"ev":"status","status":"auth_success"}]`
//! 4. Subscribe: `{"action":"subscribe","params":"T.AAPL,Q.AAPL"}`

use std::sync::Arc;

use futures_util::{StreamExt, SinkExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials,
    ExchangeError, ExchangeResult,
    ConnectionStatus, StreamEvent,
};
// Note: WebSocketConnector trait not implemented yet (can be added later if needed)

use super::endpoints::PolygonUrls;
use super::auth::PolygonAuth;
use super::parser::PolygonParser;

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// ═══════════════════════════════════════════════════════════════════════════════
// FEED AND CHANNEL DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Polygon.io WebSocket feed (separate connection per asset class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonFeed {
    /// US equity trades, quotes, aggregates.
    Stocks,
    /// Options trades, quotes, aggregates.
    Options,
    /// Forex currency pair quotes and aggregates.
    Forex,
    /// Crypto trades, quotes, aggregates.
    Crypto,
}

impl PolygonFeed {
    /// Real-time WebSocket URL for this feed.
    pub fn realtime_url(&self) -> &'static str {
        match self {
            PolygonFeed::Stocks => "wss://socket.polygon.io/stocks",
            PolygonFeed::Options => "wss://socket.polygon.io/options",
            PolygonFeed::Forex => "wss://socket.polygon.io/forex",
            PolygonFeed::Crypto => "wss://socket.polygon.io/crypto",
        }
    }

    /// Delayed (15-min) WebSocket URL for this feed.
    pub fn delayed_url(&self) -> &'static str {
        match self {
            PolygonFeed::Stocks => "wss://delayed.polygon.io/stocks",
            PolygonFeed::Options => "wss://delayed.polygon.io/options",
            PolygonFeed::Forex => "wss://delayed.polygon.io/forex",
            PolygonFeed::Crypto => "wss://delayed.polygon.io/crypto",
        }
    }
}

/// Polygon.io WebSocket channel subscription.
///
/// Channels are expressed as `PREFIX.SYMBOL` strings in the `params` field.
/// Multiple channels can be sent in a single subscribe message, comma-separated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolygonChannel {
    // ── Stocks & Options ──
    /// Individual trade prints — prefix `T.`
    Trades(String),
    /// NBBO quotes — prefix `Q.`
    Quotes(String),
    /// Per-second OHLCV aggregates — prefix `A.`
    SecondAggregates(String),
    /// Per-minute OHLCV aggregates — prefix `AM.`
    MinuteAggregates(String),

    // ── Forex ──
    /// Forex quotes — prefix `C.`
    ForexQuotes(String),
    /// Forex per-minute aggregates — prefix `CA.`
    ForexAggregates(String),

    // ── Crypto ──
    /// Crypto trades — prefix `XT.`
    CryptoTrades(String),
    /// Crypto quotes (level 2) — prefix `XQ.`
    CryptoQuotes(String),
    /// Crypto per-second aggregates — prefix `XA.`
    CryptoSecondAggregates(String),
    /// Crypto per-minute aggregates — prefix `XAS.`
    CryptoMinuteAggregates(String),
}

impl PolygonChannel {
    /// Build the `params` string for this channel (e.g. `"T.AAPL"`).
    pub fn to_param(&self) -> String {
        match self {
            PolygonChannel::Trades(s) => format!("T.{}", s.to_uppercase()),
            PolygonChannel::Quotes(s) => format!("Q.{}", s.to_uppercase()),
            PolygonChannel::SecondAggregates(s) => format!("A.{}", s.to_uppercase()),
            PolygonChannel::MinuteAggregates(s) => format!("AM.{}", s.to_uppercase()),
            PolygonChannel::ForexQuotes(s) => format!("C.{}", s.to_uppercase()),
            PolygonChannel::ForexAggregates(s) => format!("CA.{}", s.to_uppercase()),
            PolygonChannel::CryptoTrades(s) => format!("XT.{}", s.to_uppercase()),
            PolygonChannel::CryptoQuotes(s) => format!("XQ.{}", s.to_uppercase()),
            PolygonChannel::CryptoSecondAggregates(s) => format!("XA.{}", s.to_uppercase()),
            PolygonChannel::CryptoMinuteAggregates(s) => format!("XAS.{}", s.to_uppercase()),
        }
    }

    /// Build a subscribe JSON message for a slice of channels.
    ///
    /// # Example
    /// ```ignore
    /// let msg = PolygonChannel::build_subscribe_message(&[
    ///     PolygonChannel::Trades("AAPL".into()),
    ///     PolygonChannel::Quotes("AAPL".into()),
    /// ]);
    /// // {"action":"subscribe","params":"T.AAPL,Q.AAPL"}
    /// ```
    pub fn build_subscribe_message(channels: &[PolygonChannel]) -> serde_json::Value {
        let params = channels
            .iter()
            .map(|c| c.to_param())
            .collect::<Vec<_>>()
            .join(",");
        serde_json::json!({ "action": "subscribe", "params": params })
    }

    /// Build an unsubscribe JSON message for a slice of channels.
    pub fn build_unsubscribe_message(channels: &[PolygonChannel]) -> serde_json::Value {
        let params = channels
            .iter()
            .map(|c| c.to_param())
            .collect::<Vec<_>>()
            .join(",");
        serde_json::json!({ "action": "unsubscribe", "params": params })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Polygon WebSocket connector
pub struct PolygonWebSocket {
    /// Authentication
    auth: PolygonAuth,
    /// URLs
    urls: PolygonUrls,
    /// Use real-time (true) or delayed (false)
    realtime: bool,
    /// Asset-class feed this connector targets
    feed: PolygonFeed,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Event broadcast channel
    event_tx: broadcast::Sender<StreamEvent>,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
}

impl PolygonWebSocket {
    /// Create new WebSocket connector for the Stocks feed.
    pub async fn new(credentials: Credentials, realtime: bool) -> ExchangeResult<Self> {
        Self::for_feed(credentials, PolygonFeed::Stocks, realtime).await
    }

    /// Create a connector targeting a specific asset-class feed.
    pub async fn for_feed(
        credentials: Credentials,
        feed: PolygonFeed,
        realtime: bool,
    ) -> ExchangeResult<Self> {
        let auth = PolygonAuth::new(&credentials)?;
        let urls = PolygonUrls::MAINNET;
        let (event_tx, _) = broadcast::channel(1000);

        Ok(Self {
            auth,
            urls,
            realtime,
            feed,
            ws_stream: Arc::new(Mutex::new(None)),
            event_tx,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
        })
    }

    /// Create an Options feed connector.
    pub async fn options(credentials: Credentials, realtime: bool) -> ExchangeResult<Self> {
        Self::for_feed(credentials, PolygonFeed::Options, realtime).await
    }

    /// Create a Forex feed connector.
    pub async fn forex(credentials: Credentials, realtime: bool) -> ExchangeResult<Self> {
        Self::for_feed(credentials, PolygonFeed::Forex, realtime).await
    }

    /// Create a Crypto feed connector.
    pub async fn crypto(credentials: Credentials, realtime: bool) -> ExchangeResult<Self> {
        Self::for_feed(credentials, PolygonFeed::Crypto, realtime).await
    }

    /// Return the active feed.
    pub fn feed(&self) -> PolygonFeed {
        self.feed
    }

    /// Subscribe to a set of channels using the channel-based API.
    ///
    /// Builds and sends a single subscribe message covering all provided channels.
    pub async fn subscribe_channels(&self, channels: &[PolygonChannel]) -> ExchangeResult<()> {
        if channels.is_empty() {
            return Ok(());
        }
        let msg = PolygonChannel::build_subscribe_message(channels);
        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Subscribe failed: {}", e)))?;
        }
        Ok(())
    }

    /// Unsubscribe from a set of channels.
    pub async fn unsubscribe_channels(&self, channels: &[PolygonChannel]) -> ExchangeResult<()> {
        if channels.is_empty() {
            return Ok(());
        }
        let msg = PolygonChannel::build_unsubscribe_message(channels);
        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Unsubscribe failed: {}", e)))?;
        }
        Ok(())
    }

    /// Connect to WebSocket
    pub async fn connect(&self) -> ExchangeResult<()> {
        let url = if self.realtime {
            self.feed.realtime_url()
        } else {
            self.feed.delayed_url()
        };
        // Fallback to PolygonUrls for legacy callers — keep old logic accessible
        let _ = self.urls.ws_url(self.realtime); // ensure urls field stays used

        // Connect
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.status.lock().await = ConnectionStatus::Connected;

        // Wait for initial connected message
        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            if let Some(Ok(Message::Text(_msg))) = ws.next().await {
                // Should receive: [{"ev":"status","status":"connected","message":"Connected Successfully"}]
                // Just verify we got something
            }
        }

        // Authenticate
        self.authenticate().await?;

        Ok(())
    }

    /// Authenticate WebSocket connection
    async fn authenticate(&self) -> ExchangeResult<()> {
        let auth_msg = self.auth.ws_auth_message();

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(auth_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Auth failed: {}", e)))?;

            // Wait for auth success
            if let Some(Ok(Message::Text(msg))) = ws.next().await {
                let parsed: Value = serde_json::from_str(&msg)
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse auth response: {}", e)))?;

                if let Some(events) = parsed.as_array() {
                    for event in events {
                        if event.get("ev") == Some(&json!("status")) {
                            let status = event.get("status").and_then(|s| s.as_str());
                            let message = event.get("message").and_then(|m| m.as_str()).unwrap_or("");

                            if status == Some("auth_success") {
                                // Already set to Connected, no need to change
                                return Ok(());
                            } else if status == Some("auth_failed") {
                                // Check if this is a free tier limitation
                                if message.contains("subscription") || message.contains("tier") || message.contains("plan") {
                                    return Err(ExchangeError::Auth(
                                        format!("Authentication failed: {}. NOTE: WebSocket access requires Starter plan ($29/mo) or higher. Free tier (Stocks Basic) does NOT have WebSocket access.", message)
                                    ));
                                }
                                return Err(ExchangeError::Auth(format!("Authentication failed: {}", message)));
                            }
                        }
                    }
                }
            }
        }

        Err(ExchangeError::Auth("Authentication timeout. If using free tier, note that WebSocket requires Starter+ plan.".to_string()))
    }

    /// Subscribe to ticker (aggregates, trades, quotes)
    pub async fn subscribe_ticker(&self, symbol: &str) -> ExchangeResult<()> {
        // Subscribe to minute aggregates by default
        let params = format!("AM.{}", symbol.to_uppercase());

        let sub_msg = json!({
            "action": "subscribe",
            "params": params
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Subscribe failed: {}", e)))?;
        }

        Ok(())
    }

    /// Subscribe to specific channels
    pub async fn subscribe(&self, channels: Vec<String>) -> ExchangeResult<()> {
        let params = channels.join(",");

        let sub_msg = json!({
            "action": "subscribe",
            "params": params
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Subscribe failed: {}", e)))?;
        }

        Ok(())
    }

    /// Unsubscribe from channels
    pub async fn unsubscribe(&self, channels: Vec<String>) -> ExchangeResult<()> {
        let params = channels.join(",");

        let unsub_msg = json!({
            "action": "unsubscribe",
            "params": params
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(unsub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Unsubscribe failed: {}", e)))?;
        }

        Ok(())
    }

    /// Get event receiver
    pub fn subscribe_events(&self) -> broadcast::Receiver<StreamEvent> {
        self.event_tx.subscribe()
    }

    /// Start message processing loop
    pub async fn run(&self) -> ExchangeResult<()> {
        loop {
            let msg = {
                let mut ws_lock = self.ws_stream.lock().await;
                if let Some(ref mut ws) = *ws_lock {
                    match ws.next().await {
                        Some(Ok(msg)) => msg,
                        Some(Err(e)) => {
                            *self.status.lock().await = ConnectionStatus::Disconnected;
                            return Err(ExchangeError::Network(format!("WebSocket error: {}", e)));
                        }
                        None => {
                            *self.status.lock().await = ConnectionStatus::Disconnected;
                            return Err(ExchangeError::Network("WebSocket closed".to_string()));
                        }
                    }
                } else {
                    return Err(ExchangeError::Network("No WebSocket connection".to_string()));
                }
            };

            match msg {
                Message::Text(text) => {
                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                        if let Ok(events) = PolygonParser::parse_ws_message(&value) {
                            for event in events {
                                let _ = self.event_tx.send(event);
                            }
                        }
                    }
                }
                Message::Ping(_) => {
                    // Respond to ping
                    let mut ws_lock = self.ws_stream.lock().await;
                    if let Some(ref mut ws) = *ws_lock {
                        let _ = ws.send(Message::Pong(vec![])).await;
                    }
                }
                Message::Close(_) => {
                    *self.status.lock().await = ConnectionStatus::Disconnected;
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Disconnect
    pub async fn disconnect(&self) -> ExchangeResult<()> {
        let mut ws_lock = self.ws_stream.lock().await;
        if let Some(mut ws) = ws_lock.take() {
            let _ = ws.close(None).await;
        }
        *self.status.lock().await = ConnectionStatus::Disconnected;
        Ok(())
    }

    /// Get connection status
    pub async fn connection_status(&self) -> ConnectionStatus {
        *self.status.lock().await
    }
}

// WebSocketConnector trait implementation would go here if needed
// For now, this is a standalone implementation

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_feed_realtime_urls() {
        assert_eq!(PolygonFeed::Stocks.realtime_url(), "wss://socket.polygon.io/stocks");
        assert_eq!(PolygonFeed::Options.realtime_url(), "wss://socket.polygon.io/options");
        assert_eq!(PolygonFeed::Forex.realtime_url(), "wss://socket.polygon.io/forex");
        assert_eq!(PolygonFeed::Crypto.realtime_url(), "wss://socket.polygon.io/crypto");
    }

    #[test]
    fn test_polygon_feed_delayed_urls() {
        assert_eq!(PolygonFeed::Stocks.delayed_url(), "wss://delayed.polygon.io/stocks");
        assert_eq!(PolygonFeed::Crypto.delayed_url(), "wss://delayed.polygon.io/crypto");
    }

    #[test]
    fn test_channel_to_param() {
        assert_eq!(PolygonChannel::Trades("AAPL".into()).to_param(), "T.AAPL");
        assert_eq!(PolygonChannel::Quotes("aapl".into()).to_param(), "Q.AAPL");
        assert_eq!(PolygonChannel::SecondAggregates("MSFT".into()).to_param(), "A.MSFT");
        assert_eq!(PolygonChannel::MinuteAggregates("TSLA".into()).to_param(), "AM.TSLA");
        assert_eq!(PolygonChannel::ForexQuotes("C:EURUSD".into()).to_param(), "C.C:EURUSD");
        assert_eq!(PolygonChannel::CryptoTrades("X:BTCUSD".into()).to_param(), "XT.X:BTCUSD");
    }

    #[test]
    fn test_build_subscribe_message_single() {
        let msg = PolygonChannel::build_subscribe_message(&[
            PolygonChannel::Trades("AAPL".into()),
        ]);
        assert_eq!(msg["action"], "subscribe");
        assert_eq!(msg["params"], "T.AAPL");
    }

    #[test]
    fn test_build_subscribe_message_multi() {
        let msg = PolygonChannel::build_subscribe_message(&[
            PolygonChannel::Trades("AAPL".into()),
            PolygonChannel::Quotes("AAPL".into()),
            PolygonChannel::MinuteAggregates("AAPL".into()),
        ]);
        assert_eq!(msg["action"], "subscribe");
        assert_eq!(msg["params"], "T.AAPL,Q.AAPL,AM.AAPL");
    }

    #[test]
    fn test_build_unsubscribe_message() {
        let msg = PolygonChannel::build_unsubscribe_message(&[
            PolygonChannel::Trades("AAPL".into()),
        ]);
        assert_eq!(msg["action"], "unsubscribe");
        assert_eq!(msg["params"], "T.AAPL");
    }

    #[test]
    fn test_build_empty_subscribe() {
        let msg = PolygonChannel::build_subscribe_message(&[]);
        assert_eq!(msg["params"], "");
    }
}
