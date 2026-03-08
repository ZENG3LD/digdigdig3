//! Censys response parsers
//!
//! Parse JSON responses to domain types based on Censys API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct CensysParser;

impl CensysParser {
    // ═══════════════════════════════════════════════════════════════════════
    // CENSYS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse host information
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "result": {
    ///     "ip": "8.8.8.8",
    ///     "services": [
    ///       {
    ///         "port": 53,
    ///         "service_name": "DNS",
    ///         "transport_protocol": "UDP",
    ///         "certificate": "..."
    ///       }
    ///     ],
    ///     "location": {
    ///       "country": "United States",
    ///       "city": "Mountain View",
    ///       "province": "California",
    ///       "coordinates": {
    ///         "latitude": 37.386,
    ///         "longitude": -122.0838
    ///       }
    ///     },
    ///     "autonomous_system": {
    ///       "asn": 15169,
    ///       "name": "GOOGLE",
    ///       "description": "Google LLC"
    ///     },
    ///     "operating_system": {
    ///       "vendor": "Linux",
    ///       "product": "Linux Kernel"
    ///     },
    ///     "last_updated_at": "2024-01-15T10:30:00Z"
    ///   }
    /// }
    /// ```
    pub fn parse_host(data: &Value) -> ExchangeResult<CensysHost> {
        let result = data
            .get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))?;

        let ip = Self::require_str(result, "ip")?.to_string();

        let services = result
            .get("services")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|service_data| Self::parse_service(service_data).ok())
                    .collect()
            })
            .unwrap_or_default();

        let location = result
            .get("location")
            .and_then(|loc| Self::parse_location(loc).ok());

        let autonomous_system = result
            .get("autonomous_system")
            .map(|as_data| {
                let asn = Self::get_u64(as_data, "asn");
                let name = Self::get_str(as_data, "name").map(|s| s.to_string());
                let description = Self::get_str(as_data, "description").map(|s| s.to_string());
                (asn, name, description)
            });

        let operating_system = result
            .get("operating_system")
            .and_then(|os| {
                let vendor = Self::get_str(os, "vendor").map(|s| s.to_string());
                let product = Self::get_str(os, "product").map(|s| s.to_string());
                if vendor.is_some() || product.is_some() {
                    Some(format!(
                        "{} {}",
                        vendor.unwrap_or_default(),
                        product.unwrap_or_default()
                    ).trim().to_string())
                } else {
                    None
                }
            });

        let last_updated_at = Self::get_str(result, "last_updated_at").map(|s| s.to_string());

        Ok(CensysHost {
            ip,
            services,
            location,
            autonomous_system,
            operating_system,
            last_updated_at,
        })
    }

    /// Parse service information
    fn parse_service(data: &Value) -> ExchangeResult<CensysService> {
        let port = Self::require_u64(data, "port")? as u16;
        let service_name = Self::get_str(data, "service_name").map(|s| s.to_string());
        let transport_protocol = Self::require_str(data, "transport_protocol")?.to_string();
        let certificate = Self::get_str(data, "certificate").map(|s| s.to_string());

        Ok(CensysService {
            port,
            service_name,
            transport_protocol,
            certificate,
        })
    }

    /// Parse location information
    fn parse_location(data: &Value) -> ExchangeResult<CensysLocation> {
        let country = Self::get_str(data, "country").map(|s| s.to_string());
        let city = Self::get_str(data, "city").map(|s| s.to_string());
        let province = Self::get_str(data, "province").map(|s| s.to_string());

        let coordinates = data
            .get("coordinates")
            .and_then(|coords| {
                let lat = Self::get_f64(coords, "latitude");
                let lon = Self::get_f64(coords, "longitude");
                match (lat, lon) {
                    (Some(latitude), Some(longitude)) => Some((latitude, longitude)),
                    _ => None,
                }
            });

        Ok(CensysLocation {
            country,
            city,
            province,
            coordinates,
        })
    }

    /// Parse search result
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "result": {
    ///     "total": 1234,
    ///     "hits": [...]
    ///   }
    /// }
    /// ```
    pub fn parse_search_result(data: &Value) -> ExchangeResult<CensysSearchResult> {
        let result = data
            .get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))?;

        let total = Self::require_u64(result, "total")?;

        let hits = result
            .get("hits")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'hits' array".to_string()))?;

        let hosts = hits
            .iter()
            .filter_map(|host_data| {
                // Each hit might be wrapped or direct - try both patterns
                if let Some(result_obj) = host_data.get("result") {
                    Self::parse_host_from_search(result_obj).ok()
                } else {
                    Self::parse_host_from_search(host_data).ok()
                }
            })
            .collect();

        Ok(CensysSearchResult { total, hits: hosts })
    }

    /// Parse host from search hit (slightly different format than full host view)
    fn parse_host_from_search(data: &Value) -> ExchangeResult<CensysHost> {
        let ip = Self::require_str(data, "ip")?.to_string();

        let services = data
            .get("services")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|service_data| Self::parse_service(service_data).ok())
                    .collect()
            })
            .unwrap_or_default();

        let location = data
            .get("location")
            .and_then(|loc| Self::parse_location(loc).ok());

        let autonomous_system = data
            .get("autonomous_system")
            .map(|as_data| {
                let asn = Self::get_u64(as_data, "asn");
                let name = Self::get_str(as_data, "name").map(|s| s.to_string());
                let description = Self::get_str(as_data, "description").map(|s| s.to_string());
                (asn, name, description)
            });

        let operating_system = data
            .get("operating_system")
            .and_then(|os| {
                let vendor = Self::get_str(os, "vendor").map(|s| s.to_string());
                let product = Self::get_str(os, "product").map(|s| s.to_string());
                if vendor.is_some() || product.is_some() {
                    Some(format!(
                        "{} {}",
                        vendor.unwrap_or_default(),
                        product.unwrap_or_default()
                    ).trim().to_string())
                } else {
                    None
                }
            });

        let last_updated_at = Self::get_str(data, "last_updated_at").map(|s| s.to_string());

        Ok(CensysHost {
            ip,
            services,
            location,
            autonomous_system,
            operating_system,
            last_updated_at,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
        }

        // Check for error_type field (Censys uses this sometimes)
        if let Some(error_type) = response.get("error_type") {
            let message = response
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            let error_type_str = error_type.as_str().unwrap_or("unknown");

            return Err(ExchangeError::Api {
                code: 0,
                message: format!("{}: {}", error_type_str, message),
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

    fn require_u64(obj: &Value, field: &str) -> ExchangeResult<u64> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CENSYS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Censys host information
#[derive(Debug, Clone)]
pub struct CensysHost {
    pub ip: String,
    pub services: Vec<CensysService>,
    pub location: Option<CensysLocation>,
    pub autonomous_system: Option<(Option<u64>, Option<String>, Option<String>)>, // (asn, name, description)
    pub operating_system: Option<String>,
    pub last_updated_at: Option<String>,
}

/// Censys service information
#[derive(Debug, Clone)]
pub struct CensysService {
    pub port: u16,
    pub service_name: Option<String>,
    pub transport_protocol: String,
    pub certificate: Option<String>,
}

/// Censys location information
#[derive(Debug, Clone)]
pub struct CensysLocation {
    pub country: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub coordinates: Option<(f64, f64)>, // (latitude, longitude)
}

/// Censys search result
#[derive(Debug, Clone)]
pub struct CensysSearchResult {
    pub total: u64,
    pub hits: Vec<CensysHost>,
}
