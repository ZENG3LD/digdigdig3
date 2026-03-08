//! # Trading Types
//!
//! Типы для торговых операций V5.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

/// Цена
pub type Price = f64;

/// Количество
pub type Quantity = f64;

/// Актив (USDT, BTC, etc.)
pub type Asset = String;

/// Timestamp в миллисекундах
pub type Timestamp = i64;

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Направление ордера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// Тип ордера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Market => "MARKET",
            Self::Limit => "LIMIT",
            Self::StopLoss => "STOP_LOSS",
            Self::StopLossLimit => "STOP_LOSS_LIMIT",
            Self::TakeProfit => "TAKE_PROFIT",
            Self::TakeProfitLimit => "TAKE_PROFIT_LIMIT",
        }
    }
}

/// Статус ордера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Новый, еще не на рынке
    New,
    /// Активный на рынке
    Open,
    /// Частично исполнен
    PartiallyFilled,
    /// Полностью исполнен
    Filled,
    /// Отменен
    Canceled,
    /// Отклонен
    Rejected,
    /// Истек (для GTC/GTD)
    Expired,
}

/// Time in Force
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeInForce {
    /// Good Till Cancel (по умолчанию)
    #[default]
    GTC,
    /// Immediate or Cancel
    IOC,
    /// Fill or Kill
    FOK,
    /// Good Till Date
    GTD,
    /// Post Only
    PostOnly,
}

impl TimeInForce {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GTC => "GTC",
            Self::IOC => "IOC",
            Self::FOK => "FOK",
            Self::GTD => "GTD",
            Self::PostOnly => "POST_ONLY",
        }
    }
}

/// Ордер
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// ID ордера (на бирже)
    pub id: String,
    /// Client Order ID
    pub client_order_id: Option<String>,
    /// Символ
    pub symbol: String,
    /// Направление
    pub side: OrderSide,
    /// Тип
    pub order_type: OrderType,
    /// Статус
    pub status: OrderStatus,
    /// Цена (для лимитных)
    pub price: Option<Price>,
    /// Стоп-цена (для стоп-ордеров)
    pub stop_price: Option<Price>,
    /// Количество
    pub quantity: Quantity,
    /// Исполненное количество
    pub filled_quantity: Quantity,
    /// Средняя цена исполнения
    pub average_price: Option<Price>,
    /// Комиссия
    pub commission: Option<Price>,
    /// Актив комиссии
    pub commission_asset: Option<String>,
    /// Время создания
    pub created_at: Timestamp,
    /// Время обновления
    pub updated_at: Option<Timestamp>,
    /// Time in force
    pub time_in_force: TimeInForce,
}

/// Запрос на создание ордера
#[derive(Debug, Clone)]
pub struct CreateOrderRequest {
    pub symbol: super::Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>,
    pub stop_price: Option<Price>,
    pub time_in_force: TimeInForce,
    pub account_type: super::AccountType,
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITION TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Режим позиций
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PositionMode {
    /// Односторонний режим (одна позиция на символ)
    #[default]
    OneWay,
    /// Двусторонний режим (отдельные Long/Short)
    Hedge,
}

/// Сторона позиции
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
    /// Для OneWay режима, определяется знаком quantity
    Both,
}

/// Позиция
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Символ
    pub symbol: String,
    /// Сторона позиции
    pub side: PositionSide,
    /// Размер позиции (может быть отрицательным для Short)
    pub quantity: Quantity,
    /// Цена входа
    pub entry_price: Price,
    /// Mark price
    pub mark_price: Option<Price>,
    /// Нереализованная прибыль/убыток
    pub unrealized_pnl: Price,
    /// Реализованная прибыль/убыток
    pub realized_pnl: Option<Price>,
    /// Цена ликвидации
    pub liquidation_price: Option<Price>,
    /// Leverage
    pub leverage: u32,
    /// Тип маржи (Cross/Isolated)
    pub margin_type: MarginType,
    /// Маржа
    pub margin: Option<Price>,
    /// Take Profit цена
    pub take_profit: Option<Price>,
    /// Stop Loss цена
    pub stop_loss: Option<Price>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Баланс
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Актив
    pub asset: String,
    /// Доступный баланс
    pub free: f64,
    /// Заблокированный баланс (в ордерах)
    pub locked: f64,
    /// Общий баланс
    pub total: f64,
}

/// Информация об аккаунте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Тип аккаунта
    pub account_type: super::AccountType,
    /// Может торговать
    pub can_trade: bool,
    /// Может выводить
    pub can_withdraw: bool,
    /// Может депозитить
    pub can_deposit: bool,
    /// Maker комиссия (в процентах)
    pub maker_commission: f64,
    /// Taker комиссия (в процентах)
    pub taker_commission: f64,
    /// Балансы
    pub balances: Vec<Balance>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// USER TRADE (MY TRADES / FILLS)
// ═══════════════════════════════════════════════════════════════════════════════

/// Собственная сделка (fill) - результат исполнения ордера
///
/// Отличается от PublicTrade тем, что содержит информацию
/// о комиссиях и связь с ордером
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTrade {
    /// ID сделки
    pub id: String,
    /// ID ордера
    pub order_id: String,
    /// Символ
    pub symbol: String,
    /// Направление (Buy/Sell)
    pub side: OrderSide,
    /// Цена исполнения
    pub price: Price,
    /// Количество
    pub quantity: Quantity,
    /// Комиссия
    pub commission: Price,
    /// Актив комиссии
    pub commission_asset: String,
    /// Был ли maker
    pub is_maker: bool,
    /// Timestamp
    pub timestamp: Timestamp,
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE INFO TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Информация о символе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Символ (как на бирже)
    pub symbol: String,
    /// Базовый актив
    pub base_asset: String,
    /// Котируемый актив
    pub quote_asset: String,
    /// Статус (TRADING, BREAK, etc.)
    pub status: String,
    /// Точность цены
    pub price_precision: u8,
    /// Точность количества
    pub quantity_precision: u8,
    /// Минимальное количество
    pub min_quantity: Option<f64>,
    /// Максимальное количество
    pub max_quantity: Option<f64>,
    /// Шаг количества
    pub step_size: Option<f64>,
    /// Минимальный notional (price * qty)
    pub min_notional: Option<f64>,
}

/// Информация о бирже
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    /// Время сервера
    pub server_time: Option<Timestamp>,
    /// Символы
    pub symbols: Vec<SymbolInfo>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARGIN TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Тип маржи
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginType {
    Cross,
    Isolated,
}

/// Результат займа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginBorrowResult {
    pub transaction_id: String,
    pub asset: String,
    pub amount: f64,
}

/// Результат погашения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRepayResult {
    pub transaction_id: String,
    pub asset: String,
    pub amount: f64,
}

/// Информация о займе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginLoan {
    pub asset: String,
    pub borrowed: f64,
    pub interest: f64,
    pub total: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSFER TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Результат перевода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    pub transaction_id: String,
    pub asset: String,
    pub amount: f64,
}

/// История переводов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistory {
    pub transaction_id: String,
    pub asset: String,
    pub amount: f64,
    pub from_account: String,
    pub to_account: String,
    pub timestamp: Timestamp,
}

// ═══════════════════════════════════════════════════════════════════════════════
// LISTEN KEY (Binance specific)
// ═══════════════════════════════════════════════════════════════════════════════

/// Listen Key для User Data Stream (Binance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenKey {
    pub key: String,
    pub expires_at: Option<Timestamp>,
}
