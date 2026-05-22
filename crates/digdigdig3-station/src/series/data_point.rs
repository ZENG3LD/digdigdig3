//! `DataPoint` — common contract for everything a `Series<T>` can hold.
//!
//! Each market data class (trade, bar, ticker, OB snapshot, ...) provides:
//! - a fixed on-disk record size in bytes,
//! - encode/decode (little-endian, no allocations),
//! - timestamp accessor (used by warm-start / range queries),
//! - extractor from `digdigdig3::core::types::StreamEvent` (returns None if the
//!   event is for a different stream class).
//!
//! Variable-length payload is supported via opt-in blob hooks: if a type's
//! header carries a `(blob_offset, blob_len)` pointer pair at its tail, the
//! companion `.blob` file holds the actual variable-length bytes. Types that
//! do not need a blob inherit the default `None` impls and never trigger the
//! `.blob` codepath.

use digdigdig3::core::types::StreamEvent;

/// Implemented by every data-class held in a [`crate::series::Series`].
pub trait DataPoint: Sized + Clone + Send + Sync + 'static {
    /// On-disk record size in bytes. MUST be constant for the type.
    ///
    /// For types that use blob storage, this includes the trailing 12-byte
    /// `(blob_offset: u64, blob_len: u32)` pointer pair.
    const RECORD_SIZE: usize;

    /// Encode `self` to a fixed-size buffer (little-endian).
    ///
    /// For types using blob storage, the trailing 12-byte pointer is patched
    /// by [`crate::series::DiskStore`] AFTER `encode` runs — implementors do
    /// not need to fill it themselves. The buffer is zero-initialized.
    fn encode(&self, out: &mut [u8]);

    /// Decode from a fixed-size buffer. Returns None on malformed bytes.
    ///
    /// For types using blob storage, this path receives ONLY the header.
    /// `DiskStore` calls [`Self::decode_blob`] instead when the blob slice
    /// is needed to reconstruct string fields.
    fn decode(bytes: &[u8]) -> Option<Self>;

    /// Timestamp in milliseconds. Used for warm-start / range queries.
    fn timestamp_ms(&self) -> i64;

    /// Try to extract `Self` from a raw WS `StreamEvent`. Returns None if the
    /// event doesn't carry data for this class.
    fn from_stream_event(ev: &StreamEvent) -> Option<Self>;

    /// Variable-length bytes to append to the companion `.blob` file.
    ///
    /// Default: `None` — type uses fixed-size storage only. Override on
    /// types with string fields.
    ///
    /// Convention for multi-string variants: u16 length prefix per string,
    /// then UTF-8 bytes; strings in fixed order matching the type definition.
    fn encode_blob(&self) -> Option<Vec<u8>> { None }

    /// Reconstruct `Self` from header bytes + blob slice.
    ///
    /// Default: ignore blob, call [`Self::decode`]. Override on types that
    /// need to read string fields from the blob.
    fn decode_blob(header: &[u8], _blob: &[u8]) -> Option<Self> {
        Self::decode(header)
    }

    /// `Some(offset)` if the type uses blob storage; `None` otherwise.
    ///
    /// The pointer pair `(blob_offset: u64, blob_len: u32)` is written at
    /// `&header[offset..offset+12]`. Convention: header tail, so
    /// `offset = RECORD_SIZE - 12`. Returning `Some(_)` opts the type into
    /// `.blob` file creation in [`crate::series::DiskStore`].
    fn blob_pointer_offset() -> Option<usize> { None }
}
