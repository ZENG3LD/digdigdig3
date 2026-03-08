//! UK Parliament API endpoints

/// Base URLs for UK Parliament API
pub struct UkParliamentEndpoints {
    pub members_base: &'static str,
    pub bills_base: &'static str,
}

impl Default for UkParliamentEndpoints {
    fn default() -> Self {
        Self {
            members_base: "https://members-api.parliament.uk/api",
            bills_base: "https://bills-api.parliament.uk/api/v1",
        }
    }
}

/// UK Parliament API endpoint enum
#[derive(Debug, Clone)]
pub enum UkParliamentEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // MEMBERS ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Search members by name
    MembersSearch,
    /// Get member details by ID
    Member,
    /// Get member voting record
    MemberVoting,

    // ═══════════════════════════════════════════════════════════════════════
    // BILLS ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Search bills
    Bills,
    /// Get bill details by ID
    Bill,
    /// Get bill stages/progress
    BillStages,

    // ═══════════════════════════════════════════════════════════════════════
    // CONSTITUENCY ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Search constituencies
    ConstituencySearch,
}

impl UkParliamentEndpoint {
    /// Get endpoint path and base URL
    ///
    /// Returns (base_url, path) tuple
    pub fn endpoint(&self) -> (&'static str, &'static str) {
        match self {
            // Members endpoints (members_base)
            Self::MembersSearch => ("members", "/Members/Search"),
            Self::Member => ("members", "/Members"),
            Self::MemberVoting => ("members", "/Members"),

            // Bills endpoints (bills_base)
            Self::Bills => ("bills", "/Bills"),
            Self::Bill => ("bills", "/Bills"),
            Self::BillStages => ("bills", "/Bills"),

            // Constituency endpoints (members_base)
            Self::ConstituencySearch => ("members", "/Location/Constituency/Search"),
        }
    }

    /// Get full path with ID parameter
    pub fn path_with_id(&self, id: u32) -> String {
        match self {
            Self::Member => format!("/Members/{}", id),
            Self::MemberVoting => format!("/Members/{}/Voting", id),
            Self::Bill => format!("/Bills/{}", id),
            Self::BillStages => format!("/Bills/{}/Stages", id),
            _ => self.endpoint().1.to_string(),
        }
    }
}
