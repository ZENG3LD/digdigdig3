//! Semantic Scholar authentication
//!
//! Authentication type: API Key (header, optional)
//!
//! Semantic Scholar uses optional API key authentication via x-api-key header.
//! Higher rate limits with API key (1 req/sec vs 100 req/5 min).

use std::collections::HashMap;

/// Semantic Scholar authentication credentials
#[derive(Clone)]
pub struct SemanticScholarAuth {
    pub api_key: Option<String>,
}

impl SemanticScholarAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `SEMANTIC_SCHOLAR_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (lower rate limits)
    pub fn unauthenticated() -> Self {
        Self { api_key: None }
    }

    /// Add authentication to headers
    ///
    /// Semantic Scholar requires API key as a header:
    /// `x-api-key: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("x-api-key".to_string(), key.clone());
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

impl Default for SemanticScholarAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
