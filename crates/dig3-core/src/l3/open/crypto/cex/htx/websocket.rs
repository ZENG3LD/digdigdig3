//! HtxWebSocket — thin wrapper around UniversalWsTransport<HtxProtocol>.
//!
//! All connection lifecycle, ping scheduling, subscription replay, and frame
//! dispatch are handled by the framework.  This file contains only the adapter
//! boilerplate plus the `orderbook_capabilities` response copied from the
//! previous implementation.
//!
//! ## KNOWN LIMITATION
//! HTX sends server-initiated `{"ping":<ts>}` heartbeats and expects
//! `{"pong":<ts>}` replies within 5s or it disconnects.  The framework
//! (`WsProtocol` trait) has no hook for server-initiated heartbeats.
//! `UniversalWsTransport` auto-reconnects on disconnect and replays all
//! subscriptions, so event gaps are brief (~1s reconnect).
//! A proper fix requires a `WsProtocol::on_server_message` hook (Wave 3).
//!
//! ## Usage
//!
//! ```ignore
//! let ws = HtxWebSocket::new(None, false, AccountType::Spot)?;
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe(SubscriptionRequest::ticker(Symbol::new("BTC", "USDT"))).await?;
//! let stream = ws.event_stream();
//! ```

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::{Credentials, WebSocketConnector};
use crate::core::types::{
    AccountType, ConnectionStatus, ExchangeResult,
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketError, WebSocketResult,
    WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport, WsProtocol};

use super::protocol::HtxProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// HtxWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// HTX WebSocket connector backed by UniversalWsTransport.
pub struct HtxWebSocket {
    inner: UniversalWsTransport<HtxProtocol>,
    _account_type: AccountType,
    _testnet: bool,
}

impl HtxWebSocket {
    /// Create a new connector.  Does NOT connect yet — call `connect()`.
    ///
    /// `credentials` — `None` for public streams.
    /// `testnet`     — ignored (HTX has no public testnet); accepted for API compat.
    /// `account_type`— determines WS endpoint (spot vs linear-swap).
    pub fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = HtxProtocol::new(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type, _testnet: testnet })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for HtxWebSocket {
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
        // Eagerly reject NotSupported streams before queuing the subscription.
        // The transport's cmd-channel subscribe does not do this check, so we
        // must inspect subscribe_frame here to avoid silent_0_events timeouts.
        let probe = HtxProtocol::new(spec.account_type, self._testnet);
        match probe.subscribe_frame(&spec) {
            Err(e @ WebSocketError::NotSupported(_)) => return Err(e),
            _ => {}
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
        static HTX_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("mbp.5",   Some(5),   None      ),
            WsBookChannel::delta("mbp.10",  Some(10),  Some(100) ),
            WsBookChannel::delta("mbp.20",  Some(20),  Some(100) ),
            WsBookChannel::delta("mbp.150", Some(150), Some(100) ),
            WsBookChannel::delta("mbp.400", Some(400), Some(100) ),
            WsBookChannel::snapshot("depth.step0", 150, 100),
            WsBookChannel::snapshot("depth.step1", 20,  100),
            WsBookChannel::snapshot("depth.step2", 20,  100),
            WsBookChannel::snapshot("depth.step3", 20,  100),
            WsBookChannel::snapshot("depth.step4", 20,  100),
            WsBookChannel::snapshot("depth.step5", 20,  100),
        ];
        OrderbookCapabilities {
            ws_depths: &[5, 10, 20, 150, 400],
            ws_default_depth: Some(20),
            rest_max_depth: Some(150),
            rest_depth_values: &[5, 10, 20, 30, 150],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[100],
            default_speed_ms: Some(100),
            ws_channels: HTX_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: true,
            supports_aggregation: true,
            aggregation_levels: &["step0", "step1", "step2", "step3", "step4", "step5"],
        }
    }
}
