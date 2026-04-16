//! # EvmProvider — alloy-backed EVM chain provider
//!
//! Implements [`ChainProvider`] and [`EvmChain`] for all EVM-compatible networks.
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-evm` feature. Enable it in
//! your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-evm"] }
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{EvmProvider, EvmChain};
//!
//! let provider = EvmProvider::arbitrum();
//! let height = provider.get_height().await?;
//! let balance = provider.erc20_balance(token_addr, owner_addr).await?;
//! ```

use alloy::network::Ethereum;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::rpc::types::eth::{TransactionReceipt, TransactionRequest};

use async_trait::async_trait;

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// ERC-20 SELECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// `balanceOf(address)` function selector — keccak256("balanceOf(address)")[0..4]
const BALANCE_OF_SELECTOR: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];

// ═══════════════════════════════════════════════════════════════════════════════
// EVM CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// EVM-specific chain operations.
///
/// Extends [`ChainProvider`] with the full EVM JSON-RPC surface needed by
/// DeFi connectors: contract reads (`eth_call`), gas estimation, gas price
/// queries, receipt retrieval, and an ERC-20 balance convenience helper.
///
/// All EVM chains — Ethereum, Arbitrum, Optimism, Base, Polygon, BSC,
/// Avalanche — share the same implementation. The chain is selected by
/// the RPC URL and chain ID passed to [`EvmProvider::new`].
///
/// ## Object safety
///
/// This trait is **not** object-safe because it uses alloy SDK types in method
/// signatures. Store the concrete [`EvmProvider`] type in connector fields
/// rather than `Box<dyn EvmChain>`.
#[async_trait]
pub trait EvmChain: ChainProvider {
    /// Execute a read-only contract call (`eth_call`).
    ///
    /// `call` must have at minimum the `to` and `input` fields populated.
    /// `from` and `value` are optional.
    ///
    /// Returns the raw return bytes from the contract. The caller is
    /// responsible for decoding them according to the function's ABI.
    async fn eth_call(&self, call: TransactionRequest) -> Result<Bytes, ExchangeError>;

    /// Estimate gas for a transaction (`eth_estimateGas`).
    ///
    /// Returns the estimated gas units as `u64`. Add a safety buffer
    /// (e.g. multiply by 1.2) before setting `gas_limit` on the real tx.
    async fn estimate_gas(&self, call: TransactionRequest) -> Result<u64, ExchangeError>;

    /// Get the current base fee per gas in wei.
    ///
    /// Uses `eth_gasPrice` under the hood, which returns the effective gas price
    /// (base fee + priority fee on EIP-1559 chains) or the legacy gas price.
    async fn gas_price(&self) -> Result<U256, ExchangeError>;

    /// Get the suggested EIP-1559 max priority fee per gas (the "tip") in wei.
    ///
    /// Calls `eth_maxPriorityFeePerGas`. Add this to the base fee to get the
    /// `max_fee_per_gas` for EIP-1559 transactions.
    async fn max_priority_fee(&self) -> Result<U256, ExchangeError>;

    /// Get the transaction receipt once the transaction is confirmed.
    ///
    /// Returns `Ok(None)` if the transaction is pending or not found.
    /// Returns `Ok(Some(receipt))` once the transaction is mined.
    async fn get_receipt(&self, tx_hash: &str) -> Result<Option<TransactionReceipt>, ExchangeError>;

    /// Read the ERC-20 `balanceOf(address)` for `token` contract.
    ///
    /// Convenience wrapper over `eth_call` for the most common on-chain
    /// read in DeFi connectors. Returns the raw balance as `U256` in the
    /// token's smallest unit (divide by `10^decimals` for display).
    async fn erc20_balance(&self, token: Address, account: Address) -> Result<U256, ExchangeError>;

    /// Access the underlying alloy [`DynProvider`] for advanced operations.
    ///
    /// Prefer the typed methods above when possible. This escape hatch is
    /// provided for operations not yet covered by the trait surface.
    fn inner(&self) -> &DynProvider<Ethereum>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVM PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete EVM chain provider backed by alloy's type-erased HTTP provider.
///
/// One `EvmProvider` per RPC endpoint is sufficient. Multiple DeFi connectors
/// targeting the same chain can share a single `EvmProvider` instance via
/// `Arc<EvmProvider>`, reusing the same HTTP connection pool and avoiding
/// duplicate RPC calls.
///
/// ## Construction
///
/// Use the chain-specific convenience constructors for known networks:
///
/// ```rust,ignore
/// let arb  = EvmProvider::arbitrum();
/// let eth  = EvmProvider::ethereum("https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY");
/// ```
///
/// Or construct from a custom RPC URL:
///
/// ```rust,ignore
/// let provider = EvmProvider::connect_http("https://my-rpc.example.com", 42161, "arbitrum").await?;
/// ```
pub struct EvmProvider {
    /// Type-erased alloy HTTP provider
    provider: DynProvider<Ethereum>,
    /// EIP-155 numeric chain ID
    chain_id: u64,
    /// Human-readable chain name (e.g. "arbitrum", "ethereum")
    chain_name: String,
}

impl EvmProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Wrap an existing alloy [`DynProvider`].
    ///
    /// Prefer [`connect_http`][Self::connect_http] unless you already have
    /// a provider instance.
    pub fn new(provider: DynProvider<Ethereum>, chain_id: u64, chain_name: &str) -> Self {
        Self {
            provider,
            chain_id,
            chain_name: chain_name.to_lowercase(),
        }
    }

    /// Connect via an HTTP RPC URL and return a ready `EvmProvider`.
    ///
    /// Parses `rpc_url`, builds an alloy `ProviderBuilder` HTTP transport,
    /// and wraps it in a type-erased `DynProvider`.
    pub async fn connect_http(
        rpc_url: &str,
        chain_id: u64,
        chain_name: &str,
    ) -> Result<Self, ExchangeError> {
        let url: reqwest::Url = rpc_url.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid RPC URL '{}': {}", rpc_url, e))
        })?;
        let provider = ProviderBuilder::new().connect_http(url);
        Ok(Self::new(DynProvider::new(provider), chain_id, chain_name))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Known-chain convenience constructors (sync — no async needed for HTTP)
    // ─────────────────────────────────────────────────────────────────────────

    /// Ethereum mainnet using the public PublicNode RPC.
    pub fn ethereum(rpc_url: &str) -> Result<Self, ExchangeError> {
        let url: reqwest::Url = rpc_url.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid Ethereum RPC URL '{}': {}", rpc_url, e))
        })?;
        let provider = ProviderBuilder::new().connect_http(url);
        Ok(Self::new(DynProvider::new(provider), 1, "ethereum"))
    }

    /// Arbitrum One using the public Offchain Labs RPC.
    pub fn arbitrum() -> Self {
        let url: reqwest::Url = "https://arb1.arbitrum.io/rpc".parse().expect("static URL");
        let provider = ProviderBuilder::new().connect_http(url);
        Self::new(DynProvider::new(provider), 42161, "arbitrum")
    }

    /// Base mainnet using the public Coinbase RPC.
    pub fn base() -> Self {
        let url: reqwest::Url = "https://mainnet.base.org".parse().expect("static URL");
        let provider = ProviderBuilder::new().connect_http(url);
        Self::new(DynProvider::new(provider), 8453, "base")
    }

    /// Polygon PoS mainnet using the public Polygon RPC.
    pub fn polygon() -> Self {
        let url: reqwest::Url = "https://polygon-rpc.com".parse().expect("static URL");
        let provider = ProviderBuilder::new().connect_http(url);
        Self::new(DynProvider::new(provider), 137, "polygon")
    }

    /// Avalanche C-Chain using the public Ava Labs RPC.
    pub fn avalanche() -> Self {
        let url: reqwest::Url = "https://api.avax.network/ext/bc/C/rpc"
            .parse()
            .expect("static URL");
        let provider = ProviderBuilder::new().connect_http(url);
        Self::new(DynProvider::new(provider), 43114, "avalanche")
    }

    /// BNB Smart Chain mainnet using the public BSC RPC.
    pub fn bsc() -> Self {
        let url: reqwest::Url = "https://bsc-dataseed.binance.org".parse().expect("static URL");
        let provider = ProviderBuilder::new().connect_http(url);
        Self::new(DynProvider::new(provider), 56, "bsc")
    }

    /// Optimism mainnet using the public OP Labs RPC.
    pub fn optimism() -> Self {
        let url: reqwest::Url = "https://mainnet.optimism.io".parse().expect("static URL");
        let provider = ProviderBuilder::new().connect_http(url);
        Self::new(DynProvider::new(provider), 10, "optimism")
    }

    /// Human-readable chain name (e.g. `"ethereum"`, `"arbitrum"`).
    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for EvmProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Evm { chain_id: self.chain_id }
    }

    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        let pending = self
            .provider
            .send_raw_transaction(tx_bytes)
            .await
            .map_err(|e| ExchangeError::Network(format!("send_raw_transaction failed: {}", e)))?;

        Ok(format!("{:#x}", pending.tx_hash()))
    }

    async fn get_height(&self) -> Result<u64, ExchangeError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_blockNumber failed: {}", e)))
    }

    async fn get_nonce(&self, address: &str) -> Result<u64, ExchangeError> {
        let addr: Address = address.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid address '{}': {}", address, e))
        })?;
        let count = self
            .provider
            .get_transaction_count(addr)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getTransactionCount failed: {}", e)))?;
        Ok(count)
    }

    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        let addr: Address = address.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid address '{}': {}", address, e))
        })?;
        let balance = self
            .provider
            .get_balance(addr)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getBalance failed: {}", e)))?;
        Ok(balance.to_string())
    }

    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        let hash: alloy::primitives::TxHash = tx_hash.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid tx hash '{}': {}", tx_hash, e))
        })?;

        let receipt = self
            .provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getTransactionReceipt failed: {}", e)))?;

        match receipt {
            None => Ok(TxStatus::Pending),
            Some(r) => {
                let block = r.block_number.unwrap_or(0);
                if r.status() {
                    Ok(TxStatus::Confirmed { block })
                } else {
                    Ok(TxStatus::Failed {
                        reason: "transaction reverted".to_string(),
                    })
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvmChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl EvmChain for EvmProvider {
    async fn eth_call(&self, call: TransactionRequest) -> Result<Bytes, ExchangeError> {
        self.provider
            .call(call)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_call failed: {}", e)))
    }

    async fn estimate_gas(&self, call: TransactionRequest) -> Result<u64, ExchangeError> {
        self.provider
            .estimate_gas(call)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_estimateGas failed: {}", e)))
    }

    async fn gas_price(&self) -> Result<U256, ExchangeError> {
        let price_u128 = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_gasPrice failed: {}", e)))?;
        Ok(U256::from(price_u128))
    }

    async fn max_priority_fee(&self) -> Result<U256, ExchangeError> {
        let fee_u128 = self
            .provider
            .get_max_priority_fee_per_gas()
            .await
            .map_err(|e| {
                ExchangeError::Network(format!("eth_maxPriorityFeePerGas failed: {}", e))
            })?;
        Ok(U256::from(fee_u128))
    }

    async fn get_receipt(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionReceipt>, ExchangeError> {
        let hash: alloy::primitives::TxHash = tx_hash.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!("Invalid tx hash '{}': {}", tx_hash, e))
        })?;
        self.provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| {
                ExchangeError::Network(format!("eth_getTransactionReceipt failed: {}", e))
            })
    }

    async fn erc20_balance(
        &self,
        token: Address,
        account: Address,
    ) -> Result<U256, ExchangeError> {
        // ABI-encode: balanceOf(address) = selector (4 bytes) + padded address (32 bytes)
        let mut calldata = Vec::with_capacity(4 + 32);
        calldata.extend_from_slice(&BALANCE_OF_SELECTOR);
        // Left-zero-pad the 20-byte address to 32 bytes
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(account.as_slice());

        let tx = TransactionRequest::default()
            .to(token)
            .input(Bytes::from(calldata).into());

        let result = self
            .provider
            .call(tx)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_call (balanceOf) failed: {}", e)))?;

        if result.len() < 32 {
            return Err(ExchangeError::Parse(format!(
                "balanceOf returned {} bytes, expected at least 32",
                result.len()
            )));
        }

        Ok(U256::from_be_slice(&result[..32]))
    }

    fn inner(&self) -> &DynProvider<Ethereum> {
        &self.provider
    }
}

