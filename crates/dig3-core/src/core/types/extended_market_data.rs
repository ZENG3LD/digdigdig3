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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggTrade {
    /// Exchange-assigned aggregate trade id.
    pub aggregate_id: i64,
    /// Trade price.
    pub price: f64,
    /// Total quantity across all merged trades.
    pub quantity: f64,
    /// First constituent trade id.
    pub first_trade_id: i64,
    /// Last constituent trade id.
    pub last_trade_id: i64,
    /// `true` = buyer is maker (sell aggressor); `false` = buyer is taker (buy aggressor).
    pub is_buy: bool,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
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
/// Exchange-published forward-looking implied volatility index.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VolatilityIndex {
    /// Index value (annualized implied volatility, e.g., 0.85 = 85%).
    pub value: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BASIS
// ═══════════════════════════════════════════════════════════════════════════════

/// Futures basis snapshot.
///
/// Basis = futures_price − spot_index_price.
/// Positive = contango (futures above spot), negative = backwardation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Basis {
    /// Futures price minus spot index price.
    pub basis: f64,
    /// Event timestamp in milliseconds.
    pub timestamp: i64,
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
