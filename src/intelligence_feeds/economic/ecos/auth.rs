//! Bank of Korea ECOS authentication
//!
//! Authentication type: API Key (embedded in URL path)
//!
//! ECOS uses API key authentication where the key is part of the URL path structure.
//! Unlike FRED which uses query parameters, ECOS embeds the key in the path:
//! /{service}/{api_key}/{format}/{lang}/...

/// ECOS authentication credentials
#[derive(Clone)]
pub struct EcosAuth {
    pub api_key: Option<String>,
}

impl EcosAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `ECOS_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ECOS_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Get API key for embedding in URL path
    ///
    /// ECOS requires API key as part of the URL path:
    /// `/{service}/{api_key}/json/en/...`
    pub fn get_api_key(&self) -> crate::core::types::ExchangeResult<&str> {
        self.api_key
            .as_deref()
            .ok_or_else(|| crate::core::types::ExchangeError::Auth(
                "ECOS API key not configured. Set ECOS_API_KEY environment variable.".to_string()
            ))
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }
}

impl Default for EcosAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
