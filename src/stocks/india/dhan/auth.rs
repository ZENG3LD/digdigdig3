//! # Dhan Authentication
//!
//! Implementation of JWT token-based authentication for Dhan API.
//!
//! ## Authentication Flow
//!
//! 1. Generate access token using client_id, api_key, api_secret
//! 2. Token valid for 24 hours
//! 3. Include token in `access-token` header for all requests
//!
//! ## Headers
//!
//! - `access-token` - JWT token
//! - `Content-Type` - application/json
//!
//! ## Note
//! Unlike most exchanges, Dhan does NOT use HMAC signing.
//! Authentication is purely JWT token-based.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::{
    Credentials, ExchangeResult, ExchangeError, HttpClient,
};

/// Dhan authentication
#[derive(Clone)]
pub struct DhanAuth {
    client_id: String,
    api_key: String,
    api_secret: String,
    access_token: Option<String>,
    token_expiry: Option<u64>, // Unix timestamp in seconds
}

/// Request body for token generation
#[derive(Debug, Serialize)]
struct TokenRequest {
    client_id: String,
    api_key: String,
    api_secret: String,
}

/// Response from token generation
#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(default)]
    status: String,
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    remarks: String,
}

impl DhanAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        // client_id is stored in the passphrase field for Dhan
        let client_id = credentials.passphrase.clone()
            .ok_or_else(|| ExchangeError::Auth("Dhan requires client_id in passphrase field".to_string()))?;

        Ok(Self {
            client_id,
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            access_token: None,
            token_expiry: None,
        })
    }

    /// Generate new access token
    pub async fn generate_token(&mut self, base_url: &str, http_client: &HttpClient) -> ExchangeResult<String> {
        let url = format!("{}/v2/access_token", base_url);

        let request = TokenRequest {
            client_id: self.client_id.clone(),
            api_key: self.api_key.clone(),
            api_secret: self.api_secret.clone(),
        };

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = serde_json::to_string(&request)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize request: {}", e)))?;

        let body_value: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse body to Value: {}", e)))?;

        let response_value = http_client
            .post(&url, &body_value, &headers)
            .await?;

        let response: TokenResponse = serde_json::from_value(response_value)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse token response: {}", e)))?;

        if response.status != "success" || response.access_token.is_empty() {
            return Err(ExchangeError::Auth(format!(
                "Token generation failed: {}",
                response.remarks
            )));
        }

        // Token valid for 24 hours
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs();

        self.token_expiry = Some(now + 86400); // 24 hours = 86400 seconds
        self.access_token = Some(response.access_token.clone());

        Ok(response.access_token)
    }

    /// Check if token is expired or about to expire (within 1 hour)
    pub fn is_token_expired(&self) -> bool {
        if let Some(expiry) = self.token_expiry {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before UNIX epoch")
                .as_secs();

            // Consider token expired if less than 1 hour remaining
            now >= (expiry - 3600)
        } else {
            true // No token = expired
        }
    }

    /// Get current access token (or generate if expired)
    pub async fn get_token(&mut self, base_url: &str, http_client: &HttpClient) -> ExchangeResult<String> {
        if self.is_token_expired() {
            self.generate_token(base_url, http_client).await
        } else {
            self.access_token.clone()
                .ok_or_else(|| ExchangeError::Auth("No access token available".to_string()))
        }
    }

    /// Build headers for authenticated request
    pub async fn build_headers(&mut self, base_url: &str, http_client: &HttpClient) -> ExchangeResult<HashMap<String, String>> {
        let token = self.get_token(base_url, http_client).await?;

        let mut headers = HashMap::new();
        headers.insert("access-token".to_string(), token);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok(headers)
    }

    /// Build headers for unauthenticated request
    pub fn build_public_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    /// Get client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Set access token manually (useful for testing or token refresh)
    pub fn set_token(&mut self, token: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs();

        self.token_expiry = Some(now + 86400); // 24 hours
        self.access_token = Some(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_auth() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("1000000123"); // client_id

        let auth = DhanAuth::new(&credentials).unwrap();
        assert_eq!(auth.client_id(), "1000000123");
    }

    #[test]
    fn test_token_expiry() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("1000000123");

        let auth = DhanAuth::new(&credentials).unwrap();
        assert!(auth.is_token_expired()); // No token set
    }

    #[test]
    fn test_set_token() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("1000000123");

        let mut auth = DhanAuth::new(&credentials).unwrap();
        auth.set_token("test_jwt_token".to_string());

        assert!(!auth.is_token_expired());
    }

    #[test]
    fn test_public_headers() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("1000000123");

        let auth = DhanAuth::new(&credentials).unwrap();
        let headers = auth.build_public_headers();

        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
}
