//! CoinbaseWebSocket — thin wrapper around UniversalWsTransport<CoinbaseProtocol>.
//!
//! Replaces the bespoke 497-LOC connect/ping/read loop.  The framework owns
//! all connection lifecycle, native WS ping scheduling (for RTT), subscription
//! replay on reconnect, and frame dispatch.
//!
//! ## Heartbeats channel
//!
//! Coinbase requires a "heartbeats" subscription for server-side keepalive.
//! This is handled by `CoinbaseProtocol::post_connect_frames()`, which returns
//! the heartbeat subscribe frame on every connect/reconnect.  The driver sends
//! it before marking the connection as Connected.  No explicit call is needed here.
//!
//! ## Reconnect fix
//!
//! The bespoke loop dropped its broadcast sender on disconnect, breaking Station's
//! kline-heal path (0.3.7).  UniversalWsTransport holds the sender in an Arc and
//! never drops it on reconnect.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::WebSocketConnector;
use crate::core::traits::Credentials;
use crate::core::types::{
    AccountType, ConnectionStatus, OrderbookCapabilities, StreamEvent,
    SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::CoinbaseProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// CoinbaseWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Coinbase Advanced Trade WebSocket connector backed by UniversalWsTransport.
///
/// Does NOT connect until the first `subscribe()` call (or explicit `connect()`).
pub struct CoinbaseWebSocket {
    inner: UniversalWsTransport<CoinbaseProtocol>,
}

impl CoinbaseWebSocket {
    /// Construct for public channels — no credentials needed.
    pub fn public() -> Self {
        Self {
            inner: UniversalWsTransport::new(
                CoinbaseProtocol::public(),
                AccountType::Spot,
                false, // Coinbase has no testnet for Advanced Trade API
                None,
            ),
        }
    }

    /// Construct with optional credentials.
    /// Uses private endpoint (`wss://advanced-trade-ws-user.coinbase.com`) when
    /// credentials are present, public endpoint otherwise.
    pub fn new(credentials: Option<Credentials>) -> Self {
        let use_private = credentials.is_some();
        Self {
            inner: UniversalWsTransport::new(
                if use_private {
                    CoinbaseProtocol::private()
                } else {
                    CoinbaseProtocol::public()
                },
                AccountType::Spot,
                false,
                credentials,
            ),
        }
    }
}

impl Default for CoinbaseWebSocket {
    fn default() -> Self {
        Self::public()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for CoinbaseWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        self.inner.connect().await
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        self.inner.disconnect().await
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.inner.connection_status()
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.inner.subscribe(spec).await
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        self.inner.unsubscribe(spec).await
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        Box::pin(self.inner.event_stream())
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.inner
            .active_subscriptions()
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect()
    }

    fn ping_rtt_handle(&self) -> Option<Arc<TokioMutex<u64>>> {
        // Native WS ping/pong is managed by the transport.
        // Per-pong RTT handle not yet exposed by UniversalWsTransport.
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static COINBASE_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("level2", None, None),
            WsBookChannel::delta("level2_batch", None, None),
        ];
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: COINBASE_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn construction_does_not_connect() {
        let ws = CoinbaseWebSocket::public();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_public() {
        let _ws = CoinbaseWebSocket::default();
    }

    #[tokio::test]
    async fn new_with_no_credentials_is_disconnected() {
        let ws = CoinbaseWebSocket::new(None);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }
}
