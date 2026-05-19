//! Feed-level options. All toggles for the high-level layer live here as
//! `bool` flags or small enums, per the user's "bool/enum в билдере" rule.

use std::path::PathBuf;
use std::time::Duration;

/// Persistence behaviour for events flowing through the feed.
///
/// - `Off` — feed only fans out events; consumer writes to disk if needed.
/// - `Default` — feed owns a `StorageManager` rooted at `dig3_storage/`.
/// - `Custom(PathBuf)` — caller supplies a custom storage root path.
#[derive(Debug, Clone, Default)]
pub enum PersistenceOption {
    #[default]
    Off,
    Default,
    Custom(PathBuf),
}

/// Reconnect / backoff policy applied on top of `UniversalWsTransport`.
/// At v0 the transport's internal backoff is canonical and not overridable
/// from outside; this struct is the future-facing knob.
#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    pub enabled: bool,
    pub min_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            min_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
        }
    }
}

/// Whether the feed should reconstruct local order books from snapshot+delta
/// frames. v0: scaffold only.
#[derive(Debug, Clone)]
pub enum OrderbookTrackerOpt {
    Off,
    On { depth: usize },
}

impl Default for OrderbookTrackerOpt {
    fn default() -> Self { Self::Off }
}

#[allow(dead_code)] // wired in v1 (currently held but not read in feed.rs)
#[derive(Debug, Clone)]
pub(crate) struct FeedOptions {
    pub(crate) persistence: PersistenceOption,
    pub(crate) reconnect: ReconnectPolicy,
    pub(crate) orderbook: OrderbookTrackerOpt,
    pub(crate) unsub_grace: Duration,
    pub(crate) symbol_cache: bool,
    pub(crate) storage_root_override: Option<PathBuf>,
    pub(crate) broadcast_capacity: usize,
}

impl Default for FeedOptions {
    fn default() -> Self {
        Self {
            persistence: PersistenceOption::Off,
            reconnect: ReconnectPolicy::default(),
            orderbook: OrderbookTrackerOpt::Off,
            unsub_grace: Duration::from_secs(30),
            symbol_cache: false,
            storage_root_override: None,
            broadcast_capacity: 1024,
        }
    }
}
