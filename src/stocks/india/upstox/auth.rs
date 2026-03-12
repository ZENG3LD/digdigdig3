//! # Upstox Authentication
//!
//! Authentication type: OAuth 2.0 (Authorization Code flow)
//!
//! ## Authentication Flow
//!
//! 1. Redirect user to authorization URL with client_id and redirect_uri
//! 2. User logs in and authorizes the application
//! 3. Receive authorization code via redirect callback
//! 4. Exchange authorization code for access token
//! 5. Use access token in all subsequent API requests
//!
//! ## Authorization Header Format
//!
//! ```text
//! Authorization: Bearer {access_token}
//! ```
//!
//! ## Token Lifetime
//!
//! - `access_token` expires at 3:30 AM IST next day
//! - NO refresh token mechanism available
//! - Must re-authenticate daily
//! - Extended token (1 year) available for read-only operations upon request
//!
//! ## API Availability
//!
//! APIs accessible only 5:30 AM to 12:00 AM IST
//! Returns error UDAPI100074 outside this window

use std::collections::HashMap;

use crate::core::{Credentials, ExchangeResult, ExchangeError};

/// Upstox authentication credentials
#[derive(Clone)]
pub struct UpstoxAuth {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: Option<String>,
    pub redirect_uri: Option<String>,
}

impl UpstoxAuth {
    /// Create new auth from credentials
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            access_token: None,
            redirect_uri: None,
        })
    }

    /// Create auth from environment variables
    pub fn from_env() -> ExchangeResult<Self> {
        let api_key = std::env::var("UPSTOX_API_KEY")
            .map_err(|_| ExchangeError::Auth("UPSTOX_API_KEY not set".to_string()))?;
        let api_secret = std::env::var("UPSTOX_API_SECRET")
            .map_err(|_| ExchangeError::Auth("UPSTOX_API_SECRET not set".to_string()))?;
        let access_token = std::env::var("UPSTOX_ACCESS_TOKEN").ok();
        let redirect_uri = std::env::var("UPSTOX_REDIRECT_URI").ok();

        Ok(Self {
            api_key,
            api_secret,
            access_token,
            redirect_uri,
        })
    }

    /// Create auth with explicit credentials and token
    pub fn _with_token(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            access_token: Some(access_token.into()),
            redirect_uri: None,
        }
    }

    /// Set redirect URI for OAuth flow
    pub fn _with_redirect_uri(mut self, uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(uri.into());
        self
    }

    /// Set access token
    pub fn set_access_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    /// Get authorization URL for user to visit
    ///
    /// User must navigate to this URL and authorize the application.
    /// After authorization, they will be redirected to redirect_uri with code.
    ///
    /// ## Parameters
    /// - `state`: Optional state parameter for CSRF protection
    ///
    /// ## Example
    /// ```ignore
    /// let auth = UpstoxAuth::new(&credentials)?
    ///     .with_redirect_uri("https://myapp.com/callback");
    /// let url = auth.get_authorization_url(Some("random_state_123"));
    /// println!("Visit: {}", url);
    /// ```
    pub fn get_authorization_url(&self, state: Option<&str>) -> String {
        let redirect = self.redirect_uri.as_deref().unwrap_or("http://localhost");
        let mut url = format!(
            "https://api.upstox.com/v2/login/authorization/dialog?client_id={}&redirect_uri={}&response_type=code",
            self.api_key,
            urlencoding::encode(redirect)
        );

        if let Some(s) = state {
            url.push_str(&format!("&state={}", urlencoding::encode(s)));
        }

        url
    }

    /// Build request body for token exchange
    ///
    /// Call this after receiving authorization code from redirect.
    ///
    /// ## Example
    /// ```ignore
    /// let body = auth.build_token_exchange_body("AUTH_CODE_HERE");
    /// // POST to /v2/login/authorization/token with this body
    /// ```
    pub fn build_token_exchange_body(&self, code: &str) -> HashMap<String, String> {
        let mut body = HashMap::new();
        body.insert("code".to_string(), code.to_string());
        body.insert("client_id".to_string(), self.api_key.clone());
        body.insert("client_secret".to_string(), self.api_secret.clone());
        body.insert(
            "redirect_uri".to_string(),
            self.redirect_uri
                .as_deref()
                .unwrap_or("http://localhost")
                .to_string(),
        );
        body.insert("grant_type".to_string(), "authorization_code".to_string());
        body
    }

    /// Add authentication headers to request
    ///
    /// Format: `Authorization: Bearer {access_token}`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) -> ExchangeResult<()> {
        if let Some(token) = &self.access_token {
            headers.insert(
                "Authorization".to_string(),
                format!("Bearer {}", token),
            );
            headers.insert("Accept".to_string(), "application/json".to_string());
            Ok(())
        } else {
            Err(ExchangeError::Auth(
                "No access token available. Please authenticate first.".to_string(),
            ))
        }
    }

    /// Check if access token is available
    pub fn _has_token(&self) -> bool {
        self.access_token.is_some()
    }

    /// Get API key (for public endpoints or OAuth flow)
    pub fn _api_key(&self) -> &str {
        &self.api_key
    }

    /// Get API secret (for token exchange)
    pub fn _api_secret(&self) -> &str {
        &self.api_secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_token() {
        let auth = UpstoxAuth::_with_token("test_key", "test_secret", "test_token");
        assert_eq!(auth.api_key, "test_key");
        assert_eq!(auth.api_secret, "test_secret");
        assert_eq!(auth.access_token, Some("test_token".to_string()));
        assert!(auth._has_token());
    }

    #[test]
    fn test_sign_headers() {
        let auth = UpstoxAuth::with_token("key", "secret", "my_access_token");
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers).unwrap();

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer my_access_token".to_string())
        );
        assert_eq!(
            headers.get("Accept"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_sign_headers_no_token() {
        let credentials = Credentials::new("key", "secret");
        let auth = UpstoxAuth::new(&credentials).unwrap();
        let mut headers = HashMap::new();

        let result = auth.sign_headers(&mut headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_authorization_url() {
        let auth = UpstoxAuth::with_token("test_key", "secret", "token")
            .with_redirect_uri("https://myapp.com/callback");

        let url = auth.get_authorization_url(Some("state123"));

        assert!(url.contains("client_id=test_key"));
        assert!(url.contains("redirect_uri=https%3A%2F%2Fmyapp.com%2Fcallback"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=state123"));
    }

    #[test]
    fn test_build_token_exchange_body() {
        let auth = UpstoxAuth::with_token("key", "secret", "token")
            .with_redirect_uri("https://app.com/callback");

        let body = auth.build_token_exchange_body("AUTH_CODE_123");

        assert_eq!(body.get("code"), Some(&"AUTH_CODE_123".to_string()));
        assert_eq!(body.get("client_id"), Some(&"key".to_string()));
        assert_eq!(body.get("client_secret"), Some(&"secret".to_string()));
        assert_eq!(
            body.get("redirect_uri"),
            Some(&"https://app.com/callback".to_string())
        );
        assert_eq!(
            body.get("grant_type"),
            Some(&"authorization_code".to_string())
        );
    }
}
