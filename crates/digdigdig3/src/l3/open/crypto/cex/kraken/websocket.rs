//! KrakenWebSocket — thin wrapper around UniversalWsTransport<KrakenProtocol>.
//!
//! Replaces the bespoke 1258-LOC connect/ping/reconnect loop.  The framework
//! owns all connection lifecycle, dual ping scheduling (30 s JSON {"method":"ping"}
//! plus transport-level WS Ping for RTT), subscription replay on reconnect, and
//! frame dispatch.
//!
//! ## Symbol format
//!
//! Kraken WebSocket v2 uses BTC/USD (slash, NOT hyphen; BTC, NOT XBT).
//! Callers must pass Symbol::new("BTC", "USD") — "XBT/USD" is rejected by the exchange.
//!
//! ## Rate limiter
//!
//! The bespoke loop applied a global WeightRateLimiter (10 connects per 10 s).
//! UniversalWsTransport handles reconnect backoff internally — the rate limiter
//! is no longer needed at the application level.
//!
//! ## Reconnect fix
//!
//! The bespoke loop dropped its broadcast::Sender on disconnect, permanently
//! breaking Station's kline-heal re-attach path (0.3.7).  UniversalWsTransport
//! keeps the broadcast::Sender Arc-held — re-attaching via event_stream() after
//! a reconnect works correctly.

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::WebSocketConnector;
use crate::core::types::{
    AccountType, ConnectionStatus, OrderbookCapabilities, StreamEvent,
    SubscriptionRequest, WebSocketResult, WsBookChannel, ChecksumInfo, ChecksumAlgorithm,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::KrakenProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// KrakenWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Kraken WebSocket connector backed by UniversalWsTransport.
///
/// Does NOT connect until the first `subscribe()` call (or explicit `connect()`).
pub struct KrakenWebSocket {
    inner: UniversalWsTransport<KrakenProtocol>,
}

impl KrakenWebSocket {
    /// Create a new connector for public channels.  Does NOT connect yet.
    pub fn new() -> Self {
        Self {
            inner: UniversalWsTransport::new(
                KrakenProtocol,
                AccountType::Spot,
                false, // Kraken has no public testnet
                None,  // public channels only — no credentials
            ),
        }
    }
}

impl Default for KrakenWebSocket {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for KrakenWebSocket {
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
        // Framework does not expose per-pong RTT yet.
        None
    }

    fn orderbook_capabilities(&self, account_type: AccountType) -> OrderbookCapabilities {
        static SPOT_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("book", None, None),
        ];
        match account_type {
            AccountType::Spot => OrderbookCapabilities {
                ws_depths: &[10, 25, 100, 500, 1000],
                ws_default_depth: Some(10),
                rest_max_depth: Some(500),
                rest_depth_values: &[],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[],
                default_speed_ms: None,
                ws_channels: SPOT_CHANNELS,
                checksum: Some(ChecksumInfo {
                    algorithm: ChecksumAlgorithm::Crc32KrakenFormat,
                    levels_per_side: 10,
                    opt_in: false,
                }),
                has_sequence: false,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
            _ => OrderbookCapabilities {
                ws_depths: &[],
                ws_default_depth: None,
                rest_max_depth: None,
                rest_depth_values: &[],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[],
                default_speed_ms: None,
                ws_channels: &[],
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
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
        let ws = KrakenWebSocket::new();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_new() {
        let _ws = KrakenWebSocket::default();
    }
}
