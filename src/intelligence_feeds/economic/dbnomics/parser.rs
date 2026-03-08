//! DBnomics response parsers
//!
//! Parse JSON responses from DBnomics API to domain types.
//!
//! DBnomics provides access to economic data from multiple international providers
//! (IMF, World Bank, ECB, OECD, Eurostat, BIS, ILO, etc.)

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct DBnomicsParser;

impl DBnomicsParser {
    // ═══════════════════════════════════════════════════════════════════════
    // PROVIDER PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse list of providers
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "providers": {
    ///     "docs": [
    ///       {
    ///         "code": "IMF",
    ///         "name": "International Monetary Fund",
    ///         "region": "World",
    ///         "website": "https://www.imf.org/"
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    pub fn parse_providers(response: &Value) -> ExchangeResult<Vec<Provider>> {
        let docs = response
            .get("providers")
            .and_then(|p| p.get("docs"))
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'providers.docs' array".to_string()))?;

        docs.iter()
            .map(|p| {
                Ok(Provider {
                    code: Self::require_str(p, "code")?.to_string(),
                    name: Self::require_str(p, "name")?.to_string(),
                    region: Self::get_str(p, "region").map(|s| s.to_string()),
                    website: Self::get_str(p, "website").map(|s| s.to_string()),
                    terms_of_use: Self::get_str(p, "terms_of_use").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single provider
    pub fn parse_provider(response: &Value) -> ExchangeResult<Provider> {
        let provider = response
            .get("provider")
            .ok_or_else(|| ExchangeError::Parse("Missing 'provider' object".to_string()))?;

        Ok(Provider {
            code: Self::require_str(provider, "code")?.to_string(),
            name: Self::require_str(provider, "name")?.to_string(),
            region: Self::get_str(provider, "region").map(|s| s.to_string()),
            website: Self::get_str(provider, "website").map(|s| s.to_string()),
            terms_of_use: Self::get_str(provider, "terms_of_use").map(|s| s.to_string()),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATASET PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse list of datasets
    pub fn parse_datasets(response: &Value) -> ExchangeResult<Vec<Dataset>> {
        let docs = response
            .get("datasets")
            .and_then(|d| d.get("docs"))
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'datasets.docs' array".to_string()))?;

        docs.iter()
            .map(|d| {
                Ok(Dataset {
                    code: Self::require_str(d, "code")?.to_string(),
                    name: Self::require_str(d, "name")?.to_string(),
                    provider_code: Self::require_str(d, "provider_code")?.to_string(),
                    provider_name: Self::get_str(d, "provider_name").map(|s| s.to_string()),
                    nb_series: Self::get_i64(d, "nb_series"),
                    dimensions_codes_order: Self::get_string_array(d, "dimensions_codes_order"),
                })
            })
            .collect()
    }

    /// Parse single dataset
    pub fn parse_dataset(response: &Value) -> ExchangeResult<Dataset> {
        let dataset = response
            .get("dataset")
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataset' object".to_string()))?;

        Ok(Dataset {
            code: Self::require_str(dataset, "code")?.to_string(),
            name: Self::require_str(dataset, "name")?.to_string(),
            provider_code: Self::require_str(dataset, "provider_code")?.to_string(),
            provider_name: Self::get_str(dataset, "provider_name").map(|s| s.to_string()),
            nb_series: Self::get_i64(dataset, "nb_series"),
            dimensions_codes_order: Self::get_string_array(dataset, "dimensions_codes_order"),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SERIES PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse list of series
    pub fn parse_series_list(response: &Value) -> ExchangeResult<Vec<Series>> {
        let docs = response
            .get("series")
            .and_then(|s| s.get("docs"))
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'series.docs' array".to_string()))?;

        docs.iter().map(Self::parse_series_object).collect()
    }

    /// Parse single series with observations
    pub fn parse_series(response: &Value) -> ExchangeResult<Series> {
        let series = response
            .get("series")
            .ok_or_else(|| ExchangeError::Parse("Missing 'series' object".to_string()))?;

        Self::parse_series_object(series)
    }

    /// Parse series object (helper)
    fn parse_series_object(series: &Value) -> ExchangeResult<Series> {
        let observations = if let Some(obs_obj) = series.get("observations") {
            Self::parse_observations(obs_obj)?
        } else {
            Vec::new()
        };

        Ok(Series {
            code: Self::require_str(series, "code")?.to_string(),
            name: Self::require_str(series, "name")?.to_string(),
            provider_code: Self::require_str(series, "provider_code")?.to_string(),
            dataset_code: Self::require_str(series, "dataset_code")?.to_string(),
            dataset_name: Self::get_str(series, "dataset_name").map(|s| s.to_string()),
            dimensions: Self::get_dimensions(series, "dimensions"),
            period_start_day: Self::get_str(series, "period_start_day").map(|s| s.to_string()),
            period_end_day: Self::get_str(series, "period_end_day").map(|s| s.to_string()),
            observations,
        })
    }

    /// Parse observations from series
    fn parse_observations(obs_value: &Value) -> ExchangeResult<Vec<Observation>> {
        let obs_array = obs_value
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Observations must be an array".to_string()))?;

        obs_array
            .iter()
            .map(|obs| {
                Ok(Observation {
                    period: Self::require_str(obs, "period")?.to_string(),
                    value: Self::get_f64(obs, "value"),
                    original_period: Self::get_str(obs, "original_period").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LAST UPDATES PARSER
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse last updates
    pub fn parse_last_updates(response: &Value) -> ExchangeResult<Vec<LastUpdate>> {
        let docs = response
            .get("last_updates")
            .and_then(|u| u.get("docs"))
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'last_updates.docs' array".to_string()))?;

        docs.iter()
            .map(|u| {
                Ok(LastUpdate {
                    provider_code: Self::require_str(u, "provider_code")?.to_string(),
                    dataset_code: Self::require_str(u, "dataset_code")?.to_string(),
                    series_code: Self::get_str(u, "series_code").map(|s| s.to_string()),
                    indexed_at: Self::require_str(u, "indexed_at")?.to_string(),
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
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            let code = error
                .get("code")
                .and_then(|c| c.as_i64())
                .unwrap_or(0) as i32;

            return Err(ExchangeError::Api { code, message });
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

    fn get_string_array(obj: &Value, field: &str) -> Option<Vec<String>> {
        obj.get(field)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
    }

    fn get_dimensions(obj: &Value, field: &str) -> Option<Vec<Dimension>> {
        obj.get(field).and_then(|v| v.as_object()).map(|map| {
            map.iter()
                .map(|(key, value)| Dimension {
                    key: key.clone(),
                    value: value.as_str().unwrap_or("").to_string(),
                })
                .collect()
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DBNOMICS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// DBnomics data provider (e.g., IMF, World Bank, ECB)
#[derive(Debug, Clone)]
pub struct Provider {
    pub code: String,
    pub name: String,
    pub region: Option<String>,
    pub website: Option<String>,
    pub terms_of_use: Option<String>,
}

/// DBnomics dataset
#[derive(Debug, Clone)]
pub struct Dataset {
    pub code: String,
    pub name: String,
    pub provider_code: String,
    pub provider_name: Option<String>,
    pub nb_series: Option<i64>,
    pub dimensions_codes_order: Option<Vec<String>>,
}

/// DBnomics series (time series data)
#[derive(Debug, Clone)]
pub struct Series {
    pub code: String,
    pub name: String,
    pub provider_code: String,
    pub dataset_code: String,
    pub dataset_name: Option<String>,
    pub dimensions: Option<Vec<Dimension>>,
    pub period_start_day: Option<String>,
    pub period_end_day: Option<String>,
    pub observations: Vec<Observation>,
}

/// DBnomics observation (single data point)
#[derive(Debug, Clone)]
pub struct Observation {
    pub period: String,
    pub value: Option<f64>,
    pub original_period: Option<String>,
}

/// DBnomics dimension (series metadata)
#[derive(Debug, Clone)]
pub struct Dimension {
    pub key: String,
    pub value: String,
}

/// DBnomics last update
#[derive(Debug, Clone)]
pub struct LastUpdate {
    pub provider_code: String,
    pub dataset_code: String,
    pub series_code: Option<String>,
    pub indexed_at: String,
}
