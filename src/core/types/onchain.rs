//! # On-Chain Event Types
//!
//! Unified event type system for recording what happened on any blockchain.
//!
//! ## Design Principles
//!
//! - **Event registration, not interpretation**: records WHAT happened, not what it MEANS
//! - **Chain-agnostic where possible**: common fields in `OnChainEvent`, chain-specific
//!   details go into `OnChainEventType` variants or `raw`
//! - **No signals**: no whale alerts, no smart-money labels, no trading signals
//! - **Amounts as strings**: avoids precision loss across chains with different decimal systems
//! - **`serde` on everything**: required for persistence and streaming

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// CHAIN IDENTIFICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Identifies a specific blockchain network.
///
/// Used in every [`OnChainEvent`] to indicate which chain the event occurred on.
///
/// # Examples
///
/// ```
/// use digdigdig3::core::types::onchain::ChainId;
///
/// let eth = ChainId::evm("ethereum", 1);
/// let arb = ChainId::evm("arbitrum", 42161);
/// let sol = ChainId::new("solana", "mainnet-beta");
/// let osmo = ChainId::cosmos("osmosis-1");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ChainId {
    /// Chain family: `"evm"`, `"bitcoin"`, `"solana"`, `"cosmos"`, `"starknet"`,
    /// `"sui"`, `"ton"`, `"aptos"`.
    pub family: String,
    /// Human-readable network name: `"ethereum"`, `"arbitrum"`, `"mainnet-beta"`,
    /// `"osmosis-1"`, `"mainnet"`, etc.
    pub network: String,
    /// EVM numeric chain ID when applicable (`Some(1)` for Ethereum mainnet).
    /// `None` for non-EVM chains.
    pub chain_id: Option<u64>,
}

impl ChainId {
    /// Construct a generic [`ChainId`] without an EVM chain ID.
    pub fn new(family: impl Into<String>, network: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            network: network.into(),
            chain_id: None,
        }
    }

    /// Construct an EVM [`ChainId`] with a numeric chain ID.
    pub fn evm(network: impl Into<String>, chain_id: u64) -> Self {
        Self {
            family: "evm".to_string(),
            network: network.into(),
            chain_id: Some(chain_id),
        }
    }

    /// Construct a Cosmos [`ChainId`].
    ///
    /// `chain_id` is the bech32 chain identifier (e.g. `"osmosis-1"`).
    pub fn cosmos(chain_id: impl Into<String>) -> Self {
        let network = chain_id.into();
        Self {
            family: "cosmos".to_string(),
            network: network.clone(),
            chain_id: None,
        }
    }

    /// Returns a compact display string: `"evm:ethereum:1"` or `"solana:mainnet-beta"`.
    pub fn display(&self) -> String {
        match self.chain_id {
            Some(id) => format!("{}:{}:{}", self.family, self.network, id),
            None => format!("{}:{}", self.family, self.network),
        }
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIFIED ON-CHAIN EVENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified record of a single on-chain event.
///
/// Represents one discrete occurrence on a blockchain — a token transfer,
/// a DEX swap, a governance vote, etc. Every event carries location metadata
/// (which chain, which block, which transaction) and a typed payload in
/// [`OnChainEventType`].
///
/// # Amount convention
///
/// All amount fields are strings in the **chain's smallest unit** (e.g. Wei for
/// EVM, lamports for Solana, satoshis for Bitcoin). Use `decimals` fields to
/// convert to human-readable form.
///
/// # Raw field
///
/// `raw` carries the unmodified JSON from the data source for any fields that
/// do not fit into the typed payload. It is optional and may be `None` when
/// the typed payload captures everything of interest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainEvent {
    /// Which chain this event occurred on.
    pub chain: ChainId,

    /// Block number (EVM), slot (Solana), height (Bitcoin/Cosmos), or sequence number.
    pub block: u64,

    /// Transaction hash in chain-native format:
    /// - EVM: `"0x..."` hex
    /// - Solana: base58 signature
    /// - Bitcoin: txid hex (little-endian)
    /// - Cosmos: uppercase hex
    pub tx_hash: String,

    /// Position of this event within the transaction.
    ///
    /// For EVM this is the log index. For Solana this is the instruction index.
    /// `None` for chains where ordering within a transaction is not applicable
    /// (e.g. Bitcoin UTXO events).
    pub log_index: Option<u32>,

    /// Unix timestamp in seconds when the block was produced.
    pub timestamp: u64,

    /// Typed event payload — classification of what happened.
    pub event_type: OnChainEventType,

    /// Raw source data for fields not captured in `event_type`.
    ///
    /// May include full log topics, raw instruction data, or provider-specific
    /// metadata. `None` when the typed payload is sufficient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT CLASSIFICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Classification of an on-chain event — factual record of what occurred.
///
/// Each variant captures the minimum fields needed to identify the event
/// unambiguously. Interpretation (signals, labels, risk scores) is the
/// responsibility of downstream consumers, not this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OnChainEventType {
    // ─── Token Movement ────────────────────────────────────────────────────────

    /// ERC-20 / SPL token transfer, IBC coin transfer, etc.
    TokenTransfer {
        /// Contract address (EVM), mint address (Solana), or IBC denom.
        token_address: String,
        /// Human-readable ticker symbol if known at event time.
        token_symbol: Option<String>,
        /// Sender address in chain-native format.
        from: String,
        /// Recipient address in chain-native format.
        to: String,
        /// Transfer amount in the token's smallest unit.
        amount: String,
        /// Token decimal places (if known).
        decimals: Option<u8>,
        /// USD equivalent at the time of the event (if provided by data source).
        usd_value: Option<f64>,
    },

    /// Native coin transfer (ETH, SOL, BNB, ATOM, etc.).
    NativeTransfer {
        /// Sender address.
        from: String,
        /// Recipient address.
        to: String,
        /// Amount in the chain's smallest unit (Wei, lamports, uATOM, satoshis).
        amount: String,
        /// USD equivalent at the time of the event (if provided by data source).
        usd_value: Option<f64>,
    },

    // ─── DEX / Swap Activity ───────────────────────────────────────────────────

    /// Token swap on a decentralized exchange.
    DexSwap {
        /// Protocol identifier: `"uniswap_v3"`, `"raydium"`, `"osmosis_gamm"`,
        /// `"curve"`, `"balancer"`, etc.
        protocol: String,
        /// Pool/pair contract address or pool ID.
        pool_address: String,
        /// Token sold / input to the swap.
        token_in: TokenAmount,
        /// Token received / output of the swap.
        token_out: TokenAmount,
        /// Address that initiated the swap.
        sender: String,
        /// USD volume of the swap (if provided by data source).
        usd_volume: Option<f64>,
    },

    /// Liquidity added to or removed from a pool.
    LiquidityChange {
        /// Protocol identifier.
        protocol: String,
        /// Pool/pair contract address or pool ID.
        pool_address: String,
        /// Whether liquidity was added or removed.
        action: LiquidityAction,
        /// First token in the pair.
        token_a: TokenAmount,
        /// Second token in the pair.
        token_b: TokenAmount,
        /// Raw liquidity units minted or burned (protocol-specific).
        #[serde(skip_serializing_if = "Option::is_none")]
        liquidity_delta: Option<String>,
        /// Address that performed the operation.
        sender: String,
    },

    /// New trading pool created on a DEX.
    PoolCreated {
        /// Protocol identifier.
        protocol: String,
        /// Address of the newly created pool.
        pool_address: String,
        /// First token in the pair.
        token_a: TokenInfo,
        /// Second token in the pair.
        token_b: TokenInfo,
        /// Fee tier in basis points (e.g. `3000` = 0.30%).
        #[serde(skip_serializing_if = "Option::is_none")]
        fee_tier: Option<u32>,
        /// Address that created the pool.
        creator: String,
    },

    // ─── Lending / Borrowing ───────────────────────────────────────────────────

    /// Interaction with a lending protocol (Aave, Compound, Morpho, etc.).
    LendingAction {
        /// Protocol identifier: `"aave_v3"`, `"compound_v3"`, `"morpho"`, etc.
        protocol: String,
        /// Operation type.
        action: LendingActionType,
        /// Token involved in the operation.
        token: TokenAmount,
        /// Account that performed the operation.
        account: String,
        /// Collateral token for liquidation events.
        #[serde(skip_serializing_if = "Option::is_none")]
        collateral_token: Option<TokenAmount>,
    },

    // ─── Staking / Delegation ─────────────────────────────────────────────────

    /// Staking or delegation action (Cosmos, Ethereum validators, etc.).
    StakingAction {
        /// Validator address or pubkey (if applicable).
        #[serde(skip_serializing_if = "Option::is_none")]
        validator: Option<String>,
        /// Delegator or staker address.
        delegator: String,
        /// Operation type.
        action: StakingActionType,
        /// Amount staked / unstaked / claimed in the smallest unit.
        amount: String,
        /// Token denomination or symbol (`"ETH"`, `"ATOM"`, `"SOL"`, etc.).
        denom: String,
    },

    // ─── Bridge / Cross-chain ─────────────────────────────────────────────────

    /// Assets moved across chains via a bridge or IBC.
    BridgeTransfer {
        /// Bridge protocol: `"wormhole"`, `"layerzero"`, `"ibc"`, `"stargate"`,
        /// `"across"`, `"hop"`, etc.
        bridge: String,
        /// Source chain identifier (chain name or ChainId display string).
        source_chain: String,
        /// Destination chain identifier.
        dest_chain: String,
        /// Token and amount being bridged.
        token: TokenAmount,
        /// Sending address on the source chain.
        sender: String,
        /// Receiving address on the destination chain (if known at event time).
        #[serde(skip_serializing_if = "Option::is_none")]
        receiver: Option<String>,
    },

    // ─── NFT ──────────────────────────────────────────────────────────────────

    /// NFT transfer (transfer-only or sale).
    NftTransfer {
        /// Collection contract address or ID.
        collection: String,
        /// Token ID within the collection (string to handle large IDs).
        token_id: String,
        /// Sender address (`"0x000...0"` for mints).
        from: String,
        /// Recipient address.
        to: String,
        /// Sale price if this transfer is a marketplace sale.
        #[serde(skip_serializing_if = "Option::is_none")]
        price: Option<TokenAmount>,
    },

    // ─── Contract Lifecycle ───────────────────────────────────────────────────

    /// Smart contract or program deployed.
    ContractDeployed {
        /// Address of the newly deployed contract/program.
        address: String,
        /// Address that deployed the contract.
        deployer: String,
        /// Code hash or bytecode hash for the deployed contract.
        #[serde(skip_serializing_if = "Option::is_none")]
        code_hash: Option<String>,
    },

    /// Generic interaction with an existing contract.
    ///
    /// Used for contract calls that do not match any more-specific variant.
    ContractInteraction {
        /// Target contract address.
        contract: String,
        /// Function selector (EVM: `"0xa9059cbb"`) or method name if known.
        method: String,
        /// Address that sent the call.
        caller: String,
        /// Native value attached to the call (in smallest unit), if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },

    // ─── Governance ───────────────────────────────────────────────────────────

    /// On-chain governance action (DAO votes, Cosmos governance, etc.).
    GovernanceAction {
        /// Proposal or referendum ID.
        proposal_id: String,
        /// Action type.
        action: GovernanceActionType,
        /// Voter address (present for Vote actions).
        #[serde(skip_serializing_if = "Option::is_none")]
        voter: Option<String>,
        /// Vote choice: `"yes"`, `"no"`, `"abstain"`, `"no_with_veto"`.
        #[serde(skip_serializing_if = "Option::is_none")]
        vote: Option<String>,
    },

    // ─── Bitcoin-specific ─────────────────────────────────────────────────────

    /// Bitcoin UTXO consumed in a transaction.
    UtxoSpent {
        /// Previous output's transaction ID.
        prev_tx: String,
        /// Output index within `prev_tx`.
        prev_index: u32,
        /// Address that spent the UTXO (from scriptSig / witness).
        spender: String,
        /// Value of the spent UTXO in satoshis.
        amount: String,
    },

    /// Coinbase (block reward + fees) transaction.
    CoinbaseReward {
        /// Miner address (from coinbase output).
        miner: String,
        /// Total reward in satoshis (subsidy + fees).
        reward: String,
        /// Mining pool name if identifiable from coinbase script.
        #[serde(skip_serializing_if = "Option::is_none")]
        pool: Option<String>,
    },

    // ─── Mempool ──────────────────────────────────────────────────────────────

    /// Pending transaction observed in the mempool before inclusion in a block.
    ///
    /// `block` in the parent [`OnChainEvent`] will be `0` for mempool events.
    MempoolTransaction {
        /// Sender address.
        from: String,
        /// Recipient address (`None` for contract deployments).
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        /// Native value in the transaction (smallest unit).
        value: String,
        /// Gas price or fee rate (chain-specific units, string).
        #[serde(skip_serializing_if = "Option::is_none")]
        gas_price: Option<String>,
        /// Calldata or transaction payload size in bytes.
        data_size: u32,
    },

    // ─── Catch-all ────────────────────────────────────────────────────────────

    /// Chain-specific event that does not map to any typed variant above.
    ///
    /// Use this for newly encountered event shapes until a proper variant is added.
    Custom {
        /// Broad category of the event: `"perp_trade"`, `"options_exercise"`, etc.
        category: String,
        /// Specific action within the category.
        action: String,
        /// Arbitrary key-value details.
        details: HashMap<String, serde_json::Value>,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUPPORTING TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// A token with an associated amount.
///
/// Used in DEX swaps, lending actions, bridge transfers, and NFT sales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmount {
    /// Contract address (EVM), mint address (Solana), or IBC denom.
    pub address: String,
    /// Human-readable ticker symbol if known.
    pub symbol: Option<String>,
    /// Amount in the token's smallest unit.
    pub amount: String,
    /// Token decimal places (if known).
    pub decimals: Option<u8>,
}

impl TokenAmount {
    /// Construct a [`TokenAmount`] with only the required fields.
    pub fn new(address: impl Into<String>, amount: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            amount: amount.into(),
            symbol: None,
            decimals: None,
        }
    }

    /// Add a symbol to this [`TokenAmount`].
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    /// Add decimal places to this [`TokenAmount`].
    pub fn with_decimals(mut self, decimals: u8) -> Self {
        self.decimals = Some(decimals);
        self
    }
}

/// Static metadata for a token — used when no amount is needed (e.g. pool creation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Contract address (EVM), mint address (Solana), or IBC denom.
    pub address: String,
    /// Human-readable ticker symbol if known.
    pub symbol: Option<String>,
    /// Token decimal places (if known).
    pub decimals: Option<u8>,
}

impl TokenInfo {
    /// Construct a [`TokenInfo`] with only the required address.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            symbol: None,
            decimals: None,
        }
    }
}

// ─── Action enums ─────────────────────────────────────────────────────────────

/// Whether liquidity was added to or removed from a pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityAction {
    /// Liquidity deposited into the pool.
    Add,
    /// Liquidity withdrawn from the pool.
    Remove,
}

/// Type of interaction with a lending protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LendingActionType {
    /// Assets supplied as collateral or for yield.
    Supply,
    /// Supplied assets withdrawn.
    Withdraw,
    /// Assets borrowed against collateral.
    Borrow,
    /// Borrowed assets repaid.
    Repay,
    /// Under-collateralised position liquidated.
    Liquidate,
}

/// Type of staking or delegation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StakingActionType {
    /// Tokens delegated to a validator.
    Delegate,
    /// Undelegation initiated (tokens enter unbonding period).
    Undelegate,
    /// Tokens redelegated from one validator to another.
    Redelegate,
    /// Staking rewards claimed.
    ClaimRewards,
    /// Validator slash event (tokens removed as penalty).
    Slash,
}

/// Type of governance action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceActionType {
    /// New governance proposal submitted.
    Propose,
    /// Vote cast on an existing proposal.
    Vote,
    /// Passed proposal executed on-chain.
    Execute,
    /// Proposal cancelled by submitter or governance.
    Cancel,
}
