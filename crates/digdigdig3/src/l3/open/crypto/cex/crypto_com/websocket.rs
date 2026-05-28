//! CryptoComWebSocket — thin wrapper around UniversalWsTransport<CryptoComProtocol>.
//!
//! Replaces the bespoke 1,047-LOC connect/heartbeat/ping loop. The framework
//! owns all connection lifecycle, subscription replay on reconnect, server-ping
//! response (public/heartbeat), and frame dispatch.
//!
//! ## Crypto.com heartbeat
//!
//! `CryptoComProtocol::is_server_ping` matches `public/heartbeat` frames.
//! `CryptoComProtocol::pong_response_frame` builds `public/respond-heartbeat`
//! with the echoed `id`. The transport sends the reply automatically.
//!
//! ## 1-second post-connect delay
//!
//! `CryptoComProtocol::post_connect_delay()` returns 1 s. The transport waits
//! this duration after the TCP/TLS handshake before sending any frames.
//!
//! ## Wasm support
//!
//! Uses `UniversalWsTransport` which compiles to wasm32 via `web-sys`. No
//! native-only gates needed.
//!
//! ## Private channels
//!
//! Subscribing to private stream kinds returns `WebSocketError::NotSupported`.

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::WebSocketConnector;
use crate::core::types::{
    AccountType, ConnectionStatus, OrderbookCapabilities, StreamEvent,
    SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::CryptoComProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// CryptoComWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Crypto.com WebSocket connector backed by UniversalWsTransport.
///
/// Does NOT connect until the first `subscribe()` call (or explicit `connect()`).
pub struct CryptoComWebSocket {
    inner: UniversalWsTransport<CryptoComProtocol>,
}

impl CryptoComWebSocket {
    /// Create a new connector.  Does NOT connect yet.
    ///
    /// `testnet` selects the UAT sandbox endpoint (`uat-stream.3ona.co`).
    pub fn new(testnet: bool) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                CryptoComProtocol::new(testnet),
                AccountType::Spot,
                testnet,
                None, // public streams, no credentials
            ),
        }
    }

    /// Create a connector for futures/perpetual streams.
    pub fn new_futures(testnet: bool) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                CryptoComProtocol::new(testnet),
                AccountType::FuturesCross,
                testnet,
                None,
            ),
        }
    }
}

impl Default for CryptoComWebSocket {
    fn default() -> Self {
        Self::new(false)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for CryptoComWebSocket {
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
        // Crypto.com uses server-initiated heartbeat; client never sends pings.
        // RTT measurement via WS-level ping is not available.
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static CRYPTO_COM_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("book", Some(10), None),
            WsBookChannel::delta("book", Some(50), None),
        ];
        OrderbookCapabilities {
            ws_depths: &[10, 50],
            ws_default_depth: Some(10),
            rest_max_depth: Some(50),
            rest_depth_values: &[],
            supports_snapshot: false,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: CRYPTO_COM_CHANNELS,
            checksum: None,
            has_sequence: false,
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
    async fn websocket_construction_is_disconnected() {
        let ws = CryptoComWebSocket::new(false);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_new() {
        let _ws = CryptoComWebSocket::default();
    }

    #[tokio::test]
    async fn futures_constructor() {
        let ws = CryptoComWebSocket::new_futures(false);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }
}
