//! # Bitquery Connector
//!
//! GraphQL-based blockchain data provider for on-chain analytics.
//!
//! ## Important: Blockchain Data Provider (NOT an Exchange)
//!
//! Bitquery is a data provider for blockchain/on-chain data:
//! - NO trading operations (Trading trait returns UnsupportedOperation)
//! - NO account balances (Account trait returns UnsupportedOperation)
//! - NO traditional exchange data (CEX trades, orderbooks)
//!
//! ## Focus Areas
//!
//! - **DEX Trades** - Decentralized exchange trading data (Uniswap, PancakeSwap, Raydium, etc.)
//! - **Token Transfers** - ERC-20, ERC-721, ERC-1155, SPL token movements
//! - **NFT Data** - NFT marketplace trades and ownership
//! - **On-Chain Balances** - Historical and real-time wallet balances
//! - **Smart Contract Events** - Decoded event logs and function calls
//! - **Blockchain Infrastructure** - Blocks, transactions, mempool
//!
//! ## API Type: GraphQL
//!
//! Bitquery uses GraphQL, NOT REST:
//! - Single endpoint for all queries
//! - Flexible data selection via GraphQL syntax
//! - Real-time subscriptions via WebSocket (GraphQL subscriptions)
//! - 40+ blockchain networks (Ethereum, BSC, Solana, Bitcoin, etc.)
//!
//! ## Structure
//!
//! - `endpoints` - GraphQL queries and formatters
//! - `auth` - OAuth 2.0 token authentication
//! - `parser` - GraphQL response parsers
//! - `connector` - BitqueryConnector + trait implementations
//! - `websocket` - GraphQL subscription support
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::data_feeds::bitquery::BitqueryConnector;
//! use connectors_v5::Credentials;
//!
//! // Create connector with OAuth token
//! let credentials = Credentials {
//!     api_key: "ory_at_YOUR_OAUTH_TOKEN".to_string(),
//!     api_secret: String::new(),
//!     passphrase: None,
//! };
//!
//! let connector = BitqueryConnector::new(credentials).await?;
//!
//! // Get DEX trades (Uniswap V2 WETH/USDT)
//! let trades = connector.get_dex_trades(
//!     "eth",
//!     "uniswap_v2",
//!     "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
//!     "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
//!     100,
//! ).await?;
//!
//! // Get token transfers
//! let transfers = connector.get_token_transfers(
//!     "eth",
//!     "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
//!     100,
//! ).await?;
//!
//! // Get wallet balance updates
//! let balances = connector.get_balance_updates(
//!     "eth",
//!     "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb", // Vitalik
//!     100,
//! ).await?;
//! ```
//!
//! ## API Tiers
//!
//! - **Free Tier**: 10,000 points (first month), 10 req/min, 2 WS streams
//! - **Commercial**: Custom pricing, unlimited rows/streams, 24/7 support
//!
//! ## Points System
//!
//! - Realtime queries: 5 points per cube (flat rate)
//! - Archive queries: Variable (based on complexity)
//! - WebSocket subscriptions: 40 points/minute per stream
//!
//! ## Supported Blockchains (40+)
//!
//! - **EVM**: Ethereum, BSC, Polygon, Arbitrum, Base, Optimism, Avalanche, etc.
//! - **Non-EVM**: Solana, Bitcoin, Cardano, Ripple, Stellar, Algorand, Cosmos, Tron, etc.

mod endpoints;
mod auth;
mod parser;
mod connector;
mod websocket;

pub use endpoints::{BitqueryEndpoint, BitqueryUrls, BitqueryNetwork, BitqueryDataset};
pub use auth::BitqueryAuth;
pub use parser::{
    BitqueryParser,
    BitqueryResponse,
    DexTrade,
    TokenTransfer,
    BalanceUpdate,
    BlockData,
    TransactionData,
    SmartContractEvent,
};
pub use connector::BitqueryConnector;
pub use websocket::BitqueryWebSocket;
