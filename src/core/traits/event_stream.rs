//! # EventStream — event production and filtering traits
//!
//! Defines the registration layer between raw chain providers and event consumers.
//!
//! ## Role in the architecture
//!
//! ```text
//! Chain Node / RPC / Indexer API
//!        │
//!        ▼
//! ┌────────────────────┐
//! │   EventProducer    │  (this module)
//! │  get_events()      │  ← historical range query
//! │  poll_events()     │  ← incremental polling
//! └────────────────────┘
//!        │
//!        ▼ Vec<OnChainEvent>
//! downstream consumers (analytics, storage, UI)
//! ```
//!
//! ## Design constraints
//!
//! - **No interpretation**: producers emit raw factual events only.
//! - **Object-safe**: `EventProducer` is `dyn`-compatible — no generics in methods.
//! - **Chain-agnostic filter**: [`EventFilter`] uses string keys so it works
//!   across all chain families without per-chain enums.

use async_trait::async_trait;

use crate::core::types::{onchain::ChainId, onchain::OnChainEvent, ExchangeError};

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT FILTER
// ═══════════════════════════════════════════════════════════════════════════════

/// Subscription filter for on-chain event queries.
///
/// All fields are additive (`AND` across categories, `OR` within a category).
/// An empty `EventFilter` (all fields empty / `None`) matches every event.
///
/// # Example
///
/// ```
/// use digdigdig3::core::traits::event_stream::EventFilter;
///
/// // Only USDC transfers and Uniswap swaps involving a specific wallet
/// let filter = EventFilter {
///     event_types: vec!["token_transfer".to_string(), "dex_swap".to_string()],
///     addresses: vec!["0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string()],
///     token_addresses: vec!["0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()],
///     min_usd_value: Some(1000.0),
///     protocols: vec![],
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Restrict to specific event categories.
    ///
    /// Matches the `type` tag from [`OnChainEventType`] serde output:
    /// `"token_transfer"`, `"native_transfer"`, `"dex_swap"`, `"liquidity_change"`,
    /// `"pool_created"`, `"lending_action"`, `"staking_action"`, `"bridge_transfer"`,
    /// `"nft_transfer"`, `"contract_deployed"`, `"contract_interaction"`,
    /// `"governance_action"`, `"utxo_spent"`, `"coinbase_reward"`,
    /// `"mempool_transaction"`, `"custom"`.
    ///
    /// Empty list = no restriction (all event types pass through).
    pub event_types: Vec<String>,

    /// Restrict to events involving any of these addresses.
    ///
    /// Address matching is role-agnostic: `from`, `to`, `sender`, `contract`,
    /// `deployer`, `voter`, or any similar field in the event payload.
    /// Format must match the chain's native representation (e.g. checksummed
    /// hex for EVM, base58 for Solana).
    ///
    /// Empty list = no restriction.
    pub addresses: Vec<String>,

    /// Restrict to events involving any of these token contract/mint addresses.
    ///
    /// For EVM this is the ERC-20 contract address. For Solana this is the
    /// SPL mint. For Cosmos this is the IBC denom string.
    ///
    /// Empty list = no restriction.
    pub token_addresses: Vec<String>,

    /// Minimum USD value threshold.
    ///
    /// Events whose `usd_value` or `usd_volume` field is below this threshold
    /// are excluded. Events with no USD value recorded are **not** excluded
    /// (they pass through regardless of this threshold).
    ///
    /// `None` = no minimum.
    pub min_usd_value: Option<f64>,

    /// Restrict to events from specific protocol names.
    ///
    /// Matches the `protocol` field inside DEX, lending, staking, and bridge
    /// variants. Examples: `"uniswap_v3"`, `"aave_v3"`, `"wormhole"`.
    ///
    /// Empty list = no restriction.
    pub protocols: Vec<String>,
}

impl EventFilter {
    /// Returns `true` if this filter has no restrictions — all events pass.
    pub fn is_empty(&self) -> bool {
        self.event_types.is_empty()
            && self.addresses.is_empty()
            && self.token_addresses.is_empty()
            && self.min_usd_value.is_none()
            && self.protocols.is_empty()
    }

    /// Convenience constructor: filter by a single event type.
    pub fn for_type(event_type: impl Into<String>) -> Self {
        Self {
            event_types: vec![event_type.into()],
            ..Default::default()
        }
    }

    /// Convenience constructor: filter by a single address (any role).
    pub fn for_address(address: impl Into<String>) -> Self {
        Self {
            addresses: vec![address.into()],
            ..Default::default()
        }
    }

    /// Convenience constructor: filter by a single protocol.
    pub fn for_protocol(protocol: impl Into<String>) -> Self {
        Self {
            protocols: vec![protocol.into()],
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT PRODUCER TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Trait for components that produce structured on-chain events.
///
/// An `EventProducer` wraps a data source (blockchain node, indexer API,
/// WebSocket feed) and translates its output into [`OnChainEvent`] values.
///
/// ## Implementors
///
/// - Per-chain on-chain connectors (EVM, Solana, Bitcoin, Cosmos, …)
/// - Third-party indexer adapters (Bitquery, WhaleAlert, Dune, etc.)
/// - WebSocket stream adapters that convert raw messages to typed events
///
/// ## Object safety
///
/// The trait is object-safe and can be used as `Box<dyn EventProducer>` or
/// `Arc<dyn EventProducer>` to hold heterogeneous producers in a collection.
///
/// ## Error handling
///
/// Both methods return `Result<_, ExchangeError>`. Transient errors (rate
/// limits, network timeouts) should use the appropriate `ExchangeError`
/// variant rather than panicking. Callers are responsible for retry logic.
#[async_trait]
pub trait EventProducer: Send + Sync {
    /// Identifies which blockchain this producer is monitoring.
    fn chain_id(&self) -> ChainId;

    /// Fetch historical events for a closed block range `[from_block, to_block]`.
    ///
    /// Both block boundaries are **inclusive**. The returned events may be in
    /// any order; callers should sort by `(block, log_index)` if ordering matters.
    ///
    /// `filter` constrains the result set. Pass `&EventFilter::default()` to
    /// receive all events in the range.
    ///
    /// # Errors
    ///
    /// - [`ExchangeError::RateLimit`] — request was throttled by the data source.
    /// - [`ExchangeError::Network`] — transport-level failure.
    /// - [`ExchangeError::Parse`] — response could not be decoded.
    /// - [`ExchangeError::InvalidRequest`] — `to_block < from_block` or block
    ///   range exceeds provider limits.
    async fn get_events(
        &self,
        from_block: u64,
        to_block: u64,
        filter: &EventFilter,
    ) -> Result<Vec<OnChainEvent>, ExchangeError>;

    /// Fetch new events since the last successful poll.
    ///
    /// The producer is responsible for tracking the last-seen block internally.
    /// On the first call (or after a reset) this should return recent events
    /// from the current tip.
    ///
    /// Intended for lightweight periodic polling. For continuous streaming,
    /// consumers should use a WebSocket-based producer if available.
    ///
    /// `filter` constrains the result set. Pass `&EventFilter::default()` to
    /// receive all new events.
    ///
    /// # Errors
    ///
    /// Same error variants as [`get_events`](EventProducer::get_events).
    async fn poll_events(
        &self,
        filter: &EventFilter,
    ) -> Result<Vec<OnChainEvent>, ExchangeError>;
}
