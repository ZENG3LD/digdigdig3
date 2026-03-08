//! BLS authentication
//!
//! Authentication type: API Key (optional, JSON body)
//!
//! BLS v2 API uses optional API key authentication. The key is sent in the JSON request body
//! as "registrationkey". Without a key, rate limits are lower (25 queries/day vs 500/day).


/// BLS authentication credentials
#[derive(Clone)]
pub struct BlsAuth {
    pub api_key: Option<String>,
}

impl BlsAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `BLS_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("BLS_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (public access with lower rate limits)
    pub fn public() -> Self {
        Self { api_key: None }
    }

    /// Add authentication to request body
    ///
    /// BLS v2 requires API key in the JSON body:
    /// `{"registrationkey": "YOUR_KEY", ...}`
    pub fn sign_body(&self, body: &mut serde_json::Map<String, serde_json::Value>) {
        if let Some(key) = &self.api_key {
            body.insert(
                "registrationkey".to_string(),
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

impl Default for BlsAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
