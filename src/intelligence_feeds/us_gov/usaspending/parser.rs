//! USASpending.gov response parsers
//!
//! Parse JSON responses to domain types based on USASpending.gov API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct UsaSpendingParser;

impl UsaSpendingParser {
    // ═══════════════════════════════════════════════════════════════════════
    // USASPENDING-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse award data from search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [{
    ///     "Award ID": "CONT_AWD_12345",
    ///     "Recipient Name": "ACME Corporation",
    ///     "Award Amount": "1000000.00",
    ///     "Awarding Agency": "Department of Defense",
    ///     "Description": "IT Services",
    ///     "Start Date": "2023-01-01"
    ///   }]
    /// }
    /// ```
    pub fn parse_awards(response: &Value) -> ExchangeResult<Vec<UsaSpendingAward>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|award| {
                Ok(UsaSpendingAward {
                    award_id: Self::get_str(award, "Award ID")
                        .or_else(|| Self::get_str(award, "generated_unique_award_id"))
                        .map(|s| s.to_string()),
                    recipient: Self::get_str(award, "Recipient Name")
                        .or_else(|| Self::get_str(award, "recipient_name"))
                        .map(|s| s.to_string()),
                    amount: Self::get_f64(award, "Award Amount")
                        .or_else(|| Self::get_f64(award, "total_obligation"))
                        .or_else(|| {
                            Self::get_str(award, "Award Amount")
                                .or_else(|| Self::get_str(award, "total_obligation"))
                                .and_then(|s| s.parse::<f64>().ok())
                        }),
                    agency: Self::get_str(award, "Awarding Agency")
                        .or_else(|| Self::get_str(award, "awarding_agency_name"))
                        .map(|s| s.to_string()),
                    description: Self::get_str(award, "Description")
                        .or_else(|| Self::get_str(award, "description"))
                        .map(|s| s.to_string()),
                    date: Self::get_str(award, "Start Date")
                        .or_else(|| Self::get_str(award, "period_of_performance_start_date"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse agency data
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [{
    ///     "agency_id": "123",
    ///     "agency_name": "Department of Defense",
    ///     "budget_authority_amount": 750000000000.00,
    ///     "obligated_amount": 700000000000.00
    ///   }]
    /// }
    /// ```
    pub fn parse_agencies(response: &Value) -> ExchangeResult<Vec<UsaSpendingAgency>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|agency| {
                Ok(UsaSpendingAgency {
                    agency_id: Self::get_str(agency, "agency_id")
                        .or_else(|| Self::get_str(agency, "toptier_agency_id"))
                        .map(|s| s.to_string()),
                    name: Self::require_str(agency, "agency_name")
                        .or_else(|_| Self::require_str(agency, "name"))?
                        .to_string(),
                    budget_authority: Self::get_f64(agency, "budget_authority_amount")
                        .or_else(|| Self::get_f64(agency, "budgetary_resources")),
                    obligations: Self::get_f64(agency, "obligated_amount")
                        .or_else(|| Self::get_f64(agency, "total_obligations")),
                })
            })
            .collect()
    }

    /// Parse state spending data
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [{
    ///     "fips": "06",
    ///     "name": "California",
    ///     "total_prime_amount": 50000000000.00,
    ///     "population": 39500000,
    ///     "per_capita": 1265.82
    ///   }]
    /// }
    /// ```
    pub fn parse_state_spending(response: &Value) -> ExchangeResult<Vec<UsaSpendingState>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|state| {
                Ok(UsaSpendingState {
                    fips: Self::get_str(state, "fips")
                        .or_else(|| Self::get_str(state, "code"))
                        .map(|s| s.to_string()),
                    name: Self::require_str(state, "name")?.to_string(),
                    total_spending: Self::get_f64(state, "total_prime_amount")
                        .or_else(|| Self::get_f64(state, "total_obligations"))
                        .or_else(|| Self::get_f64(state, "amount")),
                    population: Self::get_i64(state, "population"),
                    per_capita: Self::get_f64(state, "per_capita")
                        .or_else(|| Self::get_f64(state, "per_capita_amount")),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Check for error field
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .or_else(|| error.get("message").and_then(|v| v.as_str()))
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
        }

        // Check for detail field (Django REST framework style)
        if let Some(detail) = response.get("detail") {
            let message = detail
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
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

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// USASPENDING-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// USASpending.gov award data
#[derive(Debug, Clone)]
pub struct UsaSpendingAward {
    pub award_id: Option<String>,
    pub recipient: Option<String>,
    pub amount: Option<f64>,
    pub agency: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>,
}

/// USASpending.gov agency data
#[derive(Debug, Clone)]
pub struct UsaSpendingAgency {
    pub agency_id: Option<String>,
    pub name: String,
    pub budget_authority: Option<f64>,
    pub obligations: Option<f64>,
}

/// USASpending.gov state spending data
#[derive(Debug, Clone)]
pub struct UsaSpendingState {
    pub fips: Option<String>,
    pub name: String,
    pub total_spending: Option<f64>,
    pub population: Option<i64>,
    pub per_capita: Option<f64>,
}
