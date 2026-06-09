//! # digdigdig3-core
//!
//! The pure-data core of [`digdigdig3`](https://docs.rs/digdigdig3): market-data
//! and trading types (`Kline`, `Ticker`, `OrderBook`, `PublicTrade`, `FundingRate`,
//! `StreamKind`, `Symbol`, `ExchangeId`, …) with **zero** connectors and **zero**
//! network I/O.
//!
//! Consumers that only need the data structures (indicator libraries, analytics,
//! backtesters) depend on this crate and avoid compiling reqwest / tokio /
//! websockets / the 47 exchange connectors that live in the full `digdigdig3` crate.
//!
//! The full `digdigdig3` crate re-exports everything here under the SAME paths
//! (`digdigdig3::core::types::*`, etc.), so it stays a drop-in superset.

pub mod core;
