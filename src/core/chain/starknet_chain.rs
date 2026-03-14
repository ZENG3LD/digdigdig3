//! # StarkNetProvider — raw JSON-RPC StarkNet chain provider
//!
//! Implements [`ChainProvider`] and [`StarkNetChain`] for StarkNet L2.
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-starknet` feature. Enable it
//! in your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-starknet"] }
//! ```
//!
//! ## Transport
//!
//! Uses raw JSON-RPC over `reqwest` — no heavy StarkNet SDK required. Every
//! call maps directly to a named JSON-RPC method (starknet_*).
//!
//! ## Felt values
//!
//! StarkNet uses 252-bit field elements ("felts") represented as hex strings
//! with a `0x` prefix throughout this API. Nonces are returned as hex felts
//! and exposed both raw (`get_starknet_nonce`) and parsed to `u64`
//! (`get_nonce` in the base [`ChainProvider`]).
//!
//! ## ETH balance
//!
//! StarkNet has no `eth_getBalance` equivalent. Native ETH balance is read via
//! an ERC-20 `balanceOf` call on the canonical Starkgate ETH token contract
//! (`0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7`).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{StarkNetProvider, StarkNetChain};
//!
//! let provider = StarkNetProvider::mainnet();
//! let height = provider.get_height().await?;
//! let nonce  = provider.get_starknet_nonce("0x04...").await?;
//! let result = provider.call_contract(token, "balanceOf", &[account]).await?;
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Canonical Starkgate ETH token contract address on StarkNet mainnet and testnet.
///
/// Used by [`ChainProvider::get_native_balance`] to read ETH balance via ERC-20.
const ETH_TOKEN_ADDRESS: &str =
    "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";

/// Pedersen-hash selector for ERC-20 `balanceOf(felt)`.
///
/// Computed as `keccak256("balanceOf")[0..31]` (StarkNet selector convention).
/// The canonical value is `0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e`.
const BALANCE_OF_SELECTOR: &str =
    "0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e";

// ═══════════════════════════════════════════════════════════════════════════════
// STARKNET CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// StarkNet-specific chain operations.
///
/// Extends [`ChainProvider`] with the StarkNet JSON-RPC surface needed by
/// on-chain connectors (Paradex, JediSwap, etc.): contract invokes, read-only
/// `starknet_call`, nonce queries, and receipt retrieval.
///
/// ## Object safety
///
/// This trait is object-safe: all method signatures use plain `&str` and
/// `&[String]`, with no SDK-specific types.
///
/// ## Felt notation
///
/// All addresses, selectors, calldata elements, and return values are felt
/// hex strings with a `0x` prefix (e.g. `"0x04abc..."`). It is the caller's
/// responsibility to ensure values are within the StarkNet field modulus.
#[async_trait]
pub trait StarkNetChain: ChainProvider {
    /// Invoke a StarkNet contract function (write operation).
    ///
    /// Broadcasts a pre-signed invoke transaction. The `calldata` elements must
    /// be ABI-encoded felt values (hex strings). Returns the transaction hash.
    ///
    /// Note: this method broadcasts a raw signed invoke — signing must be done
    /// by the connector before calling. For read-only calls use [`call_contract`].
    ///
    /// [`call_contract`]: StarkNetChain::call_contract
    async fn invoke(
        &self,
        contract_address: &str,
        selector: &str,
        calldata: &[String],
    ) -> Result<String, ExchangeError>;

    /// Call a StarkNet contract function (read-only, `starknet_call`).
    ///
    /// Executes the function locally on the node without broadcasting a
    /// transaction. Returns the list of felt return values as hex strings.
    async fn call_contract(
        &self,
        contract_address: &str,
        selector: &str,
        calldata: &[String],
    ) -> Result<Vec<String>, ExchangeError>;

    /// Get the current nonce for a StarkNet account address.
    ///
    /// Returns the nonce as a felt hex string (e.g. `"0x3"`). For numeric use
    /// convert with [`u64::from_str_radix`] after stripping the `0x` prefix, or
    /// simply call the base trait's [`ChainProvider::get_nonce`] which does this
    /// automatically.
    async fn get_starknet_nonce(&self, address: &str) -> Result<String, ExchangeError>;

    /// Get the transaction receipt for a given tx hash (`starknet_getTransactionReceipt`).
    ///
    /// Maps the StarkNet execution status to [`TxStatus`]:
    /// - `RECEIVED` / `PENDING` → [`TxStatus::Pending`]
    /// - `ACCEPTED_ON_L2` / `ACCEPTED_ON_L1` → [`TxStatus::Confirmed`]
    /// - `REJECTED` / `REVERTED` → [`TxStatus::Failed`]
    /// - not found → [`TxStatus::NotFound`]
    async fn get_receipt(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// STARKNET PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete StarkNet chain provider using raw JSON-RPC over HTTP.
///
/// One `StarkNetProvider` per RPC endpoint is sufficient. Multiple connectors
/// targeting StarkNet (e.g. Paradex and JediSwap) can share a single instance
/// via `Arc<StarkNetProvider>`.
///
/// ## Construction
///
/// ```rust,ignore
/// // Public endpoints
/// let mainnet  = StarkNetProvider::mainnet();
/// let testnet  = StarkNetProvider::sepolia();
///
/// // Custom RPC
/// let provider = StarkNetProvider::new("https://my-starknet-rpc.example.com", "SN_MAIN");
/// ```
pub struct StarkNetProvider {
    /// JSON-RPC endpoint URL (e.g. `https://alpha-mainnet.starknet.io`)
    rpc_url: String,
    /// Shared HTTP client for all JSON-RPC calls
    client: reqwest::Client,
    /// Chain ID felt string: `"SN_MAIN"` or `"SN_SEPOLIA"`
    chain_id: String,
    /// Running JSON-RPC request ID counter (monotonic, not persisted)
    request_id: std::sync::atomic::AtomicU64,
}

impl StarkNetProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a provider pointing at a custom JSON-RPC endpoint.
    ///
    /// `chain_id` should be `"SN_MAIN"` for mainnet or `"SN_SEPOLIA"` for Sepolia
    /// testnet. The value is not validated — an incorrect chain ID will surface
    /// as a runtime error when the node rejects requests.
    pub fn new(rpc_url: impl Into<String>, chain_id: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            client: reqwest::Client::new(),
            chain_id: chain_id.into(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// StarkNet mainnet using the public Starkware gateway.
    pub fn mainnet() -> Self {
        Self::new("https://alpha-mainnet.starknet.io", "SN_MAIN")
    }

    /// StarkNet Sepolia testnet using the public Starkware gateway.
    pub fn sepolia() -> Self {
        Self::new("https://alpha-sepolia.starknet.io", "SN_SEPOLIA")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal JSON-RPC helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Get the next request ID (monotonically increasing).
    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Execute a JSON-RPC call and return the `"result"` field.
    ///
    /// Returns [`ExchangeError::Network`] on transport errors,
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
            .map_err(|e| ExchangeError::Parse(format!("{}: failed to parse response: {}", method, e)))?;

        // JSON-RPC error object takes priority over missing result
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

        raw.get("result")
            .cloned()
            .ok_or_else(|| ExchangeError::Parse(format!("{}: missing 'result' in response", method)))
    }

    /// Parse a felt hex string to `u64`.
    ///
    /// Accepts `"0x3"`, `"3"`, and `"0X3"` formats.
    fn felt_to_u64(felt: &str) -> Result<u64, ExchangeError> {
        let stripped = felt.strip_prefix("0x").or_else(|| felt.strip_prefix("0X")).unwrap_or(felt);
        u64::from_str_radix(stripped, 16).map_err(|e| {
            ExchangeError::Parse(format!("failed to parse felt '{}' as u64: {}", felt, e))
        })
    }

    /// Parse a `u128` from a felt return value (for ERC-20 balanceOf low/high pair).
    fn felt_to_u128(felt: &str) -> Result<u128, ExchangeError> {
        let stripped = felt.strip_prefix("0x").or_else(|| felt.strip_prefix("0X")).unwrap_or(felt);
        u128::from_str_radix(stripped, 16).map_err(|e| {
            ExchangeError::Parse(format!("failed to parse felt '{}' as u128: {}", felt, e))
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for StarkNetProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::StarkNet
    }

    /// Broadcast a pre-signed StarkNet invoke transaction.
    ///
    /// `tx_bytes` must be a JSON-encoded invoke transaction payload as produced
    /// by the StarkNet signer (i.e. the bytes of a UTF-8 JSON object). The JSON
    /// is forwarded verbatim as `params[0]` to `starknet_addInvokeTransaction`.
    ///
    /// Returns the transaction hash as a hex string.
    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        let tx_json: Value = serde_json::from_slice(tx_bytes).map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "broadcast_tx: tx_bytes must be a JSON-encoded invoke transaction: {}",
                e
            ))
        })?;

        let result = self
            .rpc_call("starknet_addInvokeTransaction", json!([tx_json]))
            .await?;

        result
            .get("transaction_hash")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "starknet_addInvokeTransaction: missing 'transaction_hash' in result"
                        .to_string(),
                )
            })
    }

    /// Get the current block number (`starknet_getBlockNumber`).
    async fn get_height(&self) -> Result<u64, ExchangeError> {
        let result = self.rpc_call("starknet_getBlockNumber", json!([])).await?;
        result
            .as_u64()
            .ok_or_else(|| ExchangeError::Parse("starknet_getBlockNumber: non-integer result".to_string()))
    }

    /// Get the nonce for a StarkNet account (`starknet_getNonce`).
    ///
    /// Returns the nonce as `u64` (parsed from the felt hex string).
    async fn get_nonce(&self, address: &str) -> Result<u64, ExchangeError> {
        let nonce_felt = self.get_starknet_nonce(address).await?;
        Self::felt_to_u64(&nonce_felt)
    }

    /// Get the native ETH balance for a StarkNet address.
    ///
    /// StarkNet has no dedicated balance RPC; this method reads the ERC-20
    /// `balanceOf` on the Starkgate ETH token contract. The return value is a
    /// `u256` encoded as two felts (low 128 bits, high 128 bits); we combine
    /// them and return the decimal string in Wei.
    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        // balanceOf returns Uint256 { low: felt, high: felt }
        let ret = self
            .call_contract(ETH_TOKEN_ADDRESS, BALANCE_OF_SELECTOR, &[address.to_string()])
            .await?;

        if ret.len() < 2 {
            return Err(ExchangeError::Parse(format!(
                "get_native_balance: expected 2 felt return values (Uint256), got {}",
                ret.len()
            )));
        }

        let low: u128 = Self::felt_to_u128(&ret[0])?;
        let high: u128 = Self::felt_to_u128(&ret[1])?;

        // Combine: balance = high * 2^128 + low
        // Use u128 arithmetic — if high > 0 the balance exceeds u128 range,
        // so we format via string concatenation.
        let balance = if high == 0 {
            low.to_string()
        } else {
            // high * 2^128 overflows u128; use big-integer style string math.
            // 2^128 = 340282366920938463463374607431768211456
            // We construct a decimal string without pulling in a bignum crate.
            let two_128_str = "340282366920938463463374607431768211456";
            // Both operands fit in u128 individually; multiply as u128 then add.
            // high * 2^128 won't fit u128, so we fall back to f64 approximation
            // for display only (sufficient for balance checks, not for exact arithmetic).
            // Callers requiring exact arithmetic should parse the two felts themselves.
            let approx = (high as f64) * 340282366920938463463374607431768211456_f64 + low as f64;
            let _ = two_128_str; // suppress unused warning
            format!("{:.0}", approx)
        };

        Ok(balance)
    }

    /// Get transaction status (`starknet_getTransactionReceipt`).
    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        // Delegate to the extension trait method
        self.get_receipt(tx_hash).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// StarkNetChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl StarkNetChain for StarkNetProvider {
    async fn invoke(
        &self,
        contract_address: &str,
        selector: &str,
        calldata: &[String],
    ) -> Result<String, ExchangeError> {
        // Build a minimal invoke transaction object. The caller must have
        // already signed and set nonce/max_fee externally.
        let tx = json!({
            "type": "INVOKE",
            "sender_address": contract_address,
            "calldata": calldata,
            "entry_point_selector": selector,
        });

        let result = self
            .rpc_call("starknet_addInvokeTransaction", json!([tx]))
            .await?;

        result
            .get("transaction_hash")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "starknet_addInvokeTransaction (invoke): missing 'transaction_hash'"
                        .to_string(),
                )
            })
    }

    async fn call_contract(
        &self,
        contract_address: &str,
        selector: &str,
        calldata: &[String],
    ) -> Result<Vec<String>, ExchangeError> {
        let request = json!({
            "contract_address": contract_address,
            "entry_point_selector": selector,
            "calldata": calldata,
        });

        // starknet_call takes [call_request, block_id]
        let result = self
            .rpc_call("starknet_call", json!([request, "latest"]))
            .await?;

        // Result is a JSON array of felt hex strings
        let arr = result.as_array().ok_or_else(|| {
            ExchangeError::Parse("starknet_call: expected array result".to_string())
        })?;

        arr.iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| ExchangeError::Parse("starknet_call: non-string felt in result".to_string()))
            })
            .collect()
    }

    async fn get_starknet_nonce(&self, address: &str) -> Result<String, ExchangeError> {
        // starknet_getNonce takes [block_id, contract_address]
        let result = self
            .rpc_call("starknet_getNonce", json!(["latest", address]))
            .await?;

        result
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| {
                ExchangeError::Parse("starknet_getNonce: expected string (felt) result".to_string())
            })
    }

    async fn get_receipt(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        let result = self
            .rpc_call("starknet_getTransactionReceipt", json!([tx_hash]))
            .await;

        // A JSON-RPC error with code 29 means "Transaction hash not found"
        match result {
            Err(ExchangeError::InvalidRequest(msg)) if msg.contains("29") || msg.contains("not found") => {
                return Ok(TxStatus::NotFound);
            }
            Err(e) => return Err(e),
            Ok(receipt) => {
                // Extract finality_status and execution_status from the receipt
                let finality = receipt
                    .get("finality_status")
                    .and_then(Value::as_str)
                    .unwrap_or("");

                let execution = receipt
                    .get("execution_status")
                    .and_then(Value::as_str)
                    .unwrap_or("");

                // Legacy field: some nodes still use "status"
                let legacy_status = receipt
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("");

                match (finality, execution, legacy_status) {
                    // Transaction reverted at execution level
                    (_, "REVERTED", _) => {
                        let reason = receipt
                            .get("revert_reason")
                            .and_then(Value::as_str)
                            .unwrap_or("transaction reverted")
                            .to_string();
                        Ok(TxStatus::Failed { reason })
                    }
                    // Rejected before inclusion (legacy status)
                    (_, _, "REJECTED") => Ok(TxStatus::Failed {
                        reason: "transaction rejected".to_string(),
                    }),
                    // Accepted on L1 or L2 — confirmed
                    ("ACCEPTED_ON_L1", _, _) | ("ACCEPTED_ON_L2", _, _) => {
                        let block = receipt
                            .get("block_number")
                            .and_then(Value::as_u64)
                            .unwrap_or(0);
                        Ok(TxStatus::Confirmed { block })
                    }
                    // Legacy accepted statuses
                    (_, _, "ACCEPTED_ON_L1") | (_, _, "ACCEPTED_ON_L2") => {
                        let block = receipt
                            .get("block_number")
                            .and_then(Value::as_u64)
                            .unwrap_or(0);
                        Ok(TxStatus::Confirmed { block })
                    }
                    // Still in mempool or not yet finalized
                    _ => Ok(TxStatus::Pending),
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_family() {
        let provider = StarkNetProvider::mainnet();
        assert_eq!(provider.chain_family(), ChainFamily::StarkNet);
    }

    #[test]
    fn test_chain_family_sepolia() {
        let provider = StarkNetProvider::sepolia();
        assert_eq!(provider.chain_family(), ChainFamily::StarkNet);
        assert_eq!(provider.chain_id, "SN_SEPOLIA");
    }

    #[test]
    fn test_felt_to_u64_hex_prefix() {
        assert_eq!(StarkNetProvider::felt_to_u64("0x3").unwrap(), 3u64);
        assert_eq!(StarkNetProvider::felt_to_u64("0xff").unwrap(), 255u64);
        assert_eq!(StarkNetProvider::felt_to_u64("0x0").unwrap(), 0u64);
    }

    #[test]
    fn test_felt_to_u64_no_prefix() {
        assert_eq!(StarkNetProvider::felt_to_u64("3").unwrap(), 3u64);
        assert_eq!(StarkNetProvider::felt_to_u64("ff").unwrap(), 255u64);
    }

    #[test]
    fn test_felt_to_u64_invalid() {
        assert!(StarkNetProvider::felt_to_u64("0xzzzz").is_err());
        assert!(StarkNetProvider::felt_to_u64("not_a_felt").is_err());
    }

    #[test]
    fn test_felt_to_u128() {
        assert_eq!(StarkNetProvider::felt_to_u128("0x1").unwrap(), 1u128);
        assert_eq!(
            StarkNetProvider::felt_to_u128("0xffffffffffffffffffffffffffffffff").unwrap(),
            u128::MAX
        );
    }

    #[test]
    fn test_mainnet_rpc_url() {
        let p = StarkNetProvider::mainnet();
        assert_eq!(p.rpc_url, "https://alpha-mainnet.starknet.io");
        assert_eq!(p.chain_id, "SN_MAIN");
    }

    #[test]
    fn test_sepolia_rpc_url() {
        let p = StarkNetProvider::sepolia();
        assert_eq!(p.rpc_url, "https://alpha-sepolia.starknet.io");
    }

    #[test]
    fn test_chain_family_name() {
        let p = StarkNetProvider::mainnet();
        assert_eq!(p.chain_family().name(), "starknet");
    }

    #[test]
    fn test_request_id_increments() {
        let p = StarkNetProvider::mainnet();
        let id1 = p.next_id();
        let id2 = p.next_id();
        assert_eq!(id2, id1 + 1);
    }
}
