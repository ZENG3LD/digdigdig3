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

use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::Mutex as TokioMutex;

use crate::core::types::{
    AccountType, ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest, Symbol,
    WebSocketResult,
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
#[async_trait]
pub trait WebSocketConnector: Send + Sync {
    /// Подключиться к WebSocket
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()>;

    /// Отключиться от WebSocket
    async fn disconnect(&mut self) -> WebSocketResult<()>;

    /// Получить текущий статус подключения
    fn connection_status(&self) -> ConnectionStatus;

    /// Подписаться на поток данных
    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>;

    /// Отписаться от потока данных
    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>;

    /// Получить поток событий
    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>>;

    /// Получить список активных подписок
    fn active_subscriptions(&self) -> Vec<SubscriptionRequest>;

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
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Удобные методы подписки
///
/// Автоматически реализуется для всех `WebSocketConnector`.
#[async_trait]
pub trait WebSocketExt: WebSocketConnector {
    // ═══════════════════════════════════════════════════════════════════════════
    // PUBLIC STREAMS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Подписаться на тикер
    async fn subscribe_ticker(&mut self, symbol: Symbol) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::ticker(symbol)).await
    }

    /// Подписаться на сделки
    async fn subscribe_trades(&mut self, symbol: Symbol) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::trade(symbol)).await
    }

    /// Подписаться на стакан
    async fn subscribe_orderbook(&mut self, symbol: Symbol) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::orderbook(symbol)).await
    }

    /// Подписаться на свечи
    async fn subscribe_klines(&mut self, symbol: Symbol, interval: &str) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::kline(symbol, interval)).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PRIVATE STREAMS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Подписаться на обновления ордеров (private)
    async fn subscribe_orders(&mut self) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::new(
            Symbol::empty(),
            StreamType::OrderUpdate,
        ))
        .await
    }

    /// Подписаться на обновления баланса (private)
    async fn subscribe_balance(&mut self) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::new(
            Symbol::empty(),
            StreamType::BalanceUpdate,
        ))
        .await
    }

    /// Подписаться на обновления позиций (private, futures)
    async fn subscribe_positions(&mut self) -> WebSocketResult<()> {
        self.subscribe(SubscriptionRequest::new(
            Symbol::empty(),
            StreamType::PositionUpdate,
        ))
        .await
    }
}

// Blanket implementation
impl<T: WebSocketConnector> WebSocketExt for T {}
