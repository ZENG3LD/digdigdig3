//! GLEIF API endpoints

/// Base URLs for GLEIF API
pub struct GleifEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for GleifEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.gleif.org/api/v1",
            ws_base: None, // GLEIF does not support WebSocket
        }
    }
}

/// GLEIF API endpoint enum
#[derive(Debug, Clone)]
pub enum GleifEndpoint {
    /// Get LEI record by LEI code
    /// GET /lei-records/{lei}
    LeiRecord { lei: String },

    /// Search by entity name
    /// GET /lei-records?filter[entity.legalName]={name}
    SearchByName,

    /// Get direct parent
    /// GET /lei-records/{lei}/direct-parent
    DirectParent { lei: String },

    /// Get ultimate parent
    /// GET /lei-records/{lei}/ultimate-parent
    UltimateParent { lei: String },

    /// Get direct children (subsidiaries)
    /// GET /lei-records/{lei}/direct-children
    DirectChildren { lei: String },

    /// Search by country
    /// GET /lei-records?filter[entity.legalAddress.country]={iso2}
    SearchByCountry,

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get relationship records (ownership/control hierarchy links)
    /// GET /relationship-records?filter[startNode.id]={lei}
    RelationshipRecords,
    /// Map a BIC (Bank Identifier Code) to an LEI
    /// GET /lei-records?filter[bic]={bic}
    BicMaps,
    /// Get reporting exceptions for a specific LEI
    /// GET /reporting-exceptions?filter[LEI]={lei}
    ReportingExceptions { lei: String },
}

impl GleifEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::LeiRecord { lei } => format!("/lei-records/{}", lei),
            Self::SearchByName => "/lei-records".to_string(),
            Self::DirectParent { lei } => format!("/lei-records/{}/direct-parent", lei),
            Self::UltimateParent { lei } => format!("/lei-records/{}/ultimate-parent", lei),
            Self::DirectChildren { lei } => format!("/lei-records/{}/direct-children", lei),
            Self::SearchByCountry => "/lei-records".to_string(),

            // C7 additions
            Self::RelationshipRecords => "/relationship-records".to_string(),
            Self::BicMaps => "/lei-records".to_string(),
            Self::ReportingExceptions { .. } => "/reporting-exceptions".to_string(),
        }
    }
}
