//! # WebSocket Framework — Wave 0 Foundation
//!
//! Replaces duplicated per-exchange connect/ping/reconnect/dispatch loops with
//! a single generic `UniversalWsTransport<P: WsProtocol>`.
//!
//! Each exchange shrinks to a thin declarative shim (`WsProtocol` impl) providing:
//! - Endpoint URL
//! - Ping frame
//! - Subscribe/unsubscribe frames
//! - Topic extractor
//! - `TopicRegistry` mapping topic → parser
//!
//! The framework owns ALL connection lifecycle, ping scheduling, subscription replay,
//! frame routing, and unmatched-frame logging. Silent drops are architecturally
//! impossible: unmatched topic → `tracing::warn!`, never `Ok(None)`.

// base_websocket.rs is kept on disk but not compiled — Wave 2 will remove it.
// mod base_websocket;

pub mod capability_provider;
pub mod protocol;
pub mod reconnect;
// stream_kind / stream_spec / support_level are pure TYPES — extracted to the
// digdigdig3-core crate and re-exported here so `core::websocket::*` paths (and
// the item re-exports below) keep working unchanged.
pub use digdigdig3_core::core::websocket::{stream_kind, stream_spec, support_level};
pub mod topic_registry;
pub mod transport;

pub use capability_provider::CapabilityProvider;
pub use protocol::WsProtocol;
pub use reconnect::ReconnectConfig;
pub use stream_kind::{KlineInterval, StreamKind};
pub use stream_spec::StreamSpec;
pub use support_level::SupportLevel;
pub use topic_registry::{
    ParserFn, RegistryEntry, RegistryKey, TopicKey, TopicPattern, TopicRegistry,
    TopicRegistryBuilder, topic_pattern_matches,
};
pub use transport::{UniversalWsTransport, decode_binary_default};
