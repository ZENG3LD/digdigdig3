//! Congress.gov API endpoints

/// Base URLs for Congress.gov API
pub struct CongressEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for CongressEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.congress.gov/v3",
            ws_base: None, // Congress.gov does not support WebSocket
        }
    }
}

/// Congress.gov API endpoint enum
#[derive(Debug, Clone)]
pub enum CongressEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // BILL ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List bills
    Bills,
    /// Get specific bill
    Bill,
    /// Get bill actions/timeline
    BillActions,
    /// Get bill cosponsors
    BillCosponsors,
    /// Get bill subjects
    BillSubjects,
    /// Get bill summaries
    BillSummaries,
    /// Get bill titles
    BillTitles,
    /// Get bill amendments
    BillAmendments,
    /// Get bill committees
    BillCommittees,
    /// Get bill related bills
    BillRelatedBills,
    /// Get bill text
    BillText,

    // ═══════════════════════════════════════════════════════════════════════
    // MEMBER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List members
    Members,
    /// Get specific member
    Member,
    /// Get member sponsored legislation
    MemberSponsoredLegislation,
    /// Get member cosponsored legislation
    MemberCosponsoredLegislation,

    // ═══════════════════════════════════════════════════════════════════════
    // COMMITTEE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List committees
    Committees,
    /// Get specific committee
    Committee,
    /// Get committee bills
    CommitteeBills,
    /// Get committee reports
    CommitteeReports,
    /// Get committee nominations
    CommitteeNominations,
    /// Get committee prints
    CommitteePrints,

    // ═══════════════════════════════════════════════════════════════════════
    // NOMINATION ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List nominations
    Nominations,
    /// Get specific nomination
    Nomination,
    /// Get nomination actions
    NominationActions,
    /// Get nomination committees
    NominationCommittees,
    /// Get nomination hearings
    NominationHearings,

    // ═══════════════════════════════════════════════════════════════════════
    // TREATY ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List treaties
    Treaties,
    /// Get specific treaty
    Treaty,
    /// Get treaty actions
    TreatyActions,
    /// Get treaty committees
    TreatyCommittees,

    // ═══════════════════════════════════════════════════════════════════════
    // CONGRESS ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List congresses
    Congresses,
    /// Get specific congress
    Congress,

    // ═══════════════════════════════════════════════════════════════════════
    // SUMMARIES ENDPOINT
    // ═══════════════════════════════════════════════════════════════════════
    /// List bill summaries
    Summaries,
}

impl CongressEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Bills
            Self::Bills => "/bill",
            Self::Bill => "/bill", // needs parameters
            Self::BillActions => "/bill", // needs parameters + /actions
            Self::BillCosponsors => "/bill", // needs parameters + /cosponsors
            Self::BillSubjects => "/bill", // needs parameters + /subjects
            Self::BillSummaries => "/bill", // needs parameters + /summaries
            Self::BillTitles => "/bill", // needs parameters + /titles
            Self::BillAmendments => "/bill", // needs parameters + /amendments
            Self::BillCommittees => "/bill", // needs parameters + /committees
            Self::BillRelatedBills => "/bill", // needs parameters + /relatedBills
            Self::BillText => "/bill", // needs parameters + /text

            // Members
            Self::Members => "/member",
            Self::Member => "/member", // needs bioguideId
            Self::MemberSponsoredLegislation => "/member", // needs bioguideId + /sponsored-legislation
            Self::MemberCosponsoredLegislation => "/member", // needs bioguideId + /cosponsored-legislation

            // Committees
            Self::Committees => "/committee",
            Self::Committee => "/committee", // needs chamber + systemCode
            Self::CommitteeBills => "/committee", // needs chamber + systemCode + /bills
            Self::CommitteeReports => "/committee", // needs chamber + systemCode + /reports
            Self::CommitteeNominations => "/committee", // needs chamber + systemCode + /nominations
            Self::CommitteePrints => "/committee", // needs chamber + systemCode + /prints

            // Nominations
            Self::Nominations => "/nomination",
            Self::Nomination => "/nomination", // needs congress + number
            Self::NominationActions => "/nomination", // needs congress + number + /actions
            Self::NominationCommittees => "/nomination", // needs congress + number + /committees
            Self::NominationHearings => "/nomination", // needs congress + number + /hearings

            // Treaties
            Self::Treaties => "/treaty",
            Self::Treaty => "/treaty", // needs congress + number
            Self::TreatyActions => "/treaty", // needs congress + number + /actions
            Self::TreatyCommittees => "/treaty", // needs congress + number + /committees

            // Congress
            Self::Congresses => "/congress",
            Self::Congress => "/congress", // needs congress number

            // Summaries
            Self::Summaries => "/summaries",
        }
    }
}

/// Format bill URL path
///
/// Congress.gov uses: /bill/{congress}/{type}/{number}
/// Example: /bill/118/hr/3076
pub fn format_bill_path(congress: u32, bill_type: &str, number: u32) -> String {
    format!("/bill/{}/{}/{}", congress, bill_type, number)
}

/// Format bill actions path
pub fn format_bill_actions_path(congress: u32, bill_type: &str, number: u32) -> String {
    format!("/bill/{}/{}/{}/actions", congress, bill_type, number)
}

/// Format bill cosponsors path
pub fn format_bill_cosponsors_path(congress: u32, bill_type: &str, number: u32) -> String {
    format!("/bill/{}/{}/{}/cosponsors", congress, bill_type, number)
}

/// Format bill subjects path
pub fn _format_bill_subjects_path(congress: u32, bill_type: &str, number: u32) -> String {
    format!("/bill/{}/{}/{}/subjects", congress, bill_type, number)
}

/// Format bill summaries path
pub fn format_bill_summaries_path(congress: u32, bill_type: &str, number: u32) -> String {
    format!("/bill/{}/{}/{}/summaries", congress, bill_type, number)
}

/// Format member path
pub fn format_member_path(bioguide_id: &str) -> String {
    format!("/member/{}", bioguide_id)
}

/// Format committee path
pub fn format_committee_path(chamber: &str, system_code: &str) -> String {
    format!("/committee/{}/{}", chamber, system_code)
}

/// Format nomination path
pub fn _format_nomination_path(congress: u32, number: u32) -> String {
    format!("/nomination/{}/{}", congress, number)
}

/// Format treaty path
pub fn _format_treaty_path(congress: u32, number: u32) -> String {
    format!("/treaty/{}/{}", congress, number)
}

/// Format congress path
pub fn format_congress_path(congress: u32) -> String {
    format!("/congress/{}", congress)
}
