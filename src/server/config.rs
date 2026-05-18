//! Server configuration — loaded from TOML.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::types::{AccountType, ExchangeId};

/// One exchange entry in the server config.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeEntry {
    /// Exchange identifier (e.g. "Binance", "OKX").
    pub id: ExchangeId,
    /// Account types to subscribe (e.g. ["Spot", "FuturesCross"]).
    #[serde(default = "default_account_types")]
    pub account_types: Vec<AccountType>,
    /// Use testnet endpoints.
    #[serde(default)]
    pub testnet: bool,
}

fn default_account_types() -> Vec<AccountType> {
    vec![AccountType::Spot]
}

/// Top-level server configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// gRPC bind address. Default: 127.0.0.1:18260
    #[serde(default = "default_grpc_addr")]
    pub grpc_addr: String,

    /// Root directory for storage. Default: ./dig3_storage
    #[serde(default = "default_storage_root")]
    pub storage_root: PathBuf,

    /// Exchanges to connect on startup.
    #[serde(default)]
    pub exchanges: Vec<ExchangeEntry>,
}

fn default_grpc_addr() -> String {
    "127.0.0.1:18260".to_string()
}

fn default_storage_root() -> PathBuf {
    PathBuf::from("dig3_storage")
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            grpc_addr: default_grpc_addr(),
            storage_root: default_storage_root(),
            exchanges: Vec::new(),
        }
    }
}

impl ServerConfig {
    /// Load from a TOML file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&content)?;
        Ok(cfg)
    }
}
