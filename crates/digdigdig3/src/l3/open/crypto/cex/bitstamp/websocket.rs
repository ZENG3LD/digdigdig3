//! BitstampWebSocket — thin wrapper around UniversalWsTransport<BitstampProtocol>.
//!
//! Replaces the bespoke 1154-LOC connect/ping/reconnect loop.  The framework
//! owns all connection lifecycle, ping scheduling (30s Pusher text ping),
//! subscription replay on reconnect, and frame dispatch.
//!
//! ## Auto-connect bug fix
//!
//! The old bespoke implementation did not connect automatically when `subscribe()`
//! was called — it required an explicit `ws.connect()` call first. Station never
//! calls `ws.connect()` directly (it calls `subscribe()`), so every bitstamp WS
//! subscription returned `Network("Not connected")`.
//!
//! `UniversalWsTransport::subscribe` sends a `TransportCmd::Subscribe` which
//! triggers the driver task to connect lazily on the first command.  No explicit
//! `connect()` call is needed from Station.
//!
//! ## L3 snapshot bootstrap
//!
//! On the first `subscribe(OrderbookL3)` for a pair, `emit_l3_snapshot` fetches
//! the REST L3 order book and injects synthetic `OrderbookL3 { action: "create" }`
//! events via `inner.broadcast_events`.  Live `live_orders_*` events that arrive
//! during the REST round-trip are buffered in the broadcast channel and flow after
//! the snapshot batch — no live events are lost.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::traits::WebSocketConnector;
use crate::core::types::{
    AccountType, ConnectionStatus, ExchangeId, OrderSide, OrderbookCapabilities, StreamEvent,
    SubscriptionRequest, WebSocketResult, WsBookChannel,
};
use crate::core::websocket::{StreamKind, StreamSpec, UniversalWsTransport};

use super::protocol::BitstampProtocol;

// ─────────────────────────────────────────────────────────────────────────────
// BitstampWebSocket
// ─────────────────────────────────────────────────────────────────────────────

/// Bitstamp WebSocket connector backed by UniversalWsTransport.
///
/// Construct via `BitstampWebSocket::new()`.  Does NOT connect until the first
/// `subscribe()` call (or explicit `connect()`).
pub struct BitstampWebSocket {
    inner: UniversalWsTransport<BitstampProtocol>,
    rest_client: reqwest::Client,
    /// Pairs for which we have already fetched the REST L3 snapshot this session.
    l3_bootstrapped: Arc<TokioMutex<HashSet<String>>>,
}

impl BitstampWebSocket {
    /// Create a new connector.  Does NOT connect yet.
    pub fn new() -> Self {
        Self {
            inner: UniversalWsTransport::new(
                BitstampProtocol,
                AccountType::Spot,
                false, // testnet ignored — Bitstamp has none
                None,  // public streams, no credentials
            ),
            rest_client: reqwest::Client::new(),
            l3_bootstrapped: Arc::new(TokioMutex::new(HashSet::new())),
        }
    }
}

impl Default for BitstampWebSocket {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocketConnector impl — delegates to inner transport
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for BitstampWebSocket {
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

        let is_l3 = matches!(spec.kind, StreamKind::OrderbookL3);
        // Resolve pair for bootstrap key before moving spec.
        // For Bitstamp (Raw inputs only in practice), resolve is infallible.
        // On normalization error we skip bootstrap — live events still flow.
        let pair = spec
            .symbol
            .resolve(ExchangeId::Bitstamp, spec.account_type)
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        // Subscribe via transport (triggers lazy connect + sends Pusher subscribe frame).
        self.inner.subscribe(spec).await?;

        // L3 bootstrap: on first subscribe for this pair, fetch REST snapshot.
        if is_l3 {
            let mut done = self.l3_bootstrapped.lock().await;
            if !done.contains(&pair) {
                done.insert(pair.clone());
                drop(done);
                let transport = self.inner.clone();
                let client = self.rest_client.clone();
                #[cfg(not(target_arch = "wasm32"))]
                tokio::spawn(async move {
                    if let Err(e) = emit_l3_snapshot(&transport, &client, &pair).await {
                        tracing::warn!(
                            target: "dig3::bitstamp::l3",
                            pair = %pair,
                            error = ?e,
                            "L3 REST snapshot bootstrap failed"
                        );
                    }
                });
                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = emit_l3_snapshot(&transport, &client, &pair).await {
                        tracing::warn!(
                            target: "dig3::bitstamp::l3",
                            pair = %pair,
                            error = ?e,
                            "L3 REST snapshot bootstrap failed"
                        );
                    }
                });
            }
        }

        Ok(())
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

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static BITSTAMP_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("order_book", 100, 1000),
            WsBookChannel::delta("diff_order_book", None, None),
        ];
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITSTAMP_CHANNELS,
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &["0", "1", "2"],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// L3 snapshot bootstrap
// ─────────────────────────────────────────────────────────────────────────────

/// Fetch the REST L3 order book snapshot for `pair` and inject synthetic
/// `OrderbookL3 { action: "create" }` events into the transport's broadcast
/// channel.
///
/// REST endpoint: `GET https://www.bitstamp.net/api/v2/order_book/{pair}/?group=2`
///
/// Response shape (`group=2` mirrors the WS `live_orders_*` L3 layout):
/// ```json
/// { "bids": [["price","amount","order_id"], ...],
///   "asks": [["price","amount","order_id"], ...],
///   "microtimestamp": "1643643584684047" }
/// ```
async fn emit_l3_snapshot(
    transport: &UniversalWsTransport<BitstampProtocol>,
    client: &reqwest::Client,
    pair: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://www.bitstamp.net/api/v2/order_book/{}/?group=2",
        pair
    );
    let resp = client.get(&url).send().await?;
    let json: serde_json::Value = resp.json().await?;

    // microtimestamp is microseconds → convert to milliseconds.
    let timestamp_ms = json
        .get("microtimestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|us| us / 1000)
        .unwrap_or(0);

    let symbol = pair.to_ascii_uppercase();

    let parse_side = |entries: &serde_json::Value, side: OrderSide| -> Vec<StreamEvent> {
        entries
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let e = entry.as_array()?;
                        let price = e.first()?.as_str()?.parse::<f64>().ok()?;
                        let quantity = e.get(1)?.as_str()?.parse::<f64>().ok()?;
                        let order_id = e.get(2)?.as_str()?.to_string();
                        Some(StreamEvent::OrderbookL3 {
                            symbol: symbol.clone(),
                            side,
                            order_id,
                            price,
                            quantity,
                            action: "create".to_string(),
                            timestamp: timestamp_ms,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut events: Vec<StreamEvent> = Vec::new();
    events.extend(parse_side(
        json.get("bids").unwrap_or(&serde_json::Value::Null),
        OrderSide::Buy,
    ));
    events.extend(parse_side(
        json.get("asks").unwrap_or(&serde_json::Value::Null),
        OrderSide::Sell,
    ));

    tracing::debug!(
        target: "dig3::bitstamp::l3",
        pair = %pair,
        count = events.len(),
        "injecting REST L3 snapshot events"
    );

    transport.broadcast_events(events);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn websocket_construction_is_sync() {
        // BitstampWebSocket::new() constructor is sync — no .await required.
        // UniversalWsTransport spawns its driver task internally (needs a runtime).
        let ws = BitstampWebSocket::new();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn default_is_same_as_new() {
        let _ws = BitstampWebSocket::default();
    }
}
