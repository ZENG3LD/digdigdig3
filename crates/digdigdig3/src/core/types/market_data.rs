//! # Market Data Types
//!
//! Типы для рыночных данных.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// KLINE / OHLCV
// ═══════════════════════════════════════════════════════════════════════════════

/// Свеча (OHLCV)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    /// Время открытия (Unix timestamp в миллисекундах)
    pub open_time: i64,
    /// Цена открытия
    pub open: f64,
    /// Максимальная цена
    pub high: f64,
    /// Минимальная цена
    pub low: f64,
    /// Цена закрытия
    pub close: f64,
    /// Объём в базовом активе
    pub volume: f64,
    /// Объём в котируемом активе (опционально)
    pub quote_volume: Option<f64>,
    /// Время закрытия (опционально)
    pub close_time: Option<i64>,
    /// Количество сделок (опционально)
    pub trades: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TICKER
// ═══════════════════════════════════════════════════════════════════════════════

/// Тикер (24h статистика)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    /// Последняя цена
    pub last_price: f64,
    /// Лучший bid
    pub bid_price: Option<f64>,
    /// Лучший ask
    pub ask_price: Option<f64>,
    /// Максимум за 24h
    pub high_24h: Option<f64>,
    /// Минимум за 24h
    pub low_24h: Option<f64>,
    /// Объём за 24h (в базовом активе)
    pub volume_24h: Option<f64>,
    /// Объём за 24h (в котируемом активе)
    pub quote_volume_24h: Option<f64>,
    /// Изменение цены за 24h
    pub price_change_24h: Option<f64>,
    /// Изменение цены в процентах за 24h
    pub price_change_percent_24h: Option<f64>,
    /// Timestamp
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER BOOK
// ═══════════════════════════════════════════════════════════════════════════════

/// One price level in the order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub size: f64,
    /// Number of orders at this level (some exchanges provide this).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_count: Option<u32>,
}

impl OrderBookLevel {
    pub fn new(price: f64, size: f64) -> Self {
        Self { price, size, order_count: None }
    }

    pub fn with_count(price: f64, size: f64, count: u32) -> Self {
        Self { price, size, order_count: Some(count) }
    }
}

/// Convert from tuple for backwards compat
impl From<(f64, f64)> for OrderBookLevel {
    fn from((price, size): (f64, f64)) -> Self {
        Self::new(price, size)
    }
}

/// Снепшот стакана
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderBook {
    /// Bids - отсортированы по убыванию цены
    pub bids: Vec<OrderBookLevel>,
    /// Asks - отсортированы по возрастанию цены
    pub asks: Vec<OrderBookLevel>,
    /// Timestamp
    pub timestamp: i64,
    /// Sequence number (опционально, для инкрементальных обновлений)
    pub sequence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<i64>,
}

impl OrderBook {
    /// Simple constructor from tuples (backwards compat helper)
    pub fn simple(bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>, timestamp: i64) -> Self {
        Self {
            bids: bids.into_iter().map(OrderBookLevel::from).collect(),
            asks: asks.into_iter().map(OrderBookLevel::from).collect(),
            timestamp,
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        }
    }

    /// Construct from tuple slices — convenience for tests.
    pub fn from_tuples(bids: &[(f64, f64)], asks: &[(f64, f64)], timestamp: i64) -> Self {
        Self {
            bids: bids.iter().map(|&(p, s)| OrderBookLevel::new(p, s)).collect(),
            asks: asks.iter().map(|&(p, s)| OrderBookLevel::new(p, s)).collect(),
            timestamp,
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        }
    }

    /// Best bid level (highest price).
    pub fn best_bid(&self) -> Option<&OrderBookLevel> {
        self.bids.first()
    }

    /// Best ask level (lowest price).
    pub fn best_ask(&self) -> Option<&OrderBookLevel> {
        self.asks.first()
    }

    /// Mid price: (best_bid + best_ask) / 2.
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(b), Some(a)) => Some((b.price + a.price) / 2.0),
            _ => None,
        }
    }

    /// Spread: best_ask - best_bid.
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(b), Some(a)) => Some(a.price - b.price),
            _ => None,
        }
    }

    /// Sum of bid sizes up to `levels` levels.
    pub fn bid_depth(&self, levels: usize) -> f64 {
        self.bids.iter().take(levels).map(|l| l.size).sum()
    }

    /// Sum of ask sizes up to `levels` levels.
    pub fn ask_depth(&self, levels: usize) -> f64 {
        self.asks.iter().take(levels).map(|l| l.size).sum()
    }
}

/// Incremental order-book update.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderbookDelta {
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<i64>,
}

impl OrderbookDelta {
    /// Levels that were removed on bid side (size == 0.0).
    pub fn removed_bids(&self) -> impl Iterator<Item = f64> + '_ {
        self.bids.iter().filter(|l| l.size == 0.0).map(|l| l.price)
    }

    /// Levels that were removed on ask side (size == 0.0).
    pub fn removed_asks(&self) -> impl Iterator<Item = f64> + '_ {
        self.asks.iter().filter(|l| l.size == 0.0).map(|l| l.price)
    }

    /// Levels that were added or updated on bid side (size > 0.0).
    pub fn updated_bids(&self) -> impl Iterator<Item = &OrderBookLevel> {
        self.bids.iter().filter(|l| l.size > 0.0)
    }

    /// Levels that were added or updated on ask side (size > 0.0).
    pub fn updated_asks(&self) -> impl Iterator<Item = &OrderBookLevel> {
        self.asks.iter().filter(|l| l.size > 0.0)
    }

    /// Total number of changed levels across both sides.
    pub fn total_changes(&self) -> usize {
        self.bids.len() + self.asks.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING RATE (Futures)
// ═══════════════════════════════════════════════════════════════════════════════

/// Информация о funding rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRate {
    /// Символ
    pub symbol: String,
    /// Текущий funding rate
    pub rate: f64,
    /// Время следующего funding
    pub next_funding_time: Option<i64>,
    /// Timestamp
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARK PRICE (Futures)
// ═══════════════════════════════════════════════════════════════════════════════

/// Mark price информация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkPrice {
    /// Символ
    pub symbol: String,
    /// Mark price
    pub mark_price: f64,
    /// Index price (опционально)
    pub index_price: Option<f64>,
    /// Current funding rate (опционально — только для перпетуальных контрактов)
    pub funding_rate: Option<f64>,
    /// Timestamp
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPEN INTEREST (Futures)
// ═══════════════════════════════════════════════════════════════════════════════

/// Open Interest информация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenInterest {
    /// Символ
    pub symbol: String,
    /// Open interest (в контрактах или базовом активе)
    pub open_interest: f64,
    /// Open interest в USDT (опционально)
    pub open_interest_value: Option<f64>,
    /// Timestamp
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC TRADE
// ═══════════════════════════════════════════════════════════════════════════════

/// Публичная сделка (recent trades)
///
/// Отличается от Trade тем, что это публичные данные с ленты,
/// а Trade может содержать информацию о собственных сделках (fills)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicTrade {
    /// ID сделки
    pub id: String,
    /// Цена
    pub price: f64,
    /// Количество
    pub quantity: f64,
    /// Сторона (buyer был taker?)
    pub side: TradeSide,
    /// Timestamp
    pub timestamp: i64,
}

/// Сторона сделки в публичной ленте
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    /// Покупатель был taker (цена пошла вверх)
    Buy,
    /// Продавец был taker (цена пошла вниз)
    Sell,
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIQUIDATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Public liquidation event (forced position close).
///
/// Available from exchanges with public liquidation feeds (Binance Futures
/// `/fapi/v1/forceOrders`, Bybit/Hyperliquid streams).
///
/// Semantics of `side`:
/// - `Buy`  — a **long** position was liquidated (forced sell into market).
/// - `Sell` — a **short** position was liquidated (forced buy into market).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liquidation {
    /// Trading pair symbol (e.g. "BTCUSDT").
    pub symbol: String,
    /// Side of the LIQUIDATED position.
    /// `Buy` = long was liquidated (exchange sold); `Sell` = short was liquidated (exchange bought).
    pub side: TradeSide,
    /// Fill price of the liquidation order.
    pub price: f64,
    /// Fill quantity in base asset.
    pub quantity: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
    /// Quote value (price × quantity). `None` when not provided by exchange.
    pub value: Option<f64>,
}

impl Liquidation {
    /// Quote value — uses `self.value` when present, otherwise `price * quantity`.
    #[inline]
    pub fn quote_value(&self) -> f64 {
        self.value.unwrap_or(self.price * self.quantity)
    }
}
