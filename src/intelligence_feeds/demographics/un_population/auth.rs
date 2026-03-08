//! UN Population authentication
//!
//! Authentication type: None (public API)
//!
//! The UN Population API is completely public and requires no authentication.

/// UN Population authentication credentials
#[derive(Clone, Debug)]
pub struct UnPopAuth;

impl UnPopAuth {
    /// Create new auth (no credentials needed)
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnPopAuth {
    fn default() -> Self {
        Self::new()
    }
}
