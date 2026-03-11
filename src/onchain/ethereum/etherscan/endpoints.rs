//! Etherscan API endpoints

/// Base URLs for Etherscan API
pub struct EtherscanEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
    pub testnet: bool,
}

impl Default for EtherscanEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.etherscan.io/api",
            ws_base: None, // Etherscan does not support WebSocket
            testnet: false,
        }
    }
}

impl EtherscanEndpoints {
    /// Create endpoints for testnet (Sepolia)
    pub fn testnet() -> Self {
        Self {
            rest_base: "https://api-sepolia.etherscan.io/api",
            ws_base: None,
            testnet: true,
        }
    }

    /// Create endpoints for custom network
    pub fn custom(base_url: &'static str) -> Self {
        Self {
            rest_base: base_url,
            ws_base: None,
            testnet: false,
        }
    }
}

/// Etherscan API endpoint enum
#[derive(Debug, Clone)]
pub enum EtherscanEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // ACCOUNT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get ETH balance for a single address
    Balance,
    /// Get ETH balance for multiple addresses
    BalanceMulti,
    /// Get list of normal transactions
    TxList,
    /// Get list of ERC20 token transfer events
    TokenTx,
    /// Get list of internal transactions
    TxListInternal,

    // ═══════════════════════════════════════════════════════════════════════
    // STATS ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get total ETH supply
    EthSupply,
    /// Get current ETH price (USD + BTC)
    EthPrice,
    /// Get total size of the blockchain
    ChainSize,
    /// Get total supply of an ERC20 token
    TokenSupply,

    // ═══════════════════════════════════════════════════════════════════════
    // PROXY ENDPOINTS (Ethereum JSON-RPC)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get latest block number
    EthBlockNumber,
    /// Get block by number
    EthGetBlockByNumber,

    // ═══════════════════════════════════════════════════════════════════════
    // BLOCK ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get block mining reward
    BlockReward,

    // ═══════════════════════════════════════════════════════════════════════
    // GAS TRACKER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get gas price oracle
    GasOracle,

    // ═══════════════════════════════════════════════════════════════════════
    // CONTRACT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get contract ABI
    GetAbi,
}

impl EtherscanEndpoint {
    /// Get endpoint path (always "/api" - Etherscan uses query params for routing)
    pub fn path(&self) -> &'static str {
        "/api"
    }

    /// Get module and action parameters for this endpoint
    pub fn params(&self) -> (&'static str, &'static str) {
        match self {
            // Account module
            Self::Balance => ("account", "balance"),
            Self::BalanceMulti => ("account", "balancemulti"),
            Self::TxList => ("account", "txlist"),
            Self::TokenTx => ("account", "tokentx"),
            Self::TxListInternal => ("account", "txlistinternal"),

            // Stats module
            Self::EthSupply => ("stats", "ethsupply"),
            Self::EthPrice => ("stats", "ethprice"),
            Self::ChainSize => ("stats", "chainsize"),
            Self::TokenSupply => ("stats", "tokensupply"),

            // Proxy module (JSON-RPC)
            Self::EthBlockNumber => ("proxy", "eth_blockNumber"),
            Self::EthGetBlockByNumber => ("proxy", "eth_getBlockByNumber"),

            // Block module
            Self::BlockReward => ("block", "getblockreward"),

            // Gas tracker module
            Self::GasOracle => ("gastracker", "gasoracle"),

            // Contract module
            Self::GetAbi => ("contract", "getabi"),
        }
    }
}

/// Format Ethereum address for Etherscan API
///
/// Etherscan expects addresses in format: 0x...
/// Addresses should be 42 characters (0x + 40 hex chars)
pub fn format_address(address: &str) -> String {
    if address.starts_with("0x") {
        address.to_lowercase()
    } else {
        format!("0x{}", address.to_lowercase())
    }
}

/// Validate Ethereum address format
pub fn _validate_address(address: &str) -> bool {
    if !address.starts_with("0x") {
        return false;
    }
    if address.len() != 42 {
        return false;
    }
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}
