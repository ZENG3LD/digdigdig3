//! Monotonic clock abstraction for native and wasm32 targets.
//!
//! `std::time::Instant` panics at runtime on wasm32-unknown-unknown because
//! the Rust standard library does not map `Instant` to any browser API on that
//! target. The `instant` crate provides a drop-in replacement that uses
//! `js_sys::Date::now()` (millisecond resolution) on wasm32.
//!
//! Use `crate::core::rt::clock::Instant` instead of `std::time::Instant`
//! everywhere the code must compile and run on both targets.

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub use instant::Instant;
