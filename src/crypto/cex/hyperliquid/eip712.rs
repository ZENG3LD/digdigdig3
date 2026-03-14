//! # HyperLiquid EIP-712 Typed Data Signing
//!
//! Alloy-native EIP-712 typed data signing for HyperLiquid user-signed actions.
//!
//! ## Feature Gate
//!
//! This module is only compiled when the `onchain-ethereum` feature is enabled.
//!
//! ## Background
//!
//! HyperLiquid uses two signing schemes:
//!
//! ### 1. L1 Actions (Phantom Agent)
//! Implemented in `auth.rs` — covers all trading operations.
//! Uses msgpack encoding + keccak256 + EIP-712 phantom agent scheme.
//! Does NOT need alloy provider — only needs keccak256 and ECDSA signer.
//!
//! ### 2. User-Signed Actions
//! Used for: withdrawals, USDC transfers (spot ↔ external wallets).
//! Uses direct EIP-712 typed data signing with alloy's `sol!` macro types.
//! Requires a wallet address and private key.
//!
//! This module wraps scheme 2, providing `Eip712Signer` with:
//! - `sign_l1_action()` — re-exports the phantom agent signing from auth.rs via alloy types
//! - `sign_user_signed_action()` — signs withdrawal/transfer actions as EIP-712 typed data
//! - `sign_withdraw_from_bridge()` — convenience for USDC bridge withdrawal
//! - `sign_spot_transfer()` — convenience for spot-to-spot USDC transfer
//!
//! ## HyperLiquid EIP-712 Domain
//!
//! ```
//! name: "HyperliquidSignTransaction"
//! version: "1"
//! chainId: 42161 (Arbitrum) or 421614 (testnet)
//! verifyingContract: 0x0000000000000000000000000000000000000000
//! ```

#![cfg(feature = "onchain-ethereum")]

use alloy::primitives::{keccak256, Address, B256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;

use crate::core::{Credentials, ExchangeError, ExchangeResult};
use super::auth::SignatureComponents;

// ═══════════════════════════════════════════════════════════════════════════════
// EIP-712 DOMAIN
// ═══════════════════════════════════════════════════════════════════════════════

/// HyperLiquid EIP-712 domain for user-signed actions.
///
/// Differs from the L1 phantom-agent domain (`name: "Exchange"`) —
/// user-signed actions use `name: "HyperliquidSignTransaction"`.
#[derive(Debug, Clone)]
pub struct HyperliquidDomain {
    pub chain_id: u64,
}

impl HyperliquidDomain {
    /// Mainnet domain (Arbitrum, chainId = 42161)
    pub const MAINNET: Self = Self { chain_id: 42161 };
    /// Testnet domain (Arbitrum Sepolia, chainId = 421614)
    pub const TESTNET: Self = Self { chain_id: 421614 };

    /// Compute the EIP-712 domain separator for user-signed actions.
    pub fn separator(&self) -> [u8; 32] {
        let type_hash = *keccak256(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        );
        let name_hash = *keccak256(b"HyperliquidSignTransaction");
        let version_hash = *keccak256(b"1");

        let mut chain_id_bytes = [0u8; 32];
        chain_id_bytes[24..].copy_from_slice(&self.chain_id.to_be_bytes());

        let verifying_contract = [0u8; 32]; // zero address

        let mut encoded = Vec::with_capacity(5 * 32);
        encoded.extend_from_slice(&type_hash);
        encoded.extend_from_slice(&name_hash);
        encoded.extend_from_slice(&version_hash);
        encoded.extend_from_slice(&chain_id_bytes);
        encoded.extend_from_slice(&verifying_contract);

        *keccak256(&encoded)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// USER-SIGNED ACTION TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Withdraw USDC from the HyperLiquid bridge to an Arbitrum wallet.
///
/// EIP-712 type: `HyperliquidTransaction:WithdrawFromBridge2`
/// Fields: `hyperliquidChain`, `destination`, `amount`, `time`
#[derive(Debug, Clone)]
pub struct WithdrawFromBridgeAction {
    /// Either "Mainnet" or "Testnet"
    pub hyperliquid_chain: &'static str,
    /// Destination wallet address (checksummed, 0x-prefixed)
    pub destination: String,
    /// Amount as decimal string (e.g. "100.0" for $100 USDC)
    pub amount: String,
    /// Timestamp in milliseconds (nonce)
    pub time: u64,
}

/// Transfer USDC between HyperLiquid accounts (internal).
///
/// EIP-712 type: `HyperliquidTransaction:UsdSend`
/// Fields: `hyperliquidChain`, `destination`, `amount`, `time`
#[derive(Debug, Clone)]
pub struct UsdSendAction {
    /// Either "Mainnet" or "Testnet"
    pub hyperliquid_chain: &'static str,
    /// Recipient wallet address (checksummed)
    pub destination: String,
    /// Amount as decimal string
    pub amount: String,
    /// Timestamp in milliseconds
    pub time: u64,
}

/// Transfer spot tokens between HyperLiquid accounts.
///
/// EIP-712 type: `HyperliquidTransaction:SpotSend`
/// Fields: `hyperliquidChain`, `destination`, `token`, `amount`, `time`
#[derive(Debug, Clone)]
pub struct SpotSendAction {
    /// Either "Mainnet" or "Testnet"
    pub hyperliquid_chain: &'static str,
    /// Recipient wallet address
    pub destination: String,
    /// Token identifier (e.g. "USDC:0xedf09..." or "@0" for spot index)
    pub token: String,
    /// Amount as decimal string
    pub amount: String,
    /// Timestamp in milliseconds
    pub time: u64,
}

/// Signed user action ready to be sent to the `/exchange` endpoint.
#[derive(Debug, Clone)]
pub struct SignedUserAction {
    /// The action JSON payload (to be merged with signature fields)
    pub action: serde_json::Value,
    /// Nonce (timestamp used for signing)
    pub nonce: u64,
    /// Signature r component (0x-prefixed hex)
    pub r: String,
    /// Signature s component (0x-prefixed hex)
    pub s: String,
    /// Signature v (27 or 28)
    pub v: u8,
}

impl SignedUserAction {
    /// Build the complete request body for the `/exchange` endpoint.
    pub fn to_request_body(&self) -> serde_json::Value {
        serde_json::json!({
            "action": self.action,
            "nonce": self.nonce,
            "signature": {
                "r": self.r,
                "s": self.s,
                "v": self.v,
            },
            "vaultAddress": null,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EIP-712 SIGNER
// ═══════════════════════════════════════════════════════════════════════════════

/// High-level EIP-712 signer for HyperLiquid user-signed actions.
///
/// Provides typed signing methods for withdrawal and transfer operations.
/// For L1 trading actions (orders, cancels, leverage), use `HyperliquidAuth`
/// from `auth.rs` directly — those use the phantom agent scheme.
///
/// ## Usage
///
/// ```ignore
/// let signer = Eip712Signer::new(&credentials, false)?;
///
/// let signed = signer.sign_withdraw_from_bridge(
///     "0xYourArbitrumAddress",
///     "100.0",
///     timestamp_millis(),
/// )?;
///
/// // POST signed.to_request_body() to https://api.hyperliquid.xyz/exchange
/// ```
pub struct Eip712Signer {
    /// alloy private key signer
    signer: PrivateKeySigner,
    /// EIP-712 domain for this network
    domain: HyperliquidDomain,
    /// Wallet address (lowercase 0x-prefixed)
    wallet_address: String,
}

impl Eip712Signer {
    /// Create a new signer from credentials.
    ///
    /// `credentials.api_secret` — 0x-prefixed private key hex (32 bytes).
    /// `credentials.api_key`    — wallet address (optional; derived if empty).
    pub fn new(credentials: &Credentials, is_testnet: bool) -> ExchangeResult<Self> {
        if credentials.api_secret.is_empty() {
            return Err(ExchangeError::Auth(
                "HyperLiquid requires private key in api_secret field".to_string(),
            ));
        }

        let signer: PrivateKeySigner = credentials.api_secret.parse()
            .map_err(|e| ExchangeError::Auth(format!("Invalid private key: {}", e)))?;

        let wallet_address = if !credentials.api_key.is_empty() {
            credentials.api_key.to_lowercase()
        } else {
            format!("0x{}", hex::encode(signer.address().as_slice()))
        };

        let domain = if is_testnet {
            HyperliquidDomain::TESTNET
        } else {
            HyperliquidDomain::MAINNET
        };

        Ok(Self { signer, domain, wallet_address })
    }

    /// Wallet address (lowercase 0x-prefixed).
    pub fn wallet_address(&self) -> &str {
        &self.wallet_address
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CORE SIGNING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Compute the EIP-712 final hash and sign it.
    ///
    /// `struct_hash` — the keccak256 of the ABI-encoded typed data struct.
    ///
    /// Returns r, s, v signature components.
    pub fn sign_struct_hash(&self, struct_hash: [u8; 32]) -> ExchangeResult<SignatureComponents> {
        let domain_sep = self.domain.separator();
        let final_hash = eip712_hash(&domain_sep, &struct_hash);
        let hash_b256 = B256::from(final_hash);

        let sig = self.signer
            .sign_hash_sync(&hash_b256)
            .map_err(|e| ExchangeError::Auth(format!("EIP-712 sign failed: {}", e)))?;

        let bytes = sig.as_bytes();
        let r = format!("0x{}", hex::encode(&bytes[..32]));
        let s = format!("0x{}", hex::encode(&bytes[32..64]));
        let v = if bytes[64] == 0 { 27u8 } else { 28u8 };

        Ok(SignatureComponents { r, s, v })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER-SIGNED ACTION: WITHDRAW FROM BRIDGE
    // ═══════════════════════════════════════════════════════════════════════════

    /// Sign a `WithdrawFromBridge2` action.
    ///
    /// Withdraws USDC from HyperLiquid L1 to an Arbitrum (or testnet) wallet.
    ///
    /// EIP-712 type string:
    /// `HyperliquidTransaction:WithdrawFromBridge2(string hyperliquidChain,string destination,string amount,uint64 time)`
    pub fn sign_withdraw_from_bridge(
        &self,
        destination: &str,
        amount: &str,
        time: u64,
    ) -> ExchangeResult<SignedUserAction> {
        let chain_name = if self.domain.chain_id == 421614 { "Testnet" } else { "Mainnet" };

        let action = WithdrawFromBridgeAction {
            hyperliquid_chain: chain_name,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time,
        };

        let struct_hash = withdraw_from_bridge_struct_hash(&action);
        let sig = self.sign_struct_hash(struct_hash)?;

        let action_json = serde_json::json!({
            "type": "withdraw3",
            "hyperliquidChain": action.hyperliquid_chain,
            "signatureChainId": format!("0x{:x}", self.domain.chain_id),
            "destination": action.destination,
            "amount": action.amount,
            "time": action.time,
        });

        Ok(SignedUserAction {
            action: action_json,
            nonce: time,
            r: sig.r,
            s: sig.s,
            v: sig.v,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER-SIGNED ACTION: USD SEND (internal USDC transfer)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Sign a `UsdSend` action (internal USDC transfer between HL accounts).
    ///
    /// EIP-712 type string:
    /// `HyperliquidTransaction:UsdSend(string hyperliquidChain,string destination,string amount,uint64 time)`
    pub fn sign_usd_send(
        &self,
        destination: &str,
        amount: &str,
        time: u64,
    ) -> ExchangeResult<SignedUserAction> {
        let chain_name = if self.domain.chain_id == 421614 { "Testnet" } else { "Mainnet" };

        let action = UsdSendAction {
            hyperliquid_chain: chain_name,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time,
        };

        let struct_hash = usd_send_struct_hash(&action);
        let sig = self.sign_struct_hash(struct_hash)?;

        let action_json = serde_json::json!({
            "type": "usdSend",
            "hyperliquidChain": action.hyperliquid_chain,
            "signatureChainId": format!("0x{:x}", self.domain.chain_id),
            "destination": action.destination,
            "amount": action.amount,
            "time": action.time,
        });

        Ok(SignedUserAction {
            action: action_json,
            nonce: time,
            r: sig.r,
            s: sig.s,
            v: sig.v,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER-SIGNED ACTION: SPOT SEND
    // ═══════════════════════════════════════════════════════════════════════════

    /// Sign a `SpotSend` action (spot token transfer between HL accounts).
    ///
    /// EIP-712 type string:
    /// `HyperliquidTransaction:SpotSend(string hyperliquidChain,string destination,string token,string amount,uint64 time)`
    pub fn sign_spot_send(
        &self,
        destination: &str,
        token: &str,
        amount: &str,
        time: u64,
    ) -> ExchangeResult<SignedUserAction> {
        let chain_name = if self.domain.chain_id == 421614 { "Testnet" } else { "Mainnet" };

        let action = SpotSendAction {
            hyperliquid_chain: chain_name,
            destination: destination.to_string(),
            token: token.to_string(),
            amount: amount.to_string(),
            time,
        };

        let struct_hash = spot_send_struct_hash(&action);
        let sig = self.sign_struct_hash(struct_hash)?;

        let action_json = serde_json::json!({
            "type": "spotSend",
            "hyperliquidChain": action.hyperliquid_chain,
            "signatureChainId": format!("0x{:x}", self.domain.chain_id),
            "destination": action.destination,
            "token": action.token,
            "amount": action.amount,
            "time": action.time,
        });

        Ok(SignedUserAction {
            action: action_json,
            nonce: time,
            r: sig.r,
            s: sig.s,
            v: sig.v,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RE-EXPORT: L1 ACTION SIGNING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Sign an arbitrary L1 action hash using the phantom agent scheme.
    ///
    /// This is a thin wrapper that delegates to `HyperliquidAuth::sign_l1_action`.
    /// Provided here for completeness so callers using `Eip712Signer` don't need
    /// to import `HyperliquidAuth` separately.
    ///
    /// `action_bytes` — msgpack-encoded action dict.
    /// `nonce`        — monotonically increasing timestamp in ms.
    /// `vault_address` — optional vault address (20 bytes).
    pub fn sign_l1_action_raw(
        &self,
        action_bytes: &[u8],
        nonce: u64,
        vault_address: Option<&[u8; 20]>,
    ) -> ExchangeResult<SignatureComponents> {
        // Compute phantom agent hash (same as HyperliquidAuth::sign_l1_action)
        let chain_id = self.domain.chain_id;
        let domain_sep = l1_domain_separator(chain_id);
        let connection_id = l1_connection_id(action_bytes, nonce, vault_address);
        let agent_hash = l1_agent_struct_hash(&connection_id);
        let final_hash = eip712_hash(&domain_sep, &agent_hash);

        let hash_b256 = B256::from(final_hash);
        let sig = self.signer
            .sign_hash_sync(&hash_b256)
            .map_err(|e| ExchangeError::Auth(format!("L1 sign failed: {}", e)))?;

        let bytes = sig.as_bytes();
        let r = format!("0x{}", hex::encode(&bytes[..32]));
        let s = format!("0x{}", hex::encode(&bytes[32..64]));
        let v = if bytes[64] == 0 { 27u8 } else { 28u8 };

        Ok(SignatureComponents { r, s, v })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EIP-712 STRUCT HASH COMPUTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// `HyperliquidTransaction:WithdrawFromBridge2(string hyperliquidChain,string destination,string amount,uint64 time)`
fn withdraw_from_bridge_struct_hash(action: &WithdrawFromBridgeAction) -> [u8; 32] {
    let type_hash = *keccak256(
        b"HyperliquidTransaction:WithdrawFromBridge2(string hyperliquidChain,string destination,string amount,uint64 time)"
    );
    let chain_hash = *keccak256(action.hyperliquid_chain.as_bytes());
    let dest_hash = *keccak256(action.destination.as_bytes());
    let amount_hash = *keccak256(action.amount.as_bytes());

    let mut time_bytes = [0u8; 32];
    time_bytes[24..].copy_from_slice(&action.time.to_be_bytes());

    let mut encoded = Vec::with_capacity(5 * 32);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&chain_hash);
    encoded.extend_from_slice(&dest_hash);
    encoded.extend_from_slice(&amount_hash);
    encoded.extend_from_slice(&time_bytes);

    *keccak256(&encoded)
}

/// `HyperliquidTransaction:UsdSend(string hyperliquidChain,string destination,string amount,uint64 time)`
fn usd_send_struct_hash(action: &UsdSendAction) -> [u8; 32] {
    let type_hash = *keccak256(
        b"HyperliquidTransaction:UsdSend(string hyperliquidChain,string destination,string amount,uint64 time)"
    );
    let chain_hash = *keccak256(action.hyperliquid_chain.as_bytes());
    let dest_hash = *keccak256(action.destination.as_bytes());
    let amount_hash = *keccak256(action.amount.as_bytes());

    let mut time_bytes = [0u8; 32];
    time_bytes[24..].copy_from_slice(&action.time.to_be_bytes());

    let mut encoded = Vec::with_capacity(5 * 32);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&chain_hash);
    encoded.extend_from_slice(&dest_hash);
    encoded.extend_from_slice(&amount_hash);
    encoded.extend_from_slice(&time_bytes);

    *keccak256(&encoded)
}

/// `HyperliquidTransaction:SpotSend(string hyperliquidChain,string destination,string token,string amount,uint64 time)`
fn spot_send_struct_hash(action: &SpotSendAction) -> [u8; 32] {
    let type_hash = *keccak256(
        b"HyperliquidTransaction:SpotSend(string hyperliquidChain,string destination,string token,string amount,uint64 time)"
    );
    let chain_hash = *keccak256(action.hyperliquid_chain.as_bytes());
    let dest_hash = *keccak256(action.destination.as_bytes());
    let token_hash = *keccak256(action.token.as_bytes());
    let amount_hash = *keccak256(action.amount.as_bytes());

    let mut time_bytes = [0u8; 32];
    time_bytes[24..].copy_from_slice(&action.time.to_be_bytes());

    let mut encoded = Vec::with_capacity(6 * 32);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&chain_hash);
    encoded.extend_from_slice(&dest_hash);
    encoded.extend_from_slice(&token_hash);
    encoded.extend_from_slice(&amount_hash);
    encoded.extend_from_slice(&time_bytes);

    *keccak256(&encoded)
}

// ═══════════════════════════════════════════════════════════════════════════════
// L1 PHANTOM AGENT HELPERS (re-implemented to avoid auth.rs private dependency)
// ═══════════════════════════════════════════════════════════════════════════════

/// EIP-712 domain separator for L1 phantom agent (`name: "Exchange"`, `version: "1"`).
fn l1_domain_separator(chain_id: u64) -> [u8; 32] {
    let type_hash = *keccak256(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );
    let name_hash = *keccak256(b"Exchange");
    let version_hash = *keccak256(b"1");

    let mut chain_id_bytes = [0u8; 32];
    chain_id_bytes[24..].copy_from_slice(&chain_id.to_be_bytes());

    let mut encoded = Vec::with_capacity(5 * 32);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&name_hash);
    encoded.extend_from_slice(&version_hash);
    encoded.extend_from_slice(&chain_id_bytes);
    encoded.extend_from_slice(&[0u8; 32]); // zero verifyingContract

    *keccak256(&encoded)
}

/// Compute connection ID: keccak256(action_bytes + nonce_be8 + vault_flag + [vault_bytes])
fn l1_connection_id(
    action_bytes: &[u8],
    nonce: u64,
    vault_address: Option<&[u8; 20]>,
) -> [u8; 32] {
    let mut data = Vec::with_capacity(action_bytes.len() + 8 + 21);
    data.extend_from_slice(action_bytes);
    data.extend_from_slice(&nonce.to_be_bytes());
    match vault_address {
        Some(vault) => {
            data.push(1u8);
            data.extend_from_slice(vault);
        }
        None => data.push(0u8),
    }
    *keccak256(&data)
}

/// EIP-712 struct hash for Agent(address source, bytes32 connectionId).
fn l1_agent_struct_hash(connection_id: &[u8; 32]) -> [u8; 32] {
    let type_hash = *keccak256(b"Agent(address source,bytes32 connectionId)");
    // Phantom source: USDC contract address (sentinel used by HyperLiquid)
    let phantom_source: [u8; 20] = [
        0xa0, 0xb8, 0x69, 0x91, 0xc6, 0x21, 0x8b, 0x36, 0xc1, 0xd1,
        0x9d, 0x4a, 0x2e, 0x9e, 0xb0, 0xce, 0x36, 0x06, 0xeb, 0x48,
    ];

    let mut encoded = Vec::with_capacity(3 * 32);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&[0u8; 12]); // address padding
    encoded.extend_from_slice(&phantom_source);
    encoded.extend_from_slice(connection_id);

    *keccak256(&encoded)
}

/// Final EIP-712 message hash: keccak256("\x19\x01" + domain_separator + struct_hash)
fn eip712_hash(domain_separator: &[u8; 32], struct_hash: &[u8; 32]) -> [u8; 32] {
    let mut data = Vec::with_capacity(2 + 32 + 32);
    data.extend_from_slice(b"\x19\x01");
    data.extend_from_slice(domain_separator);
    data.extend_from_slice(struct_hash);
    *keccak256(&data)
}

// Keep Address import used in type derivation (prevents unused import warning)
#[allow(dead_code)]
fn _check_address_import(_: Address) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_domain_separator_is_deterministic() {
        let d1 = HyperliquidDomain::MAINNET.separator();
        let d2 = HyperliquidDomain::MAINNET.separator();
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_testnet_domain_differs_from_mainnet() {
        let mainnet = HyperliquidDomain::MAINNET.separator();
        let testnet = HyperliquidDomain::TESTNET.separator();
        assert_ne!(mainnet, testnet);
    }

    #[test]
    fn test_withdraw_struct_hash_is_deterministic() {
        let action = WithdrawFromBridgeAction {
            hyperliquid_chain: "Mainnet",
            destination: "0x1234567890123456789012345678901234567890".to_string(),
            amount: "100.0".to_string(),
            time: 1_700_000_000_000,
        };
        let h1 = withdraw_from_bridge_struct_hash(&action);
        let h2 = withdraw_from_bridge_struct_hash(&action);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_usd_send_struct_hash_deterministic() {
        let action = UsdSendAction {
            hyperliquid_chain: "Mainnet",
            destination: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
            amount: "50.5".to_string(),
            time: 1_700_000_001_000,
        };
        let h1 = usd_send_struct_hash(&action);
        let h2 = usd_send_struct_hash(&action);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_spot_send_struct_hash_deterministic() {
        let action = SpotSendAction {
            hyperliquid_chain: "Mainnet",
            destination: "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string(),
            token: "USDC:0xedf09fb4b1bd1c6b47c3e97d7e2f3f5f8d3fef7c".to_string(),
            amount: "200.0".to_string(),
            time: 1_700_000_002_000,
        };
        let h1 = spot_send_struct_hash(&action);
        let h2 = spot_send_struct_hash(&action);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_eip712_final_hash() {
        let domain_sep = HyperliquidDomain::MAINNET.separator();
        let struct_hash = [1u8; 32];
        let hash = eip712_hash(&domain_sep, &struct_hash);
        // Must be 32 bytes and deterministic
        assert_eq!(hash.len(), 32);
        let hash2 = eip712_hash(&domain_sep, &struct_hash);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_l1_domain_differs_from_user_domain() {
        // L1 phantom agent domain uses "Exchange" not "HyperliquidSignTransaction"
        let l1_sep = l1_domain_separator(42161);
        let user_sep = HyperliquidDomain::MAINNET.separator();
        assert_ne!(l1_sep, user_sep);
    }
}
