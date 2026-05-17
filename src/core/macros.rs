//! Core macros for dig3 connector usage.

/// Short syntax for constructing a [`SymbolInput`].
///
/// - `sym!("BTCUSDT")` → `SymbolInput::Raw("BTCUSDT")`
/// - `sym!(&my_symbol)` → `SymbolInput::Canonical(&my_symbol)`
///
/// For canonical from base/quote literals, construct [`Symbol`] directly:
/// `Symbol::new("BTC", "USDT")` then pass via `sym!(&sym)` or `(&sym).into()`.
///
/// [`SymbolInput`]: crate::core::types::SymbolInput
/// [`Symbol`]: crate::core::types::Symbol
#[macro_export]
macro_rules! sym {
    ($raw:literal) => {
        $crate::core::types::SymbolInput::Raw($raw)
    };
    (&$canonical:expr) => {
        $crate::core::types::SymbolInput::Canonical(&$canonical)
    };
}
