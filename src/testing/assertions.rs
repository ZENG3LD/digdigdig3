//! # Assertions — Sanity-check helpers for connector response types
//!
//! Each function returns `Ok(())` on success or `Err(String)` describing the
//! violation. Tests call these and treat `Err` as a test failure.
//!
//! Use `is_unsupported` to distinguish "feature not implemented" from "real error".

use crate::core::types::{ExchangeError, OrderBook, Kline, Ticker, Balance, Position};

// Timestamp lower bound: 2020-01-01T00:00:00Z in milliseconds
const MIN_TIMESTAMP_MS: i64 = 1_577_836_800_000;

// ═══════════════════════════════════════════════════════════════════════════════
// PRICE
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate that a price value is sane: positive, finite, and not NaN.
pub fn assert_price_sane(price: f64, context: &str) -> Result<(), String> {
    if price.is_nan() {
        return Err(format!("{context}: price is NaN"));
    }
    if price.is_infinite() {
        return Err(format!("{context}: price is infinite ({price})"));
    }
    if price <= 0.0 {
        return Err(format!("{context}: price must be positive, got {price}"));
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER BOOK
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate order book structure.
///
/// Checks:
/// - Bids are sorted descending by price
/// - Asks are sorted ascending by price
/// - All prices and sizes are positive
/// - Best bid < best ask (no crossed book)
pub fn assert_orderbook_sane(ob: &OrderBook) -> Result<(), String> {
    // Validate bid entries
    for (i, &(price, size)) in ob.bids.iter().enumerate() {
        if price <= 0.0 || price.is_nan() || price.is_infinite() {
            return Err(format!("orderbook: bid[{i}] price invalid: {price}"));
        }
        if size < 0.0 || size.is_nan() {
            return Err(format!("orderbook: bid[{i}] size invalid: {size}"));
        }
    }

    // Validate ask entries
    for (i, &(price, size)) in ob.asks.iter().enumerate() {
        if price <= 0.0 || price.is_nan() || price.is_infinite() {
            return Err(format!("orderbook: ask[{i}] price invalid: {price}"));
        }
        if size < 0.0 || size.is_nan() {
            return Err(format!("orderbook: ask[{i}] size invalid: {size}"));
        }
    }

    // Bids should be sorted descending
    for i in 1..ob.bids.len() {
        if ob.bids[i].0 > ob.bids[i - 1].0 {
            return Err(format!(
                "orderbook: bids not sorted descending at index {i}: {} > {}",
                ob.bids[i].0,
                ob.bids[i - 1].0
            ));
        }
    }

    // Asks should be sorted ascending
    for i in 1..ob.asks.len() {
        if ob.asks[i].0 < ob.asks[i - 1].0 {
            return Err(format!(
                "orderbook: asks not sorted ascending at index {i}: {} < {}",
                ob.asks[i].0,
                ob.asks[i - 1].0
            ));
        }
    }

    // Best bid must be less than best ask (no crossed book)
    if let (Some(&(best_bid, _)), Some(&(best_ask, _))) = (ob.bids.first(), ob.asks.first()) {
        if best_bid >= best_ask {
            return Err(format!(
                "orderbook: crossed book — best_bid {best_bid} >= best_ask {best_ask}"
            ));
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// KLINE
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate a single kline (OHLCV candle).
///
/// Checks:
/// - `high >= low`
/// - All OHLC prices are positive and finite
/// - `volume >= 0`
/// - `open_time` is after 2020-01-01
pub fn assert_kline_sane(kline: &Kline) -> Result<(), String> {
    for (label, price) in [
        ("open", kline.open),
        ("high", kline.high),
        ("low", kline.low),
        ("close", kline.close),
    ] {
        if price <= 0.0 || price.is_nan() || price.is_infinite() {
            return Err(format!("kline: {label} price invalid: {price}"));
        }
    }

    if kline.high < kline.low {
        return Err(format!(
            "kline: high ({}) < low ({})",
            kline.high, kline.low
        ));
    }

    if kline.volume < 0.0 || kline.volume.is_nan() {
        return Err(format!("kline: volume invalid: {}", kline.volume));
    }

    if kline.open_time < MIN_TIMESTAMP_MS {
        return Err(format!(
            "kline: open_time {} is before 2020-01-01",
            kline.open_time
        ));
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// TICKER
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate a ticker response.
///
/// Checks:
/// - `last_price > 0` and finite
/// - `volume_24h >= 0` if present
/// - `symbol` is non-empty
pub fn assert_ticker_sane(ticker: &Ticker) -> Result<(), String> {
    if ticker.symbol.is_empty() {
        return Err("ticker: symbol is empty".to_string());
    }

    assert_price_sane(ticker.last_price, "ticker.last_price")?;

    if let Some(vol) = ticker.volume_24h {
        if vol < 0.0 || vol.is_nan() {
            return Err(format!("ticker: volume_24h invalid: {vol}"));
        }
    }

    if let Some(bid) = ticker.bid_price {
        if bid <= 0.0 || bid.is_nan() || bid.is_infinite() {
            return Err(format!("ticker: bid_price invalid: {bid}"));
        }
    }

    if let Some(ask) = ticker.ask_price {
        if ask <= 0.0 || ask.is_nan() || ask.is_infinite() {
            return Err(format!("ticker: ask_price invalid: {ask}"));
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// BALANCE
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate a balance entry.
///
/// Checks:
/// - `free >= 0`
/// - `locked >= 0`
/// - `total >= 0`
/// - `asset` is non-empty
pub fn assert_balance_sane(balance: &Balance) -> Result<(), String> {
    if balance.asset.is_empty() {
        return Err("balance: asset is empty".to_string());
    }

    if balance.free < 0.0 || balance.free.is_nan() {
        return Err(format!("balance: free invalid: {}", balance.free));
    }

    if balance.locked < 0.0 || balance.locked.is_nan() {
        return Err(format!("balance: locked invalid: {}", balance.locked));
    }

    if balance.total < 0.0 || balance.total.is_nan() {
        return Err(format!("balance: total invalid: {}", balance.total));
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITION
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate a position entry.
///
/// Checks:
/// - `entry_price > 0` for open positions (quantity != 0)
/// - `symbol` is non-empty
pub fn assert_position_sane(pos: &Position) -> Result<(), String> {
    if pos.symbol.is_empty() {
        return Err("position: symbol is empty".to_string());
    }

    // For open positions, entry price must be positive
    if pos.quantity != 0.0 {
        assert_price_sane(pos.entry_price, "position.entry_price")?;
    }

    if pos.leverage == 0 {
        return Err("position: leverage must be >= 1".to_string());
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// ERROR CLASSIFICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns `true` if the error is `ExchangeError::UnsupportedOperation`.
///
/// Use this to skip tests for features that a connector deliberately does not
/// implement, rather than treating them as failures.
pub fn is_unsupported(err: &ExchangeError) -> bool {
    matches!(err, ExchangeError::UnsupportedOperation(_))
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{OrderBook, Kline, Ticker, Balance};

    // ── assert_price_sane ────────────────────────────────────────────────────

    #[test]
    fn test_price_valid() {
        assert!(assert_price_sane(50_000.0, "btc").is_ok());
    }

    #[test]
    fn test_price_nan() {
        assert!(assert_price_sane(f64::NAN, "btc").is_err());
    }

    #[test]
    fn test_price_inf() {
        assert!(assert_price_sane(f64::INFINITY, "btc").is_err());
    }

    #[test]
    fn test_price_zero() {
        assert!(assert_price_sane(0.0, "btc").is_err());
    }

    #[test]
    fn test_price_negative() {
        assert!(assert_price_sane(-1.0, "btc").is_err());
    }

    // ── assert_orderbook_sane ────────────────────────────────────────────────

    fn make_ob(bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) -> OrderBook {
        OrderBook { bids, asks, timestamp: MIN_TIMESTAMP_MS, sequence: None }
    }

    #[test]
    fn test_orderbook_valid() {
        let ob = make_ob(
            vec![(100.0, 1.0), (99.0, 2.0)],
            vec![(101.0, 1.0), (102.0, 2.0)],
        );
        assert!(assert_orderbook_sane(&ob).is_ok());
    }

    #[test]
    fn test_orderbook_crossed() {
        let ob = make_ob(
            vec![(105.0, 1.0)],
            vec![(100.0, 1.0)],
        );
        assert!(assert_orderbook_sane(&ob).is_err());
    }

    #[test]
    fn test_orderbook_bids_not_sorted() {
        let ob = make_ob(
            vec![(99.0, 1.0), (100.0, 1.0)], // ascending — wrong
            vec![(101.0, 1.0)],
        );
        assert!(assert_orderbook_sane(&ob).is_err());
    }

    #[test]
    fn test_orderbook_asks_not_sorted() {
        let ob = make_ob(
            vec![(99.0, 1.0)],
            vec![(102.0, 1.0), (101.0, 1.0)], // descending — wrong
        );
        assert!(assert_orderbook_sane(&ob).is_err());
    }

    #[test]
    fn test_orderbook_negative_size() {
        let ob = make_ob(
            vec![(100.0, -1.0)],
            vec![(101.0, 1.0)],
        );
        assert!(assert_orderbook_sane(&ob).is_err());
    }

    // ── assert_kline_sane ────────────────────────────────────────────────────

    fn make_kline(open: f64, high: f64, low: f64, close: f64) -> Kline {
        Kline {
            open_time: MIN_TIMESTAMP_MS,
            open,
            high,
            low,
            close,
            volume: 100.0,
            quote_volume: None,
            close_time: None,
            trades: None,
        }
    }

    #[test]
    fn test_kline_valid() {
        assert!(assert_kline_sane(&make_kline(100.0, 110.0, 90.0, 105.0)).is_ok());
    }

    #[test]
    fn test_kline_high_lt_low() {
        assert!(assert_kline_sane(&make_kline(100.0, 80.0, 90.0, 95.0)).is_err());
    }

    #[test]
    fn test_kline_zero_price() {
        assert!(assert_kline_sane(&make_kline(0.0, 110.0, 90.0, 105.0)).is_err());
    }

    #[test]
    fn test_kline_old_timestamp() {
        let mut k = make_kline(100.0, 110.0, 90.0, 105.0);
        k.open_time = 1_000_000; // before 2020
        assert!(assert_kline_sane(&k).is_err());
    }

    // ── assert_ticker_sane ───────────────────────────────────────────────────

    #[test]
    fn test_ticker_valid() {
        let t = Ticker {
            symbol: "BTCUSDT".to_string(),
            last_price: 50_000.0,
            bid_price: Some(49_999.0),
            ask_price: Some(50_001.0),
            high_24h: None,
            low_24h: None,
            volume_24h: Some(1000.0),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: MIN_TIMESTAMP_MS,
        };
        assert!(assert_ticker_sane(&t).is_ok());
    }

    #[test]
    fn test_ticker_empty_symbol() {
        let t = Ticker {
            symbol: "".to_string(),
            last_price: 50_000.0,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: MIN_TIMESTAMP_MS,
        };
        assert!(assert_ticker_sane(&t).is_err());
    }

    // ── assert_balance_sane ──────────────────────────────────────────────────

    #[test]
    fn test_balance_valid() {
        let b = Balance {
            asset: "USDT".to_string(),
            free: 100.0,
            locked: 10.0,
            total: 110.0,
        };
        assert!(assert_balance_sane(&b).is_ok());
    }

    #[test]
    fn test_balance_negative_free() {
        let b = Balance {
            asset: "USDT".to_string(),
            free: -1.0,
            locked: 0.0,
            total: 0.0,
        };
        assert!(assert_balance_sane(&b).is_err());
    }

    #[test]
    fn test_balance_empty_asset() {
        let b = Balance {
            asset: "".to_string(),
            free: 0.0,
            locked: 0.0,
            total: 0.0,
        };
        assert!(assert_balance_sane(&b).is_err());
    }

    // ── is_unsupported ───────────────────────────────────────────────────────

    #[test]
    fn test_is_unsupported_true() {
        let err = ExchangeError::UnsupportedOperation("not implemented".to_string());
        assert!(is_unsupported(&err));
    }

    #[test]
    fn test_is_unsupported_false() {
        let err = ExchangeError::Network("timeout".to_string());
        assert!(!is_unsupported(&err));
    }
}
