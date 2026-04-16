//! Tinkoff Invest API authentication
//!
//! Authentication type: Bearer token
//!
//! All Tinkoff Invest API endpoints require authentication.
//! The API uses simple Bearer token authentication without HMAC signing.
//!
//! Token format: `Authorization: Bearer t.xxx`
//!
//! ## Token Types
//! - Readonly: Read-only access to portfolio and market data
//! - Full-access: Complete API access including trading
//! - Account-specific: Restrict access to single trading account
//! - Sandbox: Testing environment access
//!
//! ## Token Generation
//! Generate at: https://www.tinkoff.ru/invest/settings/

use std::collections::HashMap;

/// Tinkoff Invest authentication credentials
#[derive(Clone)]
pub struct TinkoffAuth {
    /// API token (starts with "t.")
    pub token: String,
}

impl TinkoffAuth {
    /// Create new auth from environment variable
    ///
    /// Reads from `TINKOFF_TOKEN` environment variable.
    pub fn from_env() -> Self {
        let token = std::env::var("TINKOFF_TOKEN")
            .unwrap_or_default();
        Self { token }
    }

    /// Create auth with explicit token
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    /// Check if token is present
    pub fn has_token(&self) -> bool {
        !self.token.is_empty()
    }

    /// Add authentication headers to request
    ///
    /// Adds `Authorization: Bearer {token}` header.
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if self.has_token() {
            headers.insert(
                "Authorization".to_string(),
                format!("Bearer {}", self.token),
            );
        }
    }

    /// Add optional app name header for instrumentation
    ///
    /// Format: `x-app-name: github-username.repo-name`
    /// Contact al.a.volkov@tinkoff.ru for dedicated app registration.
    pub fn add_app_name(&self, headers: &mut HashMap<String, String>, app_name: &str) {
        headers.insert("x-app-name".to_string(), app_name.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_creation() {
        let auth = TinkoffAuth::new("t.test_token_123");
        assert!(auth.has_token());
        assert_eq!(auth.token, "t.test_token_123");
    }

    #[test]
    fn test_empty_auth() {
        let auth = TinkoffAuth::new("");
        assert!(!auth.has_token());
    }

    #[test]
    fn test_sign_headers() {
        let auth = TinkoffAuth::new("t.my_token");
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer t.my_token".to_string())
        );
    }

    #[test]
    fn test_app_name() {
        let auth = TinkoffAuth::new("t.token");
        let mut headers = HashMap::new();
        auth.add_app_name(&mut headers, "myuser.myrepo");

        assert_eq!(
            headers.get("x-app-name"),
            Some(&"myuser.myrepo".to_string())
        );
    }
}
