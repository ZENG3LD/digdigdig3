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
// ORDERBOOK CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Algorithm used to compute the orderbook integrity checksum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    /// CRC-32 over interleaved top-N bid+ask price:qty strings (OKX/Bitget format).
    Crc32Interleaved,
    /// CRC-32 over asks_string + bids_string with decimal-stripped numeric strings (Kraken format).
    Crc32KrakenFormat,
    /// CRC-32, exact algorithm TBD (used by Crypto.com `cs` field).
    Crc32Generic,
    /// CRC-32 over interleaved top-25 bid+ask with order IDs (Bitfinex R0).
    Crc32BitfinexRaw,
}

/// Describes the checksum coverage and algorithm for a WS orderbook channel.
#[derive(Debug, Clone, Copy)]
pub struct ChecksumInfo {
    /// Algorithm used.
    pub algorithm: ChecksumAlgorithm,
    /// Number of levels per side covered by the checksum (e.g. 10 for Kraken, 25 for OKX/Bitget).
    pub levels_per_side: u32,
    /// Whether the checksum is opt-in (must be enabled via flags, e.g. Bitfinex OB_CHECKSUM).
    pub opt_in: bool,
}

/// Describes one named WebSocket orderbook channel variant.
///
/// Some exchanges expose multiple named channels with distinct depth/speed/update-model
/// characteristics (OKX books vs books5; KuCoin level2 vs level2Depth5; HTX mbp vs depth).
/// Each variant is described here. The ws_manager picks the best-fit channel at subscription
/// time using `ws_channels` instead of raw `ws_depths` / `update_speeds_ms`.
///
/// All fields are `Copy`-safe and use `'static` lifetimes for zero-alloc use.
#[derive(Debug, Clone, Copy)]
pub struct WsBookChannel {
    /// Exchange-specific channel or topic name (e.g. "books5", "mbp.150", "level2Depth50").
    pub name: &'static str,
    /// Fixed depth of this channel. `None` = full book / not constrained to a fixed count.
    pub depth: Option<u32>,
    /// True if this channel delivers full snapshots on every push.
    /// False = delta/incremental (initial snapshot then deltas).
    pub is_snapshot: bool,
    /// Fixed update speed in milliseconds. `None` = event-driven / real-time.
    pub update_speed_ms: Option<u32>,
    /// True if this channel requires elevated account tier / VIP / API key.
    pub requires_auth_tier: bool,
}

impl WsBookChannel {
    pub const fn snapshot(name: &'static str, depth: u32, speed_ms: u32) -> Self {
        Self {
            name,
            depth: Some(depth),
            is_snapshot: true,
            update_speed_ms: Some(speed_ms),
            requires_auth_tier: false,
        }
    }

    pub const fn delta(name: &'static str, depth: Option<u32>, speed_ms: Option<u32>) -> Self {
        Self {
            name,
            depth,
            is_snapshot: false,
            update_speed_ms: speed_ms,
            requires_auth_tier: false,
        }
    }

    pub const fn with_auth_tier(mut self) -> Self {
        self.requires_auth_tier = true;
        self
    }
}

/// Declares what L2/orderbook configurations an exchange supports on WebSocket.
///
/// ## Design notes
/// - All fields use `&'static` slices or `Copy` primitives — zero-allocation, `const`-friendly.
/// - `ws_channels` is the primary field for multi-channel exchanges (OKX, HTX, KuCoin, etc.).
///   When `ws_channels` is non-empty, `ws_depths` and `update_speeds_ms` are best-effort summaries.
/// - `rest_depth_values` overrides `rest_max_depth` when an exchange requires discrete values.
///   An empty `rest_depth_values` with `rest_max_depth = Some(N)` means "any integer up to N".
/// - `checksum` is `None` for exchanges without checksums.
/// - `has_sequence` / `has_prev_sequence` describe gap-detection capability.
///   `has_prev_sequence = true` implies `has_sequence = true`.
#[derive(Debug, Clone, Copy)]
pub struct OrderbookCapabilities {
    // ── Existing fields (preserved, semantics unchanged) ─────────────────────

    /// Valid depth levels for WS orderbook subscription.
    /// Empty = exchange doesn't accept depth parameter (it decides internally).
    pub ws_depths: &'static [u32],
    /// Recommended default depth for WS subscription. None = omit depth.
    pub ws_default_depth: Option<u32>,
    /// Maximum depth available via REST get_orderbook. None = unknown/unlimited.
    pub rest_max_depth: Option<u32>,
    /// Whether the exchange supports full orderbook snapshots on WS.
    pub supports_snapshot: bool,
    /// Whether the exchange supports incremental/delta updates on WS.
    pub supports_delta: bool,
    /// Valid update speed values in milliseconds. Empty = not configurable.
    pub update_speeds_ms: &'static [u32],
    /// Recommended default update speed. None = exchange default.
    pub default_speed_ms: Option<u32>,

    // ── New: named channel variants ──────────────────────────────────────────

    /// Named WS channel variants with distinct depth/speed/model properties.
    /// Empty slice = exchange has a single implicit channel (use ws_depths / update_speeds_ms).
    /// Non-empty = use `WsBookChannel` records for channel selection logic.
    pub ws_channels: &'static [WsBookChannel],

    // ── New: REST depth precision ─────────────────────────────────────────────

    /// Discrete valid values for REST `limit` / `depth` parameter.
    /// Empty = any integer up to `rest_max_depth` is accepted.
    /// Non-empty = ONLY these values are valid (e.g. Binance Futures: 5/10/20/50/100/500/1000).
    pub rest_depth_values: &'static [u32],

    // ── New: checksum ─────────────────────────────────────────────────────────

    /// Checksum info for the primary (or only) channel. None = no checksum.
    pub checksum: Option<ChecksumInfo>,

    // ── New: sequence / gap-detection ────────────────────────────────────────

    /// True = WS messages carry a monotonic sequence/update-ID field.
    pub has_sequence: bool,
    /// True = WS messages carry a PREVIOUS sequence field enabling in-message gap detection.
    /// (e.g. Binance Futures `pu`, OKX `prevSeqId`, Deribit `prev_change_id`).
    pub has_prev_sequence: bool,

    // ── New: price aggregation ────────────────────────────────────────────────

    /// True = exchange supports price-level aggregation/grouping on WS or REST.
    pub supports_aggregation: bool,
    /// Named aggregation tiers or parameter values (e.g. "step0".."step5", "P0".."R0", "none").
    /// Empty = aggregation not available or values are numeric/continuous.
    pub aggregation_levels: &'static [&'static str],
}

impl OrderbookCapabilities {
    /// Permissive default — accepts any depth, both snapshot and delta.
    /// Used as default for connectors that haven't declared capabilities yet.
    pub const fn permissive() -> Self {
        Self {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: &[],
            rest_depth_values: &[],
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }

    /// Pick the best matching WsBookChannel for a requested depth and update model.
    ///
    /// Returns `None` if `ws_channels` is empty (caller should fall back to legacy fields).
    /// Auth-tier channels are always skipped.
    /// When `prefer_delta` is true, delta channels are preferred over snapshots.
    pub fn best_channel(&self, requested_depth: Option<u32>, prefer_delta: bool) -> Option<&WsBookChannel> {
        if self.ws_channels.is_empty() {
            return None;
        }
        // Filter out auth-tier channels
        let public: Vec<&WsBookChannel> = self.ws_channels.iter()
            .filter(|c| !c.requires_auth_tier)
            .collect();
        if public.is_empty() {
            return None;
        }
        // Prefer delta or snapshot channels
        let preferred: Vec<&&WsBookChannel> = public.iter()
            .filter(|c| if prefer_delta { !c.is_snapshot } else { c.is_snapshot })
            .collect();
        let candidates: Vec<&WsBookChannel> = if preferred.is_empty() {
            public
        } else {
            preferred.into_iter().copied().collect()
        };
        // Pick by closest depth: smallest depth >= requested, or largest depth
        candidates.into_iter().min_by_key(|c| {
            match (c.depth, requested_depth) {
                (Some(d), Some(r)) if d >= r => d - r,
                (Some(_), Some(_)) => u32::MAX,
                (None, _) => 0,
                (Some(_), None) => 0,
            }
        })
    }

    /// Pick the closest valid depth for a requested value.
    /// - If ws_depths is empty, returns ws_default_depth (exchange doesn't accept depth param).
    /// - If requested is None, returns ws_default_depth.
    /// - Otherwise finds the smallest valid depth >= requested, or the largest valid depth.
    pub fn clamp_depth(&self, requested: Option<u32>) -> Option<u32> {
        if self.ws_depths.is_empty() {
            return self.ws_default_depth;
        }
        let target = match requested {
            Some(d) => d,
            None => return self.ws_default_depth,
        };
        // Find smallest depth >= target
        let mut best = None;
        for &d in self.ws_depths {
            if d >= target {
                match best {
                    None => best = Some(d),
                    Some(b) if d < b => best = Some(d),
                    _ => {}
                }
            }
        }
        // If nothing >= target, use the largest available
        best.or_else(|| self.ws_depths.iter().copied().max())
    }

    /// Pick the closest valid update speed for a requested value.
    /// Same logic as clamp_depth but for speed.
    pub fn clamp_speed(&self, requested: Option<u32>) -> Option<u32> {
        if self.update_speeds_ms.is_empty() {
            return self.default_speed_ms;
        }
        let target = match requested {
            Some(s) => s,
            None => return self.default_speed_ms,
        };
        let mut best = None;
        for &s in self.update_speeds_ms {
            if s >= target {
                match best {
                    None => best = Some(s),
                    Some(b) if s < b => best = Some(s),
                    _ => {}
                }
            }
        }
        best.or_else(|| self.update_speeds_ms.iter().copied().min())
    }
}

impl Default for OrderbookCapabilities {
    fn default() -> Self {
        Self::permissive()
    }
}

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
