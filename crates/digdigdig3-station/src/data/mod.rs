//! `DataPoint` implementations for each market-data class Station persists.
//!
//! One file per class. Each implements [`crate::series::DataPoint`] with a
//! fixed-size little-endian record format. Symbol is NOT stored in the record
//! itself — it lives in the path (`<kind>/<exchange>/<account>/<symbol>/...`).

pub mod agg_trade;
pub mod bar;
pub mod funding_rate;
pub mod liquidation;
pub mod mark_price;
pub mod ob_snapshot;
pub mod open_interest;
pub mod ticker;
pub mod trade;

pub use agg_trade::AggTradePoint;
pub use bar::BarPoint;
pub use funding_rate::FundingRatePoint;
pub use liquidation::LiquidationPoint;
pub use mark_price::MarkPricePoint;
pub use ob_snapshot::ObSnapshotPoint;
pub use open_interest::OpenInterestPoint;
pub use ticker::TickerPoint;
pub use trade::TradePoint;
