//! # Bitquery WebSocket Connector
//!
//! GraphQL subscription support for real-time blockchain data.
//!
//! ## Protocol
//!
//! Bitquery uses GraphQL subscriptions over WebSocket:
//! - Protocol: `graphql-transport-ws` (modern) or `graphql-ws` (legacy)
//! - Authentication: Token in URL parameter
//! - Keepalive: Server sends pong/ka messages
//!
//! ## Subscription Types
//!
//! All GraphQL queries can be converted to subscriptions:
//! - Blocks (real-time block updates)
//! - Transactions (real-time transactions)
//! - DEX Trades (real-time DEX trading)
//! - Token Transfers (real-time transfers)
//! - Mempool (pending transactions)
//!
//! ## Cost
//!
//! - Free tier: 2 simultaneous streams
//! - Commercial: Unlimited streams
//! - Billing: 40 points/minute per stream

use super::auth::BitqueryAuth;
use super::endpoints::BitqueryUrls;

/// WebSocket connector for Bitquery
///
/// NOTE: WebSocket implementation is currently a stub.
/// Use HTTP queries via BitqueryConnector for now.
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

    /// Get WebSocket URL with authentication
    pub fn get_ws_url(&self) -> String {
        self.auth.get_ws_url(self.urls.websocket)
    }
}

// NOTE: Full WebSocket implementation requires:
// - tokio-tungstenite for WebSocket client
// - graphql-ws protocol implementation
// - Message parsing for GraphQL subscription protocol
// - Reconnection logic
// - Keepalive handling
//
// For now, users can use HTTP queries. WebSocket can be added later if needed.
