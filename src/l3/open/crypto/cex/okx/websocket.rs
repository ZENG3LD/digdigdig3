//! OkxWebSocket — thin wrapper around UniversalWsTransport<OkxProtocol>.
//!
//! Replaces the bespoke connect/ping/reconnect/dispatch loop.  The framework
//! owns all connection lifecycle, ping scheduling (30s "ping" text frame),
//! subscription replay on reconnect, and frame dispatch.
//!
//! ## Endpoints
//! - Public:   `wss://ws.okx.com:8443/ws/v5/public`
//! - Testnet:  `wss://wspap.okx.com:8443/ws/v5/public`
//! - Business: `wss://ws.okx.com:8443/ws/v5/business`
//!   (mark-price-candle*, index-candle* channels live here)

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::{Credentials, WebSocketConnector};
use crate::core::types::{
    AccountType, ChecksumAlgorithm, ChecksumInfo, ConnectionStatus, ExchangeResult,
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::OkxProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// OkxWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// OKX WebSocket connector backed by UniversalWsTransport.
///
/// Construct via `OkxWebSocket::new` or `OkxWebSocket::new_business`.
/// Call `connect()` before subscribing.
pub struct OkxWebSocket {
    inner: UniversalWsTransport<OkxProtocol>,
    _account_type: AccountType,
}

impl OkxWebSocket {
    /// Create a public connector.
    ///
    /// `credentials` — `None` for public streams.
    /// `testnet`     — `true` to use wspap endpoint.
    /// `account_type`— determines instId formatting (spot vs swap).
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = OkxProtocol::new(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type })
    }

    /// Create a **business** endpoint connector.
    ///
    /// Use for channels that OKX serves on `.../ws/v5/business`:
    /// `mark-price-candle*`, `index-candle*`, `funding-rate-candle*`.
    pub async fn new_business(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = OkxProtocol::new_business(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for OkxWebSocket {
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

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static OKX_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("bbo-tbt",        1,   10),
            WsBookChannel::snapshot("books5",         5,   100),
            WsBookChannel::delta("books",             Some(400), Some(100)),
            WsBookChannel::delta("books50-l2-tbt",    Some(50),  Some(10)).with_auth_tier(),
            WsBookChannel::delta("books-l2-tbt",      Some(400), Some(10)).with_auth_tier(),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 5, 50, 400],
            ws_default_depth: Some(400),
            rest_max_depth: Some(400),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[10, 100],
            default_speed_ms: Some(100),
            ws_channels: OKX_CHANNELS,
            checksum: Some(ChecksumInfo {
                algorithm: ChecksumAlgorithm::Crc32Interleaved,
                levels_per_side: 25,
                opt_in: false,
            }),
            has_sequence: true,
            has_prev_sequence: true,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}
