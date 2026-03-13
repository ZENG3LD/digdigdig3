//! OpenSky Network authentication
//!
//! Authentication type: OAuth2 client credentials flow
//!
//! OpenSky supports both anonymous and OAuth2-authenticated access:
//! - Anonymous: 10 requests per 10 seconds, limited historical data
//! - Authenticated: 4000 credits per day (credit cost varies by endpoint)
//!
//! OAuth2 token endpoint: POST https://opensky-network.org/api/oauth/token
//! Grant type: client_credentials
//! Token type: Bearer

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// OAuth2 token response from OpenSky token endpoint
#[derive(Debug, Deserialize)]
struct OAuth2TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

/// Cached OAuth2 access token with expiry tracking
#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    /// UNIX timestamp (seconds) when token expires
    expires_at: u64,
}

impl CachedToken {
    /// Returns true if token is still valid (with 30-second safety margin)
    fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expires_at > now + 30
    }
}

/// OpenSky Network OAuth2 authentication
///
/// Manages OAuth2 client credentials flow including token acquisition and refresh.
/// Falls back to anonymous access when no credentials are provided.
#[derive(Clone)]
pub struct OpenskyAuth {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    /// Cached OAuth2 access token (None = anonymous or not yet fetched)
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

impl OpenskyAuth {
    /// Token endpoint URL
    const TOKEN_URL: &'static str = "https://opensky-network.org/api/oauth/token";

    /// Create new auth from environment variables
    ///
    /// Expects environment variables: `OPENSKY_CLIENT_ID`, `OPENSKY_CLIENT_SECRET`
    /// If not present, falls back to anonymous access.
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("OPENSKY_CLIENT_ID").ok(),
            client_secret: std::env::var("OPENSKY_CLIENT_SECRET").ok(),
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Create auth with explicit client credentials
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: Some(client_id.into()),
            client_secret: Some(client_secret.into()),
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Create anonymous auth (no credentials)
    pub fn anonymous() -> Self {
        Self {
            client_id: None,
            client_secret: None,
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if authentication credentials are configured
    pub fn is_authenticated(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }

    /// Get client_id (for debugging/logging)
    pub fn get_client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    /// Fetch a new OAuth2 access token from the token endpoint
    ///
    /// Uses the client credentials grant type:
    /// `POST /api/oauth/token` with `grant_type=client_credentials`
    async fn fetch_token(
        &self,
        client: &reqwest::Client,
    ) -> Result<CachedToken, String> {
        let (client_id, client_secret) = match (&self.client_id, &self.client_secret) {
            (Some(id), Some(secret)) => (id.as_str(), secret.as_str()),
            _ => return Err("No client credentials configured".to_string()),
        };

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];

        let response = client
            .post(Self::TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Token endpoint returned HTTP {}: {}", status, body));
        }

        let token_response: OAuth2TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        // Validate token type
        if !token_response.token_type.eq_ignore_ascii_case("bearer") {
            return Err(format!(
                "Unexpected token_type '{}', expected 'bearer'",
                token_response.token_type
            ));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(CachedToken {
            access_token: token_response.access_token,
            expires_at: now + token_response.expires_in,
        })
    }

    /// Get a valid access token, fetching/refreshing if necessary
    ///
    /// Returns None for anonymous access.
    /// Returns Some(token) if authenticated and token is obtained successfully.
    /// Returns Err if authenticated but token acquisition failed.
    pub async fn get_token(
        &self,
        client: &reqwest::Client,
    ) -> Result<Option<String>, String> {
        if !self.is_authenticated() {
            return Ok(None);
        }

        // Check cached token under read lock
        {
            let cached = self.cached_token.read().await;
            if let Some(ref token) = *cached {
                if token.is_valid() {
                    return Ok(Some(token.access_token.clone()));
                }
            }
        }

        // Token missing or expired — fetch new one under write lock
        let mut cached = self.cached_token.write().await;

        // Double-check after acquiring write lock (another task may have refreshed)
        if let Some(ref token) = *cached {
            if token.is_valid() {
                return Ok(Some(token.access_token.clone()));
            }
        }

        let new_token = self.fetch_token(client).await?;
        let access_token = new_token.access_token.clone();
        *cached = Some(new_token);
        Ok(Some(access_token))
    }

    /// Add authentication to request headers
    ///
    /// For anonymous access: no headers added.
    /// For authenticated access: adds `Authorization: Bearer {token}`.
    ///
    /// NOTE: This method requires a pre-fetched token. Call `apply_auth_headers`
    /// instead when you have an async context and a `reqwest::Client` available.
    pub fn sign_headers_with_token(&self, headers: &mut HeaderMap, token: &str) {
        let auth_value = format!("Bearer {}", token);
        if let Ok(header_value) = HeaderValue::from_str(&auth_value) {
            headers.insert(AUTHORIZATION, header_value);
        }
    }

    /// Apply authentication headers asynchronously
    ///
    /// Fetches/refreshes the OAuth2 token if needed, then applies it to headers.
    /// For anonymous access, headers are left unchanged.
    pub async fn apply_auth_headers(
        &self,
        headers: &mut HeaderMap,
        client: &reqwest::Client,
    ) -> Result<(), String> {
        if let Some(token) = self.get_token(client).await? {
            self.sign_headers_with_token(headers, &token);
        }
        Ok(())
    }
}

impl Default for OpenskyAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
