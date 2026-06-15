//! HtxWebSocket — dual-transport wrapper around two UniversalWsTransport<HtxProtocol>
//! instances: one for the main market data WS and one for the `ws_index` endpoint.
//!
//! ## Why dual transports?
//!
//! HTX exposes index klines (market.{contract}.index.{period}) ONLY on the dedicated
//! `wss://api.hbdm.com/ws_index` endpoint. The main endpoints (spot WS and
//! linear-swap WS) reject these topics with "invalid topic". All other market data
//! channels are on the main endpoint. Dual transports mirror the OKX pattern (public
//! vs business endpoint split).
//!
//! `subscribe()` routes IndexPriceKline to the index transport; all other kinds go to
//! the main transport. `event_stream()` merges both streams.
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
//! let ws = HtxWebSocket::new(None, false, AccountType::FuturesCross)?;
//! ws.connect(AccountType::FuturesCross).await?;
//! ws.subscribe(SubscriptionRequest::index_price_kline("BTC-USDT", "1m")).await?;
//! let stream = ws.event_stream();
//! ```

use std::pin::Pin;
use std::sync::Arc;

use futures_util::{stream::select, Stream};
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::{Credentials, WebSocketConnector};
use crate::core::types::{
    AccountType, ConnectionStatus, ExchangeResult,
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketError, WebSocketResult,
    WsBookChannel,
};
use crate::core::websocket::{StreamKind, StreamSpec, UniversalWsTransport, WsProtocol};

use super::protocol::HtxProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// Routing helper
// ─────────────────────────────────────────────────────────────────────────────

/// Returns true if this kind belongs to the `ws_index` endpoint.
fn is_index_kind(kind: &StreamKind) -> bool {
    matches!(kind, StreamKind::IndexPriceKline { .. })
}

// ─────────────────────────────────────────────────────────────────────────────
// HtxWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// HTX WebSocket connector.
///
/// Uses two `UniversalWsTransport` instances:
/// - `main` — spot or linear-swap WS for all standard channels
/// - `index` — `ws_index` endpoint for `IndexPriceKline` channels
///
/// Both connections open eagerly on `connect()`. `subscribe()` / `unsubscribe()`
/// route to the correct transport by `StreamKind`. `event_stream()` merges both.
pub struct HtxWebSocket {
    main: UniversalWsTransport<HtxProtocol>,
    index: UniversalWsTransport<HtxProtocol>,
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
        let main_proto = HtxProtocol::new(account_type, testnet);
        let index_proto = HtxProtocol::new_index(account_type, testnet);
        let main = UniversalWsTransport::new(main_proto, account_type, testnet, credentials.clone());
        let index = UniversalWsTransport::new(index_proto, account_type, testnet, credentials);
        Ok(Self { main, index, _account_type: account_type, _testnet: testnet })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for HtxWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        self.main.connect().await?;
        self.index.connect().await?;
        Ok(())
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        self.main.disconnect().await?;
        self.index.disconnect().await?;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.main.connection_status()
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        if is_index_kind(&spec.kind) {
            self.index.subscribe(spec).await
        } else {
            // Eagerly reject WireAbsent streams before queuing the subscription.
            let probe = HtxProtocol::new(spec.account_type, self._testnet);
            match probe.subscribe_frame(&spec) {
                Err(e @ WebSocketError::WireAbsent(_)) => return Err(e),
                _ => {}
            }
            self.main.subscribe(spec).await
        }
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        if is_index_kind(&spec.kind) {
            self.index.unsubscribe(spec).await
        } else {
            self.main.unsubscribe(spec).await
        }
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let main_stream = self.main.event_stream();
        let index_stream = self.index.event_stream();
        Box::pin(select(main_stream, index_stream))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        let mut subs: Vec<SubscriptionRequest> = self.main
            .active_subscriptions()
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect();
        subs.extend(
            self.index
                .active_subscriptions()
                .into_iter()
                .map(SubscriptionRequest::from),
        );
        subs
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
