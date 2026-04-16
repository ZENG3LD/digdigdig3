//! # Angel One SmartAPI Authentication
//!
//! Three-factor authentication implementation:
//! 1. Client Code (Angel One account ID)
//! 2. Client PIN (account password)
//! 3. TOTP (Time-based One-Time Password)
//!
//! ## Token Types
//! - **JWT Token**: REST API authorization (Bearer token)
//! - **Refresh Token**: Token renewal without re-login
//! - **Feed Token**: WebSocket authentication
//!
//! ## Session Management
//! - Sessions expire at midnight (market close)
//! - Tokens can be refreshed using refresh token
//! - No HMAC signature required (token-based only)

use std::collections::HashMap;
use std::net::UdpSocket;
use totp_rs::{Algorithm, TOTP};
use crate::core::{ExchangeResult, ExchangeError};

/// Angel One authentication handler
#[derive(Clone)]
pub struct AngelOneAuth {
    pub api_key: String,
    pub client_code: String,
    pub pin: String,
    pub totp_secret: String,

    // Session tokens (populated after login)
    pub jwt_token: Option<String>,
    pub refresh_token: Option<String>,
    pub feed_token: Option<String>,
}

impl AngelOneAuth {
    /// Create new auth handler
    pub fn new(
        api_key: String,
        client_code: String,
        pin: String,
        totp_secret: String,
    ) -> Self {
        Self {
            api_key,
            client_code,
            pin,
            totp_secret,
            jwt_token: None,
            refresh_token: None,
            feed_token: None,
        }
    }

    /// Generate current TOTP code
    ///
    /// Uses TOTP-RS library to generate 6-digit code based on current time
    pub fn generate_totp(&self) -> ExchangeResult<String> {
        // Create TOTP instance with standard parameters
        // Angel One uses: 6 digits, 30 second step, SHA1 algorithm
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,                           // 6 digits
            1,                           // 1 step skew
            30,                          // 30 second step
            self.totp_secret.as_bytes().to_vec(),
        ).map_err(|e| ExchangeError::Auth(format!("Failed to create TOTP: {}", e)))?;

        let code = totp.generate_current()
            .map_err(|e| ExchangeError::Auth(format!("Failed to generate TOTP: {}", e)))?;

        Ok(code)
    }

    /// Store session tokens after login
    pub fn set_tokens(&mut self, jwt: String, refresh: String, feed: String) {
        self.jwt_token = Some(jwt);
        self.refresh_token = Some(refresh);
        self.feed_token = Some(feed);
    }

    /// Clear session tokens (on logout)
    pub fn clear_tokens(&mut self) {
        self.jwt_token = None;
        self.refresh_token = None;
        self.feed_token = None;
    }

    /// Get JWT token (for REST API requests)
    pub fn jwt_token(&self) -> ExchangeResult<&str> {
        self.jwt_token.as_deref()
            .ok_or_else(|| ExchangeError::Auth("Not logged in - JWT token missing".to_string()))
    }

    /// Get refresh token (for token renewal)
    pub fn refresh_token(&self) -> ExchangeResult<&str> {
        self.refresh_token.as_deref()
            .ok_or_else(|| ExchangeError::Auth("Refresh token missing".to_string()))
    }

    /// Get feed token (for WebSocket authentication)
    pub fn feed_token(&self) -> ExchangeResult<&str> {
        self.feed_token.as_deref()
            .ok_or_else(|| ExchangeError::Auth("Feed token missing".to_string()))
    }

    /// Add authentication headers to request
    ///
    /// Required headers for authenticated endpoints:
    /// - Authorization: Bearer {jwt_token}
    /// - X-PrivateKey: {api_key}
    /// - X-ClientLocalIP: {client_local_ip}
    /// - X-ClientPublicIP: {client_public_ip}
    /// - X-MACAddress: {mac_address}
    /// - Content-Type: application/json
    pub fn sign_headers(&self) -> ExchangeResult<HashMap<String, String>> {
        let jwt = self.jwt_token()?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", jwt));
        headers.insert("X-PrivateKey".to_string(), self.api_key.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Detect local IP via routing trick: connect UDP socket to external address,
        // then read back the local address chosen by the OS. No packets are sent.
        let local_ip = UdpSocket::bind("0.0.0.0:0")
            .and_then(|sock| {
                sock.connect("8.8.8.8:80")?;
                sock.local_addr()
            })
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        // Use locally administered MAC address as safe default.
        // 02:00:00:00:00:00 is the standard locally-administered unicast placeholder.
        let mac_address = "02:00:00:00:00:00".to_string();

        headers.insert("X-ClientLocalIP".to_string(), local_ip.clone());
        headers.insert("X-ClientPublicIP".to_string(), local_ip);
        headers.insert("X-MACAddress".to_string(), mac_address);

        Ok(headers)
    }

    /// Build login request body
    pub fn build_login_body(&self) -> ExchangeResult<serde_json::Value> {
        let totp_code = self.generate_totp()?;

        Ok(serde_json::json!({
            "clientcode": self.client_code,
            "password": self.pin,
            "totp": totp_code
        }))
    }

    /// Build token refresh request body
    pub fn build_refresh_body(&self) -> ExchangeResult<serde_json::Value> {
        let refresh = self.refresh_token()?;

        Ok(serde_json::json!({
            "refreshToken": refresh
        }))
    }

    /// Build logout request body
    pub fn build_logout_body(&self) -> serde_json::Value {
        serde_json::json!({
            "clientId": self.client_code
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_auth() {
        let auth = AngelOneAuth::new(
            "test_key".to_string(),
            "A12345".to_string(),
            "1234".to_string(),
            "JBSWY3DPEHPK3PXP".to_string(), // Example TOTP secret
        );

        assert_eq!(auth.api_key, "test_key");
        assert_eq!(auth.client_code, "A12345");
        assert!(auth.jwt_token.is_none());
    }

    #[test]
    fn test_set_tokens() {
        let mut auth = AngelOneAuth::new(
            "test_key".to_string(),
            "A12345".to_string(),
            "1234".to_string(),
            "JBSWY3DPEHPK3PXP".to_string(),
        );

        auth.set_tokens(
            "jwt123".to_string(),
            "refresh123".to_string(),
            "feed123".to_string(),
        );

        assert_eq!(auth.jwt_token().unwrap(), "jwt123");
        assert_eq!(auth.refresh_token().unwrap(), "refresh123");
        assert_eq!(auth.feed_token().unwrap(), "feed123");
    }

    #[test]
    fn test_clear_tokens() {
        let mut auth = AngelOneAuth::new(
            "test_key".to_string(),
            "A12345".to_string(),
            "1234".to_string(),
            "JBSWY3DPEHPK3PXP".to_string(),
        );

        auth.set_tokens("jwt".to_string(), "refresh".to_string(), "feed".to_string());
        auth.clear_tokens();

        assert!(auth.jwt_token.is_none());
        assert!(auth.refresh_token.is_none());
        assert!(auth.feed_token.is_none());
    }
}
