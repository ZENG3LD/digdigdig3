//! # Etherscan (Ethereum Blockchain Explorer) Connector
//!
//! Category: data_feeds
//! Type: Blockchain Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (query parameter)
//! - Free tier: Yes (5 calls/sec, 100k calls/day)
//!
//! ## Data Types
//! - Blockchain data: Yes (Ethereum)
//! - Transaction data: Yes
//! - Token data: Yes (ERC20)
//! - Gas prices: Yes
//! - Block data: Yes
//! - Smart contracts: Yes (ABI retrieval)
//!
//! ## Key Endpoints
//! - /api?module=account&action=balance - Get ETH balance
//! - /api?module=account&action=txlist - Transaction list
//! - /api?module=account&action=tokentx - ERC20 token transfers
//! - /api?module=stats&action=ethprice - ETH price
//! - /api?module=gastracker&action=gasoracle - Gas price oracle
//! - /api?module=proxy&action=eth_blockNumber - Latest block number
//! - /api?module=contract&action=getabi - Contract ABI
//!
//! ## Rate Limits
//! - Free tier: 5 requests per second
//! - Daily limit: 100,000 requests
//!
//! ## Data Coverage
//! - Ethereum mainnet and testnets
//! - Complete blockchain history
//! - Real-time data
//!
//! ## Usage Restrictions
//! - API key required for production use
//! - Rate limits enforced
//! - No commercial restrictions

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{EtherscanEndpoint, EtherscanEndpoints};
pub use auth::EtherscanAuth;
pub use parser::{
    EtherscanParser, EthBalance, EthTransaction, TokenTransfer,
    EthPrice, GasOracle, BlockReward, EtherscanResponse,
};
pub use connector::EtherscanConnector;
