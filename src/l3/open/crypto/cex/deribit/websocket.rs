//! DeribitWebSocket — thin wrapper around UniversalWsTransport<DeribitProtocol>.
//!
//! All connection lifecycle, JSON-RPC 2.0 ping scheduling (every 30 s),
//! subscription replay on reconnect, and frame dispatch are handled by the
//! framework.
//!
//! ## JSON-RPC 2.0 specifics
//! - Subscribe frames carry a monotonic `id` from `DeribitProtocol::next_id()`.
//! - Data frames: `{"method":"subscription","params":{"channel":"...","data":{...}}}`.
//! - Subscribe acks: `{"id":N,"result":["channel1",...]}` — filtered in `extract_topic`.
//! - Ping: `{"method":"public/test"}` every 30 s.
//!
//! ## Options channels
//! Options require a concrete `instrument_name` in the StreamSpec symbol
//! (e.g. `Symbol::new("BTC-30MAY26-50000-C", "")`).  The REST guard is
//! already in place via Wave 4+5 in `deribit/endpoints.rs`.
//!
//! ## Usage
//!
//! ```ignore
//! let ws = DeribitWebSocket::new(None, false, AccountType::FuturesCross).await?;
//! ws.connect(AccountType::FuturesCross).await?;
//! ws.subscribe(SubscriptionRequest::ticker(Symbol::new("BTC", "USD"))).await?;
//! let stream = ws.event_stream();
//! ```

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::{Credentials, WebSocketConnector};
use crate::core::types::{
    AccountType, ConnectionStatus, ExchangeResult, OrderbookCapabilities,
    StreamEvent, SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamSpec, UniversalWsTransport};

use super::protocol::DeribitProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// DeribitWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Deribit WebSocket connector backed by UniversalWsTransport.
///
/// Construct via `DeribitWebSocket::new(credentials, testnet, account_type)`.
pub struct DeribitWebSocket {
    inner: UniversalWsTransport<DeribitProtocol>,
    _account_type: AccountType,
}

impl DeribitWebSocket {
    /// Create a new connector.  Does NOT connect yet — call `connect()`.
    ///
    /// `credentials`  — `None` for public streams only.
    /// `testnet`      — `true` to use `test.deribit.com` endpoint.
    /// `account_type` — stored for `orderbook_capabilities()`.
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = DeribitProtocol::new(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for DeribitWebSocket {
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
        static DERIBIT_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("book.{instr}.{group}.{depth}.100ms", None, Some(100)),
            WsBookChannel::delta("book.{instr}.{group}.{depth}.agg2",  None, None    ),
            WsBookChannel::delta("book.{instr}.{group}.{depth}.raw",   None, None    ).with_auth_tier(),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 10, 20],
            ws_default_depth: Some(20),
            rest_max_depth: Some(10000),
            rest_depth_values: &[1, 5, 10, 20, 50, 100, 1000, 10000],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[100],
            default_speed_ms: Some(100),
            ws_channels: DERIBIT_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: true,
            supports_aggregation: true,
            aggregation_levels: &["none", "1", "2", "5", "10", "25", "100", "250"],
        }
    }
}
