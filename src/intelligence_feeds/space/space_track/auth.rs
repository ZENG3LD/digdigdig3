//! Space-Track authentication
//!
//! Authentication type: Username/Password with session cookies
//!
//! Space-Track uses a login endpoint that returns a session cookie.
//! Subsequent requests use the cookie for authentication.

/// Space-Track authentication credentials
#[derive(Clone)]
pub struct SpaceTrackAuth {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl SpaceTrackAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variables: `SPACE_TRACK_USERNAME` and `SPACE_TRACK_PASSWORD`
    pub fn from_env() -> Self {
        Self {
            username: std::env::var("SPACE_TRACK_USERNAME").ok(),
            password: std::env::var("SPACE_TRACK_PASSWORD").ok(),
        }
    }

    /// Create auth with explicit credentials
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: Some(username.into()),
            password: Some(password.into()),
        }
    }

    /// Generate login request body
    ///
    /// Space-Track login expects: identity=email&password=pass
    pub fn login_body(&self) -> Option<String> {
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => {
                Some(format!("identity={}&password={}", user, pass))
            }
            _ => None,
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }

    /// Get username (for debugging/logging - use carefully)
    pub fn get_username(&self) -> Option<&str> {
        self.username.as_deref()
    }
}

impl Default for SpaceTrackAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
