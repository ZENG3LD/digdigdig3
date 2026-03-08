//! Launch Library 2 authentication
//!
//! Authentication type: None (free public API)
//!
//! Launch Library 2 is a free public API that does not require authentication
//! for basic access. This module provides a stub implementation for consistency
//! with other connectors.

use std::collections::HashMap;

/// Launch Library 2 authentication credentials (stub - no auth required)
#[derive(Clone)]
pub struct LaunchLibraryAuth {
    // No authentication fields needed
    _phantom: (),
}

impl LaunchLibraryAuth {
    /// Create new auth (stub)
    pub fn new() -> Self {
        Self {
            _phantom: (),
        }
    }

    /// Create auth from environment (stub)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Add authentication to query parameters (no-op for Launch Library 2)
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // No authentication required - this is a no-op
    }

    /// Check if authentication is configured (always true for public API)
    pub fn is_authenticated(&self) -> bool {
        true
    }
}

impl Default for LaunchLibraryAuth {
    fn default() -> Self {
        Self::new()
    }
}
