use thiserror::Error;

#[derive(Debug, Error)]
pub enum StationError {
    #[error("station not yet built")]
    NotBuilt,
    #[error("subscription failed: {0}")]
    Subscribe(String),
    /// The exchange does not expose this stream on the WS wire. Subscribe
    /// MUST NOT spawn a forwarder for this combination — heal/resub would
    /// loop forever. Surfaced from `subscribe_frame` returning either
    /// `WebSocketError::NotSupported` or `WebSocketError::UnsupportedOperation`.
    /// Consumers should treat this as a quiet "skip" rather than a hard
    /// failure: it's an architectural fact about the venue, not a runtime fault.
    #[error("stream not supported by exchange: {0}")]
    StreamNotSupported(String),
    #[error("core error: {0}")]
    Core(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl StationError {
    /// True when this error originates from the venue not exposing the
    /// requested stream (wire-not-present), as opposed to a transient
    /// runtime failure. Used by `Station::subscribe(set)` to bucket
    /// per-stream outcomes in the returned `SubscribeReport`.
    pub fn is_not_supported(&self) -> bool {
        matches!(self, StationError::StreamNotSupported(_))
    }
}

pub type Result<T> = std::result::Result<T, StationError>;
