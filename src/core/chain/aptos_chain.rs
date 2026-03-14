//! # AptosProvider — raw REST HTTP Aptos chain provider
//!
//! Implements [`ChainProvider`] and [`AptosChain`] for the Aptos L1 blockchain.
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-aptos` feature. Enable it
//! in your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-aptos"] }
//! ```
//!
//! ## Transport
//!
//! Uses raw REST over `reqwest` — no aptos-sdk required (avoids the
//! `tokio_unstable` build flag requirement). Every call maps directly to the
//! Aptos REST API (`/v1/...`).
//!
//! ## APT balance
//!
//! Native APT balance is read via the account resource
//! `0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>`. The `coin.value` field
//! holds the balance in Octas (1 APT = 10^8 Octas), returned as a decimal
//! string.
//!
//! ## Transactions
//!
//! `broadcast_tx` sends BCS-encoded signed transaction bytes with the
//! `application/x.aptos.signed_transaction+bcs` content-type, as required
//! by the Aptos REST API. The returned transaction hash is the hex-encoded
//! digest (with `0x` prefix).
//!
//! ## View functions
//!
//! `view_function` executes a read-only Move function via `POST /view`. This
//! is the idiomatic way to query on-chain state without submitting transactions.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{AptosProvider, AptosChain};
//!
//! let provider = AptosProvider::mainnet();
//! let height   = provider.get_height().await?;
//! let balance  = provider.get_native_balance("0xabc...").await?;
//! let info     = provider.get_ledger_info().await?;
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// APT native coin resource type used for balance queries.
const APT_COIN_TYPE: &str = "0x1::aptos_coin::AptosCoin";

/// Content-type required by the Aptos node for BCS-encoded signed transactions.
const BCS_SIGNED_TX_CONTENT_TYPE: &str = "application/x.aptos.signed_transaction+bcs";

// ═══════════════════════════════════════════════════════════════════════════════
// APTOS CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Aptos-specific chain operations.
///
/// Extends [`ChainProvider`] with the Aptos REST API surface needed by on-chain
/// connectors (DEX integrations, DeFi protocol queries, etc.): account resource
/// reads, Move view-function calls, event queries, and table item lookups.
///
/// ## Object safety
///
/// This trait is object-safe: all method signatures use plain `&str`, `&[String]`,
/// and `&[serde_json::Value]`, with no SDK-specific types.
///
/// ## Addresses
///
/// Aptos addresses are 32-byte hex strings with a `0x` prefix,
/// e.g. `"0x1"` (framework) or `"0xabcdef..."` (user account).
#[async_trait]
pub trait AptosChain: ChainProvider {
    /// Get account info (sequence_number, authentication_key).
    ///
    /// Returns the raw JSON object from `GET /accounts/{address}`.
    async fn get_account(&self, address: &str) -> Result<Value, ExchangeError>;

    /// Get all Move resources for an account.
    ///
    /// Returns the raw JSON array from `GET /accounts/{address}/resources`.
    async fn get_account_resources(&self, address: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Get a specific resource by its fully-qualified Move type string.
    ///
    /// `resource_type` must be a fully-qualified Move struct type,
    /// e.g. `"0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>"`.
    ///
    /// Returns the raw JSON object from
    /// `GET /accounts/{address}/resource/{resource_type}`.
    async fn get_account_resource(
        &self,
        address: &str,
        resource_type: &str,
    ) -> Result<Value, ExchangeError>;

    /// Get all Move modules deployed by an account.
    ///
    /// Returns the raw JSON array from `GET /accounts/{address}/modules`.
    async fn get_account_modules(&self, address: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Get the coin balance for any coin type.
    ///
    /// `coin_type` is a fully-qualified Move type string, e.g.
    /// `"0x1::aptos_coin::AptosCoin"` for APT, or a third-party token type.
    ///
    /// Returns the balance as a decimal string in the coin's smallest unit
    /// (Octas for APT, 10^8 per APT).
    async fn get_coin_balance(
        &self,
        address: &str,
        coin_type: &str,
    ) -> Result<String, ExchangeError>;

    /// Execute a read-only Move view function.
    ///
    /// `function` is a fully-qualified Move function identifier,
    /// e.g. `"0x1::coin::balance"`.
    /// `type_args` are the generic type arguments, e.g. `["0x1::aptos_coin::AptosCoin"]`.
    /// `args` are the function arguments as JSON values.
    ///
    /// Returns the list of return values as JSON (`POST /view`).
    async fn view_function(
        &self,
        function: &str,
        type_args: &[String],
        args: &[Value],
    ) -> Result<Vec<Value>, ExchangeError>;

    /// Get recent transactions for an account.
    ///
    /// `limit` caps the number of returned transactions (default: 25, max: 25 per page).
    ///
    /// Returns the raw JSON array from `GET /accounts/{address}/transactions`.
    async fn get_account_transactions(
        &self,
        address: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError>;

    /// Get a transaction by its hash.
    ///
    /// Returns the raw JSON object from `GET /transactions/by_hash/{hash}`.
    async fn get_transaction_by_hash(&self, hash: &str) -> Result<Value, ExchangeError>;

    /// Get a block by its height (block number).
    ///
    /// `with_transactions` controls whether the response includes the full
    /// list of transactions in the block.
    ///
    /// Returns the raw JSON object from `GET /blocks/by_height/{height}`.
    async fn get_block_by_height(
        &self,
        height: u64,
        with_transactions: bool,
    ) -> Result<Value, ExchangeError>;

    /// Get events by event handle.
    ///
    /// `event_handle` is the resource type that contains the event handle field,
    /// e.g. `"0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>"`.
    /// `field_name` is the field within that resource, e.g. `"deposit_events"`.
    ///
    /// Returns the raw JSON array from
    /// `GET /accounts/{address}/events/{event_handle}/{field_name}`.
    async fn get_events(
        &self,
        address: &str,
        event_handle: &str,
        field_name: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError>;

    /// Get an item from a Move table.
    ///
    /// Used to query DEX pool state, AMM reserves, and other table-mapped data.
    ///
    /// `table_handle` is the hex handle of the table resource.
    /// `key_type` and `value_type` are fully-qualified Move type strings.
    /// `key` is the lookup key as a JSON value.
    ///
    /// Returns the raw JSON value from `POST /tables/{table_handle}/item`.
    async fn get_table_item(
        &self,
        table_handle: &str,
        key_type: &str,
        value_type: &str,
        key: Value,
    ) -> Result<Value, ExchangeError>;

    /// Estimate the current gas price in Octas per gas unit.
    ///
    /// Returns the `gas_estimate` field from `GET /estimate_gas_price`.
    async fn estimate_gas_price(&self) -> Result<u64, ExchangeError>;

    /// Get ledger info (chain_id, epoch, ledger_version, block_height, etc.).
    ///
    /// Returns the raw JSON object from `GET /` (the root index endpoint).
    async fn get_ledger_info(&self) -> Result<Value, ExchangeError>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// APTOS PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete Aptos chain provider using the Aptos REST API over HTTP.
///
/// One `AptosProvider` per RPC endpoint is sufficient. Multiple connectors
/// targeting Aptos can share a single instance via `Arc<AptosProvider>`.
///
/// ## Construction
///
/// ```rust,ignore
/// // Public Aptos Labs endpoints
/// let mainnet = AptosProvider::mainnet();
/// let testnet = AptosProvider::testnet();
///
/// // Custom node
/// let provider = AptosProvider::new("https://my-aptos-node.example.com/v1", "mainnet");
/// ```
pub struct AptosProvider {
    /// REST API base URL, e.g. `"https://fullnode.mainnet.aptoslabs.com/v1"`.
    api_url: String,
    /// Shared HTTP client for all REST calls.
    client: reqwest::Client,
    /// Human-readable network name: `"mainnet"`, `"testnet"`, or `"devnet"`.
    network: String,
}

impl AptosProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a provider pointing at a custom REST API endpoint.
    ///
    /// `api_url` should end with `/v1` (e.g. `"https://fullnode.mainnet.aptoslabs.com/v1"`).
    /// `network` is used only for [`ChainFamily`] identification (`"mainnet"`,
    /// `"testnet"`, or `"devnet"`).
    pub fn new(api_url: impl Into<String>, network: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            client: reqwest::Client::new(),
            network: network.into(),
        }
    }

    /// Aptos mainnet using the public Aptos Labs fullnode.
    pub fn mainnet() -> Self {
        Self::new(
            "https://fullnode.mainnet.aptoslabs.com/v1",
            "mainnet",
        )
    }

    /// Aptos testnet using the public Aptos Labs fullnode.
    pub fn testnet() -> Self {
        Self::new(
            "https://fullnode.testnet.aptoslabs.com/v1",
            "testnet",
        )
    }

    /// Aptos devnet using the public Aptos Labs fullnode.
    pub fn devnet() -> Self {
        Self::new(
            "https://fullnode.devnet.aptoslabs.com/v1",
            "devnet",
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal HTTP helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Perform a `GET` request to `{api_url}/{path}` and return the parsed JSON.
    ///
    /// Returns [`ExchangeError::Network`] on transport errors,
    /// [`ExchangeError::Parse`] if the response body cannot be decoded as JSON,
    /// and [`ExchangeError::InvalidRequest`] if the node returns a non-2xx status.
    async fn get(&self, path: &str) -> Result<Value, ExchangeError> {
        let url = format!("{}/{}", self.api_url, path);
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("GET {path}: {e}")))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("GET {path}: failed to parse response: {e}")))?;

        if !status.is_success() {
            let msg = body
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            return Err(ExchangeError::InvalidRequest(format!(
                "GET {path}: HTTP {status}: {msg}"
            )));
        }

        Ok(body)
    }

    /// Perform a `GET` request with an optional query parameter.
    async fn get_with_limit(&self, path: &str, limit: Option<u32>) -> Result<Value, ExchangeError> {
        let url = match limit {
            Some(l) => format!("{}/{}?limit={}", self.api_url, path, l),
            None => format!("{}/{}", self.api_url, path),
        };

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("GET {path}: {e}")))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("GET {path}: failed to parse response: {e}")))?;

        if !status.is_success() {
            let msg = body
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            return Err(ExchangeError::InvalidRequest(format!(
                "GET {path}: HTTP {status}: {msg}"
            )));
        }

        Ok(body)
    }

    /// Perform a `POST` request to `{api_url}/{path}` with a JSON body.
    ///
    /// Returns the parsed JSON response.
    async fn post_json(&self, path: &str, body: Value) -> Result<Value, ExchangeError> {
        let url = format!("{}/{}", self.api_url, path);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("POST {path}: {e}")))?;

        let status = response.status();
        let resp_body: Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("POST {path}: failed to parse response: {e}")))?;

        if !status.is_success() {
            let msg = resp_body
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            return Err(ExchangeError::InvalidRequest(format!(
                "POST {path}: HTTP {status}: {msg}"
            )));
        }

        Ok(resp_body)
    }

    /// Perform a `POST` request with raw BCS bytes.
    ///
    /// Used by [`ChainProvider::broadcast_tx`] to submit signed transactions.
    /// Returns the parsed JSON response containing the transaction hash.
    async fn post_bcs(&self, path: &str, bytes: &[u8]) -> Result<Value, ExchangeError> {
        let url = format!("{}/{}", self.api_url, path);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", BCS_SIGNED_TX_CONTENT_TYPE)
            .header("Accept", "application/json")
            .body(bytes.to_vec())
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("POST BCS {path}: {e}")))?;

        let status = response.status();
        let resp_body: Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("POST BCS {path}: failed to parse response: {e}")))?;

        if !status.is_success() {
            let msg = resp_body
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            return Err(ExchangeError::InvalidRequest(format!(
                "POST BCS {path}: HTTP {status}: {msg}"
            )));
        }

        Ok(resp_body)
    }

    /// Parse a decimal string or u64-compatible JSON value as `u64`.
    fn parse_u64_field(value: &Value, field: &str) -> Result<u64, ExchangeError> {
        // Aptos REST API returns numbers as JSON strings in some endpoints
        if let Some(s) = value.as_str() {
            s.parse::<u64>().map_err(|e| {
                ExchangeError::Parse(format!("field '{field}': cannot parse '{s}' as u64: {e}"))
            })
        } else if let Some(n) = value.as_u64() {
            Ok(n)
        } else {
            Err(ExchangeError::Parse(format!(
                "field '{field}': expected string or integer, got: {value}"
            )))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for AptosProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Aptos {
            network: self.network.clone(),
        }
    }

    /// Broadcast a BCS-encoded signed transaction.
    ///
    /// `tx_bytes` must be a BCS-encoded `SignedTransaction` as produced by the
    /// Aptos SDK or a compatible signer. The bytes are submitted with the
    /// `application/x.aptos.signed_transaction+bcs` content-type.
    ///
    /// Returns the transaction hash (with `0x` prefix).
    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        let result = self.post_bcs("transactions", tx_bytes).await?;

        result
            .get("hash")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "broadcast_tx: missing 'hash' field in transaction response".to_string(),
                )
            })
    }

    /// Get the current block height from ledger info.
    ///
    /// Calls `GET /` (root endpoint) and extracts `block_height`.
    async fn get_height(&self) -> Result<u64, ExchangeError> {
        let info = self.get("").await?;
        let height_val = info.get("block_height").ok_or_else(|| {
            ExchangeError::Parse("get_height: missing 'block_height' in ledger info".to_string())
        })?;
        Self::parse_u64_field(height_val, "block_height")
    }

    /// Get the sequence number (nonce) for an Aptos account.
    ///
    /// Calls `GET /accounts/{address}` and extracts `sequence_number`.
    async fn get_nonce(&self, address: &str) -> Result<u64, ExchangeError> {
        let account = self.get_account(address).await?;
        let seq_val = account.get("sequence_number").ok_or_else(|| {
            ExchangeError::Parse(format!(
                "get_nonce: missing 'sequence_number' for address {address}"
            ))
        })?;
        Self::parse_u64_field(seq_val, "sequence_number")
    }

    /// Get the native APT balance for an address.
    ///
    /// Reads the `0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>` resource
    /// and extracts `coin.value`. Returns the balance in Octas (1 APT = 10^8 Octas)
    /// as a decimal string.
    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        self.get_coin_balance(address, APT_COIN_TYPE).await
    }

    /// Get transaction status by hash.
    ///
    /// Calls `GET /transactions/by_hash/{hash}` and maps the Aptos
    /// `vm_status` and `success` fields to [`TxStatus`].
    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        let result = self
            .get(&format!("transactions/by_hash/{tx_hash}"))
            .await;

        match result {
            Err(ExchangeError::InvalidRequest(msg))
                if msg.contains("404")
                    || msg.contains("not found")
                    || msg.contains("Transaction not found") =>
            {
                Ok(TxStatus::NotFound)
            }
            Err(e) => Err(e),
            Ok(tx) => {
                // Check if the transaction is still pending (in mempool)
                let tx_type = tx.get("type").and_then(Value::as_str).unwrap_or("");
                if tx_type == "pending_transaction" {
                    return Ok(TxStatus::Pending);
                }

                // For committed transactions, check success field
                let success = tx.get("success").and_then(Value::as_bool).unwrap_or(false);
                if success {
                    let block_height = tx
                        .get("block_height")
                        .and_then(|v| {
                            v.as_str()
                                .and_then(|s| s.parse::<u64>().ok())
                                .or_else(|| v.as_u64())
                        })
                        .unwrap_or(0);
                    Ok(TxStatus::Confirmed { block: block_height })
                } else {
                    let reason = tx
                        .get("vm_status")
                        .and_then(Value::as_str)
                        .unwrap_or("transaction failed")
                        .to_string();
                    Ok(TxStatus::Failed { reason })
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AptosChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AptosChain for AptosProvider {
    async fn get_account(&self, address: &str) -> Result<Value, ExchangeError> {
        self.get(&format!("accounts/{address}")).await
    }

    async fn get_account_resources(&self, address: &str) -> Result<Vec<Value>, ExchangeError> {
        let result = self.get(&format!("accounts/{address}/resources")).await?;
        result
            .as_array()
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "get_account_resources: expected array for address {address}"
                ))
            })
    }

    async fn get_account_resource(
        &self,
        address: &str,
        resource_type: &str,
    ) -> Result<Value, ExchangeError> {
        // URL-encode the resource type (Move type strings contain `<`, `>`, `::`)
        let encoded_type = urlencoding::encode(resource_type);
        self.get(&format!("accounts/{address}/resource/{encoded_type}"))
            .await
    }

    async fn get_account_modules(&self, address: &str) -> Result<Vec<Value>, ExchangeError> {
        let result = self.get(&format!("accounts/{address}/modules")).await?;
        result
            .as_array()
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "get_account_modules: expected array for address {address}"
                ))
            })
    }

    async fn get_coin_balance(
        &self,
        address: &str,
        coin_type: &str,
    ) -> Result<String, ExchangeError> {
        let resource_type = format!("0x1::coin::CoinStore<{coin_type}>");
        let resource = self.get_account_resource(address, &resource_type).await?;

        // Response structure: { "type": "...", "data": { "coin": { "value": "1234" }, ... } }
        let value = resource
            .get("data")
            .and_then(|d| d.get("coin"))
            .and_then(|c| c.get("value"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "get_coin_balance: missing 'data.coin.value' in resource for {coin_type}"
                ))
            })?;

        Ok(value.to_string())
    }

    async fn view_function(
        &self,
        function: &str,
        type_args: &[String],
        args: &[Value],
    ) -> Result<Vec<Value>, ExchangeError> {
        let body = json!({
            "function": function,
            "type_arguments": type_args,
            "arguments": args,
        });

        let result = self.post_json("view", body).await?;

        result
            .as_array()
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "view_function: expected array result for function '{function}'"
                ))
            })
    }

    async fn get_account_transactions(
        &self,
        address: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError> {
        let result = self
            .get_with_limit(&format!("accounts/{address}/transactions"), limit)
            .await?;

        result
            .as_array()
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "get_account_transactions: expected array for address {address}"
                ))
            })
    }

    async fn get_transaction_by_hash(&self, hash: &str) -> Result<Value, ExchangeError> {
        self.get(&format!("transactions/by_hash/{hash}")).await
    }

    async fn get_block_by_height(
        &self,
        height: u64,
        with_transactions: bool,
    ) -> Result<Value, ExchangeError> {
        let path = format!(
            "blocks/by_height/{height}?with_transactions={with_transactions}"
        );
        self.get(&path).await
    }

    async fn get_events(
        &self,
        address: &str,
        event_handle: &str,
        field_name: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError> {
        let encoded_handle = urlencoding::encode(event_handle);
        let base_path = format!("accounts/{address}/events/{encoded_handle}/{field_name}");

        let result = self.get_with_limit(&base_path, limit).await?;

        result
            .as_array()
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "get_events: expected array for address {address}, handle {event_handle}"
                ))
            })
    }

    async fn get_table_item(
        &self,
        table_handle: &str,
        key_type: &str,
        value_type: &str,
        key: Value,
    ) -> Result<Value, ExchangeError> {
        let body = json!({
            "key_type": key_type,
            "value_type": value_type,
            "key": key,
        });

        self.post_json(&format!("tables/{table_handle}/item"), body).await
    }

    async fn estimate_gas_price(&self) -> Result<u64, ExchangeError> {
        let result = self.get("estimate_gas_price").await?;
        let gas_val = result.get("gas_estimate").ok_or_else(|| {
            ExchangeError::Parse(
                "estimate_gas_price: missing 'gas_estimate' in response".to_string(),
            )
        })?;
        Self::parse_u64_field(gas_val, "gas_estimate")
    }

    async fn get_ledger_info(&self) -> Result<Value, ExchangeError> {
        // The root endpoint `GET /` returns ledger info
        self.get("").await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_family_mainnet() {
        let provider = AptosProvider::mainnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Aptos {
                network: "mainnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_testnet() {
        let provider = AptosProvider::testnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Aptos {
                network: "testnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_devnet() {
        let provider = AptosProvider::devnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Aptos {
                network: "devnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_name() {
        let p = AptosProvider::mainnet();
        assert_eq!(p.chain_family().name(), "aptos:mainnet");
    }

    #[test]
    fn test_is_aptos_helper() {
        let family = ChainFamily::Aptos {
            network: "mainnet".to_string(),
        };
        assert!(family.is_aptos("mainnet"));
        assert!(!family.is_aptos("testnet"));
    }

    #[test]
    fn test_mainnet_api_url() {
        let p = AptosProvider::mainnet();
        assert_eq!(p.api_url, "https://fullnode.mainnet.aptoslabs.com/v1");
        assert_eq!(p.network, "mainnet");
    }

    #[test]
    fn test_testnet_api_url() {
        let p = AptosProvider::testnet();
        assert_eq!(p.api_url, "https://fullnode.testnet.aptoslabs.com/v1");
    }

    #[test]
    fn test_custom_provider() {
        let p = AptosProvider::new("https://my-node.example.com/v1", "mainnet");
        assert_eq!(p.api_url, "https://my-node.example.com/v1");
        assert_eq!(p.network, "mainnet");
    }

    #[test]
    fn test_parse_u64_field_string() {
        let v = Value::String("12345".to_string());
        assert_eq!(AptosProvider::parse_u64_field(&v, "test").unwrap(), 12345u64);
    }

    #[test]
    fn test_parse_u64_field_number() {
        let v = json!(9999u64);
        assert_eq!(AptosProvider::parse_u64_field(&v, "test").unwrap(), 9999u64);
    }

    #[test]
    fn test_parse_u64_field_invalid() {
        let v = Value::String("not_a_number".to_string());
        assert!(AptosProvider::parse_u64_field(&v, "test").is_err());
    }

    #[test]
    fn test_parse_u64_field_null() {
        let v = Value::Null;
        assert!(AptosProvider::parse_u64_field(&v, "test").is_err());
    }

    #[test]
    fn test_apt_coin_type_constant() {
        assert_eq!(APT_COIN_TYPE, "0x1::aptos_coin::AptosCoin");
    }

    #[test]
    fn test_bcs_content_type() {
        assert_eq!(
            BCS_SIGNED_TX_CONTENT_TYPE,
            "application/x.aptos.signed_transaction+bcs"
        );
    }
}
