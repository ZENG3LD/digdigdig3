//! WsProtocol trait — per-exchange protocol shim.
//!
//! Implement this for each exchange. All methods are sync (no I/O).
//! The transport calls them to construct frames and route incoming data.

use std::time::Duration;

use serde_json::Value;
use url::Url;

use crate::core::rt::WsFrame;
use crate::core::traits::Credentials;
use crate::core::types::{AccountType, WebSocketError};

use super::{
    stream_kind::StreamKind,
    stream_spec::StreamSpec,
    topic_registry::{TopicKey, TopicRegistry},
};

/// Per-exchange protocol shim.  Implement this for each exchange.
/// All methods are sync (no I/O).  The transport calls them to construct frames
/// and route incoming data.
///
/// Implementors MUST be Send + Sync + 'static (Arc-shared across tasks).
pub trait WsProtocol: Send + Sync + 'static {
    // ── Identity ──────────────────────────────────────────────────────────

    /// Short exchange name for log targets (e.g. "binance", "okx").
    fn name(&self) -> &'static str;

    /// WebSocket endpoint URL for given account type and network.
    fn endpoint(&self, account_type: AccountType, testnet: bool) -> Url;

    // ── Heartbeat ────────────────────────────────────────────────────────

    /// Frame to send as application-level ping.
    /// Return `None` to use native WebSocket Ping frames instead.
    ///
    /// - Bitget: `Some(WsFrame::Text("ping".into()))`
    /// - OKX:    `Some(WsFrame::Text("ping".into()))`
    /// - Binance: `None` (native WebSocket ping)
    /// - KuCoin: `Some(WsFrame::Text(json!({"id":..,"type":"ping"}).to_string()))`
    fn ping_frame(&self) -> Option<WsFrame>;

    /// Interval between application-level pings.
    /// Default: 30 seconds.
    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    // ── Subscription frames ───────────────────────────────────────────────

    /// Build the subscribe frame for one StreamSpec.
    /// Returns Err if the stream kind is not supported.
    fn subscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError>;

    /// Build the unsubscribe frame for one StreamSpec.
    /// Returns Err if the stream kind is not supported.
    fn unsubscribe_frame(&self, spec: &StreamSpec) -> Result<WsFrame, WebSocketError>;

    // ── Auth ──────────────────────────────────────────────────────────────

    /// Optional authentication frame sent BEFORE any subscribe frames.
    ///
    /// Return `None` for fully public connectors (Binance public, Kraken, etc.).
    /// Return `Some(msg)` for exchanges that require LOGIN before SUBSCRIBE:
    /// OKX, HTX, KuCoin futures (token-based), Bitget private.
    ///
    /// The transport sends this frame immediately after connection is established
    /// and waits `auth_ack_timeout()` for an ack before proceeding.
    fn auth_frame(&self, credentials: &Credentials) -> Option<Result<WsFrame, WebSocketError>>;

    /// How long to wait for the auth ack before timing out.
    /// Only relevant when `auth_frame` returns `Some(_)`.
    fn auth_ack_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    /// Returns true if the given raw frame is an auth success acknowledgment.
    /// Called only when `auth_frame` is `Some(_)`.
    fn is_auth_ack(&self, raw: &Value) -> bool {
        let _ = raw;
        false
    }

    // ── Frame classification ──────────────────────────────────────────────

    /// Extract the routing topic from an incoming frame.
    ///
    /// Returns `None` for:
    /// - Pong frames ("pong" text body on OKX/Bitget)
    /// - Subscribe ack frames
    /// - Auth ack frames
    /// - Heartbeat frames
    ///
    /// Returns `Some(TopicKey)` for data frames.
    ///
    /// The transport calls this, looks up in TopicRegistry, calls parser if found,
    /// or emits `tracing::warn!` if not found.
    fn extract_topic(&self, raw: &Value) -> Option<TopicKey>;

    /// Returns true if the frame is a pong response to our ping.
    /// Used to suppress warn! for unmatched pong frames.
    fn is_pong(&self, raw: &Value) -> bool {
        let _ = raw;
        false
    }

    /// Returns true if the frame is a subscribe/unsubscribe acknowledgment.
    /// Used to suppress warn! for unmatched ack frames.
    fn is_subscribe_ack(&self, raw: &Value) -> bool {
        let _ = raw;
        false
    }

    // ── Registry ─────────────────────────────────────────────────────────

    /// Return the topic registry for this exchange+account_type combination.
    ///
    /// Called once at transport construction.  The registry is built at impl time
    /// and cached — this method does NOT allocate per-call.
    ///
    /// Most exchanges need one registry per AccountType (spot vs futures have
    /// different topic formats).  Pattern: cache in `OnceLock<TopicRegistry>`.
    fn topic_registry(&self, account_type: AccountType) -> &TopicRegistry;

    // ── Capability hints (optional, all default to empty) ─────────────────

    /// Stream kinds this exchange has NO channel for (not a dig3 gap — exchange itself
    /// does not provide it for the given account type).
    fn unsupported_by_exchange(&self, account_type: AccountType) -> &'static [StreamKind] {
        let _ = account_type;
        &[]
    }

    /// Stream kinds that nominally exist but require credentials even for public data.
    fn requires_auth_kinds(&self, account_type: AccountType) -> &'static [StreamKind] {
        let _ = account_type;
        &[]
    }

    // ── Optional binary decode hook ───────────────────────────────────────

    /// Decode a binary frame to a JSON Value.
    ///
    /// Default: tries gzip, then zlib, then raw deflate, then UTF-8.
    /// Override only when the exchange uses a non-standard encoding.
    fn decode_binary(&self, bytes: &[u8]) -> Result<Value, WebSocketError> {
        crate::core::websocket::transport::decode_binary_default(bytes)
    }

    // ── Optional pre-connect hook (async) ─────────────────────────────────

    /// Optional async hook called before each connection attempt.
    ///
    /// Used by KuCoin and similar exchanges that require a REST call to get
    /// a dynamic WebSocket endpoint URL before connecting.
    ///
    /// Returns:
    /// - `Ok(None)` — use `endpoint()` as usual (default).
    /// - `Ok(Some(url))` — override the endpoint with this URL for this connection.
    /// - `Err(_)` — pre-connect failed; transport will retry with backoff.
    ///
    /// The default implementation does nothing and returns `Ok(None)`.
    fn pre_connect_hook<'a>(
        &'a self,
        _http: &'a reqwest::Client,
        _account_type: AccountType,
        _testnet: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<Url>, WebSocketError>> + Send + 'a>,
    > {
        Box::pin(std::future::ready(Ok(None)))
    }
}
