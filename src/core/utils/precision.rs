//! # Precision utilities
//!
//! Safe f64 → string conversion for exchange order prices and quantities.
//! Uses [`rust_decimal`] internally to avoid IEEE-754 drift that accumulates
//! when operating directly on `f64` values.
//!
//! ## Why not `f64` arithmetic?
//!
//! `0.1 + 0.2 == 0.30000000000000004` in IEEE-754. When you divide a price by
//! its `tick_size` with plain `f64` you may land on `99.9999…` instead of
//! `100.0`, causing the floor to snap down one tick — an invalid price that
//! the exchange will reject.
//!
//! The string-path (`price.to_string()` → `Decimal::from_str`) uses Ryu's
//! shortest-round-trip representation, which is always the "human intended"
//! value, making it the same approach as CCXT.
//!
//! ## Functions
//!
//! | Function | Rounding | Use case |
//! |---|---|---|
//! | [`safe_price`] | nearest (round) | limit/stop price fields |
//! | [`safe_qty`] | floor (truncate) | order quantity fields |
//! | [`format_price`] | nearest, trailing zeros | exchanges requiring exact decimal places |
//! | [`format_qty`] | floor, trailing zeros | exchanges requiring exact decimal places |

use rust_decimal::Decimal;
use std::str::FromStr;

/// Converts an f64 price to a string rounded to the nearest `tick_size`.
///
/// Uses string-path conversion (Ryu shortest round-trip) to avoid sub-tick
/// IEEE-754 drift. Rounding is *nearest* (same behaviour as CCXT) so that
/// `100.055` with tick `0.01` yields `"100.06"`.
///
/// Returns [`Decimal::normalize`]d output — no trailing zeros
/// (`"50000"` not `"50000.00"`). Use [`format_price`] if your exchange
/// requires a fixed number of decimal places.
///
/// # Examples
///
/// ```
/// use digdigdig3::safe_price;
/// assert_eq!(safe_price(100.05, "0.01"), "100.05");
/// assert_eq!(safe_price(100.054, "0.01"), "100.05");
/// assert_eq!(safe_price(100.055, "0.01"), "100.06");
/// // Fixes 0.1 + 0.2 drift
/// assert_eq!(safe_price(0.1_f64 + 0.2_f64, "0.01"), "0.3");
/// ```
pub fn safe_price(price: f64, tick_size: &str) -> String {
    let d = Decimal::from_str(&price.to_string()).unwrap_or_default();
    let tick = Decimal::from_str(tick_size).unwrap_or(Decimal::ONE);
    if tick.is_zero() {
        return d.normalize().to_string();
    }
    let steps = (d / tick).round();
    let rounded = steps * tick;
    rounded.normalize().to_string()
}

/// Converts an f64 quantity to a string truncated (floored) to `step_size`.
///
/// Quantity is always rounded **down** — never claim more than available.
/// This matches CCXT `TRUNCATE` mode and prevents over-spending errors.
///
/// Returns [`Decimal::normalize`]d output — no trailing zeros.
/// Use [`format_qty`] if your exchange requires a fixed decimal count.
///
/// # Examples
///
/// ```
/// use digdigdig3::safe_qty;
/// assert_eq!(safe_qty(1.999, "0.01"), "1.99");    // floor, NOT round
/// assert_eq!(safe_qty(0.12345, "0.001"), "0.123");
/// assert_eq!(safe_qty(0.999, "0.01"), "0.99");    // never rounds up
/// ```
pub fn safe_qty(qty: f64, step_size: &str) -> String {
    let d = Decimal::from_str(&qty.to_string()).unwrap_or_default();
    let step = Decimal::from_str(step_size).unwrap_or(Decimal::ONE);
    if step.is_zero() {
        return d.normalize().to_string();
    }
    let steps = (d / step).floor();
    let rounded = steps * step;
    rounded.normalize().to_string()
}

/// Formats a price with *exactly* the number of decimal places implied by
/// `tick_size`, padding with trailing zeros if needed.
///
/// Some exchanges (e.g. Binance futures) reject `"100.5"` when the tick is
/// `0.01` and require `"100.50"` instead. Use this variant in those cases.
///
/// Rounding is *nearest* (same as [`safe_price`]).
///
/// # Examples
///
/// ```
/// use digdigdig3::format_price;
/// assert_eq!(format_price(100.5,  "0.01"), "100.50");
/// assert_eq!(format_price(100.0,  "0.01"), "100.00");
/// assert_eq!(format_price(67543.2,"0.01"), "67543.20");
/// ```
pub fn format_price(price: f64, tick_size: &str) -> String {
    let d = Decimal::from_str(&price.to_string()).unwrap_or_default();
    let tick = Decimal::from_str(tick_size).unwrap_or(Decimal::ONE);
    if tick.is_zero() {
        return d.normalize().to_string();
    }
    let decimals = decimal_places(tick_size);
    let steps = (d / tick).round();
    let rounded = steps * tick;
    format!("{:.prec$}", rounded, prec = decimals)
}

/// Formats a quantity with *exactly* the number of decimal places implied by
/// `step_size`, padding with trailing zeros if needed, using floor rounding.
///
/// # Examples
///
/// ```
/// use digdigdig3::format_qty;
/// assert_eq!(format_qty(1.5,  "0.001"),   "1.500");
/// assert_eq!(format_qty(0.1,  "0.00001"), "0.10000");
/// ```
pub fn format_qty(qty: f64, step_size: &str) -> String {
    let d = Decimal::from_str(&qty.to_string()).unwrap_or_default();
    let step = Decimal::from_str(step_size).unwrap_or(Decimal::ONE);
    if step.is_zero() {
        return d.normalize().to_string();
    }
    let decimals = decimal_places(step_size);
    let steps = (d / step).floor();
    let rounded = steps * step;
    format!("{:.prec$}", rounded, prec = decimals)
}

/// Returns the number of decimal places encoded in a tick/step string.
///
/// `"0.001"` → 3, `"1"` → 0, `"0.5"` → 1.
fn decimal_places(s: &str) -> usize {
    s.find('.').map(|dot| s.len() - dot - 1).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === safe_price tests ===

    #[test]
    fn test_safe_price_basic() {
        assert_eq!(safe_price(100.05, "0.01"), "100.05");
        assert_eq!(safe_price(50000.0, "0.01"), "50000");
        assert_eq!(safe_price(1.23456, "0.001"), "1.235");
    }

    #[test]
    fn test_safe_price_sub_tick_drift() {
        // 100.05 is stored as 100.04999... in f64
        // With floor() this would give 100.04 — WRONG
        // With round() via string path — correct
        assert_eq!(safe_price(100.05, "0.01"), "100.05");
        assert_eq!(safe_price(0.1 + 0.2, "0.01"), "0.3");
    }

    #[test]
    fn test_safe_price_round_nearest() {
        assert_eq!(safe_price(100.054, "0.01"), "100.05");
        assert_eq!(safe_price(100.055, "0.01"), "100.06"); // round up
        assert_eq!(safe_price(100.056, "0.01"), "100.06");
    }

    #[test]
    fn test_safe_price_btc() {
        assert_eq!(safe_price(67543.25, "0.01"), "67543.25");
        assert_eq!(safe_price(67543.251, "0.01"), "67543.25");
        assert_eq!(safe_price(67543.255, "0.01"), "67543.26");
    }

    #[test]
    fn test_safe_price_large_tick() {
        // Some futures have tick_size = 0.5 or 1.0
        assert_eq!(safe_price(100.3, "0.5"), "100.5");
        assert_eq!(safe_price(100.2, "0.5"), "100");
        assert_eq!(safe_price(100.0, "1"), "100");
        assert_eq!(safe_price(100.6, "1"), "101");
    }

    // === safe_qty tests ===

    #[test]
    fn test_safe_qty_basic() {
        assert_eq!(safe_qty(1.999, "0.01"), "1.99"); // floor, NOT round
        assert_eq!(safe_qty(0.12345, "0.001"), "0.123");
    }

    #[test]
    fn test_safe_qty_never_exceed() {
        // Critical: quantity must NEVER be rounded UP
        assert_eq!(safe_qty(0.999, "0.01"), "0.99");
        assert_eq!(safe_qty(1.0, "0.01"), "1");
        assert_eq!(safe_qty(0.12399, "0.001"), "0.123");
    }

    #[test]
    fn test_safe_qty_btc() {
        assert_eq!(safe_qty(0.00012345, "0.00001"), "0.00012");
        assert_eq!(safe_qty(1.5, "0.001"), "1.5");
    }

    // === format_price tests (with trailing zeros) ===

    #[test]
    fn test_format_price_trailing_zeros() {
        assert_eq!(format_price(100.5, "0.01"), "100.50");
        assert_eq!(format_price(100.0, "0.01"), "100.00");
        assert_eq!(format_price(67543.2, "0.01"), "67543.20");
    }

    #[test]
    fn test_format_qty_trailing_zeros() {
        assert_eq!(format_qty(1.5, "0.001"), "1.500");
        assert_eq!(format_qty(0.1, "0.00001"), "0.10000");
    }

    // === edge cases ===

    #[test]
    fn test_zero_price() {
        assert_eq!(safe_price(0.0, "0.01"), "0");
        assert_eq!(safe_qty(0.0, "0.001"), "0");
    }

    #[test]
    fn test_very_small_values() {
        assert_eq!(safe_qty(0.00000001, "0.00000001"), "0.00000001");
        assert_eq!(safe_price(0.00000001, "0.00000001"), "0.00000001");
    }
}
