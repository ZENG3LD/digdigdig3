//! # Finnhub WebSocket Implementation
//!
//! WebSocket connector for Finnhub real-time data.
//!
//! ## Features
//! - Real-time trade executions
//! - Company news streaming
//! - Press release streaming
//! - Simple authentication via URL token
//!
//! ## Limitations
//! - 1 WebSocket connection per API key
//! - Free tier: 50 WebSocket symbols max
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = FinnhubWebSocket::new(credentials).await?;
//! ws.connect().await?;
//! ws.subscribe_ticker("AAPL").await?;
//!
//! let stream = ws.event_stream();
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(StreamEvent::Ticker(ticker)) => println!("{:?}", ticker),
//!         _ => {}
//!     }
//! }
//! ```

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
}
