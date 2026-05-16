//! BinanceWebSocket — thin wrapper around UniversalWsTransport<BinanceProtocol>.
//!
//! All connection lifecycle, ping scheduling (native WS ping every 20s),
//! subscription replay on reconnect, and frame dispatch are handled by the
//! framework.
//!
//! ## Fix: spec §3.3 silent streams
//! Old code had `_ => Ok(None)` catch-all — silently dropped unknown events.
//! The framework emits `tracing::warn!` for every unmatched topic instead,
//! making silent drops visible in logs.
//!
//! ## Usage
//!
//! ```ignore
//! let ws = BinanceWebSocket::new(None, false, AccountType::Spot).await?;
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
use crate::core::websocket::UniversalWsTransport;
use crate::core::websocket::StreamSpec;

use super::protocol::BinanceProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// BinanceWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Binance WebSocket connector backed by UniversalWsTransport.
///
/// Construct via `BinanceWebSocket::new(credentials, testnet, account_type)`.
pub struct BinanceWebSocket {
    inner: UniversalWsTransport<BinanceProtocol>,
    _account_type: AccountType,
}

impl BinanceWebSocket {
    /// Create a new connector.  Does NOT connect yet — call `connect()`.
    ///
    /// `credentials`  — `None` for public streams.
    /// `testnet`      — `true` to use testnet endpoints.
    /// `account_type` — determines which registry (spot vs futures) and endpoint.
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = BinanceProtocol::new(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for BinanceWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        // account_type bound at construction; ignore param for backward compat
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
            WsBookChannel::snapshot("depth5@100ms",  5,  100),
            WsBookChannel::snapshot("depth10@100ms", 10, 100),
            WsBookChannel::snapshot("depth20@100ms", 20, 100),
            WsBookChannel::delta("depth@100ms",  None, Some(100)),
            WsBookChannel::delta("depth@1000ms", None, Some(1000)),
        ];

        static FUTURES_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("depth5@100ms",  5,  100),
            WsBookChannel::snapshot("depth10@100ms", 10, 100),
            WsBookChannel::snapshot("depth20@100ms", 20, 100),
            WsBookChannel::delta("depth@100ms", None, Some(100)),
            WsBookChannel::delta("depth@250ms", None, Some(250)),
            WsBookChannel::delta("depth@500ms", None, Some(500)),
        ];

        match account_type {
            AccountType::Spot | AccountType::Margin => OrderbookCapabilities {
                ws_depths: &[5, 10, 20],
                ws_default_depth: Some(20),
                rest_max_depth: Some(5000),
                rest_depth_values: &[],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[100, 1000],
                default_speed_ms: Some(1000),
                ws_channels: SPOT_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: false,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
            _ => OrderbookCapabilities {
                ws_depths: &[5, 10, 20],
                ws_default_depth: Some(20),
                rest_max_depth: Some(1000),
                rest_depth_values: &[5, 10, 20, 50, 100, 500, 1000],
                supports_snapshot: true,
                supports_delta: true,
                update_speeds_ms: &[100, 250, 500],
                default_speed_ms: Some(250),
                ws_channels: FUTURES_CHANNELS,
                checksum: None,
                has_sequence: true,
                has_prev_sequence: true,
                supports_aggregation: false,
                aggregation_levels: &[],
            },
        }
    }
}
