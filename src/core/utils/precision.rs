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
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;

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

/// Convert integer precision (number of decimal places) to tick/step string.
///
/// `2` → `"0.01"`, `4` → `"0.0001"`, `0` → `"1"`, `8` → `"0.00000001"`.
fn precision_to_tick(digits: u8) -> String {
    if digits == 0 {
        return "1".to_string();
    }
    let mut s = "0.".to_string();
    for _ in 0..(digits - 1) {
        s.push('0');
    }
    s.push('1');
    s
}

// ═══════════════════════════════════════════════════════════════════════════════
// PRECISION CACHE
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-symbol precision info for safe price/qty formatting.
#[derive(Clone, Debug)]
pub struct PrecisionInfo {
    /// Tick size as string, e.g. `"0.01"`.
    pub tick_size: String,
    /// Step size as string, e.g. `"0.001"`.
    pub step_size: String,
}

/// Thread-safe cache of per-symbol precision info.
///
/// Populated from [`ExchangeInfo`] after a connector calls `get_exchange_info()`.
/// Used internally by connectors in `place_order()` / `amend_order()` to convert
/// raw `f64` prices and quantities into safe, exchange-valid strings.
///
/// # Usage
///
/// ```ignore
/// // In connector constructor or lazy init:
/// let info = self.get_exchange_info().await?;
/// self.precision.load_from_symbols(&info.symbols);
///
/// // In place_order:
/// let price_str = self.precision.price(&symbol, price);
/// let qty_str   = self.precision.qty(&symbol, quantity);
/// ```
pub struct PrecisionCache {
    cache: RwLock<HashMap<String, PrecisionInfo>>,
}

impl PrecisionCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Load precision info from a slice of [`SymbolInfo`].
    ///
    /// For each symbol:
    /// - Uses `tick_size` / `step_size` if present and > 0
    /// - Falls back to `price_precision` / `quantity_precision` integer digits
    ///
    /// Call this after `get_exchange_info()` succeeds.
    pub fn load_from_symbols(&self, symbols: &[crate::core::types::SymbolInfo]) {
        let mut cache = self.cache.write().unwrap();
        for s in symbols {
            let tick = match s.tick_size {
                Some(t) if t > 0.0 => t.to_string(),
                _ => precision_to_tick(s.price_precision),
            };
            let step = match s.step_size {
                Some(t) if t > 0.0 => t.to_string(),
                _ => precision_to_tick(s.quantity_precision),
            };
            cache.insert(s.symbol.clone(), PrecisionInfo {
                tick_size: tick,
                step_size: step,
            });
        }
    }

    /// Get safe price string for a symbol (rounded to nearest tick).
    ///
    /// Falls back to raw `f64::to_string()` if symbol is not in cache.
    pub fn price(&self, symbol: &str, price: f64) -> String {
        if let Some(info) = self.cache.read().unwrap().get(symbol) {
            safe_price(price, &info.tick_size)
        } else {
            price.to_string()
        }
    }

    /// Get safe quantity string for a symbol (floored to step_size).
    ///
    /// Falls back to raw `f64::to_string()` if symbol is not in cache.
    pub fn qty(&self, symbol: &str, qty: f64) -> String {
        if let Some(info) = self.cache.read().unwrap().get(symbol) {
            safe_qty(qty, &info.step_size)
        } else {
            qty.to_string()
        }
    }

    /// Get formatted price with trailing zeros (for exchanges requiring exact decimal places).
    pub fn formatted_price(&self, symbol: &str, price: f64) -> String {
        if let Some(info) = self.cache.read().unwrap().get(symbol) {
            format_price(price, &info.tick_size)
        } else {
            price.to_string()
        }
    }

    /// Get formatted quantity with trailing zeros.
    pub fn formatted_qty(&self, symbol: &str, qty: f64) -> String {
        if let Some(info) = self.cache.read().unwrap().get(symbol) {
            format_qty(qty, &info.step_size)
        } else {
            qty.to_string()
        }
    }

    /// Check if precision info is loaded for a symbol.
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.cache.read().unwrap().contains_key(symbol)
    }

    /// Number of symbols in cache.
    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Whether the cache is empty (no symbols loaded).
    pub fn is_empty(&self) -> bool {
        self.cache.read().unwrap().is_empty()
    }
}

impl Default for PrecisionCache {
    fn default() -> Self {
        Self::new()
    }
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

    // === precision_to_tick ===

    #[test]
    fn test_precision_to_tick() {
        assert_eq!(precision_to_tick(0), "1");
        assert_eq!(precision_to_tick(1), "0.1");
        assert_eq!(precision_to_tick(2), "0.01");
        assert_eq!(precision_to_tick(4), "0.0001");
        assert_eq!(precision_to_tick(8), "0.00000001");
    }

    // === PrecisionCache ===

    #[test]
    fn test_cache_fallback_before_load() {
        let cache = PrecisionCache::new();
        // Before loading — raw f64 toString fallback
        assert_eq!(cache.price("BTCUSDT", 67543.251), "67543.251");
        assert_eq!(cache.qty("BTCUSDT", 0.123456), "0.123456");
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_with_tick_step() {
        let cache = PrecisionCache::new();
        let symbols = vec![
            crate::core::types::SymbolInfo {
                symbol: "BTCUSDT".to_string(),
                base_asset: "BTC".to_string(),
                quote_asset: "USDT".to_string(),
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 5,
                tick_size: Some(0.01),
                step_size: Some(0.00001),
                min_quantity: None,
                max_quantity: None,
                min_notional: None,
            },
        ];
        cache.load_from_symbols(&symbols);

        assert_eq!(cache.len(), 1);
        assert!(cache.has_symbol("BTCUSDT"));

        // Price — rounds to nearest tick
        assert_eq!(cache.price("BTCUSDT", 67543.251), "67543.25");
        assert_eq!(cache.price("BTCUSDT", 67543.255), "67543.26");

        // Qty — floors to step
        assert_eq!(cache.qty("BTCUSDT", 0.123456), "0.12345");
        assert_eq!(cache.qty("BTCUSDT", 0.123459), "0.12345"); // floor, NOT round

        // Unknown symbol — fallback
        assert_eq!(cache.price("UNKNOWN", 100.123), "100.123");
    }

    #[test]
    fn test_cache_digits_fallback() {
        let cache = PrecisionCache::new();
        // No tick_size/step_size — falls back to precision digits
        let symbols = vec![
            crate::core::types::SymbolInfo {
                symbol: "ETHUSDT".to_string(),
                base_asset: "ETH".to_string(),
                quote_asset: "USDT".to_string(),
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 3,
                tick_size: None,
                step_size: None,
                min_quantity: None,
                max_quantity: None,
                min_notional: None,
            },
        ];
        cache.load_from_symbols(&symbols);

        // precision 2 → tick "0.01", precision 3 → step "0.001"
        assert_eq!(cache.price("ETHUSDT", 3456.789), "3456.79");
        assert_eq!(cache.qty("ETHUSDT", 1.2345), "1.234");
    }

    #[test]
    fn test_cache_formatted_trailing_zeros() {
        let cache = PrecisionCache::new();
        let symbols = vec![
            crate::core::types::SymbolInfo {
                symbol: "BTCUSDT".to_string(),
                base_asset: "BTC".to_string(),
                quote_asset: "USDT".to_string(),
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 3,
                tick_size: Some(0.01),
                step_size: Some(0.001),
                min_quantity: None,
                max_quantity: None,
                min_notional: None,
            },
        ];
        cache.load_from_symbols(&symbols);

        assert_eq!(cache.formatted_price("BTCUSDT", 100.5), "100.50");
        assert_eq!(cache.formatted_qty("BTCUSDT", 1.5), "1.500");
    }
}
