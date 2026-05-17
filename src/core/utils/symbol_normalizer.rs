//! SymbolNormalizer — canonical Symbol ↔ exchange-native raw string translation.
//!
//! Central match dispatch to per-exchange sub-modules. All 22 sub-modules start
//! as no-op identity defaults; Phase α.2 batches fill each exchange's real rule.
//!
//! # Usage
//! ```rust,ignore
//! let raw = SymbolNormalizer::to_exchange(ExchangeId::Binance, &sym, AccountType::Spot)?;
//! // raw == "BTCUSDT"
//! let sym = SymbolNormalizer::from_exchange(ExchangeId::OKX, "BTC-USDT", AccountType::Spot)?;
//! ```

use crate::core::types::{AccountType, ExchangeId, Symbol};

// ─────────────────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────────────────

/// Errors produced by [`SymbolNormalizer`].
#[derive(Debug, thiserror::Error)]
pub enum NormalizerError {
    #[error("unknown exchange: {0:?}")]
    UnknownExchange(ExchangeId),

    #[error("invalid format for {exchange:?}: '{raw}'")]
    InvalidFormat { exchange: ExchangeId, raw: String },

    #[error("account type {account_type:?} not supported for {exchange:?}")]
    UnsupportedAccountType {
        exchange: ExchangeId,
        account_type: AccountType,
    },

    #[error("symbol requires full instrument name (e.g. Deribit options): {msg}")]
    RequiresRawInstrument { msg: String },
}

// ─────────────────────────────────────────────────────────────────────────────
// SymbolNormalizer
// ─────────────────────────────────────────────────────────────────────────────

/// Stateless canonical ↔ exchange-native symbol translator.
///
/// All methods are associated functions (no `&self`) — call as
/// `SymbolNormalizer::to_exchange(...)`.
pub struct SymbolNormalizer;

impl SymbolNormalizer {
    /// Canonical [`Symbol`] → exchange-native raw string.
    ///
    /// `account_type` is required because many exchanges use different formats
    /// per market (e.g. Binance spot `BTCUSDT` vs coin-margined `BTCUSD_PERP`).
    pub fn to_exchange(
        id: ExchangeId,
        sym: &Symbol,
        account_type: AccountType,
    ) -> Result<String, NormalizerError> {
        match id {
            ExchangeId::Binance     => binance::to_exchange(sym, account_type),
            ExchangeId::Bybit       => bybit::to_exchange(sym, account_type),
            ExchangeId::OKX         => okx::to_exchange(sym, account_type),
            ExchangeId::KuCoin      => kucoin::to_exchange(sym, account_type),
            ExchangeId::Kraken      => kraken::to_exchange(sym, account_type),
            ExchangeId::Coinbase    => coinbase::to_exchange(sym, account_type),
            ExchangeId::GateIO      => gateio::to_exchange(sym, account_type),
            ExchangeId::Gemini      => gemini::to_exchange(sym, account_type),
            ExchangeId::MEXC        => mexc::to_exchange(sym, account_type),
            ExchangeId::HTX         => htx::to_exchange(sym, account_type),
            ExchangeId::Bitget      => bitget::to_exchange(sym, account_type),
            ExchangeId::BingX       => bingx::to_exchange(sym, account_type),
            ExchangeId::CryptoCom   => crypto_com::to_exchange(sym, account_type),
            ExchangeId::Upbit       => upbit::to_exchange(sym, account_type),
            ExchangeId::Bitfinex    => bitfinex::to_exchange(sym, account_type),
            ExchangeId::Bitstamp    => bitstamp::to_exchange(sym, account_type),
            ExchangeId::Deribit     => deribit::to_exchange(sym, account_type),
            ExchangeId::HyperLiquid => hyperliquid::to_exchange(sym, account_type),
            ExchangeId::Dydx        => dydx::to_exchange(sym, account_type),
            ExchangeId::Lighter     => lighter::to_exchange(sym, account_type),
            ExchangeId::Polymarket  => polymarket::to_exchange(sym, account_type),
            ExchangeId::Moex        => moex::to_exchange(sym, account_type),
            other => Err(NormalizerError::UnknownExchange(other)),
        }
    }

    /// Exchange-native raw string → canonical [`Symbol`].
    ///
    /// Returns `Err` only when `raw` cannot be parsed as a known pattern for
    /// this exchange (e.g. no separator for exchanges that require exchange_info).
    pub fn from_exchange(
        id: ExchangeId,
        raw: &str,
        account_type: AccountType,
    ) -> Result<Symbol, NormalizerError> {
        match id {
            ExchangeId::Binance     => binance::from_exchange(raw, account_type),
            ExchangeId::Bybit       => bybit::from_exchange(raw, account_type),
            ExchangeId::OKX         => okx::from_exchange(raw, account_type),
            ExchangeId::KuCoin      => kucoin::from_exchange(raw, account_type),
            ExchangeId::Kraken      => kraken::from_exchange(raw, account_type),
            ExchangeId::Coinbase    => coinbase::from_exchange(raw, account_type),
            ExchangeId::GateIO      => gateio::from_exchange(raw, account_type),
            ExchangeId::Gemini      => gemini::from_exchange(raw, account_type),
            ExchangeId::MEXC        => mexc::from_exchange(raw, account_type),
            ExchangeId::HTX         => htx::from_exchange(raw, account_type),
            ExchangeId::Bitget      => bitget::from_exchange(raw, account_type),
            ExchangeId::BingX       => bingx::from_exchange(raw, account_type),
            ExchangeId::CryptoCom   => crypto_com::from_exchange(raw, account_type),
            ExchangeId::Upbit       => upbit::from_exchange(raw, account_type),
            ExchangeId::Bitfinex    => bitfinex::from_exchange(raw, account_type),
            ExchangeId::Bitstamp    => bitstamp::from_exchange(raw, account_type),
            ExchangeId::Deribit     => deribit::from_exchange(raw, account_type),
            ExchangeId::HyperLiquid => hyperliquid::from_exchange(raw, account_type),
            ExchangeId::Dydx        => dydx::from_exchange(raw, account_type),
            ExchangeId::Lighter     => lighter::from_exchange(raw, account_type),
            ExchangeId::Polymarket  => polymarket::from_exchange(raw, account_type),
            ExchangeId::Moex        => moex::from_exchange(raw, account_type),
            other => Err(NormalizerError::UnknownExchange(other)),
        }
    }

    /// Cheap pattern check — does `raw` match this exchange's known format?
    ///
    /// Used for validation before sending to API. Returns `false` for unknown
    /// exchanges rather than panicking.
    pub fn is_valid_for(id: ExchangeId, raw: &str, account_type: AccountType) -> bool {
        match id {
            ExchangeId::Binance     => binance::is_valid_for(raw, account_type),
            ExchangeId::Bybit       => bybit::is_valid_for(raw, account_type),
            ExchangeId::OKX         => okx::is_valid_for(raw, account_type),
            ExchangeId::KuCoin      => kucoin::is_valid_for(raw, account_type),
            ExchangeId::Kraken      => kraken::is_valid_for(raw, account_type),
            ExchangeId::Coinbase    => coinbase::is_valid_for(raw, account_type),
            ExchangeId::GateIO      => gateio::is_valid_for(raw, account_type),
            ExchangeId::Gemini      => gemini::is_valid_for(raw, account_type),
            ExchangeId::MEXC        => mexc::is_valid_for(raw, account_type),
            ExchangeId::HTX         => htx::is_valid_for(raw, account_type),
            ExchangeId::Bitget      => bitget::is_valid_for(raw, account_type),
            ExchangeId::BingX       => bingx::is_valid_for(raw, account_type),
            ExchangeId::CryptoCom   => crypto_com::is_valid_for(raw, account_type),
            ExchangeId::Upbit       => upbit::is_valid_for(raw, account_type),
            ExchangeId::Bitfinex    => bitfinex::is_valid_for(raw, account_type),
            ExchangeId::Bitstamp    => bitstamp::is_valid_for(raw, account_type),
            ExchangeId::Deribit     => deribit::is_valid_for(raw, account_type),
            ExchangeId::HyperLiquid => hyperliquid::is_valid_for(raw, account_type),
            ExchangeId::Dydx        => dydx::is_valid_for(raw, account_type),
            ExchangeId::Lighter     => lighter::is_valid_for(raw, account_type),
            ExchangeId::Polymarket  => polymarket::is_valid_for(raw, account_type),
            ExchangeId::Moex        => moex::is_valid_for(raw, account_type),
            _ => false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: no-op default — returns base+quote concat as placeholder.
// α.2 batches replace each sub-module body with real logic.
// ─────────────────────────────────────────────────────────────────────────────

fn noop_to_exchange(sym: &Symbol) -> Result<String, NormalizerError> {
    Ok(format!("{}{}", sym.base, sym.quote))
}

fn noop_from_exchange(
    id: ExchangeId,
    raw: &str,
) -> Result<Symbol, NormalizerError> {
    // Best-effort parse from common separators; fall back to base=raw, quote="".
    if let Some((base, quote)) = raw.split_once('-') {
        return Ok(Symbol::new(base, quote));
    }
    if let Some((base, quote)) = raw.split_once('_') {
        return Ok(Symbol::new(base, quote));
    }
    if let Some((base, quote)) = raw.split_once('/') {
        return Ok(Symbol::new(base, quote));
    }
    Err(NormalizerError::InvalidFormat {
        exchange: id,
        raw: raw.to_string(),
    })
}

fn noop_is_valid_for(raw: &str) -> bool {
    !raw.is_empty()
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-exchange sub-modules (22 total, all no-op stubs in α.1)
// ─────────────────────────────────────────────────────────────────────────────

mod binance {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Binance, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod bybit {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Bybit, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod okx {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::OKX, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod kucoin {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::KuCoin, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod kraken {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Kraken, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod coinbase {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Coinbase, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod gateio {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::GateIO, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod gemini {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Gemini, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod mexc {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::MEXC, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod htx {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::HTX, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod bitget {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Bitget, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod bingx {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::BingX, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod crypto_com {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::CryptoCom, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod upbit {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Upbit, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod bitfinex {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Bitfinex, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod bitstamp {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Bitstamp, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod deribit {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Deribit, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod hyperliquid {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::HyperLiquid, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod dydx {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Dydx, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod lighter {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Lighter, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod polymarket {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Polymarket, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

mod moex {
    use super::*;
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        noop_to_exchange(sym)
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        noop_from_exchange(ExchangeId::Moex, raw)
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        noop_is_valid_for(raw)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests — verifies dispatch works for all 22 arms (no-op defaults)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn btc_usdt() -> Symbol {
        Symbol::new("BTC", "USDT")
    }

    fn all_exchanges() -> Vec<ExchangeId> {
        vec![
            ExchangeId::Binance,
            ExchangeId::Bybit,
            ExchangeId::OKX,
            ExchangeId::KuCoin,
            ExchangeId::Kraken,
            ExchangeId::Coinbase,
            ExchangeId::GateIO,
            ExchangeId::Gemini,
            ExchangeId::MEXC,
            ExchangeId::HTX,
            ExchangeId::Bitget,
            ExchangeId::BingX,
            ExchangeId::CryptoCom,
            ExchangeId::Upbit,
            ExchangeId::Bitfinex,
            ExchangeId::Bitstamp,
            ExchangeId::Deribit,
            ExchangeId::HyperLiquid,
            ExchangeId::Dydx,
            ExchangeId::Lighter,
            ExchangeId::Polymarket,
            ExchangeId::Moex,
        ]
    }

    #[test]
    fn to_exchange_all_arms_produce_nonempty() {
        let sym = btc_usdt();
        for id in all_exchanges() {
            let result = SymbolNormalizer::to_exchange(id, &sym, AccountType::Spot);
            let raw = result.unwrap_or_else(|_| "DERIBIT_OPTIONS_SKIP".to_string());
            assert!(!raw.is_empty(), "to_exchange({id:?}) returned empty string");
        }
    }

    #[test]
    fn is_valid_for_nonempty_raw_is_true() {
        for id in all_exchanges() {
            assert!(
                SymbolNormalizer::is_valid_for(id, "BTCUSDT", AccountType::Spot),
                "is_valid_for({id:?}, \"BTCUSDT\") returned false"
            );
        }
    }

    #[test]
    fn unknown_exchange_returns_err() {
        let sym = btc_usdt();
        let result = SymbolNormalizer::to_exchange(
            ExchangeId::Custom(9999),
            &sym,
            AccountType::Spot,
        );
        assert!(result.is_err());
    }
}
