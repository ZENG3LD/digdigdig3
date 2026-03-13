//! # Positions — Futures / perpetual position management
//!
//! All position mutations go through `modify_position` via `PositionModification` enum.

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, FundingRate, Position,
    PositionModification, PositionQuery,
    OpenInterest, MarkPrice, ClosedPnlRecord, LongShortRatio,
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
pub trait Positions: ExchangeIdentity {
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

    /// Get open interest for a perpetual/futures symbol.
    ///
    /// Returns the total notional open interest and optionally the USD value.
    ///
    /// Default implementation returns `UnsupportedOperation`.
    ///
    /// ~18/24: Binance Futures, Bybit, OKX, KuCoin, GateIO, Bitget, BingX,
    /// Phemex, MEXC, HTX, CryptoCom, Deribit, HyperLiquid, Lighter,
    /// Paradex, dYdX, GMX, Coinglass (data provider).
    async fn get_open_interest(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<OpenInterest> {
        let _ = (symbol, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_open_interest not implemented".into(),
        ))
    }

    /// Get the historical funding rate for a perpetual contract symbol.
    ///
    /// Returns funding rate records between the given time bounds, up to `limit`.
    ///
    /// Default implementation returns `UnsupportedOperation`.
    ///
    /// ~16/24: Binance Futures, Bybit, OKX, KuCoin, GateIO, Bitget, BingX,
    /// Phemex, MEXC, HTX, CryptoCom, Deribit, HyperLiquid, Lighter, Paradex, dYdX.
    async fn get_funding_rate_history(
        &self,
        symbol: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<FundingRate>> {
        let _ = (symbol, start_time, end_time, limit);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_funding_rate_history not implemented".into(),
        ))
    }

    /// Get the current mark price (and optionally index price + funding rate)
    /// for a perpetual/futures symbol.
    ///
    /// Default implementation returns `UnsupportedOperation`.
    ///
    /// ~18/24: all perpetuals-capable exchanges.
    async fn get_mark_price(
        &self,
        symbol: &str,
    ) -> ExchangeResult<MarkPrice> {
        let _ = symbol;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_mark_price not implemented".into(),
        ))
    }

    /// Get the closed position P&L history.
    ///
    /// Returns realized P&L records for positions that have been closed,
    /// optionally filtered by symbol and time range.
    ///
    /// Default implementation returns `UnsupportedOperation`.
    ///
    /// ~12/24: Bybit, OKX, Binance Futures, KuCoin, GateIO, Bitget, BingX,
    /// Phemex, Deribit, HyperLiquid, Paradex, dYdX.
    async fn get_closed_pnl(
        &self,
        symbol: Option<&str>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<ClosedPnlRecord>> {
        let _ = (symbol, start_time, end_time, limit);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_closed_pnl not implemented".into(),
        ))
    }

    /// Get the long/short account ratio for a symbol.
    ///
    /// Returns the proportion of accounts (or notional) that are long vs short
    /// at the given moment. A market sentiment indicator.
    ///
    /// Default implementation returns `UnsupportedOperation`.
    ///
    /// ~8/24: Binance Futures, Bybit, OKX, KuCoin Futures, Bitget, BingX,
    /// GateIO, HTX.
    async fn get_long_short_ratio(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<LongShortRatio> {
        let _ = (symbol, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_long_short_ratio not implemented".into(),
        ))
    }
}
