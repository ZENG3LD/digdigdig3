//! Sentinel Hub authentication
//!
//! Authentication type: OAuth2 Client Credentials
//!
//! Sentinel Hub uses OAuth2 client credentials flow.
//! First, obtain an access token from /oauth/token endpoint.
//! Then use the token as Bearer token in Authorization header.
//!
//! Environment variables:
//! - SENTINEL_HUB_CLIENT_ID
//! - SENTINEL_HUB_CLIENT_SECRET

use std::collections::HashMap;

/// Sentinel Hub authentication credentials
#[derive(Clone)]
pub struct SentinelHubAuth {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
}

impl SentinelHubAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variables:
    /// - `SENTINEL_HUB_CLIENT_ID`
    /// - `SENTINEL_HUB_CLIENT_SECRET`
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("SENTINEL_HUB_CLIENT_ID").ok(),
            client_secret: std::env::var("SENTINEL_HUB_CLIENT_SECRET").ok(),
            access_token: None,
        }
    }

    /// Create auth with explicit credentials
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: Some(client_id.into()),
            client_secret: Some(client_secret.into()),
            access_token: None,
        }
    }

    /// Set access token (after authentication)
    pub fn set_access_token(&mut self, token: impl Into<String>) {
        self.access_token = Some(token.into());
    }

    /// Check if credentials are configured
    pub fn has_credentials(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }

    /// Check if access token is available
    pub fn has_token(&self) -> bool {
        self.access_token.is_some()
    }

    /// Get client ID
    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    /// Get client secret
    pub fn client_secret(&self) -> Option<&str> {
        self.client_secret.as_deref()
    }

    /// Get access token
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    /// Add authentication to request headers
    ///
    /// Adds Bearer token to Authorization header:
    /// `Authorization: Bearer <token>`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(token) = &self.access_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
    }
}

impl Default for SentinelHubAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
