//! # SolanaProvider — solana-client-backed Solana chain provider
//!
//! Implements [`ChainProvider`] and [`SolanaChain`] for the Solana blockchain.
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-solana` feature. Enable it in
//! your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-solana"] }
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{SolanaProvider, SolanaChain};
//!
//! let provider = SolanaProvider::mainnet();
//! let slot = provider.get_height().await?;
//! let blockhash = provider.get_latest_blockhash().await?;
//! ```

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;

use async_trait::async_trait;

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// SOLANA CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Solana-specific chain operations.
///
/// Extends [`ChainProvider`] with the Solana RPC surface needed by
/// Solana DEX connectors: transaction submission, account data queries,
/// SPL token balance discovery, and recent blockhash retrieval.
///
/// ## Object safety
///
/// This trait is **not** object-safe because it uses solana SDK types in method
/// signatures. Store the concrete [`SolanaProvider`] type in connector fields
/// rather than `Box<dyn SolanaChain>`.
///
/// ## Usage in connectors
///
/// Jupiter and Raydium connectors accept an optional `Arc<SolanaProvider>`
/// for on-chain transaction submission. When no provider is supplied, swap
/// execution returns [`ExchangeError::UnsupportedOperation`].
#[async_trait]
pub trait SolanaChain: ChainProvider {
    /// Send a fully signed Solana transaction.
    ///
    /// Submits the transaction to the cluster and returns its base58-encoded
    /// signature (transaction ID). The transaction must have been signed by
    /// all required signers before calling this method.
    ///
    /// Use [`get_latest_blockhash`] to obtain a recent blockhash before
    /// building and signing the transaction.
    async fn send_transaction(&self, tx: &Transaction) -> Result<Signature, ExchangeError>;

    /// Fetch raw account data bytes for a given `pubkey`.
    ///
    /// Returns the raw data field from the account. An empty `Vec` means the
    /// account exists but contains no data (native SOL account). Returns
    /// `ExchangeError::ApiError` if the account does not exist.
    async fn get_account_data(&self, pubkey: &Pubkey) -> Result<Vec<u8>, ExchangeError>;

    /// Get all SPL token accounts owned by `owner`, with their token balances.
    ///
    /// Returns a list of `(token_account_pubkey, ui_amount_as_u64_in_smallest_unit)`.
    /// This wraps `getTokenAccountsByOwner` with the Token Program filter.
    async fn get_token_accounts(&self, owner: &Pubkey) -> Result<Vec<(Pubkey, u64)>, ExchangeError>;

    /// Get the latest blockhash from the cluster.
    ///
    /// Required when building Solana transactions — the blockhash is embedded
    /// in the transaction and must be recent (within ~150 blocks / ~60 seconds).
    async fn get_latest_blockhash(&self) -> Result<Hash, ExchangeError>;

    /// Access the underlying non-blocking RPC client.
    ///
    /// Prefer the typed methods above when possible. This escape hatch is
    /// provided for operations not yet covered by the trait surface.
    fn rpc_client(&self) -> &RpcClient;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SOLANA PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete Solana chain provider backed by `solana-client`'s non-blocking RPC client.
///
/// One `SolanaProvider` per RPC endpoint is sufficient. Multiple DEX connectors
/// targeting Solana (Jupiter, Raydium) can share a single `SolanaProvider` instance
/// via `Arc<SolanaProvider>`, reusing the same HTTP connection pool.
///
/// ## Construction
///
/// Use the convenience constructors for known clusters:
///
/// ```rust,ignore
/// let mainnet = SolanaProvider::mainnet();
/// let devnet  = SolanaProvider::devnet();
/// let custom  = SolanaProvider::new("https://my-rpc.example.com");
/// ```
pub struct SolanaProvider {
    /// Non-blocking RPC client
    client: RpcClient,
    /// Human-readable cluster name for logging
    cluster_name: String,
}

impl SolanaProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a `SolanaProvider` connected to the given RPC URL.
    ///
    /// Uses `CommitmentConfig::confirmed` by default — a reasonable balance
    /// between latency and finality for trading operations.
    pub fn new(rpc_url: &str) -> Self {
        Self::with_commitment(rpc_url, CommitmentConfig::confirmed())
    }

    /// Create a `SolanaProvider` with an explicit commitment level.
    pub fn with_commitment(rpc_url: &str, commitment: CommitmentConfig) -> Self {
        let client = RpcClient::new_with_commitment(rpc_url.to_string(), commitment);
        let cluster_name = Self::infer_cluster_name(rpc_url);
        Self { client, cluster_name }
    }

    /// Solana mainnet-beta using the public Solana Labs RPC.
    ///
    /// Note: The public RPC is rate-limited. For production use, obtain a
    /// dedicated endpoint from Helius, Triton, QuickNode, or Alchemy.
    pub fn mainnet() -> Self {
        Self::new("https://api.mainnet-beta.solana.com")
    }

    /// Solana devnet using the public Solana Labs devnet RPC.
    pub fn devnet() -> Self {
        Self::new("https://api.devnet.solana.com")
    }

    /// Solana testnet using the public Solana Labs testnet RPC.
    pub fn testnet() -> Self {
        Self::new("https://api.testnet.solana.com")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn infer_cluster_name(rpc_url: &str) -> String {
        if rpc_url.contains("mainnet") {
            "solana-mainnet".to_string()
        } else if rpc_url.contains("devnet") {
            "solana-devnet".to_string()
        } else if rpc_url.contains("testnet") {
            "solana-testnet".to_string()
        } else {
            "solana-custom".to_string()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for SolanaProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Solana
    }

    /// Broadcast a pre-signed serialized Solana transaction.
    ///
    /// `tx_bytes` must be a bincode-serialized [`Transaction`] that has been
    /// fully signed. Returns the base58-encoded transaction signature.
    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        let tx: Transaction = bincode::deserialize(tx_bytes).map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "Failed to deserialize transaction bytes: {}",
                e
            ))
        })?;

        let sig = self
            .client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| ExchangeError::Network(format!("send_and_confirm_transaction failed: {}", e)))?;

        Ok(sig.to_string())
    }

    /// Get the current slot (Solana's equivalent of block height).
    async fn get_height(&self) -> Result<u64, ExchangeError> {
        self.client
            .get_slot()
            .await
            .map_err(|e| ExchangeError::Network(format!("getSlot failed: {}", e)))
    }

    /// Not applicable for Solana — Solana uses recent blockhashes, not nonces.
    ///
    /// Returns `Err(ExchangeError::UnsupportedOperation)`. Use
    /// [`SolanaChain::get_latest_blockhash`] to obtain the blockhash required
    /// for building transactions.
    async fn get_nonce(&self, _address: &str) -> Result<u64, ExchangeError> {
        Err(ExchangeError::UnsupportedOperation(
            "Solana does not use per-account nonces. Use get_latest_blockhash() instead.".to_string(),
        ))
    }

    /// Get the SOL balance of an address in **lamports** (1 SOL = 1,000,000,000 lamports).
    ///
    /// `address` must be a base58-encoded Solana public key.
    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        let pubkey: Pubkey = address.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "Invalid Solana address '{}': {}",
                address, e
            ))
        })?;

        let lamports = self
            .client
            .get_balance(&pubkey)
            .await
            .map_err(|e| ExchangeError::Network(format!("getBalance failed: {}", e)))?;

        Ok(lamports.to_string())
    }

    /// Get the status of a Solana transaction by its base58-encoded signature.
    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        let sig: Signature = tx_hash.parse().map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "Invalid Solana signature '{}': {}",
                tx_hash, e
            ))
        })?;

        let status = self
            .client
            .get_signature_status(&sig)
            .await
            .map_err(|e| ExchangeError::Network(format!("getSignatureStatuses failed: {}", e)))?;

        match status {
            None => Ok(TxStatus::NotFound),
            Some(Ok(())) => Ok(TxStatus::Confirmed { block: 0 }),
            Some(Err(tx_error)) => Ok(TxStatus::Failed {
                reason: tx_error.to_string(),
            }),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SolanaChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl SolanaChain for SolanaProvider {
    async fn send_transaction(&self, tx: &Transaction) -> Result<Signature, ExchangeError> {
        self.client
            .send_and_confirm_transaction(tx)
            .await
            .map_err(|e| ExchangeError::Network(format!("send_and_confirm_transaction failed: {}", e)))
    }

    async fn get_account_data(&self, pubkey: &Pubkey) -> Result<Vec<u8>, ExchangeError> {
        let account = self
            .client
            .get_account(pubkey)
            .await
            .map_err(|e| {
                ExchangeError::ApiError(format!(
                    "getAccountInfo for {} failed: {}",
                    pubkey, e
                ))
            })?;

        Ok(account.data)
    }

    async fn get_token_accounts(
        &self,
        owner: &Pubkey,
    ) -> Result<Vec<(Pubkey, u64)>, ExchangeError> {
        use solana_client::rpc_request::TokenAccountsFilter;

        // SPL Token program ID — well-known constant, no spl-token crate needed
        const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
        let token_program: Pubkey = SPL_TOKEN_PROGRAM_ID.parse().expect("valid constant pubkey");

        let accounts = self
            .client
            .get_token_accounts_by_owner(
                owner,
                TokenAccountsFilter::ProgramId(token_program),
            )
            .await
            .map_err(|e| {
                ExchangeError::ApiError(format!(
                    "getTokenAccountsByOwner for {} failed: {}",
                    owner, e
                ))
            })?;

        let mut result = Vec::with_capacity(accounts.len());

        for keyed_account in accounts {
            let pubkey: Pubkey = keyed_account.pubkey.parse().map_err(|e| {
                ExchangeError::Parse(format!(
                    "Failed to parse token account pubkey '{}': {}",
                    keyed_account.pubkey, e
                ))
            })?;

            // Extract the token amount.
            // The RPC returns parsed JSON for token accounts containing
            // info.tokenAmount.amount as a decimal string.
            //
            // We match on UiAccountData variants:
            // - Json: the parsed representation (expected path for token accounts)
            // - Binary/LegacyBinary: raw bytes — skip, no easy amount extraction
            let amount = match &keyed_account.account.data {
                solana_account_decoder::UiAccountData::Json(parsed) => {
                    // parsed.parsed is a serde_json::Value of the account state
                    serde_json::from_value::<serde_json::Value>(parsed.parsed.clone())
                        .ok()
                        .and_then(|v| {
                            v.get("info")
                                .and_then(|info| info.get("tokenAmount"))
                                .and_then(|ta| ta.get("amount"))
                                .and_then(|a| a.as_str())
                                .and_then(|s| s.parse::<u64>().ok())
                        })
                        .unwrap_or(0)
                }
                _ => 0,
            };

            result.push((pubkey, amount));
        }

        Ok(result)
    }

    async fn get_latest_blockhash(&self) -> Result<Hash, ExchangeError> {
        self.client
            .get_latest_blockhash()
            .await
            .map_err(|e| ExchangeError::Network(format!("getLatestBlockhash failed: {}", e)))
    }

    fn rpc_client(&self) -> &RpcClient {
        &self.client
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_provider_chain_family() {
        let provider = SolanaProvider::mainnet();
        assert_eq!(provider.chain_family(), ChainFamily::Solana);
    }

    #[test]
    fn test_solana_provider_cluster_names() {
        let mainnet = SolanaProvider::mainnet();
        assert_eq!(mainnet.cluster_name, "solana-mainnet");

        let devnet = SolanaProvider::devnet();
        assert_eq!(devnet.cluster_name, "solana-devnet");

        let testnet = SolanaProvider::testnet();
        assert_eq!(testnet.cluster_name, "solana-testnet");

        let custom = SolanaProvider::new("https://my-private-rpc.com/rpc");
        assert_eq!(custom.cluster_name, "solana-custom");
    }

    #[test]
    fn test_solana_provider_new() {
        let provider = SolanaProvider::new("https://api.mainnet-beta.solana.com");
        assert_eq!(provider.chain_family(), ChainFamily::Solana);
        assert_eq!(provider.cluster_name, "solana-mainnet");
    }
}
