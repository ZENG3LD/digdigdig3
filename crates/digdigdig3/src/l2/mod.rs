//! # L2 — L1 + orderbook data (no execution)
//!
//! L2 connectors provide full market data including orderbook depth,
//! but do NOT support trading or account operations.

pub mod free;
pub mod paid;
