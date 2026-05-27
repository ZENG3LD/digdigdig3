//! Unified time-series infrastructure for Station data-classes.
//!
//! All Station-managed market data — trades, bars, orderbook snapshots,
//! tickers, mark prices, funding rates, open interest, liquidations, agg
//! trades — flows through the same plumbing:
//!
//! ```text
//!   WS event ──► forwarder ──► Series<T>.push(memory ring)
//!                          ├─► DiskStore<T>.append (fixed-record binary)
//!                          └─► broadcast::Sender ──► consumer
//!
//!   subscribe ──► Series<T>.seed_from_disk() ──► emit LastN immediately
//!             └─► then attach to live stream
//! ```
//!
//! The key abstraction is [`DataPoint`] — each market data type provides a
//! fixed-size on-disk encoding + an extractor from `StreamEvent`. Series and
//! DiskStore are generic over `DataPoint`.
//!
//! Pattern borrowed from `mylittlechart::bar-service` (SharedSeriesMap +
//! seed_from_disk + flush_dirty), generalized over data-class.

pub mod data_point;
pub mod key;
pub mod map;
pub mod series;

// DiskStore uses std::fs, sled, and blocking I/O — native-only.
#[cfg(not(target_arch = "wasm32"))]
pub mod store;

pub use data_point::DataPoint;
pub use key::{Kind, SeriesKey};
pub use map::SharedSeriesMap;
pub use series::Series;

// native-only re-exports
#[cfg(not(target_arch = "wasm32"))]
pub use key::PollSpec;

#[cfg(not(target_arch = "wasm32"))]
pub use store::DiskStore;
