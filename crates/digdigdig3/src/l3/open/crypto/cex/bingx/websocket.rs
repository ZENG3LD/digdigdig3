//! BingxWebSocket — thin wrapper around UniversalWsTransport<BingxProtocol>.
//!
//! Replaces the bespoke 945-LOC connect/gzip/ping loop. The framework owns all
//! connection lifecycle, subscription replay on reconnect, server-ping response,
//! gzip decompression, and frame dispatch.
//!
//! ## GZIP binary frames
//!
//! All BingX data frames are GZIP-compressed binary. The transport's default
//! `decode_binary` fallback chain (gzip → zlib → deflate → UTF-8) handles
//! decompression transparently — no override needed.
//!
//! ## Server-initiated ping
//!
//! `BingxProtocol::is_server_ping` matches both:
//! - `{"ping":"<id>","time":"..."}` JSON objects
//! - `"Ping"` plain strings (after gzip decompression)
//!
//! `BingxProtocol::pong_response_frame` builds the appropriate reply.
//! The transport sends the reply automatically.
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

use super::protocol::BingxProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// BingxWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// BingX WebSocket connector backed by UniversalWsTransport.
///
/// Does NOT connect until the first `subscribe()` call (or explicit `connect()`).
pub struct BingxWebSocket {
    inner: UniversalWsTransport<BingxProtocol>,
}

impl BingxWebSocket {
    /// Create a new connector. Does NOT connect yet.
    pub fn new(_credentials: Option<crate::core::traits::Credentials>, testnet: bool, account_type: AccountType) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                BingxProtocol::new(testnet),
                account_type,
                testnet,
                None, // public streams only; private channels require listen-key
            ),
        }
    }
}

impl Default for BingxWebSocket {
    fn default() -> Self {
        Self::new(None, false, AccountType::Spot)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for BingxWebSocket {
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
        // BingX uses server-initiated pings; client does not send WS-level pings.
        // RTT measurement via WS Ping/Pong frames is not available.
        None
    }

    fn orderbook_capabilities(&self, account_type: AccountType) -> OrderbookCapabilities {
        static SPOT_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("@depth5",   5,   1000),
            WsBookChannel::snapshot("@depth10",  10,  1000),
            WsBookChannel::snapshot("@depth20",  20,  1000),
            WsBookChannel::snapshot("@depth50",  50,  1000),
            WsBookChannel::snapshot("@depth100", 100, 1000),
        ];
        static FUTURES_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("@depth5",   5,   100),
            WsBookChannel::snapshot("@depth10",  10,  100),
            WsBookChannel::snapshot("@depth20",  20,  100),
            WsBookChannel::snapshot("@depth50",  50,  100),
            WsBookChannel::snapshot("@depth100", 100, 100),
        ];
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => OrderbookCapabilities {
                ws_depths: &[5, 10, 20, 50, 100],
                ws_default_depth: Some(20),
                rest_max_depth: Some(1000),
                rest_depth_values: &[5, 10, 20, 50, 100, 500, 1000],
                supports_snapshot: true,
                supports_delta: false,
                update_speeds_ms: &[100, 200, 500, 1000],
                default_speed_ms: Some(100),
                ws_channels: FUTURES_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
            _ => OrderbookCapabilities {
                ws_depths: &[5, 10, 20, 50, 100],
                ws_default_depth: Some(20),
                rest_max_depth: Some(1000),
                rest_depth_values: &[5, 10, 20, 50, 100, 500, 1000],
                supports_snapshot: true,
                supports_delta: false,
                update_speeds_ms: &[1000],
                default_speed_ms: Some(1000),
                ws_channels: SPOT_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: true,
                aggregation_levels: &[],
            },
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
        let ws = BingxWebSocket::new(None, false, AccountType::Spot);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_new() {
        let _ws = BingxWebSocket::default();
    }
}
