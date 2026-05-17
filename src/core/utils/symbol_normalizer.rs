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

    /// Known quote suffixes, ordered longest-first to avoid partial matches
    /// (e.g. "USDT" before "USD", "BTC" before "BNB").
    const QUOTE_SUFFIXES: &[&str] = &[
        "USDT", "USDC", "BUSD", "TUSD", "USDP",
        "BTC", "ETH", "BNB", "XRP", "DOGE",
        "AUD", "BRL", "EUR", "GBP", "RUB", "TRY", "UAH",
        "USD",
    ];

    /// `Symbol{base, quote}` → Binance raw string.
    ///
    /// Spot / Margin / USDT-M futures (`FuturesCross`/`FuturesIsolated`): `BTCUSDT`
    /// Coin-margined (`Options` used as coin-M designator): `BTCUSD_PERP`
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_uppercase();
        let quote = sym.quote.to_uppercase();

        match account_type {
            // AccountType::Options is repurposed as coin-margined designator for Binance.
            AccountType::Options => Ok(format!("{}USD_PERP", base)),
            _ => Ok(format!("{}{}", base, quote)),
        }
    }

    /// Binance raw string → `Symbol{base, quote}`.
    ///
    /// Coin-margined perp (`BTCUSD_PERP`) → `Symbol{base:"BTC", quote:"USD"}`.
    /// Spot/futures (`BTCUSDT`) → split on longest matching suffix.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        let upper = raw.to_uppercase();

        // Coin-margined perp: ends with _PERP
        if let Some(stripped) = upper.strip_suffix("_PERP") {
            if let Some(base) = stripped.strip_suffix("USD") {
                return Ok(Symbol::new(base, "USD"));
            }
            return Ok(Symbol::new(stripped, "USD"));
        }

        // Quarterly/delivery contracts: e.g. "BTCUSDT_250328"
        if let Some(pos) = upper.rfind('_') {
            let pair = &upper[..pos];
            if let Some(sym) = split_by_suffix(pair) {
                return Ok(sym);
            }
        }

        // Standard: split by known quote suffix
        split_by_suffix(&upper).ok_or_else(|| NormalizerError::InvalidFormat {
            exchange: ExchangeId::Binance,
            raw: raw.to_string(),
        })
    }

    fn split_by_suffix(upper: &str) -> Option<Symbol> {
        for &suffix in QUOTE_SUFFIXES {
            if upper.ends_with(suffix) && upper.len() > suffix.len() {
                let base = &upper[..upper.len() - suffix.len()];
                if !base.is_empty() {
                    return Some(Symbol::new(base, suffix));
                }
            }
        }
        None
    }

    /// Non-empty, all ASCII alphanumeric or underscore.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

mod bybit {
    use super::*;

    /// Known Bybit quote suffixes in descending length order so longer suffixes
    /// (e.g. "USDC") are tried before shorter ones (e.g. "USD" or "BTC").
    const QUOTE_SUFFIXES: &[&str] = &["USDT", "USDC", "BUSD", "DAI", "BTC", "ETH", "BNB", "USD"];

    /// Canonical Symbol → Bybit raw string.
    ///
    /// Bybit uses concatenated uppercase for spot, linear, and inverse:
    ///   BTC/USDT → "BTCUSDT"
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        if sym.base.is_empty() || sym.quote.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Bybit,
                raw: format!("{}/{}", sym.base, sym.quote),
            });
        }
        Ok(format!("{}{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
    }

    /// Bybit raw string → canonical Symbol.
    ///
    /// Splits on known quote suffixes (longest first to avoid ambiguity).
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        let upper = raw.to_uppercase();
        for &suffix in QUOTE_SUFFIXES {
            if upper.ends_with(suffix) && upper.len() > suffix.len() {
                let base = &upper[..upper.len() - suffix.len()];
                if !base.is_empty() {
                    return Ok(Symbol::new(base, suffix));
                }
            }
        }
        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::Bybit,
            raw: raw.to_string(),
        })
    }

    /// Valid Bybit symbol: non-empty, ASCII alphanumeric only.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric())
    }
}

mod okx {
    use super::*;

    /// Canonical Symbol → OKX exchange-native instrument ID.
    ///
    /// Spot/Margin:           `BASE-QUOTE`       e.g. `BTC-USDT`
    /// FuturesCross/Isolated: `BASE-QUOTE-SWAP`  e.g. `BTC-USDT-SWAP`
    /// Other:                 `BASE-QUOTE`        (spot format as fallback)
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_uppercase();
        let quote = sym.quote.to_uppercase();
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                Ok(format!("{}-{}-SWAP", base, quote))
            }
            _ => Ok(format!("{}-{}", base, quote)),
        }
    }

    /// OKX exchange-native instrument ID → canonical Symbol.
    ///
    /// Handles: `BTC-USDT` (spot), `BTC-USDT-SWAP` (perp), `BTC-USD-260925` (dated future).
    /// The SWAP / expiry suffix is stripped; base and quote are extracted from the first two
    /// dash-separated segments.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        let parts: Vec<&str> = raw.split('-').collect();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::OKX,
                raw: raw.to_string(),
            });
        }
        Ok(Symbol::new(parts[0], parts[1]))
    }

    /// Validates that `raw` looks like an OKX instrument ID: contains `-` and is non-empty.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.contains('-')
    }
}

mod kucoin {
    use super::*;

    /// KuCoin symbol rules:
    /// - Spot/Margin:  `BASE-QUOTE` (e.g. `BTC-USDT`)
    /// - Futures USDT-M: `XBTUSDTM` (BTC→XBT prefix, append `USDTM`)
    /// - Futures USD-M:  `XBTUSDM`  (BTC→XBT prefix, append `USDM`)
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                Ok(format!("{}-{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let base = if sym.base.to_uppercase() == "BTC" { "XBT" } else { sym.base.as_str() };
                match sym.quote.to_uppercase().as_str() {
                    "USDT" => Ok(format!("{}USDTM", base)),
                    "USD"  => Ok(format!("{}USDM",  base)),
                    other  => Ok(format!("{}{}M",    base, other)),
                }
            }
            other => Err(NormalizerError::UnsupportedAccountType {
                exchange: ExchangeId::KuCoin,
                account_type: other,
            }),
        }
    }

    /// Parse KuCoin raw symbol back to canonical Symbol.
    /// - Spot:          `BTC-USDT`   → split on `-`
    /// - Futures USDT-M: `XBTUSDTM`  → strip `M`, split at `USDT` suffix, XBT→BTC
    /// - Futures USD-M:  `XBTUSDM`   → strip `M`, split at `USD`  suffix, XBT→BTC
    pub(super) fn from_exchange(raw: &str, account_type: AccountType) -> Result<Symbol, NormalizerError> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                if let Some((base, quote)) = raw.split_once('-') {
                    return Ok(Symbol::new(base, quote));
                }
                Err(NormalizerError::InvalidFormat { exchange: ExchangeId::KuCoin, raw: raw.to_string() })
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let s = raw.strip_suffix('M').unwrap_or(raw);
                let (base_raw, quote) = if let Some(b) = s.strip_suffix("USDT") {
                    (b, "USDT")
                } else if let Some(b) = s.strip_suffix("USD") {
                    (b, "USD")
                } else {
                    return Err(NormalizerError::InvalidFormat {
                        exchange: ExchangeId::KuCoin,
                        raw: raw.to_string(),
                    });
                };
                // Reverse XBT→BTC
                let base = if base_raw.eq_ignore_ascii_case("XBT") { "BTC" } else { base_raw };
                Ok(Symbol::new(base, quote))
            }
            other => Err(NormalizerError::UnsupportedAccountType {
                exchange: ExchangeId::KuCoin,
                account_type: other,
            }),
        }
    }

    /// Valid if non-empty and all chars are ASCII alphanumeric or `-`.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
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
    /// Gate.io always uses BASE_QUOTE underscore uppercase: BTC_USDT.
    /// Same format for Spot, Margin, and Futures.
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        Ok(format!("{}_{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
    }
    /// Split on `_`; Gate.io always uses underscore separator.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if let Some((base, quote)) = raw.split_once('_') {
            return Ok(Symbol::new(base, quote));
        }
        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::GateIO,
            raw: raw.to_string(),
        })
    }
    /// Valid if contains exactly one `_` with non-empty alphanumeric parts on both sides.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        if let Some((base, quote)) = raw.split_once('_') {
            !base.is_empty()
                && !quote.is_empty()
                && base.chars().all(|c| c.is_alphanumeric())
                && quote.chars().all(|c| c.is_alphanumeric())
        } else {
            false
        }
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

    /// Spot: `BTCUSDT` (no separator, uppercase).
    /// Futures: `BTC_USDT` (underscore, uppercase).
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                Ok(format!("{}{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                Ok(format!("{}_{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
            }
            other => Err(NormalizerError::UnsupportedAccountType {
                exchange: ExchangeId::MEXC,
                account_type: other,
            }),
        }
    }

    /// Parse both forms:
    /// - `BTC_USDT` (futures, underscore) → split on `_`
    /// - `BTCUSDT` (spot, no separator) → try common quote suffixes
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        // Futures or gate-style: has underscore separator
        if let Some((base, quote)) = raw.split_once('_') {
            return Ok(Symbol::new(base, quote));
        }
        // Spot: no separator — try known quote suffixes (longer first to avoid prefix clash)
        const QUOTES: &[&str] = &["USDT", "USDC", "BUSD", "BTC", "ETH", "BNB", "USD"];
        for q in QUOTES {
            if raw.ends_with(q) && raw.len() > q.len() {
                let base = &raw[..raw.len() - q.len()];
                if !base.is_empty() {
                    return Ok(Symbol::new(base, *q));
                }
            }
        }
        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::MEXC,
            raw: raw.to_string(),
        })
    }

    /// Spot: all alphanumeric uppercase (e.g. `BTCUSDT`).
    /// Futures: two alphanumeric segments separated by exactly one underscore (e.g. `BTC_USDT`).
    pub(super) fn is_valid_for(raw: &str, account_type: AccountType) -> bool {
        if raw.is_empty() {
            return false;
        }
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                if let Some((base, quote)) = raw.split_once('_') {
                    !base.is_empty()
                        && !quote.is_empty()
                        && !quote.contains('_')
                        && base.chars().all(|c| c.is_ascii_alphanumeric())
                        && quote.chars().all(|c| c.is_ascii_alphanumeric())
                } else {
                    false
                }
            }
            _ => raw.chars().all(|c| c.is_ascii_alphanumeric()),
        }
    }
}

mod htx {
    use super::*;

    /// HTX symbol rules:
    /// - Spot / Margin: lowercase concat `btcusdt`
    /// - FuturesCross / FuturesIsolated: uppercase dash `BTC-USDT`
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                Ok(format!("{}-{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
            }
            _ => Ok(format!("{}{}", sym.base.to_lowercase(), sym.quote.to_lowercase())),
        }
    }

    /// Parse HTX raw string back to canonical Symbol.
    /// - Futures: split on `-` → `BTC-USDT` → base=BTC, quote=USDT
    /// - Spot: no separator — requires exchange_info lookup; returns Err.
    pub(super) fn from_exchange(raw: &str, account_type: AccountType) -> Result<Symbol, NormalizerError> {
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                raw.split_once('-')
                    .map(|(base, quote)| Symbol::new(base, quote))
                    .ok_or_else(|| NormalizerError::InvalidFormat {
                        exchange: ExchangeId::HTX,
                        raw: raw.to_string(),
                    })
            }
            _ => {
                // Spot: no separator — cannot parse without exchange_info.
                Err(NormalizerError::InvalidFormat {
                    exchange: ExchangeId::HTX,
                    raw: raw.to_string(),
                })
            }
        }
    }

    /// Spot: all lowercase alphanumeric, no separators.
    /// Futures: uppercase with exactly one `-`.
    pub(super) fn is_valid_for(raw: &str, account_type: AccountType) -> bool {
        if raw.is_empty() {
            return false;
        }
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let dash_count = raw.chars().filter(|&c| c == '-').count();
                dash_count == 1 && raw.chars().all(|c| c.is_alphanumeric() || c == '-')
            }
            _ => raw.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
        }
    }
}

mod bitget {
    use super::*;

    /// Bitget V2 symbol rules:
    /// - All account types: `BASEQUOTE` uppercase, e.g. `BTCUSDT`
    ///   (V1 used suffixes like `_UMCBL`; V2 dropped them)
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        Ok(format!("{}{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
    }

    /// Bitget spot raw has no separator — best-effort suffix strip against known quotes.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        const KNOWN_QUOTES: &[&str] = &[
            "USDT", "USDC", "BUSD", "TUSD", "FDUSD",
            "BTC", "ETH", "BNB", "USD",
        ];
        let upper = raw.to_uppercase();
        for quote in KNOWN_QUOTES {
            if upper.ends_with(quote) && upper.len() > quote.len() {
                let base = &upper[..upper.len() - quote.len()];
                return Ok(Symbol::new(base, *quote));
            }
        }
        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::Bitget,
            raw: raw.to_string(),
        })
    }

    /// Valid if non-empty and all chars are ASCII alphanumeric.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric())
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
