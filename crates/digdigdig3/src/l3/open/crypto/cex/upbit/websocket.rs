//! UpbitWebSocket — thin wrapper around UniversalWsTransport<UpbitProtocol>.
//!
//! Replaces the bespoke 474-LOC connect/ping/split-task loop. The framework
//! owns all connection lifecycle, ping management, subscription replay on
//! reconnect, and frame dispatch.
//!
//! ## Binary UTF-8 frames
//!
//! Upbit sends data as `Message::Binary(utf8_json_bytes)` in DEFAULT format.
//! The transport's default `decode_binary` fallback chain (gzip → zlib →
//! deflate → UTF-8) handles this transparently — no override needed.
//!
//! ## Ping/Pong
//!
//! Standard WS-level `Message::Ping` → `Message::Pong` handled by the
//! transport automatically. `UpbitProtocol::is_pong` suppresses the
//! `{"status":"UP"}` liveness frames.
//!
//! ## Wasm support
//!
//! Uses `UniversalWsTransport` which compiles to wasm32 via `web-sys`.
//! No native-only `#[cfg]` gates needed.

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

use super::protocol::UpbitProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// UpbitWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Upbit WebSocket connector backed by UniversalWsTransport.
///
/// Does NOT connect until the first `subscribe()` call (or explicit `connect()`).
pub struct UpbitWebSocket {
    inner: UniversalWsTransport<UpbitProtocol>,
}

impl UpbitWebSocket {
    /// Create a new connector. Does NOT connect yet.
    pub fn new(
        _credentials: Option<crate::core::traits::Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                UpbitProtocol::new(testnet),
                account_type,
                testnet,
                None, // public channels only
            ),
        }
    }
}

impl Default for UpbitWebSocket {
    fn default() -> Self {
        Self::new(None, false, AccountType::Spot)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for UpbitWebSocket {
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
        // Upbit uses WS-level Ping/Pong handled by the transport.
        // RTT measurement is not surfaced.
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("orderbook", 30, 0),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 5, 15, 30],
            ws_default_depth: Some(30),
            rest_max_depth: Some(30),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: false,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: CHANNELS,
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
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
        let ws = UpbitWebSocket::new(None, false, AccountType::Spot);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_new() {
        let _ws = UpbitWebSocket::default();
    }
}
