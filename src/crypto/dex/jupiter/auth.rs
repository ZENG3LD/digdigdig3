//! # Jupiter Authentication
//!
//! Jupiter uses simple API key authentication via `x-api-key` header.
//! No signing or HMAC required.

use std::collections::HashMap;

/// Jupiter authentication
#[derive(Clone)]
pub struct JupiterAuth {
    api_key: String,
}

impl JupiterAuth {
    /// Create new Jupiter auth with API key
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Get authentication headers
    ///
    /// Jupiter only requires the `x-api-key` header for authenticated endpoints.
    pub fn auth_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), self.api_key.clone());
        headers
    }
}
