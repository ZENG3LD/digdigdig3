//! OpenSanctions API endpoints

/// Base URLs for OpenSanctions API
pub struct OpenSanctionsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OpenSanctionsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.opensanctions.org",
            ws_base: None, // OpenSanctions does not support WebSocket
        }
    }
}

/// OpenSanctions API endpoint enum
#[derive(Debug, Clone)]
pub enum OpenSanctionsEndpoint {
    /// Search entities
    Search,
    /// Get entity details by ID
    Entity,
    /// Match entity (POST)
    Match,
    /// List all datasets
    Datasets,
    /// Get dataset details
    Dataset,
    /// List collections
    Collections,
}

impl OpenSanctionsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Search => "/search/default",
            Self::Entity => "/entities", // ID appended in connector
            Self::Match => "/match/default",
            Self::Datasets => "/datasets",
            Self::Dataset => "/datasets", // name appended in connector
            Self::Collections => "/collections",
        }
    }
}
