//! # Hyperliquid Authentication
//!
//! EIP-712 wallet-based authentication for Hyperliquid.
//!
//! ## Overview
//!
//! Hyperliquid uses Ethereum wallet signatures (EIP-712) instead of traditional API keys.
//! There are two signing schemes:
//!
//! ### 1. L1 Actions (Phantom Agent)
//! Used for trading operations:
//! - Place/cancel/modify orders
//! - Update leverage
//! - Update isolated margin
//! - USD class transfers
//!
//! ### 2. User-Signed Actions (Direct EIP-712)
//! Used for administrative operations:
//! - Withdrawals to L1
//! - Internal USDC transfers
//! - Internal spot token transfers
//!
//! ## Implementation Note
//!
//! Full EIP-712 signing requires `ethers` or `alloy` crates. This is a placeholder
//! structure that demonstrates the interface. Production implementation should:
//!
//! 1. Use `ethers::signers::LocalWallet` for key management
//! 2. Implement proper EIP-712 domain separator
//! 3. Handle msgpack serialization for L1 actions
//! 4. Implement phantom agent construction
//! 5. Handle nonce management (atomic counter, timestamp-based)
//!
//! ## Dependencies Needed
//!
//! ```toml
//! [dependencies]
//! ethers = "2.0"          # EIP-712 signing
//! rmp-serde = "1.1"       # Msgpack for L1 actions
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
    timestamp_millis,
};

/// Hyperliquid authentication handler
///
/// # Note
/// This is a placeholder implementation. Production version needs:
/// - Full EIP-712 signing with `ethers` crate
/// - Phantom agent construction for L1 actions
/// - Proper msgpack serialization
#[derive(Clone)]
pub struct HyperliquidAuth {
    /// Ethereum wallet address (lowercase)
    wallet_address: String,
    /// Private key (for signing, should be from ethers::LocalWallet)
    _private_key: String,
    /// Nonce counter (atomic for thread safety)
    nonce_counter: Arc<AtomicU64>,
}

impl HyperliquidAuth {
    /// Create new auth handler from credentials
    ///
    /// # Arguments
    /// * `credentials` - Must contain wallet private key in `api_secret` field
    ///
    /// # Note
    /// In production, this should:
    /// - Parse private key with `ethers::signers::LocalWallet::from_str()`
    /// - Extract address and ensure it's lowercase
    /// - Initialize nonce counter with current timestamp
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        if credentials.api_secret.is_empty() {
            return Err(ExchangeError::Auth(
                "Hyperliquid requires wallet private key in api_secret field".to_string()
            ));
        }

        // TODO: Parse with ethers::LocalWallet
        // let wallet = credentials.api_secret.parse::<LocalWallet>()?;
        // let address = format!("{:?}", wallet.address()).to_lowercase();

        // Placeholder implementation
        let private_key = credentials.api_secret.clone();
        let wallet_address = credentials.api_key.clone().to_lowercase();

        let nonce_counter = Arc::new(AtomicU64::new(timestamp_millis()));

        Ok(Self {
            wallet_address,
            _private_key: private_key,
            nonce_counter,
        })
    }

    /// Get wallet address (lowercase)
    pub fn wallet_address(&self) -> &str {
        &self.wallet_address
    }

    /// Get next nonce (timestamp-based)
    ///
    /// Hyperliquid nonces must be:
    /// - Unique per signer
    /// - Within time window (T - 2 days, T + 1 day)
    /// - Larger than smallest nonce in 100-nonce window
    pub fn get_next_nonce(&self) -> u64 {
        let now = timestamp_millis();
        // Ensure nonce is at least current timestamp and monotonically increasing
        self.nonce_counter.fetch_max(now, Ordering::SeqCst);
        self.nonce_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Sign L1 action (phantom agent construction)
    ///
    /// # Arguments
    /// * `action` - Serialized action JSON
    /// * `nonce` - Request nonce
    ///
    /// # Returns
    /// Signature components (r, s, v)
    ///
    /// # Implementation Note
    /// Production version should:
    /// 1. Create connection ID from nonce
    /// 2. Construct phantom agent message with msgpack
    /// 3. Sign with EIP-712
    /// 4. Return (r, s, v) components
    pub fn sign_l1_action(
        &self,
        _action: &str,
        _nonce: u64,
    ) -> ExchangeResult<SignatureComponents> {
        // TODO: Implement full EIP-712 signing
        // See: hyperliquid-python-sdk/hyperliquid/utils/signing.py::sign_l1_action

        Err(ExchangeError::Auth(
            "EIP-712 signing not yet implemented. Requires ethers crate.".to_string()
        ))
    }

    /// Sign user-signed action (direct EIP-712)
    ///
    /// # Arguments
    /// * `action` - Serialized action JSON
    /// * `nonce` - Request nonce
    ///
    /// # Returns
    /// Signature components (r, s, v)
    ///
    /// # Implementation Note
    /// Production version should:
    /// 1. Construct EIP-712 typed data structure
    /// 2. Set domain (chainId, verifyingContract)
    /// 3. Sign with wallet
    /// 4. Return (r, s, v) components
    pub fn sign_user_signed_action(
        &self,
        _action: &str,
        _nonce: u64,
    ) -> ExchangeResult<SignatureComponents> {
        // TODO: Implement full EIP-712 signing
        // See: hyperliquid-python-sdk/hyperliquid/utils/signing.py::sign_user_signed_action

        Err(ExchangeError::Auth(
            "EIP-712 signing not yet implemented. Requires ethers crate.".to_string()
        ))
    }

    /// Build signed request for /exchange endpoint
    ///
    /// # Arguments
    /// * `action` - Action object (will be serialized to JSON)
    /// * `nonce` - Request nonce
    /// * `is_l1_action` - Whether to use L1 signing (phantom agent) or user-signed
    ///
    /// # Returns
    /// Complete request body with signature
    pub fn build_signed_request(
        &self,
        action: &serde_json::Value,
        nonce: u64,
        is_l1_action: bool,
    ) -> ExchangeResult<serde_json::Value> {
        let action_str = serde_json::to_string(action)
            .map_err(|e| ExchangeError::Parse(e.to_string()))?;

        let signature = if is_l1_action {
            self.sign_l1_action(&action_str, nonce)?
        } else {
            self.sign_user_signed_action(&action_str, nonce)?
        };

        Ok(serde_json::json!({
            "action": action,
            "nonce": nonce,
            "signature": {
                "r": signature.r,
                "s": signature.s,
                "v": signature.v,
            },
            "vaultAddress": null, // For subaccount operations
        }))
    }

    /// Get headers for request (no special headers for Hyperliquid)
    pub fn get_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}

/// EIP-712 signature components
#[derive(Debug, Clone)]
pub struct SignatureComponents {
    /// First 32 bytes of signature (hex string with 0x prefix)
    pub r: String,
    /// Last 32 bytes of signature (hex string with 0x prefix)
    pub s: String,
    /// Recovery ID (27 or 28)
    pub v: u8,
}

// ═══════════════════════════════════════════════════════════════════════════════
// EIP-712 DOMAIN AND TYPE DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// EIP-712 domain separator for Hyperliquid
///
/// # Production Implementation
/// ```ignore
/// use ethers::types::transaction::eip712::{Eip712, EIP712Domain};
///
/// fn get_domain(chain_id: u64) -> EIP712Domain {
///     EIP712Domain {
///         name: Some("Exchange".to_string()),
///         version: Some("1".to_string()),
///         chain_id: Some(chain_id.into()),
///         verifying_contract: Some("0x0000000000000000000000000000000000000000".parse().unwrap()),
///         salt: None,
///     }
/// }
/// ```
#[allow(dead_code)]
fn get_chain_id(is_testnet: bool) -> u64 {
    if is_testnet {
        421614 // Arbitrum Sepolia
    } else {
        42161 // Arbitrum One
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_auth() {
        let credentials = Credentials::new(
            "0xabcdef1234567890abcdef1234567890abcdef12",
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );

        let auth = HyperliquidAuth::new(&credentials).unwrap();
        assert_eq!(auth.wallet_address(), "0xabcdef1234567890abcdef1234567890abcdef12");
    }

    #[test]
    fn test_nonce_generation() {
        let credentials = Credentials::new("0xtest", "0xprivkey");
        let auth = HyperliquidAuth::new(&credentials).unwrap();

        let nonce1 = auth.get_next_nonce();
        let nonce2 = auth.get_next_nonce();

        assert!(nonce2 > nonce1, "Nonces must be monotonically increasing");
    }

    #[test]
    fn test_chain_id() {
        assert_eq!(get_chain_id(false), 42161);
        assert_eq!(get_chain_id(true), 421614);
    }
}
