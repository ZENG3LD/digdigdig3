//! # JQuants Authentication
//!
//! Authentication type: Two-token system
//! - Refresh Token (long-lived, 7 days)
//! - ID Token (short-lived, 24 hours)
//!
//! ## Flow
//! 1. Get refresh token from dashboard or /token/auth_user endpoint
//! 2. Use refresh token to get ID token (/token/auth_refresh)
//! 3. Use ID token in Authorization header for all API calls
//! 4. Refresh ID token when it expires (24 hours)

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication credentials for JQuants
#[derive(Clone)]
pub struct JQuantsAuth {
    /// Refresh token (long-lived, 7 days)
    pub refresh_token: String,
    /// Cached ID token (short-lived, 24 hours)
    id_token: Option<String>,
    /// ID token expiry time (Unix timestamp in seconds)
    id_token_expiry: Option<u64>,
}

impl JQuantsAuth {
    /// Create new auth from refresh token
    ///
    /// The refresh token should be obtained from:
    /// - JQuants dashboard: https://jpx-jquants.com/en
    /// - OR /token/auth_user endpoint (email + password)
    pub fn new(refresh_token: impl Into<String>) -> Self {
        Self {
            refresh_token: refresh_token.into(),
            id_token: None,
            id_token_expiry: None,
        }
    }

    /// Create auth from environment variable
    ///
    /// Expects: JQUANTS_REFRESH_TOKEN
    pub fn from_env() -> Self {
        let refresh_token = std::env::var("JQUANTS_REFRESH_TOKEN")
            .unwrap_or_default();
        Self::new(refresh_token)
    }

    /// Get the refresh token
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    /// Check if ID token is cached and valid
    pub fn has_valid_id_token(&self) -> bool {
        if let (Some(_token), Some(expiry)) = (&self.id_token, self.id_token_expiry) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time is before UNIX epoch")
                .as_secs();
            now < expiry
        } else {
            false
        }
    }

    /// Get cached ID token (if valid)
    pub fn get_cached_id_token(&self) -> Option<&str> {
        if self.has_valid_id_token() {
            self.id_token.as_deref()
        } else {
            None
        }
    }

    /// Cache new ID token
    ///
    /// Sets expiry to 23 hours from now (1 hour safety margin)
    pub fn cache_id_token(&mut self, id_token: String) {
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs() + (23 * 3600); // 23 hours

        self.id_token = Some(id_token);
        self.id_token_expiry = Some(expiry);
    }

    /// Clear cached ID token (force refresh on next request)
    pub fn clear_id_token(&mut self) {
        self.id_token = None;
        self.id_token_expiry = None;
    }

    /// Add authentication headers to request
    ///
    /// Uses cached ID token if valid, otherwise caller must refresh it first.
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>, id_token: &str) {
        headers.insert("Authorization".to_string(), format!("Bearer {}", id_token));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    /// Get headers for authenticated request (if ID token is cached)
    pub fn get_auth_headers(&self) -> Option<HashMap<String, String>> {
        if let Some(id_token) = self.get_cached_id_token() {
            let mut headers = HashMap::new();
            self.sign_headers(&mut headers, id_token);
            Some(headers)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_auth() {
        let auth = JQuantsAuth::new("test_refresh_token");
        assert_eq!(auth.refresh_token(), "test_refresh_token");
        assert!(!auth.has_valid_id_token());
    }

    #[test]
    fn test_cache_id_token() {
        let mut auth = JQuantsAuth::new("refresh");
        auth.cache_id_token("id_token_123".to_string());

        assert!(auth.has_valid_id_token());
        assert_eq!(auth.get_cached_id_token(), Some("id_token_123"));
    }

    #[test]
    fn test_clear_id_token() {
        let mut auth = JQuantsAuth::new("refresh");
        auth.cache_id_token("id_token_123".to_string());
        auth.clear_id_token();

        assert!(!auth.has_valid_id_token());
        assert_eq!(auth.get_cached_id_token(), None);
    }

    #[test]
    fn test_sign_headers() {
        let auth = JQuantsAuth::new("refresh");
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers, "test_id_token");

        assert_eq!(headers.get("Authorization"), Some(&"Bearer test_id_token".to_string()));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
}
