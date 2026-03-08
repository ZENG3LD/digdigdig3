//! USASpending.gov API endpoints

/// Base URLs for USASpending.gov API
pub struct UsaSpendingEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for UsaSpendingEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.usaspending.gov/api/v2",
            ws_base: None, // USASpending.gov does not support WebSocket
        }
    }
}

/// USASpending.gov API endpoint enum
#[derive(Debug, Clone)]
pub enum UsaSpendingEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // SPENDING ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Spending explorer endpoint (POST)
    SpendingExplorer,
    /// Award spending search endpoint (POST)
    AwardSearch,
    /// State spending data
    StateSpending,
    /// Specific state spending by FIPS code
    StateSpecificSpending { fips: String },

    // ═══════════════════════════════════════════════════════════════════════
    // REFERENCE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Federal agencies list
    Agencies,
    /// Glossary of terms
    Glossary,

    // ═══════════════════════════════════════════════════════════════════════
    // AWARD ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Bulk download awards (POST)
    BulkDownloadAwards,
    /// Federal account award counts
    FederalAccountAwardCounts,

    // ═══════════════════════════════════════════════════════════════════════
    // RECIPIENT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Recipient (DUNS) lookup
    RecipientDuns,
}

impl UsaSpendingEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Spending endpoints
            Self::SpendingExplorer => "/spending/explorer/".to_string(),
            Self::AwardSearch => "/search/spending_by_award/".to_string(),
            Self::StateSpending => "/spending/state/".to_string(),
            Self::StateSpecificSpending { fips } => format!("/spending/state/{}/", fips),

            // Reference endpoints
            Self::Agencies => "/references/agency/".to_string(),
            Self::Glossary => "/references/glossary/".to_string(),

            // Award endpoints
            Self::BulkDownloadAwards => "/bulk_download/awards/".to_string(),
            Self::FederalAccountAwardCounts => "/awards/count/federal_account/".to_string(),

            // Recipient endpoints
            Self::RecipientDuns => "/recipient/duns/".to_string(),
        }
    }
}
