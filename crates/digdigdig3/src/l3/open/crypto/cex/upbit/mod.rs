//! # Upbit Exchange Connector
//!
//! Full connector for Upbit (Korean exchange).
//!
//! ## Structure
//!
//! - `endpoints` — URLs and endpoint enum
//! - `auth` — JWT-based request signing
//! - `parser` — JSON parsing (REST and WebSocket)
//! - `connector` — UpbitConnector + trait impls
//! - `protocol` — WsProtocol shim (UniversalWsTransport)
//! - `websocket` — thin wrapper over UniversalWsTransport<UpbitProtocol>
//!
//! ## Regions
//!
//! Upbit operates in multiple regions:
//! - Korea (kr) — default; KRW markets
//! - Singapore (sg)
//! - Indonesia (id)
//! - Thailand (th)
//!
//! WebSocket hardcodes the Korea endpoint (KRW markets).

mod endpoints;
mod auth;
mod parser;
mod connector;
mod protocol;
mod websocket;

pub use endpoints::{UpbitEndpoint, UpbitUrls};
pub use auth::UpbitAuth;
pub use parser::UpbitParser;
pub use connector::UpbitConnector;
pub use websocket::UpbitWebSocket;
