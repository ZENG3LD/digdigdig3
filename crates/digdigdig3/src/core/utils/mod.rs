//! # Utilities
//!
//! Общие утилиты для реализации коннекторов.
//!
//! ## Модули
//! - `crypto` - HMAC, hashing
//! - `encoding` - Base64, Hex
//! - `time` - Timestamps
//! - `rate_limiter` - Rate limiting utilities
//! - `precision` - Safe f64 → string conversion for prices and quantities
//! - `symbol_normalizer` - Canonical Symbol ↔ exchange-native raw string translation

mod crypto;
mod encoding;
mod time;
mod rate_limiter;
pub mod precision;
// symbol_normalizer is pure string logic — extracted to digdigdig3-core and
// re-exported here so `core::utils::symbol_normalizer::*` keeps working.
pub use digdigdig3_core::core::utils::symbol_normalizer;
pub mod validation_snapshot;
#[cfg(feature = "onchain-evm")]
pub mod crypto_evm;

pub use crypto::{hmac_sha256, hmac_sha256_hex, hmac_sha384, hmac_sha512, sha256, sha512};
pub use encoding::{encode_base64, encode_hex, encode_hex_lower};
pub use time::{timestamp_millis, timestamp_seconds, timestamp_iso8601};
pub(crate) use time::now_ms;
pub use rate_limiter::{
    DecayingRateLimiter, GroupRateLimiter, SimpleRateLimiter, WeightRateLimiter,
    RuntimeLimiter, RateLimitPressure, RateLimitMonitor,
};
pub use precision::{safe_price, safe_qty, format_price, format_qty, PrecisionCache, PrecisionInfo};
pub use symbol_normalizer::{NormalizerError, SymbolNormalizer};
