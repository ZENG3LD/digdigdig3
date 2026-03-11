//! Whale Alert connector
//!
//! Category: data_feeds
//! Type: Data Provider (Blockchain Transaction Analytics)
//!
//! ## Features
//! - REST API: Yes (Enterprise v2 + Developer v1 deprecated)
//! - WebSocket: Yes (Custom Alerts, Priority Alerts)
//! - Authentication: API Key (query parameter)
//! - Free tier: No (Developer v1 deprecated but functional)
//!
//! ## Data Types
//! - Price data: No (not a price provider)
//! - Historical data: Yes (30 days for transactions)
//! - Derivatives data: No
//! - Fundamentals: No
//! - On-chain: Yes (blockchain transactions, whale tracking)
//! - Macro data: No
//!
//! ## Supported Operations
//! - Track large blockchain transactions ("whale" movements)
//! - Address attribution (400+ known entities)
//! - Transaction streaming by blockchain
//! - Real-time alerts via WebSocket
//! - 11+ blockchains supported (Bitcoin, Ethereum, Tron, etc.)

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(feature = "websocket")]
mod websocket;

pub use connector::WhaleAlertConnector;
pub use auth::WhaleAlertAuth;
pub use parser::WhaleAlertParser;
pub use endpoints::{WhaleAlertEndpoint, WhaleAlertEndpoints, Blockchain, TransactionType};

#[cfg(feature = "websocket")]
pub use websocket::WhaleAlertWebSocket;
