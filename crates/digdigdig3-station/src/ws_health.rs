use serde::{Deserialize, Serialize};

/// Snapshot of a live WS connection's health metrics.
///
/// All fields are `Option` because not every exchange supports every
/// metric. `None` means "exchange/connector doesn't report this".
///
/// Obtained via [`crate::Station::ws_health`] (per-key) or
/// [`crate::Station::ws_health_for_exchange`] (aggregate).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WsHealth {
    /// Last measured ping/pong round-trip in milliseconds.
    ///
    /// `None` if the exchange connector does not expose a ping-RTT handle.
    /// Currently populated for connectors that implement
    /// `ping_rtt_handle() -> Option<Arc<Mutex<u64>>>` (e.g. OKX).
    /// Other venues return `None` until per-connector wiring is added.
    pub rtt_ms: Option<u64>,

    /// Unix epoch millis of the most recent WS message of any kind.
    ///
    /// Used to detect silence-induced reconnects. `None` if the forwarder
    /// has not yet received any message since spawn (or if the underlying
    /// atomic is not yet wired for this connector).
    pub last_message_ms: Option<i64>,

    /// `true` if the forwarder is actively connected (not in grace period,
    /// not reconnecting). Derived from the presence of a live entry in the
    /// Station mux table — always accurate (no lag).
    pub connected: bool,
}
