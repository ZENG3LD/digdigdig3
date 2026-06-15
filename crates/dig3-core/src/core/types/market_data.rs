//! # Market Data Types
//!
//! Типы для рыночных данных.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// KLINE / OHLCV
// ═══════════════════════════════════════════════════════════════════════════════

/// Свеча (OHLCV).
///
/// RAW pump: core OHLCV always present; richer fields `Option` (serde-default).
/// Field sources (live 2026-06-14): Binance kline[9]/[10] = taker buy base/quote
/// volume (previously dropped!); Kraken/BitMEX = vwap; OKX/GateIO = confirm flag;
/// Upbit = quote_acc_volume.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Объём покупок тейкера в base (Binance kline[9]) — был выкинут парсером
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_buy_base_volume: Option<f64>,
    /// Объём покупок тейкера в quote (Binance kline[10])
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_buy_quote_volume: Option<f64>,
    /// VWAP за бар (Kraken OHLC / BitMEX tradeBin)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vwap: Option<f64>,
    /// Бар закрыт/подтверждён (OKX confirm / GateIO window-closed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm: Option<bool>,
    /// Last trade size in the bucket (BitMEX tradeBin lastSize).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_size: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TICKER
// ═══════════════════════════════════════════════════════════════════════════════

/// Тикер (24h статистика).
///
/// RAW pump: `last_price`/`timestamp` always present; every venue extra is
/// `Option`. Field sources (live-probed 2026-06-14): Bybit linear carries ~35
/// fields in one object (index/mark, prev1h, single OI, funding cap/interval,
/// basis, ask1/bid1 px+size); Deribit ticker stats{} + open_interest + funding;
/// Bitget(indexPrice/markPrice/fundingRate/holdingAmount/openUtc); MEXC-fut
/// (indexPrice/fairPrice/fundingRate/holdVol); Binance(weightedAvgPrice/
/// prevClose/openPrice/firstId/lastId/count).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Timestamp (ms)
    pub timestamp: i64,

    // ── Top-of-book sizes ──
    /// Best bid size (Bybit bid1Size / Bitget bidSz).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bid_qty: Option<f64>,
    /// Best ask size (Bybit ask1Size / Bitget askSz).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_qty: Option<f64>,

    // ── Extra price stats ──
    /// Open price 24h ago (Binance openPrice / Bitget open24h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_price: Option<f64>,
    /// Previous close price (Binance prevClosePrice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_close_price: Option<f64>,
    /// Price 24h ago (Bybit prevPrice24h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_price_24h: Option<f64>,
    /// Price 1h ago (Bybit prevPrice1h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_price_1h: Option<f64>,
    /// Weighted average price 24h (Binance weightedAvgPrice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weighted_avg_price: Option<f64>,
    /// UTC-day open price (Bitget openUtc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_utc: Option<f64>,
    /// Turnover 24h in quote (Bybit turnover24h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turnover_24h: Option<f64>,

    // ── Trade-id range / count (Binance) ──
    /// First trade id of the window (Binance firstId).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<i64>,
    /// Last trade id of the window (Binance lastId).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<i64>,
    /// Trade count in the window (Binance count).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,

    // ── Derivatives context (futures tickers) ──
    /// Mark price (Bybit/Bitget markPrice / MEXC-fut fairPrice via mark).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<f64>,
    /// Index price (Bybit/Bitget/MEXC indexPrice / Deribit index_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_price: Option<f64>,
    /// Open interest (Bybit openInterest / Bitget holdingAmount / Deribit open_interest / CryptoCom oi).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f64>,
    /// Open interest value (Bybit openInterestValue).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest_value: Option<f64>,
    /// Single-side OI (Bybit singleOpenInterest).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_open_interest: Option<f64>,
    /// Current funding rate (Bybit/Bitget/MEXC fundingRate / Deribit current_funding).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_rate: Option<f64>,
    /// Next funding time (Bybit nextFundingTime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_funding_time: Option<i64>,
    /// Funding interval in hours (Bybit fundingIntervalHour).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_interval_hour: Option<f64>,
    /// Funding cap (Bybit fundingCap).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_cap: Option<f64>,
    /// Basis (Bybit basis — absolute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<f64>,
    /// Annualized basis rate (Bybit basisRate / basisRateYear).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis_rate: Option<f64>,
    /// Predicted delivery price (Bybit predictedDeliveryPrice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicted_delivery_price: Option<f64>,
    /// Delivery time for dated contracts (Bybit deliveryTime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_time: Option<i64>,
    /// Settlement price (Deribit settlement_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_price: Option<f64>,
    /// 8-hour funding figure where the venue reports it alongside current funding
    /// (Deribit funding_8h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_8h: Option<f64>,
    /// Lower price-band limit for the instrument (Deribit min_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_price: Option<f64>,
    /// Upper price-band limit for the instrument (Deribit max_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_price: Option<f64>,
    /// Notional volume where the venue reports it distinctly from quote volume
    /// (Deribit stats.volume_notional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_notional: Option<f64>,

    // ── Last-trade / window metadata ──
    /// Last trade size (OKX lastSz / Kraken-fut lastSize / BingX lastQty).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_qty: Option<f64>,
    /// Timestamp of the last trade (Kraken-fut lastTime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_trade_time: Option<i64>,
    /// Window/day open timestamp (Binance openTime / BingX openTime / MEXC openTime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_time: Option<i64>,
    /// Book-ticker update id (Binance WS bookTicker `u`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_id: Option<i64>,

    // ── Instrument state / perpetual premium (Deribit) ──
    /// Instrument state ("open"/"closed") where the venue reports it (Deribit state).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Perpetual interest value / premium where reported (Deribit interest_value).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_value: Option<f64>,
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
    /// Previous change/sequence id for gap detection (Deribit prev_change_id / Lighter begin_nonce).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_change_id: Option<i64>,
    /// Cross-transaction sequence (Bybit WS orderbook `cts`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cts: Option<i64>,
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
            prev_change_id: None,
            cts: None,
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
            prev_change_id: None,
            cts: None,
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

/// Информация о funding rate.
///
/// RAW pump: core `rate`/`timestamp` always present; every venue-specific field
/// is `Option` (serde-default). Field sources (live-probed 2026-06-14):
/// Binance(markPrice), OKX(realizedRate/premium/interestRate/impactValue/
/// max-min/settFundingRate/settState/method/formulaType/nextFundingRate/
/// prevFundingTime), HTX(estimated_rate/fee_asset), Deribit(interest_1h/8h/
/// index_price/prev_index_price), Bitget(funding_interval/min-max),
/// MEXC(collect_cycle/idx_price/fair_price/min-max), Kraken(funding_rate_prediction/
/// relative_funding_rate), BingX(funding_interval_hours).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FundingRate {
    /// Текущий/исторический funding rate
    pub rate: f64,
    /// Время следующего funding
    pub next_funding_time: Option<i64>,
    /// Timestamp (ms)
    pub timestamp: i64,

    /// Symbol/contract this rate belongs to (history endpoints carry it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Mark price at the funding point (Binance fundingRate.markPrice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<f64>,
    /// Index price at the funding point (Deribit index_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_price: Option<f64>,
    /// Previous index price (Deribit prev_index_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_index_price: Option<f64>,
    /// Realized rate after settlement (OKX realizedRate / HTX realized_rate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realized_rate: Option<f64>,
    /// Estimated next-period rate (HTX estimated_rate / Kraken fundingRatePrediction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_rate: Option<f64>,
    /// Premium component (OKX premium).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium: Option<f64>,
    /// Interest rate component (OKX interestRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_rate: Option<f64>,
    /// 1h interest accrual (Deribit interest_1h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_1h: Option<f64>,
    /// 8h interest accrual (Deribit interest_8h).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_8h: Option<f64>,
    /// Relative funding rate (Kraken relativeFundingRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_funding_rate: Option<f64>,
    /// Average premium index over the funding window (HTX avg_premium_index).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_premium_index: Option<f64>,
    /// Impact notional used in the premium formula (OKX impactValue / GateIO funding_impact_value).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_value: Option<f64>,
    /// Funding interval in hours (BingX fundingIntervalHours / Bitget fundingRateInterval / MEXC collectCycle).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_interval_hours: Option<f64>,
    /// Funding-rate cap (OKX maxFundingRate / Bitget maxFundingRate / MEXC maxFundingRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_funding_rate: Option<f64>,
    /// Funding-rate floor (OKX minFundingRate / Bitget minFundingRate / MEXC minFundingRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_funding_rate: Option<f64>,
    /// Settled funding rate of the current period (OKX settFundingRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sett_funding_rate: Option<f64>,
    /// Settlement state (OKX settState: "settled"/"processing").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sett_state: Option<String>,
    /// Computation method (OKX method: "current_period").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Formula type (OKX formulaType: "withRate"/"noRate").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula_type: Option<String>,
    /// Next-period rate when already known (OKX nextFundingRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_funding_rate: Option<f64>,
    /// Previous funding timestamp (OKX prevFundingTime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_funding_time: Option<i64>,
    /// Funding fee asset (HTX fee_asset).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_asset: Option<String>,
    /// Funding accrued so far this period (Bitfinex deriv-status NEXT_FUNDING_ACCRUED, idx8).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accrued_funding: Option<f64>,
    /// Funding step / interval counter (Bitfinex deriv-status NEXT_FUNDING_STEP, idx9).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_step: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARK PRICE (Futures)
// ═══════════════════════════════════════════════════════════════════════════════

/// Mark price информация.
///
/// RAW pump: `mark_price`/`timestamp` always present; richer fields `Option`.
/// Field sources (live-probed 2026-06-14): Binance premiumIndex(indexPrice/
/// estimatedSettlePrice/lastFundingRate/interestRate/nextFundingTime),
/// OKX(markPx only), BitMEX(indicativeSettlePrice/indicativeFundingRate),
/// MEXC(fairPrice/idxPrice), Bitfinex deriv-status [2]=deriv_price [3]=spot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkPrice {
    /// Mark price
    pub mark_price: f64,
    /// Index price (опционально)
    pub index_price: Option<f64>,
    /// Current funding rate (опционально — только для перпетуальных контрактов)
    pub funding_rate: Option<f64>,
    /// Timestamp (ms)
    pub timestamp: i64,

    /// Symbol/contract (snapshots over all symbols carry it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Estimated settlement price (Binance estimatedSettlePrice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_settle_price: Option<f64>,
    /// Indicative settlement price (BitMEX indicativeSettlePrice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indicative_settle_price: Option<f64>,
    /// Indicative (predicted) funding rate (BitMEX indicativeFundingRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indicative_funding_rate: Option<f64>,
    /// Interest rate component (Binance interestRate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_rate: Option<f64>,
    /// Next funding time (Binance/OKX nextFundingTime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_funding_time: Option<i64>,
    /// Fair price when the venue distinguishes it from mark (MEXC fairPrice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fair_price: Option<f64>,
    /// Spot/underlying price alongside mark (Bitfinex deriv-status [3]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot_price: Option<f64>,
    /// Last derivative trade price, distinct from mark (Bitfinex deriv-status [2] DERIV_PRICE).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deriv_price: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPEN INTEREST (Futures)
// ═══════════════════════════════════════════════════════════════════════════════

/// Open Interest информация.
///
/// RAW pump: `open_interest`/`timestamp` always present; richer fields `Option`.
/// Field sources (live-probed 2026-06-14): OKX(oiCcy=base/oiUsd), Binance
/// openInterestHist(sumOpenInterest/sumOpenInterestValue/CMCCirculatingSupply),
/// Bybit(singleOpenInterest+Value), HTX(amount/value/trade_amount/trade_volume/
/// trade_turnover/business_type), GateIO(open_interest_usd).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenInterest {
    /// Open interest (в контрактах или базовом активе)
    pub open_interest: f64,
    /// Open interest в quote/USD (опционально)
    pub open_interest_value: Option<f64>,
    /// Timestamp (ms)
    pub timestamp: i64,

    /// Symbol/contract this OI belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// OI denominated in base currency (OKX oiCcy / HTX amount).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest_ccy: Option<f64>,
    /// OI denominated in USD (OKX oiUsd).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest_usd: Option<f64>,
    /// Single-side (one-way) OI (Bybit singleOpenInterest).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_open_interest: Option<f64>,
    /// Single-side OI value (Bybit singleOpenInterestValue).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_open_interest_value: Option<f64>,
    /// Summed OI over the bucket (Binance openInterestHist sumOpenInterest).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum_open_interest: Option<f64>,
    /// Summed OI value over the bucket (Binance openInterestHist sumOpenInterestValue).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum_open_interest_value: Option<f64>,
    /// CoinMarketCap circulating supply snapshot (Binance openInterestHist CMCCirculatingSupply).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmc_circulating_supply: Option<f64>,
    /// Rolling trade amount in base (HTX trade_amount).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_amount: Option<f64>,
    /// Rolling trade volume in contracts (HTX trade_volume).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_volume: Option<f64>,
    /// Rolling trade turnover in quote (HTX trade_turnover).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_turnover: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC TRADE
// ═══════════════════════════════════════════════════════════════════════════════

/// Публичная сделка (recent trades).
///
/// RAW pump principle: holds EVERY field any exchange returns on its public
/// trade feed. Core fields are always present; all venue-specific fields are
/// `Option` (serde-default) — a connector fills what its wire carries, the rest
/// stay `None`. The consumer (station) decides what to use. Lose nothing.
///
/// Field sources (live-probed 2026-06-14): Binance(quoteQty/isBestMatch/isRPITrade),
/// BitMEX(grossValue/homeNotional/foreignNotional/tickDirection/trdType),
/// Deribit(index_price/mark_price/contracts/trade_seq/iv/tick_direction),
/// Lighter(usd_amount/ask_id/bid_id/block_height/maker_fee/taker_fee/is_maker_ask),
/// Bybit(isBlockTrade/seq), MEXC(tradeType), dYdX(order_type/block_height).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublicTrade {
    /// ID сделки (trade id / exec id; some venues return none — connector fallback)
    pub id: String,
    /// Цена
    pub price: f64,
    /// Количество (base asset)
    pub quantity: f64,
    /// Сторона (taker aggressor: Buy/Sell — normalized from explicit side / maker-flag / sign / enum)
    pub side: TradeSide,
    /// Timestamp (ms)
    pub timestamp: i64,

    // ── Quote-side / notional (universally dropped before; base for cluster/volume analysis) ──
    /// Котируемый объём (Binance quoteQty, BingX-swap quoteQty)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_qty: Option<f64>,
    /// Notional в base (BitMEX homeNotional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_notional: Option<f64>,
    /// Notional в quote (BitMEX foreignNotional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreign_notional: Option<f64>,
    /// Gross value (BitMEX grossValue, satoshi)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gross_value: Option<f64>,
    /// USD-сумма сделки (Lighter usd_amount)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usd_amount: Option<f64>,

    // ── Aggressor / maker flags (kept alongside `side`) ──
    /// isBuyerMaker (Binance/MEXC/BingX) — buyer was maker → taker is seller
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_buyer_maker: Option<bool>,
    /// is_maker_ask (Lighter)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_maker_ask: Option<bool>,
    /// isBestMatch (Binance spot)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_best_match: Option<bool>,
    /// isRPITrade (Binance-fut / Bybit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_rpi_trade: Option<bool>,
    /// isBlockTrade (Bybit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_block_trade: Option<bool>,
    /// Liquidation flag (Deribit / GateIO-fut is_liquidation)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_liquidation: Option<bool>,

    // ── Microstructure ──
    /// Tick direction (BitMEX ZeroPlusTick.. / Deribit 1/3..)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_direction: Option<String>,
    /// Trade type (BitMEX trdType Regular / MEXC tradeType ASK/BID)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_type: Option<String>,
    /// Order type (dYdX type LIMIT / Kraken ordertype m/l)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    /// Sequence id (Bybit seq / GateIO sequence_id / Upbit sequential_id / KuCoin sequence)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<i64>,
    /// Open/close position direction, raw venue code (MEXC-fut `O`: 1/2/3/4).
    /// Semantics NOT normalized — preserved verbatim so station can interpret.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_close_code: Option<i64>,
    /// Self/maker trade flag, raw venue code (MEXC-fut `M`: 1/2).
    /// Semantics NOT normalized — preserved verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_flag_code: Option<i64>,

    // ── Deribit-rich (price context at trade) ──
    /// Index price at trade (Deribit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_price: Option<f64>,
    /// Mark price at trade (Deribit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<f64>,
    /// Contracts (Deribit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contracts: Option<f64>,
    /// Per-instrument trade sequence (Deribit trade_seq)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_seq: Option<i64>,
    /// Implied volatility (Deribit options)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iv: Option<f64>,

    // ── On-chain / DEX (Lighter / dYdX / HyperLiquid) ──
    /// Block height (Lighter block_height / dYdX createdAtHeight)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_height: Option<i64>,
    /// Maker order id (Lighter ask_id)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_id: Option<i64>,
    /// Taker order id (Lighter bid_id)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bid_id: Option<i64>,
    /// Maker fee (Lighter maker_fee, raw int)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maker_fee: Option<i64>,
    /// Taker fee (Lighter taker_fee, raw int)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_fee: Option<i64>,
    /// Tx hash (Lighter tx_hash / HyperLiquid hash)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    /// Counterparty addresses (HyperLiquid `users`: [taker, maker] wallet addrs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<String>>,
    /// Match/aggregation id distinct from trade id (Crypto.com `m` match_id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_id: Option<String>,
    /// Maker order id (Lighter bid_id is taker; this holds the maker side when distinct).
    /// Taker position size before the fill (Lighter taker_position_size_before).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_position_size_before: Option<f64>,
    /// Maker position size before the fill (Lighter maker_position_size_before).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maker_position_size_before: Option<f64>,
    /// On-chain transaction time, distinct from match timestamp (Lighter transaction_time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_time: Option<i64>,
    /// Ask-side account id (Lighter ask_account_id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_account_id: Option<i64>,
    /// Bid-side account id (Lighter bid_account_id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bid_account_id: Option<i64>,
    /// Source/origin tag of the trade (OKX `source`, e.g. "0").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Matching-engine pool (BitMEX pool, e.g. "Primary").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pool: Option<String>,
}

/// Сторона сделки в публичной ленте
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TradeSide {
    /// Покупатель был taker (цена пошла вверх)
    #[default]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    // ── Venue-specific (live-probed 2026-06-14) ──
    /// Liquidation order id (BitMEX orderID / GateIO).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// Order type of the forced order (Binance forceOrder `o`: "LIMIT").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    /// Order status (Binance forceOrder `X`: "FILLED").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Average fill price (Binance forceOrder `ap`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_price: Option<f64>,
    /// Accumulated executed quantity (Binance forceOrder `z`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_qty: Option<f64>,
    /// Original order quantity (Binance forceOrder `q` / GateIO order_size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_qty: Option<f64>,
    /// Order price (Binance forceOrder `p` / GateIO order_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_price: Option<f64>,
    /// Fill price reported separately (GateIO fill_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_price: Option<f64>,
    /// Quantity left unfilled (GateIO left / BitMEX leavesQty).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<f64>,
    /// Signed position size when the venue reports direction via sign (GateIO size; negative = short side).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_size: Option<f64>,
    /// Bankruptcy/base price (Bitfinex liq base_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_price: Option<f64>,
    /// Position side that was closed, raw venue token (OKX details.posSide: "long"/"short").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
}

impl Liquidation {
    /// Quote value — uses `self.value` when present, otherwise `price * quantity`.
    #[inline]
    pub fn quote_value(&self) -> f64 {
        self.value.unwrap_or(self.price * self.quantity)
    }
}
