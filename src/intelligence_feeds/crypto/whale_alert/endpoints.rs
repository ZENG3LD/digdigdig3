//! Whale Alert API endpoints

/// Base URLs for Whale Alert API
pub struct WhaleAlertEndpoints {
    pub rest_base: &'static str,
    pub rest_base_v1: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for WhaleAlertEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://leviathan.whale-alert.io",  // Enterprise API v2
            rest_base_v1: "https://api.whale-alert.io/v1",   // Developer API v1 (deprecated)
            ws_base: Some("wss://leviathan.whale-alert.io/ws"), // WebSocket alerts
        }
    }
}

/// API endpoint enum
#[derive(Debug, Clone)]
pub enum WhaleAlertEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // Enterprise API v2 (Quantitative/Historical)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get supported blockchains and currencies
    Status,

    /// Get block height range for specific blockchain
    BlockchainStatus { blockchain: String },

    /// Get single transaction by hash
    Transaction { blockchain: String, hash: String },

    /// Stream transactions from start height
    Transactions { blockchain: String },

    /// Get complete block data
    Block { blockchain: String, height: u64 },

    /// Get transaction history for address
    AddressTransactions { blockchain: String, address: String },

    /// Get owner attribution for address
    AddressAttributions { blockchain: String, address: String },

    // ═══════════════════════════════════════════════════════════════════════
    // Developer API v1 (Deprecated but functional)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get status (v1)
    StatusV1,

    /// Get single transaction (v1)
    TransactionV1 { blockchain: String, hash: String },

    /// Get multiple transactions with filters (v1)
    TransactionsV1,
}

impl WhaleAlertEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Enterprise API v2
            Self::Status => "/status".to_string(),
            Self::BlockchainStatus { blockchain } => format!("/{}/status", blockchain),
            Self::Transaction { blockchain, hash } => format!("/{}/transaction/{}", blockchain, hash),
            Self::Transactions { blockchain } => format!("/{}/transactions", blockchain),
            Self::Block { blockchain, height } => format!("/{}/block/{}", blockchain, height),
            Self::AddressTransactions { blockchain, address } => {
                format!("/{}/address/{}/transactions", blockchain, address)
            }
            Self::AddressAttributions { blockchain, address } => {
                format!("/{}/address/{}/owner_attributions", blockchain, address)
            }

            // Developer API v1
            Self::StatusV1 => "/status".to_string(),
            Self::TransactionV1 { blockchain, hash } => {
                format!("/transaction/{}/{}", blockchain, hash)
            }
            Self::TransactionsV1 => "/transactions".to_string(),
        }
    }

    /// Check if this is a v1 endpoint
    pub fn is_v1(&self) -> bool {
        matches!(
            self,
            Self::StatusV1 | Self::TransactionV1 { .. } | Self::TransactionsV1
        )
    }
}

/// Supported blockchains
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blockchain {
    Bitcoin,
    Ethereum,
    Algorand,
    BitcoinCash,
    Dogecoin,
    Litecoin,
    Polygon,
    Solana,
    Ripple,
    Cardano,
    Tron,
}

impl Blockchain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bitcoin => "bitcoin",
            Self::Ethereum => "ethereum",
            Self::Algorand => "algorand",
            Self::BitcoinCash => "bitcoincash",
            Self::Dogecoin => "dogecoin",
            Self::Litecoin => "litecoin",
            Self::Polygon => "polygon",
            Self::Solana => "solana",
            Self::Ripple => "ripple",
            Self::Cardano => "cardano",
            Self::Tron => "tron",
        }
    }
}

impl From<Blockchain> for String {
    fn from(b: Blockchain) -> String {
        b.as_str().to_string()
    }
}

/// Transaction types supported by Whale Alert
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Transfer,
    Mint,
    Burn,
    Freeze,
    Unfreeze,
    Lock,
    Unlock,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transfer => "transfer",
            Self::Mint => "mint",
            Self::Burn => "burn",
            Self::Freeze => "freeze",
            Self::Unfreeze => "unfreeze",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
        }
    }
}

/// Format symbol for Whale Alert API
///
/// Whale Alert uses blockchain-specific addressing, not traditional symbols.
/// This function is included for trait compatibility but symbols are not
/// the primary way to interact with Whale Alert (use blockchain + hash instead).
pub fn _format_symbol(symbol: &crate::core::types::Symbol) -> String {
    // For crypto data feeds, typically just concatenate
    // But Whale Alert doesn't use symbols in the traditional sense
    format!("{}{}", symbol.base, symbol.quote)
}

/// Parse symbol from API format back to domain Symbol
pub fn _parse_symbol(api_symbol: &str) -> Option<crate::core::types::Symbol> {
    // Whale Alert doesn't return symbols in a standard format
    // This is primarily for compatibility
    if api_symbol.len() < 2 {
        return None;
    }

    // Try to parse common crypto pairs
    if let Some(base) = api_symbol.strip_suffix("USDT") {
        return Some(crate::core::types::Symbol::new(base, "USDT"));
    }
    if let Some(base) = api_symbol.strip_suffix("USDC") {
        return Some(crate::core::types::Symbol::new(base, "USDC"));
    }
    if let Some(base) = api_symbol.strip_suffix("USD") {
        return Some(crate::core::types::Symbol::new(base, "USD"));
    }

    None
}
