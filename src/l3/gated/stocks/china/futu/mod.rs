//! Futu OpenAPI connector
//!
//! Category: stocks/china
//! Type: Licensed broker (Hong Kong, US, Singapore, Australia)
//!
//! ## Architecture Limitation
//!
//! **IMPORTANT**: Futu OpenAPI uses a custom TCP protocol with Protocol Buffers,
//! NOT standard HTTP REST APIs. This connector is a **stub implementation** that
//! documents this limitation.
//!
//! ### Futu Architecture
//! - Uses OpenD gateway (local or remote server)
//! - Custom TCP protocol on port 11111
//! - Protocol Buffers for serialization
//! - SDK wrappers: Python, Java, C#, C++, JavaScript
//!
//! ### Integration Options
//!
//! 1. **Use Python SDK via FFI**: Call Futu's Python SDK from Rust
//! 2. **Implement Protocol Buffer client**: Reverse-engineer the protocol
//! 3. **Use external adapter**: Run Python adapter service, expose REST API
//! 4. **Wait for Rust SDK**: Contact Futu for official Rust support
//!
//! ## Features
//! - REST API: **No** (uses custom TCP)
//! - WebSocket: **No** (uses push subscriptions via TCP)
//! - Authentication: OpenD login + trade unlock
//! - Free tier: Yes (with quote subscriptions)
//!
//! ## Data Types
//! - Price data: Yes (via subscriptions)
//! - Historical data: Yes (20 years daily, minute bars)
//! - Options data: Yes (chains, Greeks, IV)
//! - Futures data: Yes (OI, positions)
//! - Broker queue: Yes (HK LV2 only)
//! - Capital flow: Yes (HK market)
//!
//! ## Current Status
//! **Not implemented** - Requires Protocol Buffer client or SDK bridge.
//!
//! All trait methods return `UnsupportedOperation` with guidance.

mod endpoints;
mod auth;
mod parser;
mod connector;

#[cfg(test)]
mod tests;

pub use connector::FutuConnector;
