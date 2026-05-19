//! # Testing — Generic test harness for all connectors
//!
//! Provides a unified test suite that can validate any connector
//! implementing the core traits. Tests automatically skip features
//! the connector doesn't support (returns UnsupportedOperation).

pub mod env_loader;
pub mod harness;
pub mod assertions;
#[path = "suites.rs"]
pub mod suites;
