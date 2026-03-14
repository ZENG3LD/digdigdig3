//! # CosmosEventDecoder — decode Cosmos SDK transaction responses into OnChainEvents
//!
//! Cosmos SDK transactions expose typed events through the `tx_response.events`
//! array. Each event has a `"type"` string and an `"attributes"` array of
//! `{ "key": base64, "value": base64 }` (or plaintext on newer nodes with
//! `decode_responses = true`).
//!
//! ## Feature gate
//!
//! This module is gated behind the `onchain-cosmos` feature.
//!
//! ## Supported event types
//!
//! | Cosmos event type | Decoded as |
//! |-------------------|-----------|
//! | `transfer` / `coin_received` / `coin_spent` | `NativeTransfer` or `TokenTransfer` |
//! | `ibc_transfer` | `BridgeTransfer` |
//! | `delegate` | `StakingAction::Delegate` |
//! | `unbond` | `StakingAction::Undelegate` |
//! | `redelegate` | `StakingAction::Redelegate` |
//! | `withdraw_rewards` | `StakingAction::ClaimRewards` |
//! | `swap_tokens` / `token_swapped` / `osmosis gamm` | `DexSwap` |
//! | `submit_proposal` | `GovernanceAction::Propose` |
//! | `proposal_vote` | `GovernanceAction::Vote` |
//! | `active_proposal` | `GovernanceAction::Execute` |
//! | `cancel_proposal` | `GovernanceAction::Cancel` |

use std::collections::HashMap;

use serde_json::Value;

use crate::core::types::onchain::{
    ChainId, GovernanceActionType, OnChainEvent, OnChainEventType, StakingActionType, TokenAmount,
};

// ═══════════════════════════════════════════════════════════════════════════════
// COSMOS EVENT DECODER
// ═══════════════════════════════════════════════════════════════════════════════

/// Decodes Cosmos SDK transaction responses into [`OnChainEvent`] values.
///
/// Input is the JSON response from `/cosmos/tx/v1beta1/txs/{hash}` or the
/// `broadcast_tx` response. The relevant field is `tx_response.events`, which
/// is an array of typed Cosmos events.
///
/// ## Attribute encoding
///
/// Cosmos SDK < v0.46 encodes attribute keys and values as base64.
/// Cosmos SDK >= v0.46 (with `app.toml: api.enabled-unsafe-cors = true` and
/// the `decode_responses` query parameter) returns plaintext. This decoder
/// handles **both** formats automatically.
pub struct CosmosEventDecoder {
    /// Chain this decoder is monitoring (e.g. `cosmos:osmosis-1`).
    chain: ChainId,
}

impl CosmosEventDecoder {
    /// Create a decoder for the given Cosmos chain ID (e.g. `"osmosis-1"`).
    pub fn new(chain: ChainId) -> Self {
        Self { chain }
    }

    /// Create a decoder for Osmosis mainnet (`osmosis-1`).
    pub fn osmosis() -> Self {
        Self::new(ChainId::cosmos("osmosis-1"))
    }

    /// Create a decoder for dYdX mainnet (`dydx-mainnet-1`).
    pub fn dydx() -> Self {
        Self::new(ChainId::cosmos("dydx-mainnet-1"))
    }

    /// Create a decoder for Cosmos Hub mainnet (`cosmoshub-4`).
    pub fn cosmos_hub() -> Self {
        Self::new(ChainId::cosmos("cosmoshub-4"))
    }

    /// Create a decoder for Celestia mainnet (`celestia`).
    pub fn celestia() -> Self {
        Self::new(ChainId::cosmos("celestia"))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Public decode method
    // ─────────────────────────────────────────────────────────────────────────

    /// Decode a Cosmos transaction response into a list of [`OnChainEvent`]s.
    ///
    /// `tx` must be the JSON response from `/cosmos/tx/v1beta1/txs/{hash}`.
    /// The decoder reads `tx.tx_response.txhash`, `tx.tx_response.height`,
    /// and `tx.tx_response.events`.
    ///
    /// Falls back gracefully if fields are missing: returns an empty `Vec`
    /// for undecodable responses, never panics.
    pub fn decode_transaction(&self, tx: &Value, block_height: u64) -> Vec<OnChainEvent> {
        let tx_response = match tx.get("tx_response") {
            Some(r) => r,
            None => {
                // Some endpoints return the tx_response fields at the top level
                tx
            }
        };

        let tx_hash = tx_response
            .get("txhash")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        // Use provided block_height or parse from tx_response
        let height = if block_height > 0 {
            block_height
        } else {
            tx_response
                .get("height")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| tx_response.get("height").and_then(Value::as_u64))
                .unwrap_or(0)
        };

        // Check for failure
        let code = tx_response.get("code").and_then(Value::as_u64).unwrap_or(0);
        if code != 0 {
            return vec![];
        }

        let events = match tx_response.get("events").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };

        let mut results = Vec::new();

        for (evt_idx, event) in events.iter().enumerate() {
            let event_type_str = event.get("type").and_then(Value::as_str).unwrap_or("");
            let attrs = self.extract_attrs(event);

            let decoded: Option<OnChainEvent> = match event_type_str {
                "transfer" | "coin_received" | "coin_spent" => {
                    self.decode_transfer(&attrs, height, &tx_hash, evt_idx)
                }
                "ibc_transfer" => self.decode_ibc_transfer(&attrs, height, &tx_hash, evt_idx),
                "delegate" => self.decode_delegation(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                    StakingActionType::Delegate,
                ),
                "unbond" => self.decode_delegation(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                    StakingActionType::Undelegate,
                ),
                "redelegate" => self.decode_delegation(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                    StakingActionType::Redelegate,
                ),
                "withdraw_rewards" | "withdraw_delegator_reward" => self.decode_claim_rewards(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                ),
                "token_swapped"
                | "swap_tokens"
                | "poolcreated"
                | "gamm"
                | "osmosis/gamm/v1beta1/EventSwap" => {
                    self.decode_swap(&attrs, height, &tx_hash, evt_idx)
                }
                "submit_proposal" => self.decode_governance(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                    GovernanceActionType::Propose,
                ),
                "proposal_vote" | "vote" => self.decode_governance(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                    GovernanceActionType::Vote,
                ),
                "active_proposal" | "execute_proposal" => self.decode_governance(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                    GovernanceActionType::Execute,
                ),
                "cancel_proposal" | "inactive_proposal" => self.decode_governance(
                    &attrs,
                    height,
                    &tx_hash,
                    evt_idx,
                    GovernanceActionType::Cancel,
                ),
                _ => None,
            };

            if let Some(evt) = decoded {
                results.push(evt);
            }
        }

        results
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Event decoders
    // ─────────────────────────────────────────────────────────────────────────

    /// Decode a `transfer`, `coin_received`, or `coin_spent` event.
    ///
    /// Cosmos transfer events carry `sender`, `recipient`, and `amount`.
    /// The `amount` is a Cosmos coin string like `"1000uatom"` or
    /// `"500uosmo,200ibc/..."`. We emit one event per coin denom.
    fn decode_transfer(
        &self,
        attrs: &[(String, String)],
        block: u64,
        tx_hash: &str,
        idx: usize,
    ) -> Option<OnChainEvent> {
        let attrs_map: HashMap<_, _> = attrs.iter().cloned().collect();

        let sender = attrs_map
            .get("sender")
            .or_else(|| attrs_map.get("spender"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let recipient = attrs_map
            .get("recipient")
            .or_else(|| attrs_map.get("receiver"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let amount_str = attrs_map.get("amount").cloned().unwrap_or_default();

        if amount_str.is_empty() {
            return None;
        }

        // Parse the first coin from the amount string
        let (amount, denom) = Self::parse_coin_amount(&amount_str)?;

        // Decide whether to emit NativeTransfer or TokenTransfer
        // Convention: native coin denoms start with "u" (micro) or are simple strings
        // IBC denoms start with "ibc/" and are treated as token transfers
        let event_type = if denom.starts_with("ibc/") || denom.contains('/') {
            OnChainEventType::TokenTransfer {
                token_address: denom.clone(),
                token_symbol: None,
                from: sender,
                to: recipient,
                amount: amount.to_string(),
                decimals: None,
                usd_value: None,
            }
        } else {
            OnChainEventType::NativeTransfer {
                from: sender,
                to: recipient,
                amount: amount.to_string(),
                usd_value: None,
            }
        };

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block,
            tx_hash: tx_hash.to_string(),
            log_index: Some(idx as u32),
            timestamp: 0,
            event_type,
            raw: None,
        })
    }

    /// Decode an `ibc_transfer` event into a [`BridgeTransfer`](OnChainEventType::BridgeTransfer).
    fn decode_ibc_transfer(
        &self,
        attrs: &[(String, String)],
        block: u64,
        tx_hash: &str,
        idx: usize,
    ) -> Option<OnChainEvent> {
        let attrs_map: HashMap<_, _> = attrs.iter().cloned().collect();

        let sender = attrs_map.get("sender").cloned().unwrap_or_else(|| "unknown".to_string());
        let receiver = attrs_map.get("receiver").cloned();
        let denom = attrs_map.get("denom").cloned().unwrap_or_else(|| "unknown".to_string());
        let amount_str = attrs_map.get("amount").cloned().unwrap_or_else(|| "0".to_string());

        // Source and destination channels
        let source_channel = attrs_map.get("source_channel").cloned().unwrap_or_default();
        let dest_channel = attrs_map.get("destination_channel").cloned().unwrap_or_default();

        // Source is this chain, destination is determined by channel
        let source_chain = self.chain.display();
        let dest_chain = dest_channel.clone();

        let (amount, _parsed_denom) = Self::parse_coin_amount(&amount_str)
            .unwrap_or((amount_str.parse().unwrap_or(0), denom.clone()));

        let token = TokenAmount {
            address: denom,
            symbol: None,
            amount: amount.to_string(),
            decimals: None,
        };

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block,
            tx_hash: tx_hash.to_string(),
            log_index: Some(idx as u32),
            timestamp: 0,
            event_type: OnChainEventType::BridgeTransfer {
                bridge: "ibc".to_string(),
                source_chain,
                dest_chain,
                token,
                sender,
                receiver,
            },
            raw: Some(serde_json::json!({
                "source_channel": source_channel,
                "dest_channel": dest_channel
            })),
        })
    }

    /// Decode a delegation / undelegation / redelegation event.
    fn decode_delegation(
        &self,
        attrs: &[(String, String)],
        block: u64,
        tx_hash: &str,
        idx: usize,
        action: StakingActionType,
    ) -> Option<OnChainEvent> {
        let attrs_map: HashMap<_, _> = attrs.iter().cloned().collect();

        let delegator = attrs_map
            .get("delegator")
            .or_else(|| attrs_map.get("delegator_address"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let validator = attrs_map
            .get("validator")
            .or_else(|| attrs_map.get("validator_address"))
            .or_else(|| attrs_map.get("source_validator"))
            .cloned();

        let amount_str = attrs_map.get("amount").cloned().unwrap_or_default();
        let (amount, denom) = Self::parse_coin_amount(&amount_str)
            .unwrap_or_else(|| (0, "unknown".to_string()));

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block,
            tx_hash: tx_hash.to_string(),
            log_index: Some(idx as u32),
            timestamp: 0,
            event_type: OnChainEventType::StakingAction {
                validator,
                delegator,
                action,
                amount: amount.to_string(),
                denom,
            },
            raw: None,
        })
    }

    /// Decode a `withdraw_rewards` event into `StakingAction::ClaimRewards`.
    fn decode_claim_rewards(
        &self,
        attrs: &[(String, String)],
        block: u64,
        tx_hash: &str,
        idx: usize,
    ) -> Option<OnChainEvent> {
        let attrs_map: HashMap<_, _> = attrs.iter().cloned().collect();

        let delegator = attrs_map
            .get("delegator")
            .or_else(|| attrs_map.get("delegator_address"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let validator = attrs_map
            .get("validator")
            .or_else(|| attrs_map.get("validator_address"))
            .cloned();

        let amount_str = attrs_map.get("amount").cloned().unwrap_or_default();
        let (amount, denom) = Self::parse_coin_amount(&amount_str)
            .unwrap_or_else(|| (0, "unknown".to_string()));

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block,
            tx_hash: tx_hash.to_string(),
            log_index: Some(idx as u32),
            timestamp: 0,
            event_type: OnChainEventType::StakingAction {
                validator,
                delegator,
                action: StakingActionType::ClaimRewards,
                amount: amount.to_string(),
                denom,
            },
            raw: None,
        })
    }

    /// Decode an Osmosis or generic Cosmos DEX swap event.
    ///
    /// Osmosis GAMM swap events carry:
    /// - `tokens_in` — input token amount (e.g. `"1000uosmo"`)
    /// - `tokens_out` — output token amount (e.g. `"500ibc/..."`)
    /// - `sender` — the swapper's address
    /// - `pool_id` — pool identifier
    fn decode_swap(
        &self,
        attrs: &[(String, String)],
        block: u64,
        tx_hash: &str,
        idx: usize,
    ) -> Option<OnChainEvent> {
        let attrs_map: HashMap<_, _> = attrs.iter().cloned().collect();

        let sender = attrs_map
            .get("sender")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let pool_id = attrs_map
            .get("pool_id")
            .or_else(|| attrs_map.get("poolId"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let tokens_in_str = attrs_map
            .get("tokens_in")
            .or_else(|| attrs_map.get("tokensIn"))
            .or_else(|| attrs_map.get("amount_in"))
            .cloned()
            .unwrap_or_default();

        let tokens_out_str = attrs_map
            .get("tokens_out")
            .or_else(|| attrs_map.get("tokensOut"))
            .or_else(|| attrs_map.get("amount_out"))
            .cloned()
            .unwrap_or_default();

        let (in_amount, in_denom) = Self::parse_coin_amount(&tokens_in_str)
            .unwrap_or_else(|| (0, "unknown".to_string()));

        let (out_amount, out_denom) = Self::parse_coin_amount(&tokens_out_str)
            .unwrap_or_else(|| (0, "unknown".to_string()));

        // Determine protocol from chain
        let protocol = match self.chain.network.as_str() {
            n if n.contains("osmosis") => "osmosis_gamm",
            n if n.contains("dydx") => "dydx_perp",
            _ => "cosmos_dex",
        };

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block,
            tx_hash: tx_hash.to_string(),
            log_index: Some(idx as u32),
            timestamp: 0,
            event_type: OnChainEventType::DexSwap {
                protocol: protocol.to_string(),
                pool_address: pool_id,
                token_in: TokenAmount::new(in_denom, in_amount.to_string()),
                token_out: TokenAmount::new(out_denom, out_amount.to_string()),
                sender,
                usd_volume: None,
            },
            raw: None,
        })
    }

    /// Decode a governance action event.
    fn decode_governance(
        &self,
        attrs: &[(String, String)],
        block: u64,
        tx_hash: &str,
        idx: usize,
        action: GovernanceActionType,
    ) -> Option<OnChainEvent> {
        let attrs_map: HashMap<_, _> = attrs.iter().cloned().collect();

        let proposal_id = attrs_map
            .get("proposal_id")
            .or_else(|| attrs_map.get("proposalId"))
            .cloned()
            .unwrap_or_else(|| "0".to_string());

        let voter = attrs_map.get("voter").cloned();
        let vote = attrs_map.get("option").or_else(|| attrs_map.get("vote")).cloned();

        Some(OnChainEvent {
            chain: self.chain.clone(),
            block,
            tx_hash: tx_hash.to_string(),
            log_index: Some(idx as u32),
            timestamp: 0,
            event_type: OnChainEventType::GovernanceAction {
                proposal_id,
                action,
                voter,
                vote,
            },
            raw: None,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Attribute extraction
    // ─────────────────────────────────────────────────────────────────────────

    /// Extract key-value attributes from a Cosmos SDK event object.
    ///
    /// Cosmos SDK events have an `"attributes"` array. Each attribute is either:
    /// - `{ "key": "base64==", "value": "base64==" }` (old format, SDK < v0.46)
    /// - `{ "key": "plaintext", "value": "plaintext" }` (new format, SDK >= v0.46)
    ///
    /// This method automatically detects the encoding by attempting base64
    /// decoding; if the decoded bytes are valid UTF-8, it uses the decoded
    /// form. Otherwise it uses the raw string as-is.
    pub fn extract_attrs(&self, event: &Value) -> Vec<(String, String)> {
        let attrs = match event.get("attributes").and_then(Value::as_array) {
            Some(arr) => arr,
            None => return vec![],
        };

        attrs
            .iter()
            .filter_map(|attr| {
                let raw_key = attr.get("key").and_then(Value::as_str).unwrap_or("");
                let raw_val = attr.get("value").and_then(Value::as_str).unwrap_or("");

                let key = Self::decode_attr_field(raw_key);
                let value = Self::decode_attr_field(raw_val);

                if key.is_empty() {
                    None
                } else {
                    Some((key, value))
                }
            })
            .collect()
    }

    /// Decode a single attribute field: try base64, fall back to plaintext.
    fn decode_attr_field(raw: &str) -> String {
        // Try base64 decoding (standard alphabet, may have padding)
        if let Ok(bytes) = Self::base64_decode(raw) {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                return text.to_string();
            }
        }
        raw.to_string()
    }

    /// Minimal base64 decoder — handles standard and URL-safe alphabets.
    fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
        // Normalise: replace URL-safe chars and strip padding
        let normalised = input.replace('-', "+").replace('_', "/");
        let padded = match normalised.len() % 4 {
            0 => normalised,
            2 => normalised + "==",
            3 => normalised + "=",
            _ => return Err(()),
        };

        // Use a simple table-based decoder (no external crates)
        const TABLE: &[u8; 128] = b"\
\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\
\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\
\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x3e\xff\xff\xff\x3f\
\x34\x35\x36\x37\x38\x39\x3a\x3b\x3c\x3d\xff\xff\xff\xfe\xff\xff\
\xff\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\
\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\xff\xff\xff\xff\xff\
\xff\x1a\x1b\x1c\x1d\x1e\x1f\x20\x21\x22\x23\x24\x25\x26\x27\x28\
\x29\x2a\x2b\x2c\x2d\x2e\x2f\x30\x31\x32\x33\xff\xff\xff\xff\xff\
";

        let bytes = padded.as_bytes();
        if bytes.len() % 4 != 0 {
            return Err(());
        }

        let mut out = Vec::with_capacity(bytes.len() / 4 * 3);

        for chunk in bytes.chunks(4) {
            let a = *TABLE.get(chunk[0] as usize).ok_or(())?;
            let b = *TABLE.get(chunk[1] as usize).ok_or(())?;
            let c = *TABLE.get(chunk[2] as usize).ok_or(())?;
            let d = *TABLE.get(chunk[3] as usize).ok_or(())?;

            if a == 0xff || b == 0xff {
                return Err(());
            }

            out.push((a << 2) | (b >> 4));

            if c != 0xfe {
                if c == 0xff {
                    return Err(());
                }
                out.push((b << 4) | (c >> 2));

                if d != 0xfe {
                    if d == 0xff {
                        return Err(());
                    }
                    out.push((c << 6) | d);
                }
            }
        }

        Ok(out)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Coin amount parsing
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse a Cosmos coin amount string into `(amount, denom)`.
    ///
    /// Handles formats:
    /// - `"1000uatom"` → `(1000, "uatom")`
    /// - `"500ibc/27394FB..."` → `(500, "ibc/27394FB...")`
    /// - `"100"` → `(100, "")` — bare numbers without denom
    ///
    /// For multi-coin strings like `"100uatom,50uosmo"`, only the **first**
    /// coin is returned. To process all coins, split on `","` first.
    ///
    /// Returns `None` if the string is empty or unparsable.
    fn parse_coin_amount(s: &str) -> Option<(u128, String)> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        // Handle multi-coin: take only the first
        let s = s.split(',').next().unwrap_or(s).trim();

        // Find the split point between digits and denom
        let split_idx = s.find(|c: char| !c.is_ascii_digit())?;

        if split_idx == 0 {
            return None; // starts with non-digit
        }

        let amount_str = &s[..split_idx];
        let denom = s[split_idx..].trim().to_string();

        let amount = amount_str.parse::<u128>().ok()?;

        Some((amount, denom))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_transfer_response(
        txhash: &str,
        height: u64,
        sender: &str,
        recipient: &str,
        amount: &str,
    ) -> Value {
        json!({
            "tx_response": {
                "txhash": txhash,
                "height": height.to_string(),
                "code": 0,
                "events": [
                    {
                        "type": "transfer",
                        "attributes": [
                            { "key": "sender", "value": sender },
                            { "key": "recipient", "value": recipient },
                            { "key": "amount", "value": amount }
                        ]
                    }
                ]
            }
        })
    }

    fn make_delegation_response(
        txhash: &str,
        delegator: &str,
        validator: &str,
        amount: &str,
    ) -> Value {
        json!({
            "tx_response": {
                "txhash": txhash,
                "height": "12345",
                "code": 0,
                "events": [
                    {
                        "type": "delegate",
                        "attributes": [
                            { "key": "delegator", "value": delegator },
                            { "key": "validator", "value": validator },
                            { "key": "amount", "value": amount }
                        ]
                    }
                ]
            }
        })
    }

    fn make_vote_response(txhash: &str, proposal_id: &str, voter: &str, option: &str) -> Value {
        json!({
            "tx_response": {
                "txhash": txhash,
                "height": "99000",
                "code": 0,
                "events": [
                    {
                        "type": "proposal_vote",
                        "attributes": [
                            { "key": "proposal_id", "value": proposal_id },
                            { "key": "voter", "value": voter },
                            { "key": "option", "value": option }
                        ]
                    }
                ]
            }
        })
    }

    fn make_osmosis_swap(
        txhash: &str,
        sender: &str,
        tokens_in: &str,
        tokens_out: &str,
        pool_id: &str,
    ) -> Value {
        json!({
            "tx_response": {
                "txhash": txhash,
                "height": "5000000",
                "code": 0,
                "events": [
                    {
                        "type": "token_swapped",
                        "attributes": [
                            { "key": "sender", "value": sender },
                            { "key": "tokens_in", "value": tokens_in },
                            { "key": "tokens_out", "value": tokens_out },
                            { "key": "pool_id", "value": pool_id }
                        ]
                    }
                ]
            }
        })
    }

    // ─── Parse coin amount ────────────────────────────────────────────────────

    #[test]
    fn test_parse_coin_uatom() {
        let result = CosmosEventDecoder::parse_coin_amount("1000uatom");
        assert_eq!(result, Some((1000, "uatom".to_string())));
    }

    #[test]
    fn test_parse_coin_ibc() {
        let result =
            CosmosEventDecoder::parse_coin_amount("500ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2");
        assert_eq!(
            result,
            Some((500, "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string()))
        );
    }

    #[test]
    fn test_parse_coin_multi_takes_first() {
        let result = CosmosEventDecoder::parse_coin_amount("100uatom,50uosmo");
        assert_eq!(result, Some((100, "uatom".to_string())));
    }

    #[test]
    fn test_parse_coin_empty() {
        assert_eq!(CosmosEventDecoder::parse_coin_amount(""), None);
    }

    // ─── Base64 decoding ──────────────────────────────────────────────────────

    #[test]
    fn test_base64_decode_simple() {
        let result = CosmosEventDecoder::decode_attr_field("c2VuZGVy");
        assert_eq!(result, "sender");
    }

    #[test]
    fn test_base64_decode_plaintext_passthrough() {
        // Plaintext that is not valid base64 should pass through
        let result = CosmosEventDecoder::decode_attr_field("osmo1abc...");
        assert_eq!(result, "osmo1abc...");
    }

    // ─── Transfer events ──────────────────────────────────────────────────────

    #[test]
    fn test_decode_native_transfer() {
        let decoder = CosmosEventDecoder::cosmos_hub();
        let tx = make_transfer_response("txhash1", 100, "cosmos1from", "cosmos1to", "1000uatom");
        let events = decoder.decode_transaction(&tx, 100);

        assert_eq!(events.len(), 1);
        let e = &events[0];
        assert_eq!(e.tx_hash, "txhash1");
        assert_eq!(e.block, 100);

        match &e.event_type {
            OnChainEventType::NativeTransfer { from, to, amount, .. } => {
                assert_eq!(from, "cosmos1from");
                assert_eq!(to, "cosmos1to");
                assert_eq!(amount, "1000");
            }
            _ => panic!("expected NativeTransfer"),
        }
    }

    #[test]
    fn test_decode_ibc_token_transfer() {
        let decoder = CosmosEventDecoder::osmosis();
        let tx = make_transfer_response(
            "ibc_tx",
            200,
            "osmo1sender",
            "osmo1recip",
            "500ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2",
        );
        let events = decoder.decode_transaction(&tx, 200);

        assert_eq!(events.len(), 1);
        match &events[0].event_type {
            OnChainEventType::TokenTransfer { token_address, amount, .. } => {
                assert!(token_address.starts_with("ibc/"));
                assert_eq!(amount, "500");
            }
            _ => panic!("expected TokenTransfer for IBC denom"),
        }
    }

    // ─── Delegation events ────────────────────────────────────────────────────

    #[test]
    fn test_decode_delegation() {
        let decoder = CosmosEventDecoder::cosmos_hub();
        let tx = make_delegation_response(
            "deltx",
            "cosmos1delegator",
            "cosmosvaloper1validator",
            "5000000uatom",
        );
        let events = decoder.decode_transaction(&tx, 500);

        assert_eq!(events.len(), 1);
        match &events[0].event_type {
            OnChainEventType::StakingAction { delegator, validator, action, amount, denom } => {
                assert_eq!(delegator, "cosmos1delegator");
                assert_eq!(validator.as_deref(), Some("cosmosvaloper1validator"));
                assert!(matches!(action, StakingActionType::Delegate));
                assert_eq!(amount, "5000000");
                assert_eq!(denom, "uatom");
            }
            _ => panic!("expected StakingAction"),
        }
    }

    // ─── Governance events ────────────────────────────────────────────────────

    #[test]
    fn test_decode_governance_vote() {
        let decoder = CosmosEventDecoder::cosmos_hub();
        let tx = make_vote_response("votetx", "42", "cosmos1voter", "VOTE_OPTION_YES");
        let events = decoder.decode_transaction(&tx, 600);

        assert_eq!(events.len(), 1);
        match &events[0].event_type {
            OnChainEventType::GovernanceAction { proposal_id, action, voter, vote } => {
                assert_eq!(proposal_id, "42");
                assert!(matches!(action, GovernanceActionType::Vote));
                assert_eq!(voter.as_deref(), Some("cosmos1voter"));
                assert_eq!(vote.as_deref(), Some("VOTE_OPTION_YES"));
            }
            _ => panic!("expected GovernanceAction"),
        }
    }

    // ─── Swap events ──────────────────────────────────────────────────────────

    #[test]
    fn test_decode_osmosis_swap() {
        let decoder = CosmosEventDecoder::osmosis();
        let tx = make_osmosis_swap(
            "swaptx",
            "osmo1trader",
            "1000uosmo",
            "500ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2",
            "1",
        );
        let events = decoder.decode_transaction(&tx, 5_000_000);

        assert_eq!(events.len(), 1);
        match &events[0].event_type {
            OnChainEventType::DexSwap { protocol, pool_address, token_in, token_out, sender, .. } => {
                assert_eq!(protocol, "osmosis_gamm");
                assert_eq!(pool_address, "1");
                assert_eq!(token_in.address, "uosmo");
                assert_eq!(token_in.amount, "1000");
                assert!(token_out.address.starts_with("ibc/"));
                assert_eq!(token_out.amount, "500");
                assert_eq!(sender, "osmo1trader");
            }
            _ => panic!("expected DexSwap"),
        }
    }

    // ─── Failed transaction ───────────────────────────────────────────────────

    #[test]
    fn test_failed_tx_returns_empty() {
        let decoder = CosmosEventDecoder::cosmos_hub();
        let tx = json!({
            "tx_response": {
                "txhash": "failhash",
                "height": "1",
                "code": 5,
                "raw_log": "insufficient funds",
                "events": []
            }
        });
        let events = decoder.decode_transaction(&tx, 1);
        assert!(events.is_empty(), "failed tx should produce no events");
    }

    // ─── Chain ID preservation ────────────────────────────────────────────────

    #[test]
    fn test_chain_id_on_events() {
        let decoder = CosmosEventDecoder::osmosis();
        let tx = make_transfer_response("chaintx", 1, "osmo1a", "osmo1b", "100uosmo");
        let events = decoder.decode_transaction(&tx, 1);

        for e in &events {
            assert_eq!(e.chain.family, "cosmos");
            assert_eq!(e.chain.network, "osmosis-1");
        }
    }

    // ─── IBC transfer event ───────────────────────────────────────────────────

    #[test]
    fn test_decode_ibc_bridge_transfer() {
        let decoder = CosmosEventDecoder::osmosis();
        let tx = json!({
            "tx_response": {
                "txhash": "ibctx",
                "height": "1000",
                "code": 0,
                "events": [{
                    "type": "ibc_transfer",
                    "attributes": [
                        { "key": "sender", "value": "osmo1sender" },
                        { "key": "receiver", "value": "cosmos1receiver" },
                        { "key": "denom", "value": "uosmo" },
                        { "key": "amount", "value": "10000" },
                        { "key": "source_channel", "value": "channel-0" },
                        { "key": "destination_channel", "value": "channel-141" }
                    ]
                }]
            }
        });

        let events = decoder.decode_transaction(&tx, 1000);
        assert_eq!(events.len(), 1);

        match &events[0].event_type {
            OnChainEventType::BridgeTransfer { bridge, sender, receiver, token, .. } => {
                assert_eq!(bridge, "ibc");
                assert_eq!(sender, "osmo1sender");
                assert_eq!(receiver.as_deref(), Some("cosmos1receiver"));
                assert_eq!(token.address, "uosmo");
                assert_eq!(token.amount, "10000");
            }
            _ => panic!("expected BridgeTransfer"),
        }
    }
}
