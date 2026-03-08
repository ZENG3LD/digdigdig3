//! OpenSanctions response parsers
//!
//! Parse JSON responses to domain types based on OpenSanctions API response formats.
//!
//! OpenSanctions uses the FollowTheMoney entity format for representing sanctions,
//! PEPs, and other watchlist entities.

use serde_json::Value;
use std::collections::HashMap;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct OpenSanctionsParser;

impl OpenSanctionsParser {
    /// Parse search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "total": 1234,
    ///   "limit": 20,
    ///   "offset": 0,
    ///   "results": [
    ///     {
    ///       "id": "Q123",
    ///       "caption": "John Doe",
    ///       "schema": "Person",
    ///       "properties": {
    ///         "name": ["John Doe"],
    ///         "birthDate": ["1970-01-01"]
    ///       },
    ///       "datasets": ["us_ofac_sdn"],
    ///       "first_seen": "2020-01-01",
    ///       "last_seen": "2024-01-01",
    ///       "last_change": "2024-01-01",
    ///       "target": true,
    ///       "referents": []
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_search_result(response: &Value) -> ExchangeResult<SanctionSearchResult> {
        let total = Self::require_u64(response, "total")?;
        let limit = Self::get_u32(response, "limit").unwrap_or(20);
        let offset = Self::get_u32(response, "offset").unwrap_or(0);

        let results_array = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        let results = results_array
            .iter()
            .map(Self::parse_entity)
            .collect::<ExchangeResult<Vec<SanctionEntity>>>()?;

        Ok(SanctionSearchResult {
            total,
            limit,
            offset,
            results,
        })
    }

    /// Parse single entity
    pub fn parse_entity(entity: &Value) -> ExchangeResult<SanctionEntity> {
        let id = Self::require_str(entity, "id")?.to_string();
        let caption = Self::require_str(entity, "caption")?.to_string();
        let schema_type = Self::require_str(entity, "schema")?.to_string();

        // Parse properties as HashMap<String, Vec<String>>
        let properties = if let Some(props) = entity.get("properties") {
            Self::parse_properties(props)?
        } else {
            HashMap::new()
        };

        // Parse datasets
        let datasets = entity
            .get("datasets")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let first_seen = Self::get_str(entity, "first_seen").map(|s| s.to_string());
        let last_seen = Self::get_str(entity, "last_seen").map(|s| s.to_string());
        let last_change = Self::get_str(entity, "last_change").map(|s| s.to_string());
        let target = Self::get_bool(entity, "target").unwrap_or(false);

        // Parse referents
        let referents = entity
            .get("referents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(SanctionEntity {
            id,
            caption,
            schema_type,
            properties,
            datasets,
            first_seen,
            last_seen,
            last_change,
            target,
            referents,
        })
    }

    /// Parse properties object to HashMap<String, Vec<String>>
    fn parse_properties(props: &Value) -> ExchangeResult<HashMap<String, Vec<String>>> {
        let obj = props
            .as_object()
            .ok_or_else(|| ExchangeError::Parse("Properties must be an object".to_string()))?;

        let mut map = HashMap::new();
        for (key, value) in obj.iter() {
            if let Some(arr) = value.as_array() {
                let values: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                map.insert(key.clone(), values);
            }
        }

        Ok(map)
    }

    /// Parse datasets list
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "datasets": [
    ///     {
    ///       "name": "us_ofac_sdn",
    ///       "title": "US OFAC Specially Designated Nationals",
    ///       "summary": "...",
    ///       "url": "https://...",
    ///       "category": "sanctions",
    ///       "publisher": "US OFAC",
    ///       "entity_count": 12345,
    ///       "last_change": "2024-01-01"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_datasets(response: &Value) -> ExchangeResult<Vec<SanctionDataset>> {
        let datasets_array = response
            .get("datasets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'datasets' array".to_string()))?;

        datasets_array
            .iter()
            .map(|dataset| {
                Ok(SanctionDataset {
                    name: Self::require_str(dataset, "name")?.to_string(),
                    title: Self::require_str(dataset, "title")?.to_string(),
                    summary: Self::get_str(dataset, "summary").map(|s| s.to_string()),
                    url: Self::get_str(dataset, "url").map(|s| s.to_string()),
                    category: Self::get_str(dataset, "category").map(|s| s.to_string()),
                    publisher: Self::get_str(dataset, "publisher").map(|s| s.to_string()),
                    entity_count: Self::get_u64(dataset, "entity_count"),
                    last_change: Self::get_str(dataset, "last_change").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single dataset
    pub fn parse_dataset(response: &Value) -> ExchangeResult<SanctionDataset> {
        Ok(SanctionDataset {
            name: Self::require_str(response, "name")?.to_string(),
            title: Self::require_str(response, "title")?.to_string(),
            summary: Self::get_str(response, "summary").map(|s| s.to_string()),
            url: Self::get_str(response, "url").map(|s| s.to_string()),
            category: Self::get_str(response, "category").map(|s| s.to_string()),
            publisher: Self::get_str(response, "publisher").map(|s| s.to_string()),
            entity_count: Self::get_u64(response, "entity_count"),
            last_change: Self::get_str(response, "last_change").map(|s| s.to_string()),
        })
    }

    /// Parse collections list
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "collections": [
    ///     {
    ///       "name": "default",
    ///       "title": "Default Collection",
    ///       "summary": "...",
    ///       "datasets": ["us_ofac_sdn", "un_sc_sanctions"]
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_collections(response: &Value) -> ExchangeResult<Vec<SanctionCollection>> {
        let collections_array = response
            .get("collections")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'collections' array".to_string()))?;

        collections_array
            .iter()
            .map(|collection| {
                let datasets = collection
                    .get("datasets")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(SanctionCollection {
                    name: Self::require_str(collection, "name")?.to_string(),
                    title: Self::require_str(collection, "title")?.to_string(),
                    summary: Self::get_str(collection, "summary").map(|s| s.to_string()),
                    datasets,
                })
            })
            .collect()
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .or_else(|| error.get("message").and_then(|v| v.as_str()))
                .unwrap_or("Unknown error")
                .to_string();

            let code = response
                .get("status")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            return Err(ExchangeError::Api { code, message });
        }
        Ok(())
    }

    // Helper methods
    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn require_u64(obj: &Value, field: &str) -> ExchangeResult<u64> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OPENSANCTIONS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Sanction entity (FollowTheMoney format)
#[derive(Debug, Clone)]
pub struct SanctionEntity {
    pub id: String,
    pub caption: String,
    pub schema_type: String,
    pub properties: HashMap<String, Vec<String>>,
    pub datasets: Vec<String>,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub last_change: Option<String>,
    pub target: bool,
    pub referents: Vec<String>,
}

/// Search result wrapper
#[derive(Debug, Clone)]
pub struct SanctionSearchResult {
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
    pub results: Vec<SanctionEntity>,
}

/// Dataset metadata
#[derive(Debug, Clone)]
pub struct SanctionDataset {
    pub name: String,
    pub title: String,
    pub summary: Option<String>,
    pub url: Option<String>,
    pub category: Option<String>,
    pub publisher: Option<String>,
    pub entity_count: Option<u64>,
    pub last_change: Option<String>,
}

/// Collection metadata
#[derive(Debug, Clone)]
pub struct SanctionCollection {
    pub name: String,
    pub title: String,
    pub summary: Option<String>,
    pub datasets: Vec<String>,
}
