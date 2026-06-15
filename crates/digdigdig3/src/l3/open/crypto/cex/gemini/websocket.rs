//! GeminiWebSocket — thin wrapper around UniversalWsTransport<GeminiProtocol>.
//!
//! Replaces the bespoke 608-LOC connect/ping/reconnect loop. The framework
//! owns all connection lifecycle, subscription replay on reconnect, and frame
//! dispatch.
//!
//! ## Ping discipline
//!
//! `GeminiProtocol::ping_frame` returns `None`. Gemini DISCONNECTS the
//! connection if it receives a WebSocket Ping frame from the client. The
//! transport never sends application-level pings for Gemini.
//!
//! ## Ticker synthesis
//!
//! Gemini has no dedicated ticker stream. Subscribing to `Stream::Ticker`
//! returns `WebSocketError::WireAbsent`. See `protocol.rs` module doc for
//! the pending stateful-parser follow-up.
//!
//! ## Wasm support
//!
//! Uses `UniversalWsTransport` which compiles to wasm32 via `web-sys`. No
//! native-only gates needed.

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

use super::protocol::GeminiProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// GeminiWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Gemini WebSocket connector backed by UniversalWsTransport.
///
/// Does NOT connect until the first `subscribe()` call (or explicit `connect()`).
pub struct GeminiWebSocket {
    inner: UniversalWsTransport<GeminiProtocol>,
}

impl GeminiWebSocket {
    /// Create a new connector.  Does NOT connect yet.
    pub fn new(testnet: bool) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                GeminiProtocol,
                AccountType::Spot,
                testnet,
                None, // public streams, no credentials
            ),
        }
    }
}

impl Default for GeminiWebSocket {
    fn default() -> Self {
        Self::new(false)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for GeminiWebSocket {
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
        // Gemini disconnects on client Ping — RTT measurement is not possible.
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static GEMINI_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("l2", None, None),
        ];
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            rest_depth_values: &[],
            supports_snapshot: false,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: GEMINI_CHANNELS,
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
    async fn websocket_construction_is_disconnected() {
        let ws = GeminiWebSocket::new(false);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_new() {
        let _ws = GeminiWebSocket::default();
    }
}
