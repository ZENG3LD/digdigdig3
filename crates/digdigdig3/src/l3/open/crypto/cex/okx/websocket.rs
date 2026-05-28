//! OkxWebSocket — dual-transport wrapper around two UniversalWsTransport<OkxProtocol>
//! instances, one for /ws/v5/public and one for /ws/v5/business.
//!
//! ## OKX WS endpoint split (migration 2023-06-20)
//! - `/ws/v5/public`   — tickers, mark-price, funding-rate, open-interest, books,
//!                       trades, liquidation-orders, index-tickers, etc.
//! - `/ws/v5/business` — `candle*`, `mark-price-candle*`, `index-candle*`
//!
//! Subscribing kline channels on the public endpoint returns error 60018.
//! This connector keeps both connections open and routes by `StreamKind`.

use std::pin::Pin;
use std::sync::Arc;

use futures_util::{stream::select, Stream};
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::{Credentials, WebSocketConnector};
use crate::core::types::{
    AccountType, ChecksumAlgorithm, ChecksumInfo, ConnectionStatus, ExchangeResult,
    OrderbookCapabilities, StreamEvent, SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamKind, StreamSpec, UniversalWsTransport};

use super::protocol::OkxProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// Routing helper
// ─────────────────────────────────────────────────────────────────────────────

/// Returns true if the stream kind belongs to the /ws/v5/business endpoint.
fn is_business_kind(kind: &StreamKind) -> bool {
    matches!(
        kind,
        StreamKind::Kline { .. }
            | StreamKind::MarkPriceKline { .. }
            | StreamKind::IndexPriceKline { .. }
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// OkxWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// OKX WebSocket connector backed by two UniversalWsTransports:
/// - `public`   → `/ws/v5/public`  (tickers, marks, funding, OI, trades, books, liq, …)
/// - `business` → `/ws/v5/business` (candle*, mark-price-candle*, index-candle*)
///
/// Both connections open eagerly on `connect()`. `subscribe()` / `unsubscribe()`
/// route to the correct transport by `StreamKind`. `event_stream()` merges both.
pub struct OkxWebSocket {
    public: UniversalWsTransport<OkxProtocol>,
    business: UniversalWsTransport<OkxProtocol>,
    _account_type: AccountType,
}

impl OkxWebSocket {
    /// Create public + business connector pair.
    ///
    /// `credentials` — `None` for public streams.
    /// `testnet`     — `true` to use wspap endpoint.
    /// `account_type`— determines instId formatting (spot vs swap).
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let public_proto = OkxProtocol::new(account_type, testnet);
        let business_proto = OkxProtocol::new_business(account_type, testnet);

        let public =
            UniversalWsTransport::new(public_proto, account_type, testnet, credentials.clone());
        let business =
            UniversalWsTransport::new(business_proto, account_type, testnet, credentials);

        Ok(Self { public, business, _account_type: account_type })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for OkxWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        // Connect both eagerly; return first error encountered.
        self.public.connect().await?;
        self.business.connect().await?;
        Ok(())
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        self.public.disconnect().await?;
        self.business.disconnect().await?;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        let pub_status = self.public.connection_status();
        let biz_status = self.business.connection_status();
        // Both must be Connected for overall Connected.
        // Any Connecting → Connecting.
        // Any Disconnected → Disconnected.
        match (pub_status, biz_status) {
            (ConnectionStatus::Connected, ConnectionStatus::Connected) => {
                ConnectionStatus::Connected
            }
            (ConnectionStatus::Disconnected, _) | (_, ConnectionStatus::Disconnected) => {
                ConnectionStatus::Disconnected
            }
            _ => ConnectionStatus::Connecting,
        }
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        if is_business_kind(&spec.kind) {
            self.business.subscribe(spec).await
        } else {
            self.public.subscribe(spec).await
        }
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let spec = StreamSpec::try_from(request)?;
        if is_business_kind(&spec.kind) {
            self.business.unsubscribe(spec).await
        } else {
            self.public.unsubscribe(spec).await
        }
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let pub_stream = self.public.event_stream();
        let biz_stream = self.business.event_stream();
        Box::pin(select(pub_stream, biz_stream))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        let mut subs: Vec<SubscriptionRequest> = self
            .public
            .active_subscriptions()
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect();
        let biz_subs: Vec<SubscriptionRequest> = self
            .business
            .active_subscriptions()
            .into_iter()
            .map(SubscriptionRequest::from)
            .collect();
        subs.extend(biz_subs);
        subs
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
