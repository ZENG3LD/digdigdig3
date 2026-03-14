//! # EVM Event Decoder
//!
//! Decodes raw EVM log entries (as `serde_json::Value`) into typed
//! [`OnChainEvent`] structs without any external ABI decoder crate.
//!
//! ## ABI encoding rules (quick reference)
//!
//! - **Topics** array: `topic[0]` = event signature keccak256 hash; subsequent
//!   topics are indexed parameters, each zero-padded to 32 bytes.
//! - **Data** field: non-indexed parameters concatenated in declaration order,
//!   each occupying a 32-byte slot, hex-encoded with `"0x"` prefix.
//! - **Addresses** in topics: the 20 meaningful bytes occupy the *last* 20 bytes
//!   of a 32-byte slot (i.e. left-padded with 12 zero bytes).
//! - **uint256** values: big-endian, left-padded to 32 bytes — stored as decimal
//!   strings to avoid precision loss.
//!
//! ## Feature gate
//!
//! Everything in this file is compiled only when the `onchain-evm` feature is
//! enabled.

use crate::core::types::onchain::{
    ChainId, LiquidityAction, OnChainEvent, OnChainEventType, TokenAmount, TokenInfo,
};

// ═══════════════════════════════════════════════════════════════════════════════
// WELL-KNOWN TOPIC0 CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Well-known EVM event topic0 (keccak256 of ABI signature).
pub mod topics {
    /// `Transfer(address indexed from, address indexed to, uint256 value)`
    /// — ERC-20 token transfer.
    pub const ERC20_TRANSFER: &str =
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    /// `Approval(address indexed owner, address indexed spender, uint256 value)`
    /// — ERC-20 approval.
    pub const ERC20_APPROVAL: &str =
        "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";

    /// Uniswap V2 `Swap(address indexed sender, uint256 amount0In, uint256 amount1In,
    /// uint256 amount0Out, uint256 amount1Out, address indexed to)`
    pub const UNISWAP_V2_SWAP: &str =
        "0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822";

    /// Uniswap V3 `Swap(address indexed sender, address indexed recipient,
    /// int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)`
    pub const UNISWAP_V3_SWAP: &str =
        "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67";

    /// Uniswap V2 Factory `PairCreated(address indexed token0, address indexed token1,
    /// address pair, uint256 pairIndex)`
    pub const PAIR_CREATED: &str =
        "0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9";

    /// Uniswap V3 Factory `PoolCreated(address indexed token0, address indexed token1,
    /// uint24 indexed fee, int24 tickSpacing, address pool)`
    pub const POOL_CREATED: &str =
        "0x783cca1c0412dd0d695e784568c96da2e9c22ff989357a2e8b1d9b2b4e6b7118";

    /// Uniswap V2 `Mint(address indexed sender, uint256 amount0, uint256 amount1)`
    pub const V2_MINT: &str =
        "0x4c209b5fc8ad50758f13e2e1088ba56a560dff690a1c6fef26394f4c03821c4f";

    /// Uniswap V2 `Burn(address indexed sender, uint256 amount0, uint256 amount1,
    /// address indexed to)`
    pub const V2_BURN: &str =
        "0xdccd412f0b1252819cb1fd330b93224ca42612892bb3f4f789976e6d81936496";

    /// WETH `Deposit(address indexed dst, uint256 wad)`
    pub const WETH_DEPOSIT: &str =
        "0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c";

    /// WETH `Withdrawal(address indexed src, uint256 wad)`
    pub const WETH_WITHDRAWAL: &str =
        "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65";
}

// ═══════════════════════════════════════════════════════════════════════════════
// DECODER
// ═══════════════════════════════════════════════════════════════════════════════

/// Decodes raw EVM JSON log entries into typed [`OnChainEvent`] values.
///
/// Construct one decoder per chain using [`EvmEventDecoder::new`] or the
/// convenience constructors [`EvmEventDecoder::ethereum`] /
/// [`EvmEventDecoder::arbitrum`], then call [`decode_log`] or
/// [`decode_block_logs`] on each log.
///
/// Unknown event signatures fall back to [`OnChainEventType::ContractInteraction`]
/// rather than returning `None`, so callers always get *something* for every log
/// that has at least a contract address and a transaction hash.
///
/// # No panics
///
/// All decode helpers return `Option` internally; any field that cannot be
/// parsed is silently omitted or substituted with an empty / sentinel value.
/// The top-level public methods never panic.
pub struct EvmEventDecoder {
    chain: ChainId,
}

impl EvmEventDecoder {
    /// Construct a decoder for any EVM chain.
    pub fn new(chain: ChainId) -> Self {
        Self { chain }
    }

    /// Decoder pre-configured for Ethereum mainnet (chain ID 1).
    pub fn ethereum() -> Self {
        Self::new(ChainId::evm("ethereum", 1))
    }

    /// Decoder pre-configured for Arbitrum One (chain ID 42161).
    pub fn arbitrum() -> Self {
        Self::new(ChainId::evm("arbitrum", 42161))
    }

    /// Returns the [`ChainId`] this decoder is configured for.
    pub fn chain_id(&self) -> &ChainId {
        &self.chain
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Public API
    // ─────────────────────────────────────────────────────────────────────────

    /// Decode a single JSON log entry into an [`OnChainEvent`].
    ///
    /// `log` is the JSON object as returned by `eth_getLogs` / `eth_getTransactionReceipt`:
    ///
    /// ```json
    /// {
    ///   "address": "0x...",
    ///   "topics": ["0xtopic0", "0xtopic1", ...],
    ///   "data": "0x...",
    ///   "blockNumber": "0x...",
    ///   "transactionHash": "0x...",
    ///   "logIndex": "0x..."
    /// }
    /// ```
    ///
    /// Returns `None` only when the log is structurally invalid (missing
    /// `transactionHash`, no `topics` array, or an unparseable `blockNumber`).
    /// All other cases produce at least a [`OnChainEventType::ContractInteraction`].
    pub fn decode_log(
        &self,
        log: &serde_json::Value,
        block_timestamp: u64,
    ) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let topic0 = topics.first()?.as_str()?;

        match topic0 {
            t if t == topics::ERC20_TRANSFER => self.decode_erc20_transfer(log, block_timestamp),
            t if t == topics::UNISWAP_V2_SWAP => self.decode_v2_swap(log, block_timestamp),
            t if t == topics::UNISWAP_V3_SWAP => self.decode_v3_swap(log, block_timestamp),
            t if t == topics::PAIR_CREATED => self.decode_pool_created_v2(log, block_timestamp),
            t if t == topics::POOL_CREATED => self.decode_pool_created_v3(log, block_timestamp),
            t if t == topics::V2_MINT => self.decode_liquidity_add(log, block_timestamp),
            t if t == topics::V2_BURN => self.decode_liquidity_remove(log, block_timestamp),
            t if t == topics::WETH_DEPOSIT => self.decode_weth_deposit(log, block_timestamp),
            t if t == topics::WETH_WITHDRAWAL => self.decode_weth_withdrawal(log, block_timestamp),
            t if t == topics::ERC20_APPROVAL => {
                // Approval events decoded as generic contract interaction — not
                // a transfer, swap, or liquidity event.
                self.decode_generic_contract_interaction(log, block_timestamp)
            }
            _ => self.decode_generic_contract_interaction(log, block_timestamp),
        }
    }

    /// Decode every log in `logs` and return the successfully decoded events.
    ///
    /// Logs that return `None` from [`decode_log`] (structurally broken entries)
    /// are silently skipped. The returned `Vec` may therefore be shorter than
    /// `logs`.
    pub fn decode_block_logs(
        &self,
        logs: &[serde_json::Value],
        block_timestamp: u64,
    ) -> Vec<OnChainEvent> {
        logs.iter()
            .filter_map(|log| self.decode_log(log, block_timestamp))
            .collect()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal decoders — one per recognised event type
    // ─────────────────────────────────────────────────────────────────────────

    /// `Transfer(address indexed from, address indexed to, uint256 value)`
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = from (address, indexed)
    /// - topics[2] = to   (address, indexed)
    /// - data[0..32] = value (uint256, non-indexed)
    fn decode_erc20_transfer(
        &self,
        log: &serde_json::Value,
        ts: u64,
    ) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let from = decode_address(topics.get(1)?.as_str()?)?;
        let to = decode_address(topics.get(2)?.as_str()?)?;
        let amount = decode_uint256(log["data"].as_str().unwrap_or("0x"), 0)
            .unwrap_or_else(|| "0".to_string());

        let token_address = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::TokenTransfer {
                token_address,
                token_symbol: None,
                from,
                to,
                amount,
                decimals: None,
                usd_value: None,
            },
        ))
    }

    /// Uniswap V2 `Swap(address indexed sender, uint256 amount0In, uint256 amount1In,
    /// uint256 amount0Out, uint256 amount1Out, address indexed to)`
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = sender (indexed)
    /// - topics[2] = to (indexed)
    /// - data slots: [0]=amount0In, [1]=amount1In, [2]=amount0Out, [3]=amount1Out
    fn decode_v2_swap(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let sender = decode_address(topics.get(1)?.as_str()?)?;

        let data = log["data"].as_str().unwrap_or("0x");
        let amount0_in = decode_uint256(data, 0).unwrap_or_else(|| "0".to_string());
        let amount1_in = decode_uint256(data, 1).unwrap_or_else(|| "0".to_string());
        let amount0_out = decode_uint256(data, 2).unwrap_or_else(|| "0".to_string());
        let amount1_out = decode_uint256(data, 3).unwrap_or_else(|| "0".to_string());

        let pool_address = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        // Determine direction: if amount0In > 0, token0 is being sold for token1.
        let (token_in_amount, token_out_amount) = if amount0_in != "0" {
            (amount0_in, amount1_out)
        } else {
            (amount1_in, amount0_out)
        };

        // Pool address is used as a placeholder token address since the V2 Swap
        // event does not emit the token addresses themselves. Downstream consumers
        // that need the actual token addresses should look them up via the pair contract.
        let token_in = TokenAmount::new(pool_address.clone(), token_in_amount);
        let token_out = TokenAmount::new(pool_address.clone(), token_out_amount);

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::DexSwap {
                protocol: "uniswap_v2".to_string(),
                pool_address,
                token_in,
                token_out,
                sender,
                usd_volume: None,
            },
        ))
    }

    /// Uniswap V3 `Swap(address indexed sender, address indexed recipient,
    /// int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)`
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = sender    (indexed)
    /// - topics[2] = recipient (indexed)
    /// - data slots: [0]=amount0 (int256), [1]=amount1 (int256),
    ///               [2]=sqrtPriceX96, [3]=liquidity, [4]=tick
    fn decode_v3_swap(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let sender = decode_address(topics.get(1)?.as_str()?)?;

        let data = log["data"].as_str().unwrap_or("0x");
        // amount0 and amount1 are int256 (signed). We decode raw 32-byte hex.
        // Negative means tokens flowing *out* of the pool to the trader.
        let amount0_raw = decode_raw_slot(data, 0).unwrap_or_else(|| "0x".to_string() + &"0".repeat(64));
        let amount1_raw = decode_raw_slot(data, 1).unwrap_or_else(|| "0x".to_string() + &"0".repeat(64));

        let pool_address = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        // Determine which amount is positive (in) and which is negative (out).
        // A positive amount0 means the trader sent token0 to the pool (token0 = in).
        let (token_in_amount, token_out_amount) = if is_positive_int256(&amount0_raw) {
            (
                hex_to_decimal_string(&amount0_raw),
                // amount1 is negative — negate it for display as a positive out-amount
                hex_negate_int256_to_string(&amount1_raw),
            )
        } else {
            (
                hex_to_decimal_string(&amount1_raw),
                hex_negate_int256_to_string(&amount0_raw),
            )
        };

        let token_in = TokenAmount::new(pool_address.clone(), token_in_amount);
        let token_out = TokenAmount::new(pool_address.clone(), token_out_amount);

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::DexSwap {
                protocol: "uniswap_v3".to_string(),
                pool_address,
                token_in,
                token_out,
                sender,
                usd_volume: None,
            },
        ))
    }

    /// Uniswap V2 Factory `PairCreated(address indexed token0, address indexed token1,
    /// address pair, uint256)`
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = token0 (indexed)
    /// - topics[2] = token1 (indexed)
    /// - data[0..32] = pair address (non-indexed, abi-encoded address = left-padded 32 bytes)
    fn decode_pool_created_v2(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let token_a_addr = decode_address(topics.get(1)?.as_str()?)?;
        let token_b_addr = decode_address(topics.get(2)?.as_str()?)?;

        let data = log["data"].as_str().unwrap_or("0x");
        // pair address is the first 32-byte slot of data (right-aligned address)
        let pool_address = decode_address_from_data(data, 0)
            .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

        let creator = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::PoolCreated {
                protocol: "uniswap_v2".to_string(),
                pool_address,
                token_a: TokenInfo::new(token_a_addr),
                token_b: TokenInfo::new(token_b_addr),
                fee_tier: None,
                creator,
            },
        ))
    }

    /// Uniswap V3 Factory `PoolCreated(address indexed token0, address indexed token1,
    /// uint24 indexed fee, int24 tickSpacing, address pool)`
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = token0 (indexed)
    /// - topics[2] = token1 (indexed)
    /// - topics[3] = fee    (indexed, uint24)
    /// - data[0..32] = tickSpacing (int24, padded)
    /// - data[32..64] = pool address (address, padded)
    fn decode_pool_created_v3(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let token_a_addr = decode_address(topics.get(1)?.as_str()?)?;
        let token_b_addr = decode_address(topics.get(2)?.as_str()?)?;

        // fee is uint24 stored in a 32-byte topic — parse the last 3 bytes
        let fee_tier = topics
            .get(3)
            .and_then(|t| t.as_str())
            .and_then(|s| {
                let hex = s.trim_start_matches("0x");
                u32::from_str_radix(hex, 16).ok()
            });

        let data = log["data"].as_str().unwrap_or("0x");
        // pool address is the second slot (offset 1)
        let pool_address = decode_address_from_data(data, 1)
            .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

        let creator = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::PoolCreated {
                protocol: "uniswap_v3".to_string(),
                pool_address,
                token_a: TokenInfo::new(token_a_addr),
                token_b: TokenInfo::new(token_b_addr),
                fee_tier,
                creator,
            },
        ))
    }

    /// Uniswap V2 `Mint(address indexed sender, uint256 amount0, uint256 amount1)`
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = sender (indexed)
    /// - data[0..32] = amount0, data[32..64] = amount1
    fn decode_liquidity_add(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let sender = decode_address(topics.get(1)?.as_str()?)?;

        let data = log["data"].as_str().unwrap_or("0x");
        let amount0 = decode_uint256(data, 0).unwrap_or_else(|| "0".to_string());
        let amount1 = decode_uint256(data, 1).unwrap_or_else(|| "0".to_string());

        let pool_address = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        let token_a = TokenAmount::new(pool_address.clone(), amount0);
        let token_b = TokenAmount::new(pool_address.clone(), amount1);

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::LiquidityChange {
                protocol: "uniswap_v2".to_string(),
                pool_address,
                action: LiquidityAction::Add,
                token_a,
                token_b,
                liquidity_delta: None,
                sender,
            },
        ))
    }

    /// Uniswap V2 `Burn(address indexed sender, uint256 amount0, uint256 amount1,
    /// address indexed to)`
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = sender (indexed)
    /// - topics[2] = to     (indexed)
    /// - data[0..32] = amount0, data[32..64] = amount1
    fn decode_liquidity_remove(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let sender = decode_address(topics.get(1)?.as_str()?)?;

        let data = log["data"].as_str().unwrap_or("0x");
        let amount0 = decode_uint256(data, 0).unwrap_or_else(|| "0".to_string());
        let amount1 = decode_uint256(data, 1).unwrap_or_else(|| "0".to_string());

        let pool_address = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        let token_a = TokenAmount::new(pool_address.clone(), amount0);
        let token_b = TokenAmount::new(pool_address.clone(), amount1);

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::LiquidityChange {
                protocol: "uniswap_v2".to_string(),
                pool_address,
                action: LiquidityAction::Remove,
                token_a,
                token_b,
                liquidity_delta: None,
                sender,
            },
        ))
    }

    /// WETH `Deposit(address indexed dst, uint256 wad)` — treated as a token
    /// transfer from the zero address to `dst`.
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = dst (indexed)
    /// - data[0..32] = wad (uint256)
    fn decode_weth_deposit(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let to = decode_address(topics.get(1)?.as_str()?)?;

        let data = log["data"].as_str().unwrap_or("0x");
        let amount = decode_uint256(data, 0).unwrap_or_else(|| "0".to_string());

        let token_address = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::TokenTransfer {
                token_address,
                token_symbol: Some("WETH".to_string()),
                from: "0x0000000000000000000000000000000000000000".to_string(),
                to,
                amount,
                decimals: Some(18),
                usd_value: None,
            },
        ))
    }

    /// WETH `Withdrawal(address indexed src, uint256 wad)` — treated as a token
    /// transfer from `src` to the zero address.
    ///
    /// Layout:
    /// - topics[0] = signature
    /// - topics[1] = src (indexed)
    /// - data[0..32] = wad (uint256)
    fn decode_weth_withdrawal(&self, log: &serde_json::Value, ts: u64) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let from = decode_address(topics.get(1)?.as_str()?)?;

        let data = log["data"].as_str().unwrap_or("0x");
        let amount = decode_uint256(data, 0).unwrap_or_else(|| "0".to_string());

        let token_address = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::TokenTransfer {
                token_address,
                token_symbol: Some("WETH".to_string()),
                from,
                to: "0x0000000000000000000000000000000000000000".to_string(),
                amount,
                decimals: Some(18),
                usd_value: None,
            },
        ))
    }

    /// Fallback: any log whose topic0 is not recognised becomes a
    /// [`OnChainEventType::ContractInteraction`].
    ///
    /// The topic0 itself is used as the `method` selector so callers can still
    /// distinguish different unknown event types.
    fn decode_generic_contract_interaction(
        &self,
        log: &serde_json::Value,
        ts: u64,
    ) -> Option<OnChainEvent> {
        let topics = log["topics"].as_array()?;
        let method = topics
            .first()
            .and_then(|t| t.as_str())
            // Shorten to 10 chars (0x + 4 bytes selector) for display parity with
            // function selectors, but keep the full hash so callers can still match it.
            .unwrap_or("0x")
            .to_string();

        let contract = log["address"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000")
            .to_lowercase();

        // The tx sender is not present in the log itself; use a placeholder.
        // Callers who need it should fetch the full transaction.
        let caller = "unknown".to_string();

        Some(self.build_event(
            log,
            ts,
            OnChainEventType::ContractInteraction {
                contract,
                method,
                caller,
                value: None,
            },
        ))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Shared builder
    // ─────────────────────────────────────────────────────────────────────────

    /// Construct the outer [`OnChainEvent`] shell from common log fields.
    ///
    /// `blockNumber` and `transactionHash` are required; `logIndex` is optional.
    /// If `blockNumber` is absent the event block is set to `0`.
    fn build_event(
        &self,
        log: &serde_json::Value,
        timestamp: u64,
        event_type: OnChainEventType,
    ) -> OnChainEvent {
        let block = parse_hex_or_decimal_u64(log["blockNumber"].as_str().unwrap_or("0x0"))
            .unwrap_or(0);

        let tx_hash = log["transactionHash"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000")
            .to_string();

        let log_index = log["logIndex"]
            .as_str()
            .and_then(|s| parse_hex_or_decimal_u64(s))
            .map(|v| v as u32);

        OnChainEvent {
            chain: self.chain.clone(),
            block,
            tx_hash,
            log_index,
            timestamp,
            event_type,
            raw: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HEX PARSING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Decode the `offset`-th 32-byte slot of `data` as a uint256 and return it as
/// a decimal string.
///
/// `data` must be a hex string with an optional `"0x"` prefix.
/// `offset` is zero-based (0 = first slot, 1 = second slot, …).
///
/// Returns `None` if `data` is too short or contains non-hex characters.
///
/// # Examples
///
/// ```ignore
/// // data = "0x" + 64 hex chars (one slot)
/// let amount = decode_uint256("0x0000000000000000000000000000000000000000000000000de0b6b3a7640000", 0);
/// assert_eq!(amount, Some("1000000000000000000".to_string())); // 1 ETH in wei
/// ```
pub fn decode_uint256(data: &str, offset: usize) -> Option<String> {
    let hex = data.trim_start_matches("0x");
    // Each slot is 64 hex characters (32 bytes)
    let start = offset * 64;
    let end = start + 64;
    if hex.len() < end {
        return None;
    }
    let slot = &hex[start..end];
    // Parse as a big integer represented in hex
    hex_to_decimal(slot)
}

/// Decode the `offset`-th 32-byte data slot as a checksummed EVM address.
///
/// ABI-encoded addresses occupy 32 bytes but only the last 20 bytes are
/// meaningful; the first 12 bytes must be zeros.
pub fn decode_address_from_data(data: &str, offset: usize) -> Option<String> {
    let hex = data.trim_start_matches("0x");
    let start = offset * 64;
    let end = start + 64;
    if hex.len() < end {
        return None;
    }
    let slot = &hex[start..end];
    // Address is right-aligned: last 40 hex chars = 20 bytes
    let addr_hex = &slot[24..]; // skip leading 24 hex chars (12 bytes of padding)
    Some(format!("0x{}", addr_hex.to_lowercase()))
}

/// Decode a 32-byte indexed topic as an EVM address.
///
/// Indexed address parameters are zero-padded on the left to 32 bytes:
/// `"0x000000000000000000000000<20-byte-address>"`.
///
/// Returns the lowercase `"0x"-prefixed 20-byte address, or `None` if the
/// topic string is malformed.
pub fn decode_address(topic: &str) -> Option<String> {
    let hex = topic.trim_start_matches("0x");
    if hex.len() < 40 {
        return None;
    }
    // Take the last 40 hex characters (20 bytes)
    let addr = &hex[hex.len() - 40..];
    Some(format!("0x{}", addr.to_lowercase()))
}

/// Return the raw 64-char hex string for the `offset`-th 32-byte data slot.
///
/// Returned value does NOT include a `"0x"` prefix.
pub(crate) fn decode_raw_slot(data: &str, offset: usize) -> Option<String> {
    let hex = data.trim_start_matches("0x");
    let start = offset * 64;
    let end = start + 64;
    if hex.len() < end {
        return None;
    }
    Some(hex[start..end].to_string())
}

/// Parse a hex string (with or without `"0x"` prefix) OR a decimal string as `u64`.
pub(crate) fn parse_hex_or_decimal_u64(s: &str) -> Option<u64> {
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Big-integer helpers (no external crate — manual 256-bit arithmetic)
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a 64-character (256-bit) hex string to a decimal string.
///
/// Performs grade-school multiplication: for each hex nibble, multiply the
/// running result by 16 and add the nibble value.  Uses a `Vec<u8>` of decimal
/// digits in little-endian order for O(n²) big-integer arithmetic — fast enough
/// for a 256-bit number (at most 78 decimal digits).
fn hex_to_decimal(hex: &str) -> Option<String> {
    // Validate input
    if hex.len() > 64 || hex.is_empty() {
        return None;
    }

    // Digits in big-endian decimal; we build them in reverse (little-endian)
    // then flip at the end.
    let mut digits: Vec<u8> = vec![0u8]; // represents 0

    for ch in hex.chars() {
        let nibble = ch.to_digit(16)? as u8;

        // digits = digits * 16
        let mut carry: u32 = 0;
        for d in digits.iter_mut() {
            let val = (*d as u32) * 16 + carry;
            *d = (val % 10) as u8;
            carry = val / 10;
        }
        while carry > 0 {
            digits.push((carry % 10) as u8);
            carry /= 10;
        }

        // digits = digits + nibble
        let mut carry: u32 = nibble as u32;
        for d in digits.iter_mut() {
            let val = (*d as u32) + carry;
            *d = (val % 10) as u8;
            carry = val / 10;
        }
        while carry > 0 {
            digits.push((carry % 10) as u8);
            carry /= 10;
        }
    }

    // Reverse to get big-endian, then convert digits to chars
    let s: String = digits
        .iter()
        .rev()
        .map(|d| (b'0' + d) as char)
        .collect();

    Some(s)
}

/// Return `true` if the 64-char raw 256-bit hex slot represents a non-negative
/// int256 (i.e. the most-significant bit is 0).
pub(crate) fn is_positive_int256(raw: &str) -> bool {
    // MSB is in the first hex character; bit 7 of first nibble (0–7 = positive)
    raw.chars()
        .next()
        .and_then(|c| c.to_digit(16))
        .map(|n| n < 8)
        .unwrap_or(true)
}

/// Interpret a 64-char raw 256-bit hex slot as a decimal string.
///
/// For positive int256 this is the same as uint256.
/// For negative int256 this still returns the two's-complement unsigned
/// representation (callers that need the signed value should negate it).
pub(crate) fn hex_to_decimal_string(raw: &str) -> String {
    hex_to_decimal(raw).unwrap_or_else(|| "0".to_string())
}

/// Two's-complement negate a 256-bit int256 and return its absolute value as a
/// decimal string.
///
/// Used to convert a negative token amount (e.g. "tokens leaving the pool")
/// from its two's-complement representation to a positive decimal string.
///
/// Formula: `~raw + 1` (bitwise NOT plus one).
pub(crate) fn hex_negate_int256_to_string(raw: &str) -> String {
    if raw.len() != 64 {
        return "0".to_string();
    }

    // Step 1: bitwise NOT every nibble
    let inverted: String = raw
        .chars()
        .map(|c| {
            let n = c.to_digit(16).unwrap_or(0);
            std::char::from_digit(n ^ 0xF, 16).unwrap_or('0')
        })
        .collect();

    // Step 2: add 1 to the inverted big-endian hex number
    let result = hex_add_one(&inverted);

    // Step 3: convert to decimal
    hex_to_decimal(&result).unwrap_or_else(|| "0".to_string())
}

/// Add 1 to a 64-character big-endian hex string (no overflow handling — if all
/// nibbles are `f` the result wraps to all-zeros, which is correct for int256 MIN).
fn hex_add_one(hex: &str) -> String {
    let mut bytes: Vec<u8> = hex
        .chars()
        .map(|c| c.to_digit(16).unwrap_or(0) as u8)
        .collect();

    let mut carry = 1u8;
    for nibble in bytes.iter_mut().rev() {
        let val = *nibble + carry;
        *nibble = val & 0xF;
        carry = val >> 4;
        if carry == 0 {
            break;
        }
    }

    bytes.iter().map(|&n| std::char::from_digit(n as u32, 16).unwrap_or('0')).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ─── hex_to_decimal ───────────────────────────────────────────────────────

    #[test]
    fn test_hex_to_decimal_zero() {
        assert_eq!(hex_to_decimal("0000000000000000000000000000000000000000000000000000000000000000"), Some("0".to_string()));
    }

    #[test]
    fn test_hex_to_decimal_one() {
        assert_eq!(hex_to_decimal("0000000000000000000000000000000000000000000000000000000000000001"), Some("1".to_string()));
    }

    #[test]
    fn test_hex_to_decimal_one_eth_in_wei() {
        // 1e18 = 0x0de0b6b3a7640000
        let padded = "0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        assert_eq!(hex_to_decimal(padded), Some("1000000000000000000".to_string()));
    }

    #[test]
    fn test_hex_to_decimal_usdc_1000() {
        // 1000 USDC (6 decimals) = 1_000_000_000
        let padded = "000000000000000000000000000000000000000000000000000000003b9aca00";
        assert_eq!(hex_to_decimal(padded), Some("1000000000".to_string()));
    }

    // ─── decode_uint256 ───────────────────────────────────────────────────────

    #[test]
    fn test_decode_uint256_first_slot() {
        let data = "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        assert_eq!(decode_uint256(data, 0), Some("1000000000000000000".to_string()));
    }

    #[test]
    fn test_decode_uint256_second_slot() {
        let data = "0x\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        assert_eq!(decode_uint256(data, 0), Some("1".to_string()));
        assert_eq!(decode_uint256(data, 1), Some("1000000000000000000".to_string()));
    }

    #[test]
    fn test_decode_uint256_out_of_bounds() {
        let data = "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        assert_eq!(decode_uint256(data, 1), None); // only 1 slot, asking for slot 1
    }

    // ─── decode_address ───────────────────────────────────────────────────────

    #[test]
    fn test_decode_address_from_topic() {
        let topic = "0x000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        assert_eq!(
            decode_address(topic),
            Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string())
        );
    }

    #[test]
    fn test_decode_address_zero() {
        let topic = "0x0000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(
            decode_address(topic),
            Some("0x0000000000000000000000000000000000000000".to_string())
        );
    }

    // ─── decode_address_from_data ─────────────────────────────────────────────

    #[test]
    fn test_decode_address_from_data_slot() {
        // Address at slot 0, left-padded with 12 zero bytes
        let data = "0x000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        assert_eq!(
            decode_address_from_data(data, 0),
            Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string())
        );
    }

    // ─── is_positive_int256 ───────────────────────────────────────────────────

    #[test]
    fn test_is_positive_int256_positive() {
        // 0x0000...0001 — positive
        let raw = "0000000000000000000000000000000000000000000000000000000000000001";
        assert!(is_positive_int256(raw));
    }

    #[test]
    fn test_is_positive_int256_negative() {
        // 0xffff...ffff (-1 in two's complement) — negative
        let raw = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert!(!is_positive_int256(raw));
    }

    // ─── hex_negate_int256 ────────────────────────────────────────────────────

    #[test]
    fn test_negate_minus_one() {
        // -1 in int256 two's complement
        let raw = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert_eq!(hex_negate_int256_to_string(raw), "1");
    }

    #[test]
    fn test_negate_minus_1e18() {
        // -1e18 in int256 two's complement
        // 1e18 = 0x0de0b6b3a7640000
        // -1e18 two's complement = 0xfffffffffffffffffffff21f494c589c0000
        // Full 256-bit: fffffffffffffffffffffffffffffffffffffffffffffffff21f494c589c0000
        let raw = "fffffffffffffffffffffffffffffffffffffffffffffffff21f494c589c0000";
        assert_eq!(hex_negate_int256_to_string(raw), "1000000000000000000");
    }

    // ─── ERC-20 Transfer decoder ──────────────────────────────────────────────

    #[test]
    fn test_decode_erc20_transfer() {
        let decoder = EvmEventDecoder::ethereum();
        let log = json!({
            "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            "topics": [
                topics::ERC20_TRANSFER,
                "0x000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045",
                "0x000000000000000000000000742d35cc6634c0532925a3b8d4c9c42d9b1e7c5a"
            ],
            "data": "0x0000000000000000000000000000000000000000000000000000000077359400",
            "blockNumber": "0x11e1234",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
            "logIndex": "0x3"
        });

        let event = decoder.decode_log(&log, 1700000000).unwrap();
        assert_eq!(event.block, 0x11e1234);
        assert_eq!(event.timestamp, 1700000000);
        assert_eq!(event.log_index, Some(3));

        match event.event_type {
            OnChainEventType::TokenTransfer { from, to, amount, token_address, .. } => {
                assert_eq!(from, "0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
                assert_eq!(to, "0x742d35cc6634c0532925a3b8d4c9c42d9b1e7c5a");
                // 0x77359400 = 2000000000 (2000 USDC with 6 decimals)
                assert_eq!(amount, "2000000000");
                assert_eq!(token_address, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
            }
            other => panic!("unexpected event type: {:?}", other),
        }
    }

    // ─── Uniswap V2 Swap decoder ──────────────────────────────────────────────

    #[test]
    fn test_decode_v2_swap() {
        let decoder = EvmEventDecoder::ethereum();
        // Swap where token0 is sold (amount0In > 0, amount1Out > 0)
        let data = concat!(
            "0x",
            "0000000000000000000000000000000000000000000000000de0b6b3a7640000", // amount0In = 1e18
            "0000000000000000000000000000000000000000000000000000000000000000", // amount1In = 0
            "0000000000000000000000000000000000000000000000000000000077359400", // amount0Out = 0
            "0000000000000000000000000000000000000000000000000000000000000000"  // amount1Out = 0 (swap direction: amount0In -> uses amount1Out from slot2)
        );
        // Reorder: amount0Out should be slot 2 = 0, amount1Out slot 3 = 0
        // Let's fix: token0In -> token1Out
        let data = concat!(
            "0x",
            "0000000000000000000000000000000000000000000000000de0b6b3a7640000", // amount0In = 1e18
            "0000000000000000000000000000000000000000000000000000000000000000", // amount1In = 0
            "0000000000000000000000000000000000000000000000000000000000000000", // amount0Out = 0
            "0000000000000000000000000000000000000000000000000000000077359400"  // amount1Out = 2000000000
        );

        let log = json!({
            "address": "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc",
            "topics": [
                topics::UNISWAP_V2_SWAP,
                "0x000000000000000000000000ef1c6e67703c7bd7107eed8303fbe6ec2554bf6b",
                "0x000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045"
            ],
            "data": data,
            "blockNumber": "0x11e1234",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
            "logIndex": "0x1"
        });

        let event = decoder.decode_log(&log, 1700000000).unwrap();
        match event.event_type {
            OnChainEventType::DexSwap { protocol, token_in, token_out, sender, .. } => {
                assert_eq!(protocol, "uniswap_v2");
                assert_eq!(token_in.amount, "1000000000000000000");
                assert_eq!(token_out.amount, "2000000000");
                assert_eq!(sender, "0xef1c6e67703c7bd7107eed8303fbe6ec2554bf6b");
            }
            other => panic!("unexpected event type: {:?}", other),
        }
    }

    // ─── Generic fallback ─────────────────────────────────────────────────────

    #[test]
    fn test_decode_unknown_event_falls_back_to_contract_interaction() {
        let decoder = EvmEventDecoder::ethereum();
        let log = json!({
            "address": "0x1234567890123456789012345678901234567890",
            "topics": [
                "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
            ],
            "data": "0x",
            "blockNumber": "0x1",
            "transactionHash": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "logIndex": "0x0"
        });

        let event = decoder.decode_log(&log, 0).unwrap();
        assert!(matches!(event.event_type, OnChainEventType::ContractInteraction { .. }));
    }

    // ─── decode_block_logs ────────────────────────────────────────────────────

    #[test]
    fn test_decode_block_logs_skips_invalid() {
        let decoder = EvmEventDecoder::ethereum();
        // Log missing topics → decode_log returns None → skipped
        let invalid = json!({ "address": "0x1234" });
        // Valid log
        let valid = json!({
            "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            "topics": [
                topics::ERC20_TRANSFER,
                "0x000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045",
                "0x000000000000000000000000742d35cc6634c0532925a3b8d4c9c42d9b1e7c5a"
            ],
            "data": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "blockNumber": "0x1",
            "transactionHash": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "logIndex": "0x0"
        });

        let events = decoder.decode_block_logs(&[invalid, valid], 0);
        assert_eq!(events.len(), 1);
    }

    // ─── WETH deposit / withdrawal ────────────────────────────────────────────

    #[test]
    fn test_decode_weth_deposit() {
        let decoder = EvmEventDecoder::ethereum();
        let log = json!({
            "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            "topics": [
                topics::WETH_DEPOSIT,
                "0x000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045"
            ],
            "data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
            "blockNumber": "0x1",
            "transactionHash": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "logIndex": "0x0"
        });

        let event = decoder.decode_log(&log, 0).unwrap();
        match event.event_type {
            OnChainEventType::TokenTransfer { from, to, amount, token_symbol, decimals, .. } => {
                assert_eq!(from, "0x0000000000000000000000000000000000000000");
                assert_eq!(to, "0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
                assert_eq!(amount, "1000000000000000000");
                assert_eq!(token_symbol, Some("WETH".to_string()));
                assert_eq!(decimals, Some(18));
            }
            other => panic!("unexpected: {:?}", other),
        }
    }

    // ─── Pool created V3 fee tier ─────────────────────────────────────────────

    #[test]
    fn test_decode_pool_created_v3_fee_tier() {
        let decoder = EvmEventDecoder::ethereum();
        // fee = 3000 (0x0BB8)
        let log = json!({
            "address": "0x1f98431c8ad98523631ae4a59f267346ea31f984",
            "topics": [
                topics::POOL_CREATED,
                "0x000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
                "0x000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", // WETH
                "0x0000000000000000000000000000000000000000000000000000000000000bb8"  // fee=3000
            ],
            // data: tickSpacing (slot 0) + pool address (slot 1)
            "data": "0x\
                0000000000000000000000000000000000000000000000000000000000000003\
                0000000000000000000000008ad599c3a0ff1de082011efddc58f1908eb6e6d8",
            "blockNumber": "0x1",
            "transactionHash": "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "logIndex": "0x0"
        });

        let event = decoder.decode_log(&log, 0).unwrap();
        match event.event_type {
            OnChainEventType::PoolCreated { protocol, fee_tier, token_a, token_b, pool_address, .. } => {
                assert_eq!(protocol, "uniswap_v3");
                assert_eq!(fee_tier, Some(3000));
                assert_eq!(token_a.address, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
                assert_eq!(token_b.address, "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
                assert_eq!(pool_address, "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8");
            }
            other => panic!("unexpected: {:?}", other),
        }
    }
}
