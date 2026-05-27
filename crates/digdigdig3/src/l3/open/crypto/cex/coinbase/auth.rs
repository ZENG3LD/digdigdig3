//! # Coinbase Authentication
//!
//! Implementation of JWT (ES256) signing for Coinbase Advanced Trade API.
//! Native-only: uses `ring` (ECDSA P-256) and `rand`, which do not compile to wasm32.
//! On wasm32 a zero-method stub is provided so the module structure is preserved.

// ─── Native implementation ────────────────────────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
mod native {
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
        alg: &'a str,
        typ: &'a str,
        kid: &'a str,
        nonce: &'a str,
    }

    /// JWT payload claims for Coinbase
    #[derive(Debug, Serialize, Deserialize)]
    struct JwtClaims {
        sub: String,
        iss: String,
        nbf: u64,
        exp: u64,
        uri: String,
    }

    impl CoinbaseAuth {
        pub fn new(credentials: &Credentials) -> Result<Self, String> {
            let pkcs8_der = Self::pem_to_der(credentials.api_secret.as_str())?;
            let rng = SystemRandom::new();
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8_der, &rng)
                .map_err(|e| format!("Invalid EC private key (ring): {}", e))?;
            Ok(Self {
                api_key_name: credentials.api_key.clone(),
                pkcs8_der,
            })
        }

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

        fn generate_nonce() -> String {
            let mut rng = rand::thread_rng();
            let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
            hex::encode(bytes)
        }

        fn current_timestamp() -> u64 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time is before UNIX epoch")
                .as_secs()
        }

        pub fn build_jwt(&self, method: &str, host: &str, path: &str) -> Result<String, String> {
            let now = Self::current_timestamp();
            let nonce = Self::generate_nonce();

            let header = CoinbaseJwtHeader {
                alg: "ES256",
                typ: "JWT",
                kid: &self.api_key_name,
                nonce: &nonce,
            };
            let header_json = serde_json::to_vec(&header)
                .map_err(|e| format!("Failed to serialise JWT header: {}", e))?;
            let header_b64 = URL_SAFE_NO_PAD.encode(&header_json);

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

            let signing_input = format!("{}.{}", header_b64, claims_b64);

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
            Ok(format!("{}.{}", signing_input, sig_b64))
        }

        pub fn build_websocket_jwt(&self, ws_host: &str) -> Result<String, String> {
            self.build_jwt("GET", ws_host, "")
        }

        pub fn sign_request(
            &self,
            method: &str,
            path: &str,
        ) -> Result<HashMap<String, String>, String> {
            let jwt = self.build_jwt(method, "api.coinbase.com", path)?;
            let mut headers = HashMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {}", jwt));
            if method.to_uppercase() == "POST" {
                headers.insert("Content-Type".to_string(), "application/json".to_string());
            }
            Ok(headers)
        }

        pub fn api_key_name(&self) -> &str {
            &self.api_key_name
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_generate_nonce() {
            let nonce = CoinbaseAuth::generate_nonce();
            assert_eq!(nonce.len(), 32);
            assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn test_current_timestamp() {
            let ts = CoinbaseAuth::current_timestamp();
            assert!(ts > 1700000000);
            assert!(ts < 2000000000);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::CoinbaseAuth;

// ─── Wasm stub ────────────────────────────────────────────────────────────────
// CoinbaseAuth is a non-constructable ZST on wasm32 — keeps connector.rs
// compiling without carrying ring/rand into the wasm binary.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct CoinbaseAuth {
    _private: (),
}

#[cfg(target_arch = "wasm32")]
impl CoinbaseAuth {
    pub fn new(_: &crate::core::Credentials) -> Result<Self, String> {
        Err("Coinbase JWT signing requires native (ring/rand not available on wasm32)".into())
    }

    pub fn sign_request(&self, _: &str, _: &str) -> Result<std::collections::HashMap<String, String>, String> {
        Err("Coinbase JWT signing not available on wasm32".into())
    }

    pub fn build_websocket_jwt(&self, _: &str) -> Result<String, String> {
        Err("Coinbase JWT signing not available on wasm32".into())
    }

    pub fn api_key_name(&self) -> &str {
        ""
    }
}
