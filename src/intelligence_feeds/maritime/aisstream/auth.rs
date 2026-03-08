//! AISStream.io authentication
//!
//! Authentication type: API Key (WebSocket message)
//!
//! AISStream uses API key authentication via the WebSocket subscription message.
//! The API key is sent as part of the initial subscription JSON.

use std::collections::HashMap;

/// AISStream.io authentication credentials
#[derive(Clone)]
pub struct AisStreamAuth {
    pub api_key: Option<String>,
}

impl AisStreamAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `AISSTREAM_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("AISSTREAM_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Add authentication to subscription message
    ///
    /// AISStream requires API key in the subscription JSON message:
    /// ```json
    /// {
    ///   "APIKey": "your-api-key-here",
    ///   "BoundingBoxes": [...]
    /// }
    /// ```
    pub fn sign_subscription(&self, subscription: &mut HashMap<String, serde_json::Value>) {
        if let Some(key) = &self.api_key {
            subscription.insert(
                "APIKey".to_string(),
                serde_json::Value::String(key.clone()),
            );
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get API key (for debugging/logging - use carefully)
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

impl Default for AisStreamAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
