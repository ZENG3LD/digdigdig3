//! # Lighter Authentication
//!
//! Lighter uses a ZK-native cryptographic stack for authentication:
//! - **Curve**: ECgFp5 (Schnorr signatures over a 5-isogeny of the Goldilocks curve)
//! - **Hash**: Poseidon2 (ZK-friendly sponge over the Goldilocks field)
//! - **Field**: Goldilocks (p = 2^64 − 2^32 + 1)
//!
//! ## Token Formats
//!
//! 1. **Auth Token** (Write / WebSocket)
//!    - Format: `{expiry_unix}:{account_index}:{api_key_index}:{base64(80-byte-signature)}`
//!    - Max expiry: 8 hours (28 800 s)
//!
//! 2. **Read-Only Token**
//!    - Same format: `{expiry_unix}:{account_index}:{api_key_index}:{base64(80-byte-signature)}`
//!    - Max expiry: 10 years
//!
//! ## Private Key Format
//!
//! The API private key (`api_secret`) is a hex-encoded 40-byte ECgFp5 scalar.
//! Example: `"0123456789abcdef..."`  (80 hex chars = 40 bytes)

use std::collections::HashMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

use super::crypto::{
    hash_auth_token_bytes, hash_create_order_bytes, hash_cancel_order_bytes,
    sign_hashed_message,
    CreateOrderFields, CancelOrderFields,
    CHAIN_ID_MAINNET, CHAIN_ID_TESTNET,
    NIL_CLIENT_ORDER_INDEX,
};

/// Lighter authentication handler.
///
/// Holds the decoded private key (40 bytes) plus account metadata extracted
/// from the credentials passphrase JSON.
#[derive(Clone, Debug)]
pub struct LighterAuth {
    /// 40-byte ECgFp5 scalar private key (hex-decoded from `api_secret`)
    private_key: Option<[u8; 40]>,
    /// API key index (from passphrase JSON: `"api_key_index"`)
    api_key_index: Option<u16>,
    /// Account index (from passphrase JSON: `"account_index"`)
    account_index: Option<u64>,
    /// L1 Ethereum address (from passphrase JSON: `"l1_address"`)
    l1_address: Option<String>,
    /// Whether this connector is using testnet
    testnet: bool,
}

impl LighterAuth {
    /// Create a new auth handler from credentials.
    ///
    /// # Credentials layout
    /// - `api_key`: unused (pass `""` if not applicable)
    /// - `api_secret`: hex-encoded 40-byte ECgFp5 private key
    /// - `passphrase`: JSON with extra parameters:
    ///   `{"api_key_index": 0, "account_index": 123, "l1_address": "0x...", "testnet": false}`
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        // Decode private key from hex (api_secret field)
        let private_key = if credentials.api_secret.is_empty() {
            None
        } else {
            let bytes = hex::decode(&credentials.api_secret).map_err(|e| {
                ExchangeError::Auth(format!(
                    "Lighter api_secret must be a hex-encoded 40-byte ECgFp5 private key: {}",
                    e
                ))
            })?;

            if bytes.len() != 40 {
                return Err(ExchangeError::Auth(format!(
                    "Lighter ECgFp5 private key must be exactly 40 bytes, got {} bytes ({}  hex chars)",
                    bytes.len(),
                    credentials.api_secret.len(),
                )));
            }

            let mut key = [0u8; 40];
            key.copy_from_slice(&bytes);
            Some(key)
        };

        // Parse extra params from passphrase JSON
        let (api_key_index, account_index, l1_address, testnet) =
            if let Some(passphrase) = &credentials.passphrase {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(passphrase) {
                    let api_key_idx = json.get("api_key_index")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u16);
                    let account_idx = json.get("account_index")
                        .and_then(|v| v.as_u64());
                    let l1_addr = json.get("l1_address")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let is_testnet = json.get("testnet")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    (api_key_idx, account_idx, l1_addr, is_testnet)
                } else {
                    (None, None, None, false)
                }
            } else {
                (None, None, None, false)
            };

        Ok(Self {
            private_key,
            api_key_index,
            account_index,
            l1_address,
            testnet,
        })
    }

    /// Create an auth handler without credentials (public-only access).
    pub fn public_only() -> Self {
        Self {
            private_key: None,
            api_key_index: None,
            account_index: None,
            l1_address: None,
            testnet: false,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AUTH TOKENS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Generate a Lighter auth token using ECgFp5+Poseidon2 Schnorr signing.
    ///
    /// The token format is:
    /// `"{deadline_unix}:{account_index}:{api_key_index}:{base64(80-byte-signature)}"`
    ///
    /// # Arguments
    /// - `expiry_seconds`: token lifetime in seconds from now (max 28 800 = 8 h)
    ///
    /// # Returns
    /// The complete auth token string ready to use as the `Authorization` header value.
    pub fn generate_auth_token(&self, expiry_seconds: u64) -> ExchangeResult<String> {
        let private_key = self.private_key.as_ref().ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter auth token requires a private key (api_secret hex-encoded 40 bytes)."
                    .to_string(),
            )
        })?;

        let account_index = self.account_index.ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter auth token requires account_index in passphrase JSON.".to_string(),
            )
        })?;

        let api_key_index = self.api_key_index.ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter auth token requires api_key_index in passphrase JSON.".to_string(),
            )
        })?;

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let deadline = now_secs + expiry_seconds;

        // Hash: Poseidon2 over the string "{deadline}:{account_index}:{api_key_index}"
        let hash_bytes = hash_auth_token_bytes(deadline, account_index as i64, api_key_index as u8);

        // Sign the 40-byte hash
        let signature = sign_hashed_message(private_key, &hash_bytes);

        // Encode signature as standard base64
        let sig_b64 = BASE64.encode(signature);

        Ok(format!("{}:{}:{}:{}", deadline, account_index, api_key_index, sig_b64))
    }

    /// Generate a read-only auth token.
    ///
    /// Uses the same signing format as a regular auth token. Lighter's server
    /// distinguishes read-only vs write access via the API key's permission level
    /// registered on-chain, not via a different token format.
    pub fn generate_readonly_token(&self, expiry_seconds: u64) -> ExchangeResult<String> {
        self.generate_auth_token(expiry_seconds)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRANSACTION SIGNING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Sign an L2CreateOrder transaction (tx_type = 14).
    ///
    /// Computes the Poseidon2 hash of all 16 transaction fields and signs it
    /// with the ECgFp5 Schnorr scheme. Returns a base64-encoded 80-byte signature.
    ///
    /// # Arguments
    ///
    /// All monetary amounts and prices are already in the Lighter wire format:
    /// - `base_amount`: signed integer in the asset's smallest representable unit
    /// - `price`: unsigned 32-bit price tick
    /// - `nonce`: next nonce fetched from `/api/v1/nextNonce`
    /// - `expired_at_ms`: Unix timestamp in **milliseconds** when the order expires
    /// - `order_expiry_ms`: order-level expiry; pass `-1` for default (now + 28 days)
    /// - `client_order_index`: pass `NIL_CLIENT_ORDER_INDEX` if none
    #[allow(clippy::too_many_arguments)]
    pub fn sign_create_order(
        &self,
        market_index: i16,
        nonce: i64,
        expired_at_ms: i64,
        base_amount: i64,
        price: u32,
        is_ask: bool,
        order_type: u8,
        time_in_force: u8,
        reduce_only: bool,
        trigger_price: u32,
        order_expiry_ms: i64,
        client_order_index: Option<i64>,
    ) -> ExchangeResult<String> {
        let private_key = self.private_key.as_ref().ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter order signing requires a private key (api_secret).".to_string(),
            )
        })?;

        let account_index = self.account_index.ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter order signing requires account_index in passphrase JSON.".to_string(),
            )
        })?;

        let api_key_index = self.api_key_index.ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter order signing requires api_key_index in passphrase JSON.".to_string(),
            )
        })?;

        let chain_id = if self.testnet {
            CHAIN_ID_TESTNET
        } else {
            CHAIN_ID_MAINNET
        };

        let fields = CreateOrderFields {
            chain_id,
            nonce,
            expired_at: expired_at_ms,
            account_index: account_index as i64,
            api_key_index: api_key_index as u8,
            market_index,
            client_order_index: client_order_index.unwrap_or(NIL_CLIENT_ORDER_INDEX),
            base_amount,
            price,
            is_ask,
            order_type,
            time_in_force,
            reduce_only,
            trigger_price,
            order_expiry: order_expiry_ms,
        };

        let hash_bytes = hash_create_order_bytes(&fields);
        let signature = sign_hashed_message(private_key, &hash_bytes);
        Ok(BASE64.encode(signature))
    }

    /// Sign an L2CancelOrder transaction (tx_type = 15).
    ///
    /// Returns a base64-encoded 80-byte signature.
    pub fn sign_cancel_order(
        &self,
        market_index: i16,
        nonce: i64,
        expired_at_ms: i64,
        order_index: i64,
    ) -> ExchangeResult<String> {
        let private_key = self.private_key.as_ref().ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter cancel-order signing requires a private key (api_secret).".to_string(),
            )
        })?;

        let account_index = self.account_index.ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter cancel-order signing requires account_index in passphrase JSON."
                    .to_string(),
            )
        })?;

        let api_key_index = self.api_key_index.ok_or_else(|| {
            ExchangeError::Auth(
                "Lighter cancel-order signing requires api_key_index in passphrase JSON."
                    .to_string(),
            )
        })?;

        let chain_id = if self.testnet {
            CHAIN_ID_TESTNET
        } else {
            CHAIN_ID_MAINNET
        };

        let fields = CancelOrderFields {
            chain_id,
            nonce,
            expired_at: expired_at_ms,
            account_index: account_index as i64,
            api_key_index: api_key_index as u8,
            market_index,
            index: order_index,
        };

        let hash_bytes = hash_cancel_order_bytes(&fields);
        let signature = sign_hashed_message(private_key, &hash_bytes);
        Ok(BASE64.encode(signature))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCESSORS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get account index.
    pub fn account_index(&self) -> Option<u64> {
        self.account_index
    }

    /// Get API key index.
    pub fn api_key_index(&self) -> Option<u16> {
        self.api_key_index
    }

    /// Get L1 Ethereum address.
    pub fn l1_address(&self) -> Option<&str> {
        self.l1_address.as_deref()
    }

    /// Returns true if this auth handler has a private key configured.
    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    /// Create HTTP headers for authenticated requests.
    ///
    /// Inserts `Authorization: {auth_token}` if a valid token can be generated,
    /// otherwise returns only the content-type header.
    pub fn create_headers(&self, auth_token: Option<&str>) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        if let Some(token) = auth_token {
            headers.insert("Authorization".to_string(), token.to_string());
        }
        headers
    }

    /// Generate and insert an Authorization header (1-hour token).
    ///
    /// Convenience wrapper used by connector methods that need auth headers.
    /// On failure (e.g. no private key configured) silently omits the header
    /// so public endpoints continue to work.
    pub fn make_auth_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        if let Ok(token) = self.generate_auth_token(3600) {
            headers.insert("Authorization".to_string(), token);
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_credentials() -> Credentials {
        // 40-byte key: all 0x01 bytes → 80 hex chars
        let private_key_hex = "01".repeat(40);
        let passphrase = r#"{"account_index": 1, "api_key_index": 0}"#;
        Credentials::new("", &private_key_hex)
            .with_passphrase(passphrase)
    }

    #[test]
    fn test_public_only() {
        let auth = LighterAuth::public_only();
        assert!(auth.account_index.is_none());
        assert!(auth.api_key_index.is_none());
        assert!(!auth.has_private_key());
    }

    #[test]
    fn test_new_parses_credentials() {
        let creds = make_test_credentials();
        let auth = LighterAuth::new(&creds).unwrap();
        assert_eq!(auth.account_index(), Some(1));
        assert_eq!(auth.api_key_index(), Some(0));
        assert!(auth.has_private_key());
    }

    #[test]
    fn test_private_key_wrong_length_errors() {
        // 32 bytes = 64 hex chars — should fail
        let short_key_hex = "ab".repeat(32);
        let creds = Credentials::new("", &short_key_hex);
        let err = LighterAuth::new(&creds).unwrap_err();
        assert!(err.to_string().contains("40 bytes"), "Expected 40 bytes error, got: {}", err);
    }

    #[test]
    fn test_generate_auth_token_produces_correct_format() {
        let creds = make_test_credentials();
        let auth = LighterAuth::new(&creds).unwrap();
        let token = auth.generate_auth_token(3600).unwrap();

        // Token must be: "{unix_ts}:{account_index}:{api_key_index}:{base64sig}"
        let parts: Vec<&str> = token.splitn(4, ':').collect();
        assert_eq!(parts.len(), 4, "Token must have 4 colon-separated parts: {}", token);

        // Part 0: deadline (valid Unix timestamp > now)
        let deadline: u64 = parts[0].parse().expect("Deadline must be a u64");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(deadline > now, "Deadline must be in the future");

        // Part 1: account_index
        assert_eq!(parts[1], "1");

        // Part 2: api_key_index
        assert_eq!(parts[2], "0");

        // Part 3: base64-encoded 80-byte signature
        let sig_bytes = BASE64.decode(parts[3]).expect("Signature must be valid base64");
        assert_eq!(sig_bytes.len(), 80, "Signature must be 80 bytes");
    }

    #[test]
    fn test_sign_create_order_returns_80_byte_base64() {
        let creds = make_test_credentials();
        let auth = LighterAuth::new(&creds).unwrap();

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let sig_b64 = auth.sign_create_order(
            0,               // market_index (ETH)
            1,               // nonce
            now_ms + 3_600_000, // expired_at_ms (+1h)
            10_000,          // base_amount
            400_000,         // price tick
            false,           // is_ask (buy)
            0,               // ORDER_TYPE_LIMIT
            2,               // TIF_POST_ONLY
            false,           // reduce_only
            0,               // trigger_price
            now_ms + 28 * 24 * 3_600_000, // order_expiry_ms
            None,            // client_order_index
        ).unwrap();

        let sig_bytes = BASE64.decode(&sig_b64).expect("Signature must be valid base64");
        assert_eq!(sig_bytes.len(), 80, "ECgFp5 Schnorr signature must be 80 bytes");
    }

    #[test]
    fn test_sign_cancel_order_returns_80_byte_base64() {
        let creds = make_test_credentials();
        let auth = LighterAuth::new(&creds).unwrap();

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let sig_b64 = auth.sign_cancel_order(
            0,               // market_index
            2,               // nonce
            now_ms + 3_600_000, // expired_at_ms
            9_876,           // order_index
        ).unwrap();

        let sig_bytes = BASE64.decode(&sig_b64).expect("Signature must be valid base64");
        assert_eq!(sig_bytes.len(), 80, "ECgFp5 Schnorr signature must be 80 bytes");
    }

    #[test]
    fn test_no_private_key_errors_gracefully() {
        let creds = Credentials::new("", "")
            .with_passphrase(r#"{"account_index": 1, "api_key_index": 0}"#);
        let auth = LighterAuth::new(&creds).unwrap();

        let err = auth.generate_auth_token(3600).unwrap_err();
        assert!(
            err.to_string().contains("private key"),
            "Expected private key error, got: {}",
            err
        );
    }
}
