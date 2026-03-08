//! # Utilities
//!
//! Общие утилиты для реализации коннекторов.
//!
//! ## Модули
//! - `crypto` - HMAC, hashing
//! - `encoding` - Base64, Hex
//! - `time` - Timestamps
//! - `rate_limiter` - Rate limiting utilities

mod crypto;
mod encoding;
mod time;
mod rate_limiter;

pub use crypto::{hmac_sha256, hmac_sha256_hex, hmac_sha384, hmac_sha512, sha256, sha512};
pub use encoding::{encode_base64, encode_hex, encode_hex_lower};
pub use time::{timestamp_millis, timestamp_seconds, timestamp_iso8601};
pub use rate_limiter::{DecayingRateLimiter, GroupRateLimiter, SimpleRateLimiter, WeightRateLimiter};
