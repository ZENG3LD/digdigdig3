//! # Interactive Brokers Authentication
//!
//! IB uses Gateway-based session authentication for individual accounts.
//! OAuth 2.0 is available for enterprise clients but requires additional setup.
//!
//! ## Authentication Methods
//! 1. **Client Portal Gateway** (individual accounts) - default
//!    - Requires manual browser login (cannot be automated)
//!    - Session maintained via periodic tickle (keep-alive)
//!    - SSL verification disabled for localhost
//!
//! 2. **OAuth 2.0** (enterprise clients) - not yet implemented
//!    - Private Key JWT authentication
//!    - Requires RSA key pair
//!    - Fully automated

use std::collections::HashMap;

/// Interactive Brokers authentication
///
/// For Gateway authentication, credentials are not stored.
/// Authentication happens via browser login to the Gateway.
#[derive(Clone, Debug)]
pub struct IBAuth {
    /// Account ID (required for trading operations)
    pub account_id: String,
    /// OAuth 2.0 access token (if using OAuth, not implemented yet)
    pub _access_token: Option<String>,
}

impl IBAuth {
    /// Create new authentication with account ID
    ///
    /// For Gateway authentication, this just stores the account ID.
    /// Actual authentication happens via browser login to Gateway.
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            _access_token: None,
        }
    }

    /// Create authentication from environment variables
    ///
    /// Reads IB_ACCOUNT_ID from environment.
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        let account_id = std::env::var("IB_ACCOUNT_ID").unwrap_or_default();
        Self::new(account_id)
    }

    /// Add authentication headers to request (for OAuth)
    ///
    /// For Gateway authentication, this is a no-op since authentication
    /// is handled via session cookies.
    #[allow(dead_code)]
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(token) = &self._access_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
        // For Gateway auth, no headers needed (uses session cookies)
    }

    /// Get account ID
    pub fn account_id(&self) -> &str {
        &self.account_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_creation() {
        let auth = IBAuth::new("DU12345");
        assert_eq!(auth.account_id(), "DU12345");
        assert!(auth._access_token.is_none());
    }

    #[test]
    fn test_sign_headers_without_token() {
        let auth = IBAuth::new("DU12345");
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);
        assert!(headers.is_empty()); // No headers for Gateway auth
    }

    #[test]
    fn test_sign_headers_with_token() {
        let mut auth = IBAuth::new("DU12345");
        auth._access_token = Some("test_token".to_string());
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);
        assert_eq!(headers.get("Authorization"), Some(&"Bearer test_token".to_string()));
    }
}
