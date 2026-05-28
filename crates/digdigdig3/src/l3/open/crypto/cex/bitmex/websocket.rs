//! BitmexWebSocket — thin wrapper around UniversalWsTransport<BitmexProtocol>.
//!
//! Delegates all lifecycle + dispatch to the framework.
//! Public market data only — no auth.

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::{Credentials, WebSocketConnector};
use crate::core::types::{
    AccountType, ConnectionStatus,
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketResult,
    WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport, WsProtocol};

use super::protocol::BitmexProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// BitmexWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// BitMEX WebSocket connector backed by `UniversalWsTransport<BitmexProtocol>`.
pub struct BitmexWebSocket {
    inner: UniversalWsTransport<BitmexProtocol>,
}

impl BitmexWebSocket {
    /// Create a new connector (does NOT connect — call `connect()` first).
    ///
    /// `testnet` — connects to `wss://ws.testnet.bitmex.com/realtime` when `true`.
    pub fn new(testnet: bool) -> Self {
        let protocol = BitmexProtocol::new(testnet);
        let inner = UniversalWsTransport::new(
            protocol,
            AccountType::FuturesCross,
            testnet,
            None::<Credentials>,
        );
        Self { inner }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for BitmexWebSocket {
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
        // Eagerly propagate NotSupported before queuing.
        let protocol = BitmexProtocol::new(false);
        if let Err(e @ crate::core::types::WebSocketError::NotSupported(_)) =
            protocol.subscribe_frame(&spec)
        {
            return Err(e);
        }
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
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static BITMEX_WS_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("orderBookL2_25", Some(25), Some(0)),
            WsBookChannel::delta("orderBookL2",    None,     Some(0)),
        ];
        OrderbookCapabilities {
            ws_depths: &[25],
            ws_default_depth: Some(25),
            rest_max_depth: Some(25),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITMEX_WS_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}
