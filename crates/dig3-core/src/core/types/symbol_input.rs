//! SymbolInput — unified symbol parameter for connector trait methods.
//!
//! Callers can pass either a raw exchange-native string or a canonical [`Symbol`]
//! that is normalized inside the call via [`SymbolNormalizer`].

use std::borrow::Cow;
use std::fmt;

use crate::core::types::{AccountType, ExchangeId, Symbol};
use crate::core::utils::symbol_normalizer::{NormalizerError, SymbolNormalizer};

// ─────────────────────────────────────────────────────────────────────────────
// Borrowed variant
// ─────────────────────────────────────────────────────────────────────────────

/// Input to any per-symbol method on a connector.
///
/// Either an exchange-native raw string or a canonical [`Symbol`] that will be
/// normalized to the exchange's native format at call time.
///
/// # Examples
///
/// ```rust,ignore
/// // Raw — used as-is, zero allocation:
/// connector.get_ticker(SymbolInput::Raw("BTCUSDT"), AccountType::Spot).await?;
///
/// // Canonical — normalized via SymbolNormalizer at call site:
/// let sym = Symbol::new("BTC", "USDT");
/// connector.get_ticker(SymbolInput::Canonical(&sym), AccountType::Spot).await?;
///
/// // Via From impls:
/// connector.get_ticker("BTCUSDT".into(), AccountType::Spot).await?;
/// connector.get_ticker((&sym).into(), AccountType::Spot).await?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolInput<'a> {
    /// Exchange-native string (e.g. `"BTCUSDT"` for Binance, `"tBTCUSD"` for Bitfinex).
    /// Used as-is — no normalization performed.
    Raw(&'a str),

    /// Canonical [`Symbol`] `{ base, quote }` — will be normalized via
    /// [`SymbolNormalizer::to_exchange`] to the exchange's native format.
    Canonical(&'a Symbol),
}

impl<'a> SymbolInput<'a> {
    /// Resolve to an exchange-native string.
    ///
    /// - `Raw(s)` → borrows `s` directly (zero allocation).
    /// - `Canonical(sym)` → allocates via [`SymbolNormalizer::to_exchange`].
    pub fn resolve(
        self,
        exchange: ExchangeId,
        account_type: AccountType,
    ) -> Result<Cow<'a, str>, NormalizerError> {
        match self {
            SymbolInput::Raw(s) => Ok(Cow::Borrowed(s)),
            SymbolInput::Canonical(sym) => Ok(Cow::Owned(
                SymbolNormalizer::to_exchange(exchange, sym, account_type)?,
            )),
        }
    }
}

impl<'a> From<&'a str> for SymbolInput<'a> {
    fn from(s: &'a str) -> Self {
        SymbolInput::Raw(s)
    }
}

impl<'a> From<&'a String> for SymbolInput<'a> {
    fn from(s: &'a String) -> Self {
        SymbolInput::Raw(s.as_str())
    }
}

impl<'a> From<&'a Symbol> for SymbolInput<'a> {
    fn from(s: &'a Symbol) -> Self {
        SymbolInput::Canonical(s)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Owned variant (for long-lived storage, e.g. StreamSpec)
// ─────────────────────────────────────────────────────────────────────────────

/// Owned counterpart to [`SymbolInput`] — used where the input must outlive a
/// borrow (e.g. inside [`StreamSpec`] which is stored in the subscription
/// registry).
///
/// [`StreamSpec`]: crate::core::websocket::StreamSpec
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OwnedSymbolInput {
    /// Exchange-native raw string, stored as owned [`String`].
    Raw(String),
    /// Canonical symbol, stored as owned [`Symbol`].
    Canonical(Symbol),
}

impl OwnedSymbolInput {
    /// Borrow as a [`SymbolInput`], suitable for passing to connector methods.
    pub fn as_borrowed(&self) -> SymbolInput<'_> {
        match self {
            OwnedSymbolInput::Raw(s) => SymbolInput::Raw(s.as_str()),
            OwnedSymbolInput::Canonical(s) => SymbolInput::Canonical(s),
        }
    }

    /// Return the raw string slice for `Raw` variant.
    ///
    /// - `Raw(s)` → borrows `s`.
    /// - `Canonical(sym)` → borrows the base field as a best-effort fallback.
    ///   WsProtocol impls should migrate to [`StreamSpec::resolve_symbol`] in θ.2
    ///   to get the proper exchange-native format.
    pub fn as_str(&self) -> &str {
        match self {
            OwnedSymbolInput::Raw(s) => s.as_str(),
            OwnedSymbolInput::Canonical(sym) => sym.base.as_str(),
        }
    }

    /// `true` if the underlying raw string is empty (or base is empty for Canonical).
    ///
    /// Bridge for existing WsProtocol impls pending θ.2 migration.
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedSymbolInput::Raw(s) => s.is_empty(),
            OwnedSymbolInput::Canonical(sym) => sym.base.is_empty(),
        }
    }

    /// Lowercase of the raw string (or base for Canonical).
    ///
    /// Bridge for existing WsProtocol impls pending θ.2 migration.
    pub fn to_lowercase(&self) -> String {
        self.as_str().to_lowercase()
    }

    /// Uppercase of the raw string (or base for Canonical).
    ///
    /// Bridge for existing WsProtocol impls pending θ.2 migration.
    pub fn to_uppercase(&self) -> String {
        self.as_str().to_uppercase()
    }

    /// Resolve to an exchange-native owned [`String`].
    pub fn resolve(
        &self,
        exchange: ExchangeId,
        account_type: AccountType,
    ) -> Result<String, NormalizerError> {
        match self {
            OwnedSymbolInput::Raw(s) => Ok(s.clone()),
            OwnedSymbolInput::Canonical(sym) => {
                SymbolNormalizer::to_exchange(exchange, sym, account_type)
            }
        }
    }
}

/// Display delegates to [`OwnedSymbolInput::as_str`] (Raw → raw, Canonical → base).
///
/// Bridge for existing WsProtocol impls pending θ.2 migration.
impl fmt::Display for OwnedSymbolInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for OwnedSymbolInput {
    fn from(s: String) -> Self {
        OwnedSymbolInput::Raw(s)
    }
}

impl From<&str> for OwnedSymbolInput {
    fn from(s: &str) -> Self {
        OwnedSymbolInput::Raw(s.to_string())
    }
}

impl From<Symbol> for OwnedSymbolInput {
    fn from(s: Symbol) -> Self {
        OwnedSymbolInput::Canonical(s)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AccountType, ExchangeId, Symbol};

    // Raw resolve → identity (no allocation for Raw variant)
    #[test]
    fn raw_resolve_identity() {
        let input = SymbolInput::Raw("BTCUSDT");
        let result = input
            .resolve(ExchangeId::Binance, AccountType::Spot)
            .expect("raw resolve should not fail");
        assert_eq!(&*result, "BTCUSDT");
        // Cow::Borrowed — no allocation
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    // Canonical resolve → normalized (Binance spot: "BTCUSDT")
    #[test]
    fn canonical_resolve_binance_spot() {
        let sym = Symbol::new("BTC", "USDT");
        let input = SymbolInput::Canonical(&sym);
        let result = input
            .resolve(ExchangeId::Binance, AccountType::Spot)
            .expect("canonical resolve should succeed for Binance");
        // Normalizer for Binance spot concatenates base+quote uppercase
        assert_eq!(&*result, "BTCUSDT");
    }

    // From<&str> impl
    #[test]
    fn from_str_ref() {
        let input: SymbolInput<'_> = "ETHUSDT".into();
        assert_eq!(input, SymbolInput::Raw("ETHUSDT"));
    }

    // From<&String> impl
    #[test]
    fn from_string_ref() {
        let s = String::from("SOLUSDT");
        let input: SymbolInput<'_> = (&s).into();
        assert_eq!(input, SymbolInput::Raw("SOLUSDT"));
    }

    // From<&Symbol> impl
    #[test]
    fn from_symbol_ref() {
        let sym = Symbol::new("ETH", "USDT");
        let input: SymbolInput<'_> = (&sym).into();
        assert_eq!(input, SymbolInput::Canonical(&sym));
    }

    // sym! macro tests live in the full `digdigdig3` crate next to the macro
    // (it stayed there in the 0.3.17 core extraction).

    // OwnedSymbolInput::Raw resolve → clone
    #[test]
    fn owned_raw_resolve() {
        let owned = OwnedSymbolInput::Raw("tBTCUSD".to_string());
        let result = owned
            .resolve(ExchangeId::Bitfinex, AccountType::Spot)
            .expect("owned raw resolve should not fail");
        assert_eq!(result, "tBTCUSD");
    }

    // OwnedSymbolInput::as_borrowed
    #[test]
    fn owned_as_borrowed() {
        let owned = OwnedSymbolInput::Raw("BTCUSDT".to_string());
        let borrowed = owned.as_borrowed();
        assert_eq!(borrowed, SymbolInput::Raw("BTCUSDT"));
    }

    // OwnedSymbolInput From impls
    #[test]
    fn owned_from_impls() {
        let from_str: OwnedSymbolInput = "XRPUSDT".into();
        let from_string: OwnedSymbolInput = String::from("XRPUSDT").into();
        let sym = Symbol::new("XRP", "USDT");
        let from_sym: OwnedSymbolInput = sym.clone().into();

        assert_eq!(from_str, OwnedSymbolInput::Raw("XRPUSDT".to_string()));
        assert_eq!(from_string, OwnedSymbolInput::Raw("XRPUSDT".to_string()));
        assert_eq!(from_sym, OwnedSymbolInput::Canonical(sym));
    }
}
