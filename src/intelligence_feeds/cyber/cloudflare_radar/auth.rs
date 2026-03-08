//! Cloudflare Radar authentication
//!
//! Authentication type: Bearer Token
//!
//! Cloudflare Radar uses bearer token authentication via Authorization header.

use std::collections::HashMap;

/// Cloudflare Radar authentication credentials
#[derive(Clone)]
pub struct CloudflareRadarAuth {
    pub token: Option<String>,
}

impl CloudflareRadarAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `CLOUDFLARE_RADAR_TOKEN`
    pub fn from_env() -> Self {
        Self {
            token: std::env::var("CLOUDFLARE_RADAR_TOKEN").ok(),
        }
    }

    /// Create auth with explicit token
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: Some(token.into()),
        }
    }

    /// Add authentication to headers
    ///
    /// Cloudflare Radar requires Bearer token in Authorization header:
    /// `Authorization: Bearer YOUR_TOKEN`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Get token (for debugging/logging - use carefully)
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}

impl Default for CloudflareRadarAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
