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

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get related entities (adjacency graph) for an entity
    EntityAdjacency { entity_id: String },
    /// Get individual statements that constitute an entity's data
    Statements,
    /// Reconciliation API (OpenRefine-compatible entity matching)
    ReconcileApi,
}

impl OpenSanctionsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Search => "/search/default".to_string(),
            Self::Entity => "/entities".to_string(), // ID appended in connector
            Self::Match => "/match/default".to_string(),
            Self::Datasets => "/datasets".to_string(),
            Self::Dataset => "/datasets".to_string(), // name appended in connector
            Self::Collections => "/collections".to_string(),

            // C7 additions
            Self::EntityAdjacency { entity_id } => format!("/entities/{}/adjacent", entity_id),
            Self::Statements => "/statements".to_string(),
            Self::ReconcileApi => "/reconcile/default".to_string(),
        }
    }
}
