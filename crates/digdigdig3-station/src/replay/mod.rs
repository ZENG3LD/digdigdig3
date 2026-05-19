//! Phase ν — replay layer for dig3.
//!
//! Reads stored `EventLog` events from `StorageManager` and re-emits them
//! through an API that mirrors the live `ExchangeHub` + `WebSocketConnector`
//! surface.  Drop-in for backtest and regression testing without modifying
//! consumer code.
//!
//! ## Quick start
//!
//! ```no_run
//! # use std::path::PathBuf;
//! # use digdigdig3_station::replay::{ReplayHub, ReplayConfig, ReplayRate};
//! # use digdigdig3::core::types::{AccountType, ExchangeId, SubscriptionRequest, Symbol};
//! # use futures_util::StreamExt;
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let hub = ReplayHub::new(ReplayConfig {
//!     storage_root: PathBuf::from("./data/events"),
//!     rate: ReplayRate::Accelerated(10.0),
//!     from_ms: None,
//!     to_ms: None,
//! }).await?;
//!
//! hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false).await?;
//! let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();
//! ws.subscribe(SubscriptionRequest::ticker(Symbol::new("BTC", "USDT"))).await?;
//!
//! let mut stream = ws.event_stream();
//! while let Some(ev) = stream.next().await {
//!     println!("{:?}", ev?);
//! }
//! # Ok(())
//! # }
//! ```

pub mod hub;
pub mod rate;
pub mod reader;
pub mod ws;

pub use hub::{ReplayConfig, ReplayHub};
pub use rate::ReplayRate;
pub use ws::ReplayWebSocket;
