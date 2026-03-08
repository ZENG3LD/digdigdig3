//! SAM.gov response parsers
//!
//! Parse JSON responses to domain types based on SAM.gov API response formats.

use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct SamGovParser;

impl SamGovParser {
    /// Parse entity search results
    ///
    /// Example response structure:
    /// ```json
    /// {
    ///   "totalRecords": 100,
    ///   "entityData": [
    ///     {
    ///       "entityRegistration": {
    ///         "ueiSAM": "...",
    ///         "cageCode": "...",
    ///         "legalBusinessName": "...",
    ///         "dbaName": "...",
    ///         "registrationStatus": "Active",
    ///         "registrationDate": "...",
    ///         "expirationDate": "...",
    ///         "physicalAddress": {...},
    ///         "entityType": "...",
    ///         "naicsCodes": [...]
    ///       }
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_entities(response: &Value) -> ExchangeResult<Vec<SamEntity>> {
        let entity_data = response
            .get("entityData")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'entityData' array".to_string()))?;

        entity_data
            .iter()
            .map(|item| {
                let reg = item
                    .get("entityRegistration")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'entityRegistration'".to_string()))?;

                Ok(SamEntity {
                    uei: Self::get_str(reg, "ueiSAM").map(|s| s.to_string()),
                    cage_code: Self::get_str(reg, "cageCode").map(|s| s.to_string()),
                    legal_business_name: Self::get_str(reg, "legalBusinessName").map(|s| s.to_string()),
                    dba_name: Self::get_str(reg, "dbaName").map(|s| s.to_string()),
                    physical_address: Self::parse_address(reg.get("physicalAddress")),
                    entity_type: Self::get_str(reg, "entityType").map(|s| s.to_string()),
                    registration_status: Self::get_str(reg, "registrationStatus").map(|s| s.to_string()),
                    naics_codes: Self::parse_naics_codes(reg.get("naicsCodes")),
                    sam_registration_date: Self::get_str(reg, "registrationDate").map(|s| s.to_string()),
                    expiration_date: Self::get_str(reg, "expirationDate").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single entity (for get by UEI)
    pub fn parse_entity(response: &Value) -> ExchangeResult<SamEntity> {
        let entities = Self::parse_entities(response)?;
        entities
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse("No entity found".to_string()))
    }

    /// Parse contract opportunities
    ///
    /// Example response structure:
    /// ```json
    /// {
    ///   "totalRecords": 50,
    ///   "opportunitiesData": [
    ///     {
    ///       "noticeId": "...",
    ///       "title": "...",
    ///       "solicitationNumber": "...",
    ///       "department": "...",
    ///       "subTier": "...",
    ///       "office": "...",
    ///       "postedDate": "...",
    ///       "responseDeadLine": "...",
    ///       "type": "...",
    ///       "description": "...",
    ///       "placeOfPerformance": {...},
    ///       "award": {"amount": "..."}
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_opportunities(response: &Value) -> ExchangeResult<Vec<SamOpportunity>> {
        let opportunities_data = response
            .get("opportunitiesData")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'opportunitiesData' array".to_string()))?;

        Ok(opportunities_data
            .iter()
            .map(|opp| {
                SamOpportunity {
                    notice_id: Self::get_str(opp, "noticeId").map(|s| s.to_string()),
                    title: Self::get_str(opp, "title").map(|s| s.to_string()),
                    solicitation_number: Self::get_str(opp, "solicitationNumber").map(|s| s.to_string()),
                    department: Self::get_str(opp, "department").map(|s| s.to_string()),
                    sub_tier: Self::get_str(opp, "subTier").map(|s| s.to_string()),
                    office: Self::get_str(opp, "office").map(|s| s.to_string()),
                    posted_date: Self::get_str(opp, "postedDate").map(|s| s.to_string()),
                    response_deadline: Self::get_str(opp, "responseDeadLine").map(|s| s.to_string()),
                    type_name: Self::get_str(opp, "type").map(|s| s.to_string()),
                    description: Self::get_str(opp, "description").map(|s| s.to_string()),
                    place_of_performance: Self::parse_address(opp.get("placeOfPerformance")),
                    award_amount: opp
                        .get("award")
                        .and_then(|a| a.get("amount"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok()),
                }
            })
            .collect())
    }

    /// Parse address from SAM.gov response
    fn parse_address(addr_value: Option<&Value>) -> Option<SamAddress> {
        addr_value.map(|addr| {
            SamAddress {
                line1: Self::get_str(addr, "addressLine1")
                    .or_else(|| Self::get_str(addr, "line1"))
                    .map(|s| s.to_string()),
                line2: Self::get_str(addr, "addressLine2")
                    .or_else(|| Self::get_str(addr, "line2"))
                    .map(|s| s.to_string()),
                city: Self::get_str(addr, "city").map(|s| s.to_string()),
                state: Self::get_str(addr, "stateOrProvinceCode")
                    .or_else(|| Self::get_str(addr, "state"))
                    .map(|s| s.to_string()),
                zip: Self::get_str(addr, "zipCode")
                    .or_else(|| Self::get_str(addr, "zip"))
                    .map(|s| s.to_string()),
                country: Self::get_str(addr, "countryCode")
                    .or_else(|| Self::get_str(addr, "country"))
                    .map(|s| s.to_string()),
            }
        })
    }

    /// Parse NAICS codes array
    fn parse_naics_codes(naics_value: Option<&Value>) -> Vec<String> {
        naics_value
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        // NAICS codes might be objects with "naicsCode" field or strings
                        item.get("naicsCode")
                            .and_then(|v| v.as_str())
                            .or_else(|| item.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error_code) = response.get("error_code") {
            let code = error_code.as_i64().unwrap_or(0) as i32;
            let message = response
                .get("error_message")
                .or_else(|| response.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api { code, message });
        }

        // Check for other error formats
        if let Some(error) = response.get("error") {
            let message = error.as_str().unwrap_or("Unknown error").to_string();
            return Err(ExchangeError::Api { code: 0, message });
        }

        Ok(())
    }

    // Helper methods
    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SAM.GOV-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// SAM.gov entity (government contractor)
#[derive(Debug, Clone)]
pub struct SamEntity {
    pub uei: Option<String>,
    pub cage_code: Option<String>,
    pub legal_business_name: Option<String>,
    pub dba_name: Option<String>,
    pub physical_address: Option<SamAddress>,
    pub entity_type: Option<String>,
    pub registration_status: Option<String>,
    pub naics_codes: Vec<String>,
    pub sam_registration_date: Option<String>,
    pub expiration_date: Option<String>,
}

/// SAM.gov contract opportunity
#[derive(Debug, Clone)]
pub struct SamOpportunity {
    pub notice_id: Option<String>,
    pub title: Option<String>,
    pub solicitation_number: Option<String>,
    pub department: Option<String>,
    pub sub_tier: Option<String>,
    pub office: Option<String>,
    pub posted_date: Option<String>,
    pub response_deadline: Option<String>,
    pub type_name: Option<String>,
    pub description: Option<String>,
    pub place_of_performance: Option<SamAddress>,
    pub award_amount: Option<f64>,
}

/// SAM.gov address
#[derive(Debug, Clone)]
pub struct SamAddress {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
}
