//! # Fyers Authentication
//!
//! OAuth 2.0 flow with SHA-256 hashing for Fyers API v3.
//!
//! ## Authentication Flow
//!
//! 1. Generate authorization URL with client_id, redirect_uri, response_type, state
//! 2. User logs in via browser (username, password, TOTP)
//! 3. Receive auth_code via redirect callback
//! 4. Calculate appIdHash: SHA256(app_id + ":" + app_secret)
//! 5. Exchange auth_code for access_token via POST /api/v3/validate-authcode
//! 6. Use access_token in all API requests
//!
//! ## Authorization Header Format
//!
//! ```text
//! Authorization: APPID:ACCESS_TOKEN
//! ```
//!
//! ## Token Lifetime
//!
//! - Access tokens expire after trading day / 24 hours
//! - NO refresh token mechanism
//! - Must re-authenticate when token expires

use std::collections::HashMap;

use crate::core::{sha256, encode_hex_lower};

/// Fyers authentication credentials
#[derive(Clone, Debug)]
pub struct FyersAuth {
    pub app_id: String,
    pub app_secret: String,
    pub access_token: Option<String>,
}

impl FyersAuth {
    /// Create new auth from environment variables
    pub fn from_env() -> Self {
        Self {
            app_id: std::env::var("FYERS_APP_ID").unwrap_or_default(),
            app_secret: std::env::var("FYERS_APP_SECRET").unwrap_or_default(),
            access_token: std::env::var("FYERS_ACCESS_TOKEN").ok(),
        }
    }

    /// Create auth with explicit credentials
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
            access_token: None,
        }
    }

    /// Create auth with existing access token
    pub fn with_token(
        app_id: impl Into<String>,
        app_secret: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
            access_token: Some(access_token.into()),
        }
    }

    /// Generate appIdHash for token exchange
    ///
    /// Formula: `SHA256(app_id + ":" + app_secret)`
    ///
    /// ## Example
    /// ```ignore
    /// let auth = FyersAuth::new("ABC123XYZ-100", "ABCDEFGH1234567890");
    /// let hash = auth.generate_app_id_hash();
    /// ```
    pub fn generate_app_id_hash(&self) -> String {
        let message = format!("{}:{}", self.app_id, self.app_secret);
        let hash = sha256(message.as_bytes());
        encode_hex_lower(&hash)
    }

    /// Generate authorization URL for user login
    ///
    /// User must navigate to this URL in browser to complete OAuth flow.
    ///
    /// ## Parameters
    /// - `redirect_uri`: Your registered redirect URI
    /// - `state`: Random string for session management (optional)
    ///
    /// ## Example
    /// ```ignore
    /// let auth = FyersAuth::new("ABC123XYZ-100", "secret");
    /// let url = auth.get_authorization_url("https://yourapp.com/callback", Some("state123"));
    /// println!("Navigate to: {}", url);
    /// ```
    pub fn get_authorization_url(&self, redirect_uri: &str, state: Option<&str>) -> String {
        let base = "https://api.fyers.in/api/v3/generate-authcode";
        let state_param = state.unwrap_or("fyers_auth");

        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&state={}",
            base,
            urlencoding::encode(&self.app_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state_param)
        )
    }

    /// Prepare token exchange request body
    ///
    /// ## Parameters
    /// - `auth_code`: Authorization code from redirect callback
    ///
    /// ## Returns
    /// HashMap ready for JSON serialization
    pub fn prepare_token_request(&self, auth_code: &str) -> HashMap<String, String> {
        let mut body = HashMap::new();
        body.insert("grant_type".to_string(), "authorization_code".to_string());
        body.insert("appIdHash".to_string(), self.generate_app_id_hash());
        body.insert("code".to_string(), auth_code.to_string());
        body
    }

    /// Add authentication headers to request
    ///
    /// Format: `Authorization: APPID:ACCESS_TOKEN`
    ///
    /// ## Example
    /// ```ignore
    /// let auth = FyersAuth::with_token("ABC123XYZ-100", "secret", "eyJ0eXAi...");
    /// let mut headers = HashMap::new();
    /// auth.sign_headers(&mut headers);
    /// ```
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(token) = &self.access_token {
            headers.insert(
                "Authorization".to_string(),
                format!("{}:{}", self.app_id, token),
            );
        }
    }

    /// Check if access token is available
    pub fn has_token(&self) -> bool {
        self.access_token.is_some()
    }

    /// Set access token after successful authentication
    pub fn set_access_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    /// Get full authorization header value
    pub fn get_auth_header_value(&self) -> Option<String> {
        self.access_token
            .as_ref()
            .map(|token| format!("{}:{}", self.app_id, token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_app_id_hash() {
        let auth = FyersAuth::new("ABC123XYZ-100", "ABCDEFGH1234567890");
        let hash = auth.generate_app_id_hash();

        // Should be a 64-character hex string (SHA-256)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Test deterministic: same input = same output
        let hash2 = auth.generate_app_id_hash();
        assert_eq!(hash, hash2);

        // Test different credentials = different hash
        let auth2 = FyersAuth::new("DIFFERENT", "SECRET");
        let hash3 = auth2.generate_app_id_hash();
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_get_authorization_url() {
        let auth = FyersAuth::new("ABC123XYZ-100", "secret");

        let url = auth.get_authorization_url("https://example.com/callback", Some("state123"));
        assert!(url.contains("client_id=ABC123XYZ-100"));
        assert!(url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=state123"));

        // Test with default state
        let url2 = auth.get_authorization_url("https://example.com/callback", None);
        assert!(url2.contains("state=fyers_auth"));
    }

    #[test]
    fn test_prepare_token_request() {
        let auth = FyersAuth::new("ABC123XYZ-100", "ABCDEFGH1234567890");
        let body = auth.prepare_token_request("auth_code_xyz");

        assert_eq!(body.get("grant_type"), Some(&"authorization_code".to_string()));
        assert_eq!(body.get("code"), Some(&"auth_code_xyz".to_string()));
        assert!(body.contains_key("appIdHash"));
        assert_eq!(body.get("appIdHash").unwrap().len(), 64); // SHA-256 hex
    }

    #[test]
    fn test_sign_headers() {
        let auth = FyersAuth::with_token("ABC123XYZ-100", "secret", "eyJ0eXAiOiJKV1Qi");
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"ABC123XYZ-100:eyJ0eXAiOiJKV1Qi".to_string())
        );
    }

    #[test]
    fn test_has_token() {
        let auth_without = FyersAuth::new("app_id", "secret");
        assert!(!auth_without.has_token());

        let auth_with = FyersAuth::with_token("app_id", "secret", "token");
        assert!(auth_with.has_token());
    }

    #[test]
    fn test_get_auth_header_value() {
        let auth = FyersAuth::with_token("ABC123", "secret", "TOKEN123");
        assert_eq!(
            auth.get_auth_header_value(),
            Some("ABC123:TOKEN123".to_string())
        );

        let auth_no_token = FyersAuth::new("ABC123", "secret");
        assert_eq!(auth_no_token.get_auth_header_value(), None);
    }
}
