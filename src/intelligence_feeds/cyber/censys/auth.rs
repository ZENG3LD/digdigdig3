//! Censys authentication
//!
//! Authentication type: HTTP Basic Auth (API ID + API Secret)
//!
//! Censys uses HTTP Basic Authentication where the API ID is the username
//! and the API Secret is the password.

use std::collections::HashMap;

/// Censys authentication credentials
#[derive(Clone)]
pub struct CensysAuth {
    pub api_id: Option<String>,
    pub api_secret: Option<String>,
}

impl CensysAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variables: `CENSYS_API_ID`, `CENSYS_API_SECRET`
    pub fn from_env() -> Self {
        Self {
            api_id: std::env::var("CENSYS_API_ID").ok(),
            api_secret: std::env::var("CENSYS_API_SECRET").ok(),
        }
    }

    /// Create auth with explicit API credentials
    pub fn new(api_id: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_id: Some(api_id.into()),
            api_secret: Some(api_secret.into()),
        }
    }

    /// Get Basic Auth credentials tuple for reqwest
    ///
    /// Returns (username, password) tuple for use with reqwest's basic_auth()
    pub fn get_basic_auth(&self) -> Option<(String, String)> {
        match (&self.api_id, &self.api_secret) {
            (Some(id), Some(secret)) => Some((id.clone(), secret.clone())),
            _ => None,
        }
    }

    /// Add authentication to headers (fallback method)
    ///
    /// Note: Prefer using get_basic_auth() with reqwest's .basic_auth() method
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let (Some(id), Some(secret)) = (&self.api_id, &self.api_secret) {
            // Manual Basic Auth header construction if needed
            let credentials = format!("{}:{}", id, secret);
            let encoded = base64_encode(&credentials);
            headers.insert("Authorization".to_string(), format!("Basic {}", encoded));
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_id.is_some() && self.api_secret.is_some()
    }

    /// Get API ID (for debugging/logging - use carefully)
    pub fn get_api_id(&self) -> Option<&str> {
        self.api_id.as_deref()
    }
}

impl Default for CensysAuth {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Simple base64 encoding for Basic Auth
fn base64_encode(input: &str) -> String {
    // Simple base64 implementation without external crate
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0F) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3F) as usize;

        result.push(BASE64_CHARS[b1] as char);
        result.push(BASE64_CHARS[b2] as char);
        result.push(if chunk.len() > 1 { BASE64_CHARS[b3] as char } else { '=' });
        result.push(if chunk.len() > 2 { BASE64_CHARS[b4] as char } else { '=' });
    }

    result
}
