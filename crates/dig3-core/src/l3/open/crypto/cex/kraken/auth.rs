//! # Kraken Authentication
//!
//! Implementation of request signing for Kraken API.
//!
//! ## Signature Algorithm (Spot REST)
//!
//! 1. Concatenate: `nonce + POST data`
//! 2. SHA256 hash of concatenated string
//! 3. Concatenate: `URI path + SHA256 hash`
//! 4. HMAC-SHA512 with base64-decoded secret
//! 5. Base64 encode the result
//!
//! ## Headers
//!
//! - `API-Key` - API key
//! - `API-Sign` - Signature (Base64)

use std::collections::HashMap;

use crate::core::{
    hmac_sha512, sha256, encode_base64,
    timestamp_millis,
    Credentials, ExchangeResult,
};

/// Kraken authentication
#[derive(Clone)]
pub struct KrakenAuth {
    api_key: String,
    api_secret: String,
    /// Nonce counter (strictly increasing)
    nonce: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl KrakenAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            nonce: std::sync::Arc::new(std::sync::Mutex::new(timestamp_millis())),
        })
    }

    /// Get next nonce (strictly increasing)
    fn get_nonce(&self) -> u64 {
        let mut nonce = self.nonce.lock().expect("Mutex poisoned");
        *nonce += 1;
        *nonce
    }

    /// Sign request and return headers + POST data
    ///
    /// # Kraken Signature Algorithm
    ///
    /// 1. Create POST data string with nonce
    /// 2. SHA256(nonce + POST data)
    /// 3. Concatenate URI path + SHA256 hash
    /// 4. HMAC-SHA512(decoded secret, message from step 3)
    /// 5. Base64 encode signature
    pub fn sign_request(
        &self,
        uri_path: &str,
        params: &HashMap<String, String>,
    ) -> (HashMap<String, String>, String) {
        let nonce = self.get_nonce();

        // Build POST data string
        let mut post_params = params.clone();
        post_params.insert("nonce".to_string(), nonce.to_string());

        let post_data = post_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Step 1: Concatenate nonce + POST data
        let nonce_post = format!("{}{}", nonce, post_data);

        // Step 2: SHA256 hash
        let sha256_hash = sha256(nonce_post.as_bytes());

        // Step 3: URI path + SHA256 hash
        let mut sign_message = uri_path.as_bytes().to_vec();
        sign_message.extend_from_slice(&sha256_hash);

        // Step 4: Decode secret and HMAC-SHA512
        // Kraken API secret is base64-encoded
        let secret_decoded = match base64::decode(&self.api_secret) {
            Ok(s) => s,
            Err(_) => {
                // If decode fails, use raw bytes as fallback
                self.api_secret.as_bytes().to_vec()
            }
        };

        let signature = hmac_sha512(&secret_decoded, &sign_message);

        // Step 5: Base64 encode signature
        let api_sign = encode_base64(&signature);

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("API-Key".to_string(), self.api_key.clone());
        headers.insert("API-Sign".to_string(), api_sign);
        headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());

        (headers, post_data)
    }

    /// Get API key (for headers without signing)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

// Need to add base64 decode function if not in utils
mod base64 {
    use crate::core::ExchangeResult;

    pub fn decode(input: &str) -> ExchangeResult<Vec<u8>> {
        // Simple base64 decode using standard alphabet
        let bytes = input.as_bytes();
        let mut result = Vec::new();

        const DECODE_TABLE: [u8; 128] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x3E, 0xFF, 0xFF, 0xFF, 0x3F,
            0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF,
            0xFF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
            0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let mut i = 0;
        while i < bytes.len() {
            // Skip whitespace
            if bytes[i].is_ascii_whitespace() {
                i += 1;
                continue;
            }

            // Get 4 chars (or less at end)
            let mut buf = [0u8; 4];
            let mut buf_len = 0;

            for slot in &mut buf {
                if i >= bytes.len() {
                    break;
                }
                let c = bytes[i];
                if c == b'=' {
                    i += 1;
                    break;
                }
                if c >= 128 || DECODE_TABLE[c as usize] == 0xFF {
                    return Err(crate::core::ExchangeError::Parse("Invalid base64 character".to_string()));
                }
                *slot = DECODE_TABLE[c as usize];
                buf_len += 1;
                i += 1;
            }

            // Decode the buffer
            if buf_len >= 2 {
                result.push((buf[0] << 2) | (buf[1] >> 4));
            }
            if buf_len >= 3 {
                result.push((buf[1] << 4) | (buf[2] >> 2));
            }
            if buf_len >= 4 {
                result.push((buf[2] << 6) | buf[3]);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = KrakenAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        params.insert("pair".to_string(), "XBTUSD".to_string());

        let (headers, post_data) = auth.sign_request("/0/private/Balance", &params);

        assert!(headers.contains_key("API-Key"));
        assert!(headers.contains_key("API-Sign"));
        assert_eq!(headers.get("API-Key"), Some(&"test_key".to_string()));
        assert!(post_data.contains("nonce="));
        assert!(post_data.contains("pair=XBTUSD"));
    }

    #[test]
    fn test_base64_decode() {
        let result = base64::decode("SGVsbG8=").unwrap();
        assert_eq!(result, b"Hello");

        let result = base64::decode("V29ybGQ=").unwrap();
        assert_eq!(result, b"World");
    }
}
