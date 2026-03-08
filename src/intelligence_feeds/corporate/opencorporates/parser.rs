//! OpenCorporates response parsers
//!
//! Parse JSON responses to domain types based on OpenCorporates API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

// ═══════════════════════════════════════════════════════════════════════════
// OPENCORPORATES-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// OpenCorporates company data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcCompany {
    pub company_number: String,
    pub name: String,
    pub jurisdiction_code: String,
    pub incorporation_date: Option<String>,
    pub dissolution_date: Option<String>,
    pub company_type: Option<String>,
    pub registry_url: Option<String>,
    pub current_status: Option<String>,
    pub registered_address: Option<String>,
    pub officers_count: Option<u32>,
}

/// OpenCorporates officer/director data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcOfficer {
    pub id: Option<String>,
    pub name: String,
    pub position: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub nationality: Option<String>,
    pub occupation: Option<String>,
    pub company: OcCompanyRef,
}

/// OpenCorporates company reference (nested in officer data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcCompanyRef {
    pub company_number: String,
    pub name: String,
    pub jurisdiction_code: String,
}

/// OpenCorporates filing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcFiling {
    pub id: Option<String>,
    pub title: String,
    pub date: Option<String>,
    pub description: Option<String>,
    pub filing_type: Option<String>,
    pub url: Option<String>,
}

/// OpenCorporates search result wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct OcSearchResult<T> {
    pub total_count: u32,
    pub total_pages: u32,
    pub page: u32,
    pub per_page: u32,
    pub results: Vec<T>,
}

// ═══════════════════════════════════════════════════════════════════════════
// PARSER IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

pub struct OcParser;

impl OcParser {
    /// Parse companies search response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": {
    ///     "companies": [
    ///       {
    ///         "company": {
    ///           "company_number": "12345678",
    ///           "name": "EXAMPLE LTD",
    ///           "jurisdiction_code": "gb",
    ///           "incorporation_date": "2020-01-15",
    ///           "dissolution_date": null,
    ///           "company_type": "Private Limited Company",
    ///           "registry_url": "https://...",
    ///           "current_status": "Active",
    ///           "registered_address_in_full": "123 Street, City",
    ///           "officers_count": 3
    ///         }
    ///       }
    ///     ],
    ///     "total_count": 100,
    ///     "total_pages": 10,
    ///     "page": 1,
    ///     "per_page": 10
    ///   }
    /// }
    /// ```
    pub fn parse_companies_search(response: &Value) -> ExchangeResult<OcSearchResult<OcCompany>> {
        let results = response
            .get("results")
            .ok_or_else(|| ExchangeError::Parse("Missing 'results'".to_string()))?;

        let total_count = Self::get_u32(results, "total_count").unwrap_or(0);
        let total_pages = Self::get_u32(results, "total_pages").unwrap_or(0);
        let page = Self::get_u32(results, "page").unwrap_or(1);
        let per_page = Self::get_u32(results, "per_page").unwrap_or(10);

        let companies_array = results
            .get("companies")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'companies' array".to_string()))?;

        let companies: Result<Vec<OcCompany>, ExchangeError> = companies_array
            .iter()
            .filter_map(|item| item.get("company"))
            .map(Self::parse_company)
            .collect();

        Ok(OcSearchResult {
            total_count,
            total_pages,
            page,
            per_page,
            results: companies?,
        })
    }

    /// Parse single company response
    pub fn parse_company_response(response: &Value) -> ExchangeResult<OcCompany> {
        let results = response
            .get("results")
            .ok_or_else(|| ExchangeError::Parse("Missing 'results'".to_string()))?;

        let company_data = results
            .get("company")
            .ok_or_else(|| ExchangeError::Parse("Missing 'company'".to_string()))?;

        Self::parse_company(company_data)
    }

    /// Parse single company object
    fn parse_company(company: &Value) -> ExchangeResult<OcCompany> {
        Ok(OcCompany {
            company_number: Self::require_str(company, "company_number")?.to_string(),
            name: Self::require_str(company, "name")?.to_string(),
            jurisdiction_code: Self::require_str(company, "jurisdiction_code")?.to_string(),
            incorporation_date: Self::get_str(company, "incorporation_date").map(|s| s.to_string()),
            dissolution_date: Self::get_str(company, "dissolution_date").map(|s| s.to_string()),
            company_type: Self::get_str(company, "company_type").map(|s| s.to_string()),
            registry_url: Self::get_str(company, "registry_url").map(|s| s.to_string()),
            current_status: Self::get_str(company, "current_status").map(|s| s.to_string()),
            registered_address: Self::get_str(company, "registered_address_in_full").map(|s| s.to_string()),
            officers_count: Self::get_u32(company, "officers_count"),
        })
    }

    /// Parse officers search response
    pub fn parse_officers_search(response: &Value) -> ExchangeResult<OcSearchResult<OcOfficer>> {
        let results = response
            .get("results")
            .ok_or_else(|| ExchangeError::Parse("Missing 'results'".to_string()))?;

        let total_count = Self::get_u32(results, "total_count").unwrap_or(0);
        let total_pages = Self::get_u32(results, "total_pages").unwrap_or(0);
        let page = Self::get_u32(results, "page").unwrap_or(1);
        let per_page = Self::get_u32(results, "per_page").unwrap_or(10);

        let officers_array = results
            .get("officers")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'officers' array".to_string()))?;

        let officers: Result<Vec<OcOfficer>, ExchangeError> = officers_array
            .iter()
            .filter_map(|item| item.get("officer"))
            .map(Self::parse_officer)
            .collect();

        Ok(OcSearchResult {
            total_count,
            total_pages,
            page,
            per_page,
            results: officers?,
        })
    }

    /// Parse company officers response
    pub fn parse_company_officers(response: &Value) -> ExchangeResult<Vec<OcOfficer>> {
        let results = response
            .get("results")
            .ok_or_else(|| ExchangeError::Parse("Missing 'results'".to_string()))?;

        let officers_array = results
            .get("officers")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'officers' array".to_string()))?;

        let officers: Result<Vec<OcOfficer>, ExchangeError> = officers_array
            .iter()
            .filter_map(|item| item.get("officer"))
            .map(Self::parse_officer)
            .collect();

        officers
    }

    /// Parse single officer object
    fn parse_officer(officer: &Value) -> ExchangeResult<OcOfficer> {
        let company_data = officer
            .get("company")
            .ok_or_else(|| ExchangeError::Parse("Missing 'company' in officer".to_string()))?;

        Ok(OcOfficer {
            id: Self::get_str(officer, "id").map(|s| s.to_string()),
            name: Self::require_str(officer, "name")?.to_string(),
            position: Self::get_str(officer, "position").map(|s| s.to_string()),
            start_date: Self::get_str(officer, "start_date").map(|s| s.to_string()),
            end_date: Self::get_str(officer, "end_date").map(|s| s.to_string()),
            nationality: Self::get_str(officer, "nationality").map(|s| s.to_string()),
            occupation: Self::get_str(officer, "occupation").map(|s| s.to_string()),
            company: OcCompanyRef {
                company_number: Self::require_str(company_data, "company_number")?.to_string(),
                name: Self::require_str(company_data, "name")?.to_string(),
                jurisdiction_code: Self::require_str(company_data, "jurisdiction_code")?.to_string(),
            },
        })
    }

    /// Parse company filings response
    pub fn parse_company_filings(response: &Value) -> ExchangeResult<Vec<OcFiling>> {
        let results = response
            .get("results")
            .ok_or_else(|| ExchangeError::Parse("Missing 'results'".to_string()))?;

        let filings_array = results
            .get("filings")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'filings' array".to_string()))?;

        let filings: Result<Vec<OcFiling>, ExchangeError> = filings_array
            .iter()
            .filter_map(|item| item.get("filing"))
            .map(Self::parse_filing)
            .collect();

        filings
    }

    /// Parse single filing object
    fn parse_filing(filing: &Value) -> ExchangeResult<OcFiling> {
        Ok(OcFiling {
            id: Self::get_str(filing, "id").map(|s| s.to_string()),
            title: Self::require_str(filing, "title")?.to_string(),
            date: Self::get_str(filing, "date").map(|s| s.to_string()),
            description: Self::get_str(filing, "description").map(|s| s.to_string()),
            filing_type: Self::get_str(filing, "filing_type").map(|s| s.to_string()),
            url: Self::get_str(filing, "url").map(|s| s.to_string()),
        })
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .or_else(|| error.get("message").and_then(|v| v.as_str()))
                .unwrap_or("Unknown error")
                .to_string();
            return Err(ExchangeError::Api { code: 0, message });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|v| v as u32)
    }
}
