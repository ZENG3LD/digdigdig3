//! GateioWebSocket — thin wrapper around UniversalWsTransport<GateIoProtocol>.
//!
//! Replaces the bespoke connect/ping/reconnect loop. The framework owns all
//! connection lifecycle, ping scheduling (20 s Gate.io JSON ping frame),
//! subscription replay on reconnect, and frame dispatch.
//!
//! Gate.io uses per-product-line WS endpoints selected by account type:
//!   Spot → wss://api.gateio.ws/ws/v4/
//!   Futures USDT → wss://fx-ws.gateio.ws/v4/ws/usdt
//!
//! ## Usage
//!
//! ```ignore
//! let ws = GateioWebSocket::new(None, false, AccountType::Spot).await?;
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
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketError, WebSocketResult,
    WsBookChannel,
};
use crate::core::websocket::{UniversalWsTransport, WsProtocol};
use crate::core::websocket::StreamSpec;

use super::protocol::GateIoProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// GateioWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Gate.io WebSocket connector backed by UniversalWsTransport.
///
/// Construct via `GateioWebSocket::new(credentials, testnet, account_type)`.
pub struct GateioWebSocket {
    inner: UniversalWsTransport<GateIoProtocol>,
    _account_type: AccountType,
}

impl GateioWebSocket {
    /// Create a new connector. Does NOT connect yet — call `connect()`.
    ///
    /// `credentials` — `None` for public streams (ticker, trade, orderbook, klines).
    /// `testnet`     — `true` to use testnet endpoints.
    /// `account_type`— determines product line (Spot / Futures).
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = GateIoProtocol::new(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for GateioWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        // account_type is bound at construction; ignore param for backward compat
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
        // Eagerly surface WireAbsent so callers get a clean error instead of
        // silent_0_events (transport loop warns but does not propagate subscribe_frame errors).
        match self.inner.protocol().subscribe_frame(&spec) {
            Err(e @ WebSocketError::WireAbsent(_)) => return Err(e),
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
        // Framework does not expose per-pong RTT yet
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static GATEIO_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel {
                name: "spot.book_ticker",
                depth: Some(1),
                is_snapshot: true,
                update_speed_ms: Some(10),
                requires_auth_tier: false,
            },
            WsBookChannel {
                name: "spot.order_book_update",
                depth: Some(100),
                is_snapshot: false,
                update_speed_ms: Some(100),
                requires_auth_tier: false,
            },
            WsBookChannel {
                name: "spot.order_book_update",
                depth: Some(20),
                is_snapshot: false,
                update_speed_ms: Some(20),
                requires_auth_tier: false,
            },
            WsBookChannel {
                name: "spot.order_book",
                depth: Some(100),
                is_snapshot: true,
                update_speed_ms: Some(100),
                requires_auth_tier: false,
            },
            WsBookChannel {
                name: "spot.obu",
                depth: Some(400),
                is_snapshot: false,
                update_speed_ms: Some(100),
                requires_auth_tier: false,
            },
            WsBookChannel {
                name: "spot.obu",
                depth: Some(50),
                is_snapshot: false,
                update_speed_ms: Some(20),
                requires_auth_tier: false,
            },
        ];
        OrderbookCapabilities {
            ws_depths: &[5, 10, 20, 50, 100, 400],
            ws_default_depth: Some(100),
            rest_max_depth: Some(1000),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[10, 20, 100, 1000],
            default_speed_ms: Some(100),
            ws_channels: GATEIO_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}
