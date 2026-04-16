//! # WebSocket Types
//!
//! Типы для WebSocket соединений.
//!
//! ## Public Streams (без авторизации)
//! - Ticker, Trade, Orderbook, Kline, MarkPrice, FundingRate
//!
//! ## Private Streams (требуют авторизации)
//! - OrderUpdate - изменения ордеров
//! - BalanceUpdate - изменения баланса
//! - PositionUpdate - изменения позиций (Futures)

use serde::{Deserialize, Serialize};

use super::{
    AccountType, Kline, MarginType, OrderBook, OrderSide, OrderStatus, OrderType,
    OrderbookDelta as OrderbookDeltaData, PositionSide, Price, PublicTrade, Quantity, Symbol,
    Ticker, Timestamp,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTION STATUS
// ═══════════════════════════════════════════════════════════════════════════════

/// Статус WebSocket соединения
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Отключено
    Disconnected,
    /// Подключается
    Connecting,
    /// Подключено
    Connected,
    /// Переподключается
    Reconnecting,
}

// ═══════════════════════════════════════════════════════════════════════════════
// STREAM TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Тип потока данных
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamType {
    // ═══════════════════════════════════════════════════════════════════════════
    // PUBLIC STREAMS (без авторизации)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Тикер
    Ticker,
    /// Сделки
    Trade,
    /// Снепшот стакана
    Orderbook,
    /// Инкрементальные обновления стакана
    OrderbookDelta,
    /// Свечи с указанным интервалом
    Kline { interval: String },
    /// Mark price (futures)
    MarkPrice,
    /// Funding rate (futures)
    FundingRate,

    // ═══════════════════════════════════════════════════════════════════════════
    // PRIVATE STREAMS (требуют авторизации)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Обновления ордеров (создание, исполнение, отмена)
    ///
    /// # Биржевые топики:
    /// - Binance Spot: executionReport
    /// - Binance Futures: ORDER_TRADE_UPDATE
    /// - Bybit: order
    /// - OKX: orders
    /// - KuCoin Spot: /spotMarket/tradeOrdersV2
    /// - KuCoin Futures: /contractMarket/tradeOrders
    OrderUpdate,

    /// Обновления баланса
    ///
    /// # Биржевые топики:
    /// - Binance Spot: outboundAccountPosition, balanceUpdate
    /// - Binance Futures: ACCOUNT_UPDATE (balance part)
    /// - Bybit: wallet
    /// - OKX: account
    /// - KuCoin Spot: /account/balance
    /// - KuCoin Futures: /contractAccount/wallet
    BalanceUpdate,

    /// Обновления позиций (только Futures)
    ///
    /// # Биржевые топики:
    /// - Binance Futures: ACCOUNT_UPDATE (position part)
    /// - Bybit: position
    /// - OKX: positions
    /// - KuCoin Futures: /contract/position
    PositionUpdate,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION
// ═══════════════════════════════════════════════════════════════════════════════

/// Запрос на подписку
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    /// Символ
    pub symbol: Symbol,
    /// Тип потока
    pub stream_type: StreamType,
    /// Account / market type (Spot, FuturesCross, etc.). Defaults to Spot.
    #[serde(default)]
    pub account_type: AccountType,
    /// Number of price levels to request (connector default if None)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    /// Update speed in ms (connector default if None)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_speed_ms: Option<u32>,
}

impl SubscriptionRequest {
    pub fn new(symbol: Symbol, stream_type: StreamType) -> Self {
        Self {
            symbol,
            stream_type,
            account_type: AccountType::default(),
            depth: None,
            update_speed_ms: None,
        }
    }

    pub fn ticker(symbol: Symbol) -> Self {
        Self::new(symbol, StreamType::Ticker)
    }

    pub fn ticker_for(symbol: Symbol, account_type: AccountType) -> Self {
        Self { symbol, stream_type: StreamType::Ticker, account_type, depth: None, update_speed_ms: None }
    }

    pub fn trade(symbol: Symbol) -> Self {
        Self::new(symbol, StreamType::Trade)
    }

    pub fn trade_for(symbol: Symbol, account_type: AccountType) -> Self {
        Self { symbol, stream_type: StreamType::Trade, account_type, depth: None, update_speed_ms: None }
    }

    pub fn orderbook(symbol: Symbol) -> Self {
        Self::new(symbol, StreamType::Orderbook)
    }

    pub fn kline(symbol: Symbol, interval: impl Into<String>) -> Self {
        Self::new(symbol, StreamType::Kline { interval: interval.into() })
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn with_speed(mut self, ms: u32) -> Self {
        self.update_speed_ms = Some(ms);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STREAM EVENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// События от WebSocket потока
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    // ═══════════════════════════════════════════════════════════════════════════
    // PUBLIC EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Обновление тикера
    Ticker(Ticker),

    /// Новая публичная сделка
    Trade(PublicTrade),

    /// Снепшот стакана
    OrderbookSnapshot(OrderBook),

    /// Инкрементальное обновление стакана
    OrderbookDelta(OrderbookDeltaData),

    /// Обновление свечи
    Kline(Kline),

    /// Mark price
    MarkPrice {
        symbol: String,
        mark_price: f64,
        index_price: Option<f64>,
        timestamp: i64,
    },

    /// Funding rate
    FundingRate {
        symbol: String,
        rate: f64,
        next_funding_time: Option<i64>,
        timestamp: i64,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // PRIVATE EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Обновление ордера
    OrderUpdate(OrderUpdateEvent),

    /// Обновление баланса
    BalanceUpdate(BalanceUpdateEvent),

    /// Обновление позиции (Futures)
    PositionUpdate(PositionUpdateEvent),
}

// ═══════════════════════════════════════════════════════════════════════════════
// PRIVATE STREAM EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Событие обновления ордера
///
/// Приходит при любом изменении ордера:
/// - Создание (New)
/// - Частичное исполнение (PartiallyFilled)
/// - Полное исполнение (Filled)
/// - Отмена (Canceled)
/// - Истечение (Expired)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdateEvent {
    /// ID ордера
    pub order_id: String,
    /// Client Order ID
    pub client_order_id: Option<String>,
    /// Символ
    pub symbol: String,
    /// Направление
    pub side: OrderSide,
    /// Тип ордера
    pub order_type: OrderType,
    /// Текущий статус
    pub status: OrderStatus,
    /// Цена ордера (для Limit)
    pub price: Option<Price>,
    /// Количество ордера
    pub quantity: Quantity,
    /// Исполненное количество
    pub filled_quantity: Quantity,
    /// Средняя цена исполнения
    pub average_price: Option<Price>,

    // Информация о последнем fill (если есть)
    /// Цена последнего fill
    pub last_fill_price: Option<Price>,
    /// Количество последнего fill
    pub last_fill_quantity: Option<Quantity>,
    /// Комиссия последнего fill
    pub last_fill_commission: Option<Price>,
    /// Актив комиссии
    pub commission_asset: Option<String>,
    /// Trade ID последнего fill
    pub trade_id: Option<String>,

    /// Timestamp события
    pub timestamp: Timestamp,
}

/// Событие обновления баланса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdateEvent {
    /// Актив
    pub asset: String,
    /// Доступный баланс (после изменения)
    pub free: Price,
    /// Заблокированный баланс
    pub locked: Price,
    /// Общий баланс
    pub total: Price,
    /// Изменение баланса (может быть отрицательным)
    pub delta: Option<Price>,
    /// Причина изменения
    pub reason: Option<BalanceChangeReason>,
    /// Timestamp
    pub timestamp: Timestamp,
}

/// Причина изменения баланса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalanceChangeReason {
    /// Deposit
    Deposit,
    /// Withdrawal
    Withdraw,
    /// Торговая операция (fill)
    Trade,
    /// Комиссия
    Commission,
    /// Funding (Futures)
    Funding,
    /// PnL реализация (Futures)
    RealizedPnl,
    /// Перевод между аккаунтами
    Transfer,
    /// Другое/неизвестно
    Other,
}

/// Событие обновления позиции (Futures)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdateEvent {
    /// Символ
    pub symbol: String,
    /// Сторона позиции
    pub side: PositionSide,
    /// Размер позиции
    pub quantity: Quantity,
    /// Цена входа
    pub entry_price: Price,
    /// Mark price
    pub mark_price: Option<Price>,
    /// Unrealized PnL
    pub unrealized_pnl: Price,
    /// Realized PnL (за сессию)
    pub realized_pnl: Option<Price>,
    /// Цена ликвидации
    pub liquidation_price: Option<Price>,
    /// Leverage
    pub leverage: Option<u32>,
    /// Margin type
    pub margin_type: Option<MarginType>,
    /// Причина изменения
    pub reason: Option<PositionChangeReason>,
    /// Timestamp
    pub timestamp: Timestamp,
}

/// Причина изменения позиции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionChangeReason {
    /// Открытие/увеличение позиции
    Trade,
    /// Изменение leverage
    LeverageChange,
    /// Изменение margin
    MarginChange,
    /// Ликвидация
    Liquidation,
    /// ADL (Auto-Deleveraging)
    Adl,
    /// Funding
    Funding,
    /// Другое
    Other,
}
