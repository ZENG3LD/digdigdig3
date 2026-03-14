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
//! ## StarkNet Signing
//!
//! Full JWT generation requires StarkNet cryptography (sign with private key).
//! When the `starknet` feature is enabled, `refresh_if_needed()` will automatically
//! sign a timestamp and POST to `/v1/auth` to obtain a fresh JWT.
//! Without the feature, pre-obtained JWT tokens can be passed via `credentials.api_key`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

#[cfg(feature = "starknet")]
use starknet_crypto::{sign, get_public_key, rfc6979_generate_k, FieldElement};

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

    /// StarkNet private key (hex string, used for signing when `starknet` feature is enabled)
    private_key: Option<String>,

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
    /// 2. **StarkNet signing** (requires `starknet` feature):
    ///    Pass the StarkNet private key (hex) in `api_secret`.
    ///    Optionally pass `{"account_address": "0x..."}` in passphrase.
    ///    Call `refresh_if_needed()` to obtain JWT automatically.
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        // api_key = JWT token (if pre-obtained)
        let jwt_token = if !credentials.api_key.is_empty() {
            // Token is provided; treat as freshly issued (we don't know the real expiry)
            Some(JwtToken::new_now(credentials.api_key.clone()))
        } else {
            None
        };

        // api_secret = StarkNet private key (hex) for signing; account_address derived or from passphrase
        let (private_key, account_address) = if !credentials.api_secret.is_empty() {
            let pk = Some(credentials.api_secret.clone());
            // Account address may be in passphrase as JSON: {"account_address": "0x..."}
            let addr = credentials.passphrase.as_ref().and_then(|p| {
                serde_json::from_str::<serde_json::Value>(p).ok()
                    .and_then(|v| v.get("account_address").and_then(|a| a.as_str()).map(|s| s.to_string()))
            });
            (pk, addr)
        } else {
            (None, None)
        };

        Ok(Self {
            jwt_token: Arc::new(RwLock::new(jwt_token)),
            account_address,
            private_key,
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

    /// Generate a signed JWT request for Paradex authentication using StarkNet ECDSA.
    ///
    /// Returns `(public_key_hex, "r_hex,s_hex")` — the public key and signature
    /// components needed for the `POST /v1/auth` headers:
    /// - `PARADEX-STARKNET-ACCOUNT: {public_key_hex}`
    /// - `PARADEX-STARKNET-SIGNATURE: [{r_hex}, {s_hex}]`
    /// - `PARADEX-TIMESTAMP: {timestamp_secs}`
    ///
    /// # Nonce generation
    ///
    /// Uses RFC 6979 deterministic nonce derivation via `rfc6979_generate_k`.
    /// This produces a unique, unpredictable `k` per (private_key, message) pair
    /// without requiring an external RNG, eliminating the private key leakage risk
    /// that a reused or predictable `k` would cause.
    #[cfg(feature = "starknet")]
    pub fn sign_auth_request(&self, timestamp: u64) -> ExchangeResult<(String, String)> {
        let private_key_hex = self.private_key.as_ref().ok_or_else(|| {
            ExchangeError::Auth(
                "StarkNet private key not configured. Provide api_secret as hex private key."
                    .to_string(),
            )
        })?;

        // Parse the StarkNet private key
        let private_key = FieldElement::from_hex_be(private_key_hex)
            .map_err(|e| ExchangeError::Auth(format!("Invalid StarkNet key: {}", e)))?;

        // Get public key for the request
        let public_key = get_public_key(&private_key);

        // Build the message to sign (timestamp-based)
        let message = FieldElement::from(timestamp);

        // Generate a secure deterministic nonce using RFC 6979.
        // This derives k from (private_key, message_hash) deterministically,
        // so k is unique per message and never requires a random source.
        // Passing `None` as seed uses the standard RFC 6979 derivation.
        let k = rfc6979_generate_k(&message, &private_key, None);

        // Sign with StarkNet ECDSA using the RFC 6979 nonce
        let signature = sign(&private_key, &message, &k)
            .map_err(|e| ExchangeError::Auth(format!("StarkNet sign failed: {}", e)))?;

        Ok((
            format!("{:#x}", public_key),
            format!("{:#x},{:#x}", signature.r, signature.s),
        ))
    }

    /// Refresh the JWT token if it is expired or about to expire
    ///
    /// This method checks token validity and triggers a refresh when needed.
    ///
    /// When the `starknet` feature is enabled, automatically signs the current
    /// timestamp and POSTs to `/v1/auth` to obtain a fresh JWT.
    ///
    /// Without the feature, returns an error instructing the caller to provide
    /// a new JWT token externally via `set_jwt_token()`.
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

        // When the `starknet` feature is enabled, perform automatic JWT refresh
        // by signing the timestamp with the StarkNet private key.
        #[cfg(feature = "starknet")]
        {
            let timestamp_secs = self.get_timestamp().await / 1000;

            let (public_key_hex, sig_str) = self.sign_auth_request(timestamp_secs)?;

            // Format as "[r_hex, s_hex]" — Paradex expects this bracket format.
            let signature_header = {
                let parts: Vec<&str> = sig_str.splitn(2, ',').collect();
                if parts.len() == 2 {
                    format!("[{}, {}]", parts[0], parts[1])
                } else {
                    format!("[{}]", sig_str)
                }
            };

            let response = _http_client
                .post(format!("{}/v1/auth", _base_url))
                .header("PARADEX-STARKNET-ACCOUNT", &public_key_hex)
                .header("PARADEX-STARKNET-SIGNATURE", &signature_header)
                .header("PARADEX-TIMESTAMP", timestamp_secs.to_string())
                .send()
                .await
                .map_err(|e| ExchangeError::Network(e.to_string()))?;

            let body: serde_json::Value = response
                .json()
                .await
                .map_err(|e| ExchangeError::Parse(e.to_string()))?;

            let jwt = body["jwt_token"]
                .as_str()
                .ok_or_else(|| {
                    ExchangeError::Parse("Missing jwt_token in auth response".to_string())
                })?;

            let expires_at = body["expires_at"].as_u64().unwrap_or(0);

            if expires_at > 0 {
                self.set_jwt_token_with_expiry(jwt.to_string(), expires_at).await;
            } else {
                self.set_jwt_token(jwt.to_string()).await;
            }

            return Ok(true);
        }

        // When `starknet` feature is disabled, instruct the caller to provide a token manually.
        #[cfg(not(feature = "starknet"))]
        Err(ExchangeError::Auth(
            "JWT token expired. Paradex requires StarkNet signing for token refresh \
             (enable the `starknet` feature or obtain a new JWT token externally \
             and call set_jwt_token() to update it)."
                .to_string(),
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

    /// Returns whether this auth has a StarkNet private key configured for signing
    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
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
