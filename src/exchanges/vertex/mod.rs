//! # Vertex Protocol Exchange Connector
//!
//! ⚠️ **IMPORTANT: SERVICE PERMANENTLY SHUT DOWN** ⚠️
//!
//! Vertex Protocol was acquired by Ink Foundation (Kraken-backed L2) and
//! completely shut down on **August 14, 2025**.
//!
//! **All endpoints are permanently offline. This connector will not work.**
//!
//! See: research/vertex/ENDPOINTS_DEEP_RESEARCH.md for full details
//!
//! ---
//!
//! Complete implementation of connector for Vertex Protocol.
//!
//! ## Structure
//!
//! - `endpoints` - URLs and endpoint enum
//! - `auth` - EIP-712 signature authentication
//! - `parser` - JSON response parsing
//! - `connector` - VertexConnector + trait implementations
//! - `websocket` - WebSocket connection
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::vertex::VertexConnector;
//!
//! let connector = VertexConnector::new(credentials, false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let order = connector.market_order(symbol, side, qty, AccountType::Spot).await?;
//!
//! // Extended methods (Vertex-specific)
//! let products = connector.get_all_products().await?;
//! let product_id = connector.get_product_id("BTC-PERP").await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{VertexEndpoint, VertexUrls};
pub use auth::{VertexAuth, TimeInForce, to_x18, from_x18};
pub use parser::VertexParser;
pub use connector::VertexConnector;
pub use websocket::VertexWebSocket;
