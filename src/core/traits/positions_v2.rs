//! # PositionsV2 — Thin positions trait (V2 architecture)
//!
//! Futures / perpetual position management.
//! All position mutations go through `modify_position` via `PositionModification` enum.

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, FundingRate, Position,
    PositionModification, PositionQuery,
};

use super::ExchangeIdentity;

/// Positions — 22/24 exchanges.
///
/// Spot-only exchanges (Bitstamp, Gemini) do not implement this trait.
/// For all others, positions represent open perpetual/futures exposures.
///
/// All position mutations are handled through `modify_position` using the
/// `PositionModification` fat enum. Connectors match only the variants
/// they support natively.
///
/// Authentication is **required** for all methods in this trait.
#[async_trait]
pub trait PositionsV2: ExchangeIdentity {
    /// Get open positions, optionally filtered to a single symbol.
    ///
    /// `query.symbol = None` returns all open positions across all symbols.
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>>;

    /// Get the current funding rate for a perpetual contract symbol.
    ///
    /// Returns the current rate, the next funding timestamp, and the
    /// predicted rate if the exchange provides it.
    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate>;

    /// Modify a position — leverage, margin mode, add/remove margin,
    /// TP/SL, or close.
    ///
    /// The connector matches the `PositionModification` variant it supports.
    /// Unsupported variants MUST return `ExchangeError::UnsupportedOperation`.
    /// Connectors MUST NOT simulate missing features by composing other methods.
    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()>;
}
