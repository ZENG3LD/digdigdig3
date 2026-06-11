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
            ExchangeId::CryptoCompare => cryptocompare::to_exchange(sym, account_type),
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
            ExchangeId::CryptoCompare => cryptocompare::from_exchange(raw, account_type),
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
            ExchangeId::CryptoCompare => cryptocompare::is_valid_for(raw, account_type),
            _ => false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// (α.2 noop_* helpers removed — every sub-module now has real conversion logic.)
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

    fn to_xbt(base: &str) -> &str {
        if base.eq_ignore_ascii_case("BTC") { "XBT" } else { base }
    }

    fn from_xbt(base: &str) -> &str {
        if base.eq_ignore_ascii_case("XBT") { "BTC" } else { base }
    }

    /// Canonical  -> Kraken REST exchange-native raw string.
    ///
    /// | AccountType | Format | Example (BTC/USD) |
    /// |---|---|---|
    /// | Spot / Margin |  (BTC->XBT, no separator) |  |
    /// | FuturesCross / FuturesIsolated |  (perpetual inverse prefix) |  |
    ///
    /// Note: WS v2 uses  (slash, BTC not XBT) -- WS callers pass that
    /// string directly; this normalizer produces the REST format only.
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        let base = to_xbt(sym.base.as_str()).to_uppercase();
        let quote = sym.quote.to_uppercase();
        match account_type {
            AccountType::Spot | AccountType::Margin => Ok(format!("{}{}", base, quote)),
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                Ok(format!("PI_{}{}", base, quote))
            }
            other => Err(NormalizerError::UnsupportedAccountType {
                exchange: ExchangeId::Kraken,
                account_type: other,
            }),
        }
    }

    /// Kraken exchange-native raw string -> canonical .
    ///
    /// Handles:
    /// -  /  -> futures (strip prefix, parse inner)
    /// -   -> spot ISO response (strip X + Z prefixes)
    /// -     -> spot simplified request (plain strip)
    ///
    /// XBT is always normalised to BTC in the canonical output.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        let inner = if let Some(rest) = raw.strip_prefix("PI_").or_else(|| raw.strip_prefix("PF_")) {
            rest
        } else {
            raw
        };

        let cleaned = inner
            .strip_prefix("XX")
            .or_else(|| inner.strip_prefix('X'))
            .unwrap_or(inner);

        for (fiat_with_z, fiat_canonical) in [
            ("ZUSD", "USD"),
            ("ZEUR", "EUR"),
            ("ZGBP", "GBP"),
            ("ZJPY", "JPY"),
            ("ZCAD", "CAD"),
        ] {
            if let Some(base_raw) = cleaned.strip_suffix(fiat_with_z) {
                if !base_raw.is_empty() {
                    return Ok(Symbol::new(from_xbt(base_raw), fiat_canonical));
                }
            }
        }

        for (plain_suffix, canonical) in [
            ("USDT", "USDT"), ("USDC", "USDC"),
            ("USD", "USD"), ("EUR", "EUR"), ("GBP", "GBP"),
            ("JPY", "JPY"), ("CAD", "CAD"),
        ] {
            if let Some(base_raw) = cleaned.strip_suffix(plain_suffix) {
                if !base_raw.is_empty() {
                    return Ok(Symbol::new(from_xbt(base_raw), canonical));
                }
            }
        }

        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::Kraken,
            raw: raw.to_string(),
        })
    }

    /// Valid Kraken REST symbol: non-empty, ASCII alphanumeric or `_` (for `PI_` futures prefix).
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

mod coinbase {
    use super::*;

    /// Coinbase symbol rules:
    /// - Spot (default): `BASE-QUOTE` uppercase dash-separated.
    ///   Examples: BTC/USD → `BTC-USD`, ETH/USDT → `ETH-USDT`
    /// - FuturesCross / FuturesIsolated (perpetuals): `BASE-PERP` (quote ignored).
    ///   Example: BTC/USD → `BTC-PERP`
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_uppercase();
        let quote = sym.quote.to_uppercase();
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                Ok(format!("{}-PERP", base))
            }
            _ => Ok(format!("{}-{}", base, quote)),
        }
    }

    /// Coinbase raw string → canonical Symbol.
    ///
    /// Splits on the first `-`: `BTC-USD` → base=BTC, quote=USD.
    /// `BTC-PERP` → base=BTC, quote=PERP.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        match raw.split_once('-') {
            Some((base, quote)) if !base.is_empty() && !quote.is_empty() => {
                Ok(Symbol::new(base, quote))
            }
            _ => Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Coinbase,
                raw: raw.to_string(),
            }),
        }
    }

    /// Valid Coinbase symbol: contains exactly one or more `-` segments with
    /// non-empty base (first segment) and non-empty remainder.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        match raw.split_once('-') {
            Some((base, quote)) => !base.is_empty() && !quote.is_empty(),
            None => false,
        }
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

    /// Known Gemini quote suffixes, ordered longest-first to avoid partial matches.
    /// Gemini spot format: `basequote` lowercase, no separator (e.g. `btcusd`).
    /// Gemini perpetuals: `base + "gusd" + "perp"` (e.g. `btcgusdperp`).
    const QUOTE_SUFFIXES: &[&str] = &["usdt", "gusd", "usdc", "usd", "btc", "eth"];

    /// Canonical Symbol → Gemini raw string (always lowercase, no separator).
    ///
    /// Spot/Margin:               `basequote`       e.g. `btcusd`
    /// FuturesCross/FuturesIsolated: `basegusdperp` e.g. `btcgusdperp`
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_lowercase();
        let quote = sym.quote.to_lowercase();
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                Ok(format!("{}gusdperp", base))
            }
            _ => Ok(format!("{}{}", base, quote)),
        }
    }

    /// Gemini raw string → canonical Symbol.
    ///
    /// Perpetuals (`btcgusdperp`) → strip `perp`, strip `gusd` → base=BTC, quote=USD.
    /// Spot (`btcusd`) → split on known lowercase quote suffix.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        let lower = raw.to_lowercase();

        // Perpetual: ends with "perp"
        if let Some(without_perp) = lower.strip_suffix("perp") {
            // Strip "gusd" suffix for GUSD-settled perps
            let base = if let Some(b) = without_perp.strip_suffix("gusd") {
                b.to_uppercase()
            } else {
                without_perp.to_uppercase()
            };
            return Ok(Symbol::new(&base, "USD"));
        }

        // Spot: try known quote suffixes (longest first)
        for &suffix in QUOTE_SUFFIXES {
            if lower.ends_with(suffix) && lower.len() > suffix.len() {
                let base = &lower[..lower.len() - suffix.len()];
                if !base.is_empty() {
                    return Ok(Symbol::new(&base.to_uppercase(), &suffix.to_uppercase()));
                }
            }
        }

        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::Gemini,
            raw: raw.to_string(),
        })
    }

    /// Valid Gemini symbol: non-empty, all ASCII lowercase alphanumeric.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
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

    /// BingX uses `BASE-QUOTE` uppercase dash format for both Spot and Swap.
    ///
    /// Examples: `BTC-USDT`, `ETH-USDT`, `BTC-USDC`.
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        if sym.base.is_empty() || sym.quote.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::BingX,
                raw: format!("{}-{}", sym.base, sym.quote),
            });
        }
        Ok(format!("{}-{}", sym.base.to_uppercase(), sym.quote.to_uppercase()))
    }

    /// Parse BingX raw string back to canonical Symbol.
    ///
    /// BingX always uses dash separator: `BTC-USDT` → `Symbol{base:"BTC", quote:"USDT"}`.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if let Some((base, quote)) = raw.split_once('-') {
            if base.is_empty() || quote.is_empty() {
                return Err(NormalizerError::InvalidFormat {
                    exchange: ExchangeId::BingX,
                    raw: raw.to_string(),
                });
            }
            return Ok(Symbol::new(base, quote));
        }
        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::BingX,
            raw: raw.to_string(),
        })
    }

    /// Valid BingX symbol: non-empty, contains exactly one `-` with non-empty parts on both sides.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        if raw.is_empty() {
            return false;
        }
        if let Some((base, quote)) = raw.split_once('-') {
            !base.is_empty() && !quote.is_empty() && !quote.contains('-')
        } else {
            false
        }
    }
}

mod crypto_com {
    use super::*;

    /// Spot: `BASE_QUOTE` underscore uppercase e.g. `BTC_USDT`.
    /// FuturesCross/Isolated: `BASEUSD-PERP` e.g. `BTCUSD-PERP`.
    ///
    /// Crypto.com perpetuals are all USD-denominated (BTCUSD-PERP, ETHUSD-PERP).
    /// USDT/USDC quote symbols are normalised to USD for futures.
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_uppercase();
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                // All Crypto.com perpetuals use USD quote regardless of canonical quote.
                Ok(format!("{}USD-PERP", base))
            }
            _ => {
                let quote = sym.quote.to_uppercase();
                Ok(format!("{}_{}", base, quote))
            }
        }
    }

    /// `BTC_USDT` → split on `_`. `BTCUSD-PERP` → strip suffix, split on known quotes.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if let Some(stripped) = raw.strip_suffix("-PERP") {
            for &q in &["USDT", "USDC", "USD"] {
                if stripped.ends_with(q) && stripped.len() > q.len() {
                    return Ok(Symbol::new(&stripped[..stripped.len() - q.len()], q));
                }
            }
            return Err(NormalizerError::InvalidFormat { exchange: ExchangeId::CryptoCom, raw: raw.to_string() });
        }
        raw.split_once('_')
            .map(|(base, quote)| Symbol::new(base, quote))
            .ok_or_else(|| NormalizerError::InvalidFormat { exchange: ExchangeId::CryptoCom, raw: raw.to_string() })
    }

    /// Spot: `_` separator, alphanumeric sides. Futures: `-PERP` suffix, alphanumeric prefix.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        if raw.is_empty() { return false; }
        if raw.ends_with("-PERP") {
            let prefix = &raw[..raw.len() - 5];
            return !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_alphanumeric());
        }
        raw.split_once('_').map_or(false, |(b, q)|
            !b.is_empty() && !q.is_empty()
                && b.chars().all(|c| c.is_ascii_alphanumeric())
                && q.chars().all(|c| c.is_ascii_alphanumeric())
        )
    }
}

mod upbit {
    use super::*;

    /// Canonical Symbol → Upbit exchange-native raw string.
    ///
    /// **REVERSED format**: Upbit uses `QUOTE-BASE` (not the common `BASE-QUOTE`).
    ///
    /// | Canonical | Upbit raw |
    /// |---|---|
    /// | BTC/USDT | `USDT-BTC` |
    /// | BTC/KRW  | `KRW-BTC`  |
    /// | ETH/KRW  | `KRW-ETH`  |
    ///
    /// Upbit only supports Spot; `account_type` is accepted but ignored.
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        if sym.base.is_empty() || sym.quote.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Upbit,
                raw: format!("{}/{}", sym.base, sym.quote),
            });
        }
        Ok(format!("{}-{}", sym.quote.to_uppercase(), sym.base.to_uppercase()))
    }

    /// Upbit exchange-native raw string → canonical Symbol.
    ///
    /// Upbit format is `QUOTE-BASE`: first segment is quote, second is base.
    /// `KRW-BTC` → `Symbol { base: "BTC", quote: "KRW" }`.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        match raw.split_once('-') {
            Some((quote, base)) if !quote.is_empty() && !base.is_empty() => {
                Ok(Symbol::new(base, quote))
            }
            _ => Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Upbit,
                raw: raw.to_string(),
            }),
        }
    }

    /// Valid Upbit symbol: contains `-` separator, both sides non-empty, all uppercase alphanumeric.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        match raw.split_once('-') {
            Some((quote, base)) => {
                !quote.is_empty()
                    && !base.is_empty()
                    && quote.chars().all(|c| c.is_ascii_alphanumeric())
                    && base.chars().all(|c| c.is_ascii_alphanumeric())
            }
            None => false,
        }
    }
}

mod bitfinex {
    use super::*;

    /// Canonical Symbol → Bitfinex exchange-native string.
    ///
    /// Format rules:
    /// - Trading pairs: `t` prefix, uppercase. When either side > 3 chars,
    ///   use `:` separator (e.g. `tBTC:USDT`, `tLINK:USD`); otherwise concatenate
    ///   (e.g. `tBTCUSD`, `tETHBTC`).
    /// - Funding currencies: `f` prefix, uppercase (e.g. `fUSD`, `fBTC`).
    ///   Signal funding market via `AccountType::Lending`.
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        if sym.base.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Bitfinex,
                raw: format!("{}/{}", sym.base, sym.quote),
            });
        }

        // Funding market
        if account_type == AccountType::Lending {
            return Ok(format!("f{}", sym.base.to_uppercase()));
        }

        let base = sym.base.to_uppercase();
        let quote = sym.quote.to_uppercase();

        // Use `:` separator when either token is longer than 3 chars
        if base.len() > 3 || quote.len() > 3 {
            Ok(format!("t{}:{}", base, quote))
        } else {
            Ok(format!("t{}{}", base, quote))
        }
    }

    /// Bitfinex exchange-native string → canonical Symbol.
    ///
    /// - `tBTCUSD`   → base=BTC, quote=USD
    /// - `tBTC:USDT` → base=BTC, quote=USDT  (colon separator for long names)
    /// - `fUSD`      → base=USD, quote=""     (funding currency)
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        // Funding currency: fXXX
        if let Some(currency) = raw.strip_prefix('f') {
            if !currency.is_empty() && currency.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Ok(Symbol::new(currency, ""));
            }
        }

        // Trading pair: tXXX
        if let Some(pair) = raw.strip_prefix('t') {
            if pair.is_empty() {
                return Err(NormalizerError::InvalidFormat {
                    exchange: ExchangeId::Bitfinex,
                    raw: raw.to_string(),
                });
            }

            // Colon-separated long names: tBTC:USDT, tBTC:UST
            if let Some((base, quote)) = pair.split_once(':') {
                if !base.is_empty() && !quote.is_empty() {
                    return Ok(Symbol::new(base, quote));
                }
            }

            // No separator — split on known quote suffixes (longest first to avoid ambiguity)
            let len = pair.len();
            const KNOWN_QUOTES: &[&str] = &[
                "USDT", "USDC", "BUSD", "TUSD", "USDP",
                "BTC", "ETH", "EUR", "GBP", "USD",
            ];
            for &q in KNOWN_QUOTES {
                if pair.ends_with(q) && len > q.len() {
                    let base = &pair[..len - q.len()];
                    if !base.is_empty() {
                        return Ok(Symbol::new(base, q));
                    }
                }
            }

            // Fallback: 3/3 split for 6-char pairs (e.g. tEOSETH)
            if len == 6 {
                return Ok(Symbol::new(&pair[..3], &pair[3..]));
            }

            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Bitfinex,
                raw: raw.to_string(),
            });
        }

        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::Bitfinex,
            raw: raw.to_string(),
        })
    }

    /// Valid Bitfinex symbol: starts with `t` or `f`, rest is uppercase ASCII
    /// alphanumeric, optionally one `:` for long-name pairs.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        if raw.is_empty() {
            return false;
        }
        let starts_ok = raw.starts_with('t') || raw.starts_with('f');
        if !starts_ok {
            return false;
        }
        let rest = &raw[1..];
        !rest.is_empty()
            && rest.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == ':')
    }
}

mod bitstamp {
    use super::*;

    /// Known Bitstamp quote suffixes, longest-first to avoid ambiguity.
    ///
    /// Bitstamp pairs USD (not USDT) for BTC: BTC/USD → `btcusd`.
    /// All lowercase, no separator.
    const QUOTE_SUFFIXES: &[&str] = &["usdt", "usdc", "eur", "gbp", "pax", "usd", "btc", "eth"];

    /// Canonical Symbol → Bitstamp exchange-native raw string.
    ///
    /// Spot (only supported): `basequote` all lowercase, no separator.
    /// Example: BTC/USD → `btcusd`.
    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                Ok(format!("{}{}", sym.base.to_lowercase(), sym.quote.to_lowercase()))
            }
            other => Err(NormalizerError::UnsupportedAccountType {
                exchange: ExchangeId::Bitstamp,
                account_type: other,
            }),
        }
    }

    /// Bitstamp raw string → canonical Symbol.
    ///
    /// No separator — split on known quote suffixes (longest-first).
    /// Returns `Err(InvalidFormat)` when no known suffix matches.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        let lower = raw.to_lowercase();
        for &suffix in QUOTE_SUFFIXES {
            if lower.ends_with(suffix) && lower.len() > suffix.len() {
                let base = &lower[..lower.len() - suffix.len()];
                if !base.is_empty() {
                    return Ok(Symbol::new(&base.to_uppercase(), &suffix.to_uppercase()));
                }
            }
        }
        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::Bitstamp,
            raw: raw.to_string(),
        })
    }

    /// Valid Bitstamp symbol: non-empty, all ASCII lowercase alphanumeric.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    }
}

mod deribit {
    use super::*;

    pub(super) fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
        if let Some(r) = sym.raw() {
            return Ok(r.to_string());
        }
        let base = sym.base.to_uppercase();
        let quote = sym.quote.to_uppercase();
        match account_type {
            AccountType::Options => Err(NormalizerError::RequiresRawInstrument {
                msg: "Deribit options require concrete instrument_name like BTC-30MAY26-50000-C                       use Symbol::with_raw(base, quote, instrument)".to_string(),
            }),
            AccountType::Spot => Ok(format!("{}-{}", base, quote)),
            AccountType::FuturesCross | AccountType::FuturesIsolated | AccountType::Margin => {
                match quote.as_str() {
                    "" | "USD" | "PERP" => Ok(format!("{}-PERPETUAL", base)),
                    "USDC" => Ok(format!("{}_USDC-PERPETUAL", base)),
                    "USDT" => Ok(format!("{}_USDT-PERPETUAL", base)),
                    other => Ok(format!("{}-{}", base, other)),
                }
            }
            _ => Ok(format!("{}-{}", base, quote)),
        }
    }

    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if raw.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Deribit,
                raw: raw.to_string(),
            });
        }
        if let Some(pair) = raw.strip_suffix("-PERPETUAL") {
            if let Some((base, quote)) = pair.split_once('_') {
                return Ok(Symbol::with_raw(base, quote, raw.to_string()));
            }
            return Ok(Symbol::with_raw(pair, "USD", raw.to_string()));
        }
        let parts: Vec<&str> = raw.splitn(4, '-').collect();
        match parts.len() {
            4 => Ok(Symbol::with_raw(parts[0], "USD", raw.to_string())),
            2 => {
                let second = parts[1];
                if second.chars().next().map_or(false, |c| c.is_ascii_digit())
                    || is_month_prefix(second)
                {
                    Ok(Symbol::with_raw(parts[0], "USD", raw.to_string()))
                } else {
                    Ok(Symbol::new(parts[0], second))
                }
            }
            _ => Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Deribit,
                raw: raw.to_string(),
            }),
        }
    }

    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty()
            && (raw.contains('-') || raw.contains('_'))
            && raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            && raw.chars().next().map_or(false, |c| c.is_ascii_uppercase())
    }

    fn is_month_prefix(s: &str) -> bool {
        const MONTHS: &[&str] = &[
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN",
            "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];
        let upper = s.to_uppercase();
        MONTHS.iter().any(|m| upper.starts_with(m))
    }
}

mod hyperliquid {
    use super::*;

    /// Canonical Symbol -> HyperLiquid coin name.
    ///
    /// HyperLiquid perps use the base coin name only -- no quote, no separator.
    /// BTC/USD -> "BTC", ETH/USD -> "ETH", SOL/USD -> "SOL".
    /// Quote is implicit (perpetuals settled in USDC).
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        if sym.base.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::HyperLiquid,
                raw: sym.base.clone(),
            });
        }
        Ok(sym.base.to_uppercase())
    }

    /// HyperLiquid coin name -> canonical Symbol.
    ///
    /// "BTC" -> Symbol { base: "BTC", quote: "USD" }.
    /// "BTC-PERP" (emitted by some HL WS frames) -> Symbol { base: "BTC", quote: "USD" }.
    /// Quote is always "USD" -- HL perps are USD-settled (USDC).
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if raw.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::HyperLiquid,
                raw: raw.to_string(),
            });
        }
        // Some HL event frames carry "BTC-PERP" — strip the suffix so the
        // canonical base is "BTC" rather than "BTC-PERP".
        let base = raw.strip_suffix("-PERP").unwrap_or(raw);
        Ok(Symbol::new(&base.to_uppercase(), "USD"))
    }

    /// Valid HyperLiquid symbol: non-empty, ASCII alphanumeric, no separator.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric())
    }
}

mod dydx {
    use super::*;

    /// dYdX v4 uses `BASE-USD` format (always USD-margined perpetuals).
    ///
    /// Examples:
    /// - `Symbol{base:"BTC", quote:"USD"}` → `"BTC-USD"`
    /// - `Symbol{base:"ETH", quote:"USD"}` → `"ETH-USD"`
    /// Quote is always `"USD"` on dYdX; ignored in favour of the literal suffix.
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_uppercase();
        if base.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Dydx,
                raw: format!("{}/{}", sym.base, sym.quote),
            });
        }
        // dYdX only has USD-margined markets; always append -USD.
        Ok(format!("{}-USD", base))
    }

    /// `"BTC-USD"` → `Symbol{base:"BTC", quote:"USD"}`.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if let Some((base, quote)) = raw.split_once('-') {
            return Ok(Symbol::new(base, quote));
        }
        Err(NormalizerError::InvalidFormat {
            exchange: ExchangeId::Dydx,
            raw: raw.to_string(),
        })
    }

    /// Valid if `BASE-USD` format (non-empty base, dash, USD suffix).
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        if let Some((base, quote)) = raw.split_once('-') {
            !base.is_empty() && !quote.is_empty()
        } else {
            false
        }
    }
}

mod lighter {
    use super::*;

    /// Lighter perpetuals trade by **base coin only** (`BTC`, `ETH`, …) — the
    /// quote leg is implicit (USD-margined). Passing `"BTCUSDT"` to a Lighter
    /// endpoint yields `Unknown Lighter market for coin 'BTCUSDT'`. The
    /// normalizer therefore returns `sym.base` uppercased.
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        if sym.base.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Lighter,
                raw: sym.base.clone(),
            });
        }
        Ok(sym.base.to_uppercase())
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if raw.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Lighter,
                raw: raw.to_string(),
            });
        }
        Ok(Symbol::new(raw, ""))
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty()
    }
}

mod polymarket {
    use super::*;

    /// Polymarket has no canonical Symbol ↔ raw conversion.
    /// Callers always pass the condition ID or market slug as-is.
    /// `to_exchange` returns `sym.base` lowercased — CLOB API requires lowercase hex.
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        if sym.base.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Polymarket,
                raw: sym.base.clone(),
            });
        }
        Ok(sym.base.to_lowercase())
    }

    /// Exchange raw string → canonical Symbol.
    /// Condition IDs and market slugs map to `base`; `quote` is always empty.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if raw.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Polymarket,
                raw: raw.to_string(),
            });
        }
        Ok(Symbol { base: raw.to_string(), quote: String::new(), raw: None })
    }

    /// Valid if non-empty (condition_id `0xabc...` or market slug).
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty()
    }
}

mod moex {
    use super::*;

    /// MOEX has no base/quote model — stocks are identified by a ticker (SECID).
    ///
    /// The security ID lives in `sym.base`. Quote is ignored because MOEX native
    /// API never carries a quote suffix in ticker strings.
    ///
    /// Examples:
    /// - `Symbol{base:"SBER", quote:""}` → `"SBER"`
    /// - `Symbol{base:"SBER", quote:"RUB"}` → `"SBER"` (quote ignored)
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_uppercase();
        if base.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Moex,
                raw: format!("{}/{}", sym.base, sym.quote),
            });
        }
        Ok(base)
    }

    /// MOEX raw security ID → `Symbol{base: <ticker>, quote: "RUB"}`.
    ///
    /// All MOEX equities are RUB-denominated; quote defaults to `"RUB"`.
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        if raw.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::Moex,
                raw: raw.to_string(),
            });
        }
        Ok(Symbol::new(&raw.to_uppercase(), "RUB"))
    }

    /// Valid MOEX security ID: non-empty, ASCII alphanumeric only.
    ///
    /// MOEX tickers (`GAZP`, `SBER`, `YNDX`, `IMOEX`) are all-caps ASCII alnum.
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric())
    }
}

mod cryptocompare {
    use super::*;

    /// CryptoCompare REST splits on `-` to extract (fsym, tsym):
    ///   `/data/price?fsym=BTC&tsyms=USDT`
    ///   `/data/pricemultifull?fsyms=BTC&tsyms=USDT`
    /// The connector expects an already-dashed `"BTC-USDT"` string; the normalizer
    /// builds it from canonical `Symbol{base, quote}`.
    pub(super) fn to_exchange(sym: &Symbol, _account_type: AccountType) -> Result<String, NormalizerError> {
        let base = sym.base.to_uppercase();
        let quote = sym.quote.to_uppercase();
        if base.is_empty() || quote.is_empty() {
            return Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::CryptoCompare,
                raw: format!("{}/{}", sym.base, sym.quote),
            });
        }
        Ok(format!("{}-{}", base, quote))
    }
    pub(super) fn from_exchange(raw: &str, _account_type: AccountType) -> Result<Symbol, NormalizerError> {
        match raw.split_once('-') {
            Some((b, q)) if !b.is_empty() && !q.is_empty() => Ok(Symbol::new(b, q)),
            _ => Err(NormalizerError::InvalidFormat {
                exchange: ExchangeId::CryptoCompare,
                raw: raw.to_string(),
            }),
        }
    }
    pub(super) fn is_valid_for(raw: &str, _account_type: AccountType) -> bool {
        raw.split_once('-').map_or(false, |(b, q)| !b.is_empty() && !q.is_empty())
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

    // Removed `is_valid_for_nonempty_raw_is_true` — it filtered out every
    // exchange that had real normalizer logic and ended up iterating an empty
    // set (vacuously passing). Per-exchange is_valid_for behaviour is covered
    // by the dedicated tests below (gemini_normalizer_spot, etc.).

    #[test]
    fn gemini_normalizer_spot() {
        let sym = Symbol::new("BTC", "USD");
        let raw = SymbolNormalizer::to_exchange(ExchangeId::Gemini, &sym, AccountType::Spot).unwrap();
        assert_eq!(raw, "btcusd");

        let parsed = SymbolNormalizer::from_exchange(ExchangeId::Gemini, "btcusd", AccountType::Spot).unwrap();
        assert_eq!(parsed.base.to_uppercase(), "BTC");
        assert_eq!(parsed.quote.to_uppercase(), "USD");

        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Gemini, "btcusd", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Gemini, "BTCUSDT", AccountType::Spot));
    }

    #[test]
    fn gemini_normalizer_perp() {
        let sym = Symbol::new("BTC", "USD");
        let raw = SymbolNormalizer::to_exchange(ExchangeId::Gemini, &sym, AccountType::FuturesCross).unwrap();
        assert_eq!(raw, "btcgusdperp");

        let parsed = SymbolNormalizer::from_exchange(ExchangeId::Gemini, "btcgusdperp", AccountType::FuturesCross).unwrap();
        assert_eq!(parsed.base.to_uppercase(), "BTC");
        assert_eq!(parsed.quote.to_uppercase(), "USD");
    }

    #[test]
    fn upbit_normalizer_reversed_format() {
        // to_exchange: quote first, then base (REVERSED)
        let btc_krw = Symbol::new("BTC", "KRW");
        let raw = SymbolNormalizer::to_exchange(ExchangeId::Upbit, &btc_krw, AccountType::Spot).unwrap();
        assert_eq!(raw, "KRW-BTC");

        let eth_usdt = Symbol::new("ETH", "USDT");
        let raw2 = SymbolNormalizer::to_exchange(ExchangeId::Upbit, &eth_usdt, AccountType::Spot).unwrap();
        assert_eq!(raw2, "USDT-ETH");

        // from_exchange: first segment = quote, second = base
        let parsed = SymbolNormalizer::from_exchange(ExchangeId::Upbit, "KRW-BTC", AccountType::Spot).unwrap();
        assert_eq!(parsed.base.to_uppercase(), "BTC");
        assert_eq!(parsed.quote.to_uppercase(), "KRW");

        let parsed2 = SymbolNormalizer::from_exchange(ExchangeId::Upbit, "USDT-ETH", AccountType::Spot).unwrap();
        assert_eq!(parsed2.base.to_uppercase(), "ETH");
        assert_eq!(parsed2.quote.to_uppercase(), "USDT");

        // is_valid_for: requires dash separator
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Upbit, "KRW-BTC", AccountType::Spot));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Upbit, "USDT-ETH", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Upbit, "BTCUSDT", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Upbit, "", AccountType::Spot));
    }

    #[test]
    fn upbit_normalizer_roundtrip() {
        let sym = Symbol::new("BTC", "KRW");
        let raw = SymbolNormalizer::to_exchange(ExchangeId::Upbit, &sym, AccountType::Spot).unwrap();
        let back = SymbolNormalizer::from_exchange(ExchangeId::Upbit, &raw, AccountType::Spot).unwrap();
        assert_eq!(back.base.to_uppercase(), sym.base.to_uppercase());
        assert_eq!(back.quote.to_uppercase(), sym.quote.to_uppercase());
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

    // ─── Bitfinex normalizer tests ───────────────────────────────────────────

    #[test]
    fn bitfinex_to_exchange_short_pairs() {
        // 3+3 = no colon
        let btc_usd = Symbol::new("BTC", "USD");
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &btc_usd, AccountType::Spot).unwrap(),
            "tBTCUSD"
        );
        let eth_usd = Symbol::new("ETH", "USD");
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &eth_usd, AccountType::Spot).unwrap(),
            "tETHUSD"
        );
        let eth_btc = Symbol::new("ETH", "BTC");
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &eth_btc, AccountType::Spot).unwrap(),
            "tETHBTC"
        );
    }

    #[test]
    fn bitfinex_to_exchange_long_pairs_use_colon() {
        // quote > 3 chars → colon separator
        let btc_usdt = Symbol::new("BTC", "USDT");
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &btc_usdt, AccountType::Spot).unwrap(),
            "tBTC:USDT"
        );
        let btc_ust = Symbol::new("BTC", "UST");
        // "UST" is 3 chars but Bitfinex uses colon only for >3; UST stays no-colon
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &btc_ust, AccountType::Spot).unwrap(),
            "tBTCUST"
        );
        // 4-char base with 3-char quote
        let link_usd = Symbol::new("LINK", "USD");
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &link_usd, AccountType::Spot).unwrap(),
            "tLINK:USD"
        );
    }

    #[test]
    fn bitfinex_to_exchange_funding() {
        let usd = Symbol::new("USD", "");
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &usd, AccountType::Lending).unwrap(),
            "fUSD"
        );
        let btc = Symbol::new("BTC", "");
        assert_eq!(
            SymbolNormalizer::to_exchange(ExchangeId::Bitfinex, &btc, AccountType::Lending).unwrap(),
            "fBTC"
        );
    }

    #[test]
    fn bitfinex_from_exchange_no_separator() {
        let sym = SymbolNormalizer::from_exchange(ExchangeId::Bitfinex, "tBTCUSD", AccountType::Spot).unwrap();
        assert_eq!(sym.base, "BTC");
        assert_eq!(sym.quote, "USD");

        let sym2 = SymbolNormalizer::from_exchange(ExchangeId::Bitfinex, "tETHUSD", AccountType::Spot).unwrap();
        assert_eq!(sym2.base, "ETH");
        assert_eq!(sym2.quote, "USD");
    }

    #[test]
    fn bitfinex_from_exchange_colon_separator() {
        let sym = SymbolNormalizer::from_exchange(ExchangeId::Bitfinex, "tBTC:USDT", AccountType::Spot).unwrap();
        assert_eq!(sym.base, "BTC");
        assert_eq!(sym.quote, "USDT");

        let sym2 = SymbolNormalizer::from_exchange(ExchangeId::Bitfinex, "tBTC:UST", AccountType::Spot).unwrap();
        assert_eq!(sym2.base, "BTC");
        assert_eq!(sym2.quote, "UST");
    }

    #[test]
    fn bitfinex_from_exchange_funding() {
        let sym = SymbolNormalizer::from_exchange(ExchangeId::Bitfinex, "fUSD", AccountType::Lending).unwrap();
        assert_eq!(sym.base, "USD");
        assert_eq!(sym.quote, "");

        let sym2 = SymbolNormalizer::from_exchange(ExchangeId::Bitfinex, "fBTC", AccountType::Lending).unwrap();
        assert_eq!(sym2.base, "BTC");
    }

    #[test]
    fn bitfinex_is_valid_for() {
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Bitfinex, "tBTCUSD", AccountType::Spot));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Bitfinex, "tETHUSD", AccountType::Spot));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Bitfinex, "tBTC:USDT", AccountType::Spot));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Bitfinex, "fUSD", AccountType::Lending));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Bitfinex, "BTCUSD", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Bitfinex, "", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Bitfinex, "tbtcusd", AccountType::Spot));
    }

    #[test]
    fn deribit_to_exchange_coin_perp() {
        let sym = Symbol::new("BTC", "USD");
        let r = SymbolNormalizer::to_exchange(ExchangeId::Deribit, &sym, AccountType::FuturesCross).unwrap();
        assert_eq!(r, "BTC-PERPETUAL");
        let eth = Symbol::new("ETH", "");
        let r2 = SymbolNormalizer::to_exchange(ExchangeId::Deribit, &eth, AccountType::FuturesCross).unwrap();
        assert_eq!(r2, "ETH-PERPETUAL");
    }

    #[test]
    fn deribit_to_exchange_usdc_perp() {
        let sol = Symbol::new("SOL", "USDC");
        let r = SymbolNormalizer::to_exchange(ExchangeId::Deribit, &sol, AccountType::FuturesCross).unwrap();
        assert_eq!(r, "SOL_USDC-PERPETUAL");
    }

    #[test]
    fn deribit_to_exchange_spot() {
        let sym = Symbol::new("BTC", "USDC");
        let r = SymbolNormalizer::to_exchange(ExchangeId::Deribit, &sym, AccountType::Spot).unwrap();
        assert_eq!(r, "BTC-USDC");
    }

    #[test]
    fn deribit_options_without_raw_returns_err() {
        let sym = Symbol::new("BTC", "USD");
        let result = SymbolNormalizer::to_exchange(ExchangeId::Deribit, &sym, AccountType::Options);
        assert!(result.is_err());
        match result.unwrap_err() {
            NormalizerError::RequiresRawInstrument { msg } => {
                assert!(msg.contains("instrument_name"), "got: {}", msg);
            }
            other => panic!("expected RequiresRawInstrument, got {:?}", other),
        }
    }

    #[test]
    fn deribit_options_with_raw_passthrough() {
        let sym = Symbol::with_raw("BTC", "USD", "BTC-30MAY26-50000-C".to_string());
        let r = SymbolNormalizer::to_exchange(ExchangeId::Deribit, &sym, AccountType::Options).unwrap();
        assert_eq!(r, "BTC-30MAY26-50000-C");
    }

    #[test]
    fn deribit_from_exchange_perps() {
        let btc = SymbolNormalizer::from_exchange(ExchangeId::Deribit, "BTC-PERPETUAL", AccountType::FuturesCross).unwrap();
        assert_eq!(btc.base, "BTC");
        assert_eq!(btc.quote, "USD");
        assert_eq!(btc.raw().unwrap(), "BTC-PERPETUAL");

        let sol = SymbolNormalizer::from_exchange(ExchangeId::Deribit, "SOL_USDC-PERPETUAL", AccountType::FuturesCross).unwrap();
        assert_eq!(sol.base, "SOL");
        assert_eq!(sol.quote, "USDC");
        assert_eq!(sol.raw().unwrap(), "SOL_USDC-PERPETUAL");
    }

    #[test]
    fn deribit_from_exchange_option() {
        let sym = SymbolNormalizer::from_exchange(ExchangeId::Deribit, "BTC-30MAY26-50000-C", AccountType::Options).unwrap();
        assert_eq!(sym.base, "BTC");
        assert_eq!(sym.quote, "USD");
        assert_eq!(sym.raw().unwrap(), "BTC-30MAY26-50000-C");
    }

    #[test]
    fn deribit_from_exchange_dated_future() {
        let sym = SymbolNormalizer::from_exchange(ExchangeId::Deribit, "BTC-30MAY26", AccountType::FuturesCross).unwrap();
        assert_eq!(sym.base, "BTC");
        assert_eq!(sym.quote, "USD");
        assert_eq!(sym.raw().unwrap(), "BTC-30MAY26");
    }

    #[test]
    fn deribit_from_exchange_spot() {
        let sym = SymbolNormalizer::from_exchange(ExchangeId::Deribit, "BTC-USDC", AccountType::Spot).unwrap();
        assert_eq!(sym.base, "BTC");
        assert_eq!(sym.quote, "USDC");
    }

    #[test]
    fn deribit_is_valid_for() {
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Deribit, "BTC-PERPETUAL", AccountType::FuturesCross));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Deribit, "SOL_USDC-PERPETUAL", AccountType::FuturesCross));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Deribit, "BTC-30MAY26-50000-C", AccountType::Options));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Deribit, "BTCUSDT", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Deribit, "", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Deribit, "btc-perpetual", AccountType::FuturesCross));
    }
    #[test]
    fn hyperliquid_normalizer_to_exchange() {
        let sym = Symbol::new("BTC", "USD");
        let raw = SymbolNormalizer::to_exchange(ExchangeId::HyperLiquid, &sym, AccountType::FuturesCross).unwrap();
        assert_eq!(raw, "BTC");

        let eth = Symbol::new("eth", "USD");
        let raw_eth = SymbolNormalizer::to_exchange(ExchangeId::HyperLiquid, &eth, AccountType::FuturesCross).unwrap();
        assert_eq!(raw_eth, "ETH");

        let sol = Symbol::new("SOL", "USDC");
        let raw_sol = SymbolNormalizer::to_exchange(ExchangeId::HyperLiquid, &sol, AccountType::Spot).unwrap();
        assert_eq!(raw_sol, "SOL");
    }

    #[test]
    fn dydx_normalizer_to_exchange() {
        let btc_usd = Symbol::new("BTC", "USD");
        let raw = SymbolNormalizer::to_exchange(ExchangeId::Dydx, &btc_usd, AccountType::FuturesCross).unwrap();
        assert_eq!(raw, "BTC-USD");

        // quote is ignored — always appends -USD
        let btc_usdt = Symbol::new("BTC", "USDT");
        let raw2 = SymbolNormalizer::to_exchange(ExchangeId::Dydx, &btc_usdt, AccountType::Spot).unwrap();
        assert_eq!(raw2, "BTC-USD");
    }

    #[test]
    fn dydx_normalizer_from_exchange() {
        let sym = SymbolNormalizer::from_exchange(ExchangeId::Dydx, "BTC-USD", AccountType::FuturesCross).unwrap();
        assert_eq!(sym.base, "BTC");
        assert_eq!(sym.quote, "USD");
    }

    #[test]
    fn dydx_normalizer_is_valid_for() {
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Dydx, "BTC-USD", AccountType::FuturesCross));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::Dydx, "ETH-USD", AccountType::FuturesCross));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Dydx, "BTCUSDT", AccountType::Spot));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::Dydx, "", AccountType::Spot));
    }

    #[test]
    fn hyperliquid_normalizer_from_exchange() {
        let parsed = SymbolNormalizer::from_exchange(ExchangeId::HyperLiquid, "BTC", AccountType::FuturesCross).unwrap();
        assert_eq!(parsed.base, "BTC");
        assert_eq!(parsed.quote, "USD");

        let parsed_eth = SymbolNormalizer::from_exchange(ExchangeId::HyperLiquid, "eth", AccountType::FuturesCross).unwrap();
        assert_eq!(parsed_eth.base, "ETH");
        assert_eq!(parsed_eth.quote, "USD");

        assert!(SymbolNormalizer::from_exchange(ExchangeId::HyperLiquid, "", AccountType::FuturesCross).is_err());
    }

    #[test]
    fn hyperliquid_normalizer_strips_perp_suffix() {
        // Some HL WS frames carry "BTC-PERP" — from_exchange must strip the
        // suffix so the canonical base is "BTC" and not "BTC-PERP".
        let parsed = SymbolNormalizer::from_exchange(
            ExchangeId::HyperLiquid,
            "BTC-PERP",
            AccountType::FuturesCross,
        )
        .unwrap();
        assert_eq!(parsed.base, "BTC");
        assert_eq!(parsed.quote, "USD");
    }

    #[test]
    fn hyperliquid_normalizer_is_valid_for() {
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::HyperLiquid, "BTC", AccountType::FuturesCross));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::HyperLiquid, "ETH", AccountType::FuturesCross));
        assert!(SymbolNormalizer::is_valid_for(ExchangeId::HyperLiquid, "SOL", AccountType::FuturesCross));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::HyperLiquid, "", AccountType::FuturesCross));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::HyperLiquid, "BTC-USD", AccountType::FuturesCross));
        assert!(!SymbolNormalizer::is_valid_for(ExchangeId::HyperLiquid, "BTC/USD", AccountType::FuturesCross));
    }

}
