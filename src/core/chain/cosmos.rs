//! # CosmosProvider — cosmrs-backed Cosmos SDK chain provider
//!
//! Implements [`ChainProvider`] and [`CosmosChain`] for all Cosmos SDK chains
//! (dYdX, Osmosis, Cosmos Hub, etc.).
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-cosmos` feature. Enable it
//! in your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-cosmos"] }
//! ```
//!
//! ## Key feature: sequence number management
//!
//! Cosmos SDK transactions require a monotonically increasing sequence number
//! per address. Concurrent `place_order` calls with the same sequence number
//! will fail on-chain — only the first tx with a given sequence is accepted.
//!
//! `CosmosProvider` maintains an internal `Mutex<HashMap<address, sequence>>`
//! cache. `next_sequence()` increments and returns atomically, so concurrent
//! callers always get distinct sequence numbers even without awaiting the
//! on-chain result.
//!
//! The cache is refreshed from the chain when a new address is seen or when
//! `refresh_sequence()` is called explicitly (e.g. after a tx fails with a
//! "sequence mismatch" error).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{CosmosProvider, CosmosChain};
//!
//! let provider = CosmosProvider::dydx_mainnet();
//! let height = provider.get_height().await?;
//! let (account_number, sequence) = provider.query_account("dydx1abc...").await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// COSMOS CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Cosmos SDK-specific chain operations.
///
/// Extends [`ChainProvider`] with sequence number management, tx simulation,
/// and account queries specific to the Cosmos SDK transaction model.
///
/// ## Object safety
///
/// This trait is object-safe: all methods take `&self` with no generics.
/// It can be stored as `Arc<dyn CosmosChain>`.
#[async_trait]
pub trait CosmosChain: ChainProvider {
    /// Return the currently cached sequence number for `address`.
    ///
    /// If the address has not been seen before, queries the chain first
    /// and populates the cache. Use [`next_sequence`] when building a new tx.
    async fn get_sequence(&self, address: &str) -> Result<u64, ExchangeError>;

    /// Atomically increment and return the next sequence number.
    ///
    /// This is the method to call when building a new transaction. It
    /// ensures that concurrent callers always get distinct values, even if
    /// multiple txs are in-flight simultaneously without waiting for on-chain
    /// inclusion.
    ///
    /// If the address is not in the cache, the chain is queried once to
    /// obtain the current on-chain sequence, then incremented before
    /// returning.
    async fn next_sequence(&self, address: &str) -> Result<u64, ExchangeError>;

    /// Force-refresh the sequence from the chain and return the new value.
    ///
    /// Call this after a tx fails with a "sequence mismatch" error so that
    /// subsequent txs use the correct on-chain sequence.
    async fn refresh_sequence(&self, address: &str) -> Result<u64, ExchangeError>;

    /// Query the chain for `(account_number, sequence)` of `address`.
    ///
    /// This is a direct chain query — the result is not cached. Prefer
    /// [`get_sequence`] / [`next_sequence`] for normal tx building.
    async fn query_account(&self, address: &str) -> Result<(u64, u64), ExchangeError>;

    /// Simulate a tx and return the estimated gas units.
    ///
    /// `tx_bytes` is the protobuf-serialised `TxRaw`. The simulation
    /// endpoint on most Cosmos nodes returns the `gas_used` field; add a
    /// safety multiplier (e.g. × 1.5) before setting `gas_limit`.
    async fn simulate(&self, tx_bytes: &[u8]) -> Result<u64, ExchangeError>;

    /// Broadcast a signed `TxRaw` and return the tx hash.
    ///
    /// Equivalent to `ChainProvider::broadcast_tx` but named explicitly for
    /// Cosmos to match the Cosmos SDK vocabulary.
    async fn broadcast_tx_sync(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// COSMOS PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete Cosmos SDK chain provider.
///
/// Communicates with the chain via REST (LCD/API) endpoints. The sequence
/// number cache prevents nonce collisions when multiple transactions are
/// built in close succession without waiting for on-chain inclusion.
///
/// ## Construction
///
/// Use the chain-specific convenience constructors for known networks:
///
/// ```rust,ignore
/// let provider = CosmosProvider::dydx_mainnet();
/// let provider = CosmosProvider::dydx_testnet();
/// let provider = CosmosProvider::new("https://lcd.osmosis.zone", "osmosis-1");
/// ```
///
/// ## Thread safety
///
/// `CosmosProvider` is `Send + Sync` and intended to be wrapped in `Arc`
/// and shared across multiple connectors on the same chain.
pub struct CosmosProvider {
    /// LCD/REST endpoint (e.g. `"https://dydx-rest.publicnode.com"`)
    endpoint: String,
    /// Cosmos chain ID (e.g. `"dydx-mainnet-1"`)
    chain_id: String,
    /// HTTP client shared by all requests
    http: reqwest::Client,
    /// Sequence number cache: address → next usable sequence
    ///
    /// The stored value is the **next** sequence to use, not the last used.
    /// When `next_sequence` is called, the value is returned and incremented
    /// atomically under the lock.
    sequences: Arc<Mutex<HashMap<String, u64>>>,
}

impl CosmosProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a `CosmosProvider` for any Cosmos SDK chain.
    ///
    /// `endpoint` is the LCD/REST URL (no trailing slash).
    /// `chain_id` is the Cosmos chain identifier string.
    pub fn new(endpoint: &str, chain_id: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client construction is infallible with valid config");

        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            chain_id: chain_id.to_string(),
            http,
            sequences: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// dYdX v4 mainnet (`dydx-mainnet-1`).
    ///
    /// Uses the public PublicNode LCD endpoint.
    pub fn dydx_mainnet() -> Self {
        Self::new(
            "https://dydx-rest.publicnode.com",
            "dydx-mainnet-1",
        )
    }

    /// dYdX v4 testnet (`dydx-testnet-4`).
    pub fn dydx_testnet() -> Self {
        Self::new(
            "https://dydx-testnet-rest.publicnode.com",
            "dydx-testnet-4",
        )
    }

    /// Osmosis mainnet (`osmosis-1`).
    pub fn osmosis_mainnet() -> Self {
        Self::new("https://lcd.osmosis.zone", "osmosis-1")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Query `cosmos/auth/v1beta1/accounts/{address}` and parse
    /// `(account_number, sequence)`.
    async fn fetch_account_info(
        &self,
        address: &str,
    ) -> Result<(u64, u64), ExchangeError> {
        let url = format!(
            "{}/cosmos/auth/v1beta1/accounts/{}",
            self.endpoint, address
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!(
                "CosmosProvider: account query failed for {}: {}",
                address, e
            )))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExchangeError::Network(format!(
                "CosmosProvider: account query HTTP {} for {}: {}",
                status, address, body
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!(
                "CosmosProvider: account JSON parse error: {}",
                e
            )))?;

        let account = json
            .get("account")
            .ok_or_else(|| ExchangeError::Parse(
                "CosmosProvider: missing 'account' field in response".to_string()
            ))?;

        Self::extract_account_fields(account)
    }

    /// Extract `(account_number, sequence)` from a JSON account object.
    ///
    /// Handles both the flat layout (`{account_number, sequence}`) and
    /// the wrapped layout (`{base_account: {account_number, sequence}}`).
    fn extract_account_fields(
        account: &serde_json::Value,
    ) -> Result<(u64, u64), ExchangeError> {
        // Try flat layout — fields may be strings or integers
        let num_opt = account.get("account_number")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| v.as_u64())
            });

        let seq_opt = account.get("sequence")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| v.as_u64())
            });

        if let (Some(num), Some(seq)) = (num_opt, seq_opt) {
            return Ok((num, seq));
        }

        // Try nested under base_account (many vesting/module account types)
        if let Some(base) = account.get("base_account") {
            return Self::extract_account_fields(base);
        }

        // Try nested under value.base_account (older Cosmos SDK layout)
        if let Some(val) = account.get("value") {
            if let Some(base) = val.get("base_account") {
                return Self::extract_account_fields(base);
            }
            // Or value itself is the account
            return Self::extract_account_fields(val);
        }

        Err(ExchangeError::Parse(format!(
            "CosmosProvider: cannot extract account_number/sequence from: {}",
            account
        )))
    }

    /// Query the latest block height via
    /// `cosmos/base/tendermint/v1beta1/blocks/latest`.
    async fn fetch_latest_height(&self) -> Result<u64, ExchangeError> {
        let url = format!(
            "{}/cosmos/base/tendermint/v1beta1/blocks/latest",
            self.endpoint
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!(
                "CosmosProvider: block height query failed: {}",
                e
            )))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExchangeError::Network(format!(
                "CosmosProvider: blocks/latest HTTP {}: {}",
                status, body
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!(
                "CosmosProvider: block JSON parse error: {}",
                e
            )))?;

        let height_str = json
            .pointer("/block/header/height")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(
                "CosmosProvider: missing block.header.height".to_string()
            ))?;

        height_str.parse::<u64>().map_err(|e| {
            ExchangeError::Parse(format!(
                "CosmosProvider: block height parse error: {}",
                e
            ))
        })
    }

    /// Query native token balance via `cosmos/bank/v1beta1/balances/{address}`.
    ///
    /// Returns the amount of the first coin in the balance list as a decimal
    /// string, or `"0"` if the balance list is empty.
    async fn fetch_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        let url = format!(
            "{}/cosmos/bank/v1beta1/balances/{}",
            self.endpoint, address
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!(
                "CosmosProvider: balance query failed for {}: {}",
                address, e
            )))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExchangeError::Network(format!(
                "CosmosProvider: balance HTTP {} for {}: {}",
                status, address, body
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!(
                "CosmosProvider: balance JSON parse error: {}",
                e
            )))?;

        // Balances is an array of {denom, amount} objects.
        // Return the first balance amount as string, or "0" if empty.
        let amount = json
            .get("balances")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|coin| coin.get("amount"))
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();

        Ok(amount)
    }

    /// Broadcast a `TxRaw` via the REST `cosmos/tx/v1beta1/txs` endpoint.
    ///
    /// Returns the transaction hash on success.
    async fn broadcast_tx_rest(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        use base64::Engine as _;

        let url = format!("{}/cosmos/tx/v1beta1/txs", self.endpoint);

        // REST broadcast expects the TxRaw as base64 inside JSON:
        //   { "tx_bytes": "<base64>", "mode": "BROADCAST_MODE_SYNC" }
        let encoded = base64::engine::general_purpose::STANDARD.encode(tx_bytes);
        let body = serde_json::json!({
            "tx_bytes": encoded,
            "mode": "BROADCAST_MODE_SYNC"
        });

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!(
                "CosmosProvider: broadcast_tx POST failed: {}",
                e
            )))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!(
                "CosmosProvider: broadcast_tx JSON parse error: {}",
                e
            )))?;

        // Non-zero code means the chain rejected the tx
        if let Some(code) = json
            .pointer("/tx_response/code")
            .and_then(|v| v.as_u64())
        {
            if code != 0 {
                let raw_log = json
                    .pointer("/tx_response/raw_log")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                return Err(ExchangeError::Api {
                    code: code as i32,
                    message: format!("CosmosProvider broadcast rejected: {}", raw_log),
                });
            }
        }

        let txhash = json
            .pointer("/tx_response/txhash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(
                "CosmosProvider: missing tx_response.txhash in broadcast response".to_string()
            ))?
            .to_string();

        Ok(txhash)
    }

    /// Query tx status via `cosmos/tx/v1beta1/txs/{hash}`.
    async fn fetch_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        let url = format!("{}/cosmos/tx/v1beta1/txs/{}", self.endpoint, tx_hash);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!(
                "CosmosProvider: tx status query failed: {}",
                e
            )))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(TxStatus::NotFound);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExchangeError::Network(format!(
                "CosmosProvider: tx status HTTP {}: {}",
                status, body
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!(
                "CosmosProvider: tx status JSON parse error: {}",
                e
            )))?;

        let code = json
            .pointer("/tx_response/code")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let height = json
            .pointer("/tx_response/height")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| v.as_u64())
            })
            .unwrap_or(0);

        if code == 0 {
            Ok(TxStatus::Confirmed { block: height })
        } else {
            let reason = json
                .pointer("/tx_response/raw_log")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string();
            Ok(TxStatus::Failed { reason })
        }
    }

    /// Simulate a tx via `cosmos/tx/v1beta1/simulate` and return gas used.
    async fn fetch_simulate(&self, tx_bytes: &[u8]) -> Result<u64, ExchangeError> {
        use base64::Engine as _;

        let url = format!("{}/cosmos/tx/v1beta1/simulate", self.endpoint);

        let encoded = base64::engine::general_purpose::STANDARD.encode(tx_bytes);
        let body = serde_json::json!({ "tx_bytes": encoded });

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!(
                "CosmosProvider: simulate POST failed: {}",
                e
            )))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExchangeError::Network(format!(
                "CosmosProvider: simulate HTTP {}: {}",
                status, body
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!(
                "CosmosProvider: simulate JSON parse error: {}",
                e
            )))?;

        let gas_used = json
            .pointer("/gas_info/gas_used")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| v.as_u64())
            })
            .ok_or_else(|| ExchangeError::Parse(
                "CosmosProvider: missing gas_info.gas_used in simulate response".to_string()
            ))?;

        Ok(gas_used)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for CosmosProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Cosmos { chain_id: self.chain_id.clone() }
    }

    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        self.broadcast_tx_rest(tx_bytes).await
    }

    async fn get_height(&self) -> Result<u64, ExchangeError> {
        self.fetch_latest_height().await
    }

    /// Returns the current account sequence number (Cosmos nonce).
    async fn get_nonce(&self, address: &str) -> Result<u64, ExchangeError> {
        self.get_sequence(address).await
    }

    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        self.fetch_native_balance(address).await
    }

    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        self.fetch_tx_status(tx_hash).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CosmosChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CosmosChain for CosmosProvider {
    async fn get_sequence(&self, address: &str) -> Result<u64, ExchangeError> {
        {
            let cache = self.sequences.lock().await;
            if let Some(&seq) = cache.get(address) {
                return Ok(seq);
            }
        }
        // Address not in cache — fetch from chain and populate
        let (_account_number, sequence) = self.fetch_account_info(address).await?;
        {
            let mut cache = self.sequences.lock().await;
            // Only insert if not already set by a concurrent caller
            cache.entry(address.to_string()).or_insert(sequence);
        }
        Ok(sequence)
    }

    async fn next_sequence(&self, address: &str) -> Result<u64, ExchangeError> {
        // Ensure the cache is populated before taking the lock for mutation
        {
            let needs_fetch = {
                let cache = self.sequences.lock().await;
                !cache.contains_key(address)
            };

            if needs_fetch {
                let (_account_number, sequence) = self.fetch_account_info(address).await?;
                let mut cache = self.sequences.lock().await;
                // Double-check: another concurrent caller may have set it
                cache.entry(address.to_string()).or_insert(sequence);
            }
        }

        // Atomically take the current value and increment for the next caller
        let mut cache = self.sequences.lock().await;
        let seq = cache
            .get_mut(address)
            .expect("just inserted above; cache entry must exist");
        let current = *seq;
        *seq = current + 1;
        Ok(current)
    }

    async fn refresh_sequence(&self, address: &str) -> Result<u64, ExchangeError> {
        let (_account_number, sequence) = self.fetch_account_info(address).await?;
        let mut cache = self.sequences.lock().await;
        cache.insert(address.to_string(), sequence);
        Ok(sequence)
    }

    async fn query_account(&self, address: &str) -> Result<(u64, u64), ExchangeError> {
        self.fetch_account_info(address).await
    }

    async fn simulate(&self, tx_bytes: &[u8]) -> Result<u64, ExchangeError> {
        self.fetch_simulate(tx_bytes).await
    }

    async fn broadcast_tx_sync(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        self.broadcast_tx_rest(tx_bytes).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_provider_chain_family() {
        let provider = CosmosProvider::dydx_mainnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Cosmos { chain_id: "dydx-mainnet-1".to_string() }
        );
    }

    #[test]
    fn test_cosmos_provider_testnet_chain_id() {
        let provider = CosmosProvider::dydx_testnet();
        match provider.chain_family() {
            ChainFamily::Cosmos { chain_id } => {
                assert_eq!(chain_id, "dydx-testnet-4");
            }
            _ => panic!("Expected Cosmos chain family"),
        }
    }

    #[test]
    fn test_extract_account_fields_flat_strings() {
        let json = serde_json::json!({
            "account_number": "42",
            "sequence": "7"
        });
        let (num, seq) = CosmosProvider::extract_account_fields(&json).unwrap();
        assert_eq!(num, 42);
        assert_eq!(seq, 7);
    }

    #[test]
    fn test_extract_account_fields_flat_integers() {
        // Some nodes return integers, not strings
        let json = serde_json::json!({
            "account_number": 5,
            "sequence": 12
        });
        let (num, seq) = CosmosProvider::extract_account_fields(&json).unwrap();
        assert_eq!(num, 5);
        assert_eq!(seq, 12);
    }

    #[test]
    fn test_extract_account_fields_nested_base_account() {
        let json = serde_json::json!({
            "@type": "/cosmos.auth.v1beta1.BaseVestingAccount",
            "base_account": {
                "account_number": "100",
                "sequence": "3"
            }
        });
        let (num, seq) = CosmosProvider::extract_account_fields(&json).unwrap();
        assert_eq!(num, 100);
        assert_eq!(seq, 3);
    }

    #[tokio::test]
    async fn test_sequence_cache_atomicity() {
        // Verify that consecutive next_sequence calls return distinct values
        // without hitting the network (cache is pre-seeded).
        let provider = Arc::new(CosmosProvider::dydx_mainnet());

        // Pre-seed the cache to avoid network calls in this unit test
        {
            let mut cache = provider.sequences.lock().await;
            cache.insert("dydx1test".to_string(), 10u64);
        }

        // next_sequence returns the current value and increments
        let seq1 = provider.next_sequence("dydx1test").await.unwrap();
        let seq2 = provider.next_sequence("dydx1test").await.unwrap();
        let seq3 = provider.next_sequence("dydx1test").await.unwrap();

        assert_eq!(seq1, 10, "first call should return the seeded value");
        assert_eq!(seq2, 11, "second call should return seeded + 1");
        assert_eq!(seq3, 12, "third call should return seeded + 2");

        // Cache now holds 13 as the next pending sequence
        let cache = provider.sequences.lock().await;
        assert_eq!(*cache.get("dydx1test").unwrap(), 13u64);
    }

    #[tokio::test]
    async fn test_get_sequence_returns_cached() {
        let provider = CosmosProvider::dydx_mainnet();

        {
            let mut cache = provider.sequences.lock().await;
            cache.insert("dydx1abc".to_string(), 5u64);
        }

        // get_sequence should return the cached value without network
        let seq = provider.get_sequence("dydx1abc").await.unwrap();
        assert_eq!(seq, 5);

        // get_sequence does NOT increment the cache
        let seq2 = provider.get_sequence("dydx1abc").await.unwrap();
        assert_eq!(seq2, 5);
    }
}
