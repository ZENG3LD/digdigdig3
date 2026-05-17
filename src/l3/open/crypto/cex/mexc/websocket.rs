//! MexcWebSocket — thin wrapper around UniversalWsTransport<MexcProtocol>.
//!
//! ## Spot (wss://wbs-api.mexc.com/ws)
//!
//! All market data arrives as **protobuf binary frames**.  The `MexcProtocol`
//! overrides `decode_binary` to extract the channel name and store the raw bytes
//! as a synthetic JSON value so the registry parsers can call
//! `MexcParser::parse_protobuf_message` with the original data.
//!
//! ## Futures (wss://contract.mexc.com/edge)
//!
//! Pure JSON text frames.  Account type `FuturesCross` or `FuturesIsolated`
//! connects to the futures endpoint and uses JSON parsers.
//!
//! ## Usage
//!
//! ```ignore
//! let ws = MexcWebSocket::new(None, AccountType::Spot).await?;
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
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketResult,
    WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::MexcProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// MexcWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// MEXC WebSocket connector backed by UniversalWsTransport.
pub struct MexcWebSocket {
    inner: UniversalWsTransport<MexcProtocol>,
    account_type: AccountType,
}

impl MexcWebSocket {
    /// Create a new connector.  Does NOT connect yet — call `connect()`.
    ///
    /// `credentials` — `None` for public streams.
    /// `account_type` — `Spot` or `FuturesCross`/`FuturesIsolated`.
    pub async fn new(
        credentials: Option<Credentials>,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = MexcProtocol::new(account_type);
        let inner = UniversalWsTransport::new(protocol, account_type, false, credentials);
        Ok(Self { inner, account_type })
    }

    /// Convenience constructor — public spot connection (backward compat).
    pub async fn new_spot(credentials: Option<Credentials>) -> ExchangeResult<Self> {
        Self::new(credentials, AccountType::Spot).await
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for MexcWebSocket {
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
        static MEXC_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("aggre.depth@10ms", None, Some(10)),
            WsBookChannel::delta("aggre.depth@100ms", None, Some(100)),
        ];
        OrderbookCapabilities {
            ws_depths: &[5, 10, 20],
            ws_default_depth: None,
            rest_max_depth: Some(5000),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[10, 100],
            default_speed_ms: None,
            ws_channels: MEXC_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &[],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Backward-compat shim for callers that used `MexcWebSocket::new(credentials)`
// ─────────────────────────────────────────────────────────────────────────────

impl MexcWebSocket {
    /// Legacy constructor used by connector_manager/factory.rs.
    ///
    /// Equivalent to `new(credentials, AccountType::Spot)`.
    pub async fn new_legacy(credentials: Option<Credentials>) -> ExchangeResult<Self> {
        Self::new(credentials, AccountType::Spot).await
    }
}
