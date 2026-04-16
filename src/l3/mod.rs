//! # L3 — Full Stack Connectors (L1 + L2 + Execution)
//!
//! Split into two access tiers:
//! - **open/** — Works without registration or API keys (CEX, DEX, prediction markets)
//! - **gated/** — Requires account, API keys, or KYC (brokers)

pub mod open;
pub mod gated;
