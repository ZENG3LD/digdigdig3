//! EU Parliament response parsers
//!
//! Parse JSON-LD responses to domain types.
//!
//! EU Parliament API returns JSON-LD format. We parse as regular JSON,
//! ignoring @context/@id/@type fields.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct EuParliamentParser;

impl EuParliamentParser {
    // ═══════════════════════════════════════════════════════════════════════
    // MEP PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse MEPs list
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "@context": "...",
    ///   "data": [
    ///     {
    ///       "id": "12345",
    ///       "givenName": "John",
    ///       "familyName": "Doe",
    ///       "country": "Belgium",
    ///       "politicalGroup": "EPP",
    ///       "active": true
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_meps(response: &Value) -> ExchangeResult<Vec<EuMep>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|mep| {
                Ok(EuMep {
                    id: Self::get_str(mep, "id")
                        .or_else(|| Self::get_str(mep, "@id"))
                        .unwrap_or("unknown")
                        .to_string(),
                    given_name: Self::get_str(mep, "givenName")
                        .or_else(|| Self::get_str(mep, "given_name"))
                        .map(|s| s.to_string()),
                    family_name: Self::get_str(mep, "familyName")
                        .or_else(|| Self::get_str(mep, "family_name"))
                        .map(|s| s.to_string()),
                    country: Self::get_str(mep, "country")
                        .or_else(|| Self::get_str(mep, "countryCode"))
                        .or_else(|| Self::get_str(mep, "country_code"))
                        .map(|s| s.to_string()),
                    political_group: Self::get_str(mep, "politicalGroup")
                        .or_else(|| Self::get_str(mep, "political_group"))
                        .or_else(|| Self::get_str(mep, "group"))
                        .map(|s| s.to_string()),
                    active: Self::get_bool(mep, "active").unwrap_or(true),
                })
            })
            .collect()
    }

    /// Parse single MEP
    pub fn parse_mep(response: &Value) -> ExchangeResult<EuMep> {
        // If response is already a single object
        if response.get("id").is_some() || response.get("@id").is_some() {
            return Ok(EuMep {
                id: Self::get_str(response, "id")
                    .or_else(|| Self::get_str(response, "@id"))
                    .unwrap_or("unknown")
                    .to_string(),
                given_name: Self::get_str(response, "givenName")
                    .or_else(|| Self::get_str(response, "given_name"))
                    .map(|s| s.to_string()),
                family_name: Self::get_str(response, "familyName")
                    .or_else(|| Self::get_str(response, "family_name"))
                    .map(|s| s.to_string()),
                country: Self::get_str(response, "country")
                    .or_else(|| Self::get_str(response, "countryCode"))
                    .or_else(|| Self::get_str(response, "country_code"))
                    .map(|s| s.to_string()),
                political_group: Self::get_str(response, "politicalGroup")
                    .or_else(|| Self::get_str(response, "political_group"))
                    .or_else(|| Self::get_str(response, "group"))
                    .map(|s| s.to_string()),
                active: Self::get_bool(response, "active").unwrap_or(true),
            });
        }

        // Otherwise, parse first from data array
        let meps = Self::parse_meps(response)?;
        meps.into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse("No MEP found".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DOCUMENT PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse plenary documents
    pub fn parse_documents(response: &Value) -> ExchangeResult<Vec<EuDocument>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|doc| {
                Ok(EuDocument {
                    id: Self::get_str(doc, "id")
                        .or_else(|| Self::get_str(doc, "@id"))
                        .unwrap_or("unknown")
                        .to_string(),
                    title: Self::get_str(doc, "title")
                        .or_else(|| Self::get_str(doc, "name"))
                        .map(|s| s.to_string()),
                    date: Self::get_str(doc, "date")
                        .or_else(|| Self::get_str(doc, "dateDocument"))
                        .or_else(|| Self::get_str(doc, "date_document"))
                        .map(|s| s.to_string()),
                    document_type: Self::get_str(doc, "documentType")
                        .or_else(|| Self::get_str(doc, "document_type"))
                        .or_else(|| Self::get_str(doc, "type"))
                        .map(|s| s.to_string()),
                    reference: Self::get_str(doc, "reference")
                        .or_else(|| Self::get_str(doc, "identifier"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single document
    pub fn parse_document(response: &Value) -> ExchangeResult<EuDocument> {
        // If response is already a single object
        if response.get("id").is_some() || response.get("@id").is_some() {
            return Ok(EuDocument {
                id: Self::get_str(response, "id")
                    .or_else(|| Self::get_str(response, "@id"))
                    .unwrap_or("unknown")
                    .to_string(),
                title: Self::get_str(response, "title")
                    .or_else(|| Self::get_str(response, "name"))
                    .map(|s| s.to_string()),
                date: Self::get_str(response, "date")
                    .or_else(|| Self::get_str(response, "dateDocument"))
                    .or_else(|| Self::get_str(response, "date_document"))
                    .map(|s| s.to_string()),
                document_type: Self::get_str(response, "documentType")
                    .or_else(|| Self::get_str(response, "document_type"))
                    .or_else(|| Self::get_str(response, "type"))
                    .map(|s| s.to_string()),
                reference: Self::get_str(response, "reference")
                    .or_else(|| Self::get_str(response, "identifier"))
                    .map(|s| s.to_string()),
            });
        }

        // Otherwise, parse first from data array
        let docs = Self::parse_documents(response)?;
        docs.into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse("No document found".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MEETING PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse meetings
    pub fn parse_meetings(response: &Value) -> ExchangeResult<Vec<EuMeeting>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|meeting| {
                Ok(EuMeeting {
                    id: Self::get_str(meeting, "id")
                        .or_else(|| Self::get_str(meeting, "@id"))
                        .unwrap_or("unknown")
                        .to_string(),
                    date: Self::get_str(meeting, "date")
                        .or_else(|| Self::get_str(meeting, "dateStart"))
                        .or_else(|| Self::get_str(meeting, "date_start"))
                        .map(|s| s.to_string()),
                    title: Self::get_str(meeting, "title")
                        .or_else(|| Self::get_str(meeting, "name"))
                        .map(|s| s.to_string()),
                    location: Self::get_str(meeting, "location")
                        .or_else(|| Self::get_str(meeting, "place"))
                        .map(|s| s.to_string()),
                    committee: Self::get_str(meeting, "committee")
                        .or_else(|| Self::get_str(meeting, "committeeId"))
                        .or_else(|| Self::get_str(meeting, "committee_id"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMMITTEE PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse committees
    pub fn parse_committees(response: &Value) -> ExchangeResult<Vec<EuCommittee>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|committee| {
                Ok(EuCommittee {
                    id: Self::get_str(committee, "id")
                        .or_else(|| Self::get_str(committee, "@id"))
                        .unwrap_or("unknown")
                        .to_string(),
                    name: Self::get_str(committee, "name")
                        .or_else(|| Self::get_str(committee, "title"))
                        .map(|s| s.to_string()),
                    abbreviation: Self::get_str(committee, "abbreviation")
                        .or_else(|| Self::get_str(committee, "acronym"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .or_else(|| error.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            let code = error
                .get("code")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            return Err(ExchangeError::Api { code, message });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EU PARLIAMENT-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Member of European Parliament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuMep {
    pub id: String,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub political_group: Option<String>,
    #[serde(default)]
    pub active: bool,
}

/// EU Parliament document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuDocument {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub document_type: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
}

/// EU Parliament meeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuMeeting {
    pub id: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub committee: Option<String>,
}

/// EU Parliament committee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuCommittee {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub abbreviation: Option<String>,
}
