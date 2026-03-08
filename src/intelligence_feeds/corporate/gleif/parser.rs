//! GLEIF response parsers
//!
//! Parse JSON:API responses to domain types based on GLEIF API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ExchangeError;
type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct GleifParser;

impl GleifParser {
    // ═══════════════════════════════════════════════════════════════════════
    // GLEIF-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse LEI records response (JSON:API format)
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": [{
    ///     "type": "lei-records",
    ///     "id": "549300XOCZUOQA850F50",
    ///     "attributes": {
    ///       "lei": "549300XOCZUOQA850F50",
    ///       "entity": {
    ///         "legalName": {"name": "APPLE INC."},
    ///         "legalAddress": {"country": "US", "addressLines": [...]},
    ///         "headquartersAddress": {...},
    ///         "jurisdiction": "US-DE",
    ///         "category": "GENERAL"
    ///       },
    ///       "registration": {
    ///         "initialRegistrationDate": "2012-06-06T00:00:00.000Z",
    ///         "lastUpdateDate": "2023-01-01T00:00:00.000Z",
    ///         "status": "ISSUED"
    ///       }
    ///     }
    ///   }]
    /// }
    /// ```
    pub fn parse_lei_records(response: &Value) -> ExchangeResult<Vec<GleifEntity>> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))?;

        // Handle both single object and array responses
        let records = if data.is_array() {
            data.as_array()
                .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?
                .clone()
        } else {
            vec![data.clone()]
        };

        let mut entities = Vec::new();
        for record in records {
            if let Ok(entity) = Self::parse_entity(&record) {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// Parse a single entity record
    fn parse_entity(record: &Value) -> ExchangeResult<GleifEntity> {
        let attributes = record
            .get("attributes")
            .ok_or_else(|| ExchangeError::Parse("Missing 'attributes' field".to_string()))?;

        let lei = Self::get_str(attributes, "lei")
            .ok_or_else(|| ExchangeError::Parse("Missing 'lei' field".to_string()))?
            .to_string();

        let entity = attributes
            .get("entity")
            .ok_or_else(|| ExchangeError::Parse("Missing 'entity' field".to_string()))?;

        let legal_name = entity
            .get("legalName")
            .and_then(|n| n.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();

        let jurisdiction = Self::get_str(entity, "jurisdiction")
            .map(|s| s.to_string());

        let category = Self::get_str(entity, "category")
            .map(|s| s.to_string());

        // Parse legal address
        let country = entity
            .get("legalAddress")
            .and_then(|addr| addr.get("country"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        let registered_address = entity
            .get("legalAddress")
            .and_then(|addr| Self::parse_address(addr).ok());

        let headquarters_address = entity
            .get("headquartersAddress")
            .and_then(|addr| Self::parse_address(addr).ok());

        // Parse registration info
        let registration = attributes.get("registration");
        let status = registration
            .and_then(|r| r.get("status"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let registration_date = registration
            .and_then(|r| r.get("initialRegistrationDate"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let last_update = registration
            .and_then(|r| r.get("lastUpdateDate"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        Ok(GleifEntity {
            lei,
            legal_name,
            jurisdiction,
            country,
            category,
            status,
            registered_address,
            headquarters_address,
            registration_date,
            last_update,
        })
    }

    /// Parse address object
    fn parse_address(addr: &Value) -> ExchangeResult<String> {
        let lines = addr
            .get("addressLines")
            .and_then(|l| l.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let city = Self::get_str(addr, "city").unwrap_or("");
        let region = Self::get_str(addr, "region").unwrap_or("");
        let country = Self::get_str(addr, "country").unwrap_or("");
        let postal_code = Self::get_str(addr, "postalCode").unwrap_or("");

        let mut parts = Vec::new();
        if !lines.is_empty() {
            parts.push(lines);
        }
        if !city.is_empty() {
            parts.push(city.to_string());
        }
        if !region.is_empty() {
            parts.push(region.to_string());
        }
        if !postal_code.is_empty() {
            parts.push(postal_code.to_string());
        }
        if !country.is_empty() {
            parts.push(country.to_string());
        }

        Ok(parts.join(", "))
    }

    /// Parse relationship response
    pub fn parse_relationship(response: &Value) -> ExchangeResult<Option<GleifEntity>> {
        // Check if data exists
        let data = response.get("data");

        if data.is_none() || data.unwrap().is_null() {
            return Ok(None);
        }

        let data = data.unwrap();
        let entity = Self::parse_entity(data)?;
        Ok(Some(entity))
    }

    /// Parse children response
    pub fn parse_children(response: &Value) -> ExchangeResult<Vec<GleifEntity>> {
        Self::parse_lei_records(response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(errors) = response.get("errors") {
            if let Some(err_array) = errors.as_array() {
                if let Some(first_error) = err_array.first() {
                    let message = first_error
                        .get("detail")
                        .or_else(|| first_error.get("title"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();

                    let code = first_error
                        .get("status")
                        .and_then(|s| s.as_str())
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(400);

                    return Err(ExchangeError::Api { code, message });
                }
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GLEIF-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// GLEIF legal entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GleifEntity {
    pub lei: String,
    pub legal_name: String,
    pub jurisdiction: Option<String>,
    pub country: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub registered_address: Option<String>,
    pub headquarters_address: Option<String>,
    pub registration_date: Option<String>,
    pub last_update: Option<String>,
}

/// GLEIF relationship (parent/child)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GleifRelationship {
    pub parent_lei: String,
    pub parent_name: Option<String>,
    pub relationship_type: String,
}

/// GLEIF ownership chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GleifOwnershipChain {
    pub entity: GleifEntity,
    pub direct_parent: Option<GleifEntity>,
    pub ultimate_parent: Option<GleifEntity>,
    pub children: Vec<GleifEntity>,
}
