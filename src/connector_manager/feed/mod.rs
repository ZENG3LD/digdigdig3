//! `MarketFeed` — high-level event-stream facade over `ExchangeHub`.
//!
//! Wraps `WebSocketConnector::subscribe` + `event_stream()` into a unified
//! per-subscription `tokio::sync::broadcast` channel so multiple consumers
//! (charting, indicators, persistence) can fan out from one upstream socket.
//!
//! Exchanges live in the hub. The feed never instantiates connectors —
//! it borrows `Arc<dyn WebSocketConnector>` via `hub.ws(id, account_type)`.
//!
//! Options are turned on/off through `FeedBuilder` (persistence, ws_multiplex,
//! reconnect backoff, orderbook tracker, symbol cache). v0 wires only the
//! subscribe API; storage/tracker/reconnect get scaffolded but disabled.

mod builder;
mod handle;
mod options;
mod feed;

pub use builder::FeedBuilder;
pub use feed::MarketFeed;
pub use handle::{FeedHandle, FeedEvent};
pub use options::{PersistenceOption, ReconnectPolicy, OrderbookTrackerOpt};
