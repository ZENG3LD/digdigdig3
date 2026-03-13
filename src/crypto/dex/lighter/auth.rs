//! # Lighter Authentication
//!
//! Lighter uses a ZK-native cryptographic stack for authentication — **NOT** standard ECDSA.
//!
//! ## Cryptographic Stack
//!
//! - **Curve**: ECgFp5 (a 5-isogeny of the Goldilocks curve, Schnorr signatures)
//! - **Hash**: Poseidon2 (ZK-friendly sponge hash over the Goldilocks field)
//! - **Field**: Goldilocks (p = 2^64 − 2^32 + 1)
//!
//! This stack is incompatible with standard `k256`, `secp256k1`, or `ed25519` libraries.
//! Implementing it requires either the `lighter-sdk` Rust crate or a custom ECgFp5
//! implementation of Schnorr signing over the Goldilocks field.
//!
//! ## Token Formats (structure only — signing not implemented)
//!
//! 1. **Auth Token** (Write / WebSocket)
//!    - Format: `{expiry_unix}:{account_index}:{api_key_index}:{random_hex}`
//!    - The random_hex part must be replaced by a valid Schnorr+Poseidon2 signature.
//!    - Max expiry: 8 hours (28 800 s)
//!
//! 2. **Read-Only Token**
//!    - Format: `ro:{account_index}:{single|all}:{expiry_unix}:{random_hex}`
//!    - Max expiry: 10 years
//!
//! ## Current Status
//!
//! All signing operations return `ExchangeError::Auth` with a descriptive message.
//! Public market data endpoints do not require authentication and work as expected.

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// Lighter authentication handler
#[derive(Clone)]
pub struct LighterAuth {
    _api_key_private: Option<String>,
    api_key_index: Option<u16>,
    account_index: Option<u64>,
    l1_address: Option<String>,
}

impl LighterAuth {
    /// Create new auth handler
    ///
    /// # Arguments
    /// * `credentials` - API credentials
    ///   - `api_key`: Can be used to pass account_index (as string)
    ///   - `api_secret`: API key private key
    ///   - `passphrase`: Can be JSON with additional params: {"api_key_index": 1, "account_index": 123, "l1_address": "0x..."}
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        // Try to parse passphrase as JSON for extra params
        let (api_key_index, account_index, l1_address) = if let Some(passphrase) = &credentials.passphrase {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(passphrase) {
                let api_key_idx = json.get("api_key_index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16);
                let account_idx = json.get("account_index")
                    .and_then(|v| v.as_u64());
                let l1_addr = json.get("l1_address")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                (api_key_idx, account_idx, l1_addr)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        Ok(Self {
            _api_key_private: Some(credentials.api_secret.clone()),
            api_key_index,
            account_index,
            l1_address,
        })
    }

    /// Create auth handler without credentials (public-only)
    pub fn public_only() -> Self {
        Self {
            _api_key_private: None,
            api_key_index: None,
            account_index: None,
            l1_address: None,
        }
    }

    /// Generate standard auth token.
    ///
    /// **NOT IMPLEMENTED** — Lighter auth tokens require a Schnorr signature over the
    /// Goldilocks field using ECgFp5 + Poseidon2 hashing. This cryptographic stack is not
    /// available in standard Rust crates (`k256`, `secp256k1`, etc.) and requires either
    /// the official `lighter-sdk` crate or a custom ECgFp5 implementation.
    ///
    /// Token format (for reference): `{expiry_unix}:{account_index}:{api_key_index}:{signature}`
    pub fn generate_auth_token(&self, _expiry_seconds: u64) -> ExchangeResult<String> {
        Err(ExchangeError::Auth(
            "Lighter auth token generation requires ZK-native Schnorr+ECgFp5+Poseidon2 signing \
             over the Goldilocks field. This is incompatible with standard ECDSA libraries. \
             Use the official lighter-sdk or implement ECgFp5 signing manually."
                .to_string(),
        ))
    }

    /// Generate read-only auth token.
    ///
    /// **NOT IMPLEMENTED** — Same signing requirement as `generate_auth_token`.
    /// Read-only tokens use the format `ro:{account_index}:{single|all}:{expiry_unix}:{signature}`.
    pub fn generate_readonly_token(&self, _expiry_seconds: u64, _scope: &str) -> ExchangeResult<String> {
        Err(ExchangeError::Auth(
            "Lighter read-only token generation requires ZK-native Schnorr+ECgFp5+Poseidon2 signing. \
             Use the official lighter-sdk or implement ECgFp5 signing manually."
                .to_string(),
        ))
    }

    /// Sign a Lighter transaction.
    ///
    /// **NOT IMPLEMENTED** — Lighter transaction signing requires:
    /// - **Curve**: ECgFp5 (Schnorr over the Goldilocks field, NOT secp256k1/ECDSA)
    /// - **Hash**: Poseidon2 sponge (ZK-friendly, NOT SHA-256/Keccak)
    ///
    /// Transaction types include:
    /// - `tx_type = 14` — L2CreateOrder
    /// - `tx_type = 15` — L2CancelOrder
    ///
    /// To implement: add the `lighter-sdk` crate (if published) or port the ECgFp5
    /// Schnorr+Poseidon2 signing logic from the Lighter TypeScript SDK.
    pub fn sign_transaction(
        &self,
        _tx_type: u8,
        _tx_data: &HashMap<String, serde_json::Value>,
    ) -> ExchangeResult<String> {
        Err(ExchangeError::Auth(
            "Lighter transaction signing requires ZK-native Schnorr+ECgFp5+Poseidon2 signing \
             (NOT standard ECDSA/secp256k1). Use the official lighter-sdk or port the \
             ECgFp5 signing from the Lighter TypeScript SDK."
                .to_string(),
        ))
    }

    /// Get account index
    pub fn account_index(&self) -> Option<u64> {
        self.account_index
    }

    /// Get L1 address
    pub fn l1_address(&self) -> Option<&str> {
        self.l1_address.as_deref()
    }

    /// Create headers for authenticated requests
    ///
    /// For Lighter, most authenticated endpoints use query parameters
    /// rather than headers, but WebSocket uses auth token in subscription.
    pub fn create_headers(&self, _auth_token: Option<&str>) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_only() {
        let auth = LighterAuth::public_only();
        assert!(auth.account_index.is_none());
        assert!(auth.api_key_index.is_none());
    }

    #[test]
    fn test_generate_auth_token_returns_error() {
        // Signing requires ZK-native ECgFp5+Poseidon2 — not yet implemented.
        let passphrase = r#"{"account_index": 1, "api_key_index": 3}"#;
        let credentials = Credentials::new("dummy_key", "dummy_secret")
            .with_passphrase(passphrase);

        let auth = LighterAuth::new(&credentials).unwrap();
        let result = auth.generate_auth_token(3600);

        assert!(result.is_err(), "Expected auth token generation to fail (ZK signing not implemented)");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("ECgFp5"), "Error should mention ECgFp5: {}", err_msg);
    }

    #[test]
    fn test_generate_readonly_token_returns_error() {
        // Signing requires ZK-native ECgFp5+Poseidon2 — not yet implemented.
        let passphrase = r#"{"account_index": 1}"#;
        let credentials = Credentials::new("dummy_key", "dummy_secret")
            .with_passphrase(passphrase);

        let auth = LighterAuth::new(&credentials).unwrap();
        let result = auth.generate_readonly_token(86400, "single");

        assert!(result.is_err(), "Expected readonly token generation to fail (ZK signing not implemented)");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("ECgFp5"), "Error should mention ECgFp5: {}", err_msg);
    }
}
