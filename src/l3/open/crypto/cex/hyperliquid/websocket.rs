//! Hyperliquid WebSocket — thin wrapper over UniversalWsTransport.
//!
//! All connection lifecycle, ping, reconnect, and frame dispatch is handled
//! by `UniversalWsTransport<HyperliquidProtocol>`. This struct preserves the
//! public `HyperliquidWebSocket::new(testnet)` API required by the factory.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::Mutex;

use crate::core::types::{
    AccountType, ConnectionStatus, OrderbookCapabilities, StreamEvent, StreamType,
    SubscriptionRequest, Symbol, WebSocketResult,
};
use crate::core::traits::WebSocketConnector;
use crate::core::websocket::{StreamSpec, StreamKind, UniversalWsTransport};

use super::protocol::HyperliquidProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// HyperliquidWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Hyperliquid WebSocket connector — delegates to UniversalWsTransport.
pub struct HyperliquidWebSocket {
    inner: UniversalWsTransport<HyperliquidProtocol>,
    /// Ping RTT tracking handle (kept for API compatibility with `ping_rtt_handle`).
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl HyperliquidWebSocket {
    /// Create a new Hyperliquid WebSocket connector.
    pub fn new(testnet: bool) -> Self {
        let protocol = HyperliquidProtocol::new(testnet);
        let inner = UniversalWsTransport::new(
            protocol,
            AccountType::FuturesCross,
            testnet,
            None, // public connector — no credentials
        );
        Self {
            inner,
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        }
    }

    /// Create public connector (alias for `new`).
    pub fn public(testnet: bool) -> Self {
        Self::new(testnet)
    }

    /// Subscribe to `allMids` channel — all coin mid prices in one snapshot.
    ///
    /// Sends `{"method":"subscribe","subscription":{"type":"allMids","dex":""}}`.
    /// Events arrive as `StreamEvent::Ticker` per coin.
    pub async fn subscribe_all_mids(&self) -> WebSocketResult<()> {
        let spec = StreamSpec {
            kind: StreamKind::Ticker,
            symbol: String::new(),
            account_type: AccountType::FuturesCross,
            depth: None,
            speed_ms: None,
        };
        self.inner.subscribe(spec).await
    }

    /// Subscribe to `userNonFundingLedgerUpdates` for a wallet address.
    ///
    /// Events arrive as `StreamEvent::BalanceUpdate` per ledger entry.
    pub async fn subscribe_non_funding_ledger(&self, user_address: &str) -> WebSocketResult<()> {
        let request = SubscriptionRequest::new(
            Symbol::new(user_address, ""),
            StreamType::BalanceUpdate,
        );
        let spec = StreamSpec::try_from(request)?;
        self.inner.subscribe(spec).await
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegate to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl WebSocketConnector for HyperliquidWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        // transport is bound to FuturesCross at construction; ignore argument
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

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(Arc::clone(&self.ws_ping_rtt_ms))
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: Some(20),
            rest_max_depth: Some(20),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: false,
            update_speeds_ms: &[],
            default_speed_ms: Some(500),
            ws_channels: &[],
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &["null", "2", "3", "4", "5"],
        }
    }
}
