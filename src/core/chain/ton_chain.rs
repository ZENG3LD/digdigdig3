//! # TonProvider — pure HTTP TON chain provider
//!
//! Implements [`ChainProvider`] and [`TonChain`] for the TON (Telegram Open
//! Network) blockchain using the TON Center REST API v2.
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-ton` feature. Enable it
//! in your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-ton"] }
//! ```
//!
//! ## Transport
//!
//! Uses raw REST over `reqwest` — no tonlib-rs or C++ FFI required.
//! Every call maps directly to a named TON Center REST endpoint.
//!
//! ## Addresses
//!
//! TON addresses use a base64url-encoded format (EQ... for user-friendly,
//! raw 0: form also accepted). This provider passes addresses through as-is
//! to the API.
//!
//! ## Balance units
//!
//! `get_native_balance` returns the balance in **nanoTON** (1 TON = 1e9 nanoTON)
//! as a decimal string.
//!
//! ## API key
//!
//! TON Center imposes a rate limit of ~1 req/s without an API key.
//! Provide an API key via [`TonProvider::with_api_key`] to increase this limit.
//! The key is passed as `?api_key=...` on every request.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{TonProvider, TonChain};
//!
//! let provider = TonProvider::mainnet();
//! let height = provider.get_height().await?;
//! let info = provider.get_address_information("EQD...").await?;
//! let masterchain = provider.get_masterchain_info().await?;
//! ```

use async_trait::async_trait;
use serde_json::Value;

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Public TON Center mainnet API base URL (v2).
const TONCENTER_MAINNET: &str = "https://toncenter.com/api/v2";

/// Public TON Center testnet API base URL (v2).
const TONCENTER_TESTNET: &str = "https://testnet.toncenter.com/api/v2";

// ═══════════════════════════════════════════════════════════════════════════════
// TON CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// TON-specific chain operations.
///
/// Extends [`ChainProvider`] with the TON REST API surface needed by on-chain
/// connectors: account info, transaction history, jetton (token) operations,
/// NFT queries, smart contract reads, and block-level data.
///
/// ## Object safety
///
/// This trait is object-safe: all method signatures use plain `&str` and
/// `&[serde_json::Value]`, with no SDK-specific types.
///
/// ## Address format
///
/// TON addresses are base64url-encoded strings in the user-friendly form
/// (e.g. `"EQD..."`). Raw forms (`0:...`) are also accepted by most endpoints.
#[async_trait]
pub trait TonChain: ChainProvider {
    /// Get detailed account information (balance, code, data, state).
    ///
    /// Maps to `GET /getAddressInformation?address=...`.
    /// Returns the full JSON response including `balance`, `code`, `data`,
    /// and `state` (`"active"`, `"uninitialized"`, `"frozen"`).
    async fn get_address_information(&self, address: &str) -> Result<Value, ExchangeError>;

    /// Get transaction history for an address.
    ///
    /// Maps to `GET /getTransactions?address=...&limit=...`.
    /// Returns up to `limit` transactions in reverse chronological order.
    async fn get_transactions(
        &self,
        address: &str,
        limit: u32,
    ) -> Result<Vec<Value>, ExchangeError>;

    /// Run a get-method on a smart contract (read-only).
    ///
    /// Maps to `POST /runGetMethod` with body `{address, method, stack}`.
    /// Returns the full response including `exit_code` and `stack`.
    async fn run_get_method(
        &self,
        address: &str,
        method: &str,
        stack: &[Value],
    ) -> Result<Value, ExchangeError>;

    /// Get the jetton (token) wallet address for an owner.
    ///
    /// Calls `get_wallet_address` get-method on the jetton master contract,
    /// passing the owner address as a slice parameter.
    /// Returns the resulting jetton wallet address as a string.
    async fn get_jetton_wallet(
        &self,
        jetton_master: &str,
        owner: &str,
    ) -> Result<String, ExchangeError>;

    /// Get the jetton wallet balance.
    ///
    /// Calls the `get_wallet_data` get-method on the jetton wallet contract
    /// and returns the balance (first stack element) as a decimal string.
    async fn get_jetton_balance(&self, jetton_wallet: &str) -> Result<String, ExchangeError>;

    /// Get NFT items for a collection.
    ///
    /// Calls `GET /getNftItems?collection_address=...` and returns the items array.
    async fn get_nft_items(&self, collection: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Get masterchain info (last block seqno, state root hash, etc.).
    ///
    /// Maps to `GET /getMasterchainInfo`.
    async fn get_masterchain_info(&self) -> Result<Value, ExchangeError>;

    /// Get block header for a specific shard block.
    ///
    /// Maps to `GET /getBlockHeader?workchain=...&shard=...&seqno=...`.
    async fn get_block_header(
        &self,
        workchain: i32,
        shard: i64,
        seqno: u32,
    ) -> Result<Value, ExchangeError>;

    /// Get a configuration parameter from the TON masterchain config.
    ///
    /// Maps to `GET /getConfigParam?config_id=...`.
    /// Configuration parameters include validator sets (34), gas prices (20/21),
    /// storage prices (18), workchain descriptors (12), etc.
    async fn get_config_param(&self, param: u32) -> Result<Value, ExchangeError>;

    /// Estimate fees for sending a message to an address.
    ///
    /// Maps to `POST /estimateFee` with body `{address, body, ...}`.
    /// `body` should be a base64-encoded BOC of the message body.
    /// Returns the estimated fees breakdown.
    async fn estimate_fee(&self, address: &str, body: &str) -> Result<Value, ExchangeError>;

    /// Send a signed BOC (Bag of Cells) to the network.
    ///
    /// Maps to `POST /sendBoc` with body `{boc: "..."}`.
    /// `boc` must be a base64-encoded signed BOC.
    /// Returns the response (typically `{"ok": true}`).
    async fn send_boc(&self, boc: &str) -> Result<Value, ExchangeError>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TON PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete TON chain provider using raw REST over HTTP (no tonlib-rs / C++ FFI).
///
/// One `TonProvider` per API endpoint is sufficient. Multiple connectors
/// targeting TON can share a single instance via `Arc<TonProvider>`.
///
/// ## Construction
///
/// ```rust,ignore
/// // Public endpoints (no API key — rate-limited to ~1 req/s)
/// let mainnet = TonProvider::mainnet();
/// let testnet = TonProvider::testnet();
///
/// // With API key (higher rate limits)
/// let provider = TonProvider::mainnet().with_api_key("your_key_here");
///
/// // Custom RPC endpoint
/// let provider = TonProvider::new("https://my-toncenter.example.com/api/v2", "mainnet");
/// ```
pub struct TonProvider {
    /// API base URL (e.g. `https://toncenter.com/api/v2`)
    api_url: String,
    /// Shared HTTP client for all REST calls
    client: reqwest::Client,
    /// Optional API key for rate-limit bypass (`?api_key=...`)
    api_key: Option<String>,
    /// Network identifier: `"mainnet"` or `"testnet"`
    network: String,
}

impl TonProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a provider pointing at a custom TON Center-compatible REST endpoint.
    ///
    /// `network` should be `"mainnet"` or `"testnet"`. The value is not validated
    /// at construction time — it is used only for [`ChainFamily::Ton`] identification.
    pub fn new(api_url: impl Into<String>, network: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            client: reqwest::Client::new(),
            api_key: None,
            network: network.into(),
        }
    }

    /// TON mainnet using the public TON Center API (rate-limited without key).
    pub fn mainnet() -> Self {
        Self::new(TONCENTER_MAINNET, "mainnet")
    }

    /// TON testnet using the public TON Center testnet API.
    pub fn testnet() -> Self {
        Self::new(TONCENTER_TESTNET, "testnet")
    }

    /// Attach an API key for higher rate limits on TON Center.
    ///
    /// Returns `self` for builder-style chaining.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal HTTP helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Build a GET URL for the given endpoint with optional query parameters.
    ///
    /// Appends `?api_key=...` when an API key is set.
    fn build_get_url(&self, endpoint: &str, params: &[(&str, &str)]) -> String {
        let mut url = format!("{}/{}", self.api_url, endpoint);
        let mut first = true;

        let all_params: Vec<(&str, &str)> = if let Some(ref key) = self.api_key {
            // We need to extend params with api_key — handled below
            let _ = key; // used below via self.api_key
            params.to_vec()
        } else {
            params.to_vec()
        };

        for (k, v) in &all_params {
            if first {
                url.push('?');
                first = false;
            } else {
                url.push('&');
            }
            url.push_str(k);
            url.push('=');
            // Percent-encode the value (simple pass-through for addresses)
            url.push_str(&urlencoding::encode(v));
        }

        if let Some(ref key) = self.api_key {
            if first {
                url.push('?');
            } else {
                url.push('&');
            }
            url.push_str("api_key=");
            url.push_str(key);
        }

        url
    }

    /// Perform a GET request and return the `"result"` field from the response.
    ///
    /// TON Center wraps all responses as:
    /// ```json
    /// {"ok": true, "result": ...}
    /// ```
    /// or on error:
    /// ```json
    /// {"ok": false, "error": "...", "code": 400}
    /// ```
    async fn get(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<Value, ExchangeError> {
        let url = self.build_get_url(endpoint, params);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("{}: {}", endpoint, e)))?;

        let raw: Value = response
            .json()
            .await
            .map_err(|e| {
                ExchangeError::Parse(format!("{}: failed to parse response: {}", endpoint, e))
            })?;

        self.extract_result(endpoint, raw)
    }

    /// Perform a POST request with a JSON body and return the `"result"` field.
    async fn post(&self, endpoint: &str, body: Value) -> Result<Value, ExchangeError> {
        let url = if let Some(ref key) = self.api_key {
            format!("{}/{}?api_key={}", self.api_url, endpoint, key)
        } else {
            format!("{}/{}", self.api_url, endpoint)
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("{}: {}", endpoint, e)))?;

        let raw: Value = response
            .json()
            .await
            .map_err(|e| {
                ExchangeError::Parse(format!("{}: failed to parse response: {}", endpoint, e))
            })?;

        self.extract_result(endpoint, raw)
    }

    /// Extract `"result"` from a TON Center response envelope, or map `"error"` to
    /// [`ExchangeError::InvalidRequest`].
    fn extract_result(&self, endpoint: &str, raw: Value) -> Result<Value, ExchangeError> {
        // Check the `ok` flag first
        let ok = raw.get("ok").and_then(Value::as_bool).unwrap_or(false);
        if !ok {
            let code = raw.get("code").and_then(Value::as_i64).unwrap_or(-1);
            let msg = raw
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown API error");
            return Err(ExchangeError::InvalidRequest(format!(
                "{}: API error {}: {}",
                endpoint, code, msg
            )));
        }

        raw.get("result")
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse(format!("{}: missing 'result' in response", endpoint))
            })
    }

    /// Parse a decimal or hex string as `u64`.
    fn parse_u64(s: &str, context: &str) -> Result<u64, ExchangeError> {
        // TON API returns numbers as JSON numbers or decimal strings
        let trimmed = s.trim();
        if let Some(stripped) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
            u64::from_str_radix(stripped, 16).map_err(|e| {
                ExchangeError::Parse(format!("{}: failed to parse '{}' as u64 hex: {}", context, s, e))
            })
        } else {
            trimmed.parse::<u64>().map_err(|e| {
                ExchangeError::Parse(format!("{}: failed to parse '{}' as u64: {}", context, s, e))
            })
        }
    }

    /// Extract a `u64` from a JSON value that may be a number or numeric string.
    fn value_as_u64(v: &Value, context: &str) -> Result<u64, ExchangeError> {
        if let Some(n) = v.as_u64() {
            return Ok(n);
        }
        if let Some(s) = v.as_str() {
            return Self::parse_u64(s, context);
        }
        Err(ExchangeError::Parse(format!(
            "{}: expected numeric value, got: {}",
            context, v
        )))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for TonProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Ton {
            network: self.network.clone(),
        }
    }

    /// Broadcast a pre-signed BOC (Bag of Cells) to the TON network.
    ///
    /// `tx_bytes` must be a UTF-8 string containing a base64-encoded signed BOC
    /// (as produced by the TON wallet signer). The bytes are decoded to a string
    /// and forwarded to `sendBoc`.
    ///
    /// Returns `"ok"` on success (TON Center does not return a tx hash from sendBoc
    /// synchronously — use `get_tx_status` with the hash computed from the BOC).
    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        let boc = std::str::from_utf8(tx_bytes).map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "broadcast_tx: tx_bytes must be UTF-8 base64-encoded BOC: {}",
                e
            ))
        })?;

        let result = self.send_boc(boc).await?;

        // sendBoc returns {"@type": "ok"} or similar on success
        let type_field = result
            .get("@type")
            .and_then(Value::as_str)
            .unwrap_or("ok");
        Ok(type_field.to_string())
    }

    /// Get the current masterchain block seqno (best block height).
    ///
    /// Maps to `GET /getMasterchainInfo` → `result.last.seqno`.
    async fn get_height(&self) -> Result<u64, ExchangeError> {
        let result = self.get_masterchain_info().await?;

        let seqno = result
            .get("last")
            .and_then(|last| last.get("seqno"))
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "getMasterchainInfo: missing 'last.seqno' in result".to_string(),
                )
            })?;

        Self::value_as_u64(seqno, "getMasterchainInfo.last.seqno")
    }

    /// Get the account sequence number (nonce) for a TON address.
    ///
    /// Maps to `GET /getAddressInformation` → `result.seqno`.
    ///
    /// Note: not all TON accounts have a seqno (only wallet contracts do).
    /// For non-wallet contracts this returns 0.
    async fn get_nonce(&self, address: &str) -> Result<u64, ExchangeError> {
        let info = self.get_address_information(address).await?;

        // seqno is present only for wallet contracts
        match info.get("seqno") {
            Some(v) => Self::value_as_u64(v, "getAddressInformation.seqno"),
            None => Ok(0),
        }
    }

    /// Get the native TON balance for an address in nanoTON.
    ///
    /// Maps to `GET /getAddressBalance?address=...`.
    /// Returns a decimal string in nanoTON (1 TON = 1,000,000,000 nanoTON).
    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        let result = self
            .get("getAddressBalance", &[("address", address)])
            .await?;

        // Result is a string (decimal nanoTON) or a JSON number
        match &result {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            other => Err(ExchangeError::Parse(format!(
                "getAddressBalance: unexpected result type: {}",
                other
            ))),
        }
    }

    /// Get transaction status by hash.
    ///
    /// TON does not have a direct `getTransactionByHash` REST endpoint in v2.
    /// This method searches the recent transaction history for the given address
    /// by scanning `getTransactions`. Since TON transaction hashes uniquely
    /// identify a logical message, we require the address to be embedded in the
    /// hash in the format `"<address>:<hash>"` — OR the caller may pass just the
    /// hash, in which case this returns `TxStatus::Pending` (cannot determine
    /// without address context).
    ///
    /// For a more reliable approach, use the TON v3 API or index directly.
    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        // Accept compound format "address:hash" for address-scoped lookup
        if let Some((address, hash)) = tx_hash.split_once(':') {
            let txs = self.get_transactions(address, 50).await?;

            for tx in &txs {
                let tx_id = tx
                    .get("transaction_id")
                    .and_then(|id| id.get("hash"))
                    .and_then(Value::as_str)
                    .unwrap_or("");

                if tx_id == hash {
                    // Found — extract block info from lt (logical time)
                    let lt = tx
                        .get("transaction_id")
                        .and_then(|id| id.get("lt"))
                        .and_then(|v| {
                            v.as_u64()
                                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                        })
                        .unwrap_or(0);

                    return Ok(TxStatus::Confirmed { block: lt });
                }
            }

            // Not found in recent history
            return Ok(TxStatus::NotFound);
        }

        // No address context — cannot determine status from hash alone
        Ok(TxStatus::Pending)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TonChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl TonChain for TonProvider {
    async fn get_address_information(&self, address: &str) -> Result<Value, ExchangeError> {
        self.get("getAddressInformation", &[("address", address)])
            .await
    }

    async fn get_transactions(
        &self,
        address: &str,
        limit: u32,
    ) -> Result<Vec<Value>, ExchangeError> {
        let limit_str = limit.to_string();
        let result = self
            .get(
                "getTransactions",
                &[("address", address), ("limit", &limit_str)],
            )
            .await?;

        result
            .as_array()
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse("getTransactions: expected array result".to_string())
            })
    }

    async fn run_get_method(
        &self,
        address: &str,
        method: &str,
        stack: &[Value],
    ) -> Result<Value, ExchangeError> {
        let body = serde_json::json!({
            "address": address,
            "method": method,
            "stack": stack,
        });
        self.post("runGetMethod", body).await
    }

    async fn get_jetton_wallet(
        &self,
        jetton_master: &str,
        owner: &str,
    ) -> Result<String, ExchangeError> {
        // Call `get_wallet_address` on the jetton master contract.
        // Stack: [["tvm.Slice", "<owner address as cell>"]]
        // For TON Center v2, we pass the owner as a "tvm.Slice" cell.
        let stack = vec![serde_json::json!(["tvm.Slice", owner])];
        let result = self
            .run_get_method(jetton_master, "get_wallet_address", &stack)
            .await?;

        // Result stack[0] is the wallet address cell
        let stack_arr = result
            .get("stack")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ExchangeError::Parse("get_wallet_address: missing 'stack' in result".to_string())
            })?;

        let first = stack_arr.first().ok_or_else(|| {
            ExchangeError::Parse("get_wallet_address: empty stack in result".to_string())
        })?;

        // Stack element is ["tvm.Cell", "..."] or similar
        let addr = first
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "get_wallet_address: unexpected stack element format".to_string(),
                )
            })?;

        Ok(addr.to_string())
    }

    async fn get_jetton_balance(&self, jetton_wallet: &str) -> Result<String, ExchangeError> {
        // Call `get_wallet_data` on the jetton wallet contract — no args needed
        let result = self
            .run_get_method(jetton_wallet, "get_wallet_data", &[])
            .await?;

        let exit_code = result
            .get("exit_code")
            .and_then(Value::as_i64)
            .unwrap_or(-1);

        if exit_code != 0 {
            return Err(ExchangeError::InvalidRequest(format!(
                "get_wallet_data: contract returned exit_code {}",
                exit_code
            )));
        }

        // Stack[0] is the balance (tvm.Number or tvm.Int)
        let stack_arr = result
            .get("stack")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ExchangeError::Parse("get_wallet_data: missing 'stack' in result".to_string())
            })?;

        let first = stack_arr.first().ok_or_else(|| {
            ExchangeError::Parse("get_wallet_data: empty stack".to_string())
        })?;

        // ["tvm.Int", "1234567890"] or ["num", "0x1234"]
        let balance_str = first
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "get_wallet_data: unexpected stack element format".to_string(),
                )
            })?;

        // Normalize hex to decimal if needed
        if let Some(hex) = balance_str
            .strip_prefix("0x")
            .or_else(|| balance_str.strip_prefix("0X"))
        {
            // Convert hex to decimal string
            let n = u128::from_str_radix(hex, 16).map_err(|e| {
                ExchangeError::Parse(format!(
                    "get_wallet_data: failed to parse balance '{}': {}",
                    balance_str, e
                ))
            })?;
            Ok(n.to_string())
        } else {
            Ok(balance_str.to_string())
        }
    }

    async fn get_nft_items(&self, collection: &str) -> Result<Vec<Value>, ExchangeError> {
        let result = self
            .get("getNftItems", &[("collection_address", collection)])
            .await?;

        result
            .as_array()
            .cloned()
            .ok_or_else(|| {
                ExchangeError::Parse("getNftItems: expected array result".to_string())
            })
    }

    async fn get_masterchain_info(&self) -> Result<Value, ExchangeError> {
        self.get("getMasterchainInfo", &[]).await
    }

    async fn get_block_header(
        &self,
        workchain: i32,
        shard: i64,
        seqno: u32,
    ) -> Result<Value, ExchangeError> {
        let workchain_str = workchain.to_string();
        let shard_str = shard.to_string();
        let seqno_str = seqno.to_string();
        self.get(
            "getBlockHeader",
            &[
                ("workchain", &workchain_str),
                ("shard", &shard_str),
                ("seqno", &seqno_str),
            ],
        )
        .await
    }

    async fn get_config_param(&self, param: u32) -> Result<Value, ExchangeError> {
        let param_str = param.to_string();
        self.get("getConfigParam", &[("config_id", &param_str)])
            .await
    }

    async fn estimate_fee(&self, address: &str, body: &str) -> Result<Value, ExchangeError> {
        let request_body = serde_json::json!({
            "address": address,
            "body": body,
            "ignore_chksig": true,
        });
        self.post("estimateFee", request_body).await
    }

    async fn send_boc(&self, boc: &str) -> Result<Value, ExchangeError> {
        let body = serde_json::json!({ "boc": boc });
        self.post("sendBoc", body).await
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
        let provider = TonProvider::mainnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Ton {
                network: "mainnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_testnet() {
        let provider = TonProvider::testnet();
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Ton {
                network: "testnet".to_string()
            }
        );
    }

    #[test]
    fn test_chain_family_name() {
        let p = TonProvider::mainnet();
        assert_eq!(p.chain_family().name(), "ton:mainnet");
    }

    #[test]
    fn test_mainnet_url() {
        let p = TonProvider::mainnet();
        assert_eq!(p.api_url, TONCENTER_MAINNET);
        assert_eq!(p.network, "mainnet");
    }

    #[test]
    fn test_testnet_url() {
        let p = TonProvider::testnet();
        assert_eq!(p.api_url, TONCENTER_TESTNET);
        assert_eq!(p.network, "testnet");
    }

    #[test]
    fn test_custom_url() {
        let p = TonProvider::new("https://custom.example.com/api/v2", "mainnet");
        assert_eq!(p.api_url, "https://custom.example.com/api/v2");
    }

    #[test]
    fn test_with_api_key() {
        let p = TonProvider::mainnet().with_api_key("test_key_123");
        assert_eq!(p.api_key.as_deref(), Some("test_key_123"));
    }

    #[test]
    fn test_no_api_key_by_default() {
        let p = TonProvider::mainnet();
        assert!(p.api_key.is_none());
    }

    #[test]
    fn test_build_get_url_no_params_no_key() {
        let p = TonProvider::mainnet();
        let url = p.build_get_url("getMasterchainInfo", &[]);
        assert_eq!(url, format!("{}/getMasterchainInfo", TONCENTER_MAINNET));
    }

    #[test]
    fn test_build_get_url_with_params_no_key() {
        let p = TonProvider::mainnet();
        let url = p.build_get_url("getAddressBalance", &[("address", "EQAbc")]);
        assert_eq!(
            url,
            format!("{}/getAddressBalance?address=EQAbc", TONCENTER_MAINNET)
        );
    }

    #[test]
    fn test_build_get_url_with_key_no_params() {
        let p = TonProvider::mainnet().with_api_key("mykey");
        let url = p.build_get_url("getMasterchainInfo", &[]);
        assert_eq!(
            url,
            format!("{}/getMasterchainInfo?api_key=mykey", TONCENTER_MAINNET)
        );
    }

    #[test]
    fn test_build_get_url_with_params_and_key() {
        let p = TonProvider::mainnet().with_api_key("mykey");
        let url = p.build_get_url("getAddressBalance", &[("address", "EQAbc")]);
        assert_eq!(
            url,
            format!(
                "{}/getAddressBalance?address=EQAbc&api_key=mykey",
                TONCENTER_MAINNET
            )
        );
    }

    #[test]
    fn test_parse_u64_decimal() {
        assert_eq!(TonProvider::parse_u64("12345", "test").unwrap(), 12345u64);
        assert_eq!(TonProvider::parse_u64("0", "test").unwrap(), 0u64);
    }

    #[test]
    fn test_parse_u64_hex() {
        assert_eq!(TonProvider::parse_u64("0xff", "test").unwrap(), 255u64);
        assert_eq!(TonProvider::parse_u64("0x10", "test").unwrap(), 16u64);
    }

    #[test]
    fn test_parse_u64_invalid() {
        assert!(TonProvider::parse_u64("not_a_number", "test").is_err());
        assert!(TonProvider::parse_u64("0xzzzz", "test").is_err());
    }

    #[test]
    fn test_value_as_u64_number() {
        let v = serde_json::json!(42u64);
        assert_eq!(TonProvider::value_as_u64(&v, "test").unwrap(), 42u64);
    }

    #[test]
    fn test_value_as_u64_string() {
        let v = serde_json::json!("9999999");
        assert_eq!(
            TonProvider::value_as_u64(&v, "test").unwrap(),
            9999999u64
        );
    }

    #[test]
    fn test_value_as_u64_invalid() {
        let v = serde_json::json!("not_a_number");
        assert!(TonProvider::value_as_u64(&v, "test").is_err());
    }

    #[test]
    fn test_extract_result_ok() {
        let p = TonProvider::mainnet();
        let raw = serde_json::json!({"ok": true, "result": {"seqno": 123}});
        let result = p.extract_result("test", raw).unwrap();
        assert_eq!(result["seqno"], 123);
    }

    #[test]
    fn test_extract_result_error() {
        let p = TonProvider::mainnet();
        let raw = serde_json::json!({
            "ok": false,
            "error": "invalid address",
            "code": 400
        });
        let err = p.extract_result("test", raw).unwrap_err();
        match err {
            ExchangeError::InvalidRequest(msg) => {
                assert!(msg.contains("invalid address"));
                assert!(msg.contains("400"));
            }
            other => panic!("expected InvalidRequest, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_result_missing_result() {
        let p = TonProvider::mainnet();
        let raw = serde_json::json!({"ok": true});
        let err = p.extract_result("test", raw).unwrap_err();
        match err {
            ExchangeError::Parse(msg) => {
                assert!(msg.contains("missing 'result'"));
            }
            other => panic!("expected Parse, got {:?}", other),
        }
    }

    #[test]
    fn test_is_ton_family() {
        let p = TonProvider::mainnet();
        let fam = p.chain_family();
        assert!(matches!(fam, ChainFamily::Ton { network } if network == "mainnet"));
    }
}
