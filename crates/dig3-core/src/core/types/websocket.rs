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
    AccountType, AggTrade, AuctionEvent, Basis, BlockTrade, CompositeIndex, FundingRate,
    FundingSettlement, HistoricalVolatility, IndexPrice, InsuranceFund, Kline, Liquidation,
    LongShortRatio, MarginType, MarketWarning, MarkPrice, OpenInterest, OptionGreeks, OrderBook,
    OrderSide, OrderbookL3Event, OrderStatus, OrderType, OrderbookDelta as OrderbookDeltaData, PositionSide,
    PredictedFunding, Price, PublicTrade, Quantity, RiskLimit, SettlementEvent, Symbol, Ticker,
    Timestamp, VolatilityIndex,
};
use crate::core::websocket::stream_kind::KlineInterval;

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
    /// Public liquidation events (forced position closes)
    Liquidation,
    /// Open interest snapshots/updates (futures)
    OpenInterest,
    /// Long/short ratio (market sentiment, futures)
    LongShortRatio,
    /// Aggregated trade stream
    AggTrade,
    /// Composite index price
    CompositeIndex,
    /// Mark price kline (futures)
    MarkPriceKline { interval: String },
    /// Index price kline (futures)
    IndexPriceKline { interval: String },
    /// Premium index kline (futures)
    PremiumIndexKline { interval: String },
    /// Index price updates
    IndexPrice,
    /// Historical volatility (options)
    HistoricalVolatility,
    /// Insurance fund updates (futures)
    InsuranceFund,
    /// Basis (futures)
    Basis,
    /// Option Greeks (delta, gamma, vega, theta, rho) and implied volatility
    OptionGreeks,
    /// Volatility index (e.g. DVOL)
    VolatilityIndex,
    /// Block trade / RFQ event
    BlockTrade,
    /// Auction event (indicative price, matched state)
    AuctionEvent,
    /// Market warning / halt notification
    MarketWarning,
    /// Full order-level (L3) orderbook update
    OrderbookL3,
    /// Settlement event (expiry/delivery)
    SettlementEvent,
    /// Risk limit tier update
    RiskLimit,
    /// Predicted funding rate before settlement
    PredictedFunding,
    /// Funding settlement (actual paid rate)
    FundingSettlement,

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
    Ticker { symbol: String, ticker: Ticker },

    /// Новая публичная сделка
    Trade { symbol: String, trade: PublicTrade },

    /// Снепшот стакана
    OrderbookSnapshot { symbol: String, book: OrderBook },

    /// Инкрементальное обновление стакана
    OrderbookDelta { symbol: String, delta: OrderbookDeltaData },

    /// Обновление свечи
    Kline { symbol: String, interval: KlineInterval, kline: Kline },

    /// Mark price. Carries the full `MarkPrice` struct so the WS feed is lossless
    /// (estimated/indicative settle, interest rate, fair/spot price, …) — `symbol`
    /// stays on the variant as the routing key.
    MarkPrice { symbol: String, mark: MarkPrice },

    /// Funding rate. Carries the full `FundingRate` struct (premium, realized,
    /// sett state, interval, caps, …) — lossless WS, `symbol` is the routing key.
    FundingRate { symbol: String, funding: FundingRate },

    /// Public liquidation event (forced position close). Carries the full
    /// `Liquidation` struct (order id/type/status, avg/fill/order price, …).
    Liquidation { symbol: String, liquidation: Liquidation },

    /// Open interest update. Carries the full `OpenInterest` struct
    /// (ccy/usd, single/sum OI, trade rollups, …).
    OpenInterestUpdate { symbol: String, open_interest: OpenInterest },

    /// Long/short ratio update. Carries the full `LongShortRatio` struct
    /// (top-trader splits, user counts, buy/sell/locked, …).
    LongShortRatio { symbol: String, ratio: LongShortRatio },

    /// Aggregated trade (combines multiple trades at same price/time). Carries
    /// the full `AggTrade` struct (is_best_match, non_rpi_qty, quote_qty, …).
    AggTrade { symbol: String, agg: AggTrade },

    /// Composite index price update. Carries the full `CompositeIndex` struct.
    CompositeIndex { symbol: String, index: CompositeIndex },

    /// Mark price kline update (futures)
    MarkPriceKline { symbol: String, interval: KlineInterval, kline: Kline },

    /// Index price kline update (futures)
    IndexPriceKline { symbol: String, interval: KlineInterval, kline: Kline },

    /// Premium index kline update (futures)
    PremiumIndexKline { symbol: String, interval: KlineInterval, kline: Kline },

    /// Index price update. Carries the full `IndexPrice` struct.
    IndexPrice { symbol: String, index_price: IndexPrice },

    /// Historical volatility update (options). Carries the full `HistoricalVolatility` struct.
    HistoricalVolatility { symbol: String, hv: HistoricalVolatility },

    /// Insurance fund update (futures). Carries the full `InsuranceFund` struct.
    InsuranceFund { symbol: String, fund: InsuranceFund },

    /// Basis update (futures). Carries the full `Basis` struct (futures/index price,
    /// basis rate, annualized, contract type) — lossless WS.
    Basis { symbol: String, basis: Basis },

    /// Option Greeks and implied volatility. Carries the full `OptionGreeks` struct.
    OptionGreeks { symbol: String, greeks: OptionGreeks },

    /// Volatility index (e.g. DVOL.BTC). Carries the full `VolatilityIndex` struct
    /// (OHLC where the venue provides candles) — lossless WS.
    VolatilityIndex { symbol: String, vol_index: VolatilityIndex },

    /// Block trade / RFQ event. Carries the full `BlockTrade` struct.
    BlockTrade { symbol: String, block: BlockTrade },

    /// Auction event (indicative price, matched state). Carries the full `AuctionEvent` struct.
    AuctionEvent { symbol: String, auction: AuctionEvent },

    /// Market warning / halt notification. Carries the full `MarketWarning` struct.
    ///
    /// `symbol` on the variant is `None` for venue-wide notifications (e.g.
    /// Hyperliquid global `notifications` channel); `Some(s)` for per-instrument
    /// warnings. The inner struct's `symbol` mirrors the per-instrument case.
    MarketWarning { symbol: Option<String>, warning: MarketWarning },

    /// Full order-level (L3) orderbook update. Carries the full `OrderbookL3Event` struct.
    OrderbookL3 { symbol: String, event: OrderbookL3Event },

    /// Settlement event (expiry/delivery). Carries the full `SettlementEvent` struct.
    SettlementEvent { symbol: String, settlement: SettlementEvent },

    /// Risk limit tier update. Carries the full `RiskLimit` struct.
    RiskLimit { symbol: String, risk_limit: RiskLimit },

    /// Predicted funding rate before settlement. Carries the full `PredictedFunding` struct.
    PredictedFunding { symbol: String, predicted: PredictedFunding },

    /// Funding settlement (actual paid rate). Carries the full `FundingSettlement` struct.
    FundingSettlement {
        symbol: String,
        settlement: FundingSettlement,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // PRIVATE EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Обновление ордера
    OrderUpdate { symbol: String, event: OrderUpdateEvent },

    /// Обновление баланса
    BalanceUpdate(BalanceUpdateEvent),

    /// Обновление позиции (Futures)
    PositionUpdate { symbol: String, event: PositionUpdateEvent },

    /// Batch fan-out — one wire frame produced N events of any variant.
    ///
    /// Use when a single WS frame carries multiple homogeneous payloads that
    /// must each become a distinct downstream event. Primary use case: WS
    /// `trades` channels that pack many trades per frame (HyperLiquid sends
    /// up to ~16 trades in one `data: [...]`). The transport-layer
    /// dispatcher flattens `Batch(vec)` and re-emits each contained event,
    /// so consumers see N events not one — preserving the lossless contract.
    ///
    /// Nesting (`Batch` inside `Batch`) is supported by the flattener but
    /// should be avoided.
    Batch(Vec<StreamEvent>),
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
