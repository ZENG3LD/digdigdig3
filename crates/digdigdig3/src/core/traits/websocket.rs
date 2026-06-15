//! # WebSocket Core Traits
//!
//! Минимальные WebSocket трейты для core функциональности.
//!
//! ## Public Streams (без авторизации)
//! - Ticker, Trade, Orderbook, Kline, MarkPrice, FundingRate
//!
//! ## Private Streams (требуют авторизации)
//! - OrderUpdate, BalanceUpdate, PositionUpdate

use std::pin::Pin;
use std::sync::Arc;

use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::types::{
    AccountType, ConnectionStatus, OrderbookCapabilities, StreamEvent, StreamType,
    SubscriptionRequest, Symbol, WebSocketResult,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CORE WEBSOCKET TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Core WebSocket коннектор
///
/// Минимальный интерфейс для WebSocket подключений.
///
/// # Методы
/// - `connect` - подключиться
/// - `disconnect` - отключиться
/// - `connection_status` - статус
/// - `subscribe` - подписаться на поток
/// - `unsubscribe` - отписаться
/// - `event_stream` - получить поток событий
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait WebSocketConnector: Send + Sync {
    /// Подключиться к WebSocket
    async fn connect(&self, account_type: AccountType) -> WebSocketResult<()> {
        let _ = account_type;
        Err(crate::core::types::WebSocketError::NotImplemented(
            "WebSocket not supported".into(),
        ))
    }

    /// Отключиться от WebSocket
    async fn disconnect(&self) -> WebSocketResult<()> {
        Err(crate::core::types::WebSocketError::NotImplemented(
            "WebSocket not supported".into(),
        ))
    }

    /// Получить текущий статус подключения
    fn connection_status(&self) -> ConnectionStatus {
        ConnectionStatus::Disconnected
    }

    /// Подписаться на поток данных
    async fn subscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let _ = request;
        Err(crate::core::types::WebSocketError::NotImplemented(
            "WebSocket not supported".into(),
        ))
    }

    /// Отписаться от потока данных
    async fn unsubscribe(&self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let _ = request;
        Err(crate::core::types::WebSocketError::NotImplemented(
            "WebSocket not supported".into(),
        ))
    }

    /// Получить поток событий
    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        Box::pin(futures_util::stream::empty())
    }

    /// Получить список активных подписок
    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        Vec::new()
    }

    /// Проверить наличие подписки
    fn has_subscription(&self, request: &SubscriptionRequest) -> bool {
        self.active_subscriptions().contains(request)
    }

    /// Get a shared handle to the WS ping round-trip time (ms).
    ///
    /// Returns `None` by default.  Connectors that measure ping/pong RTT
    /// should override this and return `Some(arc)` pointing to a `u64`
    /// that is updated on every pong.
    fn ping_rtt_handle(&self) -> Option<Arc<TokioMutex<u64>>> {
        None
    }

    /// Returns the exchange's L2/orderbook capabilities for the given account type.
    ///
    /// Connectors with different capabilities per market type (e.g. Binance Spot vs Futures)
    /// should match on `account_type` and return the appropriate struct.
    /// The default implementation ignores `account_type` and returns permissive defaults.
    fn orderbook_capabilities(&self, account_type: AccountType) -> OrderbookCapabilities {
        let _ = account_type;
        OrderbookCapabilities::permissive()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Удобные методы подписки
///
/// Автоматически реализуется для всех `WebSocketConnector`.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait WebSocketExt: WebSocketConnector {
    // ═══════════════════════════════════════════════════════════════════════════
    // PUBLIC STREAMS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Подписаться на тикер
    ///
    /// `symbol` — raw exchange-native string (e.g. `"BTCUSDT"`, `"BTC-USDT"`).
    async fn subscribe_ticker(&self, symbol: String) -> WebSocketResult<()> {
        let sym = Symbol::with_raw("", "", symbol);
        self.subscribe(SubscriptionRequest::ticker(sym)).await
    }

    /// Подписаться на сделки
    ///
    /// `symbol` — raw exchange-native string.
    async fn subscribe_trades(&self, symbol: String) -> WebSocketResult<()> {
        let sym = Symbol::with_raw("", "", symbol);
        self.subscribe(SubscriptionRequest::trade(sym)).await
    }

    /// Подписаться на стакан
    ///
    /// `symbol` — raw exchange-native string.
    async fn subscribe_orderbook(&self, symbol: String) -> WebSocketResult<()> {
        let sym = Symbol::with_raw("", "", symbol);
        self.subscribe(SubscriptionRequest::orderbook(sym)).await
    }

    /// Подписаться на свечи
    ///
    /// `symbol` — raw exchange-native string.
    async fn subscribe_klines(&self, symbol: String, interval: &str) -> WebSocketResult<()> {
        let sym = Symbol::with_raw("", "", symbol);
        self.subscribe(SubscriptionRequest::kline(sym, interval)).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PRIVATE STREAMS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Подписаться на обновления ордеров (private)
    async fn subscribe_orders(&self) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::new(
            Symbol::with_raw("", "", String::new()),
            StreamType::OrderUpdate,
        ))
        .await
    }

    /// Подписаться на обновления баланса (private)
    async fn subscribe_balance(&self) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::new(
            Symbol::with_raw("", "", String::new()),
            StreamType::BalanceUpdate,
        ))
        .await
    }

    /// Подписаться на обновления позиций (private, futures)
    async fn subscribe_positions(&self) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::new(
            Symbol::with_raw("", "", String::new()),
            StreamType::PositionUpdate,
        ))
        .await
    }
}

// Blanket implementation
impl<T: WebSocketConnector> WebSocketExt for T {}
