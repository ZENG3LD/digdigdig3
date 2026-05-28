//! DydxWebSocket — thin wrapper around UniversalWsTransport<DydxProtocol>.
//!
//! ## Public data only
//!
//! dYdX v4 public channels (orderbook, trades, candles, markets) require
//! zero authentication. Private channels (subaccounts, positions) are
//! NOT supported through this connector by design — they are native-only
//! and require wallet signing.
//!
//! ## Wasm support
//!
//! `DydxProtocol` passes the standard WS transport requirements (text frames,
//! no binary frames, standard Ping/Pong). `UniversalWsTransport` compiles to
//! wasm32 via `web-sys`. No `#[cfg(not(target_arch = "wasm32"))]` gates needed.
//!
//! ## Topic routing
//!
//! Topics use `"<channel>:<id>"` string keys. The initial `subscribed` snapshot
//! is routed through `extract_topic` and handled by the same parser as deltas.
//! Parsers (in `protocol.rs`) read the `type` field to distinguish snapshot
//! from delta where needed (e.g. orderbook).

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex;

use crate::core::traits::WebSocketConnector;
use crate::core::types::{
    AccountType, ConnectionStatus, OrderbookCapabilities, StreamEvent,
    SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::DydxProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// DydxWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// dYdX v4 WebSocket connector backed by UniversalWsTransport.
///
/// Public market-data only. Does NOT connect until the first `subscribe()`
/// call (or explicit `connect()`).
pub struct DydxWebSocket {
    inner: UniversalWsTransport<DydxProtocol>,
    /// Ping RTT handle for API compatibility with `ping_rtt_handle`.
    /// dYdX uses protocol-level Ping/Pong — RTT is not measured at
    /// application level, so this is always `Some(0)`.
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl DydxWebSocket {
    /// Create a new connector. Does NOT connect yet.
    pub fn new(testnet: bool, account_type: AccountType) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                DydxProtocol::new(testnet),
                account_type,
                testnet,
                None, // public streams only — no credentials
            ),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a public connector (alias for `new`).
    pub fn public(testnet: bool) -> Self {
        Self::new(testnet, AccountType::FuturesCross)
    }
}

impl Default for DydxWebSocket {
    fn default() -> Self {
        Self::new(false, AccountType::FuturesCross)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for DydxWebSocket {
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

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        // dYdX uses protocol-level Ping/Pong (no application-level ping).
        // RTT is tracked by the OS/transport layer, not this connector.
        Some(Arc::clone(&self.ws_ping_rtt_ms))
    }

    /// dYdX v4 orderbook capabilities.
    ///
    /// Single channel `v4_orderbook`: full snapshot on subscribe, then incremental
    /// deltas. Depth is server-controlled (up to ~100 levels per side). No client
    /// depth parameter. No checksum. `message_id` (connection-level sequence) for
    /// gap detection. Perpetuals only.
    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("v4_orderbook", None, None),
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
            ws_channels: CHANNELS,
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
    async fn construction_is_disconnected() {
        let ws = DydxWebSocket::new(false, AccountType::FuturesCross);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_disconnected() {
        let ws = DydxWebSocket::default();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn public_is_disconnected() {
        let ws = DydxWebSocket::public(false);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn testnet_construction_is_disconnected() {
        let ws = DydxWebSocket::new(true, AccountType::FuturesCross);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }
}
