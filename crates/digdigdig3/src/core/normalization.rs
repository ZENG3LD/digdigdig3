//! Canonical event normalization — typed conversion from raw exchange events
//! to a strict shape suitable for indicators/storage/backtest.
//!
//! # Invariants
//!
//! - `price` / `quantity` fields are [`rust_decimal::Decimal`] — no f64 rounding.
//! - `timestamp_ms` is always UTC milliseconds regardless of source unit (s / ms / µs / ns).
//! - `side` uses [`TradeSide`] (Buy/Sell) for trades.
//! - `symbol` is raw exchange-native (from Phase α — no internal normalization here).
//!
//! # Usage
//!
//! ```rust
//! use digdigdig3::core::normalization::{Canonicalize, CanonicalEvent};
//!
//! // Given a StreamEvent from any exchange WebSocket:
//! // if let Some(canonical) = event.canonicalize() {
//! //     match canonical {
//! //         CanonicalEvent::Trade(t) => println!("price={} qty={}", t.price, t.quantity),
//! //         _ => {}
//! //     }
//! // }
//! ```

use rust_decimal::Decimal;

use crate::core::types::{
    OrderBook, OrderbookDelta as OrderbookDeltaData, PublicTrade, Ticker, TradeSide,
};
use crate::core::types::{Kline, StreamEvent};
use crate::core::websocket::stream_kind::KlineInterval;


// ═══════════════════════════════════════════════════════════════════════════════
// CANONICAL TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Canonical trade — every field strict, no nulls, no f64.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalTrade {
    /// Raw exchange-native symbol (e.g. "BTCUSDT", "BTC-USDT").
    pub symbol: String,
    /// Fill price as exact Decimal.
    pub price: Decimal,
    /// Fill quantity as exact Decimal.
    pub quantity: Decimal,
    /// Aggressor side.
    pub side: TradeSide,
    /// UTC milliseconds.
    pub timestamp_ms: i64,
    /// Exchange-assigned trade identifier, if emitted.
    pub trade_id: Option<String>,
}

/// Canonical 24-hour ticker snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalTicker {
    pub symbol: String,
    pub last_price: Decimal,
    pub bid_price: Option<Decimal>,
    pub ask_price: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    /// UTC milliseconds.
    pub timestamp_ms: i64,
}

/// A single canonical price level (bid or ask).
///
/// `quantity == Decimal::ZERO` signals removal (used in delta updates).
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// Canonical orderbook snapshot.
///
/// - `bids` sorted **descending** by price (best bid first).
/// - `asks` sorted **ascending** by price (best ask first).
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalOrderbook {
    pub symbol: String,
    /// Sorted descending by price.
    pub bids: Vec<CanonicalLevel>,
    /// Sorted ascending by price.
    pub asks: Vec<CanonicalLevel>,
    pub sequence: Option<u64>,
    pub timestamp_ms: i64,
}

/// Canonical incremental orderbook update.
///
/// A level with `quantity == Decimal::ZERO` means "remove this price level".
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalOrderbookDelta {
    pub symbol: String,
    /// Bid-side updates (quantity=0 → remove).
    pub bid_updates: Vec<CanonicalLevel>,
    /// Ask-side updates (quantity=0 → remove).
    pub ask_updates: Vec<CanonicalLevel>,
    pub first_update_id: Option<u64>,
    pub last_update_id: Option<u64>,
    pub prev_update_id: Option<u64>,
    /// UTC milliseconds.
    pub timestamp_ms: i64,
}

/// Canonical OHLCV kline.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalKline {
    pub symbol: String,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    /// Open time — UTC milliseconds.
    pub open_time_ms: i64,
    /// Close time — UTC milliseconds.
    pub close_time_ms: i64,
    /// Typed kline interval (e.g. `KlineInterval::new("1m")`).
    pub interval: KlineInterval,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANONICAL EVENT ENVELOPE
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified canonical event — output of [`Canonicalize`] on [`StreamEvent`].
#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalEvent {
    Trade(CanonicalTrade),
    Ticker(CanonicalTicker),
    Orderbook(CanonicalOrderbook),
    OrderbookDelta(CanonicalOrderbookDelta),
    Kline(CanonicalKline),
    /// Events not yet mapped to a canonical form (private events, MarkPrice, etc.).
    Other,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANONICALIZE TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Conversion to a canonical typed form.
///
/// Returns `None` if critical fields are missing or unparseable.
/// Callers receive typed exact-precision data without per-exchange massaging.
pub trait Canonicalize {
    type Output;
    fn canonicalize(&self) -> Option<Self::Output>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TIMESTAMP NORMALIZER
// ═══════════════════════════════════════════════════════════════════════════════

/// Normalize a raw exchange timestamp to UTC milliseconds.
///
/// Heuristic based on digit count of the absolute value:
/// - ≤ 10 digits (≤ 9_999_999_999) → seconds → × 1_000
/// - 13 digits (10_000_000_000 to 9_999_999_999_999) → milliseconds → identity
/// - 16 digits → microseconds → ÷ 1_000
/// - ≥ 19 digits → nanoseconds → ÷ 1_000_000
/// - Zero or negative below seconds range → 0
pub fn normalize_ts_to_ms(ts: i64) -> i64 {
    let abs = ts.unsigned_abs();
    if abs > 10_000_000_000_000_000 {
        // nanoseconds (19-digit range)
        ts / 1_000_000
    } else if abs > 10_000_000_000_000 {
        // microseconds (16-digit range)
        ts / 1_000
    } else if abs > 10_000_000_000 {
        // milliseconds (13-digit range) — already correct
        ts
    } else if abs > 0 {
        // seconds (10-digit range)
        ts * 1_000
    } else {
        0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER: f64 → Decimal
// ═══════════════════════════════════════════════════════════════════════════════

#[inline]
fn f64_to_decimal(v: f64) -> Option<Decimal> {
    Decimal::try_from(v).ok()
}

#[inline]
fn f64_to_decimal_opt(v: Option<f64>) -> Option<Decimal> {
    v.and_then(|x| Decimal::try_from(x).ok())
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER: OrderBookLevel → CanonicalLevel
// ═══════════════════════════════════════════════════════════════════════════════

fn to_canonical_level(price: f64, size: f64) -> Option<CanonicalLevel> {
    Some(CanonicalLevel {
        price: f64_to_decimal(price)?,
        quantity: f64_to_decimal(size)?,
    })
}

fn levels_from_book_levels(
    levels: &[crate::core::types::OrderBookLevel],
) -> Vec<CanonicalLevel> {
    levels
        .iter()
        .filter_map(|l| to_canonical_level(l.price, l.size))
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERNAL HELPERS: payload → Canonical*
// ═══════════════════════════════════════════════════════════════════════════════
//
// These are crate-private free functions, not `impl Canonicalize for Payload`.
// Symbol (and interval, for klines) is taken as a parameter from the caller —
// the only correct caller is `<StreamEvent as Canonicalize>::canonicalize()`
// which destructures the variant and supplies the routing key.
//
// Keeping these as free fns (not trait impls) prevents external callers from
// invoking `payload.canonicalize()` and getting back a struct with
// `symbol: String::new()` placeholder.

fn trade_to_canonical(payload: &PublicTrade, symbol: String) -> Option<CanonicalTrade> {
    Some(CanonicalTrade {
        symbol,
        price: f64_to_decimal(payload.price)?,
        quantity: f64_to_decimal(payload.quantity)?,
        side: payload.side,
        timestamp_ms: normalize_ts_to_ms(payload.timestamp),
        trade_id: Some(payload.id.clone()),
    })
}

fn ticker_to_canonical(payload: &Ticker, symbol: String) -> Option<CanonicalTicker> {
    Some(CanonicalTicker {
        symbol,
        last_price: f64_to_decimal(payload.last_price)?,
        bid_price: f64_to_decimal_opt(payload.bid_price),
        ask_price: f64_to_decimal_opt(payload.ask_price),
        volume_24h: f64_to_decimal_opt(payload.volume_24h),
        timestamp_ms: normalize_ts_to_ms(payload.timestamp),
    })
}

fn orderbook_to_canonical(payload: &OrderBook, symbol: String) -> Option<CanonicalOrderbook> {
    let mut bids = levels_from_book_levels(&payload.bids);
    let mut asks = levels_from_book_levels(&payload.asks);

    bids.sort_by(|a, b| b.price.cmp(&a.price)); // descending
    asks.sort_by(|a, b| a.price.cmp(&b.price)); // ascending

    let sequence = payload
        .sequence
        .as_deref()
        .and_then(|s| s.parse::<u64>().ok())
        .or(payload.last_update_id);

    Some(CanonicalOrderbook {
        symbol,
        bids,
        asks,
        sequence,
        timestamp_ms: normalize_ts_to_ms(payload.timestamp),
    })
}

fn orderbook_delta_to_canonical(
    payload: &OrderbookDeltaData,
    symbol: String,
) -> Option<CanonicalOrderbookDelta> {
    Some(CanonicalOrderbookDelta {
        symbol,
        bid_updates: levels_from_book_levels(&payload.bids),
        ask_updates: levels_from_book_levels(&payload.asks),
        first_update_id: payload.first_update_id,
        last_update_id: payload.last_update_id,
        prev_update_id: payload.prev_update_id,
        timestamp_ms: normalize_ts_to_ms(payload.timestamp),
    })
}

fn kline_to_canonical(
    payload: &Kline,
    symbol: String,
    interval: KlineInterval,
) -> Option<CanonicalKline> {
    Some(CanonicalKline {
        symbol,
        open: f64_to_decimal(payload.open)?,
        high: f64_to_decimal(payload.high)?,
        low: f64_to_decimal(payload.low)?,
        close: f64_to_decimal(payload.close)?,
        volume: f64_to_decimal(payload.volume)?,
        open_time_ms: normalize_ts_to_ms(payload.open_time),
        close_time_ms: payload
            .close_time
            .map(normalize_ts_to_ms)
            .unwrap_or_else(|| normalize_ts_to_ms(payload.open_time)),
        interval,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPL: StreamEvent → CanonicalEvent
// ═══════════════════════════════════════════════════════════════════════════════

impl Canonicalize for StreamEvent {
    type Output = CanonicalEvent;

    fn canonicalize(&self) -> Option<CanonicalEvent> {
        match self {
            StreamEvent::Trade { symbol, trade } => {
                trade_to_canonical(trade, symbol.clone()).map(CanonicalEvent::Trade)
            }
            StreamEvent::Ticker { symbol, ticker } => {
                ticker_to_canonical(ticker, symbol.clone()).map(CanonicalEvent::Ticker)
            }
            StreamEvent::OrderbookSnapshot { symbol, book } => {
                orderbook_to_canonical(book, symbol.clone()).map(CanonicalEvent::Orderbook)
            }
            StreamEvent::OrderbookDelta { symbol, delta } => {
                orderbook_delta_to_canonical(delta, symbol.clone())
                    .map(CanonicalEvent::OrderbookDelta)
            }
            StreamEvent::Kline { symbol, interval, kline } => {
                kline_to_canonical(kline, symbol.clone(), interval.clone())
                    .map(CanonicalEvent::Kline)
            }
            StreamEvent::MarkPriceKline { symbol, interval, kline }
            | StreamEvent::IndexPriceKline { symbol, interval, kline }
            | StreamEvent::PremiumIndexKline { symbol, interval, kline } => {
                kline_to_canonical(kline, symbol.clone(), interval.clone())
                    .map(CanonicalEvent::Kline)
            }
            // All other variants (private events, mark price, funding, etc.)
            _ => Some(CanonicalEvent::Other),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::FromStr;

    // ── timestamp_normalization ──────────────────────────────────────────────

    #[test]
    fn timestamp_seconds_to_ms() {
        // 10-digit: seconds
        assert_eq!(normalize_ts_to_ms(1_700_000_000), 1_700_000_000_000);
    }

    #[test]
    fn timestamp_ms_identity() {
        // 13-digit: already milliseconds
        assert_eq!(normalize_ts_to_ms(1_700_000_000_000), 1_700_000_000_000);
    }

    #[test]
    fn timestamp_us_to_ms() {
        // 16-digit: microseconds → milliseconds
        assert_eq!(normalize_ts_to_ms(1_700_000_000_000_000), 1_700_000_000_000);
    }

    #[test]
    fn timestamp_ns_to_ms() {
        // 19-digit: nanoseconds → milliseconds
        assert_eq!(
            normalize_ts_to_ms(1_700_000_000_000_000_000),
            1_700_000_000_000
        );
    }

    #[test]
    fn timestamp_zero() {
        assert_eq!(normalize_ts_to_ms(0), 0);
    }

    // ── trade_canonicalize_basic ─────────────────────────────────────────────

    #[test]
    fn trade_canonicalize_basic() {
        let trade = PublicTrade {
            id: "12345".to_string(),
            price: 65432.1,
            quantity: 0.5,
            side: TradeSide::Buy,
            timestamp: 1_700_000_000_000,
        };

        let c = trade_to_canonical(&trade, "BTCUSDT".to_string()).expect("should canonicalize");
        assert_eq!(c.symbol, "BTCUSDT");
        assert_eq!(c.price, Decimal::try_from(65432.1_f64).unwrap());
        assert_eq!(c.quantity, Decimal::try_from(0.5_f64).unwrap());
        assert_eq!(c.side, TradeSide::Buy);
        assert_eq!(c.timestamp_ms, 1_700_000_000_000);
        assert_eq!(c.trade_id, Some("12345".to_string()));
    }

    #[test]
    fn trade_canonicalize_sell_side() {
        let trade = PublicTrade {
            id: "99".to_string(),
            price: 3200.0,
            quantity: 1.0,
            side: TradeSide::Sell,
            timestamp: 1_700_000_001_000,
        };
        let c = trade_to_canonical(&trade, "ETHUSDT".to_string()).expect("should canonicalize");
        assert_eq!(c.symbol, "ETHUSDT");
        assert_eq!(c.side, TradeSide::Sell);
    }

    // ── ticker_canonicalize_missing_bid_ask ──────────────────────────────────

    #[test]
    fn ticker_canonicalize_missing_bid_ask() {
        let ticker = Ticker {
            last_price: 180.0,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: 1_700_000_000_000,
        };

        let c = ticker_to_canonical(&ticker, "AAPL".to_string()).expect("should canonicalize");
        assert_eq!(c.symbol, "AAPL");
        assert_eq!(c.last_price, Decimal::try_from(180.0_f64).unwrap());
        assert!(c.bid_price.is_none());
        assert!(c.ask_price.is_none());
        assert!(c.volume_24h.is_none());
    }

    #[test]
    fn ticker_canonicalize_with_bid_ask() {
        let ticker = Ticker {
            last_price: 65000.0,
            bid_price: Some(64999.0),
            ask_price: Some(65001.0),
            high_24h: None,
            low_24h: None,
            volume_24h: Some(1234.5),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: 1_700_000_000_000,
        };

        let c = ticker_to_canonical(&ticker, "BTCUSDT".to_string()).expect("should canonicalize");
        assert!(c.bid_price.is_some());
        assert!(c.ask_price.is_some());
        assert!(c.volume_24h.is_some());
    }

    // ── kline_canonicalize_basic ─────────────────────────────────────────────

    #[test]
    fn kline_canonicalize_basic() {
        let kline = Kline {
            open_time: 1_700_000_000_000,
            open: 64000.0,
            high: 65000.0,
            low: 63500.0,
            close: 64800.0,
            volume: 123.456,
            quote_volume: None,
            close_time: Some(1_700_000_059_999),
            trades: None,
        };

        let c = kline_to_canonical(&kline, "BTCUSDT".to_string(), KlineInterval::new("1m"))
            .expect("should canonicalize");
        assert_eq!(c.symbol, "BTCUSDT");
        assert_eq!(c.interval, KlineInterval::new("1m"));
        assert_eq!(c.open, Decimal::try_from(64000.0_f64).unwrap());
        assert_eq!(c.high, Decimal::try_from(65000.0_f64).unwrap());
        assert_eq!(c.low, Decimal::try_from(63500.0_f64).unwrap());
        assert_eq!(c.close, Decimal::try_from(64800.0_f64).unwrap());
        assert_eq!(c.volume, Decimal::try_from(123.456_f64).unwrap());
        assert_eq!(c.open_time_ms, 1_700_000_000_000);
        assert_eq!(c.close_time_ms, 1_700_000_059_999);
    }

    // ── orderbook_canonical_sort_invariant ───────────────────────────────────

    #[test]
    fn orderbook_canonical_sort_invariant() {
        use crate::core::types::OrderBookLevel;

        // Deliberately unsorted input
        let ob = OrderBook {
            bids: vec![
                OrderBookLevel::new(100.0, 1.0),
                OrderBookLevel::new(102.0, 0.5),
                OrderBookLevel::new(101.0, 2.0),
            ],
            asks: vec![
                OrderBookLevel::new(105.0, 1.0),
                OrderBookLevel::new(103.0, 3.0),
                OrderBookLevel::new(104.0, 2.0),
            ],
            timestamp: 1_700_000_000_000,
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        };

        let c = orderbook_to_canonical(&ob, "BTCUSDT".to_string()).expect("should canonicalize");

        // Bids: descending
        assert_eq!(c.bids[0].price, Decimal::from_str("102").unwrap());
        assert_eq!(c.bids[1].price, Decimal::from_str("101").unwrap());
        assert_eq!(c.bids[2].price, Decimal::from_str("100").unwrap());

        // Asks: ascending
        assert_eq!(c.asks[0].price, Decimal::from_str("103").unwrap());
        assert_eq!(c.asks[1].price, Decimal::from_str("104").unwrap());
        assert_eq!(c.asks[2].price, Decimal::from_str("105").unwrap());
    }

    // ── stream_event_canonicalize ────────────────────────────────────────────

    #[test]
    fn stream_event_trade_canonicalize() {
        let event = StreamEvent::Trade {
            symbol: "BTCUSDT".to_string(),
            trade: PublicTrade {
                id: "1".to_string(),
                price: 65000.0,
                quantity: 0.1,
                side: TradeSide::Buy,
                timestamp: 1_700_000_000_000,
            },
        };

        match event.canonicalize() {
            Some(CanonicalEvent::Trade(t)) => {
                assert_eq!(t.symbol, "BTCUSDT");
            }
            other => panic!("expected CanonicalEvent::Trade, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_ticker_canonicalize() {
        let event = StreamEvent::Ticker {
            symbol: "ETHUSDT".to_string(),
            ticker: Ticker {
                last_price: 3000.0,
                bid_price: None,
                ask_price: None,
                high_24h: None,
                low_24h: None,
                volume_24h: None,
                quote_volume_24h: None,
                price_change_24h: None,
                price_change_percent_24h: None,
                timestamp: 1_700_000_000_000,
            },
        };

        match event.canonicalize() {
            Some(CanonicalEvent::Ticker(t)) => assert_eq!(t.symbol, "ETHUSDT"),
            other => panic!("expected CanonicalEvent::Ticker, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_other_canonicalize() {
        let event = StreamEvent::FundingRate {
            symbol: "BTCUSDT".to_string(),
            rate: 0.0001,
            next_funding_time: None,
            timestamp: 1_700_000_000_000,
        };
        assert!(matches!(event.canonicalize(), Some(CanonicalEvent::Other)));
    }

    #[test]
    fn stream_event_orderbook_snapshot_carries_symbol() {
        let event = StreamEvent::OrderbookSnapshot {
            symbol: "BTCUSDT".to_string(),
            book: crate::core::types::OrderBook {
                bids: vec![],
                asks: vec![],
                timestamp: 1_700_000_000_000,
                sequence: None,
                last_update_id: None,
                first_update_id: None,
                prev_update_id: None,
                event_time: None,
                transaction_time: None,
                checksum: None,
            },
        };
        match event.canonicalize() {
            Some(CanonicalEvent::Orderbook(ob)) => assert_eq!(ob.symbol, "BTCUSDT"),
            other => panic!("expected Orderbook, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_orderbook_delta_carries_symbol() {
        let event = StreamEvent::OrderbookDelta {
            symbol: "ETHUSDT".to_string(),
            delta: crate::core::types::OrderbookDelta {
                bids: vec![],
                asks: vec![],
                timestamp: 1_700_000_000_000,
                first_update_id: None,
                last_update_id: None,
                prev_update_id: None,
                event_time: None,
                checksum: None,
            },
        };
        match event.canonicalize() {
            Some(CanonicalEvent::OrderbookDelta(od)) => assert_eq!(od.symbol, "ETHUSDT"),
            other => panic!("expected OrderbookDelta, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_kline_carries_symbol_and_interval() {
        let event = StreamEvent::Kline {
            symbol: "BTCUSDT".to_string(),
            interval: KlineInterval::new("1m"),
            kline: crate::core::types::Kline {
                open_time: 1_700_000_000_000,
                open: 50000.0,
                high: 50100.0,
                low: 49900.0,
                close: 50050.0,
                volume: 10.0,
                close_time: Some(1_700_000_060_000),
                quote_volume: None,
                trades: None,
            },
        };
        match event.canonicalize() {
            Some(CanonicalEvent::Kline(k)) => {
                assert_eq!(k.symbol, "BTCUSDT");
                assert_eq!(k.interval, KlineInterval::new("1m"));
            }
            other => panic!("expected Kline, got {:?}", other),
        }
    }

    #[test]
    fn stream_event_mark_price_kline_carries_symbol_and_interval() {
        let event = StreamEvent::MarkPriceKline {
            symbol: "BTCUSDT".to_string(),
            interval: KlineInterval::new("5m"),
            kline: crate::core::types::Kline {
                open_time: 1_700_000_000_000,
                open: 50000.0,
                high: 50100.0,
                low: 49900.0,
                close: 50050.0,
                volume: 0.0,
                close_time: None,
                quote_volume: None,
                trades: None,
            },
        };
        match event.canonicalize() {
            Some(CanonicalEvent::Kline(k)) => {
                assert_eq!(k.symbol, "BTCUSDT");
                assert_eq!(k.interval, KlineInterval::new("5m"));
            }
            other => panic!("expected Kline, got {:?}", other),
        }
    }
}
