//! # GMX On-Chain Integration
//!
//! Alloy-powered on-chain helpers for GMX V2 position management on Arbitrum.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `onchain-evm` feature is enabled.
//!
//! ## Architecture
//!
//! GMX V2 uses the `ExchangeRouter` contract as the single entry point.
//! All trading operations go through `multicall()` — a bundle of:
//! 1. `sendWnt` (wrap ETH for execution fee) or `sendTokens` (transfer collateral)
//! 2. `createOrder` (market/limit increase/decrease position)
//!
//! This module provides:
//! - `GmxOnchain` — alloy provider wrapper with GMX V2 contract knowledge
//! - `create_position_onchain()` — builds an increase-position multicall transaction
//! - `close_position_onchain()` — builds a decrease-position multicall transaction
//!
//! ## Contract Addresses (Arbitrum Mainnet)
//!
//! - ExchangeRouter: `0x7C68C7866A64FA2160F78EEaE12217FFbf871fa8`
//! - OrderVault: `0x31eF83a530Fde1B38EE9A18093A333D8Bbbc40D5`
//! - WETH (Arbitrum): `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1`

#![cfg(feature = "onchain-evm")]

use std::sync::Arc;

use alloy::primitives::{Address, U256};
use alloy::rpc::types::eth::TransactionRequest;

use crate::core::chain::EvmProvider;
use crate::core::{ExchangeError, ExchangeResult};

// ═══════════════════════════════════════════════════════════════════════════════
// GMX V2 CONTRACT ADDRESSES (Arbitrum Mainnet)
// ═══════════════════════════════════════════════════════════════════════════════

/// GMX V2 ExchangeRouter on Arbitrum
pub const EXCHANGE_ROUTER_ARBITRUM: &str = "0x7C68C7866A64FA2160F78EEaE12217FFbf871fa8";

/// GMX V2 OrderVault on Arbitrum (receives collateral before createOrder)
pub const ORDER_VAULT_ARBITRUM: &str = "0x31eF83a530Fde1B38EE9A18093A333D8Bbbc40D5";

/// WETH on Arbitrum (used as execution-fee token in sendWnt)
pub const WETH_ARBITRUM: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";

/// GMX V2 ExchangeRouter on Avalanche
pub const EXCHANGE_ROUTER_AVALANCHE: &str = "0x11D62807dAE812a0F1571243460Bf94325F43BB7";

/// GMX V2 OrderVault on Avalanche
pub const ORDER_VAULT_AVALANCHE: &str = "0xD3D60D22d415aD43b7e64b510D86A30f19B1B12c";

/// WAVAX on Avalanche
pub const WAVAX_AVALANCHE: &str = "0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7";

// Function selectors
/// `multicall(bytes[])` selector
const MULTICALL_SELECTOR: [u8; 4] = [0xac, 0x9e, 0x01, 0x5f];

/// `sendWnt(address,uint256)` selector
const SEND_WNT_SELECTOR: [u8; 4] = [0x6d, 0xb7, 0x3b, 0x4a];

/// `sendTokens(address,address,uint256)` selector
const SEND_TOKENS_SELECTOR: [u8; 4] = [0x1a, 0x49, 0x29, 0x78];

/// `createOrder(CreateOrderParams)` selector
const CREATE_ORDER_SELECTOR: [u8; 4] = [0x7e, 0xe6, 0xb8, 0x1d];

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER PARAMETERS
// ═══════════════════════════════════════════════════════════════════════════════

/// GMX V2 order type discriminant.
///
/// Maps to `Order.OrderType` enum in GMX V2 contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GmxOrderType {
    /// Immediately-executable market increase (open long/short)
    MarketIncrease = 0,
    /// Limit increase — queued until price condition is met
    LimitIncrease = 1,
    /// Immediately-executable market decrease (reduce/close position)
    MarketDecrease = 3,
    /// Limit decrease — queued until price condition is met
    LimitDecrease = 4,
    /// Stop-loss decrease
    StopLossDecrease = 5,
    /// Liquidation (protocol-only)
    Liquidation = 6,
}

/// Direction of the position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GmxPositionSide {
    Long,
    Short,
}

/// Parameters for opening or increasing a GMX V2 position.
#[derive(Debug, Clone)]
pub struct CreatePositionParams {
    /// Market token address (e.g. ETH/USD GM pool token)
    pub market: Address,
    /// Collateral token address (e.g. WETH for longs, USDC for shorts)
    pub collateral_token: Address,
    /// Size delta in USD (30-decimal precision, e.g. 1000 USD = 1000 * 10^30)
    pub size_delta_usd: U256,
    /// Initial collateral delta amount (token units)
    pub initial_collateral_delta_amount: U256,
    /// Trigger price (30-decimal; 0 for market orders)
    pub trigger_price: U256,
    /// Acceptable price (slippage guard; use max for longs, min for shorts)
    pub acceptable_price: U256,
    /// Execution fee in ETH wei — paid to keeper
    pub execution_fee: U256,
    /// Long or short
    pub side: GmxPositionSide,
    /// Order type
    pub order_type: GmxOrderType,
    /// Wallet that receives output on decrease / close
    pub receiver: Address,
    /// UI referral code (32 bytes, zero = no referral)
    pub referral_code: [u8; 32],
}

/// Parameters for closing or decreasing a GMX V2 position.
#[derive(Debug, Clone)]
pub struct ClosePositionParams {
    /// Market token address
    pub market: Address,
    /// Collateral token address (must match existing position's collateral)
    pub collateral_token: Address,
    /// Size to decrease in USD (30-decimal; use full position size to fully close)
    pub size_delta_usd: U256,
    /// Collateral amount to withdraw (0 = keep remaining collateral in position)
    pub initial_collateral_delta_amount: U256,
    /// Trigger price (0 for market)
    pub trigger_price: U256,
    /// Acceptable price (slippage guard)
    pub acceptable_price: U256,
    /// Execution fee
    pub execution_fee: U256,
    /// Long or short
    pub side: GmxPositionSide,
    /// Order type (typically MarketDecrease or LimitDecrease)
    pub order_type: GmxOrderType,
    /// Receiver of released collateral
    pub receiver: Address,
    /// Referral code (zero bytes = none)
    pub referral_code: [u8; 32],
}

// ═══════════════════════════════════════════════════════════════════════════════
// GMX ON-CHAIN PROVIDER
// ═══════════════════════════════════════════════════════════════════════════════

/// On-chain provider wrapper for GMX V2 position management.
///
/// Wraps a shared [`EvmProvider`] and exposes typed helpers for building GMX V2
/// `multicall` transactions that open and close positions.
///
/// Multiple connectors targeting the same chain can share a single
/// `Arc<EvmProvider>` to reuse the same HTTP connection pool.
///
/// The caller is responsible for signing and broadcasting the resulting
/// `TransactionRequest` via their preferred alloy signer.
///
/// ## Usage
///
/// ```ignore
/// let onchain = GmxOnchain::arbitrum();
///
/// let tx = onchain.create_position_onchain(&params, from_address)?;
/// // Sign `tx`, then call provider.send_raw_transaction(...)
/// ```
pub struct GmxOnchain {
    /// Shared EVM chain provider
    provider: Arc<EvmProvider>,
    /// Chain name: "arbitrum" or "avalanche"
    chain: String,
}

impl GmxOnchain {
    /// Create a `GmxOnchain` from an existing shared [`EvmProvider`].
    ///
    /// `chain` — `"arbitrum"` or `"avalanche"` (used to select contract addresses).
    pub fn with_provider(provider: Arc<EvmProvider>, chain: &str) -> Self {
        Self {
            provider,
            chain: chain.to_lowercase(),
        }
    }

    /// Create provider for Arbitrum One using the public Offchain Labs RPC.
    pub fn arbitrum() -> Self {
        Self::with_provider(Arc::new(EvmProvider::arbitrum()), "arbitrum")
    }

    /// Create provider for Avalanche C-Chain using the public Ava Labs RPC.
    pub fn avalanche() -> Self {
        Self::with_provider(Arc::new(EvmProvider::avalanche()), "avalanche")
    }

    /// ExchangeRouter address for the current chain.
    pub fn exchange_router(&self) -> ExchangeResult<Address> {
        let addr = match self.chain.as_str() {
            "arbitrum" | "arb" => EXCHANGE_ROUTER_ARBITRUM,
            "avalanche" | "avax" => EXCHANGE_ROUTER_AVALANCHE,
            other => return Err(ExchangeError::InvalidRequest(
                format!("Unknown chain '{}'; supported: arbitrum, avalanche", other)
            )),
        };
        addr.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("Router address parse: {}", e)))
    }

    /// OrderVault address for the current chain.
    pub fn order_vault(&self) -> ExchangeResult<Address> {
        let addr = match self.chain.as_str() {
            "arbitrum" | "arb" => ORDER_VAULT_ARBITRUM,
            "avalanche" | "avax" => ORDER_VAULT_AVALANCHE,
            other => return Err(ExchangeError::InvalidRequest(
                format!("Unknown chain '{}'; supported: arbitrum, avalanche", other)
            )),
        };
        addr.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("Vault address parse: {}", e)))
    }

    /// Wrapped native token address for execution-fee deposits (WETH / WAVAX).
    pub fn wrapped_native_token(&self) -> ExchangeResult<Address> {
        let addr = match self.chain.as_str() {
            "arbitrum" | "arb" => WETH_ARBITRUM,
            "avalanche" | "avax" => WAVAX_AVALANCHE,
            other => return Err(ExchangeError::InvalidRequest(
                format!("Unknown chain '{}'; supported: arbitrum, avalanche", other)
            )),
        };
        addr.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("WNT address parse: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITION MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Build an unsigned GMX V2 increase-position (open/add) transaction.
    ///
    /// The transaction is a `ExchangeRouter.multicall([sendWnt, sendTokens, createOrder])`
    /// call.  The caller must:
    /// 1. Set a gas limit appropriate for Arbitrum (≈3,000,000 gas recommended).
    /// 2. Set gas price / EIP-1559 fee fields.
    /// 3. Set `value` equal to `params.execution_fee` (ETH sent with the call).
    /// 4. Sign and broadcast via `provider.send_raw_transaction(rlp_bytes).await`.
    pub fn create_position_onchain(
        &self,
        params: &CreatePositionParams,
        from: Address,
    ) -> ExchangeResult<TransactionRequest> {
        if params.size_delta_usd.is_zero() {
            return Err(ExchangeError::InvalidRequest(
                "size_delta_usd must be > 0".to_string(),
            ));
        }
        if params.initial_collateral_delta_amount.is_zero() {
            return Err(ExchangeError::InvalidRequest(
                "initial_collateral_delta_amount must be > 0".to_string(),
            ));
        }

        let router = self.exchange_router()?;
        let vault = self.order_vault()?;
        let wnt = self.wrapped_native_token()?;

        // Build each sub-call
        let send_wnt_call = encode_send_wnt(vault, params.execution_fee);
        let send_tokens_call = encode_send_tokens(
            params.collateral_token,
            vault,
            params.initial_collateral_delta_amount,
        );
        let create_order_call = encode_create_order(
            params.receiver,
            params.market,
            params.collateral_token,
            params.size_delta_usd,
            params.initial_collateral_delta_amount,
            params.trigger_price,
            params.acceptable_price,
            params.execution_fee,
            params.side == GmxPositionSide::Long,
            params.order_type,
            params.referral_code,
            wnt,
        );

        let calldata = encode_multicall(&[send_wnt_call, send_tokens_call, create_order_call]);

        let tx = TransactionRequest::default()
            .from(from)
            .to(router)
            .value(params.execution_fee)
            .input(alloy::primitives::Bytes::from(calldata).into());

        Ok(tx)
    }

    /// Build an unsigned GMX V2 decrease-position (reduce/close) transaction.
    ///
    /// For a full close, set `params.size_delta_usd` to the full position size
    /// and `params.initial_collateral_delta_amount` to the full collateral.
    ///
    /// The transaction structure is `multicall([sendWnt, createOrder])`.
    /// No `sendTokens` is needed for decrease orders.
    pub fn close_position_onchain(
        &self,
        params: &ClosePositionParams,
        from: Address,
    ) -> ExchangeResult<TransactionRequest> {
        if params.size_delta_usd.is_zero() {
            return Err(ExchangeError::InvalidRequest(
                "size_delta_usd must be > 0 for close/decrease".to_string(),
            ));
        }

        let router = self.exchange_router()?;
        let vault = self.order_vault()?;
        let wnt = self.wrapped_native_token()?;

        let send_wnt_call = encode_send_wnt(vault, params.execution_fee);
        let create_order_call = encode_create_order(
            params.receiver,
            params.market,
            params.collateral_token,
            params.size_delta_usd,
            params.initial_collateral_delta_amount,
            params.trigger_price,
            params.acceptable_price,
            params.execution_fee,
            params.side == GmxPositionSide::Long,
            params.order_type,
            params.referral_code,
            wnt,
        );

        let calldata = encode_multicall(&[send_wnt_call, create_order_call]);

        let tx = TransactionRequest::default()
            .from(from)
            .to(router)
            .value(params.execution_fee)
            .input(alloy::primitives::Bytes::from(calldata).into());

        Ok(tx)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CHAIN QUERIES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get the current block number on the connected chain.
    pub async fn get_block_number(&self) -> ExchangeResult<u64> {
        use crate::core::chain::ChainProvider;
        self.provider.get_height().await
    }

    /// Get the native balance (ETH / AVAX) of `wallet_address`.
    pub async fn get_native_balance(&self, wallet_address: &str) -> ExchangeResult<U256> {
        use crate::core::chain::ChainProvider;
        let balance_str = self.provider.get_native_balance(wallet_address).await?;
        balance_str
            .parse::<U256>()
            .map_err(|e| ExchangeError::Parse(format!("Balance parse error: {}", e)))
    }

    /// Access the underlying [`EvmProvider`] for advanced operations.
    pub fn provider(&self) -> &Arc<EvmProvider> {
        &self.provider
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ABI ENCODING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Encode `sendWnt(address receiver, uint256 amount)` calldata.
fn encode_send_wnt(receiver: Address, amount: U256) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + 64);
    data.extend_from_slice(&SEND_WNT_SELECTOR);
    data.extend_from_slice(&pad_address(receiver));
    data.extend_from_slice(&u256_to_be32(amount));
    data
}

/// Encode `sendTokens(address token, address receiver, uint256 amount)` calldata.
fn encode_send_tokens(token: Address, receiver: Address, amount: U256) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + 96);
    data.extend_from_slice(&SEND_TOKENS_SELECTOR);
    data.extend_from_slice(&pad_address(token));
    data.extend_from_slice(&pad_address(receiver));
    data.extend_from_slice(&u256_to_be32(amount));
    data
}

/// Encode `createOrder(CreateOrderParams params)` calldata.
///
/// The `CreateOrderParams` struct is ABI-encoded as a tuple.
/// This produces a minimal "no-callback, no-swap-path" create order.
/// The dynamic `swapPath[]` is encoded as an empty array.
#[allow(clippy::too_many_arguments)]
fn encode_create_order(
    receiver: Address,
    market: Address,
    collateral_token: Address,
    size_delta_usd: U256,
    initial_collateral_delta_amount: U256,
    trigger_price: U256,
    acceptable_price: U256,
    execution_fee: U256,
    is_long: bool,
    order_type: GmxOrderType,
    referral_code: [u8; 32],
    wnt_token: Address,
) -> Vec<u8> {
    // Encode CreateOrderParams using manual ABI encoding.
    // The struct has a dynamic sub-struct (Addresses, which contains swapPath[]),
    // so the outer params tuple is itself dynamic.
    //
    // Layout:
    //   selector (4)
    //   + offset to params (32) = 32 (single argument)
    //   + params encoding (dynamic tuple):
    //     head: [offset_addresses, numbers (7 words), orderType, decPosSwapType,
    //            isLong, shouldUnwrap, autoCancel, referralCode] = 8 * 32 = 256 bytes
    //     data: addresses tuple encoding

    let zero_u256 = U256::ZERO;
    let zero_addr = Address::ZERO;

    let mut buf: Vec<u8> = Vec::with_capacity(4 + 20 * 32);
    buf.extend_from_slice(&CREATE_ORDER_SELECTOR);

    // Outer: offset to params tuple = 32 (standard single-arg ABI encoding)
    buf.extend_from_slice(&u256_to_be32(U256::from(32u64)));

    // Params head (8 words):
    // [0] offset to addresses struct (dynamic)
    // [1..7] numbers (7 static words)
    // [8] orderType
    // [9] decreasePositionSwapType
    // [10] isLong
    // [11] shouldUnwrapNativeToken
    // [12] autoCancel
    // [13] referralCode
    let params_head_size: usize = 8 * 32; // 256 bytes
    // Offset to addresses from start of params = params_head_size
    buf.extend_from_slice(&u256_to_be32(U256::from(params_head_size as u64)));

    // Numbers (7 words, static)
    buf.extend_from_slice(&u256_to_be32(size_delta_usd));
    buf.extend_from_slice(&u256_to_be32(initial_collateral_delta_amount));
    buf.extend_from_slice(&u256_to_be32(trigger_price));
    buf.extend_from_slice(&u256_to_be32(acceptable_price));
    buf.extend_from_slice(&u256_to_be32(execution_fee));
    buf.extend_from_slice(&u256_to_be32(zero_u256)); // callbackGasLimit = 0
    buf.extend_from_slice(&u256_to_be32(zero_u256)); // minOutputAmount = 0

    // orderType (uint8 padded to 32)
    buf.extend_from_slice(&pad_u8(order_type as u8));
    // decreasePositionSwapType = 0 (NoSwap)
    buf.extend_from_slice(&pad_u8(0u8));
    // isLong (bool)
    buf.extend_from_slice(&pad_bool(is_long));
    // shouldUnwrapNativeToken = true
    buf.extend_from_slice(&pad_bool(true));
    // autoCancel = false
    buf.extend_from_slice(&pad_bool(false));
    // referralCode (bytes32)
    buf.extend_from_slice(&referral_code);

    // Addresses struct (dynamic because swapPath is address[]):
    // Head: 7 words (6 addresses + 1 offset for swapPath)
    // Tail: swapPath length (0)
    let addresses_head_size: usize = 7 * 32;

    buf.extend_from_slice(&pad_address(receiver));
    buf.extend_from_slice(&pad_address(zero_addr));   // cancellationReceiver = zero
    buf.extend_from_slice(&pad_address(zero_addr));   // callbackContract = zero
    buf.extend_from_slice(&pad_address(wnt_token));   // uiFeeReceiver = WNT
    buf.extend_from_slice(&pad_address(market));
    buf.extend_from_slice(&pad_address(collateral_token));
    // Offset to swapPath[] from start of Addresses encoding
    buf.extend_from_slice(&u256_to_be32(U256::from(addresses_head_size as u64)));

    // swapPath[] = [] (empty array, length = 0)
    buf.extend_from_slice(&u256_to_be32(U256::ZERO));

    buf
}

/// Encode `multicall(bytes[] data)` calldata.
///
/// ABI encodes an array of bytes blobs as the sole argument to multicall.
fn encode_multicall(calls: &[Vec<u8>]) -> Vec<u8> {
    let n = calls.len();
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&MULTICALL_SELECTOR);

    // Single arg `data` is a dynamic bytes[] → offset = 32
    buf.extend_from_slice(&u256_to_be32(U256::from(32u64)));

    // Array length
    buf.extend_from_slice(&u256_to_be32(U256::from(n as u64)));

    // Head: offsets for each bytes element (relative to start of array data = after length word)
    let head_size = n * 32;
    let mut current_offset: usize = head_size;
    for call in calls {
        buf.extend_from_slice(&u256_to_be32(U256::from(current_offset as u64)));
        let padded_len = (call.len() + 31) / 32 * 32;
        current_offset += 32 + padded_len;
    }

    // Tail: each bytes element = length word + padded data
    for call in calls {
        buf.extend_from_slice(&u256_to_be32(U256::from(call.len() as u64)));
        buf.extend_from_slice(call);
        // Pad to 32-byte boundary
        let rem = call.len() % 32;
        if rem != 0 {
            let padding = 32 - rem;
            buf.extend_from_slice(&vec![0u8; padding]);
        }
    }

    buf
}

// ═══════════════════════════════════════════════════════════════════════════════
// ABI PRIMITIVE HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn pad_address(addr: Address) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[12..].copy_from_slice(addr.as_slice());
    word
}

fn pad_u8(v: u8) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[31] = v;
    word
}

fn pad_bool(v: bool) -> [u8; 32] {
    pad_u8(if v { 1 } else { 0 })
}

fn u256_to_be32(v: U256) -> [u8; 32] {
    v.to_be_bytes()
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE BUILDERS
// ═══════════════════════════════════════════════════════════════════════════════

impl CreatePositionParams {
    /// Build a market-increase (open long) position on the ETH/USD market on Arbitrum.
    ///
    /// `size_delta_usd` — position size in 30-decimal USD (e.g. `1000 * 10^30` for $1000).
    /// `collateral_amount` — WETH amount to deposit as collateral (in wei).
    /// `acceptable_price` — max entry price in 30-decimal precision (slippage guard).
    /// `execution_fee` — ETH amount for keeper in wei (~0.001 ETH typical).
    /// `receiver` — wallet that holds the position and receives output on close.
    pub fn open_eth_long(
        size_delta_usd: U256,
        collateral_amount: U256,
        acceptable_price: U256,
        execution_fee: U256,
        receiver: Address,
    ) -> ExchangeResult<Self> {
        // ETH/USD GM market on Arbitrum
        let market: Address = "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336"
            .parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("ETH market parse: {}", e)))?;
        let weth: Address = WETH_ARBITRUM
            .parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("WETH parse: {}", e)))?;

        Ok(Self {
            market,
            collateral_token: weth,
            size_delta_usd,
            initial_collateral_delta_amount: collateral_amount,
            trigger_price: U256::ZERO,
            acceptable_price,
            execution_fee,
            side: GmxPositionSide::Long,
            order_type: GmxOrderType::MarketIncrease,
            receiver,
            referral_code: [0u8; 32],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_send_wnt_length() {
        let vault: Address = ORDER_VAULT_ARBITRUM.parse().unwrap();
        let amount = U256::from(1_000_000_000_000_000u64); // 0.001 ETH
        let calldata = encode_send_wnt(vault, amount);
        // 4 (selector) + 32 (receiver) + 32 (amount) = 68
        assert_eq!(calldata.len(), 4 + 32 + 32);
        assert_eq!(&calldata[..4], &SEND_WNT_SELECTOR);
    }

    #[test]
    fn test_encode_send_tokens_length() {
        let token: Address = WETH_ARBITRUM.parse().unwrap();
        let vault: Address = ORDER_VAULT_ARBITRUM.parse().unwrap();
        let amount = U256::from(5_000_000_000_000_000_000u64); // 5 WETH
        let calldata = encode_send_tokens(token, vault, amount);
        // 4 + 32 + 32 + 32 = 100
        assert_eq!(calldata.len(), 4 + 32 + 32 + 32);
        assert_eq!(&calldata[..4], &SEND_TOKENS_SELECTOR);
    }

    #[test]
    fn test_encode_multicall_structure() {
        let call1 = vec![0xde, 0xad, 0xbe, 0xef];
        let call2 = vec![0xca, 0xfe];
        let calldata = encode_multicall(&[call1, call2]);
        // selector (4) + offset to array (32) + length (32) + 2 offsets (64) + ...
        assert!(calldata.len() >= 4 + 32 + 32 + 64);
        assert_eq!(&calldata[..4], &MULTICALL_SELECTOR);
    }

    #[test]
    fn test_order_type_discriminant() {
        assert_eq!(GmxOrderType::MarketIncrease as u8, 0);
        assert_eq!(GmxOrderType::MarketDecrease as u8, 3);
        assert_eq!(GmxOrderType::LimitIncrease as u8, 1);
    }

    #[test]
    fn test_arbitrum_constructor_chain_name() {
        let onchain = GmxOnchain::arbitrum();
        assert_eq!(onchain.chain, "arbitrum");
    }

    #[test]
    fn test_avalanche_constructor_chain_name() {
        let onchain = GmxOnchain::avalanche();
        assert_eq!(onchain.chain, "avalanche");
    }
}
