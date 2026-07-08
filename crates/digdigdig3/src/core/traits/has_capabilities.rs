//! Per-connector capability declaration.
//!
//! Every connector MUST explicitly implement `capabilities()` — there is no
//! default impl. This prevents silent all-false declarations and forces
//! conscious surface mapping.

use crate::core::types::{
    ConnectorCapabilities, KlineIntervalCapabilities, TradeHistoryCapabilities, ValidationStamp,
};

/// Declare the full capability surface of a connector.
///
/// Examined by the pool to filter connectors that can satisfy a given
/// operation before calling. Every `CoreConnector` implementor must provide
/// a fully explicit declaration — the trait intentionally has no default impl.
pub trait HasCapabilities: Send + Sync {
    /// Return a declarative map of what this connector supports.
    fn capabilities(&self) -> ConnectorCapabilities;

    /// Empirical validation results from the last `e2e_smoke` harness run.
    ///
    /// `None` = connector was never smoke-tested with live exchange data.
    /// Connectors override this with a 1-liner delegating to `validation_snapshot::validation_for`.
    fn validation_status(&self) -> Option<&'static ValidationStamp> {
        None
    }

    /// Declare how deep this connector's public trade-history endpoints
    /// actually reach, per account class, plus kline backward-pagination
    /// honesty. See `TradeHistoryCapabilities` for the tier semantics.
    ///
    /// Default is `TradeHistoryCapabilities::conservative_default()` —
    /// `RecentOnly { max_trades: 1000 }` for both spot and futures, kline
    /// backpage `false`. Connectors migrated to this model override with an
    /// explicit declaration; unmigrated connectors stay honest-pessimistic
    /// instead of silently claiming depth they cannot deliver.
    fn trade_history_capabilities(&self) -> TradeHistoryCapabilities {
        TradeHistoryCapabilities::conservative_default()
    }

    /// Declare which kline intervals this connector's REST history + WS
    /// kline channel natively serve, per account class. See
    /// `KlineIntervalCapabilities` for the exact semantics and why the
    /// consuming TF dropdown needs this (Native / Aggregated / LiveOnly
    /// classification).
    ///
    /// Default is `KlineIntervalCapabilities::conservative_default()` —
    /// the common `1m 5m 15m 30m 1h 4h 1d` set for both spot and futures.
    /// Connectors audited for this model override with an explicit,
    /// probe-verified declaration; unmigrated connectors under-claim
    /// rather than silently claiming intervals they cannot deliver.
    fn kline_interval_capabilities(&self) -> KlineIntervalCapabilities {
        KlineIntervalCapabilities::conservative_default()
    }
}
