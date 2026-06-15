use std::path::PathBuf;
use std::time::Duration;

use crate::{PersistenceConfig, Result, Station};

/// Fluent builder for [`Station`].
#[derive(Debug)]
pub struct StationBuilder {
    pub(crate) storage_root: PathBuf,
    pub(crate) persistence: PersistenceConfig,
    pub(crate) warm_start: usize,
    pub(crate) gap_heal: crate::GapHealConfig,
    pub(crate) unsubscribe_grace: Duration,
    /// Whether to issue a one-shot REST `get_orderbook` seed on first subscribe
    /// to an `Orderbook` or `OrderbookDelta` stream. Default: `false`.
    pub(crate) orderbook_rest_seed: bool,
    /// Depth passed to `get_orderbook` when `orderbook_rest_seed` is `true`.
    /// Default: `1000`.
    pub(crate) orderbook_seed_depth: usize,
}

impl Default for StationBuilder {
    fn default() -> Self {
        let env_root = std::env::var_os("DIG3_STORAGE_ROOT").map(PathBuf::from);
        Self {
            storage_root: env_root.unwrap_or_else(|| PathBuf::from("./dig3_storage")),
            persistence: PersistenceConfig::default(),
            warm_start: 500,
            gap_heal: crate::GapHealConfig::default(),
            unsubscribe_grace: Duration::ZERO,
            orderbook_rest_seed: false,
            orderbook_seed_depth: 1000,
        }
    }
}

impl StationBuilder {
    pub fn new() -> Self { Self::default() }

    /// Override the root directory under which all Station-managed artefacts
    /// (trades, bars, snapshots, indexes) are written.
    pub fn storage_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.storage_root = root.into();
        self
    }

    /// Configure trade/bar/snapshot persistence.
    pub fn persistence(mut self, cfg: PersistenceConfig) -> Self {
        self.persistence = cfg;
        self
    }

    /// Number of most-recent points to emit from disk on subscribe BEFORE live
    /// stream takes over. 0 disables warm-start. Acts as the in-memory
    /// series capacity too.
    ///
    /// Default: `500` (≈5 minutes of history for most warm-seedable kinds at
    /// typical event rates; gives derived bars a dense enough trade window to
    /// bootstrap without waiting for live events).
    pub fn warm_start(mut self, n: usize) -> Self {
        self.warm_start = n;
        self
    }

    /// Configure proactive gap-heal: when a live event arrives whose timestamp
    /// jumps further than the configured threshold past the last seen event,
    /// the station REST-backfills the missing window before emitting the live
    /// event. Off by default.
    ///
    /// On wasm32 the REST pull uses browser fetch; it succeeds for the 9
    /// proxy-override venues (Binance/Bybit/OKX/Bitget/Bitstamp/Coinbase/Kraken/
    /// Deribit/HTX) and silently returns an empty window for other venues until
    /// their REST CORS proxies are wired (Wave 4-C).
    pub fn gap_heal(mut self, cfg: crate::GapHealConfig) -> Self {
        self.gap_heal = cfg;
        self
    }

    /// How long to keep a WS forwarder alive after its last consumer drops.
    /// During this window a new subscription for the same key reuses the live
    /// connection — no reconnect, no REST re-seed.
    ///
    /// Default: `Duration::ZERO` (immediate drop, current behaviour).
    ///
    /// Typical UI value: `Duration::from_secs(30)` so a quick symbol-flip
    /// (BTC → ETH → BTC) stays on the same socket.
    pub fn unsubscribe_grace(mut self, grace: Duration) -> Self {
        self.unsubscribe_grace = grace;
        self
    }

    /// On first subscribe to an [`Stream::Orderbook`] or [`Stream::OrderbookDelta`]
    /// stream for a `(exchange, symbol, account)` key, also issue a one-shot REST
    /// `get_orderbook` call so the live book is seeded with up to `depth` levels
    /// before WS deltas start applying.
    ///
    /// Default: `false` (WS-only — matches pre-0.3.13 behaviour).
    ///
    /// On failure (REST not connected, exchange returns error, empty snapshot)
    /// the station continues with WS-only and logs a warning. Never aborts the
    /// subscribe.
    pub fn orderbook_rest_seed(mut self, enable: bool) -> Self {
        self.orderbook_rest_seed = enable;
        self
    }

    /// Depth of the REST seed snapshot. Only meaningful when
    /// [`Self::orderbook_rest_seed`] is `true`.
    ///
    /// Passed directly to `get_orderbook(symbol, Some(depth as u16), account)`.
    /// Clamped to `1..=u16::MAX` internally.
    ///
    /// Default: `1000`.
    pub fn orderbook_seed_depth(mut self, depth: usize) -> Self {
        self.orderbook_seed_depth = depth;
        self
    }

    pub async fn build(self) -> Result<Station> {
        Station::from_builder(self).await
    }
}
