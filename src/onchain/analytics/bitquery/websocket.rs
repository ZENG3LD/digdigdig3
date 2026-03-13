//! # Bitquery WebSocket Connector
//!
//! GraphQL subscription support for real-time blockchain data.
//!
//! ## Protocol
//!
//! Bitquery uses GraphQL subscriptions over WebSocket:
//! - Protocol: `graphql-transport-ws` (modern) or `graphql-ws` (legacy)
//! - Authentication: Token in URL parameter (`?token=ory_at_...`)
//! - Keepalive: Server sends `connection_ack` and ping/pong messages
//!
//! ## Subscription Types
//!
//! - `subscribe_blocks` — Real-time block updates
//! - `subscribe_dex_trades` — Real-time DEX trades
//!
//! ## Cost
//!
//! - Free tier: 2 simultaneous streams
//! - Commercial: Unlimited streams
//! - Billing: 40 points/minute per stream
//!
//! ## Usage
//!
//! ```ignore
//! let ws = BitqueryWebSocket::new(auth);
//! let mut stream = ws.subscribe_blocks("eth").await?;
//! while let Some(msg) = stream.recv().await {
//!     println!("{:?}", msg);
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use super::auth::BitqueryAuth;
use super::endpoints::{BitqueryUrls, _build_blocks_subscription, _build_dex_trades_subscription};

// ═══════════════════════════════════════════════════════════════════════════════
// GRAPHQL-WS PROTOCOL MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Message sent from client to server (graphql-transport-ws protocol)
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    /// Initialize the connection
    ConnectionInit {
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// Start a subscription
    Subscribe {
        id: String,
        payload: SubscribePayload,
    },
    /// Terminate a subscription
    Complete { id: String },
}

/// Subscription payload
#[derive(Debug, Serialize)]
struct SubscribePayload {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_name: Option<String>,
}

/// Message received from server (graphql-transport-ws protocol)
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    /// Server acknowledged connection
    ConnectionAck,
    /// Subscription data event
    Next { id: String, payload: Value },
    /// Subscription completed
    Complete { id: String },
    /// Error in subscription
    Error { id: String, payload: Value },
    /// Server ping
    Ping,
    /// Server pong
    Pong,
    /// Connection keep-alive (legacy graphql-ws protocol)
    #[serde(rename = "ka")]
    KeepAlive,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION MESSAGE (public type)
// ═══════════════════════════════════════════════════════════════════════════════

/// A message received from a GraphQL subscription
#[derive(Debug, Clone)]
pub enum SubscriptionMessage {
    /// Data payload from the subscription
    Data(Value),
    /// Subscription completed (server-side)
    Complete,
    /// Error payload
    Error(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// WebSocket connector for Bitquery GraphQL subscriptions
///
/// Implements the `graphql-transport-ws` protocol to receive real-time
/// blockchain data via GraphQL subscriptions.
pub struct BitqueryWebSocket {
    auth: BitqueryAuth,
    urls: BitqueryUrls,
}

impl BitqueryWebSocket {
    /// Create new WebSocket connector
    pub fn new(auth: BitqueryAuth) -> Self {
        Self {
            auth,
            urls: BitqueryUrls::default(),
        }
    }

    /// Get WebSocket URL with authentication token embedded as query parameter
    pub fn get_ws_url(&self) -> String {
        self.auth.get_ws_url(self.urls.websocket)
    }

    /// Subscribe to real-time block data for a blockchain network
    ///
    /// # Arguments
    /// - `network` - Blockchain network identifier (e.g., "eth", "bsc", "solana")
    ///
    /// # Returns
    /// An mpsc receiver that yields `SubscriptionMessage` for each new block.
    ///
    /// # Example
    /// ```ignore
    /// let ws = BitqueryWebSocket::new(auth);
    /// let mut rx = ws.subscribe_blocks("eth").await?;
    /// while let Some(msg) = rx.recv().await {
    ///     if let SubscriptionMessage::Data(data) = msg {
    ///         println!("New block: {}", data);
    ///     }
    /// }
    /// ```
    pub async fn subscribe_blocks(
        &self,
        network: &str,
    ) -> Result<mpsc::Receiver<SubscriptionMessage>, String> {
        let query = _build_blocks_subscription(network);
        let sub_id = format!("blocks_{}", network);
        self.subscribe_with_query(sub_id, query).await
    }

    /// Subscribe to real-time DEX trade data for a blockchain network
    ///
    /// # Arguments
    /// - `network` - Blockchain network identifier (e.g., "eth", "bsc", "polygon")
    /// - `protocol` - Optional DEX protocol filter (e.g., "uniswap_v2", "pancakeswap")
    ///
    /// # Returns
    /// An mpsc receiver that yields `SubscriptionMessage` for each new DEX trade.
    ///
    /// # Example
    /// ```ignore
    /// let ws = BitqueryWebSocket::new(auth);
    /// // All DEX trades on Ethereum
    /// let mut rx = ws.subscribe_dex_trades("eth", None).await?;
    /// // Only Uniswap V2 trades
    /// let mut rx = ws.subscribe_dex_trades("eth", Some("uniswap_v2")).await?;
    /// ```
    pub async fn subscribe_dex_trades(
        &self,
        network: &str,
        protocol: Option<&str>,
    ) -> Result<mpsc::Receiver<SubscriptionMessage>, String> {
        let query = _build_dex_trades_subscription(network, protocol);
        let sub_id = format!("dex_trades_{}", network);
        self.subscribe_with_query(sub_id, query).await
    }

    /// Internal: open a WebSocket connection and start a named subscription
    ///
    /// Performs the full graphql-transport-ws handshake:
    /// 1. Connect to WebSocket endpoint with auth token
    /// 2. Send `connection_init`
    /// 3. Wait for `connection_ack`
    /// 4. Send `subscribe` with the GraphQL subscription query
    /// 5. Forward incoming `next` messages to the returned channel
    async fn subscribe_with_query(
        &self,
        sub_id: String,
        query: String,
    ) -> Result<mpsc::Receiver<SubscriptionMessage>, String> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::{connect_async, tungstenite::Message};

        let ws_url = self.get_ws_url();
        debug!("Bitquery WS: connecting to {}", ws_url);

        let (ws_stream, _response) = connect_async(&ws_url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {}", e))?;

        let (mut write, mut read) = ws_stream.split();

        // Step 1: Send connection_init
        let init_msg = ClientMessage::ConnectionInit { payload: None };
        let init_json = serde_json::to_string(&init_msg)
            .map_err(|e| format!("Serialization error: {}", e))?;
        write
            .send(Message::Text(init_json.into()))
            .await
            .map_err(|e| format!("Failed to send connection_init: {}", e))?;

        // Step 2: Wait for connection_ack
        let ack_timeout = tokio::time::Duration::from_secs(10);
        tokio::time::timeout(ack_timeout, async {
            loop {
                match read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(msg) = serde_json::from_str::<ServerMessage>(&text) {
                            match msg {
                                ServerMessage::ConnectionAck => return Ok(()),
                                ServerMessage::KeepAlive => continue,
                                other => {
                                    warn!("Bitquery WS: unexpected message before ack: {:?}", other);
                                }
                            }
                        }
                    }
                    Some(Err(e)) => return Err(format!("WebSocket error waiting for ack: {}", e)),
                    None => return Err("Connection closed before ack".to_string()),
                    _ => {}
                }
            }
        })
        .await
        .map_err(|_| "Timeout waiting for connection_ack".to_string())??;

        // Step 3: Send subscribe message
        let subscribe_msg = ClientMessage::Subscribe {
            id: sub_id.clone(),
            payload: SubscribePayload {
                query,
                variables: None,
                operation_name: None,
            },
        };
        let subscribe_json = serde_json::to_string(&subscribe_msg)
            .map_err(|e| format!("Serialization error: {}", e))?;
        write
            .send(Message::Text(subscribe_json.into()))
            .await
            .map_err(|e| format!("Failed to send subscribe: {}", e))?;

        // Step 4: Spawn background task to forward messages
        let (tx, rx) = mpsc::channel::<SubscriptionMessage>(256);
        let sub_id_clone = sub_id.clone();

        tokio::spawn(async move {
            loop {
                match read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(ServerMessage::Next { id, payload }) if id == sub_id_clone => {
                                if tx.send(SubscriptionMessage::Data(payload)).await.is_err() {
                                    debug!("Bitquery WS: receiver dropped, stopping subscription");
                                    break;
                                }
                            }
                            Ok(ServerMessage::Complete { id }) if id == sub_id_clone => {
                                let _ = tx.send(SubscriptionMessage::Complete).await;
                                break;
                            }
                            Ok(ServerMessage::Error { id, payload }) if id == sub_id_clone => {
                                let error_msg = payload.to_string();
                                let _ = tx.send(SubscriptionMessage::Error(error_msg)).await;
                                break;
                            }
                            Ok(ServerMessage::Ping) => {
                                // Respond with pong
                                let pong = ClientMessage::ConnectionInit { payload: None };
                                if let Ok(json) = serde_json::to_string(&pong) {
                                    let _ = write.send(Message::Text(json.into())).await;
                                }
                            }
                            Ok(ServerMessage::KeepAlive) | Ok(ServerMessage::Pong) => {
                                // Ignore keep-alive and pong
                            }
                            Ok(_) => {
                                // Message for different subscription or unknown type
                            }
                            Err(e) => {
                                warn!("Bitquery WS: failed to parse message: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = write.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        let _ = tx
                            .send(SubscriptionMessage::Error("Connection closed".to_string()))
                            .await;
                        break;
                    }
                    Some(Err(e)) => {
                        let _ = tx
                            .send(SubscriptionMessage::Error(format!("WS error: {}", e)))
                            .await;
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(rx)
    }
}
