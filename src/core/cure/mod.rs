//! Dataset cure utilities — integrity check, deduplication, sequence-gap detection.
//!
//! ## Modules
//!
//! - `integrity` — read-only analysis: counts, time gaps, sequence gaps, parse errors
//! - `dedup` — SHA256-keyed deduplication, writes to sister stream
//! - `gap` — orderbook sequence-gap detection via [`OrderBookTracker`]
//! - `repair` — combined pipeline: integrity → dedup → gap report

pub mod dedup;
pub mod gap;
pub mod integrity;
pub mod repair;

pub use dedup::Deduper;
pub use gap::{GapDetector, GapInfo};
pub use integrity::{IntegrityChecker, IntegrityReport, SequenceGap, TimeGap};
pub use repair::{RepairPipeline, RepairReport};
