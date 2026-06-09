//! `core` — the data-type layer extracted from `digdigdig3`.
//!
//! Path-compatible with the parent crate: `digdigdig3::core::types::X` and
//! `digdigdig3_core::core::types::X` name the same item. The parent re-exports
//! these modules so existing `digdigdig3::core::*` paths keep working.

pub mod types;
pub mod websocket;
pub mod utils;
