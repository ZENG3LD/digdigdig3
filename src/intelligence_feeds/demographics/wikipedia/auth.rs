//! Wikipedia Pageviews authentication
//!
//! Authentication type: None (User-Agent header only)
//!
//! Wikipedia Pageviews API is completely free and doesn't require authentication.
//! Only a User-Agent header is required to identify the client.

use std::collections::HashMap;

/// Wikipedia Pageviews authentication (User-Agent only)
#[derive(Clone)]
pub struct WikipediaAuth {
    pub user_agent: String,
}

impl WikipediaAuth {
    /// Create new auth with default User-Agent
    pub fn new() -> Self {
        Self {
            user_agent: "NemoTradingBot/1.0".to_string(),
        }
    }

    /// Create auth from environment variable (for consistency with other connectors)
    ///
    /// If WIKIPEDIA_USER_AGENT is set, uses it. Otherwise uses default.
    pub fn from_env() -> Self {
        Self {
            user_agent: std::env::var("WIKIPEDIA_USER_AGENT")
                .unwrap_or_else(|_| "NemoTradingBot/1.0".to_string()),
        }
    }

    /// Create auth with custom User-Agent
    pub fn with_user_agent(user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: user_agent.into(),
        }
    }

    /// Add User-Agent header to request headers
    pub fn add_headers(&self, headers: &mut HashMap<String, String>) {
        headers.insert("User-Agent".to_string(), self.user_agent.clone());
    }

    /// Get User-Agent string
    pub fn get_user_agent(&self) -> &str {
        &self.user_agent
    }
}

impl Default for WikipediaAuth {
    fn default() -> Self {
        Self::new()
    }
}
