//! BybitWebSocket — thin wrapper around UniversalWsTransport<BybitProtocol>.
//!
//! The framework owns all connection lifecycle, ping scheduling (20s
//! `{"op":"ping"}` text frame), subscription replay on reconnect, and frame
//! dispatch. The old bespoke implementation is superseded.
//!
//! ## Fix: section 3.3 silent-stream bug class
//! The old implementation dropped unmatched topics silently.
//! The framework emits tracing::warn for every unmatched topic.
//!
//! ## Usage
//!
//! ```ignore
//! let ws = BybitWebSocket::new(None, false, AccountType::Spot).await?;
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe(SubscriptionRequest::ticker(Symbol::new("BTC", "USDT"))).await?;
//! let stream = ws.event_stream();
//! ```

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::{Credentials, WebSocketConnector};
use crate::core::types::{
    AccountType, ConnectionStatus, ExchangeResult,
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketResult,
    WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::BybitProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// BybitWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Bybit V5 WebSocket connector backed by UniversalWsTransport.
///
/// Construct via `BybitWebSocket::new(credentials, testnet, account_type)`.
pub struct BybitWebSocket {
    inner: UniversalWsTransport<BybitProtocol>,
    _account_type: AccountType,
}

impl BybitWebSocket {
    /// Create a new connector. Does NOT connect yet — call `connect()`.
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = BybitProtocol::new(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for BybitWebSocket {
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
        None
    }

    fn orderbook_capabilities(&self, account_type: AccountType) -> OrderbookCapabilities {
        static SPOT_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("orderbook.1",    1,   10),
            WsBookChannel::delta("orderbook.50",   Some(50),   Some(20)),
            WsBookChannel::delta("orderbook.200",  Some(200),  Some(100)),
            WsBookChannel::delta("orderbook.1000", Some(1000), Some(200)),
        ];
        static LINEAR_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("orderbook.1",    1,   10),
            WsBookChannel::delta("orderbook.50",   Some(50),   Some(20)),
            WsBookChannel::delta("orderbook.200",  Some(200),  Some(100)),
            WsBookChannel::delta("orderbook.1000", Some(1000), Some(200)),
        ];
        static OPTION_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("orderbook.25",  Some(25),  Some(20)),
            WsBookChannel::delta("orderbook.100", Some(100), Some(100)),
        ];

        match account_type {
            AccountType::Options => OrderbookCapabilities {
                ws_depths: &[25, 100],
                ws_default_depth: Some(25),
                rest_max_depth: Some(25),
                rest_depth_values: &[],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[20, 100],
                default_speed_ms: Some(20),
                ws_channels: OPTION_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
            AccountType::Spot => OrderbookCapabilities {
                ws_depths: &[1, 50, 200, 1000],
                ws_default_depth: Some(50),
                rest_max_depth: Some(200),
                rest_depth_values: &[],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[10, 20, 100, 200],
                default_speed_ms: Some(20),
                ws_channels: SPOT_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
            _ => OrderbookCapabilities {
                ws_depths: &[1, 50, 200, 1000],
                ws_default_depth: Some(50),
                rest_max_depth: Some(500),
                rest_depth_values: &[],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[10, 20, 100, 200],
                default_speed_ms: Some(20),
                ws_channels: LINEAR_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
        }
    }
}
