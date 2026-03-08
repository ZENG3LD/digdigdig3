//! # Feodo Tracker (abuse.ch) Connector
//!
//! Category: data_feeds
//! Type: Threat Intelligence / Botnet C2 Tracker
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free, CC0 license)
//!
//! ## Data Types
//! - C2 blocklists: Yes (IP addresses, ports, metadata)
//! - Malware families: 5 tracked (Emotet, TrickBot, QakBot, Dridex, BazarLoader)
//! - Threat intelligence: Yes (ASN, geolocation, timestamps)
//! - IDS/IPS rules: Available but not covered by this connector
//!
//! ## Key Endpoints
//! - /downloads/ipblocklist.json - Full blocklist (30-day, with metadata)
//! - /downloads/ipblocklist_recommended.json - Recommended IPs (lowest FP rate)
//! - /downloads/ipblocklist_aggressive.csv - Historical blocklist (all time)
//!
//! ## Rate Limits
//! - No hard limits
//! - Recommended poll rate: Every 5-15 minutes
//! - Data generation: Every 5 minutes
//!
//! ## Data Coverage
//! - Botnet families: Emotet, TrickBot, QakBot, Dridex, BazarLoader
//! - C2 infrastructure: Global monitoring
//! - Historical depth: Aggressive list contains all tracked C2s
//! - Update frequency: Every 5 minutes
//!
//! ## Current Status (February 2026)
//! - Datasets are currently EMPTY due to successful law enforcement operations:
//!   - Operation Emotet (2021): Dismantled Emotet
//!   - Operation Endgame (2024): Disrupted TrickBot, QakBot, BazarLoader
//! - Infrastructure remains active and will track new C2s if they emerge
//!
//! ## Usage Restrictions
//! - License: CC0 (Creative Commons Zero)
//! - Commercial use: Permitted without limitations
//! - Non-commercial use: Permitted without limitations
//! - Attribution: Not required but appreciated
//! - Data provided "as is" on best effort basis

mod auth;
mod connector;
mod endpoints;
mod parser;

pub use auth::FeodoTrackerAuth;
pub use connector::FeodoTrackerConnector;
pub use endpoints::{FeodoTrackerEndpoint, FeodoTrackerEndpoints};
pub use parser::{BlocklistStats, C2Server, C2Status, FeodoTrackerParser};
