//! BitfinexWebSocket — thin wrapper around UniversalWsTransport<BitfinexProtocol>.
//!
//! Replaces the bespoke 1,197-LOC connect/heartbeat/ping loop. The framework
//! owns all connection lifecycle, subscription replay on reconnect, and frame
//! dispatch.
//!
//! ## chanId integer routing
//!
//! `BitfinexProtocol` holds an `Arc<StdMutex<HashMap<u64, TopicKey>>>` that maps
//! server-assigned chanIds to per-symbol topic keys. Populated from subscribe acks
//! via `is_subscribe_ack`; cleared on overwrite when acks arrive after reconnect.
//!
//! ## Application-level ping
//!
//! `BitfinexProtocol::ping_frame()` returns `{"event":"ping","cid":0}` every 20 s.
//! Server replies with `{"event":"pong","ts":...}`, which `is_pong` matches.
//!
//! ## Symbol extraction
//!
//! Bitfinex data frames are `[chanId, data]` — symbol is NOT in the frame.
//! `extract_topic` looks up the chanId and stores the symbol in a thread-local
//! so parser functions can emit the correct `symbol` field.
//!
//! ## Wasm support
//!
//! Uses `UniversalWsTransport` which compiles to wasm32 via `web-sys`. No
//! native-only gates needed.
//!
//! ## Public channels only
//!
//! Private channel support (authentication) is not implemented in this migration.
//! Subscribing to private stream kinds returns `WebSocketError::WireAbsent`.

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::WebSocketConnector;
use crate::core::types::{
    AccountType, ChecksumAlgorithm, ChecksumInfo, ConnectionStatus, OrderbookCapabilities,
    StreamEvent, SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::BitfinexProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// BitfinexWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Bitfinex WebSocket connector backed by UniversalWsTransport.
///
/// Does NOT connect until the first `subscribe()` call (or explicit `connect()`).
pub struct BitfinexWebSocket {
    inner: UniversalWsTransport<BitfinexProtocol>,
}

impl BitfinexWebSocket {
    /// Create a new connector. Does NOT connect yet.
    ///
    /// `testnet`: Bitfinex has no public testnet; the parameter is accepted for
    /// API uniformity but has no effect.
    pub fn new(testnet: bool) -> Self {
        Self {
            inner: UniversalWsTransport::new(
                BitfinexProtocol::new(testnet),
                AccountType::Spot,
                testnet,
                None, // public streams, no credentials
            ),
        }
    }
}

impl Default for BitfinexWebSocket {
    fn default() -> Self {
        Self::new(false)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for BitfinexWebSocket {
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
        // Bitfinex uses application-level ping/pong (JSON) for keepalive.
        // Native WS-frame RTT measurement is not used on this connector.
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static BITFINEX_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("book/P0", None, None),
            WsBookChannel::delta("book/P1", None, None),
            WsBookChannel::delta("book/P2", None, None),
            WsBookChannel::delta("book/P3", None, None),
            WsBookChannel::delta("book/P4", None, None),
            WsBookChannel::delta("book/R0", None, None),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 25, 100, 250],
            ws_default_depth: Some(25),
            rest_max_depth: Some(250),
            rest_depth_values: &[1, 25, 100, 250],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITFINEX_CHANNELS,
            checksum: Some(ChecksumInfo {
                algorithm: ChecksumAlgorithm::Crc32Interleaved,
                levels_per_side: 25,
                opt_in: true,
            }),
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &["P0", "P1", "P2", "P3", "P4", "R0"],
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
        let ws = BitfinexWebSocket::new(false);
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_new() {
        let _ws = BitfinexWebSocket::default();
    }
}
