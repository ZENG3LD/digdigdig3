//! `DataPoint` — common contract for everything a `Series<T>` can hold.
//!
//! Each market data class (trade, bar, ticker, OB snapshot, ...) provides:
//! - a fixed on-disk record size in bytes,
//! - encode/decode (little-endian, no allocations),
//! - timestamp accessor (used by warm-start / range queries),
//! - extractor from `digdigdig3::core::types::StreamEvent` (returns None if the
//!   event is for a different stream class).

use digdigdig3::core::types::StreamEvent;

/// Implemented by every data-class held in a [`crate::series::Series`].
pub trait DataPoint: Sized + Clone + Send + Sync + 'static {
    /// On-disk record size in bytes. MUST be constant for the type.
    const RECORD_SIZE: usize;

    /// Encode `self` to a fixed-size buffer (little-endian).
    fn encode(&self, out: &mut [u8]);

    /// Decode from a fixed-size buffer. Returns None on malformed bytes.
    fn decode(bytes: &[u8]) -> Option<Self>;

    /// Timestamp in milliseconds. Used for warm-start / range queries.
    fn timestamp_ms(&self) -> i64;

    /// Try to extract `Self` from a raw WS `StreamEvent`. Returns None if the
    /// event doesn't carry data for this class.
    fn from_stream_event(ev: &StreamEvent) -> Option<Self>;
}
