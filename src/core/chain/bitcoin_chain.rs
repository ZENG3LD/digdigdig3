//! # BitcoinProvider — raw JSON-RPC Bitcoin chain provider
//!
//! Implements [`ChainProvider`] and [`BitcoinChain`] for Bitcoin networks.
//!
//! ## Feature gate
//!
//! This entire module is gated behind the `onchain-bitcoin` feature. Enable it
//! in your `Cargo.toml`:
//!
//! ```toml
//! digdigdig3 = { version = "...", features = ["onchain-bitcoin"] }
//! ```
//!
//! ## Transport
//!
//! Uses raw Bitcoin JSON-RPC 1.0 over `reqwest` with optional HTTP Basic Auth.
//! No `bitcoincore-rpc` crate is required — this provider connects to any
//! Bitcoin RPC endpoint: Bitcoin Core, Blockstream Esplora (with JSON-RPC
//! compatibility), QuickNode, Alchemy, etc.
//!
//! ## Bitcoin JSON-RPC format
//!
//! ```json
//! {"jsonrpc": "1.0", "id": 1, "method": "getblockcount", "params": []}
//! ```
//!
//! Auth: HTTP Basic Auth with `rpcuser:rpcpassword` (Bitcoin Core default).
//!
//! ## Addresses
//!
//! Bitcoin addresses are passed as plain strings in their native encoding:
//! - Legacy: base58check (P2PKH: `1...`, P2SH: `3...`)
//! - SegWit: bech32 (P2WPKH/P2WSH: `bc1q...`)
//! - Taproot: bech32m (`bc1p...`)
//!
//! ## Balance units
//!
//! `get_native_balance` returns the balance in **satoshis** as a decimal
//! string. Bitcoin Core returns balances in BTC (float), which we convert:
//! `satoshis = round(btc * 100_000_000)`.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use digdigdig3::core::chain::{BitcoinProvider, BitcoinChain};
//!
//! let provider = BitcoinProvider::mainnet("http://127.0.0.1:8332", Some(("user", "pass")));
//! let height = provider.get_height().await?;
//! let info = provider.get_blockchain_info().await?;
//! let mempool = provider.get_raw_mempool().await?;
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

use super::provider::{ChainFamily, ChainProvider, TxStatus};
use crate::core::types::ExchangeError;

// ═══════════════════════════════════════════════════════════════════════════════
// BITCOIN CHAIN EXTENSION TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitcoin-specific chain operations.
///
/// Extends [`ChainProvider`] with the Bitcoin JSON-RPC surface needed by
/// on-chain monitoring and wallet tooling: raw transactions, blocks, mempool
/// queries, UTXO lookups, fee estimation, and blockchain metadata.
///
/// ## Object safety
///
/// This trait is object-safe: all method signatures use plain `&str`,
/// `u32`, `u64`, and `Vec<String>` — no Bitcoin SDK types in signatures.
///
/// ## Compatibility
///
/// All methods map directly to standard Bitcoin Core JSON-RPC methods and
/// are compatible with any node that implements the Bitcoin RPC spec
/// (Bitcoin Core, btcd, etc.).
#[async_trait]
pub trait BitcoinChain: ChainProvider {
    /// Get raw transaction by txid (`getrawtransaction` with verbose=true).
    ///
    /// Returns the full decoded transaction as a JSON object. The structure
    /// matches Bitcoin Core's `getrawtransaction` verbose output:
    /// `txid`, `hash`, `version`, `size`, `vsize`, `weight`, `locktime`,
    /// `vin` (inputs), `vout` (outputs), `blockhash`, `confirmations`, etc.
    ///
    /// Requires `txindex=1` on the node unless the tx is in the mempool
    /// or in a block explicitly queried via `blockhash`.
    async fn get_raw_transaction(&self, txid: &str) -> Result<Value, ExchangeError>;

    /// Get a block by its hash (`getblock` with verbosity=2).
    ///
    /// Returns the fully decoded block including all transactions. The
    /// verbosity level 2 ensures each transaction in `tx` is a full object
    /// (same as `getrawtransaction` verbose output), not just a txid string.
    async fn get_block(&self, block_hash: &str) -> Result<Value, ExchangeError>;

    /// Get a block by height (`getblockhash` + `getblock`).
    ///
    /// First resolves the block hash at the given height, then fetches the
    /// full block with verbosity=2.
    async fn get_block_by_height(&self, height: u64) -> Result<Value, ExchangeError>;

    /// Get the list of transaction IDs currently in the mempool (`getrawmempool`).
    ///
    /// Returns a `Vec<String>` of txid hex strings. For detailed mempool
    /// entries (fee, size, ancestors) use [`get_mempool_entry`].
    async fn get_raw_mempool(&self) -> Result<Vec<String>, ExchangeError>;

    /// Get detailed mempool entry for a specific transaction (`getmempoolentry`).
    ///
    /// Returns fee, virtual size, weight, ancestor/descendant counts, and
    /// the time the transaction entered the mempool.
    async fn get_mempool_entry(&self, txid: &str) -> Result<Value, ExchangeError>;

    /// Scan the UTXO set for an address (`scantxoutset`).
    ///
    /// Returns the list of unspent transaction outputs for the given address.
    /// Each entry contains `txid`, `vout`, `scriptPubKey`, `desc`, `amount`
    /// (in BTC), `height`, and `coinbase`.
    ///
    /// Note: `scantxoutset` is a long-running operation on initial scan.
    /// For real-time wallet monitoring, prefer subscribing to ZMQ notifications
    /// on the node or using a purpose-built indexer.
    async fn list_unspent(&self, address: &str) -> Result<Vec<Value>, ExchangeError>;

    /// Get blockchain metadata (`getblockchaininfo`).
    ///
    /// Returns `chain`, `blocks`, `headers`, `bestblockhash`, `difficulty`,
    /// `mediantime`, `verificationprogress`, `chainwork`, `pruned`, etc.
    async fn get_blockchain_info(&self) -> Result<Value, ExchangeError>;

    /// Get the estimated network hash rate (`getnetworkhashps`).
    ///
    /// Returns the estimated hashes per second for the most recent 120 blocks
    /// (Bitcoin Core default). The value reflects the total network mining power.
    async fn get_network_hashrate(&self) -> Result<f64, ExchangeError>;

    /// Estimate the fee rate for confirmation within `conf_target` blocks
    /// (`estimatesmartfee`).
    ///
    /// Returns the recommended fee rate in **satoshis per virtual byte** (sat/vB).
    /// Bitcoin Core returns the fee in BTC/kB; this method converts to sat/vB:
    /// `sat_per_vbyte = btc_per_kb * 100_000_000 / 1000`.
    ///
    /// `conf_target` must be between 1 and 1008. Common values:
    /// - 1 = next block (high priority)
    /// - 6 = ~1 hour
    /// - 144 = ~1 day (economy)
    async fn estimate_smart_fee(&self, conf_target: u32) -> Result<f64, ExchangeError>;

    /// Decode a raw transaction hex string (`decoderawtransaction`).
    ///
    /// Returns the fully decoded transaction structure without broadcasting it.
    /// Useful for inspecting a signed transaction before sending.
    async fn decode_raw_transaction(&self, hex: &str) -> Result<Value, ExchangeError>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// BITCOIN PROVIDER STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete Bitcoin chain provider using raw JSON-RPC 1.0 over HTTP.
///
/// One `BitcoinProvider` per RPC endpoint is sufficient. Multiple components
/// monitoring the Bitcoin chain can share a single instance via
/// `Arc<BitcoinProvider>`, reusing the HTTP connection pool.
///
/// ## Construction
///
/// ```rust,ignore
/// // Bitcoin Core on localhost (mainnet)
/// let provider = BitcoinProvider::mainnet(
///     "http://127.0.0.1:8332",
///     Some(("rpcuser", "rpcpassword")),
/// );
///
/// // Bitcoin testnet
/// let testnet = BitcoinProvider::testnet(
///     "http://127.0.0.1:18332",
///     Some(("user", "pass")),
/// );
///
/// // Public node (no auth)
/// let public = BitcoinProvider::mainnet("https://bitcoin-rpc.publicnode.com", None);
/// ```
pub struct BitcoinProvider {
    /// JSON-RPC endpoint URL (e.g. `http://127.0.0.1:8332`)
    rpc_url: String,
    /// Shared HTTP client for all JSON-RPC calls
    client: reqwest::Client,
    /// Optional HTTP Basic Auth credentials (rpcuser, rpcpassword)
    auth: Option<(String, String)>,
    /// Network name: `"mainnet"`, `"testnet"`, `"signet"`, or `"regtest"`
    network: String,
    /// Monotonically increasing JSON-RPC request ID
    request_id: AtomicU64,
}

impl BitcoinProvider {
    // ─────────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a provider for the given network.
    ///
    /// `rpc_url` — full URL including port, e.g. `"http://127.0.0.1:8332"`.
    /// `auth` — optional `(rpcuser, rpcpassword)` for HTTP Basic Auth.
    /// `network` — `"mainnet"`, `"testnet"`, `"signet"`, or `"regtest"`.
    pub fn new(
        rpc_url: impl Into<String>,
        auth: Option<(&str, &str)>,
        network: impl Into<String>,
    ) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            client: reqwest::Client::new(),
            auth: auth.map(|(u, p)| (u.to_string(), p.to_string())),
            network: network.into(),
            request_id: AtomicU64::new(1),
        }
    }

    /// Create a provider targeting Bitcoin mainnet (port 8332).
    ///
    /// `rpc_url` — e.g. `"http://127.0.0.1:8332"` or a hosted endpoint URL.
    /// `auth` — optional `(rpcuser, rpcpassword)`.
    pub fn mainnet(rpc_url: impl Into<String>, auth: Option<(&str, &str)>) -> Self {
        Self::new(rpc_url, auth, "mainnet")
    }

    /// Create a provider targeting Bitcoin testnet3 (port 18332).
    ///
    /// `rpc_url` — e.g. `"http://127.0.0.1:18332"`.
    /// `auth` — optional `(rpcuser, rpcpassword)`.
    pub fn testnet(rpc_url: impl Into<String>, auth: Option<(&str, &str)>) -> Self {
        Self::new(rpc_url, auth, "testnet")
    }

    /// Create a provider targeting Bitcoin signet (port 38332).
    ///
    /// `rpc_url` — e.g. `"http://127.0.0.1:38332"`.
    /// `auth` — optional `(rpcuser, rpcpassword)`.
    pub fn signet(rpc_url: impl Into<String>, auth: Option<(&str, &str)>) -> Self {
        Self::new(rpc_url, auth, "signet")
    }

    /// Create a provider targeting a local regtest node (port 18443).
    ///
    /// `rpc_url` — e.g. `"http://127.0.0.1:18443"`.
    /// `auth` — optional `(rpcuser, rpcpassword)`.
    pub fn regtest(rpc_url: impl Into<String>, auth: Option<(&str, &str)>) -> Self {
        Self::new(rpc_url, auth, "regtest")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Get the next request ID (monotonically increasing).
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Execute a Bitcoin JSON-RPC 1.0 call and return the `"result"` field.
    ///
    /// Bitcoin Core uses JSON-RPC 1.0, which differs from 2.0:
    /// - `"jsonrpc"` field is `"1.0"` (not `"2.0"`)
    /// - `"id"` is always present (not optional)
    /// - Errors come in `"error"` field (object with `code` and `message`)
    ///
    /// Returns [`ExchangeError::Network`] on transport errors,
    /// [`ExchangeError::Parse`] if the response cannot be decoded,
    /// and [`ExchangeError::InvalidRequest`] if the node returns a JSON-RPC
    /// error object.
    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value, ExchangeError> {
        let id = self.next_id();
        let body = json!({
            "jsonrpc": "1.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let mut request = self.client.post(&self.rpc_url).json(&body);

        if let Some((user, pass)) = &self.auth {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("{}: {}", method, e)))?;

        let raw: Value = response.json().await.map_err(|e| {
            ExchangeError::Parse(format!("{}: failed to parse response: {}", method, e))
        })?;

        // JSON-RPC error object takes priority over missing result
        if let Some(err_obj) = raw.get("error") {
            if !err_obj.is_null() {
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
        }

        raw.get("result")
            .cloned()
            .ok_or_else(|| ExchangeError::Parse(format!("{}: missing 'result' in response", method)))
    }

    /// Convert BTC (float) to satoshis (integer), returned as a decimal string.
    ///
    /// Bitcoin Core returns balance amounts as floating-point BTC values.
    /// We multiply by 1e8 and round to avoid floating-point precision issues.
    fn btc_to_satoshis(btc: f64) -> String {
        let satoshis = (btc * 100_000_000.0).round() as u64;
        satoshis.to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChainProvider IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ChainProvider for BitcoinProvider {
    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Bitcoin {
            network: self.network.clone(),
        }
    }

    /// Broadcast a pre-signed raw transaction (`sendrawtransaction`).
    ///
    /// `tx_bytes` must be the raw transaction hex string encoded as UTF-8 bytes
    /// (i.e. the bytes of the hex string, not the binary transaction itself).
    ///
    /// Returns the txid as a hex string.
    async fn broadcast_tx(&self, tx_bytes: &[u8]) -> Result<String, ExchangeError> {
        let hex_str = std::str::from_utf8(tx_bytes).map_err(|e| {
            ExchangeError::InvalidRequest(format!(
                "broadcast_tx: tx_bytes must be UTF-8 hex string: {}",
                e
            ))
        })?;

        let result = self
            .rpc_call("sendrawtransaction", json!([hex_str]))
            .await?;

        result
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| ExchangeError::Parse("sendrawtransaction: expected string txid".to_string()))
    }

    /// Get the current block count (`getblockcount`).
    async fn get_height(&self) -> Result<u64, ExchangeError> {
        let result = self.rpc_call("getblockcount", json!([])).await?;
        result
            .as_u64()
            .ok_or_else(|| ExchangeError::Parse("getblockcount: non-integer result".to_string()))
    }

    /// Bitcoin has no account nonces — always returns 0.
    ///
    /// Bitcoin's UTXO model does not use nonces. This method exists only to
    /// satisfy the [`ChainProvider`] trait interface. Callers building Bitcoin
    /// transactions should use UTXO selection instead.
    async fn get_nonce(&self, _address: &str) -> Result<u64, ExchangeError> {
        Ok(0)
    }

    /// Get the confirmed BTC balance for an address in satoshis.
    ///
    /// Uses `scantxoutset` to scan the UTXO set for the given address.
    /// Returns the total confirmed balance as a decimal satoshi string.
    ///
    /// Note: `scantxoutset` can be slow on first call as it scans the full
    /// UTXO set. The result is the sum of all unspent outputs for the address.
    async fn get_native_balance(&self, address: &str) -> Result<String, ExchangeError> {
        let descriptor = format!("addr({})", address);
        let result = self
            .rpc_call("scantxoutset", json!(["start", [{"desc": descriptor}]]))
            .await?;

        let btc_amount = result
            .get("total_amount")
            .and_then(Value::as_f64)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "scantxoutset: missing or invalid 'total_amount' field".to_string(),
                )
            })?;

        Ok(Self::btc_to_satoshis(btc_amount))
    }

    /// Get transaction status by txid (`gettransaction`).
    ///
    /// Maps Bitcoin confirmation count to [`TxStatus`]:
    /// - 0 confirmations → [`TxStatus::Pending`] (in mempool)
    /// - ≥1 confirmation → [`TxStatus::Confirmed`] with block height
    /// - RPC error "Invalid or non-wallet transaction" → [`TxStatus::NotFound`]
    ///
    /// Note: `gettransaction` only works for wallet transactions. For
    /// non-wallet transactions use [`BitcoinChain::get_raw_transaction`] and
    /// check the `confirmations` field manually.
    async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, ExchangeError> {
        let result = self.rpc_call("gettransaction", json!([tx_hash])).await;

        match result {
            Err(ExchangeError::InvalidRequest(msg))
                if msg.contains("-5")
                    || msg.contains("Invalid or non-wallet")
                    || msg.contains("not found") =>
            {
                return Ok(TxStatus::NotFound);
            }
            Err(e) => return Err(e),
            Ok(tx) => {
                let confirmations = tx.get("confirmations").and_then(Value::as_i64).unwrap_or(0);

                if confirmations <= 0 {
                    // 0 = mempool; negative = conflicted (treat as pending)
                    Ok(TxStatus::Pending)
                } else {
                    // Get the block height from blockheight field (added in Bitcoin Core 0.21)
                    // Fall back to 0 if not available (older nodes)
                    let block = tx.get("blockheight").and_then(Value::as_u64).unwrap_or(0);
                    Ok(TxStatus::Confirmed { block })
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BitcoinChain IMPL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BitcoinChain for BitcoinProvider {
    async fn get_raw_transaction(&self, txid: &str) -> Result<Value, ExchangeError> {
        // verbose=true returns the decoded transaction object
        self.rpc_call("getrawtransaction", json!([txid, true])).await
    }

    async fn get_block(&self, block_hash: &str) -> Result<Value, ExchangeError> {
        // verbosity=2: full block with decoded transactions
        self.rpc_call("getblock", json!([block_hash, 2])).await
    }

    async fn get_block_by_height(&self, height: u64) -> Result<Value, ExchangeError> {
        // Step 1: resolve hash from height
        let hash_result = self.rpc_call("getblockhash", json!([height])).await?;
        let block_hash = hash_result
            .as_str()
            .ok_or_else(|| ExchangeError::Parse("getblockhash: expected string hash".to_string()))?;

        // Step 2: fetch full block by hash
        self.get_block(block_hash).await
    }

    async fn get_raw_mempool(&self) -> Result<Vec<String>, ExchangeError> {
        // verbose=false returns array of txid strings
        let result = self.rpc_call("getrawmempool", json!([false])).await?;

        let arr = result
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("getrawmempool: expected array".to_string()))?;

        arr.iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| ExchangeError::Parse("getrawmempool: non-string txid".to_string()))
            })
            .collect()
    }

    async fn get_mempool_entry(&self, txid: &str) -> Result<Value, ExchangeError> {
        self.rpc_call("getmempoolentry", json!([txid])).await
    }

    async fn list_unspent(&self, address: &str) -> Result<Vec<Value>, ExchangeError> {
        let descriptor = format!("addr({})", address);
        let result = self
            .rpc_call("scantxoutset", json!(["start", [{"desc": descriptor}]]))
            .await?;

        let unspents = result
            .get("unspents")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ExchangeError::Parse("scantxoutset: missing 'unspents' array".to_string())
            })?;

        Ok(unspents.clone())
    }

    async fn get_blockchain_info(&self) -> Result<Value, ExchangeError> {
        self.rpc_call("getblockchaininfo", json!([])).await
    }

    async fn get_network_hashrate(&self) -> Result<f64, ExchangeError> {
        // nblocks=-1 uses the default (120 blocks); nheight=-1 uses the chain tip
        let result = self.rpc_call("getnetworkhashps", json!([])).await?;

        result
            .as_f64()
            .ok_or_else(|| ExchangeError::Parse("getnetworkhashps: expected numeric result".to_string()))
    }

    async fn estimate_smart_fee(&self, conf_target: u32) -> Result<f64, ExchangeError> {
        let result = self
            .rpc_call("estimatesmartfee", json!([conf_target]))
            .await?;

        // estimatesmartfee returns { "feerate": <BTC/kB>, "blocks": <n> }
        // or { "errors": [...], "blocks": <n> } if estimation fails
        if let Some(errors) = result.get("errors") {
            if let Some(err_arr) = errors.as_array() {
                if !err_arr.is_empty() {
                    let msg = err_arr
                        .first()
                        .and_then(Value::as_str)
                        .unwrap_or("fee estimation failed");
                    return Err(ExchangeError::InvalidRequest(format!(
                        "estimatesmartfee: {}",
                        msg
                    )));
                }
            }
        }

        let feerate_btc_per_kb = result
            .get("feerate")
            .and_then(Value::as_f64)
            .ok_or_else(|| {
                ExchangeError::Parse(
                    "estimatesmartfee: missing or invalid 'feerate' field".to_string(),
                )
            })?;

        // Convert BTC/kB to sat/vB: multiply by 1e8 (sat/BTC) then divide by 1000 (B/kB)
        let sat_per_vbyte = feerate_btc_per_kb * 100_000_000.0 / 1000.0;
        Ok(sat_per_vbyte)
    }

    async fn decode_raw_transaction(&self, hex: &str) -> Result<Value, ExchangeError> {
        self.rpc_call("decoderawtransaction", json!([hex])).await
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
        let provider = BitcoinProvider::mainnet("http://127.0.0.1:8332", None);
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Bitcoin {
                network: "mainnet".to_string()
            }
        );
        assert_eq!(provider.chain_family().name(), "bitcoin:mainnet");
    }

    #[test]
    fn test_chain_family_testnet() {
        let provider = BitcoinProvider::testnet("http://127.0.0.1:18332", None);
        assert_eq!(
            provider.chain_family(),
            ChainFamily::Bitcoin {
                network: "testnet".to_string()
            }
        );
        assert_eq!(provider.chain_family().name(), "bitcoin:testnet");
    }

    #[test]
    fn test_chain_family_signet() {
        let provider = BitcoinProvider::signet("http://127.0.0.1:38332", None);
        assert_eq!(provider.chain_family().name(), "bitcoin:signet");
    }

    #[test]
    fn test_chain_family_regtest() {
        let provider = BitcoinProvider::regtest("http://127.0.0.1:18443", None);
        assert_eq!(provider.chain_family().name(), "bitcoin:regtest");
    }

    #[test]
    fn test_chain_family_is_bitcoin() {
        let provider = BitcoinProvider::mainnet("http://127.0.0.1:8332", None);
        assert!(provider.chain_family().is_bitcoin("mainnet"));
        assert!(!provider.chain_family().is_bitcoin("testnet"));
    }

    #[test]
    fn test_btc_to_satoshis_whole() {
        assert_eq!(BitcoinProvider::btc_to_satoshis(1.0), "100000000");
        assert_eq!(BitcoinProvider::btc_to_satoshis(0.0), "0");
        assert_eq!(BitcoinProvider::btc_to_satoshis(21_000_000.0), "2100000000000000");
    }

    #[test]
    fn test_btc_to_satoshis_fractional() {
        assert_eq!(BitcoinProvider::btc_to_satoshis(0.00000001), "1");
        assert_eq!(BitcoinProvider::btc_to_satoshis(0.5), "50000000");
        assert_eq!(BitcoinProvider::btc_to_satoshis(1.23456789), "123456789");
    }

    #[test]
    fn test_request_id_increments() {
        let provider = BitcoinProvider::mainnet("http://127.0.0.1:8332", None);
        let id1 = provider.next_id();
        let id2 = provider.next_id();
        let id3 = provider.next_id();
        assert_eq!(id1 + 1, id2);
        assert_eq!(id2 + 1, id3);
    }

    #[test]
    fn test_auth_construction() {
        let provider =
            BitcoinProvider::mainnet("http://127.0.0.1:8332", Some(("rpcuser", "rpcpass")));
        assert!(provider.auth.is_some());
        let (user, pass) = provider.auth.unwrap();
        assert_eq!(user, "rpcuser");
        assert_eq!(pass, "rpcpass");
    }

    #[test]
    fn test_no_auth_construction() {
        let provider = BitcoinProvider::mainnet("http://127.0.0.1:8332", None);
        assert!(provider.auth.is_none());
    }

    #[test]
    fn test_custom_network() {
        let provider = BitcoinProvider::new("http://localhost:8332", None, "regtest");
        assert_eq!(provider.network, "regtest");
        assert!(provider.chain_family().is_bitcoin("regtest"));
    }

    #[tokio::test]
    async fn test_get_nonce_always_zero() {
        let provider = BitcoinProvider::mainnet("http://127.0.0.1:8332", None);
        // Bitcoin has no nonces — must always return 0 without making any RPC call
        let nonce = provider.get_nonce("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq").await;
        assert_eq!(nonce.unwrap(), 0);
    }
}
