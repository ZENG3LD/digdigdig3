//! # Uniswap DEX Connector
//!
//! Implementation of Uniswap V3 decentralized exchange connector.
//!
//! ## Architecture
//!
//! - `endpoints` - API URLs and endpoint enum
//! - `auth` - Minimal authentication (API keys)
//! - `parser` - JSON/GraphQL response parsing
//! - `connector` - UniswapConnector + trait implementations
//! - `websocket` - WebSocket connection (Ethereum events)
//!
//! ## Key Differences from CEX
//!
//! - **Token Addresses**: Uses contract addresses (0x...) instead of symbols
//! - **No Order Book**: AMM-based pricing with concentrated liquidity
//! - **Multiple Pools**: Same pair can have multiple pools with different fee tiers
//! - **On-Chain**: All data is on Ethereum blockchain
//! - **Gas Costs**: Every transaction requires ETH for gas
//!
//! ## APIs Used
//!
//! 1. **Trading API (REST)** - Uniswap Labs hosted service
//!    - Quote generation
//!    - Swap execution
//!    - Requires API key
//!
//! 2. **The Graph Subgraph (GraphQL)** - Decentralized indexing
//!    - Historical data
//!    - Pool analytics
//!    - Public or with API key
//!
//! 3. **Ethereum RPC (JSON-RPC)** - Direct blockchain access
//!    - On-chain data
//!    - Contract calls
//!    - Token balances
//!
//! 4. **Ethereum WebSocket** - Real-time events
//!    - Swap events
//!    - Liquidity changes
//!    - Block updates
//!
//! ## Symbol Format
//!
//! Uniswap uses token contract addresses, not symbols:
//! - WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`
//! - USDC: `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
//!
//! The connector provides helpers to map common symbols to addresses.
//!
//! ## Fee Tiers
//!
//! Uniswap V3 has multiple fee tiers per pair:
//! - 0.01% (100 bps) - Stablecoins
//! - 0.05% (500 bps) - Low volatility
//! - 0.30% (3000 bps) - Standard
//! - 1.00% (10000 bps) - Exotic pairs
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::exchanges::uniswap::UniswapConnector;
//!
//! // Create connector with API key
//! let connector = UniswapConnector::new(Some(credentials), false).await?;
//!
//! // Core methods (from traits)
//! let price = connector.get_price(
//!     Symbol::new("WETH", "USDC"),
//!     AccountType::Spot
//! ).await?;
//!
//! // Extended methods (Uniswap-specific)
//! let quote = connector.get_quote("WETH", "USDC", "1000000000000000000", AccountType::Spot).await?;
//! ```
//!
//! ## Limitations (Phase 2)
//!
//! - Read-only market data (no trading execution)
//! - Limited pool coverage (known pools only)
//! - Simplified orderbook simulation
//! - WebSocket placeholder (not fully implemented)
//!
//! ## Future Enhancements (Phase 3+)
//!
//! - Transaction signing and execution
//! - Liquidity provision (add/remove)
//! - Multi-hop routing
//! - Full WebSocket event streaming
//! - Multi-chain support (Arbitrum, Polygon, etc.)
//! - V4 hooks support

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{
    UniswapEndpoint, UniswapUrls, FeeTier, PoolMetadata,
    tokens, contracts, KNOWN_POOLS, find_pool_metadata, find_pool_by_address
};
pub use auth::UniswapAuth;
pub use parser::UniswapParser;
pub use connector::UniswapConnector;
pub use websocket::{UniswapWebSocket, UniswapEvent, SwapData, MintData, BurnData};
