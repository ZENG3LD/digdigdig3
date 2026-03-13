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
    /// Символ
    pub symbol: String,
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

/// Снепшот стакана
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// Bids (цена, размер) - отсортированы по убыванию цены
    pub bids: Vec<(f64, f64)>,
    /// Asks (цена, размер) - отсортированы по возрастанию цены
    pub asks: Vec<(f64, f64)>,
    /// Timestamp
    pub timestamp: i64,
    /// Sequence number (опционально, для инкрементальных обновлений)
    pub sequence: Option<String>,
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
    /// Символ
    pub symbol: String,
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
