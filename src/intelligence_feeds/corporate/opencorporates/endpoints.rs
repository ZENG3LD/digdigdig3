//! OpenCorporates API endpoints

/// Base URLs for OpenCorporates API
pub struct OpenCorporatesEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OpenCorporatesEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.opencorporates.com/v0.4",
            ws_base: None, // OpenCorporates does not support WebSocket
        }
    }
}

/// OpenCorporates API endpoint enum
#[derive(Debug, Clone)]
pub enum OpenCorporatesEndpoint {
    /// Search companies
    CompaniesSearch,
    /// Get specific company
    Company { jurisdiction: String, company_number: String },
    /// Search officers
    OfficersSearch,
    /// Get company officers
    CompanyOfficers { jurisdiction: String, company_number: String },
    /// Get company filings
    CompanyFilings { jurisdiction: String, company_number: String },
    /// Search corporate groupings
    CorporateGroupingsSearch,
    /// List jurisdictions
    Jurisdictions,
}

impl OpenCorporatesEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::CompaniesSearch => "/companies/search".to_string(),
            Self::Company { jurisdiction, company_number } => {
                format!("/companies/{}/{}", jurisdiction, company_number)
            }
            Self::OfficersSearch => "/officers/search".to_string(),
            Self::CompanyOfficers { jurisdiction, company_number } => {
                format!("/companies/{}/{}/officers", jurisdiction, company_number)
            }
            Self::CompanyFilings { jurisdiction, company_number } => {
                format!("/companies/{}/{}/filings", jurisdiction, company_number)
            }
            Self::CorporateGroupingsSearch => "/corporate_groupings/search".to_string(),
            Self::Jurisdictions => "/jurisdictions".to_string(),
        }
    }
}
