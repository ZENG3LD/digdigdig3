//! StreamSpec — internal subscription specification for UniversalWsTransport.
//!
//! Replaces SubscriptionRequest inside the framework layer. SubscriptionRequest
//! is kept for the public WebSocketConnector trait (backward compat).

use crate::core::types::{
    AccountType, StreamType, SubscriptionRequest, Symbol, WebSocketError, WebSocketResult,
};

use super::stream_kind::{KlineInterval, StreamKind};

/// Internal subscription specification used by UniversalWsTransport.
///
/// Converted from SubscriptionRequest at subscribe() time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamSpec {
    pub kind: StreamKind,
    pub symbol: Symbol,
    pub account_type: AccountType,
    /// Depth hint for orderbook channels. None = exchange default.
    pub depth: Option<u32>,
    /// Speed hint in ms. None = exchange default.
    pub speed_ms: Option<u32>,
}

impl TryFrom<SubscriptionRequest> for StreamSpec {
    type Error = WebSocketError;

    fn try_from(req: SubscriptionRequest) -> WebSocketResult<Self> {
        let kind = StreamKind::try_from(req.stream_type)?;
        Ok(Self {
            kind,
            symbol: req.symbol,
            account_type: req.account_type,
            depth: req.depth,
            speed_ms: req.update_speed_ms,
        })
    }
}

impl From<StreamSpec> for SubscriptionRequest {
    fn from(spec: StreamSpec) -> Self {
        let stream_type = StreamType::from(spec.kind);
        SubscriptionRequest {
            symbol: spec.symbol,
            stream_type,
            account_type: spec.account_type,
            depth: spec.depth,
            update_speed_ms: spec.speed_ms,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// StreamType → StreamKind (lossless)
// ─────────────────────────────────────────────────────────────────────────────

impl TryFrom<StreamType> for StreamKind {
    type Error = WebSocketError;

    fn try_from(st: StreamType) -> WebSocketResult<Self> {
        Ok(match st {
            StreamType::Ticker => StreamKind::Ticker,
            StreamType::Trade => StreamKind::Trade,
            StreamType::Orderbook => StreamKind::Orderbook,
            StreamType::OrderbookDelta => StreamKind::OrderbookDelta,
            StreamType::OrderbookL3 => StreamKind::OrderbookL3,
            StreamType::Kline { interval } => StreamKind::Kline {
                interval: KlineInterval::new(interval),
            },
            StreamType::MarkPrice => StreamKind::MarkPrice,
            StreamType::FundingRate => StreamKind::FundingRate,
            StreamType::Liquidation => StreamKind::Liquidation,
            StreamType::OpenInterest => StreamKind::OpenInterest,
            StreamType::LongShortRatio => StreamKind::LongShortRatio,
            StreamType::AggTrade => StreamKind::AggTrade,
            StreamType::CompositeIndex => StreamKind::CompositeIndex,
            StreamType::MarkPriceKline { interval } => StreamKind::MarkPriceKline {
                interval: KlineInterval::new(interval),
            },
            StreamType::IndexPriceKline { interval } => StreamKind::IndexPriceKline {
                interval: KlineInterval::new(interval),
            },
            StreamType::PremiumIndexKline { interval } => StreamKind::PremiumIndexKline {
                interval: KlineInterval::new(interval),
            },
            StreamType::IndexPrice => StreamKind::IndexPrice,
            StreamType::HistoricalVolatility => StreamKind::HistoricalVolatility,
            StreamType::InsuranceFund => StreamKind::InsuranceFund,
            StreamType::Basis => StreamKind::Basis,
            StreamType::OptionGreeks => StreamKind::OptionGreeks,
            StreamType::VolatilityIndex => StreamKind::VolatilityIndex,
            StreamType::BlockTrade => StreamKind::BlockTrade,
            StreamType::AuctionEvent => StreamKind::AuctionEvent,
            StreamType::MarketWarning => StreamKind::MarketWarning,
            StreamType::SettlementEvent => StreamKind::SettlementEvent,
            StreamType::RiskLimit => StreamKind::RiskLimit,
            StreamType::PredictedFunding => StreamKind::PredictedFunding,
            StreamType::FundingSettlement => StreamKind::FundingSettlement,
            StreamType::OrderUpdate => StreamKind::OrderUpdate,
            StreamType::BalanceUpdate => StreamKind::BalanceUpdate,
            StreamType::PositionUpdate => StreamKind::PositionUpdate,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// StreamKind → StreamType (lossless round-trip)
// ─────────────────────────────────────────────────────────────────────────────

impl From<StreamKind> for StreamType {
    fn from(kind: StreamKind) -> Self {
        match kind {
            StreamKind::Ticker => StreamType::Ticker,
            StreamKind::Trade => StreamType::Trade,
            StreamKind::Orderbook => StreamType::Orderbook,
            StreamKind::OrderbookDelta => StreamType::OrderbookDelta,
            StreamKind::OrderbookL3 => StreamType::OrderbookL3,
            StreamKind::Kline { interval } => StreamType::Kline {
                interval: interval.0,
            },
            StreamKind::MarkPrice => StreamType::MarkPrice,
            StreamKind::FundingRate => StreamType::FundingRate,
            StreamKind::Liquidation => StreamType::Liquidation,
            StreamKind::OpenInterest => StreamType::OpenInterest,
            StreamKind::LongShortRatio => StreamType::LongShortRatio,
            StreamKind::AggTrade => StreamType::AggTrade,
            StreamKind::CompositeIndex => StreamType::CompositeIndex,
            StreamKind::MarkPriceKline { interval } => StreamType::MarkPriceKline {
                interval: interval.0,
            },
            StreamKind::IndexPriceKline { interval } => StreamType::IndexPriceKline {
                interval: interval.0,
            },
            StreamKind::PremiumIndexKline { interval } => StreamType::PremiumIndexKline {
                interval: interval.0,
            },
            StreamKind::IndexPrice => StreamType::IndexPrice,
            StreamKind::HistoricalVolatility => StreamType::HistoricalVolatility,
            StreamKind::InsuranceFund => StreamType::InsuranceFund,
            StreamKind::Basis => StreamType::Basis,
            StreamKind::OptionGreeks => StreamType::OptionGreeks,
            StreamKind::VolatilityIndex => StreamType::VolatilityIndex,
            StreamKind::BlockTrade => StreamType::BlockTrade,
            StreamKind::AuctionEvent => StreamType::AuctionEvent,
            StreamKind::MarketWarning => StreamType::MarketWarning,
            StreamKind::SettlementEvent => StreamType::SettlementEvent,
            StreamKind::RiskLimit => StreamType::RiskLimit,
            StreamKind::PredictedFunding => StreamType::PredictedFunding,
            StreamKind::FundingSettlement => StreamType::FundingSettlement,
            StreamKind::OrderUpdate => StreamType::OrderUpdate,
            StreamKind::BalanceUpdate => StreamType::BalanceUpdate,
            StreamKind::PositionUpdate => StreamType::PositionUpdate,
        }
    }
}
