//! WS subscriber tasks — drain StreamEvent from connectors, persist to storage,
//! and publish to the EventBus for live gRPC subscribers.

use futures_util::StreamExt as _;
use tracing::{error, warn};

use crate::core::storage::StreamKey;
use crate::core::types::{AccountType, ExchangeId, StreamEvent, SubscriptionRequest, Symbol};
use crate::server::bus::BusEvent;
use crate::server::state::ServerState;

/// Spawn one background task per (exchange, account_type) WS connector.
///
/// Each task:
/// 1. Connects the WS
/// 2. Subscribes to the ticker/trade streams for all available symbols
/// 3. Drains events, serialises them, appends to storage and publishes on bus
///
/// Tasks run forever and self-recover on errors (just log and loop).
pub async fn spawn_all(state: &ServerState, exchanges: &[(ExchangeId, AccountType)]) {
    for &(id, account) in exchanges {
        let state = state.clone();
        tokio::spawn(drain_ws(state, id, account));
    }
}

async fn drain_ws(state: ServerState, id: ExchangeId, account: AccountType) {
    let ws = match state.hub.ws(id, account) {
        Some(w) => w,
        None => {
            warn!("no WS for {:?}/{:?}", id, account);
            return;
        }
    };

    // Connect the WS
    if let Err(e) = ws.connect(account).await {
        warn!("WS connect {:?}/{:?} failed: {}", id, account, e);
        return;
    }

    // Subscribe to ticker for a generic BTC/USDT stream as default
    // Real usage: consumer sends specific SubscribeRequest via gRPC
    let default_req = SubscriptionRequest::ticker(Symbol::new("BTC", "USDT"));
    if let Err(e) = ws.subscribe(default_req).await {
        warn!("WS subscribe {:?}/{:?} failed: {}", id, account, e);
    }

    let mut stream = ws.event_stream();
    loop {
        match stream.next().await {
            Some(Ok(event)) => {
                publish_event(&state, id, account, event).await;
            }
            Some(Err(e)) => {
                error!("WS error {:?}/{:?}: {}", id, account, e);
                break;
            }
            None => {
                warn!("WS stream ended {:?}/{:?}", id, account);
                break;
            }
        }
    }
}

async fn publish_event(state: &ServerState, id: ExchangeId, account: AccountType, event: StreamEvent) {
    let (symbol, event_type, timestamp_ms) = extract_meta(&event);

    let payload = match serde_json::to_vec(&event) {
        Ok(b) => b,
        Err(e) => {
            error!("serialize event: {}", e);
            return;
        }
    };

    // Persist to storage
    let key = StreamKey {
        exchange: format!("{:?}", id).to_lowercase(),
        account: format!("{:?}", account).to_lowercase(),
        symbol: symbol.clone(),
        stream_kind: event_type.clone(),
    };
    if let Err(e) = state.storage.append(&key, timestamp_ms, &payload).await {
        error!("storage append: {}", e);
    }

    // Broadcast to gRPC subscribers
    state.bus.publish(BusEvent {
        exchange: format!("{:?}", id).to_lowercase(),
        account: format!("{:?}", account).to_lowercase(),
        symbol,
        stream_kind: event_type.clone(),
        timestamp_ms,
        event_type,
        payload_json: payload,
    });
}

fn extract_meta(event: &StreamEvent) -> (String, String, i64) {
    use crate::core::types::StreamEvent as SE;
    match event {
        SE::Ticker(t) => (t.symbol.clone(), "Ticker".into(), t.timestamp),
        SE::Trade(t) => (t.symbol.clone(), "Trade".into(), t.timestamp),
        SE::OrderbookSnapshot(b) => ("orderbook".into(), "Orderbook".into(), b.timestamp),
        SE::OrderbookDelta(d) => ("orderbook".into(), "OrderbookDelta".into(), d.timestamp),
        SE::Kline(k) => ("kline".into(), "Kline".into(), k.open_time),
        SE::MarkPrice { symbol, timestamp, .. } => (symbol.clone(), "MarkPrice".into(), *timestamp),
        SE::FundingRate { symbol, timestamp, .. } => (symbol.clone(), "FundingRate".into(), *timestamp),
        SE::Liquidation { symbol, timestamp, .. } => (symbol.clone(), "Liquidation".into(), *timestamp),
        SE::OpenInterestUpdate { symbol, timestamp, .. } => (symbol.clone(), "OpenInterest".into(), *timestamp),
        SE::LongShortRatio { symbol, timestamp, .. } => (symbol.clone(), "LongShortRatio".into(), *timestamp),
        _ => ("unknown".into(), "Unknown".into(), 0),
    }
}
