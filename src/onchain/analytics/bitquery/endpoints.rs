//! # Bitquery GraphQL Queries and Endpoints
//!
//! Bitquery uses GraphQL, not REST. This module provides:
//! - GraphQL query templates
//! - Network and dataset enums
//! - Query builders for common operations

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Bitquery API
#[derive(Debug, Clone)]
pub struct BitqueryUrls {
    pub graphql: &'static str,
    pub websocket: &'static str,
}

impl Default for BitqueryUrls {
    fn default() -> Self {
        Self {
            graphql: "https://streaming.bitquery.io/graphql",
            websocket: "wss://streaming.bitquery.io/graphql",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BLOCKCHAIN NETWORKS
// ═══════════════════════════════════════════════════════════════════════════════

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitqueryNetwork {
    // EVM Chains
    Ethereum,
    Bsc,
    Polygon,
    Arbitrum,
    Base,
    Optimism,
    Avalanche,
    Fantom,
    Cronos,
    Celo,
    Moonbeam,
    Klaytn,

    // Non-EVM Chains
    Solana,
    Bitcoin,
    Litecoin,
    BitcoinCash,
    Cardano,
    Ripple,
    Stellar,
    Algorand,
    Cosmos,
    Tron,
}

impl BitqueryNetwork {
    /// Get network identifier for GraphQL queries
    pub fn as_str(&self) -> &'static str {
        match self {
            // EVM
            Self::Ethereum => "eth",
            Self::Bsc => "bsc",
            Self::Polygon => "polygon",
            Self::Arbitrum => "arbitrum",
            Self::Base => "base",
            Self::Optimism => "optimism",
            Self::Avalanche => "avalanche",
            Self::Fantom => "fantom",
            Self::Cronos => "cronos",
            Self::Celo => "celo",
            Self::Moonbeam => "moonbeam",
            Self::Klaytn => "klaytn",

            // Non-EVM
            Self::Solana => "solana",
            Self::Bitcoin => "bitcoin",
            Self::Litecoin => "litecoin",
            Self::BitcoinCash => "bitcoincash",
            Self::Cardano => "cardano",
            Self::Ripple => "ripple",
            Self::Stellar => "stellar",
            Self::Algorand => "algorand",
            Self::Cosmos => "cosmos",
            Self::Tron => "tron",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATASET TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Dataset type for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitqueryDataset {
    /// Historical data from blockchain genesis
    Archive,
    /// Real-time data (<1s latency)
    Realtime,
    /// Combined archive + realtime
    Combined,
}

impl BitqueryDataset {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Archive => "archive",
            Self::Realtime => "realtime",
            Self::Combined => "combined",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GRAPHQL QUERY ENDPOINTS (Cubes)
// ═══════════════════════════════════════════════════════════════════════════════

/// GraphQL query types (cubes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitqueryEndpoint {
    // Blockchain Infrastructure
    Blocks,
    Transactions,
    MempoolTransactions,

    // Token Data
    Transfers,
    BalanceUpdates,

    // DEX Trading
    DexTrades,

    // NFT Data
    NftTrades,

    // Smart Contracts
    Events,
    Calls,

    // Solana-specific
    SolanaInstructions,

    // Bitcoin-specific
    BitcoinInputs,
    BitcoinOutputs,
}

impl BitqueryEndpoint {
    /// Get cube name for GraphQL query
    pub fn cube_name(&self) -> &'static str {
        match self {
            Self::Blocks => "Blocks",
            Self::Transactions => "Transactions",
            Self::MempoolTransactions => "MempoolTransactions",
            Self::Transfers => "Transfers",
            Self::BalanceUpdates => "BalanceUpdates",
            Self::DexTrades => "DEXTrades",
            Self::NftTrades => "NFTTrades",
            Self::Events => "Events",
            Self::Calls => "Calls",
            Self::SolanaInstructions => "Instructions",
            Self::BitcoinInputs => "Inputs",
            Self::BitcoinOutputs => "Outputs",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GRAPHQL QUERY BUILDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Build GraphQL query for DEX trades
pub fn build_dex_trades_query(
    network: &str,
    dataset: &str,
    protocol: Option<&str>,
    buy_currency: Option<&str>,
    sell_currency: Option<&str>,
    limit: u32,
) -> String {
    let mut filters = Vec::new();

    if let Some(proto) = protocol {
        filters.push(format!("Dex: {{ProtocolName: {{is: \"{}\"}}}}", proto));
    }

    if let Some(buy) = buy_currency {
        filters.push(format!("Buy: {{Currency: {{SmartContract: {{is: \"{}\"}}}}}}", buy));
    }

    if let Some(sell) = sell_currency {
        filters.push(format!("Sell: {{Currency: {{SmartContract: {{is: \"{}\"}}}}}}", sell));
    }

    let where_clause = if filters.is_empty() {
        String::new()
    } else {
        format!("where: {{Trade: {{{}}}}}", filters.join(", "))
    };

    format!(
        r#"{{
  EVM(network: {}, dataset: {}) {{
    DEXTrades(
      {}
      limit: {{count: {}}}
      orderBy: {{descending: Block_Time}}
    ) {{
      Trade {{
        Buy {{
          Amount
          Price
          PriceInUSD
          Currency {{
            Symbol
            Name
            SmartContract
          }}
        }}
        Sell {{
          Amount
          Currency {{
            Symbol
            Name
            SmartContract
          }}
          Seller
        }}
        Dex {{
          ProtocolName
          ProtocolFamily
          SmartContract
        }}
        Index
      }}
      Transaction {{
        Hash
        From
      }}
      Block {{
        Time
        Number
      }}
    }}
  }}
}}"#,
        network, dataset, where_clause, limit
    )
}

/// Build GraphQL query for token transfers
pub fn build_transfers_query(
    network: &str,
    dataset: &str,
    currency_address: Option<&str>,
    sender: Option<&str>,
    receiver: Option<&str>,
    limit: u32,
) -> String {
    let mut filters = Vec::new();

    if let Some(addr) = currency_address {
        filters.push(format!("Currency: {{SmartContract: {{is: \"{}\"}}}}", addr));
    }

    if let Some(from) = sender {
        filters.push(format!("Sender: {{is: \"{}\"}}", from));
    }

    if let Some(to) = receiver {
        filters.push(format!("Receiver: {{is: \"{}\"}}", to));
    }

    let where_clause = if filters.is_empty() {
        String::new()
    } else {
        format!("where: {{Transfer: {{{}}}}}", filters.join(", "))
    };

    format!(
        r#"{{
  EVM(network: {}, dataset: {}) {{
    Transfers(
      {}
      limit: {{count: {}}}
      orderBy: {{descending: Block_Time}}
    ) {{
      Transfer {{
        Amount
        Sender
        Receiver
        Currency {{
          Symbol
          Name
          SmartContract
          Decimals
          Fungible
        }}
        Type
        Id
      }}
      Transaction {{
        Hash
      }}
      Block {{
        Time
        Number
      }}
    }}
  }}
}}"#,
        network, dataset, where_clause, limit
    )
}

/// Build GraphQL query for balance updates
pub fn build_balance_updates_query(
    network: &str,
    dataset: &str,
    address: &str,
    limit: u32,
) -> String {
    format!(
        r#"{{
  EVM(network: {}, dataset: {}) {{
    BalanceUpdates(
      where: {{BalanceUpdate: {{Address: {{is: "{}"}}}}}}
      limit: {{count: {}}}
      orderBy: {{descending: Block_Time}}
    ) {{
      BalanceUpdate {{
        Address
        Amount
        Type
        Currency {{
          Symbol
          Name
          SmartContract
          Decimals
        }}
      }}
      Transaction {{
        Hash
      }}
      Block {{
        Time
        Number
      }}
    }}
  }}
}}"#,
        network, dataset, address, limit
    )
}

/// Build GraphQL query for blocks
pub fn build_blocks_query(
    network: &str,
    dataset: &str,
    limit: u32,
) -> String {
    format!(
        r#"{{
  EVM(network: {}, dataset: {}) {{
    Blocks(
      limit: {{count: {}}}
      orderBy: {{descending: Block_Number}}
    ) {{
      Block {{
        Number
        Time
        Hash
        GasLimit
        GasUsed
        BaseFee
        Coinbase
        Difficulty
        Size
        TxCount
      }}
    }}
  }}
}}"#,
        network, dataset, limit
    )
}

/// Build GraphQL query for transactions
pub fn build_transactions_query(
    network: &str,
    dataset: &str,
    tx_hash: Option<&str>,
    from_address: Option<&str>,
    to_address: Option<&str>,
    limit: u32,
) -> String {
    let mut filters = Vec::new();

    if let Some(hash) = tx_hash {
        filters.push(format!("Hash: {{is: \"{}\"}}", hash));
    }

    if let Some(from) = from_address {
        filters.push(format!("From: {{is: \"{}\"}}", from));
    }

    if let Some(to) = to_address {
        filters.push(format!("To: {{is: \"{}\"}}", to));
    }

    let where_clause = if filters.is_empty() {
        String::new()
    } else {
        format!("where: {{Transaction: {{{}}}}}", filters.join(", "))
    };

    format!(
        r#"{{
  EVM(network: {}, dataset: {}) {{
    Transactions(
      {}
      limit: {{count: {}}}
      orderBy: {{descending: Block_Time}}
    ) {{
      Transaction {{
        Hash
        From
        To
        Value
        Gas
        GasPrice
        GasUsed
        Nonce
        Index
        Type
        Cost
      }}
      Receipt {{
        Status
        GasUsed
        EffectiveGasPrice
      }}
      Block {{
        Time
        Number
      }}
    }}
  }}
}}"#,
        network, dataset, where_clause, limit
    )
}

/// Build GraphQL query for smart contract events
pub fn build_events_query(
    network: &str,
    dataset: &str,
    contract_address: &str,
    event_name: Option<&str>,
    limit: u32,
) -> String {
    let event_filter = if let Some(name) = event_name {
        format!("Signature: {{Name: {{is: \"{}\"}}}}, SmartContract: {{is: \"{}\"}}", name, contract_address)
    } else {
        format!("SmartContract: {{is: \"{}\"}}", contract_address)
    };

    format!(
        r#"{{
  EVM(network: {}, dataset: {}) {{
    Events(
      where: {{Log: {{{}}}}}
      limit: {{count: {}}}
      orderBy: {{descending: Block_Time}}
    ) {{
      Log {{
        Signature
        SignatureName
        SmartContract
      }}
      Arguments {{
        Name
        Type
        Value
      }}
      Transaction {{
        Hash
      }}
      Block {{
        Time
        Number
      }}
    }}
  }}
}}"#,
        network, dataset, event_filter, limit
    )
}

/// Build GraphQL subscription for real-time blocks
pub fn _build_blocks_subscription(network: &str) -> String {
    format!(
        r#"subscription {{
  EVM(network: {}, dataset: realtime) {{
    Blocks {{
      Block {{
        Number
        Time
        Hash
        TxCount
      }}
    }}
  }}
}}"#,
        network
    )
}

/// Build GraphQL subscription for real-time DEX trades
pub fn _build_dex_trades_subscription(
    network: &str,
    protocol: Option<&str>,
) -> String {
    let where_clause = if let Some(proto) = protocol {
        format!("where: {{Trade: {{Dex: {{ProtocolName: {{is: \"{}\"}}}}}}}}", proto)
    } else {
        String::new()
    };

    format!(
        r#"subscription {{
  EVM(network: {}, dataset: realtime) {{
    DEXTrades({}) {{
      Trade {{
        Buy {{
          Amount
          Price
          Currency {{
            Symbol
          }}
        }}
        Sell {{
          Amount
          Currency {{
            Symbol
          }}
        }}
        Dex {{
          ProtocolName
        }}
      }}
      Block {{
        Time
      }}
    }}
  }}
}}"#,
        network, where_clause
    )
}
