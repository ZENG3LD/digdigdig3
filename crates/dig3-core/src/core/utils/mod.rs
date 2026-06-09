//! Pure utility types needed by the core data types.
//!
//! Only `symbol_normalizer` (string normalization for symbol parsing) lives
//! here — the crypto / encoding / time / rate-limiter helpers stay in the full
//! `digdigdig3` crate, since they are connector-side concerns.

pub mod symbol_normalizer;

pub use symbol_normalizer::{NormalizerError, SymbolNormalizer};
