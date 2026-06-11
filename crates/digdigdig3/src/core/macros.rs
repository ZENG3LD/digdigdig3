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

#[cfg(test)]
mod tests {
    use crate::core::types::{Symbol, SymbolInput};

    // Moved from dig3-core symbol_input.rs tests: the macro lives in THIS
    // crate post-extraction, so its tests must too.
    #[test]
    fn sym_macro_raw_literal() {
        let input = crate::sym!("BTCUSDT");
        assert_eq!(input, SymbolInput::Raw("BTCUSDT"));
    }

    #[test]
    fn sym_macro_canonical() {
        let sym = Symbol::new("BTC", "USDT");
        let input = crate::sym!(&sym);
        assert_eq!(input, SymbolInput::Canonical(&sym));
    }
}
