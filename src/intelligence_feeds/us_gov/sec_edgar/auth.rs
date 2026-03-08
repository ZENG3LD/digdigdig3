//! SEC EDGAR authentication
//!
//! Authentication type: User-Agent Header (required)
//!
//! SEC EDGAR does not require API keys, but REQUIRES a User-Agent header
//! with your company name and email address. Requests without proper
//! User-Agent will be blocked.
//!
//! Format: "CompanyName email@example.com"
//! Example: "NemoTrading contact@nemotrading.com"

use reqwest::header::HeaderMap;

/// SEC EDGAR authentication credentials
#[derive(Clone)]
pub struct SecEdgarAuth {
    pub user_agent: String,
}

impl SecEdgarAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `SEC_EDGAR_USER_AGENT`
    /// Format: "CompanyName email@example.com"
    pub fn from_env() -> Self {
        let user_agent = std::env::var("SEC_EDGAR_USER_AGENT")
            .unwrap_or_else(|_| "NemoTrading contact@example.com".to_string());
        
        Self { user_agent }
    }

    /// Create auth with explicit User-Agent string
    ///
    /// # Arguments
    /// * `user_agent` - User-Agent string in format "CompanyName email@example.com"
    pub fn new(user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: user_agent.into(),
        }
    }

    /// Add authentication to request headers
    ///
    /// SEC EDGAR requires User-Agent header on all requests
    pub fn sign_headers(&self, headers: &mut HeaderMap) {
        headers.insert(
            reqwest::header::USER_AGENT,
            self.user_agent.parse().unwrap_or_else(|_| {
                reqwest::header::HeaderValue::from_static("NemoTrading contact@example.com")
            }),
        );
    }

    /// Get User-Agent string
    pub fn get_user_agent(&self) -> &str {
        &self.user_agent
    }
}

impl Default for SecEdgarAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
