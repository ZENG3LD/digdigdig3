//! # Gemini Exchange Connector
//!
//! Full connector implementation for Gemini.
//!
//! ## Structure
//!
//! - `endpoints` — URLs and endpoint enum
//! - `auth`      — Request signing (HMAC-SHA384)
//! - `parser`    — JSON response parsing
//! - `connector` — GeminiConnector + trait impls
//! - `protocol`  — GeminiProtocol WsProtocol shim (wasm-compatible)
//! - `websocket` — GeminiWebSocket thin wrapper (wasm-compatible via UniversalWsTransport)

mod endpoints;
mod auth;
mod parser;
mod connector;
pub(crate) mod protocol;
mod websocket;

pub use endpoints::{GeminiEndpoint, GeminiUrls};
pub use auth::GeminiAuth;
pub use parser::GeminiParser;
pub use connector::GeminiConnector;
pub use websocket::GeminiWebSocket;
