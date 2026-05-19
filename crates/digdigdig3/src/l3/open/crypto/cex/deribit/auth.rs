//! # Deribit Authentication
//!
//! OAuth 2.0 style authentication implementation for Deribit API.
//!
//! ## Authentication Flow
//!
//! 1. Call `public/auth` with client credentials or client signature
//! 2. Receive access token (15 min expiry) and refresh token
//! 3. Use access token in Authorization header: `Bearer {token}`
//! 4. Refresh token before expiry using refresh token grant
//!
//! ## Grant Types
//!
//! - **client_credentials**: Simple (sends client_id + client_secret)
//! - **client_signature**: Secure (HMAC-SHA256 signature, doesn't send secret)
//! - **refresh_token**: Extend session without re-authenticating

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::{
    hmac_sha256, encode_hex, timestamp_millis,
    Credentials, ExchangeResult, ExchangeError,
};

/// Deribit authentication handler
#[derive(Clone)]
pub struct DeribitAuth {
    client_id: String,
    client_secret: String,
    /// Access token (JWT)
    access_token: Option<String>,
    /// Refresh token
    refresh_token: Option<String>,
    /// Token expiry time
    expires_at: Option<Instant>,
}

impl DeribitAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            client_id: credentials.api_key.clone(),
            client_secret: credentials.api_secret.clone(),
            access_token: None,
            refresh_token: None,
            expires_at: None,
        })
    }

    /// Check if we have a valid access token
    pub fn has_valid_token(&self) -> bool {
        if let (Some(_token), Some(expires)) = (&self.access_token, self.expires_at) {
            // Consider token valid if more than 60 seconds remaining
            expires > Instant::now() + Duration::from_secs(60)
        } else {
            false
        }
    }

    /// Get access token (if valid)
    pub fn access_token(&self) -> Option<&str> {
        if self.has_valid_token() {
            self.access_token.as_deref()
        } else {
            None
        }
    }

    /// Get refresh token
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    /// Store authentication response
    pub fn store_tokens(
        &mut self,
        access_token: String,
        refresh_token: String,
        expires_in: u64,
    ) {
        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);
        self.expires_at = Some(Instant::now() + Duration::from_secs(expires_in));
    }

    /// Clear tokens (on error or logout)
    pub fn clear_tokens(&mut self) {
        self.access_token = None;
        self.refresh_token = None;
        self.expires_at = None;
    }

    /// Build client credentials grant params
    pub fn client_credentials_params(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), serde_json::json!("client_credentials"));
        params.insert("client_id".to_string(), serde_json::json!(self.client_id));
        params.insert("client_secret".to_string(), serde_json::json!(self.client_secret));
        params
    }

    /// Build client signature grant params (more secure)
    ///
    /// # Signature Algorithm
    /// 1. Generate timestamp (milliseconds)
    /// 2. Generate random nonce
    /// 3. Create message: `{timestamp}\n{nonce}\n{data}`
    /// 4. Sign with HMAC-SHA256
    /// 5. Encode as hex
    pub fn client_signature_params(&self) -> HashMap<String, serde_json::Value> {
        let timestamp = timestamp_millis();
        let nonce = generate_nonce();
        let data = ""; // Optional additional data

        // Build signature string
        let message = format!("{}\n{}\n{}", timestamp, nonce, data);

        // HMAC-SHA256
        let signature_bytes = hmac_sha256(
            self.client_secret.as_bytes(),
            message.as_bytes(),
        );
        let signature = encode_hex(&signature_bytes);

        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), serde_json::json!("client_signature"));
        params.insert("client_id".to_string(), serde_json::json!(self.client_id));
        params.insert("timestamp".to_string(), serde_json::json!(timestamp));
        params.insert("nonce".to_string(), serde_json::json!(nonce));
        params.insert("signature".to_string(), serde_json::json!(signature));
        params.insert("data".to_string(), serde_json::json!(data));

        params
    }

    /// Build refresh token grant params
    pub fn refresh_token_params(&self) -> ExchangeResult<HashMap<String, serde_json::Value>> {
        let refresh_token = self.refresh_token.as_ref()
            .ok_or_else(|| ExchangeError::Auth("No refresh token available".to_string()))?;

        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), serde_json::json!("refresh_token"));
        params.insert("refresh_token".to_string(), serde_json::json!(refresh_token));

        Ok(params)
    }

    /// Get Authorization header value
    pub fn auth_header(&self) -> ExchangeResult<String> {
        let token = self.access_token()
            .ok_or_else(|| ExchangeError::Auth("No valid access token".to_string()))?;

        Ok(format!("Bearer {}", token))
    }

    /// Get client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

/// Generate random nonce for client signature
fn generate_nonce() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const NONCE_LEN: usize = 16;

    let mut rng = rand::thread_rng();
    (0..NONCE_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_credentials_params() {
        let credentials = Credentials::new("test_client_id", "test_client_secret");
        let auth = DeribitAuth::new(&credentials).unwrap();

        let params = auth.client_credentials_params();
        assert_eq!(params.get("grant_type").unwrap(), "client_credentials");
        assert_eq!(params.get("client_id").unwrap(), "test_client_id");
        assert_eq!(params.get("client_secret").unwrap(), "test_client_secret");
    }

    #[test]
    fn test_client_signature_params() {
        let credentials = Credentials::new("test_client_id", "test_client_secret");
        let auth = DeribitAuth::new(&credentials).unwrap();

        let params = auth.client_signature_params();
        assert_eq!(params.get("grant_type").unwrap(), "client_signature");
        assert_eq!(params.get("client_id").unwrap(), "test_client_id");
        assert!(params.contains_key("timestamp"));
        assert!(params.contains_key("nonce"));
        assert!(params.contains_key("signature"));
        assert!(params.contains_key("data"));

        // Verify signature is hex-encoded (64 chars for SHA256)
        let signature = params.get("signature").unwrap().as_str().unwrap();
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_token_lifecycle() {
        let credentials = Credentials::new("test_id", "test_secret");
        let mut auth = DeribitAuth::new(&credentials).unwrap();

        // Initially no token
        assert!(!auth.has_valid_token());
        assert!(auth.access_token().is_none());

        // Store tokens
        auth.store_tokens(
            "test_access_token".to_string(),
            "test_refresh_token".to_string(),
            900, // 15 min
        );

        // Now we have valid token
        assert!(auth.has_valid_token());
        assert_eq!(auth.access_token(), Some("test_access_token"));
        assert_eq!(auth.refresh_token(), Some("test_refresh_token"));

        // Clear tokens
        auth.clear_tokens();
        assert!(!auth.has_valid_token());
        assert!(auth.access_token().is_none());
    }

    #[test]
    fn test_auth_header() {
        let credentials = Credentials::new("test_id", "test_secret");
        let mut auth = DeribitAuth::new(&credentials).unwrap();

        // No token - should error
        assert!(auth.auth_header().is_err());

        // With token
        auth.store_tokens(
            "my_jwt_token".to_string(),
            "my_refresh_token".to_string(),
            900,
        );

        let header = auth.auth_header().unwrap();
        assert_eq!(header, "Bearer my_jwt_token");
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        // Should be 16 chars
        assert_eq!(nonce1.len(), 16);
        assert_eq!(nonce2.len(), 16);

        // Should be different
        assert_ne!(nonce1, nonce2);

        // Should be alphanumeric
        assert!(nonce1.chars().all(|c| c.is_alphanumeric()));
    }
}
