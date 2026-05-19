//! Polymarket L2 Authentication
//!
//! Implements HMAC-SHA256 signing for authenticated CLOB API endpoints.
//!
//! ## Authentication Headers
//!
//! Authenticated requests include:
//! - `POLY_ADDRESS` — Polygon wallet address (0x...)
//! - `POLY_API_KEY` — API key (UUID format)
//! - `POLY_SIGNATURE` — HMAC-SHA256 of `timestamp + method + path + [body]`
//! - `POLY_TIMESTAMP` — Unix timestamp in milliseconds
//! - `POLY_PASSPHRASE` — API passphrase
//!
//! ## Signature Algorithm
//!
//! 1. Construct message: `timestamp + method + path + [body]`
//! 2. Decode the base64 secret
//! 3. Compute HMAC-SHA256(decoded_secret, message)
//! 4. Return base64 URL-safe encoded result

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Authentication error
#[derive(Debug)]
pub enum AuthError {
    /// Failed to get system time
    TimeError(String),
    /// Invalid base64 secret
    InvalidSecret(String),
    /// HMAC computation failed
    HmacError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimeError(msg) => write!(f, "Time error: {}", msg),
            Self::InvalidSecret(msg) => write!(f, "Invalid secret: {}", msg),
            Self::HmacError(msg) => write!(f, "HMAC error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

// ═══════════════════════════════════════════════════════════════════════════
// CREDENTIALS
// ═══════════════════════════════════════════════════════════════════════════

/// L2 API credentials for authenticated CLOB endpoints
#[derive(Debug, Clone)]
pub struct PolymarketCredentials {
    /// Polygon wallet address (0x + 40 hex chars)
    pub address: String,
    /// API key (UUID format)
    pub api_key: String,
    /// Base64-encoded secret for HMAC signing (URL-safe or standard)
    pub secret: String,
    /// API passphrase
    pub passphrase: String,
}

impl PolymarketCredentials {
    /// Create credentials from provided values
    pub fn new(
        address: impl Into<String>,
        api_key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            api_key: api_key.into(),
            secret: secret.into(),
            passphrase: passphrase.into(),
        }
    }

    /// Load credentials from environment variables
    ///
    /// Variables: `POLY_ADDRESS`, `POLY_API_KEY`, `POLY_SECRET`, `POLY_PASSPHRASE`
    pub fn from_env() -> Option<Self> {
        let address = std::env::var("POLY_ADDRESS").ok()?;
        let api_key = std::env::var("POLY_API_KEY").ok()?;
        let secret = std::env::var("POLY_SECRET").ok()?;
        let passphrase = std::env::var("POLY_PASSPHRASE").ok()?;

        Some(Self {
            address,
            api_key,
            secret,
            passphrase,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AUTH IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

/// Polymarket authenticator (public mode — no-op; authenticated mode — L2 HMAC)
#[derive(Clone)]
pub struct PolymarketAuth {
    credentials: Option<PolymarketCredentials>,
}

impl PolymarketAuth {
    /// Create public authenticator (no credentials)
    pub fn new() -> Self {
        Self { credentials: None }
    }

    /// Create authenticator from environment variables
    pub fn from_env() -> Self {
        Self {
            credentials: PolymarketCredentials::from_env(),
        }
    }

    /// Create authenticated instance with explicit credentials
    pub fn with_credentials(creds: PolymarketCredentials) -> Self {
        Self {
            credentials: Some(creds),
        }
    }

    /// Whether L2 credentials are configured
    pub fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    /// Build authentication headers for a request
    ///
    /// Returns `None` if no credentials are configured.
    pub fn build_headers(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Option<HashMap<String, String>> {
        let creds = self.credentials.as_ref()?;
        let timestamp = get_timestamp_ms().ok()?.to_string();

        let signature = sign_request(&creds.secret, &timestamp, method, path, body).ok()?;

        let mut headers = HashMap::new();
        headers.insert("POLY_ADDRESS".to_string(), creds.address.clone());
        headers.insert("POLY_API_KEY".to_string(), creds.api_key.clone());
        headers.insert("POLY_SIGNATURE".to_string(), signature);
        headers.insert("POLY_TIMESTAMP".to_string(), timestamp);
        headers.insert("POLY_PASSPHRASE".to_string(), creds.passphrase.clone());

        Some(headers)
    }
}

impl Default for PolymarketAuth {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIGNING
// ═══════════════════════════════════════════════════════════════════════════

/// Get current Unix timestamp in milliseconds
pub fn get_timestamp_ms() -> Result<u64, AuthError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .map_err(|e| AuthError::TimeError(e.to_string()))
}

/// Decode base64 (URL-safe or standard) string to bytes
fn base64_decode(encoded: &str) -> Result<Vec<u8>, AuthError> {
    // Try URL-safe first (Polymarket uses URL-safe base64)
    if let Ok(decoded) = URL_SAFE.decode(encoded) {
        return Ok(decoded);
    }

    // Fall back to standard base64 (with padding normalization)
    let standard = encoded.replace('-', "+").replace('_', "/");
    let padded = match standard.len() % 4 {
        2 => format!("{}==", standard),
        3 => format!("{}=", standard),
        _ => standard,
    };

    base64::engine::general_purpose::STANDARD
        .decode(&padded)
        .map_err(|e| AuthError::InvalidSecret(format!("Base64 decode failed: {}", e)))
}

/// Encode bytes as URL-safe base64 (no padding)
fn base64_encode_url(data: &[u8]) -> String {
    URL_SAFE.encode(data)
}

/// Compute HMAC-SHA256 signature for a Polymarket API request
///
/// ## Message format
///
/// `timestamp + method + path + [body]`
///
/// ## Parameters
///
/// - `secret` — Base64-encoded API secret
/// - `timestamp` — Unix timestamp in milliseconds as string
/// - `method` — HTTP method: "GET", "POST", "DELETE"
/// - `path` — Request path, e.g. "/orders"
/// - `body` — Optional JSON body
pub fn sign_request(
    secret: &str,
    timestamp: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<String, AuthError> {
    // Build the message to sign
    let mut message = format!("{}{}{}", timestamp, method, path);
    if let Some(b) = body {
        if !b.is_empty() {
            message.push_str(b);
        }
    }

    // Decode the secret
    let key = base64_decode(secret)?;

    // Compute HMAC-SHA256
    let mut mac = Hmac::<Sha256>::new_from_slice(&key)
        .map_err(|e| AuthError::HmacError(format!("Invalid HMAC key: {}", e)))?;
    mac.update(message.as_bytes());
    let result = mac.finalize().into_bytes();

    // Return URL-safe base64 encoded signature
    Ok(base64_encode_url(&result))
}

/// Create WebSocket authentication message for the user channel
///
/// The WS user channel uses raw credentials (not HMAC-signed).
///
/// # Returns
///
/// JSON string to send as the first message after connecting.
pub fn _create_ws_auth_message(creds: &PolymarketCredentials, markets: &[String]) -> String {
    let markets_json: Vec<serde_json::Value> = markets
        .iter()
        .map(|m| serde_json::Value::String(m.clone()))
        .collect();

    serde_json::json!({
        "type": "user",
        "auth": {
            "apiKey": creds.api_key,
            "secret": creds.secret,
            "passphrase": creds.passphrase
        },
        "markets": markets_json
    })
    .to_string()
}
