//! # Connector Manager
//!
//! Unified interface for all 51+ exchange connectors.
//!
//! ## Architecture
//!
//! Connectors are dispatched through `Arc<dyn CoreConnector>` — a trait object
//! that composes all core sub-traits (ExchangeIdentity, MarketData, MarketDataPublic,
//! Trading, Account, Positions, etc.) with Send + Sync + 'static bounds.
//!
//! ## Components
//!
//! - `ConnectorPool` - Lock-free pool of `Arc<dyn CoreConnector>` by exchange
//! - `ConnectorFactory` - Constructs and wraps connectors from config
//! - `ConnectorAggregator` - Fan-out aggregation across multiple connectors
//! - `ConnectorRegistry` - Static metadata (supported features, categories)
//! - `ConnectorConfig` - Credentials and config management
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::connector_manager::{ConnectorFactory, CoreConnector};
//! use std::sync::Arc;
//!
//! // Build a connector through the factory
//! let connector: Arc<dyn CoreConnector> = factory.create_connector(exchange_id, credentials).await?;
//!
//! // Use core trait methods
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let balance = connector.get_balance(query).await?;
//!
//! // Get connector metadata
//! let id = connector.id();
//! let name = connector.exchange_name();
//! ```

mod registry;
mod pool;
mod aggregator;
mod config;
mod factory;

pub use registry::{
    AuthType, ConnectorCategory, ConnectorMetadata, ConnectorRegistry, Features,
};
pub use pool::{ConnectorPool, ConnectorPoolBuilder};
pub use aggregator::{BestBidAsk, ConnectorAggregator, ConnectorAggregatorBuilder};
pub use config::{ConnectorConfig, ConnectorConfigManager, ExchangeCredentials};
pub use factory::ConnectorFactory;
pub use crate::core::traits::CoreConnector;
