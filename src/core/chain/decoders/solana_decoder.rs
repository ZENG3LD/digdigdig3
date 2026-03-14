//! # SolanaEventDecoder — decode Solana transaction data into OnChainEvents
//!
//! Solana does not have Ethereum-style log topics. Events are inferred from:
//!
//! - **Inner instructions** — parsed program invocations with typed data
//! - **Log messages** — `"Program log: ..."` strings emitted by on-chain programs
//! - **Account balance changes** — pre/post SOL balance deltas → NativeTransfer
//! - **Pre/post token balances** — SPL token deltas → TokenTransfer
//!
//! ## Feature gate
//!
//! This module is gated behind the `onchain-solana` feature.

use serde_json::Value;

use crate::core::types::onchain::{ChainId, OnChainEvent, OnChainEventType, TokenAmount};

// ═══════════════════════════════════════════════════════════════════════════════
// SOLANA EVENT DECODER
// ═══════════════════════════════════════════════════════════════════════════════

/// Decodes Solana RPC transaction responses into [`OnChainEvent`] values.
///
/// Input format is the JSON response from `getTransaction` with
/// `encoding = "jsonParsed"` and `maxSupportedTransactionVersion = 0`.
///
/// ## Event detection strategy
///
/// 1. **SOL transfers**: compare `meta.preBalances` vs `meta.postBalances`
///    for each account key. Decreases are senders, increases are receivers.
///
/// 2. **SPL token transfers**: compare `meta.preTokenBalances` vs
///    `meta.postTokenBalances`. A decrease in one account paired with an
///    increase in another account of the same mint → `TokenTransfer`.
///
/// 3. **Swap detection**: scan `meta.logMessages` for known program log patterns
///    from Raydium and Jupiter.
///
/// 4. **Program invocations**: detect calls to known programs from the
///    `transaction.message.instructions` array.
pub struct SolanaEventDecoder {
    /// Chain this decoder is monitoring (`solana:mainnet-beta`, etc.).
    chain: ChainId,
}

// Well-known Solana program IDs
impl SolanaEventDecoder {
    /// SPL Token Program (fungible tokens).
    pub const TOKEN_PROGRAM: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    /// SPL Token-2022 Program (next-gen token standard).
    pub const TOKEN_2022_PROGRAM: &'static str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
    /// System Program (SOL transfers, account creation).
    pub const SYSTEM_PROGRAM: &'static str = "11111111111111111111111111111111";
    /// Raydium AMM v4.
    pub const RAYDIUM_AMM: &'static str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
    /// Raydium Concentrated Liquidity (CLMM).
    pub const RAYDIUM_CLMM: &'static str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
    /// Jupiter Aggregator v6.
    pub const JUPITER_V6: &'static str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
    /// Orca Whirlpools (CLMM).
    pub const ORCA_WHIRLPOOL: &'static str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
    /// Associated Token Account Program.
    pub const ASSOCIATED_TOKEN_PROGRAM: &'static str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bNX";
    /// Metaplex Token Metadata Program.
    pub const METADATA_PROGRAM: &'static str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
}

impl SolanaEventDecoder {
    /// Create a decoder for Solana mainnet-beta.
    pub fn new() -> Self {
        Self {
            chain: ChainId::new("solana", "mainnet-beta"),
        }
    }

    /// Create a decoder for Solana devnet.
    pub fn devnet() -> Self {
        Self {
            chain: ChainId::new("solana", "devnet"),
        }
    }

    /// Create a decoder for a specific cluster name.
    pub fn for_cluster(cluster: impl Into<String>) -> Self {
        Self {
            chain: ChainId::new("solana", cluster),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Public decode methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Decode a single Solana transaction into a list of [`OnChainEvent`]s.
    ///
    /// `tx` must be the JSON object returned by `getTransaction` with
    /// `encoding = "jsonParsed"` and full metadata.
    ///
    /// `slot` — the slot number of the containing block.
    /// `block_time` — Unix timestamp (seconds) when the block was produced.
    ///
    /// Returns an empty `Vec` if the transaction failed (err != null) or
    /// cannot be decoded. Never panics.
    pub fn decode_transaction(
        &self,
        tx: &Value,
        slot: u64,
        block_time: u64,
    ) -> Vec<OnChainEvent> {
        // Skip failed transactions
        if let Some(meta) = tx.get("meta") {
            if !meta.get("err").map_or(true, Value::is_null) {
                return vec![];
            }
        }

        let tx_hash = self.extract_signature(tx);
        let meta = match tx.get("meta") {
            Some(m) => m,
            None => return vec![],
        };

        let mut events = Vec::new();

        // 1. Decode SOL transfers from balance changes
        events.extend(self.decode_sol_transfers(tx, meta, &tx_hash, slot, block_time));

        // 2. Decode SPL token transfers from token balance changes
        events.extend(self.decode_token_transfers(tx, meta, &tx_hash, slot, block_time));

        // 3. Detect swaps from log messages
        let logs = self.extract_log_messages(meta);
        if let Some(swap_event) = self.detect_swap_from_logs(&logs, &tx_hash, slot, block_time) {
            events.push(swap_event);
        }

        events
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SOL transfer detection
    // ─────────────────────────────────────────────────────────────────────────

    /// Decode native SOL transfers from pre/post balance deltas.
    ///
    /// Compares `meta.preBalances[i]` vs `meta.postBalances[i]` for each
    /// account in `transaction.message.accountKeys`. Net-negative accounts
    /// are senders, net-positive accounts are receivers.
    ///
    /// To avoid generating spurious events from fee payments, the fee account
    /// (index 0, the fee payer) is filtered: only the portion of its balance
    /// decrease beyond the transaction fee is treated as a transfer.
    fn decode_sol_transfers(
        &self,
        tx: &Value,
        meta: &Value,
        tx_hash: &str,
        slot: u64,
        block_time: u64,
    ) -> Vec<OnChainEvent> {
        let account_keys = match self.extract_account_keys(tx) {
            Some(keys) => keys,
            None => return vec![],
        };

        let pre_balances = match meta.get("preBalances").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };
        let post_balances = match meta.get("postBalances").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };

        if pre_balances.len() != post_balances.len()
            || pre_balances.len() != account_keys.len()
        {
            return vec![];
        }

        let fee = meta.get("fee").and_then(Value::as_u64).unwrap_or(0);

        // Build balance change map: index → (address, delta_lamports as i64)
        let mut senders: Vec<(String, u64)> = Vec::new();
        let mut receivers: Vec<(String, u64)> = Vec::new();

        for i in 0..account_keys.len() {
            let pre = pre_balances[i].as_u64().unwrap_or(0);
            let post = post_balances[i].as_u64().unwrap_or(0);

            if pre > post {
                let mut decrease = pre - post;
                // Subtract fee from the fee payer (index 0) to avoid double-counting
                if i == 0 {
                    decrease = decrease.saturating_sub(fee);
                }
                if decrease > 0 {
                    senders.push((account_keys[i].clone(), decrease));
                }
            } else if post > pre {
                receivers.push((account_keys[i].clone(), post - pre));
            }
        }

        // Pair senders with receivers. For simplicity, emit one NativeTransfer
        // per (sender, receiver) pair. Use the first sender for all receivers
        // when there are multiple receivers (typical transfer pattern).
        let from = senders
            .first()
            .map(|(addr, _)| addr.clone())
            .unwrap_or_else(|| "unknown".to_string());

        receivers
            .into_iter()
            .enumerate()
            .filter(|(_, (_, amount))| *amount > 0)
            .map(|(idx, (to, amount))| OnChainEvent {
                chain: self.chain.clone(),
                block: slot,
                tx_hash: tx_hash.to_string(),
                log_index: Some(idx as u32),
                timestamp: block_time,
                event_type: OnChainEventType::NativeTransfer {
                    from: from.clone(),
                    to,
                    amount: amount.to_string(),
                    usd_value: None,
                },
                raw: None,
            })
            .collect()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SPL token transfer detection
    // ─────────────────────────────────────────────────────────────────────────

    /// Decode SPL token transfers from pre/post token balance changes.
    ///
    /// Compares `meta.preTokenBalances` vs `meta.postTokenBalances`. For each
    /// mint, finds accounts whose balance decreased (senders) and increased
    /// (receivers) and emits a `TokenTransfer` for each pair.
    fn decode_token_transfers(
        &self,
        tx: &Value,
        meta: &Value,
        tx_hash: &str,
        slot: u64,
        block_time: u64,
    ) -> Vec<OnChainEvent> {
        let account_keys = match self.extract_account_keys(tx) {
            Some(keys) => keys,
            None => return vec![],
        };

        let pre = meta
            .get("preTokenBalances")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let post = meta
            .get("postTokenBalances")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        // Build maps: (account_index, mint) → amount_string
        let pre_map = Self::build_token_balance_map(&pre);
        let post_map = Self::build_token_balance_map(&post);

        // Collect all (account_index, mint) keys
        let mut all_keys: Vec<(u64, String)> = pre_map.keys().cloned().collect();
        for key in post_map.keys() {
            if !all_keys.contains(key) {
                all_keys.push(key.clone());
            }
        }

        let mut events = Vec::new();

        for (group_idx, mint) in all_keys.iter().enumerate() {
            let pre_amount = pre_map
                .get(mint)
                .and_then(|v| v.parse::<u128>().ok())
                .unwrap_or(0);
            let post_amount = post_map
                .get(mint)
                .and_then(|v| v.parse::<u128>().ok())
                .unwrap_or(0);

            if pre_amount == post_amount {
                continue;
            }

            let (account_idx, mint_addr) = mint;
            let account_addr = account_keys
                .get(*account_idx as usize)
                .cloned()
                .unwrap_or_else(|| format!("account_{}", account_idx));

            // Find the owner (wallet) of this token account from the balance entry
            let owner = Self::find_token_owner(&pre, &post, *account_idx, mint_addr);

            if pre_amount > post_amount {
                // This account lost tokens (sender)
                let amount = pre_amount - post_amount;
                events.push(OnChainEvent {
                    chain: self.chain.clone(),
                    block: slot,
                    tx_hash: tx_hash.to_string(),
                    log_index: Some(group_idx as u32),
                    timestamp: block_time,
                    event_type: OnChainEventType::TokenTransfer {
                        token_address: mint_addr.clone(),
                        token_symbol: None,
                        from: owner.unwrap_or_else(|| account_addr.clone()),
                        to: "unknown".to_string(),
                        amount: amount.to_string(),
                        decimals: Self::find_token_decimals(&pre, &post, *account_idx),
                        usd_value: None,
                    },
                    raw: None,
                });
            } else {
                // This account gained tokens (receiver)
                let amount = post_amount - pre_amount;
                events.push(OnChainEvent {
                    chain: self.chain.clone(),
                    block: slot,
                    tx_hash: tx_hash.to_string(),
                    log_index: Some(group_idx as u32),
                    timestamp: block_time,
                    event_type: OnChainEventType::TokenTransfer {
                        token_address: mint_addr.clone(),
                        token_symbol: None,
                        from: "unknown".to_string(),
                        to: owner.unwrap_or_else(|| account_addr.clone()),
                        amount: amount.to_string(),
                        decimals: Self::find_token_decimals(&pre, &post, *account_idx),
                        usd_value: None,
                    },
                    raw: None,
                });
            }
        }

        // Pair senders and receivers for the same mint
        Self::pair_token_transfers(&mut events);

        events
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Swap detection from log messages
    // ─────────────────────────────────────────────────────────────────────────

    /// Detect a DEX swap from Solana transaction log messages.
    ///
    /// Scans `meta.logMessages` for patterns emitted by known protocols:
    ///
    /// - **Raydium AMM v4**: looks for `"Program 675kPX9... invoke"`
    /// - **Raydium CLMM**: looks for `"Program CAMMCzo5... invoke"`
    /// - **Jupiter v6**: looks for `"Program JUP6LkbZ... invoke"` and
    ///   `"Program log: Instruction: Route"` or `"swap"`
    /// - **Orca Whirlpool**: looks for `"Program whirLbMi... invoke"`
    ///
    /// When a swap is detected, emits a `DexSwap` event with the protocol
    /// identified. Token amounts are not parsed from logs (they require
    /// instruction data decoding with ABI schemas); use the `TokenTransfer`
    /// events from the same transaction for amounts.
    pub fn detect_swap_from_logs(
        &self,
        logs: &[String],
        tx_hash: &str,
        slot: u64,
        block_time: u64,
    ) -> Option<OnChainEvent> {
        let joined = logs.join("\n");

        let (protocol, pool_address) = if joined.contains(Self::JUPITER_V6) {
            ("jupiter_v6", Self::JUPITER_V6)
        } else if joined.contains(Self::RAYDIUM_CLMM) {
            ("raydium_clmm", Self::RAYDIUM_CLMM)
        } else if joined.contains(Self::RAYDIUM_AMM) {
            ("raydium_amm_v4", Self::RAYDIUM_AMM)
        } else if joined.contains(Self::ORCA_WHIRLPOOL) {
            ("orca_whirlpool", Self::ORCA_WHIRLPOOL)
        } else {
            return None;
        };

        // Verify there's actual swap activity (not just program invoke overhead)
        let is_swap = joined.to_lowercase().contains("swap")
            || joined.contains("Instruction: Route")
            || joined.contains("Instruction: Swap");

        if !is_swap {
            return None;
        }

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block: slot,
            tx_hash: tx_hash.to_string(),
            log_index: None,
            timestamp: block_time,
            event_type: OnChainEventType::DexSwap {
                protocol: protocol.to_string(),
                pool_address: pool_address.to_string(),
                token_in: TokenAmount::new("unknown", "0"),
                token_out: TokenAmount::new("unknown", "0"),
                sender: "unknown".to_string(),
                usd_volume: None,
            },
            raw: Some(serde_json::json!({ "logs": logs })),
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Extract the transaction signature (first entry in `transaction.signatures`).
    fn extract_signature(&self, tx: &Value) -> String {
        tx.get("transaction")
            .and_then(|t| t.get("signatures"))
            .and_then(Value::as_array)
            .and_then(|sigs| sigs.first())
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract the list of account pubkeys from the transaction message.
    ///
    /// Handles both legacy messages (flat `accountKeys` array of strings) and
    /// v0 messages with `accountKeys` as objects with a `pubkey` field.
    fn extract_account_keys(&self, tx: &Value) -> Option<Vec<String>> {
        let keys = tx
            .get("transaction")
            .and_then(|t| t.get("message"))
            .and_then(|m| m.get("accountKeys"))
            .and_then(Value::as_array)?;

        Some(
            keys.iter()
                .map(|k| {
                    // Object form: { "pubkey": "...", "signer": bool, "writable": bool }
                    if let Some(pubkey) = k.get("pubkey").and_then(Value::as_str) {
                        pubkey.to_string()
                    } else {
                        // String form (legacy)
                        k.as_str().unwrap_or("unknown").to_string()
                    }
                })
                .collect(),
        )
    }

    /// Extract log messages from transaction metadata.
    fn extract_log_messages(&self, meta: &Value) -> Vec<String> {
        meta.get("logMessages")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Build a map of `(account_index, mint)` → `amount_string` from token balance entries.
    fn build_token_balance_map(balances: &[Value]) -> std::collections::HashMap<(u64, String), String> {
        let mut map = std::collections::HashMap::new();

        for entry in balances {
            let account_index = entry.get("accountIndex").and_then(Value::as_u64);
            let mint = entry.get("mint").and_then(Value::as_str);
            let amount = entry
                .get("uiTokenAmount")
                .and_then(|u| u.get("amount"))
                .and_then(Value::as_str);

            if let (Some(idx), Some(mint), Some(amount)) = (account_index, mint, amount) {
                map.insert((idx, mint.to_string()), amount.to_string());
            }
        }

        map
    }

    /// Find the wallet owner of a token account from balance entries.
    fn find_token_owner(
        pre: &[Value],
        post: &[Value],
        account_idx: u64,
        mint: &str,
    ) -> Option<String> {
        for entry in pre.iter().chain(post.iter()) {
            let idx = entry.get("accountIndex").and_then(Value::as_u64)?;
            let entry_mint = entry.get("mint").and_then(Value::as_str)?;
            if idx == account_idx && entry_mint == mint {
                return entry.get("owner").and_then(Value::as_str).map(str::to_string);
            }
        }
        None
    }

    /// Find the decimal places for a token from balance entries.
    fn find_token_decimals(pre: &[Value], post: &[Value], account_idx: u64) -> Option<u8> {
        for entry in pre.iter().chain(post.iter()) {
            let idx = entry.get("accountIndex").and_then(Value::as_u64)?;
            if idx == account_idx {
                return entry
                    .get("uiTokenAmount")
                    .and_then(|u| u.get("decimals"))
                    .and_then(Value::as_u64)
                    .map(|d| d as u8);
            }
        }
        None
    }

    /// Pair sender and receiver `TokenTransfer` events for the same mint.
    ///
    /// When a token transfer involves one sender and one receiver of the same
    /// mint, replace the `"unknown"` placeholders with the actual addresses.
    fn pair_token_transfers(events: &mut Vec<OnChainEvent>) {
        // Collect indices of sender (from != unknown) and receiver (to != unknown) events
        // grouped by token_address
        use std::collections::HashMap;

        let mut senders_by_mint: HashMap<String, Vec<usize>> = HashMap::new();
        let mut receivers_by_mint: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, event) in events.iter().enumerate() {
            if let OnChainEventType::TokenTransfer { token_address, from, to, .. } = &event.event_type {
                if to == "unknown" {
                    senders_by_mint
                        .entry(token_address.clone())
                        .or_default()
                        .push(i);
                } else if from == "unknown" {
                    receivers_by_mint
                        .entry(token_address.clone())
                        .or_default()
                        .push(i);
                }
            }
        }

        // For each mint with exactly one sender and one receiver, merge them
        for (mint, sender_indices) in &senders_by_mint {
            if let Some(receiver_indices) = receivers_by_mint.get(mint) {
                if sender_indices.len() == 1 && receiver_indices.len() == 1 {
                    let sender_idx = sender_indices[0];
                    let receiver_idx = receiver_indices[0];

                    // Extract sender address before mutating
                    let sender_addr = if let OnChainEventType::TokenTransfer { from, .. } = &events[sender_idx].event_type {
                        from.clone()
                    } else {
                        continue;
                    };

                    // Update receiver's `from` field
                    if let OnChainEventType::TokenTransfer { from, .. } = &mut events[receiver_idx].event_type {
                        *from = sender_addr.clone();
                    }

                    // Mark sender event for removal by setting amount to "0"
                    // (we'll filter it out — the merged receiver event is canonical)
                    if let OnChainEventType::TokenTransfer { amount, to, .. } = &mut events[sender_idx].event_type {
                        // Extract receiver address
                        let receiver_addr = if let OnChainEventType::TokenTransfer { to: recv_to, .. } = &events[receiver_idx].event_type {
                            recv_to.clone()
                        } else {
                            continue;
                        };
                        *to = receiver_addr;
                        let _ = amount; // keep as-is
                    }
                }
            }
        }

        // Remove duplicate TokenTransfer events where from == to (paired and collapsed above)
        events.retain(|e| {
            if let OnChainEventType::TokenTransfer { from, to, .. } = &e.event_type {
                !(from == "unknown" && to != "unknown")
                    && !(from != "unknown" && to == "unknown" && from == to)
            } else {
                true
            }
        });
    }
}

impl Default for SolanaEventDecoder {
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

    fn make_sol_transfer_tx(sig: &str, from: &str, to: &str, lamports: u64) -> Value {
        json!({
            "transaction": {
                "signatures": [sig],
                "message": {
                    "accountKeys": [
                        { "pubkey": from, "signer": true, "writable": true },
                        { "pubkey": to, "signer": false, "writable": true },
                        { "pubkey": "11111111111111111111111111111111", "signer": false, "writable": false }
                    ]
                }
            },
            "meta": {
                "err": null,
                "fee": 5000u64,
                "preBalances": [lamports + 5000u64, 0u64, 1u64],
                "postBalances": [0u64, lamports, 1u64],
                "logMessages": ["Program 11111111111111111111111111111111 invoke [1]", "Program 11111111111111111111111111111111 success"],
                "preTokenBalances": [],
                "postTokenBalances": []
            }
        })
    }

    fn make_jupiter_swap_tx(sig: &str) -> Value {
        json!({
            "transaction": {
                "signatures": [sig],
                "message": {
                    "accountKeys": [
                        { "pubkey": "trader111", "signer": true, "writable": true },
                        { "pubkey": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", "signer": false, "writable": false }
                    ]
                }
            },
            "meta": {
                "err": null,
                "fee": 5000u64,
                "preBalances": [1_000_000u64, 0u64],
                "postBalances": [995_000u64, 0u64],
                "logMessages": [
                    "Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 invoke [1]",
                    "Program log: Instruction: Route",
                    "Program log: swap",
                    "Program JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 success"
                ],
                "preTokenBalances": [],
                "postTokenBalances": []
            }
        })
    }

    #[test]
    fn test_decode_sol_transfer() {
        let decoder = SolanaEventDecoder::new();
        let tx = make_sol_transfer_tx("sig1", "wallet_a", "wallet_b", 1_000_000);
        let events = decoder.decode_transaction(&tx, 250_000_000, 1_700_000_000);

        // Should produce at least one NativeTransfer
        let transfers: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.event_type, OnChainEventType::NativeTransfer { .. }))
            .collect();

        assert!(!transfers.is_empty(), "expected NativeTransfer events");

        let t = &transfers[0];
        assert_eq!(t.block, 250_000_000);
        assert_eq!(t.tx_hash, "sig1");

        match &t.event_type {
            OnChainEventType::NativeTransfer { from, to, amount, .. } => {
                assert_eq!(from, "wallet_a");
                assert_eq!(to, "wallet_b");
                assert_eq!(amount, "1000000");
            }
            _ => panic!("expected NativeTransfer"),
        }
    }

    #[test]
    fn test_detect_jupiter_swap() {
        let decoder = SolanaEventDecoder::new();
        let tx = make_jupiter_swap_tx("swapsig");
        let events = decoder.decode_transaction(&tx, 250_000_001, 1_700_000_001);

        let swaps: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.event_type, OnChainEventType::DexSwap { .. }))
            .collect();

        assert!(!swaps.is_empty(), "expected DexSwap event");

        match &swaps[0].event_type {
            OnChainEventType::DexSwap { protocol, pool_address, .. } => {
                assert_eq!(protocol, "jupiter_v6");
                assert_eq!(pool_address, SolanaEventDecoder::JUPITER_V6);
            }
            _ => panic!("expected DexSwap"),
        }
    }

    #[test]
    fn test_failed_tx_returns_empty() {
        let decoder = SolanaEventDecoder::new();
        let tx = json!({
            "transaction": { "signatures": ["failsig"], "message": { "accountKeys": [] } },
            "meta": {
                "err": { "InstructionError": [0, "InvalidArgument"] },
                "fee": 5000u64,
                "preBalances": [],
                "postBalances": [],
                "logMessages": [],
                "preTokenBalances": [],
                "postTokenBalances": []
            }
        });

        let events = decoder.decode_transaction(&tx, 1, 1);
        assert!(events.is_empty(), "failed tx should produce no events");
    }

    #[test]
    fn test_empty_tx_returns_empty() {
        let decoder = SolanaEventDecoder::new();
        let events = decoder.decode_transaction(&json!({}), 1, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn test_detect_swap_from_logs_raydium() {
        let decoder = SolanaEventDecoder::new();
        let logs = vec![
            "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 invoke [1]".to_string(),
            "Program log: swap".to_string(),
        ];

        let event = decoder.detect_swap_from_logs(&logs, "txhash", 1, 1);
        assert!(event.is_some());

        match &event.unwrap().event_type {
            OnChainEventType::DexSwap { protocol, .. } => {
                assert_eq!(protocol, "raydium_amm_v4");
            }
            _ => panic!("expected DexSwap"),
        }
    }

    #[test]
    fn test_detect_swap_no_match() {
        let decoder = SolanaEventDecoder::new();
        let logs = vec![
            "Program 11111111111111111111111111111111 invoke [1]".to_string(),
            "Program 11111111111111111111111111111111 success".to_string(),
        ];

        let event = decoder.detect_swap_from_logs(&logs, "txhash", 1, 1);
        assert!(event.is_none());
    }

    #[test]
    fn test_chain_id_mainnet() {
        let decoder = SolanaEventDecoder::new();
        let tx = make_sol_transfer_tx("s1", "a", "b", 1000);
        let events = decoder.decode_transaction(&tx, 1, 1);
        for e in &events {
            assert_eq!(e.chain.family, "solana");
            assert_eq!(e.chain.network, "mainnet-beta");
        }
    }
}
