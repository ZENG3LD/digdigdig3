//! Zerodha Kite Connect authentication
//!
//! Authentication type: Custom OAuth-like flow (NOT standard OAuth 2.0)
//!
//! ## Authentication Flow
//!
//! 1. User navigates to login URL: `https://kite.zerodha.com/connect/login?v=3&api_key={api_key}`
//! 2. After successful login, receives `request_token` via redirect callback
//! 3. Calculate checksum: `SHA256(api_key + request_token + api_secret)`
//! 4. Exchange for `access_token` via POST to `/session/token`
//! 5. Use `access_token` in all subsequent requests
//!
//! ## Authorization Header Format
//!
//! ```text
//! Authorization: token {api_key}:{access_token}
//! ```
//!
//! **NOT** `Bearer {token}` - Zerodha uses custom "token" scheme!
//!
//! ## Token Lifetime
//!
//! - `access_token` expires daily at 6:00 AM IST (regulatory requirement)
//! - NO refresh token mechanism available
//! - Must re-authenticate daily

use std::collections::HashMap;
use sha2::{Sha256, Digest};

/// Zerodha authentication credentials
#[derive(Clone)]
pub struct ZerodhaAuth {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: Option<String>,
}

impl ZerodhaAuth {
    /// Create new auth from environment variables
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ZERODHA_API_KEY")
                .unwrap_or_default(),
            api_secret: std::env::var("ZERODHA_API_SECRET")
                .unwrap_or_default(),
            access_token: std::env::var("ZERODHA_ACCESS_TOKEN").ok(),
        }
    }

    /// Create auth with explicit credentials
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            access_token: None,
        }
    }

    /// Create auth with existing access token
    pub fn with_token(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            access_token: Some(access_token.into()),
        }
    }

    /// Generate SHA-256 checksum for token exchange
    ///
    /// Formula: `SHA256(api_key + request_token + api_secret)`
    ///
    /// ## Example
    /// ```ignore
    /// let auth = ZerodhaAuth::new("my_api_key", "my_api_secret");
    /// let checksum = auth.generate_checksum("request_token_abc123");
    /// ```
    pub fn generate_checksum(&self, request_token: &str) -> String {
        let message = format!("{}{}{}", self.api_key, request_token, self.api_secret);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Add authentication headers to request
    ///
    /// Format: `Authorization: token {api_key}:{access_token}`
    ///
    /// **Critical**: Zerodha uses "token" scheme, NOT "Bearer"!
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(token) = &self.access_token {
            headers.insert(
                "Authorization".to_string(),
                format!("token {}:{}", self.api_key, token),
            );
        }
    }

    /// Check if access token is available
    pub fn has_token(&self) -> bool {
        self.access_token.is_some()
    }

    /// Get login URL for user authentication
    ///
    /// User must navigate to this URL to complete OAuth-like flow.
    ///
    /// ## Parameters
    /// - `redirect_params`: Optional custom data to receive back in callback (URL-encoded)
    ///
    /// ## Example
    /// ```ignore
    /// let auth = ZerodhaAuth::new("my_api_key", "my_secret");
    /// let url = auth.get_login_url(None);
    /// println!("Navigate to: {}", url);
    /// ```
    pub fn get_login_url(&self, redirect_params: Option<&str>) -> String {
        let base = "https://kite.zerodha.com/connect/login?v=3";
        if let Some(params) = redirect_params {
            format!("{}&api_key={}&redirect_params={}", base, self.api_key, params)
        } else {
            format!("{}&api_key={}", base, self.api_key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_checksum() {
        // Test with known values (from Zerodha documentation examples)
        let auth = ZerodhaAuth::new("my_api_key", "my_api_secret");
        let checksum = auth.generate_checksum("request_token");

        // Should be a 64-character hex string (SHA-256)
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));

        // Test deterministic: same input = same output
        let checksum2 = auth.generate_checksum("request_token");
        assert_eq!(checksum, checksum2);

        // Test different input = different output
        let checksum3 = auth.generate_checksum("different_token");
        assert_ne!(checksum, checksum3);
    }

    #[test]
    fn test_sign_headers() {
        let auth = ZerodhaAuth::with_token("my_api_key", "my_secret", "my_access_token");
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"token my_api_key:my_access_token".to_string())
        );
    }

    #[test]
    fn test_get_login_url() {
        let auth = ZerodhaAuth::new("test_key", "test_secret");

        let url = auth.get_login_url(None);
        assert!(url.contains("api_key=test_key"));
        assert!(url.contains("v=3"));

        let url_with_params = auth.get_login_url(Some("custom_data"));
        assert!(url_with_params.contains("redirect_params=custom_data"));
    }

    #[test]
    fn test_has_token() {
        let auth_without_token = ZerodhaAuth::new("key", "secret");
        assert!(!auth_without_token.has_token());

        let auth_with_token = ZerodhaAuth::with_token("key", "secret", "token");
        assert!(auth_with_token.has_token());
    }
}
