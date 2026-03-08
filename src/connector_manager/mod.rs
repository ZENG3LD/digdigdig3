//! # Connector Manager
//!
//! Unified interface for all 51+ exchange connectors.
//!
//! ## Architecture
//!
//! This module provides a single enum-based type `AnyConnector` that wraps
//! all exchange connectors and implements all core traits via delegation.
//!
//! ## Components
//!
//! - `AnyConnector` - Enum wrapper for all connectors (51+ variants)
//! - Macros - Code generation for trait delegation
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::connector_manager::AnyConnector;
//! use std::sync::Arc;
//!
//! // Wrap any connector in the enum
//! let connector = AnyConnector::Binance(Arc::new(binance_connector));
//!
//! // Use core trait methods
//! let price = connector.get_price(symbol, AccountType::Spot).await?;
//! let balance = connector.get_balance(None, AccountType::Spot).await?;
//!
//! // Get connector metadata
//! let id = connector.id();
//! let name = connector.exchange_name();
//! ```
//!
//! ## Benefits
//!
//! 1. **Type Safety** - Compile-time guarantees for all connectors
//! 2. **No Dynamic Dispatch** - Enum match is faster than `dyn Trait`
//! 3. **Cheap Cloning** - Arc wrapper makes cloning O(1)
//! 4. **Single Type** - Store heterogeneous connectors in collections
//!
//! ## Trait Coverage
//!
//! Currently implemented:
//! - `ExchangeIdentity` - 3/3 methods (exchange_id, is_testnet, supported_account_types)
//! - `MarketData` - 5/5 methods (get_price, get_orderbook, get_klines, get_ticker, ping)
//!
//! To be added:
//! - `Trading` - market_order, limit_order, cancel_order, etc.
//! - `Account` - get_balance, get_account_info
//! - `Positions` - get_positions, get_funding_rate, set_leverage

#[macro_use]
mod macros;
mod connector;
mod registry;
mod pool;
mod aggregator;
mod config;
mod factory;

pub use connector::AnyConnector;
pub use registry::{
    AuthType, ConnectorCategory, ConnectorMetadata, ConnectorRegistry, Features, RateLimits,
};
pub use pool::{ConnectorPool, ConnectorPoolBuilder};
pub use aggregator::{BestBidAsk, ConnectorAggregator, ConnectorAggregatorBuilder};
pub use config::{ConnectorConfig, ConnectorConfigManager, ExchangeCredentials};
pub use factory::ConnectorFactory;
