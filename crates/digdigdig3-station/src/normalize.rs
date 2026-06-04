//! STATION-side normalization over the RAW `SymbolInfo` core emits.
//!
//! The core connector layer is raw + complete: `get_exchange_info` returns ALL
//! symbols with the venue-native `status` string verbatim (`"online"`,
//! `"Trading"`, `"live"`, `"tradable"`, `"1"`, `"active"`, ...) and never
//! filters. The raw↔normalized boundary lives HERE, opt-in: a consumer that
//! wants a canonical status or an active-only universe calls these helpers; one
//! that wants the raw truth ignores them. Nothing here mutates or loses the raw
//! `SymbolInfo` — `status`/`instrument_type`/`extra` stay intact.

use digdigdig3::core::types::SymbolInfo;

/// Canonical, exchange-agnostic trading status — a STATION normalization of the
/// many venue-native `status` strings into one vocabulary. Opt-in; the raw
/// `SymbolInfo.status` is always still available for callers who want it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolStatus {
    /// Open for trading (online / Trading / live / tradable / active / 1 / ...).
    Trading,
    /// Temporarily not trading (suspend / halt / break / cancel-only /
    /// post-only / limit-only / reduce-only / paused / CAUTION).
    Halted,
    /// Listed but not yet live (pre-launch / pre-trading / pre-open / preopen).
    PreLaunch,
    /// Permanently gone or settled (delisted / delisting / closed / settled /
    /// expired / inactive / offline / disabled / 0).
    Closed,
    /// No native status, or an unrecognized token.
    Unknown,
}

/// Map a raw `SymbolInfo`'s native `status` string to a [`SymbolStatus`].
///
/// Token-based union over all venues' vocabularies (case-insensitive). This is
/// normalization (many→few), so it's STATION's job — never core's.
pub fn canonical_status(sym: &SymbolInfo) -> SymbolStatus {
    let s = sym.status.trim().to_ascii_lowercase();
    if s.is_empty() {
        return SymbolStatus::Unknown;
    }
    // Numeric statuses (MEXC / BingX): exact match — "1" = active, "0" = inactive.
    if s == "1" {
        return SymbolStatus::Trading;
    }
    if s == "0" {
        return SymbolStatus::Closed;
    }
    // MOEX single-letter board status: "A" admitted, "S" suspended (exact —
    // a `contains` would mis-match any string with these letters).
    if s == "a" {
        return SymbolStatus::Trading;
    }
    if s == "s" {
        return SymbolStatus::Halted;
    }
    // Pre-launch first (some venues say "preopen"/"prelaunch" which would else
    // partial-match nothing).
    const PRELAUNCH: &[&str] = &["prelaunch", "pre-launch", "pretrading", "pre_trading", "preopen", "pre-open", "pre_open"];
    const HALTED: &[&str] = &[
        "suspend", "suspended", "halt", "halted", "break", "cancel-only", "cancelonly",
        "post-only", "postonly", "post_only", "limit-only", "limitonly", "limit_only",
        "reduce-only", "reduceonly", "reduce_only", "paused", "pause", "caution", "untradable",
        "not_available_for_trading", "not-available-for-trading",
    ];
    const CLOSED: &[&str] = &[
        "delist", "delisted", "delisting", "closed", "close", "settled", "settle",
        "expired", "expire", "inactive", "offline", "disabled", "disable", "unlisted", "end_of_day",
    ];
    const TRADING: &[&str] = &[
        "trading", "online", "live", "tradable", "active", "open", "normal", "enabled",
        "enabletrading", "security_trading_status_normal_trading",
    ];

    if PRELAUNCH.iter().any(|t| s.contains(t)) {
        SymbolStatus::PreLaunch
    } else if HALTED.iter().any(|t| s.contains(t)) {
        SymbolStatus::Halted
    } else if CLOSED.iter().any(|t| s.contains(t)) {
        SymbolStatus::Closed
    } else if TRADING.iter().any(|t| s == *t || s.contains(t)) {
        SymbolStatus::Trading
    } else {
        SymbolStatus::Unknown
    }
}

/// Opt-in active-only filter — keep only currently-`Trading` symbols. This is
/// the restriction core used to apply inline (and which we removed to keep core
/// raw); it now lives here as a consumer choice.
pub fn active_only(symbols: Vec<SymbolInfo>) -> Vec<SymbolInfo> {
    symbols
        .into_iter()
        .filter(|s| canonical_status(s) == SymbolStatus::Trading)
        .collect()
}

/// Borrowing variant — count/inspect without consuming.
pub fn is_active(sym: &SymbolInfo) -> bool {
    canonical_status(sym) == SymbolStatus::Trading
}

#[cfg(test)]
mod tests {
    use super::*;
    use digdigdig3::core::types::SymbolInfo;

    fn sym(status: &str) -> SymbolInfo {
        SymbolInfo { status: status.to_string(), ..Default::default() }
    }

    #[test]
    fn maps_native_statuses() {
        // Trading vocab across venues.
        for t in ["TRADING", "online", "live", "tradable", "active", "open", "Normal", "1", "Trading"] {
            assert_eq!(canonical_status(&sym(t)), SymbolStatus::Trading, "{t}");
        }
        // Halted (incl Tinkoff's verbose enum).
        for t in ["suspend", "PostOnly", "cancel-only", "CAUTION", "untradable", "SECURITY_TRADING_STATUS_NOT_AVAILABLE_FOR_TRADING"] {
            assert_eq!(canonical_status(&sym(t)), SymbolStatus::Halted, "{t}");
        }
        // Pre-launch.
        for t in ["PreLaunch", "preopen", "PreTrading"] {
            assert_eq!(canonical_status(&sym(t)), SymbolStatus::PreLaunch, "{t}");
        }
        // Closed/gone.
        for t in ["delisted", "delisting", "closed", "Settled", "expired", "offline", "Disabled", "0"] {
            assert_eq!(canonical_status(&sym(t)), SymbolStatus::Closed, "{t}");
        }
        // Unknown / absent.
        assert_eq!(canonical_status(&sym("")), SymbolStatus::Unknown);
        assert_eq!(canonical_status(&sym("wat")), SymbolStatus::Unknown);
    }

    #[test]
    fn active_only_filters() {
        let v = vec![sym("Trading"), sym("delisted"), sym("PreLaunch"), sym("online"), sym("")];
        let out = active_only(v);
        assert_eq!(out.len(), 2); // Trading + online
        assert!(out.iter().all(|s| is_active(s)));
    }
}
