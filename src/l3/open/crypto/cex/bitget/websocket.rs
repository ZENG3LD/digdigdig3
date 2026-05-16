//! BitgetWebSocket — thin wrapper around UniversalWsTransport<BitgetProtocol>.
//!
//! Replaces the bespoke connect/ping/reconnect loop.  The framework owns all
//! connection lifecycle, ping scheduling (30s "ping" text frame), subscription
//! replay on reconnect, and frame dispatch.
//!
//! ## Fix: §3.1 reconnect bug
//! The old implementation called `event_tx.take()` on close, causing the
//! broadcast channel to die.  Subsequent `subscribe()` succeeded on wire but
//! consumers received nothing.  The framework's `broadcast::Sender` is
//! Arc-held and never dropped on reconnect.
//!
//! ## Usage
//!
//! ```ignore
//! let ws = BitgetWebSocket::new(None, false, AccountType::Spot).await?;
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
    WsBookChannel, ChecksumInfo, ChecksumAlgorithm,
};
use crate::core::websocket::UniversalWsTransport;
use crate::core::websocket::StreamSpec;

use super::protocol::BitgetProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// BitgetWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Bitget WebSocket connector backed by UniversalWsTransport.
///
/// Construct via `BitgetWebSocket::new(credentials, testnet, account_type)`.
pub struct BitgetWebSocket {
    inner: UniversalWsTransport<BitgetProtocol>,
    _account_type: AccountType,
}

impl BitgetWebSocket {
    /// Create a new connector.  Does NOT connect yet — call `connect()`.
    ///
    /// `credentials` — `None` for public streams (ticker, trade, orderbook, klines, etc.).
    /// `testnet`     — `true` to use `wspap.bitget.com`.
    /// `account_type`— determines instType in subscription args.
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let protocol = BitgetProtocol::new(account_type, testnet);
        let inner = UniversalWsTransport::new(protocol, account_type, testnet, credentials);
        Ok(Self { inner, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for BitgetWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        // account_type is bound at construction; ignore the param for backward compat
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
        // Framework does not expose per-pong RTT yet
        None
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static BITGET_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("books1",  1,   100),
            WsBookChannel::snapshot("books5",  5,   150),
            WsBookChannel::snapshot("books15", 15,  150),
            WsBookChannel::delta("books",      None, Some(150)),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 5, 15],
            ws_default_depth: None,
            rest_max_depth: Some(150),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITGET_CHANNELS,
            checksum: Some(ChecksumInfo {
                algorithm: ChecksumAlgorithm::Crc32Interleaved,
                levels_per_side: 25,
                opt_in: false,
            }),
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}
