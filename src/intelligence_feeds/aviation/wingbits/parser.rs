//! Wingbits response parsers
//!
//! Parse JSON responses to domain types based on Wingbits API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct WingbitsParser;

impl WingbitsParser {
    // ═══════════════════════════════════════════════════════════════════════
    // WINGBITS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse aircraft details
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "icao24": "a12345",
    ///   "registration": "N12345",
    ///   "serial_number": "12345",
    ///   "manufacturer_icao": "BOEING",
    ///   "manufacturer_name": "The Boeing Company",
    ///   "model": "737-800",
    ///   "typecode": "B738",
    ///   "icao_aircraft_type": "L2J",
    ///   "category_description": "Large 2-engine jet",
    ///   "operator": "American Airlines",
    ///   "operator_callsign": "AMERICAN",
    ///   "operator_icao": "AAL",
    ///   "owner": "American Airlines Inc.",
    ///   "built": "2015"
    /// }
    /// ```
    pub fn parse_aircraft_details(data: &Value) -> ExchangeResult<AircraftDetails> {
        let icao24 = Self::require_str(data, "icao24")?.to_string();
        let registration = Self::get_str(data, "registration").map(|s| s.to_string());
        let serial_number = Self::get_str(data, "serial_number").map(|s| s.to_string());
        let manufacturer_icao = Self::get_str(data, "manufacturer_icao").map(|s| s.to_string());
        let manufacturer_name = Self::get_str(data, "manufacturer_name").map(|s| s.to_string());
        let model = Self::get_str(data, "model").map(|s| s.to_string());
        let typecode = Self::get_str(data, "typecode").map(|s| s.to_string());
        let icao_aircraft_type = Self::get_str(data, "icao_aircraft_type").map(|s| s.to_string());
        let category_description = Self::get_str(data, "category_description").map(|s| s.to_string());
        let operator = Self::get_str(data, "operator").map(|s| s.to_string());
        let operator_callsign = Self::get_str(data, "operator_callsign").map(|s| s.to_string());
        let operator_icao = Self::get_str(data, "operator_icao").map(|s| s.to_string());
        let owner = Self::get_str(data, "owner").map(|s| s.to_string());
        let built = Self::get_str(data, "built").map(|s| s.to_string());
        let engines = Self::get_str(data, "engines").map(|s| s.to_string());

        // Determine category based on type code and description
        let category = Self::determine_category(
            typecode.as_deref(),
            category_description.as_deref(),
            operator.as_deref(),
            owner.as_deref(),
        );

        Ok(AircraftDetails {
            icao24,
            registration,
            serial_number,
            manufacturer_icao,
            manufacturer_name,
            model,
            typecode,
            icao_aircraft_type,
            category_description,
            category,
            operator,
            operator_callsign,
            operator_icao,
            owner,
            built,
            engines,
        })
    }

    /// Parse batch aircraft details response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [
    ///     { "icao24": "a12345", ... },
    ///     { "icao24": "a67890", ... }
    ///   ]
    /// }
    /// ```
    pub fn parse_batch_details(data: &Value) -> ExchangeResult<Vec<AircraftDetails>> {
        let results = data
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        let aircraft_list = results
            .iter()
            .filter_map(|aircraft_data| Self::parse_aircraft_details(aircraft_data).ok())
            .collect();

        Ok(aircraft_list)
    }

    /// Determine aircraft category from available data
    fn determine_category(
        typecode: Option<&str>,
        description: Option<&str>,
        operator: Option<&str>,
        owner: Option<&str>,
    ) -> AircraftCategory {
        // Check military keywords first
        if Self::is_military_aircraft(operator, owner) {
            return AircraftCategory::Military;
        }

        // Check typecode patterns
        if let Some(tc) = typecode {
            let tc_upper = tc.to_uppercase();

            // Helicopter patterns
            if tc_upper.contains("H25") || tc_upper.contains("H60") || tc_upper.contains("EC")
                || tc_upper.starts_with("AS") || tc_upper.starts_with("BK") {
                return AircraftCategory::Helicopter;
            }
        }

        // Check description
        if let Some(desc) = description {
            let desc_lower = desc.to_lowercase();

            if desc_lower.contains("jet") {
                return AircraftCategory::Jet;
            }
            if desc_lower.contains("turboprop") {
                return AircraftCategory::Turboprop;
            }
            if desc_lower.contains("piston") {
                return AircraftCategory::Piston;
            }
            if desc_lower.contains("helicopter") || desc_lower.contains("rotorcraft") {
                return AircraftCategory::Helicopter;
            }
            if desc_lower.contains("glider") {
                return AircraftCategory::Glider;
            }
        }

        AircraftCategory::Unknown
    }

    /// Check if aircraft is military based on operator/owner keywords
    fn is_military_aircraft(operator: Option<&str>, owner: Option<&str>) -> bool {
        let military_keywords = [
            "air force",
            "navy",
            "army",
            "marine",
            "military",
            "defense",
            "armed forces",
            "coast guard",
            "national guard",
            "usaf",
            "usn",
            "usmc",
            "raf",
            "luftwaffe",
            "ministere",
        ];

        let check_text = |text: &str| -> bool {
            let text_lower = text.to_lowercase();
            military_keywords.iter().any(|keyword| text_lower.contains(keyword))
        };

        if let Some(op) = operator {
            if check_text(op) {
                return true;
            }
        }

        if let Some(own) = owner {
            if check_text(own) {
                return true;
            }
        }

        false
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

        if let Some(message) = response.get("message") {
            if let Some(msg_str) = message.as_str() {
                // Check if it's an error message
                if msg_str.to_lowercase().contains("error")
                    || msg_str.to_lowercase().contains("not found")
                    || msg_str.to_lowercase().contains("invalid") {
                    return Err(ExchangeError::Api {
                        code: 0,
                        message: msg_str.to_string(),
                    });
                }
            }
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
}

// ═══════════════════════════════════════════════════════════════════════════
// WINGBITS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Aircraft category classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AircraftCategory {
    /// Jet aircraft
    Jet,
    /// Turboprop aircraft
    Turboprop,
    /// Piston engine aircraft
    Piston,
    /// Helicopter/rotorcraft
    Helicopter,
    /// Glider
    Glider,
    /// Military aircraft
    Military,
    /// Unknown or unclassified
    Unknown,
}

/// Aircraft details from Wingbits enrichment
#[derive(Debug, Clone)]
pub struct AircraftDetails {
    /// ICAO 24-bit address (hex, lowercase)
    pub icao24: String,
    /// Aircraft registration/tail number (e.g., "N12345")
    pub registration: Option<String>,
    /// Manufacturer serial number
    pub serial_number: Option<String>,
    /// Manufacturer ICAO code
    pub manufacturer_icao: Option<String>,
    /// Manufacturer full name
    pub manufacturer_name: Option<String>,
    /// Aircraft model (e.g., "737-800")
    pub model: Option<String>,
    /// ICAO type code (e.g., "B738")
    pub typecode: Option<String>,
    /// ICAO aircraft type designator (e.g., "L2J")
    pub icao_aircraft_type: Option<String>,
    /// Category description (e.g., "Large 2-engine jet")
    pub category_description: Option<String>,
    /// Aircraft category
    pub category: AircraftCategory,
    /// Operating airline/organization
    pub operator: Option<String>,
    /// Operator callsign
    pub operator_callsign: Option<String>,
    /// Operator ICAO code
    pub operator_icao: Option<String>,
    /// Owner name
    pub owner: Option<String>,
    /// Year built
    pub built: Option<String>,
    /// Engine information
    pub engines: Option<String>,
}
