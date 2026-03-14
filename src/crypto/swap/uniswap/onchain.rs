//! # Uniswap On-Chain Integration
//!
//! Alloy-powered on-chain helpers for Uniswap V3 swap execution.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `onchain-ethereum` feature is enabled.
//!
//! ## Architecture
//!
//! - `UniswapOnchain` — wraps an alloy `DynProvider` and exposes high-level
//!   swap building primitives
//! - `build_swap_tx()` — builds an unsigned Uniswap V3 `exactInputSingle` transaction
//! - `get_token_balance_onchain()` — queries ERC-20 balance via alloy eth_call
//! - `get_eth_balance()` — queries native ETH balance

#![cfg(feature = "onchain-ethereum")]

use alloy::network::Ethereum;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::rpc::types::eth::TransactionRequest;

use crate::core::{ExchangeError, ExchangeResult};

// ═══════════════════════════════════════════════════════════════════════════════
// UNISWAP V3 CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Uniswap V3 SwapRouter02 address (mainnet)
pub const SWAP_ROUTER_MAINNET: &str = "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45";

/// Uniswap V3 SwapRouter02 address (Sepolia testnet)
pub const SWAP_ROUTER_TESTNET: &str = "0x3bFA4769FB09eefC5a80d6E87c3B9C650f7Ae48e";

/// ERC-20 `balanceOf(address)` function selector
const BALANCE_OF_SELECTOR: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];

/// Uniswap V3 `exactInputSingle` function selector
/// keccak256("exactInputSingle((address,address,uint24,address,uint256,uint256,uint160))")
const EXACT_INPUT_SINGLE_SELECTOR: [u8; 4] = [0x41, 0x4b, 0xf3, 0x89];

// ═══════════════════════════════════════════════════════════════════════════════
// SWAP PARAMETERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parameters for a Uniswap V3 `exactInputSingle` swap.
///
/// Maps to the Solidity struct:
/// ```solidity
/// struct ExactInputSingleParams {
///     address tokenIn;
///     address tokenOut;
///     uint24 fee;
///     address recipient;
///     uint256 amountIn;
///     uint256 amountOutMinimum;
///     uint160 sqrtPriceLimitX96;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ExactInputSingleParams {
    /// Input token contract address
    pub token_in: Address,
    /// Output token contract address
    pub token_out: Address,
    /// Pool fee tier in hundredths of a basis point (e.g. 500 = 0.05%, 3000 = 0.30%)
    pub fee: u32,
    /// Address that receives the output tokens
    pub recipient: Address,
    /// Exact amount of `token_in` to swap (in wei / smallest unit)
    pub amount_in: U256,
    /// Minimum acceptable output amount (slippage guard)
    pub amount_out_minimum: U256,
    /// Price limit (0 = no limit; use only for advanced routing)
    pub sqrt_price_limit_x96: U256,
}

/// Result of a submitted swap transaction
#[derive(Debug, Clone)]
pub struct SwapTxResult {
    /// Transaction hash (0x-prefixed hex)
    pub tx_hash: String,
    /// Input amount used
    pub amount_in: U256,
    /// Minimum output amount specified
    pub amount_out_minimum: U256,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNISWAP ON-CHAIN PROVIDER
// ═══════════════════════════════════════════════════════════════════════════════

/// On-chain provider wrapper for Uniswap V3 interactions.
///
/// Wraps an alloy type-erased `DynProvider<Ethereum>` and exposes typed helpers for:
/// - Building `exactInputSingle` calldata
/// - Querying ERC-20 token balances
/// - Sending signed swap transactions
///
/// The caller is responsible for signing and broadcasting the resulting
/// `TransactionRequest` via their preferred alloy signer.
///
/// ## Usage
///
/// ```ignore
/// let onchain = UniswapOnchain::new("https://ethereum-rpc.publicnode.com", false)?;
///
/// let balance = onchain.get_token_balance_onchain(
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
///     "0xYourWalletAddress",
/// ).await?;
/// ```
pub struct UniswapOnchain {
    /// Alloy HTTP provider connected to an Ethereum node
    provider: DynProvider<Ethereum>,
    /// Whether we are on testnet (affects router address)
    testnet: bool,
}

impl UniswapOnchain {
    /// Connect to an Ethereum JSON-RPC endpoint and build the provider.
    ///
    /// `rpc_url` — HTTP(S) URL of any Ethereum-compatible RPC node.
    /// `testnet` — if `true`, uses Sepolia router address.
    pub fn new(rpc_url: &str, testnet: bool) -> ExchangeResult<Self> {
        let url: reqwest::Url = rpc_url.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("Invalid RPC URL '{}': {}", rpc_url, e)))?;

        let provider = ProviderBuilder::new().connect_http(url);

        Ok(Self {
            provider: DynProvider::new(provider),
            testnet,
        })
    }

    /// Address of the Uniswap V3 SwapRouter02 for the current network.
    pub fn router_address(&self) -> ExchangeResult<Address> {
        let addr_str = if self.testnet {
            SWAP_ROUTER_TESTNET
        } else {
            SWAP_ROUTER_MAINNET
        };
        addr_str.parse::<Address>()
            .map_err(|e| ExchangeError::InvalidRequest(format!("Invalid router address: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CALLDATA BUILDER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Encode `exactInputSingle` calldata for the SwapRouter02.
    ///
    /// Packs the ABI-encoded `ExactInputSingleParams` struct after the 4-byte
    /// function selector. All values are ABI-encoded as 32-byte big-endian words.
    pub fn encode_exact_input_single(params: &ExactInputSingleParams) -> Bytes {
        let mut calldata = Vec::with_capacity(4 + 7 * 32);
        calldata.extend_from_slice(&EXACT_INPUT_SINGLE_SELECTOR);

        // Each field ABI-encoded as 32-byte word (big-endian, left-zero-padded)
        calldata.extend_from_slice(&pad_address(params.token_in));
        calldata.extend_from_slice(&pad_address(params.token_out));
        calldata.extend_from_slice(&pad_u32(params.fee));
        calldata.extend_from_slice(&pad_address(params.recipient));
        calldata.extend_from_slice(&u256_to_be32(params.amount_in));
        calldata.extend_from_slice(&u256_to_be32(params.amount_out_minimum));
        calldata.extend_from_slice(&u256_to_be32(params.sqrt_price_limit_x96));

        Bytes::from(calldata)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SWAP EXECUTION
    // ═══════════════════════════════════════════════════════════════════════════

    /// Build an unsigned `exactInputSingle` swap transaction.
    ///
    /// Returns a `TransactionRequest` ready to be signed and sent.
    /// The caller must:
    /// 1. Set nonce via `provider.get_transaction_count(from).await`.
    /// 2. Set gas price / EIP-1559 max fee fields.
    /// 3. Sign via `alloy::signers::local::PrivateKeySigner`.
    /// 4. Broadcast via `provider.send_raw_transaction(&rlp_bytes).await`.
    pub fn build_swap_tx(
        &self,
        params: &ExactInputSingleParams,
        from: Address,
    ) -> ExchangeResult<TransactionRequest> {
        if params.amount_in.is_zero() {
            return Err(ExchangeError::InvalidRequest(
                "Swap amount_in must be greater than zero".to_string(),
            ));
        }

        let router = self.router_address()?;
        let calldata = Self::encode_exact_input_single(params);

        let tx = TransactionRequest::default()
            .from(from)
            .to(router)
            .input(calldata.into());

        Ok(tx)
    }

    /// Send a pre-built and signed raw transaction to the network.
    ///
    /// `raw_tx` — RLP-encoded signed transaction bytes.
    ///
    /// Returns the transaction hash as a 0x-prefixed hex string.
    pub async fn send_raw_transaction(&self, raw_tx: &[u8]) -> ExchangeResult<String> {
        let pending = self.provider
            .send_raw_transaction(raw_tx)
            .await
            .map_err(|e| ExchangeError::Network(format!("send_raw_transaction failed: {}", e)))?;

        Ok(format!("{:#x}", pending.tx_hash()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TOKEN BALANCE QUERY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Query the ERC-20 token balance of `wallet_address` via alloy eth_call.
    ///
    /// Returns the raw balance as `U256` (in the token's smallest unit).
    /// Divide by `10^decimals` to get human-readable amount.
    ///
    /// `token_address` — ERC-20 contract address (0x-prefixed hex).
    /// `wallet_address` — wallet address to query (0x-prefixed hex).
    pub async fn get_token_balance_onchain(
        &self,
        token_address: &str,
        wallet_address: &str,
    ) -> ExchangeResult<U256> {
        let token: Address = token_address.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("Invalid token address '{}': {}", token_address, e)))?;
        let wallet: Address = wallet_address.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("Invalid wallet address '{}': {}", wallet_address, e)))?;

        // Encode: balanceOf(address) = selector ++ padded wallet address
        let mut calldata = Vec::with_capacity(4 + 32);
        calldata.extend_from_slice(&BALANCE_OF_SELECTOR);
        calldata.extend_from_slice(&pad_address(wallet));

        let tx = TransactionRequest::default()
            .to(token)
            .input(Bytes::from(calldata).into());

        let result = self.provider
            .call(tx)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_call (balanceOf) failed: {}", e)))?;

        // Result is 32-byte big-endian uint256
        if result.len() < 32 {
            return Err(ExchangeError::Parse(format!(
                "balanceOf returned {} bytes, expected 32",
                result.len()
            )));
        }
        let balance = U256::from_be_slice(&result[..32]);
        Ok(balance)
    }

    /// Query the native ETH balance of `wallet_address`.
    ///
    /// Returns the balance in wei as `U256`.
    pub async fn get_eth_balance(&self, wallet_address: &str) -> ExchangeResult<U256> {
        let wallet: Address = wallet_address.parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("Invalid wallet address: {}", e)))?;

        let balance = self.provider
            .get_balance(wallet)
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_getBalance failed: {}", e)))?;

        Ok(balance)
    }

    /// Get the current block number.
    pub async fn get_block_number(&self) -> ExchangeResult<u64> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| ExchangeError::Network(format!("eth_blockNumber failed: {}", e)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ABI ENCODING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Pad an `Address` to a 32-byte ABI word (left-zero-padded).
fn pad_address(addr: Address) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[12..].copy_from_slice(addr.as_slice());
    word
}

/// Pad a `u32` to a 32-byte ABI word (right-aligned big-endian).
fn pad_u32(v: u32) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[28..].copy_from_slice(&v.to_be_bytes());
    word
}

/// Convert a `U256` to a 32-byte big-endian array for ABI encoding.
fn u256_to_be32(v: U256) -> [u8; 32] {
    v.to_be_bytes()
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE CONSTRUCTORS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExactInputSingleParams {
    /// Build params for a WETH→USDC swap on mainnet (0.05% pool).
    ///
    /// `amount_in_wei` — amount of WETH in wei (1 WETH = 10^18 wei).
    /// `min_usdc_out` — minimum USDC to receive (6 decimals; 1 USDC = 1_000_000).
    /// `recipient` — address that will receive the USDC.
    pub fn weth_to_usdc(
        amount_in_wei: U256,
        min_usdc_out: U256,
        recipient: Address,
    ) -> ExchangeResult<Self> {
        // WETH mainnet: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        let token_in: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
            .parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("WETH address parse error: {}", e)))?;
        // USDC mainnet: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
        let token_out: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            .parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("USDC address parse error: {}", e)))?;

        Ok(Self {
            token_in,
            token_out,
            fee: 500, // 0.05% pool
            recipient,
            amount_in: amount_in_wei,
            amount_out_minimum: min_usdc_out,
            sqrt_price_limit_x96: U256::ZERO,
        })
    }

    /// Build params for a USDC→WETH swap on mainnet (0.05% pool).
    ///
    /// `amount_in_usdc` — amount of USDC (6 decimals; 1 USDC = 1_000_000).
    /// `min_weth_out` — minimum WETH to receive (in wei).
    /// `recipient` — address that will receive the WETH.
    pub fn usdc_to_weth(
        amount_in_usdc: U256,
        min_weth_out: U256,
        recipient: Address,
    ) -> ExchangeResult<Self> {
        let token_in: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            .parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("USDC address parse error: {}", e)))?;
        let token_out: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
            .parse()
            .map_err(|e| ExchangeError::InvalidRequest(format!("WETH address parse error: {}", e)))?;

        Ok(Self {
            token_in,
            token_out,
            fee: 500,
            recipient,
            amount_in: amount_in_usdc,
            amount_out_minimum: min_weth_out,
            sqrt_price_limit_x96: U256::ZERO,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_exact_input_single_length() {
        let recipient: Address = "0x0000000000000000000000000000000000000001"
            .parse()
            .unwrap();
        let params = ExactInputSingleParams {
            token_in: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().unwrap(),
            token_out: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse().unwrap(),
            fee: 500,
            recipient,
            amount_in: U256::from(1_000_000_000_000_000_000u64), // 1 ETH in wei
            amount_out_minimum: U256::from(1_000_000u64), // 1 USDC minimum
            sqrt_price_limit_x96: U256::ZERO,
        };
        let calldata = UniswapOnchain::encode_exact_input_single(&params);
        // 4 (selector) + 7 * 32 (params) = 228 bytes
        assert_eq!(calldata.len(), 4 + 7 * 32);
        assert_eq!(&calldata[..4], &EXACT_INPUT_SINGLE_SELECTOR);
    }

    #[test]
    fn test_pad_address() {
        let addr: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            .parse()
            .unwrap();
        let word = pad_address(addr);
        // First 12 bytes must be zero
        assert_eq!(&word[..12], &[0u8; 12]);
        // Last 20 bytes must be the address
        assert_eq!(&word[12..], addr.as_slice());
    }

    #[test]
    fn test_u256_roundtrip() {
        let v = U256::from(12345678u64);
        let bytes = u256_to_be32(v);
        let back = U256::from_be_slice(&bytes);
        assert_eq!(v, back);
    }
}
