//! # Finnhub WebSocket Implementation
//!
//! WebSocket connector for Finnhub real-time data.
//!
//! ## Endpoint
//! `wss://ws.finnhub.io?token=API_KEY`
//!
//! ## Channels
//! | Channel | Subscribe type | Description |
//! |---------|---------------|-------------|
//! | Trades | `"subscribe"` | Real-time trade executions per symbol |
//! | News | `"subscribe-news"` | Company news stream |
//! | Press Releases | `"subscribe-pr"` | Press release stream |
//!
//! ## Protocol
//! Subscribe: `{"type": "subscribe", "symbol": "AAPL"}`
//! Unsubscribe: `{"type": "unsubscribe", "symbol": "AAPL"}`
//! Receive: `{"type": "trade", "data": [...], "s": "AAPL"}`
//!
//! ## Limitations
//! - 1 WebSocket connection per API key
//! - Free tier: 50 WebSocket symbols max

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

use super::endpoints::FinnhubUrls;
use super::auth::FinnhubAuth;
use super::parser::FinnhubParser;

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// ═══════════════════════════════════════════════════════════════════════════════
// CHANNEL DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Finnhub WebSocket channel descriptor.
///
/// Each variant describes a single subscription action.
/// Finnhub uses per-symbol granularity (no wildcard).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinnhubChannel {
    /// Real-time trade executions (`"subscribe"` / `"unsubscribe"`).
    Trades(String),
    /// Company news stream (`"subscribe-news"` / `"unsubscribe-news"`).
    News(String),
    /// Press release stream (`"subscribe-pr"` / `"unsubscribe-pr"`).
    PressReleases(String),
}

impl FinnhubChannel {
    /// Return the `type` value used in a subscribe message.
    pub fn subscribe_type(&self) -> &'static str {
        match self {
            FinnhubChannel::Trades(_) => "subscribe",
            FinnhubChannel::News(_) => "subscribe-news",
            FinnhubChannel::PressReleases(_) => "subscribe-pr",
        }
    }

    /// Return the `type` value used in an unsubscribe message.
    pub fn unsubscribe_type(&self) -> &'static str {
        match self {
            FinnhubChannel::Trades(_) => "unsubscribe",
            FinnhubChannel::News(_) => "unsubscribe-news",
            FinnhubChannel::PressReleases(_) => "unsubscribe-pr",
        }
    }

    /// Return the symbol associated with this channel.
    pub fn symbol(&self) -> &str {
        match self {
            FinnhubChannel::Trades(s)
            | FinnhubChannel::News(s)
            | FinnhubChannel::PressReleases(s) => s.as_str(),
        }
    }

    /// Build the subscribe JSON message for this channel.
    pub fn subscribe_message(&self) -> serde_json::Value {
        serde_json::json!({
            "type": self.subscribe_type(),
            "symbol": self.symbol().to_uppercase()
        })
    }

    /// Build the unsubscribe JSON message for this channel.
    pub fn unsubscribe_message(&self) -> serde_json::Value {
        serde_json::json!({
            "type": self.unsubscribe_type(),
            "symbol": self.symbol().to_uppercase()
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Finnhub WebSocket connector
pub struct FinnhubWebSocket {
    /// Authentication
    auth: FinnhubAuth,
    /// URLs
    urls: FinnhubUrls,
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Event broadcast channel
    event_tx: broadcast::Sender<StreamEvent>,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
}

impl FinnhubWebSocket {
    /// Create new WebSocket connector
    pub async fn new(credentials: Credentials) -> ExchangeResult<Self> {
        let auth = FinnhubAuth::new(&credentials)?;
        let urls = FinnhubUrls::MAINNET;

        // Create broadcast channel for events
        let (event_tx, _) = broadcast::channel(1000);

        Ok(Self {
            auth,
            urls,
            ws_stream: Arc::new(Mutex::new(None)),
            event_tx,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
        })
    }

    /// Connect to WebSocket
    pub async fn connect(&self) -> ExchangeResult<()> {
        // Finnhub authenticates via URL parameter
        let url = self.auth.ws_url_with_auth(self.urls.websocket_url());

        // Connect
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| ExchangeError::Network(format!("WebSocket connection failed: {}", e)))?;

        *self.ws_stream.lock().await = Some(ws_stream);
        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    /// Subscribe to ticker (trade stream)
    pub async fn subscribe_ticker(&self, symbol: &str) -> ExchangeResult<()> {
        let sub_msg = json!({
            "type": "subscribe",
            "symbol": symbol.to_uppercase()
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Subscribe failed: {}", e)))?;
        }

        Ok(())
    }

    /// Subscribe to company news
    pub async fn subscribe_news(&self, symbol: &str) -> ExchangeResult<()> {
        let sub_msg = json!({
            "type": "subscribe-news",
            "symbol": symbol.to_uppercase()
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Subscribe news failed: {}", e)))?;
        }

        Ok(())
    }

    /// Subscribe to press releases
    pub async fn subscribe_press_releases(&self, symbol: &str) -> ExchangeResult<()> {
        let sub_msg = json!({
            "type": "subscribe-pr",
            "symbol": symbol.to_uppercase()
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(sub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Subscribe press releases failed: {}", e)))?;
        }

        Ok(())
    }

    /// Unsubscribe from ticker
    pub async fn unsubscribe_ticker(&self, symbol: &str) -> ExchangeResult<()> {
        let unsub_msg = json!({
            "type": "unsubscribe",
            "symbol": symbol.to_uppercase()
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(unsub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Unsubscribe failed: {}", e)))?;
        }

        Ok(())
    }

    /// Unsubscribe from news
    pub async fn unsubscribe_news(&self, symbol: &str) -> ExchangeResult<()> {
        let unsub_msg = json!({
            "type": "unsubscribe-news",
            "symbol": symbol.to_uppercase()
        });

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(unsub_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Unsubscribe news failed: {}", e)))?;
        }

        Ok(())
    }

    /// Subscribe using a `FinnhubChannel` descriptor.
    ///
    /// This is the canonical channel-based subscription API.
    ///
    /// # Example
    /// ```ignore
    /// ws.subscribe_channel(&FinnhubChannel::Trades("AAPL".into())).await?;
    /// ws.subscribe_channel(&FinnhubChannel::News("TSLA".into())).await?;
    /// ```
    pub async fn subscribe_channel(&self, channel: &FinnhubChannel) -> ExchangeResult<()> {
        let msg = channel.subscribe_message();

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(msg.to_string()))
                .await
                .map_err(|e| ExchangeError::Network(format!("Subscribe channel failed: {}", e)))?;
        }

        Ok(())
    }

    /// Unsubscribe using a `FinnhubChannel` descriptor.
    pub async fn unsubscribe_channel(&self, channel: &FinnhubChannel) -> ExchangeResult<()> {
        let msg = channel.unsubscribe_message();

        if let Some(ref mut ws) = *self.ws_stream.lock().await {
            ws.send(Message::Text(msg.to_string()))
                .await
                .map_err(|e| {
                    ExchangeError::Network(format!("Unsubscribe channel failed: {}", e))
                })?;
        }

        Ok(())
    }

    /// Subscribe to multiple channels in a single batch.
    ///
    /// Each channel results in a separate WebSocket message (Finnhub does not
    /// support batched subscriptions in a single frame).
    pub async fn subscribe_channels(&self, channels: &[FinnhubChannel]) -> ExchangeResult<()> {
        for channel in channels {
            self.subscribe_channel(channel).await?;
        }
        Ok(())
    }

    /// Unsubscribe from multiple channels.
    pub async fn unsubscribe_channels(&self, channels: &[FinnhubChannel]) -> ExchangeResult<()> {
        for channel in channels {
            self.unsubscribe_channel(channel).await?;
        }
        Ok(())
    }

    /// Disconnect from WebSocket
    pub async fn disconnect(&self) -> ExchangeResult<()> {
        if let Some(mut ws) = self.ws_stream.lock().await.take() {
            ws.close(None)
                .await
                .map_err(|e| ExchangeError::Network(format!("Disconnect failed: {}", e)))?;
        }

        *self.status.lock().await = ConnectionStatus::Disconnected;
        Ok(())
    }

    /// Get connection status
    pub async fn status(&self) -> ConnectionStatus {
        *self.status.lock().await
    }

    /// Get event stream receiver
    pub fn event_stream(&self) -> broadcast::Receiver<StreamEvent> {
        self.event_tx.subscribe()
    }

    /// Start receiving messages (run in background task)
    pub async fn start_receiving(&self) -> ExchangeResult<()> {
        loop {
            let msg = {
                let mut ws_guard = self.ws_stream.lock().await;
                if let Some(ref mut ws) = *ws_guard {
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
                    return Err(ExchangeError::Network("WebSocket not connected".to_string()));
                }
            };

            match msg {
                Message::Text(text) => {
                    // Parse message
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json_msg) => {
                            match FinnhubParser::parse_ws_message(&json_msg) {
                                Ok(events) => {
                                    for event in events {
                                        // Broadcast event (ignore send errors if no receivers)
                                        let _ = self.event_tx.send(event);
                                    }
                                }
                                Err(e) => {
                                    // Log parse error but continue
                                    eprintln!("Failed to parse WebSocket message: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON: {}", e);
                        }
                    }
                }
                Message::Ping(data) => {
                    // Respond to ping with pong
                    let mut ws_guard = self.ws_stream.lock().await;
                    if let Some(ref mut ws) = *ws_guard {
                        let _ = ws.send(Message::Pong(data)).await;
                    }
                }
                Message::Close(_) => {
                    *self.status.lock().await = ConnectionStatus::Disconnected;
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let credentials = Credentials::new("test_api_key", "");
        let ws = FinnhubWebSocket::new(credentials).await;
        assert!(ws.is_ok());
    }

    #[test]
    fn test_subscribe_message_format() {
        let msg = json!({
            "type": "subscribe",
            "symbol": "AAPL"
        });

        assert_eq!(msg["type"], "subscribe");
        assert_eq!(msg["symbol"], "AAPL");
    }

    #[test]
    fn test_unsubscribe_message_format() {
        let msg = json!({
            "type": "unsubscribe",
            "symbol": "AAPL"
        });

        assert_eq!(msg["type"], "unsubscribe");
        assert_eq!(msg["symbol"], "AAPL");
    }

    #[test]
    fn test_channel_trades_subscribe() {
        let ch = FinnhubChannel::Trades("AAPL".into());
        let msg = ch.subscribe_message();
        assert_eq!(msg["type"], "subscribe");
        assert_eq!(msg["symbol"], "AAPL");
    }

    #[test]
    fn test_channel_trades_unsubscribe() {
        let ch = FinnhubChannel::Trades("AAPL".into());
        let msg = ch.unsubscribe_message();
        assert_eq!(msg["type"], "unsubscribe");
        assert_eq!(msg["symbol"], "AAPL");
    }

    #[test]
    fn test_channel_news_subscribe() {
        let ch = FinnhubChannel::News("TSLA".into());
        let msg = ch.subscribe_message();
        assert_eq!(msg["type"], "subscribe-news");
        assert_eq!(msg["symbol"], "TSLA");
    }

    #[test]
    fn test_channel_press_releases_subscribe() {
        let ch = FinnhubChannel::PressReleases("MSFT".into());
        let msg = ch.subscribe_message();
        assert_eq!(msg["type"], "subscribe-pr");
        assert_eq!(msg["symbol"], "MSFT");
    }

    #[test]
    fn test_channel_symbol() {
        assert_eq!(FinnhubChannel::Trades("AAPL".into()).symbol(), "AAPL");
        assert_eq!(FinnhubChannel::News("TSLA".into()).symbol(), "TSLA");
    }
}
