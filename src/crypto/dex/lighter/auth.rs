//! # Lighter Authentication
//!
//! Lighter uses cryptographic signature-based authentication with ECDSA.
//!
//! ## Authentication Methods
//!
//! 1. **Transaction Signing** (Write Operations)
//!    - ECDSA signature with API key private key
//!    - Used for orders, cancellations, modifications
//!    - Requires nonce management
//!
//! 2. **Auth Tokens** (Read Operations)
//!    - Standard format: `{expiry}:{account_index}:{api_key_index}:{random_hex}`
//!    - Max expiry: 8 hours
//!    - Used for WebSocket and authenticated REST endpoints
//!
//! 3. **Read-Only Tokens**
//!    - Format: `ro:{account_index}:{single|all}:{expiry}:{random_hex}`
//!    - Max expiry: 10 years
//!    - Safer for data access without trading
//!
//! ## Implementation Note
//!
//! Phase 1: Focus on PUBLIC market data (no auth needed)
//! Phase 2: Implement auth token generation for account data
//! Phase 3: Implement transaction signing for trading

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
    timestamp_seconds,
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

    /// Generate standard auth token
    ///
    /// Format: `{expiry_unix}:{account_index}:{api_key_index}:{random_hex}`
    ///
    /// # Arguments
    /// * `expiry_seconds` - Expiry duration in seconds (max 8 hours = 28800)
    ///
    /// # Note
    /// This is a placeholder implementation. The actual token should be signed
    /// using the API key private key in a production implementation.
    pub fn generate_auth_token(&self, expiry_seconds: u64) -> ExchangeResult<String> {
        let account_index = self.account_index
            .ok_or_else(|| ExchangeError::Auth("account_index required for auth token".to_string()))?;
        let api_key_index = self.api_key_index
            .ok_or_else(|| ExchangeError::Auth("api_key_index required for auth token".to_string()))?;

        // Max 8 hours
        let max_expiry = 8 * 60 * 60;
        let expiry = std::cmp::min(expiry_seconds, max_expiry);

        let expiry_time = timestamp_seconds() + expiry;
        let random_hex = self.generate_random_hex();

        Ok(format!("{}:{}:{}:{}", expiry_time, account_index, api_key_index, random_hex))
    }

    /// Generate read-only auth token
    ///
    /// Format: `ro:{account_index}:{single|all}:{expiry_unix}:{random_hex}`
    ///
    /// # Arguments
    /// * `expiry_seconds` - Expiry duration in seconds (min 1 day, max 10 years)
    /// * `scope` - "single" or "all" for sub-accounts
    pub fn generate_readonly_token(&self, expiry_seconds: u64, scope: &str) -> ExchangeResult<String> {
        let account_index = self.account_index
            .ok_or_else(|| ExchangeError::Auth("account_index required for readonly token".to_string()))?;

        // Min 1 day, max 10 years
        let min_expiry = 24 * 60 * 60;
        let max_expiry = 10 * 365 * 24 * 60 * 60;
        let expiry = std::cmp::max(min_expiry, std::cmp::min(expiry_seconds, max_expiry));

        let expiry_time = timestamp_seconds() + expiry;
        let random_hex = self.generate_random_hex();

        Ok(format!("ro:{}:{}:{}:{}", account_index, scope, expiry_time, random_hex))
    }

    /// Sign transaction (placeholder for Phase 3)
    ///
    /// # Note
    /// This requires ECDSA signing implementation with the API key private key.
    /// For now, returns an error to indicate it's not implemented.
    pub fn sign_transaction(
        &self,
        _tx_type: u8,
        _tx_data: &HashMap<String, serde_json::Value>,
    ) -> ExchangeResult<String> {
        Err(ExchangeError::Auth(
            "Transaction signing not yet implemented (Phase 3)".to_string()
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

    /// Generate random hex string (8 characters)
    fn generate_random_hex(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Simple random hex based on timestamp
        // In production, use a proper CSPRNG
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .subsec_nanos();

        format!("{:08x}", nanos)
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
    fn test_generate_auth_token() {
        let passphrase = r#"{"account_index": 1, "api_key_index": 3}"#;
        let credentials = Credentials::new("dummy_key", "dummy_secret")
            .with_passphrase(passphrase);

        let auth = LighterAuth::new(&credentials).unwrap();
        let token = auth.generate_auth_token(3600).unwrap();

        // Token format: {expiry}:{account_index}:{api_key_index}:{random_hex}
        let parts: Vec<&str> = token.split(':').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[1], "1"); // account_index
        assert_eq!(parts[2], "3"); // api_key_index
    }

    #[test]
    fn test_generate_readonly_token() {
        let passphrase = r#"{"account_index": 1}"#;
        let credentials = Credentials::new("dummy_key", "dummy_secret")
            .with_passphrase(passphrase);

        let auth = LighterAuth::new(&credentials).unwrap();
        let token = auth.generate_readonly_token(86400, "single").unwrap();

        // Token format: ro:{account_index}:{single|all}:{expiry}:{random_hex}
        let parts: Vec<&str> = token.split(':').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "ro");
        assert_eq!(parts[1], "1"); // account_index
        assert_eq!(parts[2], "single");
    }
}
