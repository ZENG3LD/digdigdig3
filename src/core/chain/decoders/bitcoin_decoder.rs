//! # BitcoinEventDecoder — decode raw Bitcoin RPC transactions into OnChainEvents
//!
//! Bitcoin does not have Ethereum-style event logs. Instead, events are derived
//! by inspecting the transaction structure:
//!
//! - **Coinbase transactions** → [`CoinbaseReward`](OnChainEventType::CoinbaseReward)
//! - **OP_RETURN outputs** → noted in `raw` for inscription/Rune tracking
//! - **UTXO inputs** → [`UtxoSpent`](OnChainEventType::UtxoSpent) per input
//! - **All outputs** → [`NativeTransfer`](OnChainEventType::NativeTransfer) per output
//! - **Mempool transactions** → [`MempoolTransaction`](OnChainEventType::MempoolTransaction)
//!
//! ## Feature gate
//!
//! This module is gated behind the `onchain-bitcoin` feature.

use std::collections::HashMap;

use serde_json::Value;

use crate::core::types::onchain::{ChainId, OnChainEvent, OnChainEventType};

// ═══════════════════════════════════════════════════════════════════════════════
// BITCOIN EVENT DECODER
// ═══════════════════════════════════════════════════════════════════════════════

/// Decodes raw Bitcoin JSON-RPC transaction and block data into [`OnChainEvent`] values.
///
/// Bitcoin's UTXO model differs from account-based chains: there are no logs,
/// no token contracts, and no program execution traces. All observable events
/// are derived from the transaction's inputs (`vin`) and outputs (`vout`).
///
/// ## Input format
///
/// All methods accept a `&serde_json::Value` that matches the verbose output
/// of `getrawtransaction` (verbosity=true) or `getblock` (verbosity=2).
///
/// ## Event mapping
///
/// | Transaction shape | Events produced |
/// |-------------------|-----------------|
/// | Coinbase | One `CoinbaseReward` |
/// | Regular tx inputs | One `UtxoSpent` per input with address |
/// | Regular tx outputs | One `NativeTransfer` per output with address |
/// | OP_RETURN output | One `NativeTransfer` with raw OP_RETURN data attached |
/// | Mempool tx | One `MempoolTransaction` |
pub struct BitcoinEventDecoder {
    /// Chain this decoder is monitoring (`bitcoin:mainnet`, etc.).
    chain: ChainId,
    /// Optional label map: Bitcoin address → human-readable label.
    ///
    /// Used to attach a `pool` tag on `CoinbaseReward` when the miner address
    /// matches a known mining pool address. Not required for correct operation.
    known_addresses: HashMap<String, String>,
}

impl BitcoinEventDecoder {
    /// Create a decoder for Bitcoin mainnet with no known address labels.
    pub fn new() -> Self {
        Self {
            chain: ChainId::new("bitcoin", "mainnet"),
            known_addresses: HashMap::new(),
        }
    }

    /// Create a decoder for the specified network (`"mainnet"`, `"testnet"`, etc.).
    pub fn for_network(network: impl Into<String>) -> Self {
        Self {
            chain: ChainId::new("bitcoin", network),
            known_addresses: HashMap::new(),
        }
    }

    /// Register a known address label (e.g. mining pool, exchange hot wallet).
    ///
    /// Labels appear as the `pool` field in [`CoinbaseReward`](OnChainEventType::CoinbaseReward)
    /// when the miner address is recognised, and are included in `raw` for
    /// other event types.
    pub fn with_label(mut self, address: impl Into<String>, label: impl Into<String>) -> Self {
        self.known_addresses.insert(address.into(), label.into());
        self
    }

    /// Add a batch of address labels from an iterator of `(address, label)` pairs.
    pub fn with_labels(
        mut self,
        labels: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (addr, label) in labels {
            self.known_addresses.insert(addr.into(), label.into());
        }
        self
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Public decode methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Decode a single verbose Bitcoin transaction into a list of [`OnChainEvent`]s.
    ///
    /// `tx` must be the JSON object returned by `getrawtransaction` with
    /// `verbose=true`, or a transaction object embedded in a `getblock` response
    /// with verbosity=2.
    ///
    /// `block_height` — the height of the containing block. Pass `0` if unknown.
    /// `block_time` — Unix timestamp of the block. Pass `0` if unknown.
    ///
    /// Returns an empty `Vec` if the transaction cannot be decoded (e.g. missing
    /// required fields), never panics.
    pub fn decode_transaction(
        &self,
        tx: &Value,
        block_height: u64,
        block_time: u64,
    ) -> Vec<OnChainEvent> {
        let txid = match tx.get("txid").and_then(Value::as_str) {
            Some(id) => id.to_string(),
            None => return vec![],
        };

        let mut events = Vec::new();

        if self.is_coinbase(tx) {
            // Coinbase transaction — emit a single CoinbaseReward event
            if let Some(evt) = self.decode_coinbase(tx, &txid, block_height, block_time) {
                events.push(evt);
            }
        } else {
            // Regular transaction — emit UtxoSpent for each input
            events.extend(self.decode_inputs(tx, &txid, block_height, block_time));
        }

        // Emit NativeTransfer for each output (coinbase outputs included)
        events.extend(self.decode_outputs(tx, &txid, block_height, block_time));

        events
    }

    /// Decode all transactions in a full Bitcoin block.
    ///
    /// `block` must be the JSON object returned by `getblock` with verbosity=2.
    /// Each transaction in the `"tx"` array is decoded via [`decode_transaction`].
    ///
    /// Returns all events from all transactions in block order.
    pub fn decode_block(&self, block: &Value) -> Vec<OnChainEvent> {
        let height = block
            .get("height")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let time = block
            .get("time")
            .and_then(Value::as_u64)
            .unwrap_or(0);

        let txs = match block.get("tx").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };

        txs.iter()
            .flat_map(|tx| self.decode_transaction(tx, height, time))
            .collect()
    }

    /// Decode a mempool transaction (not yet in a block).
    ///
    /// `tx` must be the JSON object returned by `getrawtransaction` with
    /// `verbose=true` for a transaction currently in the mempool.
    ///
    /// Returns a single [`MempoolTransaction`](OnChainEventType::MempoolTransaction)
    /// event with `block = 0`. The `gas_price` field is populated from the
    /// `fees.base` field if present (sat/vByte as a string).
    pub fn decode_mempool_tx(&self, tx: &Value) -> Vec<OnChainEvent> {
        let txid = match tx.get("txid").and_then(Value::as_str) {
            Some(id) => id.to_string(),
            None => return vec![],
        };

        // Derive `from` address from first input's scriptSig/coinbase
        let from = self.first_input_address(tx).unwrap_or_else(|| "unknown".to_string());

        // Derive `to` and total value from first non-OP_RETURN output
        let (to, value) = self.first_output_info(tx);

        // Total output size of the transaction in bytes (vsize preferred)
        let data_size = tx
            .get("vsize")
            .or_else(|| tx.get("size"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;

        // Fee rate from mempool entry if available (sat/vB)
        let gas_price = tx
            .get("fees")
            .and_then(|f| f.get("base"))
            .and_then(Value::as_f64)
            .map(|btc| {
                let sats = (btc * 100_000_000.0).round() as u64;
                sats.to_string()
            });

        let event = OnChainEvent {
            chain: self.chain.clone(),
            block: 0,
            tx_hash: txid,
            log_index: None,
            timestamp: 0,
            event_type: OnChainEventType::MempoolTransaction {
                from,
                to,
                value,
                gas_price,
                data_size,
            },
            raw: None,
        };

        vec![event]
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers — classification
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns `true` if the transaction is a coinbase transaction.
    ///
    /// A Bitcoin coinbase transaction has exactly one input whose `txid` is
    /// `"0000000000000000000000000000000000000000000000000000000000000000"` and
    /// whose `vout` index is `0xFFFFFFFF` (4294967295). The easier check:
    /// the coinbase field in `vin[0]` is present.
    pub fn is_coinbase(&self, tx: &Value) -> bool {
        tx.get("vin")
            .and_then(Value::as_array)
            .and_then(|vin| vin.first())
            .and_then(|input| input.get("coinbase"))
            .is_some()
    }

    /// Extract all `(address, amount_satoshi)` pairs from transaction outputs.
    ///
    /// Returns only outputs that have a decodable address. OP_RETURN outputs
    /// and bare multisig outputs without a known address are skipped.
    pub fn extract_outputs(&self, tx: &Value) -> Vec<(String, String)> {
        let vout = match tx.get("vout").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };

        vout.iter()
            .filter_map(|output| {
                let btc = output.get("value").and_then(Value::as_f64)?;
                let satoshis = Self::btc_to_satoshis(btc);
                let address = Self::extract_address_from_output(output)?;
                Some((address, satoshis))
            })
            .collect()
    }

    /// Detect an OP_RETURN output and return its hex data payload, if present.
    ///
    /// OP_RETURN outputs are used for Bitcoin meta-protocols: Ordinal inscriptions,
    /// Runes, Stamps, and other embedded-data schemes. The raw hex following
    /// `OP_RETURN` is returned as-is without further interpretation.
    pub fn detect_op_return(&self, tx: &Value) -> Option<String> {
        let vout = tx.get("vout").and_then(Value::as_array)?;

        for output in vout {
            let script_type = output
                .get("scriptPubKey")
                .and_then(|s| s.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("");

            if script_type == "nulldata" {
                // The hex field on scriptPubKey contains the full script bytes
                let hex = output
                    .get("scriptPubKey")
                    .and_then(|s| s.get("hex"))
                    .and_then(Value::as_str)
                    .map(str::to_string);
                return hex;
            }
        }

        None
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers — event construction
    // ─────────────────────────────────────────────────────────────────────────

    fn decode_coinbase(
        &self,
        tx: &Value,
        txid: &str,
        block_height: u64,
        block_time: u64,
    ) -> Option<OnChainEvent> {
        // Sum all outputs for the total reward (block subsidy + fees)
        let total_sats: u64 = tx
            .get("vout")
            .and_then(Value::as_array)
            .map(|vout| {
                vout.iter()
                    .filter_map(|o| o.get("value").and_then(Value::as_f64))
                    .map(|btc| (btc * 100_000_000.0).round() as u64)
                    .sum()
            })
            .unwrap_or(0);

        // Miner address: first output with a decodable address
        let miner = tx
            .get("vout")
            .and_then(Value::as_array)
            .and_then(|vout| {
                vout.iter()
                    .find_map(|o| Self::extract_address_from_output(o))
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Try to identify the mining pool from known addresses
        let pool = self.known_addresses.get(&miner).cloned();

        // Also check coinbase script for known pool tags (text in coinbase field)
        let pool = pool.or_else(|| {
            tx.get("vin")
                .and_then(Value::as_array)
                .and_then(|vin| vin.first())
                .and_then(|input| input.get("coinbase"))
                .and_then(Value::as_str)
                .and_then(|hex| self.identify_pool_from_coinbase_hex(hex))
        });

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block: block_height,
            tx_hash: txid.to_string(),
            log_index: None,
            timestamp: block_time,
            event_type: OnChainEventType::CoinbaseReward {
                miner,
                reward: total_sats.to_string(),
                pool,
            },
            raw: None,
        })
    }

    fn decode_inputs(
        &self,
        tx: &Value,
        txid: &str,
        block_height: u64,
        block_time: u64,
    ) -> Vec<OnChainEvent> {
        let vin = match tx.get("vin").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };

        vin.iter()
            .enumerate()
            .filter_map(|(idx, input)| {
                // Skip coinbase inputs
                if input.get("coinbase").is_some() {
                    return None;
                }

                let prev_tx = input.get("txid").and_then(Value::as_str)?;
                let prev_index = input.get("vout").and_then(Value::as_u64).unwrap_or(0) as u32;

                // The spending address comes from the scriptSig or txinwitness
                let spender = self.extract_spender_address(input)
                    .unwrap_or_else(|| "unknown".to_string());

                // The spent value comes from the `prevout` object (if available, verbosity=3)
                // or falls back to "0" — we don't make RPC calls here
                let amount = input
                    .get("prevout")
                    .and_then(|po| po.get("value"))
                    .and_then(Value::as_f64)
                    .map(Self::btc_to_satoshis)
                    .unwrap_or_else(|| "0".to_string());

                Some(OnChainEvent {
                    chain: self.chain.clone(),
                    block: block_height,
                    tx_hash: txid.to_string(),
                    log_index: Some(idx as u32),
                    timestamp: block_time,
                    event_type: OnChainEventType::UtxoSpent {
                        prev_tx: prev_tx.to_string(),
                        prev_index,
                        spender,
                        amount,
                    },
                    raw: None,
                })
            })
            .collect()
    }

    fn decode_outputs(
        &self,
        tx: &Value,
        txid: &str,
        block_height: u64,
        block_time: u64,
    ) -> Vec<OnChainEvent> {
        let vout = match tx.get("vout").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };

        // Derive a single `from` address from the first input (best-effort)
        let from = self.first_input_address(tx).unwrap_or_else(|| "coinbase".to_string());

        vout.iter()
            .enumerate()
            .filter_map(|(idx, output)| {
                let btc = output.get("value").and_then(Value::as_f64).unwrap_or(0.0);
                let amount = Self::btc_to_satoshis(btc);

                let script_type = output
                    .get("scriptPubKey")
                    .and_then(|s| s.get("type"))
                    .and_then(Value::as_str)
                    .unwrap_or("");

                // OP_RETURN outputs: emit NativeTransfer with raw data attached
                if script_type == "nulldata" {
                    let op_return_hex = output
                        .get("scriptPubKey")
                        .and_then(|s| s.get("hex"))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();

                    return Some(OnChainEvent {
                        chain: self.chain.clone(),
                        block: block_height,
                        tx_hash: txid.to_string(),
                        log_index: Some(idx as u32),
                        timestamp: block_time,
                        event_type: OnChainEventType::NativeTransfer {
                            from: from.clone(),
                            to: "OP_RETURN".to_string(),
                            amount,
                            usd_value: None,
                        },
                        raw: Some(serde_json::json!({ "op_return": op_return_hex })),
                    });
                }

                // Regular outputs with a decodable address
                let to = Self::extract_address_from_output(output)?;

                Some(OnChainEvent {
                    chain: self.chain.clone(),
                    block: block_height,
                    tx_hash: txid.to_string(),
                    log_index: Some(idx as u32),
                    timestamp: block_time,
                    event_type: OnChainEventType::NativeTransfer {
                        from: from.clone(),
                        to,
                        amount,
                        usd_value: None,
                    },
                    raw: None,
                })
            })
            .collect()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Address extraction helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Extract the destination address from a transaction output (`vout` entry).
    ///
    /// Tries in order:
    /// 1. `scriptPubKey.address` (single address, Bitcoin Core >= 0.18)
    /// 2. First entry of `scriptPubKey.addresses` (legacy multi-address field)
    fn extract_address_from_output(output: &Value) -> Option<String> {
        let spk = output.get("scriptPubKey")?;

        // Bitcoin Core >= 0.18 returns a single `address` field
        if let Some(addr) = spk.get("address").and_then(Value::as_str) {
            return Some(addr.to_string());
        }

        // Older nodes return `addresses` array
        spk.get("addresses")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    /// Extract the sender address from a transaction input (`vin` entry).
    ///
    /// For native SegWit / Taproot inputs the address can be reconstructed
    /// from the witness pubkey, but that requires hashing. Bitcoin Core
    /// provides it in `prevout.scriptPubKey.address` at verbosity=3.
    /// At verbosity=2 we fall back to scriptSig's `address` hint if present.
    fn extract_spender_address(&self, input: &Value) -> Option<String> {
        // Verbosity 3: prevout with address
        if let Some(addr) = input
            .get("prevout")
            .and_then(|po| po.get("scriptPubKey"))
            .and_then(|s| s.get("address"))
            .and_then(Value::as_str)
        {
            return Some(addr.to_string());
        }

        // Verbosity 2: scriptSig may have address for P2PKH
        input
            .get("scriptSig")
            .and_then(|ss| ss.get("address"))
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    /// Best-effort: get the first input's spending address.
    fn first_input_address(&self, tx: &Value) -> Option<String> {
        let vin = tx.get("vin").and_then(Value::as_array)?;
        for input in vin {
            if input.get("coinbase").is_some() {
                return Some("coinbase".to_string());
            }
            if let Some(addr) = self.extract_spender_address(input) {
                return Some(addr);
            }
        }
        None
    }

    /// Best-effort: get the first non-OP_RETURN output address and its value.
    fn first_output_info(&self, tx: &Value) -> (Option<String>, String) {
        let vout = match tx.get("vout").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return (None, "0".to_string()),
        };

        for output in vout {
            let script_type = output
                .get("scriptPubKey")
                .and_then(|s| s.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("");

            if script_type == "nulldata" {
                continue;
            }

            let addr = Self::extract_address_from_output(output);
            let btc = output.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            let sats = Self::btc_to_satoshis(btc);
            return (addr, sats);
        }

        (None, "0".to_string())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Pool identification
    // ─────────────────────────────────────────────────────────────────────────

    /// Attempt to identify a mining pool from the coinbase scriptSig hex.
    ///
    /// Many pools embed an ASCII tag in the coinbase field, e.g.:
    /// - Foundry USA: `/Foundry USA Pool`
    /// - AntPool: `/AntPool/`
    /// - F2Pool: `f2pool`
    /// - Binance Pool: `/Binance/`
    fn identify_pool_from_coinbase_hex(&self, hex: &str) -> Option<String> {
        // Decode hex bytes, interpret as lossy UTF-8 for tag scanning
        let bytes = (0..hex.len())
            .step_by(2)
            .filter_map(|i| {
                u8::from_str_radix(hex.get(i..i + 2)?, 16).ok()
            })
            .collect::<Vec<u8>>();

        let text = String::from_utf8_lossy(&bytes).to_lowercase();

        // Well-known pool signatures (case-insensitive)
        let known_pools = [
            ("foundry", "Foundry USA"),
            ("antpool", "AntPool"),
            ("f2pool", "F2Pool"),
            ("binance", "Binance Pool"),
            ("viabtc", "ViaBTC"),
            ("luxor", "Luxor"),
            ("mara pool", "MARA Pool"),
            ("marathon", "MARA Pool"),
            ("braiins", "Braiins Pool"),
            ("slush", "Braiins Pool"),
            ("poolin", "Poolin"),
            ("btc.com", "BTC.com"),
            ("1hash", "1Hash"),
            ("btcpool", "BTC Pool"),
            ("huobi", "Huobi Pool"),
            ("spider", "SpiderPool"),
            ("spiderpool", "SpiderPool"),
            ("secpool", "SecPool"),
            ("titan", "Titan"),
            ("sbicrypto", "SBICrypto"),
        ];

        for (tag, label) in &known_pools {
            if text.contains(tag) {
                return Some(label.to_string());
            }
        }

        None
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Conversion helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Convert a BTC float amount to a satoshi decimal string.
    ///
    /// Multiplies by 1e8 and rounds to avoid floating-point drift.
    fn btc_to_satoshis(btc: f64) -> String {
        let satoshis = (btc * 100_000_000.0).round() as u64;
        satoshis.to_string()
    }
}

impl Default for BitcoinEventDecoder {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_coinbase_tx(txid: &str, miner_addr: &str, reward_btc: f64) -> Value {
        json!({
            "txid": txid,
            "vin": [{ "coinbase": "deadbeef", "sequence": 4294967295u64 }],
            "vout": [{
                "value": reward_btc,
                "n": 0,
                "scriptPubKey": {
                    "type": "witness_v0_keyhash",
                    "address": miner_addr
                }
            }]
        })
    }

    fn make_regular_tx(txid: &str, from_addr: &str, to_addr: &str, value_btc: f64) -> Value {
        json!({
            "txid": txid,
            "vin": [{
                "txid": "aaaa",
                "vout": 0,
                "prevout": {
                    "value": value_btc,
                    "scriptPubKey": {
                        "type": "witness_v0_keyhash",
                        "address": from_addr
                    }
                }
            }],
            "vout": [{
                "value": value_btc,
                "n": 0,
                "scriptPubKey": {
                    "type": "witness_v0_keyhash",
                    "address": to_addr
                }
            }]
        })
    }

    #[test]
    fn test_is_coinbase_true() {
        let decoder = BitcoinEventDecoder::new();
        let tx = make_coinbase_tx("abc", "bc1qminer", 3.125);
        assert!(decoder.is_coinbase(&tx));
    }

    #[test]
    fn test_is_coinbase_false() {
        let decoder = BitcoinEventDecoder::new();
        let tx = make_regular_tx("abc", "bc1qfrom", "bc1qto", 0.5);
        assert!(!decoder.is_coinbase(&tx));
    }

    #[test]
    fn test_decode_coinbase_event() {
        let decoder = BitcoinEventDecoder::new();
        let tx = make_coinbase_tx("cbtxid", "bc1qminer123", 3.125);
        let events = decoder.decode_transaction(&tx, 840_000, 1_700_000_000);

        // Coinbase tx: 1 CoinbaseReward + 1 NativeTransfer (the output)
        assert_eq!(events.len(), 2);

        let reward_event = &events[0];
        assert_eq!(reward_event.block, 840_000);
        assert_eq!(reward_event.timestamp, 1_700_000_000);
        assert_eq!(reward_event.tx_hash, "cbtxid");

        match &reward_event.event_type {
            OnChainEventType::CoinbaseReward { miner, reward, pool } => {
                assert_eq!(miner, "bc1qminer123");
                assert_eq!(reward, "312500000"); // 3.125 BTC in satoshis
                assert!(pool.is_none());
            }
            _ => panic!("expected CoinbaseReward"),
        }
    }

    #[test]
    fn test_decode_regular_tx_utxo_spent() {
        let decoder = BitcoinEventDecoder::new();
        let tx = make_regular_tx("rxtxid", "bc1qfrom", "bc1qto", 0.5);
        let events = decoder.decode_transaction(&tx, 840_001, 1_700_000_100);

        // 1 UtxoSpent + 1 NativeTransfer
        assert_eq!(events.len(), 2);

        match &events[0].event_type {
            OnChainEventType::UtxoSpent { prev_tx, prev_index, spender, amount } => {
                assert_eq!(prev_tx, "aaaa");
                assert_eq!(*prev_index, 0);
                assert_eq!(spender, "bc1qfrom");
                assert_eq!(amount, "50000000"); // 0.5 BTC
            }
            _ => panic!("expected UtxoSpent"),
        }

        match &events[1].event_type {
            OnChainEventType::NativeTransfer { from, to, amount, .. } => {
                assert_eq!(from, "bc1qfrom");
                assert_eq!(to, "bc1qto");
                assert_eq!(amount, "50000000");
            }
            _ => panic!("expected NativeTransfer"),
        }
    }

    #[test]
    fn test_detect_op_return() {
        let decoder = BitcoinEventDecoder::new();
        let tx = json!({
            "txid": "opret",
            "vin": [{ "txid": "prev", "vout": 0 }],
            "vout": [
                {
                    "value": 0.0,
                    "n": 0,
                    "scriptPubKey": {
                        "type": "nulldata",
                        "hex": "6a14deadbeef1234"
                    }
                }
            ]
        });

        let result = decoder.detect_op_return(&tx);
        assert_eq!(result, Some("6a14deadbeef1234".to_string()));
    }

    #[test]
    fn test_decode_op_return_output() {
        let decoder = BitcoinEventDecoder::new();
        let tx = json!({
            "txid": "opret",
            "vin": [{ "txid": "prev", "vout": 0 }],
            "vout": [{
                "value": 0.0,
                "n": 0,
                "scriptPubKey": { "type": "nulldata", "hex": "6a14deadbeef" }
            }]
        });

        let events = decoder.decode_transaction(&tx, 1, 1);
        assert_eq!(events.len(), 1);

        match &events[0].event_type {
            OnChainEventType::NativeTransfer { to, .. } => {
                assert_eq!(to, "OP_RETURN");
            }
            _ => panic!("expected NativeTransfer for OP_RETURN output"),
        }

        // Raw field should contain op_return data
        assert!(events[0].raw.is_some());
    }

    #[test]
    fn test_mempool_tx_decode() {
        let decoder = BitcoinEventDecoder::new();
        let tx = json!({
            "txid": "memtx",
            "vin": [{ "txid": "prev", "vout": 0, "prevout": {
                "value": 0.001,
                "scriptPubKey": { "address": "bc1qsender" }
            }}],
            "vout": [{
                "value": 0.001,
                "n": 0,
                "scriptPubKey": { "type": "witness_v0_keyhash", "address": "bc1qrecipient" }
            }],
            "vsize": 141
        });

        let events = decoder.decode_mempool_tx(&tx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].block, 0);

        match &events[0].event_type {
            OnChainEventType::MempoolTransaction { from, to, value, data_size, .. } => {
                assert_eq!(from, "bc1qsender");
                assert_eq!(to.as_deref(), Some("bc1qrecipient"));
                assert_eq!(value, "100000"); // 0.001 BTC
                assert_eq!(*data_size, 141);
            }
            _ => panic!("expected MempoolTransaction"),
        }
    }

    #[test]
    fn test_known_label_on_coinbase() {
        let decoder = BitcoinEventDecoder::new()
            .with_label("bc1qantpool", "AntPool");

        let tx = make_coinbase_tx("cb2", "bc1qantpool", 3.125);
        let events = decoder.decode_transaction(&tx, 1, 1);

        match &events[0].event_type {
            OnChainEventType::CoinbaseReward { pool, .. } => {
                assert_eq!(pool.as_deref(), Some("AntPool"));
            }
            _ => panic!("expected CoinbaseReward"),
        }
    }

    #[test]
    fn test_empty_tx_returns_empty() {
        let decoder = BitcoinEventDecoder::new();
        let events = decoder.decode_transaction(&json!({}), 1, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn test_btc_to_satoshis() {
        assert_eq!(BitcoinEventDecoder::btc_to_satoshis(1.0), "100000000");
        assert_eq!(BitcoinEventDecoder::btc_to_satoshis(0.00000001), "1");
        assert_eq!(BitcoinEventDecoder::btc_to_satoshis(3.125), "312500000");
    }
}
