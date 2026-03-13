//! # Uniswap Endpoints
//!
//! URL'ы и endpoint enum для Uniswap APIs.
//!
//! Uniswap использует несколько API:
//! - Trading API (REST) - quotes, swaps, order status
//! - The Graph Subgraph (GraphQL) - historical data, analytics
//! - Ethereum RPC (JSON-RPC) - on-chain data, contract calls
//! - Ethereum WebSocket - real-time events

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Uniswap API
#[derive(Debug, Clone)]
pub struct UniswapUrls {
    /// Trading API (REST)
    pub trading_api: &'static str,
    /// The Graph Subgraph (GraphQL) - V3
    pub subgraph_v3: &'static str,
    /// Ethereum RPC (JSON-RPC)
    pub eth_rpc: &'static str,
    /// Ethereum WebSocket
    pub eth_ws: &'static str,
    /// Chain ID
    pub chain_id: u64,
}

impl UniswapUrls {
    /// Ethereum Mainnet URLs
    pub const MAINNET: Self = Self {
        trading_api: "https://trade-api.gateway.uniswap.org/v1",
        subgraph_v3: "https://gateway.thegraph.com/api/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV",
        eth_rpc: "https://ethereum-rpc.publicnode.com", // Free public RPC (no API key needed)
        eth_ws: "wss://ethereum-rpc.publicnode.com", // Free public WebSocket
        chain_id: 1,
    };

    /// Ethereum Sepolia Testnet URLs
    pub const TESTNET: Self = Self {
        trading_api: "https://beta.trade-api.gateway.uniswap.org/v1",
        subgraph_v3: "https://api.studio.thegraph.com/query/24660/uniswap-v3-sepolia/version/latest",
        eth_rpc: "https://ethereum-sepolia-rpc.publicnode.com",
        eth_ws: "wss://ethereum-sepolia-rpc.publicnode.com",
        chain_id: 11155111,
    };

    /// Get base URL for specific API
    pub fn api_url(&self, endpoint: UniswapEndpoint) -> &str {
        match endpoint {
            UniswapEndpoint::Quote
            | UniswapEndpoint::Swap
            | UniswapEndpoint::CheckApproval
            | UniswapEndpoint::OrderStatus
            | UniswapEndpoint::SwapStatus
            | UniswapEndpoint::SwappableTokens => self.trading_api,

            UniswapEndpoint::PoolsQuery
            | UniswapEndpoint::SwapsQuery
            | UniswapEndpoint::TokensQuery
            | UniswapEndpoint::PositionsQuery
            | UniswapEndpoint::FactoryQuery => self.subgraph_v3,

            UniswapEndpoint::EthCall
            | UniswapEndpoint::EthGetBalance
            | UniswapEndpoint::EthGetTransactionReceipt
            | UniswapEndpoint::EthBlockNumber => self.eth_rpc,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Uniswap API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniswapEndpoint {
    // === TRADING API (REST) ===
    /// POST /quote - Get swap quote
    Quote,
    /// POST /swap - Execute swap (get transaction data)
    Swap,
    /// POST /check_approval - Check token approval
    CheckApproval,
    /// GET /orders - Get order status (UniswapX)
    OrderStatus,
    /// GET /swaps - Get swap status
    SwapStatus,
    /// GET /swappable_tokens - List available tokens
    SwappableTokens,

    // === THE GRAPH SUBGRAPH (GraphQL) ===
    /// Query pools
    PoolsQuery,
    /// Query swaps
    SwapsQuery,
    /// Query tokens
    TokensQuery,
    /// Query user positions
    PositionsQuery,
    /// Query factory stats
    FactoryQuery,

    // === ETHEREUM RPC (JSON-RPC) ===
    /// eth_call - Call contract method
    EthCall,
    /// eth_getBalance - Get token balance
    EthGetBalance,
    /// eth_getTransactionReceipt - Get transaction receipt
    EthGetTransactionReceipt,
    /// eth_blockNumber - Get latest block number
    EthBlockNumber,
}

impl UniswapEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Trading API
            Self::Quote => "/quote",
            Self::Swap => "/swap",
            Self::CheckApproval => "/approval",
            Self::OrderStatus => "/orders",
            Self::SwapStatus => "/swaps",
            Self::SwappableTokens => "/tokens",

            // Subgraph - all use same GraphQL endpoint
            Self::PoolsQuery
            | Self::SwapsQuery
            | Self::TokensQuery
            | Self::PositionsQuery
            | Self::FactoryQuery => "/graphql",

            // Ethereum RPC - all use same JSON-RPC endpoint
            Self::EthCall
            | Self::EthGetBalance
            | Self::EthGetTransactionReceipt
            | Self::EthBlockNumber => "/",
        }
    }

    /// HTTP method
    pub fn method(&self) -> &'static str {
        match self {
            Self::Quote | Self::Swap | Self::CheckApproval => "POST",

            Self::PoolsQuery
            | Self::SwapsQuery
            | Self::TokensQuery
            | Self::PositionsQuery
            | Self::FactoryQuery
            | Self::EthCall
            | Self::EthGetBalance
            | Self::EthGetTransactionReceipt
            | Self::EthBlockNumber => "POST", // GraphQL and JSON-RPC use POST

            _ => "GET",
        }
    }

    /// Requires authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // Trading API requires API key
            Self::Quote | Self::Swap | Self::CheckApproval => true,

            // Public endpoints
            _ => false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOKEN ADDRESS CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Common token addresses on Ethereum Mainnet
pub mod tokens {
    /// WETH (Wrapped Ether)
    pub const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    /// USDC (USD Coin)
    pub const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    /// USDT (Tether USD)
    pub const USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    /// DAI (Dai Stablecoin)
    pub const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    /// WBTC (Wrapped Bitcoin)
    pub const WBTC: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
    /// UNI (Uniswap)
    pub const UNI: &str = "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984";
}

/// Uniswap V3 contract addresses on Ethereum Mainnet
pub mod contracts {
    /// V3 Factory
    pub const V3_FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
    /// V3 Router
    pub const V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
    /// V3 Quoter
    pub const V3_QUOTER: &str = "0xb27308f9F90D607463BB33eA1BeBb41C27CE5AB6";
    /// V3 Position Manager
    pub const V3_POSITION_MANAGER: &str = "0xC36442b4a4522E871399CD717aBDD847Ab11FE88";
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format token address from symbol
///
/// Uniswap uses token contract addresses instead of symbols.
/// This function maps common symbols to their addresses.
///
/// # Note
/// For unknown symbols, returns the symbol as-is (assumes it's already an address).
pub fn format_token_address(symbol: &str, _account_type: AccountType) -> String {
    match symbol.to_uppercase().as_str() {
        "WETH" | "ETH" => tokens::WETH.to_string(),
        "USDC" => tokens::USDC.to_string(),
        "USDT" => tokens::USDT.to_string(),
        "DAI" => tokens::DAI.to_string(),
        "WBTC" | "BTC" => tokens::WBTC.to_string(),
        "UNI" => tokens::UNI.to_string(),
        // If starts with 0x, assume it's already an address
        s if s.starts_with("0X") => s.to_string(),
        // Otherwise return as-is (user should provide address)
        s => s.to_string(),
    }
}

/// Get symbol from token address
pub fn _get_symbol_from_address(address: &str) -> &str {
    match address.to_lowercase().as_str() {
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" => "WETH",
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" => "USDC",
        "0xdac17f958d2ee523a2206206994597c13d831ec7" => "USDT",
        "0x6b175474e89094c44da98b954eedeac495271d0f" => "DAI",
        "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599" => "WBTC",
        "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984" => "UNI",
        _ => address,
    }
}

/// Fee tiers for Uniswap V3 pools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeTier {
    /// 0.01% - Stablecoins
    Lowest = 100,
    /// 0.05% - Low volatility
    Low = 500,
    /// 0.30% - Standard
    Medium = 3000,
    /// 1.00% - Exotic pairs
    High = 10000,
}

impl FeeTier {
    /// Get fee tier from basis points
    pub fn from_bps(bps: u32) -> Option<Self> {
        match bps {
            100 => Some(Self::Lowest),
            500 => Some(Self::Low),
            3000 => Some(Self::Medium),
            10000 => Some(Self::High),
            _ => None,
        }
    }

    /// Get fee as percentage
    pub fn as_percentage(&self) -> f64 {
        (*self as u32) as f64 / 10000.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POOL REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Pool metadata for RPC-based price queries
#[derive(Debug, Clone)]
pub struct PoolMetadata {
    /// Pool contract address
    pub address: &'static str,
    /// Token0 symbol
    pub token0_symbol: &'static str,
    /// Token0 decimals
    pub token0_decimals: u8,
    /// Token1 symbol
    pub token1_symbol: &'static str,
    /// Token1 decimals
    pub token1_decimals: u8,
    /// Fee tier in basis points
    pub fee_tier: u32,
}

/// Known Uniswap V3 pools on Ethereum Mainnet
pub const KNOWN_POOLS: &[PoolMetadata] = &[
    // ETH/USDC 0.05% - Most liquid ETH pool
    PoolMetadata {
        address: "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640",
        token0_symbol: "USDC",
        token0_decimals: 6,
        token1_symbol: "WETH",
        token1_decimals: 18,
        fee_tier: 500,
    },
    // ETH/USDT 0.05%
    PoolMetadata {
        address: "0x11b815efB8f581194ae79006d24E0d814B7697F6",
        token0_symbol: "WETH",
        token0_decimals: 18,
        token1_symbol: "USDT",
        token1_decimals: 6,
        fee_tier: 500,
    },
    // WBTC/ETH 0.3%
    PoolMetadata {
        address: "0xCBCdF9626bC03E24f779434178A73a0B4bad62eD",
        token0_symbol: "WBTC",
        token0_decimals: 8,
        token1_symbol: "WETH",
        token1_decimals: 18,
        fee_tier: 3000,
    },
    // ETH/USDC 0.3% - Alternative pool
    PoolMetadata {
        address: "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8",
        token0_symbol: "USDC",
        token0_decimals: 6,
        token1_symbol: "WETH",
        token1_decimals: 18,
        fee_tier: 3000,
    },
    // ETH/USDT 0.3% - Alternative pool
    PoolMetadata {
        address: "0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36",
        token0_symbol: "WETH",
        token0_decimals: 18,
        token1_symbol: "USDT",
        token1_decimals: 6,
        fee_tier: 3000,
    },
];

/// Find pool metadata by symbol pair
///
/// Returns the pool with lowest fee tier (most liquid) for the given pair.
pub fn find_pool_metadata(base: &str, quote: &str) -> Option<&'static PoolMetadata> {
    // Normalize symbols (ETH -> WETH, BTC -> WBTC)
    let base = match base {
        "ETH" => "WETH",
        "BTC" => "WBTC",
        s => s,
    };
    let quote = match quote {
        "ETH" => "WETH",
        "BTC" => "WBTC",
        s => s,
    };

    // Find pools matching the pair (in either direction)
    let mut matches: Vec<_> = KNOWN_POOLS
        .iter()
        .filter(|pool| {
            (pool.token0_symbol == base && pool.token1_symbol == quote)
                || (pool.token0_symbol == quote && pool.token1_symbol == base)
        })
        .collect();

    // Sort by fee tier (lowest first = most liquid)
    matches.sort_by_key(|pool| pool.fee_tier);

    matches.first().copied()
}

/// Find pool metadata by address
pub fn find_pool_by_address(address: &str) -> Option<&'static PoolMetadata> {
    let address_lower = address.to_lowercase();
    KNOWN_POOLS
        .iter()
        .find(|pool| pool.address.to_lowercase() == address_lower)
}
