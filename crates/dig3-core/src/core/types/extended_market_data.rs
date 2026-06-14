//! # Extended Market Data Types
//!
//! Derivatives, options, and advanced market data types.
//! Source of truth for: HistoricalVolatility, VolatilityIndex, Basis, IndexPrice,
//! CompositeIndex, InsuranceFund, SettlementEvent, BlockTrade, OrderbookL3Event,
//! RiskLimit, PredictedFunding, FundingSettlement, AuctionEvent, MarketWarning,
//! OptionGreeks, AggTrade.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// AGGREGATED TRADE
// ═══════════════════════════════════════════════════════════════════════════════

/// Aggregated trade event.
///
/// Represents one or more consecutive trades at the same price, same side,
/// merged by the exchange into a single event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggTrade {
    /// Exchange-assigned aggregate trade id. (MEXC returns null → 0.)
    pub aggregate_id: i64,
    /// Trade price.
    pub price: f64,
    /// Total quantity across all merged trades.
    pub quantity: f64,
    /// First constituent trade id. `last - first + 1` = number of merged fills
    /// (the whole point of aggTrade — density of trades at this price). MEXC null → 0.
    pub first_trade_id: i64,
    /// Last constituent trade id.
    pub last_trade_id: i64,
    /// `false` = buyer is taker (buy aggressor); `true` = buyer is maker (sell aggressor).
    /// (Derived from the venue's `m`/isBuyerMaker flag.)
    pub is_buy: bool,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
    /// isBestMatch (Binance spot `M`; absent on futures).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_best_match: Option<bool>,
    /// Non-RPI quantity (Binance USDⓈ-M `nq` — qty excluding RPI orders).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_rpi_qty: Option<f64>,
    /// Quote-asset quantity where the venue provides it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_qty: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HISTORICAL VOLATILITY
// ═══════════════════════════════════════════════════════════════════════════════

/// Historical volatility snapshot.
///
/// Exchange-published realized/historical volatility metric.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HistoricalVolatility {
    /// Annualized volatility value (e.g., 0.85 = 85%).
    pub volatility: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VOLATILITY INDEX
// ═══════════════════════════════════════════════════════════════════════════════

/// Volatility index snapshot (e.g., DVOL, BVOL).
///
/// Exchange-published forward-looking implied volatility index. The Deribit
/// `get_volatility_index_data` endpoint returns OHLC candles `[ts, open, high,
/// low, close]` (live-probed 2026-06-14), not a single scalar — `value` holds
/// the close for scalar consumers, OHLC fields hold the full candle.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct VolatilityIndex {
    /// Index value (close of the candle / instantaneous value).
    pub value: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
    /// Candle open (Deribit DVOL OHLC[1]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<f64>,
    /// Candle high (Deribit DVOL OHLC[2]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high: Option<f64>,
    /// Candle low (Deribit DVOL OHLC[3]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low: Option<f64>,
    /// Candle close (Deribit DVOL OHLC[4]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BASIS
// ═══════════════════════════════════════════════════════════════════════════════

/// Futures basis snapshot.
///
/// Basis = futures_price − spot_index_price.
/// Positive = contango (futures above spot), negative = backwardation.
/// Field sources (live-probed 2026-06-14): Binance futures/data/basis
/// (indexPrice/futuresPrice/basisRate/annualizedBasisRate/contractType), HTX.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Basis {
    /// Futures price minus spot index price.
    pub basis: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
    /// Futures/contract price (Binance futuresPrice / HTX contract_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub futures_price: Option<f64>,
    /// Spot index price (Binance indexPrice / HTX index_price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_price: Option<f64>,
    /// Basis rate = basis / index (Binance basisRate / HTX basis_rate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis_rate: Option<f64>,
    /// Annualized basis rate (Binance annualizedBasisRate; empty string → None).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annualized_basis_rate: Option<f64>,
    /// Contract type (Binance contractType: "PERPETUAL"/"CURRENT_QUARTER"/...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
}

/// Taker buy/sell volume over a time bucket.
///
/// Aggressor-side flow: `buy_volume` = taker-buy (market buys lifting the ask),
/// `sell_volume` = taker-sell (market sells hitting the bid). The buy/sell ratio
/// is a common order-flow imbalance signal.
/// Field sources (live-probed 2026-06-14): Binance takerlongshortRatio
/// (buyVol/sellVol/buySellRatio), GateIO contract_stats (long/short_taker_size).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct TakerVolume {
    /// Taker buy (aggressor-buy) volume in the bucket.
    pub buy_volume: f64,
    /// Taker sell (aggressor-sell) volume in the bucket.
    pub sell_volume: f64,
    /// Bucket timestamp in milliseconds.
    pub timestamp: i64,
    /// Buy/sell ratio precomputed by the venue (Binance buySellRatio).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_sell_ratio: Option<f64>,
    /// Long-taker size from a bundled stats endpoint (GateIO long_taker_size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_taker_size: Option<f64>,
    /// Short-taker size from a bundled stats endpoint (GateIO short_taker_size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_taker_size: Option<f64>,
}

/// Bucketed liquidation aggregate (long/short forced-close sizes over a time
/// window), distinct from a per-event [`Liquidation`]. Source: GateIO
/// `contract_stats` (live-probed 2026-06-15) reports long/short liquidation
/// size, base amount, and USD value per stats bucket.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LiquidationAggregate {
    /// Bucket timestamp in milliseconds.
    pub timestamp: i64,
    /// Long liquidation size in contracts (GateIO long_liq_size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_liq_size: Option<f64>,
    /// Short liquidation size in contracts (GateIO short_liq_size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_liq_size: Option<f64>,
    /// Long liquidation amount in base (GateIO long_liq_amount).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_liq_amount: Option<f64>,
    /// Short liquidation amount in base (GateIO short_liq_amount).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_liq_amount: Option<f64>,
    /// Long liquidation value in USD (GateIO long_liq_usd).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_liq_usd: Option<f64>,
    /// Short liquidation value in USD (GateIO short_liq_usd).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_liq_usd: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// INDEX PRICE
// ═══════════════════════════════════════════════════════════════════════════════

/// Index price snapshot.
///
/// Typically the spot price underlying a perpetual or futures contract.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IndexPrice {
    /// Index price value.
    pub price: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITE INDEX
// ═══════════════════════════════════════════════════════════════════════════════

/// Composite index snapshot.
///
/// Represents a weighted basket price (e.g., Binance composite index).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeIndex {
    /// Weighted basket price.
    pub price: f64,
    /// Constituent symbols and their weights: `(symbol, weight)`.
    pub components: Vec<(String, f64)>,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// INSURANCE FUND
// ═══════════════════════════════════════════════════════════════════════════════

/// Insurance fund balance snapshot.
///
/// Exchange insurance fund used to cover losses from underwater liquidations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InsuranceFund {
    /// Current fund balance in quote currency.
    pub balance: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SETTLEMENT EVENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Contract settlement event.
///
/// Published when a futures or options contract settles at expiry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SettlementEvent {
    /// Final settlement price of the contract.
    pub settlement_price: f64,
    /// Scheduled settlement time in milliseconds.
    pub settlement_time: i64,
    /// Event publication timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BLOCK TRADE
// ═══════════════════════════════════════════════════════════════════════════════

/// Block trade event.
///
/// Large trades reported separately from the regular order book (OTC desk or
/// block trade facility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTrade {
    /// Exchange-assigned block trade identifier.
    pub block_id: String,
    /// Execution price.
    pub price: f64,
    /// Execution quantity in base asset.
    pub quantity: f64,
    /// `true` = buyer aggressor (buy block), `false` = seller aggressor (sell block).
    pub is_buy: bool,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
    /// `true` = price is expressed as implied volatility rather than a currency amount.
    pub is_iv: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDERBOOK L3
// ═══════════════════════════════════════════════════════════════════════════════

/// Side of an L3 orderbook entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderBookSide {
    /// Buy side (bids).
    Bid,
    /// Sell side (asks).
    Ask,
}

/// Action applied to an individual L3 order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum L3Action {
    /// New order placed at this price level.
    Add,
    /// Existing order quantity or price changed.
    Modify,
    /// Order fully cancelled or filled and removed.
    Delete,
}

/// Level-3 orderbook event — individual order-level mutation.
///
/// Carries add, modify, or delete for a single named order in the book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookL3Event {
    /// Which side of the book this order belongs to.
    pub side: OrderBookSide,
    /// Exchange-assigned order identifier.
    pub order_id: String,
    /// Price of the order.
    pub price: f64,
    /// Remaining quantity of the order (0 on `Delete`).
    pub quantity: f64,
    /// Action applied to this order.
    pub action: L3Action,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// RISK LIMIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Risk limit tier snapshot.
///
/// Exchange-published margin tier defining maximum leverage and position size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RiskLimit {
    /// Tier index (1-based, higher = larger position allowed).
    pub tier: u32,
    /// Maximum leverage allowed at this tier.
    pub max_leverage: f64,
    /// Maximum notional position value in quote currency at this tier.
    pub max_position_value: f64,
    /// Maintenance margin ratio (fraction, e.g., 0.005 = 0.5%).
    pub mmr: f64,
    /// Initial margin ratio (fraction, e.g., 0.01 = 1%).
    pub imr: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PREDICTED FUNDING
// ═══════════════════════════════════════════════════════════════════════════════

/// Predicted funding rate snapshot.
///
/// Published by exchanges ahead of the actual funding settlement to give
/// market participants advance notice.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PredictedFunding {
    /// Exchange's predicted funding rate for the next period (e.g., 0.0001 = 0.01%).
    pub predicted_rate: f64,
    /// Timestamp of the next funding settlement in milliseconds.
    pub next_funding_time: i64,
    /// Event publication timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING SETTLEMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Funding settlement event.
///
/// Published by the exchange after each funding period closes, confirming
/// the rate that was applied.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FundingSettlement {
    /// Actual settled funding rate (e.g., 0.0001 = 0.01%).
    pub settled_rate: f64,
    /// Timestamp at which the funding was applied in milliseconds.
    pub settlement_time: i64,
    /// Event publication timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUCTION EVENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Auction event snapshot.
///
/// Published during exchange opening, indicative, and closing auction phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionEvent {
    /// Exchange-assigned auction identifier.
    pub auction_id: String,
    /// Indicative clearing price at current auction state.
    pub indicative_price: f64,
    /// Indicative clearing quantity at current auction state.
    pub indicative_qty: f64,
    /// Auction phase: `"opening"` | `"indicative"` | `"closing"`
    pub state: String,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET WARNING
// ═══════════════════════════════════════════════════════════════════════════════

/// Market warning event.
///
/// Symbol is kept here because warnings are inherently contextual to a specific
/// instrument — callers cannot route without knowing the target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketWarning {
    /// Symbol this warning applies to (e.g., `"BTCUSDT"`).
    pub symbol: String,
    /// Warning kind identifier (exchange-defined, e.g., `"high_volatility"`, `"margin_call"`).
    pub warning_kind: String,
    /// Human-readable warning message from the exchange.
    pub message: String,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPTION GREEKS
// ═══════════════════════════════════════════════════════════════════════════════

/// Option Greeks snapshot from exchange feed.
///
/// Covers the standard first- and second-order sensitivities plus
/// implied volatility variants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OptionGreeks {
    /// Delta — sensitivity to underlying price change (−1 to +1).
    pub delta: f64,
    /// Gamma — rate of change of delta.
    pub gamma: f64,
    /// Vega — sensitivity to implied volatility change.
    pub vega: f64,
    /// Theta — time decay per day.
    pub theta: f64,
    /// Rho — sensitivity to risk-free rate change.
    pub rho: f64,
    /// Mark implied volatility.
    pub mark_iv: f64,
    /// Best bid implied volatility. `None` when not provided by exchange.
    pub bid_iv: Option<f64>,
    /// Best ask implied volatility. `None` when not provided by exchange.
    pub ask_iv: Option<f64>,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}
