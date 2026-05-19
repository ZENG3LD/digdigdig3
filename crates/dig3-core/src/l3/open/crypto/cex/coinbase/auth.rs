//! # Coinbase Authentication
//!
//! Implementation of JWT (ES256) signing for Coinbase Advanced Trade API.
//!
//! ## Signature Algorithm
//!
//! 1. Create JWT header with ES256, kid (API key name), and nonce
//! 2. Create JWT payload with sub, iss, nbf, exp, and uri
//! 3. Sign with ECDSA P-256 private key
//! 4. Return JWT as "Bearer {token}"
//!
//! ## Required Headers
//!
//! - `Authorization` - "Bearer {jwt_token}"
//!
//! ## Key Differences from KuCoin/Bybit
//!
//! - **JWT-based**: Uses ES256 (ECDSA P-256) instead of HMAC-SHA256
//! - **Single header**: Only Authorization header needed
//! - **EC Private Key**: PEM-encoded EC private key (not simple API secret)
//! - **2-minute expiration**: JWTs expire after 120 seconds
//! - **Nonce required**: Random 16-byte hex string for replay protection
//!
//! ## Manual JWT construction
//!
//! `jsonwebtoken::Header` does not support arbitrary custom fields, so the nonce
//! cannot be inserted via that API.  Instead the JWT is built by hand:
//!
//! 1. Serialise the header (including `nonce`) to JSON, then base64url-encode.
//! 2. Serialise the claims to JSON, then base64url-encode.
//! 3. Sign `header_b64 + "." + claims_b64` with ECDSA P-256 / SHA-256 via `ring`.
//! 4. Concatenate all three parts with `.` separators.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::core::Credentials;

/// Coinbase authentication handler
#[derive(Clone)]
pub struct CoinbaseAuth {
    /// API key name (e.g., "organizations/{org_id}/apiKeys/{key_id}")
    api_key_name: String,
    /// Raw PKCS#8 DER bytes of the EC private key (used by `ring` for signing)
    pkcs8_der: Vec<u8>,
}

/// JWT header for Coinbase — includes `nonce` which `jsonwebtoken::Header` cannot carry
#[derive(Debug, Serialize, Deserialize)]
struct CoinbaseJwtHeader<'a> {
    /// Algorithm — always "ES256"
    alg: &'a str,
    /// Token type — always "JWT"
    typ: &'a str,
    /// API key identifier
    kid: &'a str,
    /// Random nonce (16 bytes as lowercase hex) for replay protection
    nonce: &'a str,
}

/// JWT payload claims for Coinbase
#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    /// Subject - API key name
    sub: String,
    /// Issuer - always "cdp"
    iss: String,
    /// Not before - current timestamp
    nbf: u64,
    /// Expiration - current timestamp + 120 seconds
    exp: u64,
    /// URI - "{METHOD} {HOST}{PATH}"
    uri: String,
}

impl CoinbaseAuth {
    /// Create new auth handler
    ///
    /// # Arguments
    ///
    /// * `credentials` - Credentials with:
    ///   - `api_key`: API key name (e.g., "organizations/{org_id}/apiKeys/{key_id}")
    ///   - `api_secret`: EC private key in PEM format (PKCS#8)
    ///
    /// # Errors
    ///
    /// Returns error if private key is invalid or cannot be parsed.
    pub fn new(credentials: &Credentials) -> Result<Self, String> {
        // Decode PKCS#8 DER from PEM manually — strip the BEGIN/END header lines,
        // concatenate the base64-encoded body, then standard-base64-decode.
        // This avoids relying on jsonwebtoken::EncodingKey::inner() which is pub(crate).
        let pkcs8_der = Self::pem_to_der(credentials.api_secret.as_str())?;

        // Validate that ring can load the key before storing it.
        let rng = SystemRandom::new();
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8_der, &rng)
            .map_err(|e| format!("Invalid EC private key (ring): {}", e))?;

        Ok(Self {
            api_key_name: credentials.api_key.clone(),
            pkcs8_der,
        })
    }

    /// Decode a PEM-encoded key to raw DER bytes.
    ///
    /// Supports "PRIVATE KEY" (PKCS#8) and "EC PRIVATE KEY" (SEC1/PKCS#1 EC).
    /// Strips the `-----BEGIN ...-----` / `-----END ...-----` lines and
    /// base64-decodes the body.
    fn pem_to_der(pem: &str) -> Result<Vec<u8>, String> {
        let body: String = pem
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("");
        BASE64_STANDARD
            .decode(body.as_bytes())
            .map_err(|e| format!("Failed to decode PEM body: {}", e))
    }

    /// Generate random 16-byte nonce as lowercase hex
    fn generate_nonce() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }

    /// Get current Unix timestamp in seconds
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs()
    }

    /// Build a Coinbase JWT manually so that `nonce` is present in the header.
    ///
    /// The `jsonwebtoken` crate's `Header` struct does not support arbitrary
    /// extra fields, so we construct the three JWT segments by hand:
    ///
    /// ```text
    /// jwt = base64url(header_json) + "." + base64url(claims_json) + "." + base64url(sig)
    /// ```
    ///
    /// The signature covers `header_b64 + "." + claims_b64` and is produced with
    /// ECDSA P-256 + SHA-256 via the `ring` crate.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method ("GET", "POST", etc.)
    /// * `host`   - API hostname (e.g., "api.coinbase.com")
    /// * `path`   - Request path (e.g., "/api/v3/brokerage/accounts")
    ///
    /// # URI Format
    ///
    /// `"{METHOD} {HOST}{PATH}"` — no scheme, no query string.
    pub fn build_jwt(&self, method: &str, host: &str, path: &str) -> Result<String, String> {
        let now = Self::current_timestamp();
        let nonce = Self::generate_nonce();

        // --- 1. Header (with nonce) ---
        let header = CoinbaseJwtHeader {
            alg: "ES256",
            typ: "JWT",
            kid: &self.api_key_name,
            nonce: &nonce,
        };
        let header_json = serde_json::to_vec(&header)
            .map_err(|e| format!("Failed to serialise JWT header: {}", e))?;
        let header_b64 = URL_SAFE_NO_PAD.encode(&header_json);

        // --- 2. Claims ---
        let uri = format!("{} {}{}", method.to_uppercase(), host, path);
        let claims = JwtClaims {
            sub: self.api_key_name.clone(),
            iss: "cdp".to_string(),
            nbf: now,
            exp: now + 120,
            uri,
        };
        let claims_json = serde_json::to_vec(&claims)
            .map_err(|e| format!("Failed to serialise JWT claims: {}", e))?;
        let claims_b64 = URL_SAFE_NO_PAD.encode(&claims_json);

        // --- 3. Signing input ---
        let signing_input = format!("{}.{}", header_b64, claims_b64);

        // --- 4. ECDSA P-256 / SHA-256 signature via ring ---
        let rng = SystemRandom::new();
        let key_pair = EcdsaKeyPair::from_pkcs8(
            &ECDSA_P256_SHA256_FIXED_SIGNING,
            &self.pkcs8_der,
            &rng,
        )
        .map_err(|e| format!("Failed to load signing key: {}", e))?;

        let signature = key_pair
            .sign(&rng, signing_input.as_bytes())
            .map_err(|e| format!("ECDSA signing failed: {}", e))?;

        let sig_b64 = URL_SAFE_NO_PAD.encode(signature.as_ref());

        // --- 5. Assemble ---
        Ok(format!("{}.{}", signing_input, sig_b64))
    }

    /// Build JWT for WebSocket connection
    ///
    /// # Arguments
    ///
    /// * `ws_host` - WebSocket hostname (e.g., "advanced-trade-ws.coinbase.com")
    ///
    /// # WebSocket URI Format
    ///
    /// The URI for WebSocket is: `"GET {ws_host}"`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let jwt = auth.build_websocket_jwt("advanced-trade-ws-user.coinbase.com")?;
    /// // URI will be: "GET advanced-trade-ws-user.coinbase.com"
    /// ```
    pub fn build_websocket_jwt(&self, ws_host: &str) -> Result<String, String> {
        self.build_jwt("GET", ws_host, "")
    }

    /// Sign request and return headers
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method ("GET", "POST", etc.)
    /// * `path` - Request path (e.g., "/api/v3/brokerage/accounts")
    ///
    /// # Returns
    ///
    /// HashMap with "Authorization" header containing "Bearer {jwt}"
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let headers = auth.sign_request("GET", "/api/v3/brokerage/accounts");
    /// // headers = {"Authorization": "Bearer eyJhbGci..."}
    /// ```
    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
    ) -> Result<HashMap<String, String>, String> {
        let jwt = self.build_jwt(method, "api.coinbase.com", path)?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", jwt));

        // Add Content-Type for POST requests
        if method.to_uppercase() == "POST" {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }

        Ok(headers)
    }

    /// Get API key name
    pub fn api_key_name(&self) -> &str {
        &self.api_key_name
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a valid EC private key
    // For actual testing, use a test key or mock the encoding

    #[test]
    fn test_generate_nonce() {
        let nonce = CoinbaseAuth::generate_nonce();
        assert_eq!(nonce.len(), 32); // 16 bytes = 32 hex chars
        assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_current_timestamp() {
        let ts = CoinbaseAuth::current_timestamp();
        assert!(ts > 1700000000); // After Nov 2023
        assert!(ts < 2000000000); // Before May 2033
    }

    // Note: Full JWT testing requires valid EC key pair
    // Will be tested in integration tests with real credentials
}
