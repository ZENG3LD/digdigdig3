//! UK Companies House API endpoints

/// Base URLs for UK Companies House API
pub struct UkCompaniesHouseEndpoints {
    pub rest_base: &'static str,
}

impl Default for UkCompaniesHouseEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.company-information.service.gov.uk",
        }
    }
}

/// UK Companies House API endpoint enum
#[derive(Debug, Clone)]
pub enum UkCompaniesHouseEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // SEARCH ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Search for companies by name
    SearchCompanies,

    // ═══════════════════════════════════════════════════════════════════════
    // COMPANY ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get company profile by company number
    Company { company_number: String },
    /// Get company officers (directors, secretaries)
    CompanyOfficers { company_number: String },
    /// Get persons with significant control (beneficial owners)
    CompanyPsc { company_number: String },
    /// Get filing history
    CompanyFilingHistory { company_number: String },
    /// Get charges/mortgages
    CompanyCharges { company_number: String },
    /// Get insolvency information
    CompanyInsolvency { company_number: String },

    // ═══════════════════════════════════════════════════════════════════════
    // OFFICER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all appointments for an officer (cross-company)
    OfficerAppointments { officer_id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Search across all entity types (companies, officers, disqualified directors)
    SearchAll,
    /// Search for officers (directors, secretaries) by name
    SearchOfficers,
    /// Search disqualified directors
    SearchDisqualified,
    /// Get disqualification details for a company or officer
    Disqualifications { officer_id: String },
    /// Get charges/mortgages detail for a specific charge
    ChargeDetail { company_number: String, charge_id: String },
    /// Get a specific filing document metadata
    FilingDetail { company_number: String, transaction_id: String },
    /// Get details for a specific PSC (Person with Significant Control) statement
    PscDetail { company_number: String, statement_id: String },
}

impl UkCompaniesHouseEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Search
            Self::SearchCompanies => "/search/companies".to_string(),

            // Company
            Self::Company { company_number } => {
                format!("/company/{}", company_number)
            }
            Self::CompanyOfficers { company_number } => {
                format!("/company/{}/officers", company_number)
            }
            Self::CompanyPsc { company_number } => {
                format!("/company/{}/persons-with-significant-control", company_number)
            }
            Self::CompanyFilingHistory { company_number } => {
                format!("/company/{}/filing-history", company_number)
            }
            Self::CompanyCharges { company_number } => {
                format!("/company/{}/charges", company_number)
            }
            Self::CompanyInsolvency { company_number } => {
                format!("/company/{}/insolvency", company_number)
            }

            // Officers
            Self::OfficerAppointments { officer_id } => {
                format!("/officers/{}/appointments", officer_id)
            }

            // C7 additions
            Self::SearchAll => "/search".to_string(),
            Self::SearchOfficers => "/search/officers".to_string(),
            Self::SearchDisqualified => "/search/disqualified-officers".to_string(),
            Self::Disqualifications { officer_id } => {
                format!("/disqualified-officers/natural/{}", officer_id)
            }
            Self::ChargeDetail { company_number, charge_id } => {
                format!("/company/{}/charges/{}", company_number, charge_id)
            }
            Self::FilingDetail { company_number, transaction_id } => {
                format!("/company/{}/filing-history/{}", company_number, transaction_id)
            }
            Self::PscDetail { company_number, statement_id } => {
                format!(
                    "/company/{}/persons-with-significant-control-statements/{}",
                    company_number, statement_id
                )
            }
        }
    }
}
