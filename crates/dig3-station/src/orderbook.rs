//! OrderBookTracker — in-memory orderbook reconstruction from snapshot + delta stream.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use digdigdig3_station::orderbook::{OrderBookTracker, OrderBookError};
//!
//! let mut tracker = OrderBookTracker::new("BTCUSDT");
//! tracker.apply_snapshot(&snapshot)?;
//! tracker.apply_delta(&delta)?;
//! let (best_bid, best_ask) = tracker.bbo().unwrap();
//! ```
//!
//! Apply a snapshot first, then deltas. Detects sequence gaps when `prev_update_id` is
//! populated on the delta and `last_update_id` was set on the previous update.

use std::collections::BTreeMap;
use rust_decimal::Decimal;

use digdigdig3_core::core::types::{OrderBook, OrderbookDelta};

// ─────────────────────────────────────────────────────────────────────────────
// DecimalKey — BTreeMap-safe wrapper
// ─────────────────────────────────────────────────────────────────────────────

/// Newtype wrapping [`Decimal`] so it can be used as a [`BTreeMap`] key.
///
/// [`Decimal`] implements `PartialOrd` but not `Ord` in all versions.
/// This wrapper provides a total order that matches numeric ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DecimalKey(Decimal);

impl PartialOrd for DecimalKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DecimalKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during orderbook reconstruction.
#[derive(Debug, thiserror::Error)]
pub enum OrderBookError {
    /// A delta arrived whose `prev_update_id` does not match the book's `last_update_id`.
    ///
    /// The book is now in an inconsistent state; the consumer should re-request a snapshot.
    #[error("sequence gap: book last_update_id={last}, delta prev_update_id={got}")]
    SequenceGap { last: u64, got: u64 },

    /// A delta was applied before any snapshot was loaded.
    #[error("delta applied before snapshot — call apply_snapshot first")]
    NoSnapshot,

    /// The symbol on the snapshot or delta does not match the tracker's symbol.
    #[error("symbol mismatch: tracker={tracker}, incoming={incoming}")]
    SymbolMismatch { tracker: String, incoming: String },

    /// A price or size value could not be converted to `Decimal`.
    #[error("invalid price/size value: {value}")]
    InvalidDecimal { value: f64 },
}

// ─────────────────────────────────────────────────────────────────────────────
// OrderBookTracker
// ─────────────────────────────────────────────────────────────────────────────

/// Maintains a live orderbook from snapshot + delta stream.
///
/// # Ordering
/// - `bids`: ascending `DecimalKey` → iterate in **reverse** for descending price order
/// - `asks`: ascending `DecimalKey` → iterate forward for ascending price order
///
/// # Sequence gap detection
/// When a delta carries `prev_update_id` and the book has a known `last_update_id`,
/// they must match.  On mismatch [`OrderBookError::SequenceGap`] is returned and the
/// book is left unchanged — the consumer should request a fresh snapshot.
#[derive(Debug, Clone)]
pub struct OrderBookTracker {
    /// Symbol this tracker was created for.
    pub symbol: String,
    /// Bids keyed by price ascending; iterate in reverse for best-bid-first.
    bids: BTreeMap<DecimalKey, Decimal>,
    /// Asks keyed by price ascending; iterate forward for best-ask-first.
    asks: BTreeMap<DecimalKey, Decimal>,
    /// `last_update_id` from the most recent snapshot or delta.
    last_update_id: Option<u64>,
    /// Timestamp (Unix ms) of the most recent update.
    last_timestamp_ms: i64,
    /// Whether a snapshot has been applied.
    has_snapshot: bool,
}

impl OrderBookTracker {
    /// Create a new empty tracker for the given symbol.
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: None,
            last_timestamp_ms: 0,
            has_snapshot: false,
        }
    }

    // ── Snapshot ──────────────────────────────────────────────────────────────

    /// Replace the full book state with the given snapshot.
    ///
    /// The `snapshot.symbol` is not checked (snapshots from `OrderBook` REST
    /// calls do not carry a symbol field — the caller is responsible for
    /// routing the right snapshot to the right tracker).
    /// Pass `symbol` explicitly via the second argument to verify.
    ///
    /// Clears all previous bids/asks, then populates from the snapshot levels.
    pub fn apply_snapshot(&mut self, snapshot: &OrderBook) -> Result<(), OrderBookError> {
        self.bids.clear();
        self.asks.clear();

        for level in &snapshot.bids {
            let price = to_decimal(level.price)?;
            let qty = to_decimal(level.size)?;
            if qty > Decimal::ZERO {
                self.bids.insert(DecimalKey(price), qty);
            }
        }
        for level in &snapshot.asks {
            let price = to_decimal(level.price)?;
            let qty = to_decimal(level.size)?;
            if qty > Decimal::ZERO {
                self.asks.insert(DecimalKey(price), qty);
            }
        }

        self.last_update_id = snapshot.last_update_id;
        self.last_timestamp_ms = snapshot.timestamp;
        self.has_snapshot = true;
        Ok(())
    }

    // ── Delta ─────────────────────────────────────────────────────────────────

    /// Apply an incremental delta to the live book.
    ///
    /// Rules:
    /// - Returns [`OrderBookError::NoSnapshot`] if no snapshot has been applied.
    /// - Checks `prev_update_id` against `last_update_id` when both are present.
    /// - A level with size `0.0` removes that price level; otherwise upserts.
    pub fn apply_delta(&mut self, delta: &OrderbookDelta) -> Result<(), OrderBookError> {
        if !self.has_snapshot {
            return Err(OrderBookError::NoSnapshot);
        }

        // Sequence check: best-effort — not all exchanges populate prev_update_id
        if let (Some(last), Some(prev)) = (self.last_update_id, delta.prev_update_id) {
            if prev != last {
                return Err(OrderBookError::SequenceGap { last, got: prev });
            }
        }

        for level in &delta.bids {
            let price = to_decimal(level.price)?;
            let qty = to_decimal(level.size)?;
            if qty == Decimal::ZERO {
                self.bids.remove(&DecimalKey(price));
            } else {
                self.bids.insert(DecimalKey(price), qty);
            }
        }
        for level in &delta.asks {
            let price = to_decimal(level.price)?;
            let qty = to_decimal(level.size)?;
            if qty == Decimal::ZERO {
                self.asks.remove(&DecimalKey(price));
            } else {
                self.asks.insert(DecimalKey(price), qty);
            }
        }

        if let Some(uid) = delta.last_update_id {
            self.last_update_id = Some(uid);
        }
        self.last_timestamp_ms = delta.timestamp;
        Ok(())
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Top `n` bid levels, highest price first.
    pub fn top_bids(&self, n: usize) -> Vec<(Decimal, Decimal)> {
        self.bids.iter().rev().take(n).map(|(k, v)| (k.0, *v)).collect()
    }

    /// Top `n` ask levels, lowest price first.
    pub fn top_asks(&self, n: usize) -> Vec<(Decimal, Decimal)> {
        self.asks.iter().take(n).map(|(k, v)| (k.0, *v)).collect()
    }

    /// Best bid/ask pair (best_bid, best_ask). `None` if either side is empty.
    pub fn bbo(&self) -> Option<(Decimal, Decimal)> {
        let bid = self.bids.iter().rev().next()?.0 .0;
        let ask = self.asks.iter().next()?.0 .0;
        Some((bid, ask))
    }

    /// Mid price: `(best_bid + best_ask) / 2`. `None` if book is empty.
    pub fn mid(&self) -> Option<Decimal> {
        let (b, a) = self.bbo()?;
        Some((b + a) / Decimal::new(2, 0))
    }

    /// Spread: `best_ask - best_bid`. `None` if book is empty.
    pub fn spread(&self) -> Option<Decimal> {
        let (b, a) = self.bbo()?;
        Some(a - b)
    }

    /// Sum of all bid quantities.
    pub fn total_bid_volume(&self) -> Decimal {
        self.bids.values().copied().sum()
    }

    /// Sum of all ask quantities.
    pub fn total_ask_volume(&self) -> Decimal {
        self.asks.values().copied().sum()
    }

    /// `(bid_levels, ask_levels)` — number of distinct price levels per side.
    pub fn depth(&self) -> (usize, usize) {
        (self.bids.len(), self.asks.len())
    }

    /// `last_update_id` from the most recent snapshot or delta.
    pub fn last_update_id(&self) -> Option<u64> {
        self.last_update_id
    }

    /// Unix millisecond timestamp of the most recent update.
    pub fn last_timestamp_ms(&self) -> i64 {
        self.last_timestamp_ms
    }

    /// Whether a snapshot has been applied.
    pub fn has_snapshot(&self) -> bool {
        self.has_snapshot
    }

    /// Reset the tracker to an empty state (keeps symbol).
    pub fn reset(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.last_update_id = None;
        self.last_timestamp_ms = 0;
        self.has_snapshot = false;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn to_decimal(v: f64) -> Result<Decimal, OrderBookError> {
    Decimal::try_from(v).map_err(|_| OrderBookError::InvalidDecimal { value: v })
}
