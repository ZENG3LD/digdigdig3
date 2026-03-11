//! Whale Alert WebSocket connector
//!
//! Provides real-time transaction alerts via WebSocket.
//!
//! ## Alert Types
//! - Transaction Alerts (large blockchain transactions)
//! - Social Alerts (Whale Alert Twitter/Telegram posts)
//!
//! ## Subscription Requirements
//! - Minimum value: $100,000 USD
//! - Authentication: API key in connection URL
//! - Rate limits: 100 alerts/hour (Custom), 10,000/hour (Priority)

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

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
    /// Create new WebSocket connector
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

    /// Subscribe to transaction alerts with filters
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
                "min_value_usd must be at least $100,000".to_string()
            ));
        }

        let subscription = WhaleAlertSubscription::Alerts {
            blockchains,
            symbols,
            tx_types,
            min_value_usd,
            channel_id,
        };

        // TODO: Send subscription message via WebSocket
        // For now, this is a placeholder implementation
        Err(WebSocketError::UnsupportedOperation(
            "Whale Alert WebSocket support is not yet fully implemented".to_string()
        ))
    }

    /// Subscribe to social media alerts
    pub async fn subscribe_socials(&mut self, channel_id: Option<String>) -> WebSocketResult<()> {
        let _subscription = WhaleAlertSubscription::Socials { channel_id };

        // TODO: Send subscription message via WebSocket
        Err(WebSocketError::UnsupportedOperation(
            "Whale Alert WebSocket support is not yet fully implemented".to_string()
        ))
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
