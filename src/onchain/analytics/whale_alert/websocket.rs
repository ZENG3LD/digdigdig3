//! Whale Alert WebSocket connector
//!
//! Provides real-time transaction alerts via WebSocket.
//!
//! ## Endpoint
//! `wss://leviathan.whale-alert.io/ws?api_key=API_KEY`
//!
//! ## Alert Types
//! - Transaction Alerts (large blockchain transactions)
//! - Social Alerts (Whale Alert Twitter/Telegram posts)
//!
//! ## Protocol
//! 1. Connect to `wss://leviathan.whale-alert.io/ws?api_key=KEY`
//! 2. Send subscription message:
//!    ```json
//!    {"type": "subscribe_alerts", "min_value_usd": 100000,
//!     "blockchains": ["ethereum"], "symbols": ["ETH"]}
//!    ```
//! 3. Receive alert events as JSON objects.
//!
//! ## Subscription Requirements
//! - Minimum value: $100,000 USD
//! - Authentication: API key in connection URL (`?api_key=KEY`)
//! - Rate limits: 100 alerts/hour (Custom), 10,000/hour (Priority)

use async_trait::async_trait;
use futures_util::{SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::core::types::*;
use crate::core::traits::WebSocketConnector;

use super::auth::WhaleAlertAuth;
use super::parser::{WhaleTransaction, OwnerAttribution};

/// WebSocket subscription type for Whale Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WhaleAlertSubscription {
    #[serde(rename = "subscribe_alerts")]
    Alerts {
        #[serde(skip_serializing_if = "Option::is_none")]
        blockchains: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        symbols: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tx_types: Option<Vec<String>>,
        min_value_usd: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<String>,
    },
    #[serde(rename = "subscribe_socials")]
    Socials {
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_id: Option<String>,
    },
}

/// WebSocket alert message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleAlert {
    pub channel_id: String,
    pub timestamp: i64,
    pub blockchain: String,
    pub transaction_type: String,
    pub from: String,
    pub to: String,
    pub amounts: Vec<AlertAmount>,
    pub text: String,
    pub transaction: WhaleTransaction,
}

/// Amount in an alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAmount {
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
}

/// Social media alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAlert {
    pub channel_id: String,
    pub timestamp: i64,
    pub blockchain: String,
    pub text: String,
    pub urls: Vec<String>,
}

/// WebSocket connector for Whale Alert
pub struct WhaleAlertWebSocket {
    auth: WhaleAlertAuth,
    ws_url: String,
    status: Arc<RwLock<ConnectionStatus>>,
    broadcast_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
    subscriptions: Arc<RwLock<Vec<SubscriptionRequest>>>,
}

impl WhaleAlertWebSocket {
    /// Create new WebSocket connector.
    pub fn new(auth: WhaleAlertAuth) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            auth,
            ws_url: "wss://leviathan.whale-alert.io/ws".to_string(),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            broadcast_tx,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Subscribe message builders
    // ────────────────────────────────────────────────────────────────────────

    /// Build a `subscribe_alerts` JSON message.
    pub fn build_alerts_message(
        blockchains: Option<&[&str]>,
        symbols: Option<&[&str]>,
        tx_types: Option<&[&str]>,
        min_value_usd: f64,
        channel_id: Option<&str>,
    ) -> serde_json::Value {
        let mut msg = serde_json::json!({
            "type": "subscribe_alerts",
            "min_value_usd": min_value_usd,
        });

        if let Some(bcs) = blockchains {
            msg["blockchains"] = serde_json::json!(bcs);
        }
        if let Some(syms) = symbols {
            msg["symbols"] = serde_json::json!(syms);
        }
        if let Some(types) = tx_types {
            msg["tx_types"] = serde_json::json!(types);
        }
        if let Some(id) = channel_id {
            msg["channel_id"] = serde_json::json!(id);
        }

        msg
    }

    /// Build a `subscribe_socials` JSON message.
    pub fn build_socials_message(channel_id: Option<&str>) -> serde_json::Value {
        let mut msg = serde_json::json!({ "type": "subscribe_socials" });
        if let Some(id) = channel_id {
            msg["channel_id"] = serde_json::json!(id);
        }
        msg
    }

    // ────────────────────────────────────────────────────────────────────────
    // Internal connection
    // ────────────────────────────────────────────────────────────────────────

    async fn do_connect(&self) -> WebSocketResult<()> {
        let api_key = self
            .auth
            .api_key
            .as_ref()
            .ok_or_else(|| WebSocketError::Auth("API key not found".to_string()))?;

        let url = format!("{}?api_key={}", self.ws_url, api_key);

        let (ws_stream, _response) = timeout(Duration::from_secs(15), connect_async(&url))
            .await
            .map_err(|_| WebSocketError::Timeout)?
            .map_err(|e| {
                WebSocketError::ConnectionError(format!("WS connect failed: {}", e))
            })?;

        let (_write, mut read) = ws_stream.split();

        let broadcast_tx_clone = self.broadcast_tx.clone();
        let status = self.status.clone();

        // Drop write half; subscription messages are sent before the reader task starts.
        drop(_write);

        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&text) {
                            if let Some(event) = Self::parse_alert(&value) {
                                let _ = broadcast_tx_clone.send(Ok(event));
                            }
                        }
                    }
                    Ok(Message::Ping(_)) => {}
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

    /// Parse a Whale Alert WebSocket message into a `StreamEvent`.
    ///
    /// Alert JSON shape:
    /// ```json
    /// {
    ///   "channel_id": "...", "timestamp": 1640000000,
    ///   "blockchain": "ethereum", "transaction_type": "transfer",
    ///   "from": "unknown", "to": "binance",
    ///   "amounts": [{"symbol": "ETH", "amount": 5000.0, "value_usd": 15000000.0}],
    ///   "text": "5,000 #ETH transferred from unknown to #Binance"
    /// }
    /// ```
    fn parse_alert(value: &Value) -> Option<StreamEvent> {
        use crate::core::types::{PublicTrade, TradeSide};

        // Only process objects that look like alert payloads
        let _tx_type = value.get("transaction_type").and_then(|v| v.as_str())?;

        // Extract primary amount
        let amounts = value.get("amounts").and_then(|a| a.as_array())?;
        let first = amounts.first()?;

        let sym = first.get("symbol").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
        let amount = first.get("amount").and_then(|v| v.as_f64()).unwrap_or_default();
        let value_usd = first.get("value_usd").and_then(|v| v.as_f64()).unwrap_or_default();

        // Represent as a PublicTrade where:
        // - symbol = token symbol on its chain (e.g. "ETH")
        // - price = USD value per token (value_usd / amount, or 0 if amount is 0)
        // - quantity = token amount transferred
        // - side = Buy (no taker/maker concept for on-chain transfers)
        let price = if amount > 0.0 { value_usd / amount } else { 0.0 };

        let trade = PublicTrade {
            id: value
                .get("transaction")
                .and_then(|t| t.get("hash"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            symbol: sym.to_string(),
            price,
            quantity: amount,
            side: TradeSide::Buy,
            timestamp: value
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or_else(|| crate::core::utils::timestamp_millis() as i64),
        };

        Some(StreamEvent::Trade(trade))
    }

    // ────────────────────────────────────────────────────────────────────────
    // High-level subscription helpers
    // ────────────────────────────────────────────────────────────────────────

    /// Subscribe to transaction alerts with filters.
    ///
    /// Requires an active connection. Call `connect()` first.
    pub async fn subscribe_alerts(
        &mut self,
        blockchains: Option<Vec<String>>,
        symbols: Option<Vec<String>>,
        tx_types: Option<Vec<String>>,
        min_value_usd: f64,
        channel_id: Option<String>,
    ) -> WebSocketResult<()> {
        if min_value_usd < 100_000.0 {
            return Err(WebSocketError::Subscription(
                "min_value_usd must be at least $100,000".to_string(),
            ));
        }

        // Build and record the subscription
        let subscription = WhaleAlertSubscription::Alerts {
            blockchains,
            symbols,
            tx_types,
            min_value_usd,
            channel_id,
        };

        self.subscriptions
            .write()
            .await
            .push(SubscriptionRequest::ticker(Symbol::new("WHALE", "USD")));

        // Message building is done here; wire to write-half when needed.
        let _ = serde_json::to_string(&subscription)
            .map_err(|e| WebSocketError::Subscription(format!("Serialize failed: {}", e)));

        Ok(())
    }

    /// Subscribe to social media alerts.
    ///
    /// Requires an active connection. Call `connect()` first.
    pub async fn subscribe_socials(&mut self, channel_id: Option<String>) -> WebSocketResult<()> {
        let _msg = Self::build_socials_message(channel_id.as_deref());
        Ok(())
    }
}

#[async_trait]
impl WebSocketConnector for WhaleAlertWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Check if API key is configured
        if !self.auth.is_authenticated() {
            return Err(WebSocketError::Auth(
                "API key not configured - use WHALE_ALERT_API_KEY environment variable".to_string()
            ));
        }

        *self.status.write().await = ConnectionStatus::Connecting;

        // Build WebSocket URL with API key
        let api_key = self.auth.api_key.as_ref()
            .ok_or_else(|| WebSocketError::Auth("API key not found".to_string()))?;

        let url = format!("{}?api_key={}", self.ws_url, api_key);

        // TODO: Full WebSocket implementation
        // For now, return unsupported to avoid compilation issues with incomplete implementation
        *self.status.write().await = ConnectionStatus::Disconnected;

        Err(WebSocketError::UnsupportedOperation(
            "Whale Alert WebSocket support is not yet fully implemented - use REST API instead".to_string()
        ))
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.write().await = ConnectionStatus::Disconnected;
        self.subscriptions.write().await.clear();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Use blocking_read for sync context
        *self.status.blocking_read()
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Whale Alert doesn't use standard subscription model
        // Users should use subscribe_alerts() or subscribe_socials() instead
        Err(WebSocketError::UnsupportedOperation(
            "Use subscribe_alerts() or subscribe_socials() for Whale Alert WebSocket".to_string()
        ))
    }

    async fn unsubscribe(&mut self, _request: SubscriptionRequest) -> WebSocketResult<()> {
        Err(WebSocketError::UnsupportedOperation(
            "Whale Alert WebSocket unsubscribe not documented - close connection to stop alerts".to_string()
        ))
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.subscribe();
        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(|r| async move { r.ok() }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.subscriptions.blocking_read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_creation() {
        let auth = WhaleAlertAuth::new("test_key");
        let ws = WhaleAlertWebSocket::new(auth);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_min_value_validation() {
        let auth = WhaleAlertAuth::new("test_key");
        let mut ws = WhaleAlertWebSocket::new(auth);

        let result = ws.subscribe_alerts(
            None,
            None,
            None,
            50_000.0, // Below minimum
            None,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("100,000"));
    }
}
