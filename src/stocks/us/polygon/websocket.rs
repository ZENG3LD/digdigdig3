//! # Polygon.io WebSocket Implementation
//!
//! WebSocket connector for Polygon.io real-time and delayed data.
//!
//! ## Features
//! - Real-time and 15-minute delayed streams
//! - Minute/Second aggregates
//! - Trades and Quotes
//! - Simple authentication
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = PolygonWebSocket::new(credentials, true).await?;
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
// Note: WebSocketConnector trait not implemented yet (can be added later if needed)

use super::endpoints::PolygonUrls;
use super::auth::PolygonAuth;
use super::parser::PolygonParser;

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

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
    /// WebSocket stream
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    /// Event broadcast channel
    event_tx: broadcast::Sender<StreamEvent>,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
}

impl PolygonWebSocket {
    /// Create new WebSocket connector
    pub async fn new(credentials: Credentials, realtime: bool) -> ExchangeResult<Self> {
        let auth = PolygonAuth::new(&credentials)?;
        let urls = PolygonUrls::MAINNET;

        // Create broadcast channel for events
        let (event_tx, _) = broadcast::channel(1000);

        Ok(Self {
            auth,
            urls,
            realtime,
            ws_stream: Arc::new(Mutex::new(None)),
            event_tx,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
        })
    }

    /// Connect to WebSocket
    pub async fn connect(&self) -> ExchangeResult<()> {
        let url = self.urls.ws_url(self.realtime);

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
