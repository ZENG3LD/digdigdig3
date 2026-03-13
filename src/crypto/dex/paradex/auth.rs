//! # Paradex Authentication
//!
//! JWT-based authentication for Paradex API.
//!
//! ## Algorithm
//!
//! Paradex uses JWT tokens for authorization:
//! 1. JWT token is obtained via POST /v1/auth with a StarkNet signature
//! 2. Token expires after 5 minutes; recommended to refresh every 3 minutes
//! 3. For WebSocket: authenticate once, no repeated authorization required
//!
//! ## Headers for private endpoints
//!
//! - `Authorization: Bearer {jwt_token}`
//!
//! ## Token Expiry
//!
//! Paradex JWTs have a fixed 5-minute (300-second) expiry.
//! This implementation tracks `issued_at` and automatically detects expiry
//! with a 30-second safety margin (effective 270-second window).
//!
//! ## StarkNet Signing Note
//!
//! Full JWT generation requires StarkNet cryptography (sign with private key).
//! This implementation stores and refreshes pre-obtained JWT tokens.
//! For complete StarkNet signing, integrate the `starknet` crate.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// JWT lifetime in seconds (Paradex tokens expire after 5 minutes)
const JWT_LIFETIME_SECS: u64 = 300;

/// Safety margin in seconds — refresh token before it actually expires
const JWT_SAFETY_MARGIN_SECS: u64 = 30;

/// Cached JWT token with expiry tracking
#[derive(Debug, Clone)]
struct JwtToken {
    /// The raw JWT token string
    token: String,
    /// UNIX timestamp (seconds) when this token was issued
    issued_at: u64,
    /// UNIX timestamp (seconds) when this token expires
    expires_at: u64,
}

impl JwtToken {
    /// Create a new token issued right now with the standard 5-minute lifetime
    fn new_now(token: String) -> Self {
        let now = current_timestamp_secs();
        Self {
            token,
            issued_at: now,
            expires_at: now + JWT_LIFETIME_SECS,
        }
    }

    /// Create a token with a custom expiry (e.g., parsed from JWT claims)
    fn new_with_expiry(token: String, expires_at: u64) -> Self {
        Self {
            token,
            issued_at: current_timestamp_secs(),
            expires_at,
        }
    }

    /// Returns true if the token is still valid (accounting for safety margin)
    fn is_valid(&self) -> bool {
        let now = current_timestamp_secs();
        self.expires_at > now + JWT_SAFETY_MARGIN_SECS
    }

    /// Returns true if the token has completely expired (no safety margin)
    fn is_expired(&self) -> bool {
        let now = current_timestamp_secs();
        now >= self.expires_at
    }

    /// Seconds until expiry (0 if already expired)
    fn secs_until_expiry(&self) -> u64 {
        let now = current_timestamp_secs();
        self.expires_at.saturating_sub(now)
    }

    /// Age of this token in seconds
    fn age_secs(&self) -> u64 {
        let now = current_timestamp_secs();
        now.saturating_sub(self.issued_at)
    }
}

/// Returns current UNIX timestamp in seconds
fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Returns current UNIX timestamp in milliseconds
fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUTH
// ═══════════════════════════════════════════════════════════════════════════════

/// Paradex authentication (JWT-based with expiry tracking)
///
/// Tracks the JWT token lifetime and provides infrastructure for automatic refresh.
/// The token expires every 5 minutes; this handler detects expiry with a
/// 30-second safety margin and signals that refresh is needed.
#[derive(Clone)]
pub struct ParadexAuth {
    /// Cached JWT token with expiry tracking
    jwt_token: Arc<RwLock<Option<JwtToken>>>,

    /// StarkNet account address (needed for JWT generation via signing)
    #[allow(dead_code)]
    account_address: Option<String>,

    /// Time offset: server_time - local_time (milliseconds)
    time_offset_ms: Arc<RwLock<i64>>,
}

impl ParadexAuth {
    /// Create a new auth handler
    ///
    /// IMPORTANT: Paradex requires a JWT token for private endpoints.
    ///
    /// # Usage variants:
    ///
    /// 1. **JWT token passed via credentials.api_key** (pre-obtained token):
    ///    ```ignore
    ///    let creds = Credentials::new("jwt_token_here", "");
    ///    let auth = ParadexAuth::new(&creds)?;
    ///    ```
    ///
    /// 2. **StarkNet account (future full implementation)**:
    ///    Pass account_address in api_secret; call `refresh_token()` to obtain JWT.
    ///    Requires starknet-rs integration (TODO).
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        // api_key = JWT token (if pre-obtained)
        let jwt_token = if !credentials.api_key.is_empty() {
            // Token is provided; treat as freshly issued (we don't know the real expiry)
            Some(JwtToken::new_now(credentials.api_key.clone()))
        } else {
            None
        };

        // api_secret = StarkNet account address (for future signing support)
        let account_address = if !credentials.api_secret.is_empty() {
            Some(credentials.api_secret.clone())
        } else {
            None
        };

        Ok(Self {
            jwt_token: Arc::new(RwLock::new(jwt_token)),
            account_address,
            time_offset_ms: Arc::new(RwLock::new(0)),
        })
    }

    /// Sync time with the Paradex server
    ///
    /// Call this during connector initialization using `/system/time`.
    pub async fn sync_time(&self, server_time_ms: i64) {
        let local_time = current_timestamp_millis() as i64;
        let mut offset = self.time_offset_ms.write().await;
        *offset = server_time_ms - local_time;
    }

    /// Get adjusted timestamp in milliseconds (accounts for server time offset)
    pub async fn get_timestamp(&self) -> u64 {
        let local = current_timestamp_millis() as i64;
        let offset = *self.time_offset_ms.read().await;
        (local + offset) as u64
    }

    /// Set a new JWT token (replaces the current token and resets expiry tracking)
    pub async fn set_jwt_token(&self, token: String) {
        let mut jwt = self.jwt_token.write().await;
        *jwt = Some(JwtToken::new_now(token));
    }

    /// Set a new JWT token with an explicit expiry timestamp (UNIX seconds)
    ///
    /// Use this when the token response includes an explicit `expires_at` field.
    pub async fn set_jwt_token_with_expiry(&self, token: String, expires_at: u64) {
        let mut jwt = self.jwt_token.write().await;
        *jwt = Some(JwtToken::new_with_expiry(token, expires_at));
    }

    /// Get the raw JWT token string
    ///
    /// Returns an error if no token has been set.
    pub async fn get_jwt_token(&self) -> ExchangeResult<String> {
        let jwt = self.jwt_token.read().await;
        jwt.as_ref()
            .map(|t| t.token.clone())
            .ok_or_else(|| ExchangeError::Auth(
                "JWT token not set. Paradex requires authentication.".to_string()
            ))
    }

    /// Check if the current token is still valid (respects safety margin)
    ///
    /// Returns true if a token exists and is not about to expire.
    pub async fn is_token_valid(&self) -> bool {
        let jwt = self.jwt_token.read().await;
        jwt.as_ref().map_or(false, |t| t.is_valid())
    }

    /// Check if the current token has expired (no safety margin)
    ///
    /// Returns true if no token exists or the token is past its expiry.
    pub async fn is_token_expired(&self) -> bool {
        let jwt = self.jwt_token.read().await;
        jwt.as_ref().map_or(true, |t| t.is_expired())
    }

    /// Get seconds until token expiry (0 if expired or no token)
    pub async fn secs_until_expiry(&self) -> u64 {
        let jwt = self.jwt_token.read().await;
        jwt.as_ref().map_or(0, |t| t.secs_until_expiry())
    }

    /// Get token age in seconds (0 if no token)
    pub async fn token_age_secs(&self) -> u64 {
        let jwt = self.jwt_token.read().await;
        jwt.as_ref().map_or(0, |t| t.age_secs())
    }

    /// Refresh the JWT token if it is expired or about to expire
    ///
    /// This method checks token validity and triggers a refresh when needed.
    ///
    /// # StarkNet Signing Required
    ///
    /// TODO: Full token refresh requires StarkNet cryptographic signing:
    /// 1. Add `starknet` crate dependency
    /// 2. Generate StarkNet signature over the auth message
    /// 3. POST to `/v1/auth` with the required headers:
    ///    - `PARADEX-STARKNET-ACCOUNT: {account_address}`
    ///    - `PARADEX-STARKNET-SIGNATURE: [{r}, {s}]`
    ///    - `PARADEX-TIMESTAMP: {unix_seconds}`
    ///
    /// For now, this method returns an error when called without a `starknet` backend,
    /// indicating that the caller must obtain a new JWT externally and call
    /// `set_jwt_token()` to update it.
    ///
    /// # Returns
    /// - `Ok(false)` — Token is still valid, no refresh needed
    /// - `Ok(true)` — Token was successfully refreshed
    /// - `Err(...)` — Refresh failed (token expired and no signing backend available)
    pub async fn refresh_if_needed(
        &self,
        _http_client: &reqwest::Client,
        _base_url: &str,
    ) -> ExchangeResult<bool> {
        if self.is_token_valid().await {
            return Ok(false);
        }

        // TODO: Implement StarkNet-based JWT refresh.
        //
        // When `starknet` crate is available:
        //
        // let timestamp_secs = self.get_timestamp().await / 1000;
        // let message = create_auth_message(timestamp_secs);
        // let signature = sign_starknet(&self.private_key, &message)?;
        //
        // let account = self.account_address.as_ref().ok_or_else(|| {
        //     ExchangeError::Auth("StarkNet account address required for refresh".to_string())
        // })?;
        //
        // let response = _http_client
        //     .post(format!("{}/v1/auth", _base_url))
        //     .header("PARADEX-STARKNET-ACCOUNT", account)
        //     .header("PARADEX-STARKNET-SIGNATURE", format!("[{}, {}]", signature.r, signature.s))
        //     .header("PARADEX-TIMESTAMP", timestamp_secs.to_string())
        //     .send()
        //     .await
        //     .map_err(|e| ExchangeError::Network(e.to_string()))?;
        //
        // let body: serde_json::Value = response.json().await
        //     .map_err(|e| ExchangeError::Parse(e.to_string()))?;
        //
        // let jwt = body["jwt_token"].as_str()
        //     .ok_or_else(|| ExchangeError::Parse("Missing jwt_token in auth response".to_string()))?;
        // let expires_at = body["expires_at"].as_u64().unwrap_or(0);
        //
        // if expires_at > 0 {
        //     self.set_jwt_token_with_expiry(jwt.to_string(), expires_at).await;
        // } else {
        //     self.set_jwt_token(jwt.to_string()).await;
        // }
        //
        // return Ok(true);

        Err(ExchangeError::Auth(
            "JWT token expired. Paradex requires StarkNet signing for token refresh \
             (starknet-rs integration required). Please obtain a new JWT token externally \
             and call set_jwt_token() to update it.".to_string()
        ))
    }

    /// Sign request and return headers
    ///
    /// For Paradex, signing means adding the `Authorization: Bearer {jwt}` header.
    /// Checks token expiry before signing and returns an error if token needs refresh.
    pub async fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        _body: &str,
    ) -> ExchangeResult<HashMap<String, String>> {
        // Warn if token will expire soon but don't block (caller handles refresh)
        if !self.is_token_valid().await {
            return Err(ExchangeError::Auth(
                format!(
                    "JWT token expired or expiring soon ({}s remaining). \
                     Call refresh_if_needed() before signing.",
                    self.secs_until_expiry().await
                )
            ));
        }

        let jwt = self.get_jwt_token().await?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", jwt));
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok(headers)
    }

    /// Sign request, attempting auto-refresh if token is expired
    ///
    /// This is the preferred method for connector use. It:
    /// 1. Checks token validity
    /// 2. Attempts refresh if needed (returns error if StarkNet signing unavailable)
    /// 3. Returns headers with valid token
    pub async fn sign_request_with_refresh(
        &self,
        method: &str,
        endpoint: &str,
        body: &str,
        http_client: &reqwest::Client,
        base_url: &str,
    ) -> ExchangeResult<HashMap<String, String>> {
        // Attempt refresh if needed
        self.refresh_if_needed(http_client, base_url).await?;
        self.sign_request(method, endpoint, body).await
    }

    /// Get timestamp header value (milliseconds, server-adjusted)
    pub async fn get_timestamp_header(&self) -> u64 {
        self.get_timestamp().await
    }

    /// Returns whether this auth has a StarkNet account configured (for future signing)
    pub fn has_account_address(&self) -> bool {
        self.account_address.is_some()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_token() {
        let credentials = Credentials::new("test_jwt_token", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        let token = auth.get_jwt_token().await.unwrap();
        assert_eq!(token, "test_jwt_token");
    }

    #[tokio::test]
    async fn test_sign_request() {
        let credentials = Credentials::new("test_jwt_token", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("GET", "/account", "").await.unwrap();

        assert!(headers.contains_key("Authorization"));
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer test_jwt_token".to_string())
        );
    }

    #[tokio::test]
    async fn test_set_jwt_token() {
        let credentials = Credentials::new("", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        // Initially no token
        assert!(auth.get_jwt_token().await.is_err());

        // Set token
        auth.set_jwt_token("new_token".to_string()).await;

        // Now should work
        let token = auth.get_jwt_token().await.unwrap();
        assert_eq!(token, "new_token");
    }

    #[tokio::test]
    async fn test_token_validity() {
        let credentials = Credentials::new("test_token", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        // Fresh token should be valid
        assert!(auth.is_token_valid().await);
        assert!(!auth.is_token_expired().await);

        // Should have significant time remaining
        let remaining = auth.secs_until_expiry().await;
        assert!(remaining > JWT_SAFETY_MARGIN_SECS);
        assert!(remaining <= JWT_LIFETIME_SECS);
    }

    #[tokio::test]
    async fn test_token_with_explicit_expiry() {
        let credentials = Credentials::new("", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        // Set token with explicit future expiry
        let future_expiry = current_timestamp_secs() + 600; // 10 minutes
        auth.set_jwt_token_with_expiry("token_with_expiry".to_string(), future_expiry)
            .await;

        assert!(auth.is_token_valid().await);
        let remaining = auth.secs_until_expiry().await;
        assert!(remaining > 500); // Should have ~570 seconds (600 - 30 margin)
    }

    #[tokio::test]
    async fn test_expired_token() {
        let credentials = Credentials::new("", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        // Set token with past expiry
        let past_expiry = current_timestamp_secs().saturating_sub(60); // expired 60 seconds ago
        auth.set_jwt_token_with_expiry("expired_token".to_string(), past_expiry)
            .await;

        assert!(!auth.is_token_valid().await);
        assert!(auth.is_token_expired().await);
        assert_eq!(auth.secs_until_expiry().await, 0);
    }

    #[tokio::test]
    async fn test_sign_request_fails_on_expired_token() {
        let credentials = Credentials::new("", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        // Set expired token
        let past_expiry = current_timestamp_secs().saturating_sub(60);
        auth.set_jwt_token_with_expiry("expired_token".to_string(), past_expiry)
            .await;

        // sign_request should fail
        let result = auth.sign_request("GET", "/account", "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expired"));
    }

    #[tokio::test]
    async fn test_no_token_fails_sign() {
        let credentials = Credentials::new("", "");
        let auth = ParadexAuth::new(&credentials).unwrap();

        // No token set
        assert!(!auth.is_token_valid().await);
        let result = auth.sign_request("GET", "/account", "").await;
        assert!(result.is_err());
    }
}
