//! # SuiProvider — raw JSON-RPC Sui chain provider
//!
//! Implements [`ChainProvider`] and [`SuiChain`] for the Sui L1 blockchain.
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-sui` feature. Enable it in
//! your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-sui"] }
//! ```
//!
//! ## Transport
//!
//! Uses raw JSON-RPC over `reqwest` — no heavy sui-sdk required.
//! Every call maps directly to a named JSON-RPC method (`sui_*` / `suix_*`).
//!
//! ## Sui addresses
//!
//! Sui addresses are 32-byte hex strings with a `0x` prefix, e.g.
//! `"0x0000000000000000000000000000000000000000000000000000000000000002"`.
//! Object IDs use the same format.
//!
//! ## Balance units
//!
//! All SUI amounts are in MIST (the smallest unit). 1 SUI = 1_000_000_000 MIST.
//!
//! ## Transaction execution
//!
//! Sui uses an object-based model. There are no account nonces. Instead,
//! each transaction consumes specific owned objects (including a gas coin).
//! [`ChainProvider::get_nonce`] always returns `ExchangeError::UnsupportedOperation`
//! because the concept does not apply to Sui.
//!
//! [`ChainProvider::broadcast_tx`] accepts a JSON payload containing
//! `tx_bytes` (base64 BCS-encoded transaction) and `signatures` (array of
//! base64 serialized signatures). The payload must be UTF-8 JSON, e.g.:
//! ```json
//! {"tx_bytes":"AAAA...","signatures":["AAAA..."]}
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{SuiProvider, SuiChain};
//!
//! let provider = SuiProvider::mainnet();
//! let height = provider.get_height().await?;
//! let balances = provider.get_all_balances("0xabc...").await?;
//! let events = provider.query_events(
//!     serde_json::json!({"MoveEventType": "0xdee9::clob_v2::OrderPlaced<0x2::sui::SUI,0xaf8cd5edc19c4512f4259f0bee101a40d41ebed738afa4b..::usdc::USDC>"}),
//!     Some(50),
//! ).await?;
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// SUI CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Sui-specific chain operations.
///
/// Extends [`ChainProvider`] with the full Sui JSON-RPC surface needed by
/// DeFi connectors: object queries, coin balance reads, event subscriptions,
/// and Move call inspection.
///
/// ## Object safety
///
/// This trait is object-safe: all method signatures use plain `&str`, `String`,
/// `serde_json::Value`, and standard Rust types.
///
/// ## Move call (devInspect)
///
/// [`SuiChain::dev_inspect`] allows read-only simulation of a Move call without
/// broadcasting a transaction. Use it to read on-chain state from smart contracts
/// (e.g. DEX pool prices, LP balances).
#[async_trait]
pub trait SuiChain: ChainProvider {
    /// Get a Sui object by its ID.
    ///
    /// Returns the full object with content, owner, type, and fields.
    /// Object ID format: `"0x0000...0002"` (32-byte hex with `0x` prefix).
    ///
    /// Uses `sui_getObject` with `showContent: true, showOwner: true, showType: true`.
    async fn get_object(&self, object_id: &str) -> Result<Value, ExchangeError>;

    /// Get all objects owned by the given address.
    ///
    /// Returns the raw array of object summaries from `suix_getOwnedObjects`.
    /// Each entry contains `objectId`, `version`, `digest`, and optionally
    /// `type_` and `content` depending on requested options.
    async fn get_owned_objects(&self, owner: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Get all coin balances for an address.
    ///
    /// Returns one balance entry per coin type the address holds (SUI + all
    /// other coin types). Each entry has `coinType`, `totalBalance`,
    /// and `coinObjectCount`.
    ///
    /// Uses `suix_getAllBalances`.
    async fn get_all_balances(&self, owner: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Get coin objects of a specific type for an address.
    ///
    /// `coin_type` — optional Move type string (e.g. `"0x2::sui::SUI"`).
    /// If `None`, defaults to the native SUI coin.
    ///
    /// Returns coin objects from `suix_getCoins`.
    async fn get_coins(
        &self,
        owner: &str,
        coin_type: Option<&str>,
    ) -> Result<Vec<Value>, ExchangeError>;

    /// Execute a Move call in inspection mode (read-only, no gas consumed).
    ///
    /// `sender` — the sender address (used for ownership checks in the VM).
    /// `move_call` — a JSON object describing the PTB (programmable transaction block):
    /// ```json
    /// {
    ///   "kind": "moveCall",
    ///   "target": "0xpackage::module::function",
    ///   "arguments": [...],
    ///   "typeArguments": [...]
    /// }
    /// ```
    ///
    /// Returns the raw `devInspectTransactionBlock` result including
    /// `effects`, `events`, and `results`.
    async fn dev_inspect(
        &self,
        sender: &str,
        move_call: Value,
    ) -> Result<Value, ExchangeError>;

    /// Get a transaction block by its digest.
    ///
    /// Returns the full transaction block with effects, events, and object
    /// changes. Uses `sui_getTransactionBlock` with
    /// `showEffects: true, showEvents: true, showObjectChanges: true`.
    async fn get_transaction_block(&self, digest: &str) -> Result<Value, ExchangeError>;

    /// Query transaction blocks matching a filter.
    ///
    /// `filter` — a Sui `TransactionBlockResponseQuery` filter object, e.g.:
    /// ```json
    /// {"ToAddress": "0xabc..."}
    /// {"FromAddress": "0xabc..."}
    /// {"InputObject": "0xobjectid..."}
    /// ```
    ///
    /// `limit` — maximum number of results to return (default 50, max 100).
    ///
    /// Returns array of transaction block digests with their effects.
    async fn query_transaction_blocks(
        &self,
        filter: Value,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError>;

    /// Get all events emitted by a specific transaction digest.
    ///
    /// Returns the raw events array from `sui_getEvents`.
    async fn get_events(&self, digest: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Query events matching a filter with optional limit.
    ///
    /// `filter` — a Sui `EventFilter` object, e.g.:
    /// ```json
    /// {"MoveEventType": "0xpackage::module::EventName"}
    /// {"Sender": "0xabc..."}
    /// {"Transaction": "digest_string"}
    /// {"Package": "0xpackage_id"}
    /// {"TimeRange": {"startTime": 1680000000000, "endTime": 1680100000000}}
    /// ```
    ///
    /// `limit` — maximum number of events to return (default 50, max 100).
    ///
    /// Returns array of event objects. Each event has `id`, `packageId`,
    /// `transactionModule`, `sender`, `type_`, `parsedJson`, `bcs`.
    async fn query_events(
        &self,
        filter: Value,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError>;

    /// Get dynamic fields of a shared/owned object.
    ///
    /// Useful for reading DEX pool state, shared registries, and other
    /// objects that use the `dynamic_field` Move module.
    ///
    /// Returns array of `DynamicFieldInfo` objects with `name`, `type_`,
    /// `objectType`, `objectId`, `version`, and `digest`.
    async fn get_dynamic_fields(&self, parent_id: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Get the latest checkpoint sequence number.
    ///
    /// Equivalent to block height on other chains. Checkpoints are finalized
    /// bundles of Sui transactions. Returns the full checkpoint object including
    /// `sequenceNumber`, `digest`, `timestampMs`, and `transactions`.
    async fn get_latest_checkpoint(&self) -> Result<Value, ExchangeError>;

    /// Get the total number of transaction blocks on the network.
    ///
    /// Uses `sui_getTotalTransactionBlocks`. Useful for monitoring indexing
    /// progress and network activity.
    async fn get_total_transaction_blocks(&self) -> Result<u64, ExchangeError>;

    /// Get the chain identifier string for this Sui network.
    ///
    /// Uses `sui_getChainIdentifier`. Returns the chain's genesis checkpoint
    /// digest prefix, e.g. `"35834a8a"` for mainnet.
    ///
    /// Useful for verifying that the provider is connected to the expected
    /// network without hardcoding network-specific constants.
    async fn get_chain_identifier(&self) -> Result<String, ExchangeError>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUI PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete Sui chain provider using raw JSON-RPC over HTTP.
///
/// One `SuiProvider` per RPC endpoint is sufficient. Multiple connectors
/// targeting the same Sui network (e.g. DeepBook and Turbos on mainnet) can
/// share a single instance via `Arc<SuiProvider>`.
///
/// ## Construction
///
/// ```rust,ignore
/// // Public RPC endpoints
/// let mainnet = SuiProvider::mainnet();
/// let testnet = SuiProvider::testnet();
///
/// // Custom RPC
/// let provider = SuiProvider::new("https://my-sui-node.example.com", "mainnet");
/// ```
pub struct SuiProvider {
    /// JSON-RPC endpoint URL
    rpc_url: String,
    /// Shared HTTP client for all JSON-RPC calls
    client: reqwest::Client,
    /// Network identifier: "mainnet", "testnet", or "devnet"
    network: String,
    /// Monotonically increasing JSON-RPC request ID counter
    request_id: std::sync::atomic::AtomicU64,
}

impl SuiProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a provider pointing at a custom JSON-RPC endpoint.
    ///
    /// `network` should be `"mainnet"`, `"testnet"`, or `"devnet"`. The value
    /// is stored for display purposes only — it does not affect RPC calls.
    pub fn new(rpc_url: impl Into<String>, network: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            client: reqwest::Client::new(),
            network: network.into(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Sui mainnet using the public Mysten Labs full-node RPC.
    pub fn mainnet() -> Self {
        Self::new("https://fullnode.mainnet.sui.io:443", "mainnet")
    }

    /// Sui testnet using the public Mysten Labs full-node RPC.
    pub fn testnet() -> Self {
        Self::new("https://fullnode.testnet.sui.io:443", "testnet")
    }

    /// Sui devnet using the public Mysten Labs full-node RPC.
    pub fn devnet() -> Self {
        Self::new("https://fullnode.devnet.sui.io:443", "devnet")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal JSON-RPC helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Get the next request ID (monotonically increasing).
    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Execute a Sui JSON-RPC call and return the `"result"` field.
    ///
    /// Returns [`ExchangeError::Network`] on transport failures,
    /// [`ExchangeError::Parse`] if the response cannot be decoded,
    /// and [`ExchangeError::InvalidRequest`] if the node returns a JSON-RPC
    /// `"error"` object.
    async fn rpc_call(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, ExchangeError> {
        let id = self.next_id();
        let body = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("{}: {}", method, e)))?;

        let raw: Value = response
            .json()
            .await
            .map_err(|e| {
                ExchangeError::Parse(format!("{}: failed to parse response: {}", method, e))
            })?;

        // JSON-RPC error object takes priority
        if let Some(err_obj) = raw.get("error") {
            let code = err_obj.get("code").and_then(Value::as_i64).unwrap_or(-1);
            let msg = err_obj
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown RPC error");
            return Err(ExchangeError::InvalidRequest(format!(
                "{}: RPC error {}: {}",
                method, code, msg
            )));
        }

        raw.get("result").cloned().ok_or_else(|| {
            ExchangeError::Parse(format!("{}: missing 'result' in response", method))
        })
    }

    /// Helper: extract a JSON array from an RPC result's `"data"` field.
    ///
    /// Many Sui paginated endpoints wrap their results in `{ "data": [...], "nextCursor": ... }`.
    fn extract_data_array(method: &str, result: Value) -> Result<Vec<Value>, ExchangeError> {
        // Some endpoints return the array directly, others wrap in { data: [...] }
        if let Some(data) = result.get("data") {
            return data
                .as_array()
                .cloned()
                .ok_or_else(|| {
                    ExchangeError::Parse(format!("{}: 'data' field is not an array", method))
                });
        }
        result.as_array().cloned().ok_or_else(|| {
            ExchangeError::Parse(format!(
                "{}: expected array or object with 'data' field, got: {}",
                method,
                result
            ))
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for SuiProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Sui {
            network: self.network.clone(),
        }
    }

    /// Broadcast a pre-signed Sui transaction block.
    ///
    /// `tx_bytes` must be a UTF-8 JSON payload with the following shape:
    /// ```json
    /// {
    ///   "tx_bytes": "<base64 BCS-encoded TransactionData>",
    ///   "signatures": ["<base64 serialized Signature>", ...]
    /// }
    /// ```
    ///
    /// Uses `sui_executeTransactionBlock` with
    /// `showEffects: true, showObjectChanges: true`.
    ///
    /// Returns the transaction digest on success.
    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        let payload: Value = serde_json::from_slice(tx_bytes).map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "broadcast_tx: tx_bytes must be a JSON payload {{\"tx_bytes\": \"...\", \"signatures\": [...]}}: {}",
                e
            ))
        })?;

        let tx_b64 = payload
            .get("tx_bytes")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExchangeError::InvalidRequest(
                    "broadcast_tx: missing 'tx_bytes' field in JSON payload".to_string(),
                )
            })?;

        let signatures = payload
            .get("signatures")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| {
                ExchangeError::InvalidRequest(
                    "broadcast_tx: missing 'signatures' array in JSON payload".to_string(),
                )
            })?;

        let options = json!({
            "showEffects": true,
            "showObjectChanges": true,
        });

        let result = self
            .rpc_call(
                "sui_executeTransactionBlock",
                json!([tx_b64, signatures, options, "WaitForLocalExecution"]),
            )
            .await?;

        result
            .get("digest")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "sui_executeTransactionBlock: missing 'digest' in result".to_string(),
                )
            })
    }

    /// Get the latest checkpoint sequence number.
    ///
    /// Sui uses checkpoints as the equivalent of block numbers.
    /// Uses `sui_getLatestCheckpointSequenceNumber`.
    async fn get_height(&self) -> Result<u64, ExchangeError> {
        let result = self
            .rpc_call("sui_getLatestCheckpointSequenceNumber", json!([]))
            .await?;

        // The result is a string-encoded u64 (Sui uses strings for large integers)
        match result.as_str() {
            Some(s) => s.parse::<u64>().map_err(|e| {
                ExchangeError::Parse(format!(
                    "sui_getLatestCheckpointSequenceNumber: failed to parse '{}' as u64: {}",
                    s, e
                ))
            }),
            None => result.as_u64().ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "sui_getLatestCheckpointSequenceNumber: expected string or integer, got: {}",
                    result
                ))
            }),
        }
    }

    /// Not applicable for Sui — Sui uses an object-based model, not account nonces.
    ///
    /// Always returns [`ExchangeError::UnsupportedOperation`].
    async fn get_nonce(&self, _address: &str) -> Result<u64, ExchangeError> {
        Err(ExchangeError::UnsupportedOperation(
            "Sui does not use account nonces; use object-based transactions instead".to_string(),
        ))
    }

    /// Get the native SUI balance for an address in MIST (1 SUI = 1_000_000_000 MIST).
    ///
    /// Uses `suix_getBalance` with coin type `"0x2::sui::SUI"`.
    /// Returns the total balance as a decimal string in MIST.
    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        let result = self
            .rpc_call(
                "suix_getBalance",
                json!([address, "0x2::sui::SUI"]),
            )
            .await?;

        result
            .get("totalBalance")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "suix_getBalance: missing 'totalBalance' string in result".to_string(),
                )
            })
    }

    /// Get transaction status by digest.
    ///
    /// Uses `sui_getTransactionBlock` with `showEffects: true`.
    /// Maps Sui execution status to [`TxStatus`]:
    /// - `"success"` → [`TxStatus::Confirmed`] with the checkpoint as `block`
    /// - `"failure"` → [`TxStatus::Failed`] with the error message
    /// - not found (RPC error) → [`TxStatus::NotFound`]
    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        let options = json!({"showEffects": true});
        let result = self
            .rpc_call("sui_getTransactionBlock", json!([tx_hash, options]))
            .await;

        match result {
            Err(ExchangeError::InvalidRequest(msg))
                if msg.contains("not found")
                    || msg.contains("Could not find")
                    || msg.contains("-32602")
                    || msg.contains("-32000") =>
            {
                return Ok(TxStatus::NotFound);
            }
            Err(e) => return Err(e),
            Ok(tx) => {
                let effects = tx.get("effects");
                let status_str = effects
                    .and_then(|e| e.get("status"))
                    .and_then(|s| s.get("status"))
                    .and_then(Value::as_str)
                    .unwrap_or("");

                let error_str = effects
                    .and_then(|e| e.get("status"))
                    .and_then(|s| s.get("error"))
                    .and_then(Value::as_str)
                    .unwrap_or("transaction failed")
                    .to_string();

                let checkpoint = tx
                    .get("checkpoint")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| tx.get("checkpoint").and_then(Value::as_u64))
                    .unwrap_or(0);

                match status_str {
                    "success" => Ok(TxStatus::Confirmed { block: checkpoint }),
                    "failure" => Ok(TxStatus::Failed { reason: error_str }),
                    // Transaction known but not yet checkpointed
                    _ => Ok(TxStatus::Pending),
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SuiChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl SuiChain for SuiProvider {
    async fn get_object(&self, object_id: &str) -> Result<Value, ExchangeError> {
        let options = json!({
            "showContent": true,
            "showOwner": true,
            "showType": true,
            "showDisplay": false,
            "showBcs": false,
        });

        let result = self
            .rpc_call("sui_getObject", json!([object_id, options]))
            .await?;

        // The result may be { "data": {...} } or contain an "error" field
        if let Some(err) = result.get("error") {
            let code = err.get("code").and_then(Value::as_str).unwrap_or("unknown");
            let msg = err
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("object not found or deleted");
            return Err(ExchangeError::InvalidRequest(format!(
                "sui_getObject {}: {} — {}",
                object_id, code, msg
            )));
        }

        // Return the data field if present, otherwise the whole result
        Ok(result
            .get("data")
            .cloned()
            .unwrap_or(result))
    }

    async fn get_owned_objects(&self, owner: &str) -> Result<Vec<Value>, ExchangeError> {
        let query = json!({
            "filter": null,
            "options": {
                "showType": true,
                "showContent": false,
                "showOwner": false,
            }
        });

        let result = self
            .rpc_call("suix_getOwnedObjects", json!([owner, query, null, 50]))
            .await?;

        Self::extract_data_array("suix_getOwnedObjects", result)
    }

    async fn get_all_balances(&self, owner: &str) -> Result<Vec<Value>, ExchangeError> {
        let result = self
            .rpc_call("suix_getAllBalances", json!([owner]))
            .await?;

        result.as_array().cloned().ok_or_else(|| {
            ExchangeError::Parse("suix_getAllBalances: expected array result".to_string())
        })
    }

    async fn get_coins(
        &self,
        owner: &str,
        coin_type: Option<&str>,
    ) -> Result<Vec<Value>, ExchangeError> {
        let coin_type_value: Value = match coin_type {
            Some(ct) => Value::String(ct.to_string()),
            None => Value::String("0x2::sui::SUI".to_string()),
        };

        let result = self
            .rpc_call("suix_getCoins", json!([owner, coin_type_value, null, 50]))
            .await?;

        Self::extract_data_array("suix_getCoins", result)
    }

    async fn dev_inspect(
        &self,
        sender: &str,
        move_call: Value,
    ) -> Result<Value, ExchangeError> {
        // devInspectTransactionBlock takes:
        //   sender: address
        //   transaction: TransactionBlockKind (PTB JSON)
        //   gasPrice: optional u64 (as string)
        //   epoch: optional u64 (as string)
        self.rpc_call(
            "sui_devInspectTransactionBlock",
            json!([sender, move_call, null, null]),
        )
        .await
    }

    async fn get_transaction_block(&self, digest: &str) -> Result<Value, ExchangeError> {
        let options = json!({
            "showEffects": true,
            "showEvents": true,
            "showObjectChanges": true,
            "showInput": true,
            "showBalanceChanges": false,
        });

        self.rpc_call("sui_getTransactionBlock", json!([digest, options]))
            .await
    }

    async fn query_transaction_blocks(
        &self,
        filter: Value,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError> {
        let limit_val = limit.unwrap_or(50).min(100);

        let query = json!({
            "filter": filter,
            "options": {
                "showEffects": true,
                "showEvents": false,
            }
        });

        let result = self
            .rpc_call(
                "suix_queryTransactionBlocks",
                json!([query, null, limit_val, false]),
            )
            .await?;

        Self::extract_data_array("suix_queryTransactionBlocks", result)
    }

    async fn get_events(&self, digest: &str) -> Result<Vec<Value>, ExchangeError> {
        let result = self
            .rpc_call("sui_getEvents", json!([digest]))
            .await?;

        result.as_array().cloned().ok_or_else(|| {
            ExchangeError::Parse("sui_getEvents: expected array result".to_string())
        })
    }

    async fn query_events(
        &self,
        filter: Value,
        limit: Option<u32>,
    ) -> Result<Vec<Value>, ExchangeError> {
        let limit_val = limit.unwrap_or(50).min(100);

        let result = self
            .rpc_call(
                "suix_queryEvents",
                json!([filter, null, limit_val, false]),
            )
            .await?;

        Self::extract_data_array("suix_queryEvents", result)
    }

    async fn get_dynamic_fields(&self, parent_id: &str) -> Result<Vec<Value>, ExchangeError> {
        let result = self
            .rpc_call("suix_getDynamicFields", json!([parent_id, null, 50]))
            .await?;

        Self::extract_data_array("suix_getDynamicFields", result)
    }

    async fn get_latest_checkpoint(&self) -> Result<Value, ExchangeError> {
        self.rpc_call("sui_getLatestCheckpointSequenceNumber", json!([]))
            .await
            .and_then(|seq| {
                // Fetch the full checkpoint by sequence number
                let seq_str = match seq.as_str() {
                    Some(s) => s.to_string(),
                    None => seq.to_string(),
                };
                // Return the sequence as a simple Value — for the full checkpoint
                // object use rpc_call("sui_getCheckpoint", ...) directly
                Ok(json!({ "sequenceNumber": seq_str }))
            })
    }

    async fn get_total_transaction_blocks(&self) -> Result<u64, ExchangeError> {
        let result = self
            .rpc_call("sui_getTotalTransactionBlocks", json!([]))
            .await?;

        // Returns a string-encoded u64
        match result.as_str() {
            Some(s) => s.parse::<u64>().map_err(|e| {
                ExchangeError::Parse(format!(
                    "sui_getTotalTransactionBlocks: failed to parse '{}' as u64: {}",
                    s, e
                ))
            }),
            None => result.as_u64().ok_or_else(|| {
                ExchangeError::Parse(format!(
                    "sui_getTotalTransactionBlocks: expected string or integer, got: {}",
                    result
                ))
            }),
        }
    }

    async fn get_chain_identifier(&self) -> Result<String, ExchangeError> {
        let result = self
            .rpc_call("sui_getChainIdentifier", json!([]))
            .await?;

        result
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "sui_getChainIdentifier: expected string result".to_string(),
                )
            })
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
        let provider = SuiProvider::mainnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Sui {
                network: "mainnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_testnet() {
        let provider = SuiProvider::testnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Sui {
                network: "testnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_devnet() {
        let provider = SuiProvider::devnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Sui {
                network: "devnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_name() {
        let p = SuiProvider::mainnet();
        assert_eq!(p.chain_family().name(), "sui:mainnet");

        let t = SuiProvider::testnet();
        assert_eq!(t.chain_family().name(), "sui:testnet");
    }

    #[test]
    fn test_is_sui() {
        let family = ChainFamily::Sui {
            network: "mainnet".to_string(),
        };
        assert!(family.is_sui("mainnet"));
        assert!(!family.is_sui("testnet"));
    }

    #[test]
    fn test_rpc_url_mainnet() {
        let p = SuiProvider::mainnet();
        assert_eq!(p.rpc_url, "https://fullnode.mainnet.sui.io:443");
        assert_eq!(p.network, "mainnet");
    }

    #[test]
    fn test_rpc_url_testnet() {
        let p = SuiProvider::testnet();
        assert_eq!(p.rpc_url, "https://fullnode.testnet.sui.io:443");
        assert_eq!(p.network, "testnet");
    }

    #[test]
    fn test_custom_rpc() {
        let p = SuiProvider::new("https://my-sui-rpc.example.com", "mainnet");
        assert_eq!(p.rpc_url, "https://my-sui-rpc.example.com");
        assert_eq!(p.network, "mainnet");
    }

    #[test]
    fn test_request_id_increments() {
        let p = SuiProvider::mainnet();
        let id1 = p.next_id();
        let id2 = p.next_id();
        let id3 = p.next_id();
        assert_eq!(id2, id1 + 1);
        assert_eq!(id3, id2 + 1);
    }

    #[tokio::test]
    async fn test_get_nonce_unsupported() {
        let p = SuiProvider::mainnet();
        let result = p.get_nonce("0xabc").await;
        assert!(matches!(result, Err(ExchangeError::UnsupportedOperation(_))));
    }

    #[test]
    fn test_extract_data_array_with_data_field() {
        let result = json!({
            "data": [{"id": "0x1"}, {"id": "0x2"}],
            "nextCursor": null,
            "hasNextPage": false
        });
        let arr = SuiProvider::extract_data_array("test", result).unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_extract_data_array_direct_array() {
        let result = json!([{"id": "0x1"}, {"id": "0x2"}, {"id": "0x3"}]);
        let arr = SuiProvider::extract_data_array("test", result).unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_extract_data_array_error_on_invalid() {
        let result = json!({"foo": "bar"});
        let err = SuiProvider::extract_data_array("test_method", result).unwrap_err();
        match err {
            ExchangeError::Parse(msg) => assert!(msg.contains("test_method")),
            other => panic!("expected Parse error, got {:?}", other),
        }
    }

    #[test]
    fn test_chain_family_not_evm() {
        let family = ChainFamily::Sui {
            network: "mainnet".to_string(),
        };
        assert!(!family.is_evm(1));
        assert!(!family.is_evm(42161));
    }
}
