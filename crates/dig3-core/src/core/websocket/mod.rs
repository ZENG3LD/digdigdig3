//! WebSocket stream-description TYPES (no transport / connection logic).
//!
//! Only the pure type definitions live here — `StreamKind`, `KlineInterval`,
//! `StreamSpec`, `SupportLevel`. The actual WS framework (transport, protocol,
//! reconnect, topic registry) stays in the full `digdigdig3` crate.

pub mod stream_kind;
pub mod stream_spec;
pub mod support_level;

pub use stream_kind::{KlineInterval, StreamKind};
pub use stream_spec::StreamSpec;
pub use support_level::SupportLevel;
