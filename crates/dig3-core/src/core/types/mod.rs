//! # V5 Types
//!
//! Типы данных для V5 коннекторов.
//! Независимые от V4, упрощённые для использования агентами.

mod capabilities;
mod common;
mod extended_market_data;
mod market_data;
mod trading;
mod websocket;
mod responses;
mod validation;
mod symbol_input;

pub use capabilities::{
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
    ChecksumAlgorithm, ChecksumInfo, WsBookChannel, OrderbookCapabilities,
    RateLimitCapabilities, LimitModel,
    EndpointWeight, RestLimitPool, DecayingLimitConfig, WsLimits,
    ConnectorCapabilities,
};
pub use common::*;
pub use extended_market_data::{
    AggTrade,
    HistoricalVolatility, VolatilityIndex, Basis, TakerVolume, IndexPrice, CompositeIndex,
    InsuranceFund, SettlementEvent, BlockTrade,
    OrderBookSide, L3Action, OrderbookL3Event,
    RiskLimit, PredictedFunding, FundingSettlement,
    AuctionEvent, MarketWarning, OptionGreeks,
};
pub use market_data::*;
pub use trading::*;
pub use websocket::*;
pub use responses::{
    OrderResult, CancelAllResponse, BracketResponse, OcoResponse,
    AlgoOrderResponse, TransferResponse, DepositAddress, WithdrawResponse,
    FundsRecord, FeeInfo, PlaceOrderResponse,
    ClosedPnlRecord, LongShortRatio,
    FundingPayment, FundingFilter,
    LedgerEntry, LedgerEntryType, LedgerFilter,
};

// ── Wave 0: WebSocket framework types ────────────────────────────────────────
// Re-exported so callers can use `digdigdig3::core::types::StreamKind` etc.
pub use crate::core::websocket::{KlineInterval, StreamKind, StreamSpec, SupportLevel};

// ── Phase γ: Empirical validation types ──────────────────────────────────────
pub use validation::{ValidationStamp, FieldValidation};

// ── Phase θ: Unified symbol input ────────────────────────────────────────────
pub use symbol_input::{OwnedSymbolInput, SymbolInput};
