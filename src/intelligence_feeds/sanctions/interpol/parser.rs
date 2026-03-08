//! INTERPOL response parsers
//!
//! Parse JSON responses to domain types based on INTERPOL API response formats.
//!
//! INTERPOL Red Notices API provides information about wanted persons,
//! missing persons, and UN Security Council special notices.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct InterpolParser;

impl InterpolParser {
    /// Parse search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "total": 1234,
    ///   "query": { "page": 1, "resultPerPage": 20 },
    ///   "_embedded": {
    ///     "notices": [
    ///       {
    ///         "entity_id": "2023/12345",
    ///         "name": "DOE",
    ///         "forename": "John",
    ///         "date_of_birth": "1970/01/01",
    ///         "nationalities": ["US"],
    ///         "sex_id": "M",
    ///         "arrest_warrants": [
    ///           {
    ///             "issuing_country_id": "US",
    ///             "charge": "Murder"
    ///           }
    ///         ]
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    pub fn parse_search_result(response: &Value) -> ExchangeResult<InterpolSearchResult> {
        let total = Self::require_u64(response, "total")?;

        let query = response.get("query").cloned();

        let notices_array = response
            .get("_embedded")
            .and_then(|v| v.get("notices"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing '_embedded.notices' array".to_string()))?;

        let notices = notices_array
            .iter()
            .map(Self::parse_notice)
            .collect::<ExchangeResult<Vec<InterpolNotice>>>()?;

        Ok(InterpolSearchResult {
            total,
            query,
            notices,
        })
    }

    /// Parse single notice
    pub fn parse_notice(notice: &Value) -> ExchangeResult<InterpolNotice> {
        let entity_id = Self::require_str(notice, "entity_id")?.to_string();
        let name = Self::require_str(notice, "name")?.to_string();
        let forename = Self::get_str(notice, "forename").map(|s| s.to_string());
        let date_of_birth = Self::get_str(notice, "date_of_birth").map(|s| s.to_string());

        // Parse nationalities
        let nationalities = notice
            .get("nationalities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let sex_id = Self::get_str(notice, "sex_id").map(|s| s.to_string());
        let country_of_birth_id = Self::get_str(notice, "country_of_birth_id").map(|s| s.to_string());
        let place_of_birth = Self::get_str(notice, "place_of_birth").map(|s| s.to_string());

        // Parse languages_spoken_ids
        let languages_spoken_ids = notice
            .get("languages_spoken_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Parse arrest warrants
        let arrest_warrants = notice
            .get("arrest_warrants")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|warrant| Self::parse_arrest_warrant(warrant).ok())
                    .collect()
            })
            .unwrap_or_default();

        let weight = Self::get_u32(notice, "weight");
        let height = Self::get_f64(notice, "height");
        let eyes_colors_id = Self::get_str(notice, "eyes_colors_id").map(|s| s.to_string());
        let hairs_id = Self::get_str(notice, "hairs_id").map(|s| s.to_string());
        let distinguishing_marks = Self::get_str(notice, "distinguishing_marks").map(|s| s.to_string());

        Ok(InterpolNotice {
            entity_id,
            name,
            forename,
            date_of_birth,
            nationalities,
            sex_id,
            country_of_birth_id,
            place_of_birth,
            languages_spoken_ids,
            arrest_warrants,
            weight,
            height,
            eyes_colors_id,
            hairs_id,
            distinguishing_marks,
        })
    }

    /// Parse arrest warrant
    fn parse_arrest_warrant(warrant: &Value) -> ExchangeResult<ArrestWarrant> {
        let issuing_country_id = Self::require_str(warrant, "issuing_country_id")?.to_string();
        let charge = Self::get_str(warrant, "charge").map(|s| s.to_string());
        let charge_translation = Self::get_str(warrant, "charge_translation").map(|s| s.to_string());

        Ok(ArrestWarrant {
            issuing_country_id,
            charge,
            charge_translation,
        })
    }

    /// Parse images response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "_embedded": {
    ///     "images": [
    ///       {
    ///         "picture_id": "12345",
    ///         "_links": {
    ///           "self": {
    ///             "href": "https://ws-public.interpol.int/..."
    ///           }
    ///         }
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    pub fn parse_images(response: &Value) -> ExchangeResult<Vec<InterpolImage>> {
        let images_array = response
            .get("_embedded")
            .and_then(|v| v.get("images"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing '_embedded.images' array".to_string()))?;

        images_array
            .iter()
            .map(|image| {
                let picture_id = Self::require_str(image, "picture_id")?.to_string();
                let href = image
                    .get("_links")
                    .and_then(|v| v.get("self"))
                    .and_then(|v| v.get("href"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExchangeError::Parse("Missing image href".to_string()))?
                    .to_string();

                Ok(InterpolImage {
                    picture_id,
                    href,
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

    fn _get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INTERPOL-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// INTERPOL notice (red, yellow, or UN)
#[derive(Debug, Clone)]
pub struct InterpolNotice {
    pub entity_id: String,
    pub name: String,
    pub forename: Option<String>,
    pub date_of_birth: Option<String>,
    pub nationalities: Vec<String>,
    pub sex_id: Option<String>,
    pub country_of_birth_id: Option<String>,
    pub place_of_birth: Option<String>,
    pub languages_spoken_ids: Vec<String>,
    pub arrest_warrants: Vec<ArrestWarrant>,
    pub weight: Option<u32>,
    pub height: Option<f64>,
    pub eyes_colors_id: Option<String>,
    pub hairs_id: Option<String>,
    pub distinguishing_marks: Option<String>,
}

/// Arrest warrant information
#[derive(Debug, Clone)]
pub struct ArrestWarrant {
    pub issuing_country_id: String,
    pub charge: Option<String>,
    pub charge_translation: Option<String>,
}

/// Search result wrapper
#[derive(Debug, Clone)]
pub struct InterpolSearchResult {
    pub total: u64,
    pub query: Option<Value>,
    pub notices: Vec<InterpolNotice>,
}

/// INTERPOL notice image
#[derive(Debug, Clone)]
pub struct InterpolImage {
    pub picture_id: String,
    pub href: String,
}
