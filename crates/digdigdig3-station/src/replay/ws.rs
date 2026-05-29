//! ReplayWebSocket — implements `WebSocketConnector` for stored events.
//!
//! Each `subscribe` call spawns a task that reads records from
//! `StorageManager`, optionally sleeps according to `ReplayRate`, then sends
//! decoded `StreamEvent`s on the broadcast channel.
//!
//! On wasm32:
//! - `tokio::spawn` → `wasm_bindgen_futures::spawn_local`
//! - `tokio::time::sleep` → `gloo_timers::future::sleep`
//! - `std::time::Instant::now()` → `js_sys::Date::now()` (f64 ms counter)

use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures_util::Stream;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use crate::storage::{StorageManager, StreamKey};
use digdigdig3::core::traits::WebSocketConnector;
use digdigdig3::core::types::{
    AccountType, ConnectionStatus, ExchangeId, OrderbookCapabilities, StreamEvent,
    StreamType, SubscriptionRequest, WebSocketError, WebSocketResult,
};

use super::reader::load_records;
use super::hub::ReplayConfig;

// ── channel capacity ──────────────────────────────────────────────────────────

const CHANNEL_CAP: usize = 4096;

// ── ReplayWebSocket ───────────────────────────────────────────────────────────

/// Drop-in `WebSocketConnector` that replays stored events.
pub struct ReplayWebSocket {
    id: ExchangeId,
    account: AccountType,
    storage: Arc<StorageManager>,
    config: ReplayConfig,
    event_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
    /// Guarded by std RwLock — never held across .await.
    active_subs: RwLock<Vec<SubscriptionRequest>>,
    /// Guarded by std RwLock — never held across .await.
    state: RwLock<ConnectionStatus>,
}

impl ReplayWebSocket {
    pub(crate) fn new(
        id: ExchangeId,
        account: AccountType,
        storage: Arc<StorageManager>,
        config: ReplayConfig,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(CHANNEL_CAP);
        Self {
            id,
            account,
            storage,
            config,
            event_tx,
            active_subs: RwLock::new(vec![]),
            state: RwLock::new(ConnectionStatus::Disconnected),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WebSocketConnector for ReplayWebSocket {
    async fn connect(&self, _account_type: AccountType) -> WebSocketResult<()> {
        *self
            .state
            .write()
            .map_err(|_| WebSocketError::ProtocolError("state lock poisoned".into()))? =
            ConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&self) -> WebSocketResult<()> {
        *self
            .state
            .write()
            .map_err(|_| WebSocketError::ProtocolError("state lock poisoned".into()))? =
            ConnectionStatus::Disconnected;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.state
            .read()
            .map(|g| *g)
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.active_subs
            .write()
            .map_err(|_| WebSocketError::ProtocolError("subs lock poisoned".into()))?
            .push(request.clone());

        let symbol_str = request
            .symbol
            .raw()
            .map(|s| s.to_string())
            .unwrap_or_else(|| request.symbol.to_concat());

        let key = StreamKey {
            exchange: self.id.as_str().to_string(),
            account: self.account.as_key_str().to_string(),
            symbol: symbol_str,
            stream_kind: stream_type_tag(&request.stream_type),
        };

        let storage = self.storage.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();

        let fut = async move {
            replay_task(storage, config, key, event_tx).await;
        };

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(fut);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(fut);

        Ok(())
    }

    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let mut subs = self
            .active_subs
            .write()
            .map_err(|_| WebSocketError::ProtocolError("subs lock poisoned".into()))?;
        subs.retain(|s| s != &request);
        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = BroadcastStream::new(self.event_tx.subscribe()).map(|r| match r {
            Ok(v) => v,
            Err(_lagged) => {
                Err(WebSocketError::ProtocolError("replay channel lagged".into()))
            }
        });
        Box::pin(rx)
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        self.active_subs
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities::permissive()
    }
}

// ── replay task ───────────────────────────────────────────────────────────────

async fn replay_task(
    storage: Arc<StorageManager>,
    config: ReplayConfig,
    key: StreamKey,
    event_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
) {
    let from = config.from_ms.unwrap_or(0);
    let to = config.to_ms.unwrap_or(i64::MAX);

    let records = match load_records(&storage, &key, from, to).await {
        Ok(r) => r,
        Err(e) => {
            let _ = event_tx.send(Err(WebSocketError::ProtocolError(format!(
                "replay read error: {e}"
            ))));
            return;
        }
    };

    if records.is_empty() {
        return;
    }

    // Record the real-time start point in a platform-compatible way.
    #[cfg(not(target_arch = "wasm32"))]
    let start_real = std::time::Instant::now();
    #[cfg(target_arch = "wasm32")]
    let start_real_ms = js_sys::Date::now();

    let start_sim = records[0].0;

    for (ts_ms, payload) in records {
        let event = match serde_json::from_slice::<StreamEvent>(&payload) {
            Ok(e) => e,
            Err(e) => {
                // Emit parse error and continue — don't abort the whole stream.
                let _ = event_tx.send(Err(WebSocketError::Parse(format!(
                    "replay decode: {e}"
                ))));
                continue;
            }
        };

        let sim_elapsed = ts_ms - start_sim;

        #[cfg(not(target_arch = "wasm32"))]
        let real_elapsed = start_real.elapsed().as_millis() as i64;
        #[cfg(target_arch = "wasm32")]
        let real_elapsed = (js_sys::Date::now() - start_real_ms) as i64;

        if let Some(delay) = config.rate.delay_for(sim_elapsed, real_elapsed) {
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(delay).await;
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::sleep(delay).await;
        }

        // No receivers left — no point continuing.
        if event_tx.send(Ok(event)).is_err() {
            break;
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn stream_type_tag(st: &StreamType) -> String {
    match st {
        StreamType::Ticker => "ticker".into(),
        StreamType::Trade => "trade".into(),
        StreamType::Orderbook => "orderbook".into(),
        StreamType::OrderbookDelta => "orderbook_delta".into(),
        StreamType::Kline { interval } => format!("kline_{interval}"),
        StreamType::MarkPrice => "mark_price".into(),
        StreamType::FundingRate => "funding_rate".into(),
        StreamType::Liquidation => "liquidation".into(),
        StreamType::OpenInterest => "open_interest".into(),
        StreamType::LongShortRatio => "long_short_ratio".into(),
        StreamType::AggTrade => "agg_trade".into(),
        StreamType::CompositeIndex => "composite_index".into(),
        StreamType::MarkPriceKline { interval } => format!("mark_price_kline_{interval}"),
        StreamType::IndexPriceKline { interval } => format!("index_price_kline_{interval}"),
        StreamType::PremiumIndexKline { interval } => format!("premium_index_kline_{interval}"),
        StreamType::IndexPrice => "index_price".into(),
        StreamType::HistoricalVolatility => "historical_volatility".into(),
        StreamType::InsuranceFund => "insurance_fund".into(),
        StreamType::Basis => "basis".into(),
        StreamType::OptionGreeks => "option_greeks".into(),
        StreamType::VolatilityIndex => "volatility_index".into(),
        StreamType::BlockTrade => "block_trade".into(),
        StreamType::AuctionEvent => "auction_event".into(),
        StreamType::MarketWarning => "market_warning".into(),
        StreamType::OrderbookL3 => "orderbook_l3".into(),
        StreamType::SettlementEvent => "settlement_event".into(),
        StreamType::RiskLimit => "risk_limit".into(),
        StreamType::PredictedFunding => "predicted_funding".into(),
        StreamType::FundingSettlement => "funding_settlement".into(),
        StreamType::OrderUpdate => "order_update".into(),
        StreamType::BalanceUpdate => "balance_update".into(),
        StreamType::PositionUpdate => "position_update".into(),
    }
}
