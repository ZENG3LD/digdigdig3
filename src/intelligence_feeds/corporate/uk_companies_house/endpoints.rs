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
        }
    }
}
