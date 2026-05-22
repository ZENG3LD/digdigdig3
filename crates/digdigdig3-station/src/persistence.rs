//! Persistence configuration for the Station builder.
//!
//! Actual disk I/O lives in [`crate::series::DiskStore`], generic over
//! [`crate::series::DataPoint`]. This module only carries on/off toggles
//! threaded through the builder.

use serde::{Deserialize, Serialize};

/// Per-kind persistence toggles. Master `enabled` gates everything.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub trades: bool,
    pub agg_trades: bool,
    pub klines: bool,
    pub tickers: bool,
    pub orderbook_snapshots: bool,
    pub mark_price: bool,
    pub funding_rate: bool,
    pub open_interest: bool,
    pub liquidations: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trades: false,
            agg_trades: false,
            klines: false,
            tickers: false,
            orderbook_snapshots: false,
            mark_price: false,
            funding_rate: false,
            open_interest: false,
            liquidations: false,
        }
    }
}

impl PersistenceConfig {
    /// Enable persistence for every supported kind.
    pub fn on() -> Self {
        Self {
            enabled: true,
            trades: true,
            agg_trades: true,
            klines: true,
            tickers: true,
            orderbook_snapshots: true,
            mark_price: true,
            funding_rate: true,
            open_interest: true,
            liquidations: true,
        }
    }

    pub fn trades(mut self, on: bool) -> Self { self.trades = on; self }
    pub fn agg_trades(mut self, on: bool) -> Self { self.agg_trades = on; self }
    pub fn klines(mut self, on: bool) -> Self { self.klines = on; self }
    pub fn tickers(mut self, on: bool) -> Self { self.tickers = on; self }
    pub fn orderbook_snapshots(mut self, on: bool) -> Self { self.orderbook_snapshots = on; self }
    pub fn mark_price(mut self, on: bool) -> Self { self.mark_price = on; self }
    pub fn funding_rate(mut self, on: bool) -> Self { self.funding_rate = on; self }
    pub fn open_interest(mut self, on: bool) -> Self { self.open_interest = on; self }
    pub fn liquidations(mut self, on: bool) -> Self { self.liquidations = on; self }

    /// Should `kind` be persisted given the current config?
    pub fn is_enabled_for(&self, kind: &crate::series::Kind) -> bool {
        if !self.enabled {
            return false;
        }
        use crate::series::Kind::*;
        match kind {
            Trade => self.trades,
            AggTrade => self.agg_trades,
            Kline(_) => self.klines,
            Ticker => self.tickers,
            Orderbook => self.orderbook_snapshots,
            MarkPrice => self.mark_price,
            FundingRate => self.funding_rate,
            OpenInterest => self.open_interest,
            Liquidation => self.liquidations,
            // Extended types — numeric variants persist if globally enabled.
            // String-bearing variants (BlockTrade, OrderbookL3, AuctionEvent,
            // MarketWarning) use header + companion `.blob` storage via
            // `DataPoint::blob_pointer_offset` and persist normally.
            IndexPrice | CompositeIndex | VolatilityIndex | HistoricalVolatility
            | Basis | InsuranceFund | SettlementEvent | PredictedFunding
            | FundingSettlement | RiskLimit | OptionGreeks
            | MarkPriceKline(_) | IndexPriceKline(_) | PremiumIndexKline(_)
            | BlockTrade | OrderbookL3 | AuctionEvent | MarketWarning => self.enabled,
        }
    }
}
