//! `DataPoint` implementations for each market-data class Station persists.
//!
//! One file per class. Each implements [`crate::series::DataPoint`] with a
//! fixed-size little-endian record format. Symbol is NOT stored in the record
//! itself — it lives in the path (`<kind>/<exchange>/<account>/<symbol>/...`).

pub mod agg_trade;
pub mod auction_event;
pub mod bar;
pub mod basis;
pub mod block_trade;
pub mod composite_index;
pub mod funding_rate;
pub mod funding_settlement;
pub mod historical_volatility;
pub mod index_price;
pub mod index_price_kline;
pub mod insurance_fund;
pub mod liquidation;
pub mod mark_price;
pub mod mark_price_kline;
pub mod market_warning;
pub mod ob_delta;
pub mod ob_snapshot;
pub mod open_interest;
pub mod option_greeks;
pub mod orderbook_l3;
pub mod predicted_funding;
pub mod premium_index_kline;
pub mod risk_limit;
pub mod settlement_event;
pub mod ticker;
pub mod trade;
pub mod volatility_index;

pub use agg_trade::AggTradePoint;
pub use auction_event::AuctionEventPoint;
pub use bar::BarPoint;
pub use basis::BasisPoint;
pub use block_trade::BlockTradePoint;
pub use composite_index::CompositeIndexPoint;
pub use funding_rate::FundingRatePoint;
pub use funding_settlement::FundingSettlementPoint;
pub use historical_volatility::HistoricalVolatilityPoint;
pub use index_price::IndexPricePoint;
pub use index_price_kline::IndexPriceKlinePoint;
pub use insurance_fund::InsuranceFundPoint;
pub use liquidation::LiquidationPoint;
pub use mark_price::MarkPricePoint;
pub use mark_price_kline::MarkPriceKlinePoint;
pub use market_warning::MarketWarningPoint;
pub use ob_delta::ObDeltaPoint;
pub use ob_snapshot::ObSnapshotPoint;
pub use open_interest::OpenInterestPoint;
pub use option_greeks::OptionGreeksPoint;
pub use orderbook_l3::OrderbookL3Point;
pub use predicted_funding::PredictedFundingPoint;
pub use premium_index_kline::PremiumIndexKlinePoint;
pub use risk_limit::RiskLimitPoint;
pub use settlement_event::SettlementEventPoint;
pub use ticker::TickerPoint;
pub use trade::TradePoint;
pub use volatility_index::VolatilityIndexPoint;
