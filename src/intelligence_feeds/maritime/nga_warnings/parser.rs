//! NGA Maritime Warnings response parsers
//!
//! Parse JSON responses to domain types based on NGA MSI API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct NgaWarningsParser;

impl NgaWarningsParser {
    // ═══════════════════════════════════════════════════════════════════════
    // BROADCAST WARNINGS PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse broadcast warnings list
    ///
    /// Expected JSON format from NGA MSI API
    pub fn parse_broadcast_warnings(response: &Value) -> ExchangeResult<Vec<MaritimeWarning>> {
        // Check for API error first
        Self::check_error(response)?;

        // Try to extract warnings from various possible response formats
        let warnings_array = if let Some(arr) = response.as_array() {
            arr
        } else if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
            data
        } else if let Some(warnings) = response.get("warnings").and_then(|v| v.as_array()) {
            warnings
        } else if let Some(items) = response.get("items").and_then(|v| v.as_array()) {
            items
        } else {
            return Err(ExchangeError::Parse(
                "Invalid warnings response format - no array found".to_string(),
            ));
        };

        warnings_array
            .iter()
            .map(Self::parse_warning)
            .collect()
    }

    /// Parse a single warning object
    pub fn parse_warning(obj: &Value) -> ExchangeResult<MaritimeWarning> {
        let id = Self::get_str(obj, "id")
            .or_else(|| Self::get_str(obj, "reference"))
            .or_else(|| Self::get_str(obj, "msgNo"))
            .unwrap_or("unknown")
            .to_string();

        let message_number = Self::get_str(obj, "msgNo")
            .or_else(|| Self::get_str(obj, "message_number"))
            .or_else(|| Self::get_str(obj, "messageNumber"))
            .map(|s| s.to_string());

        let year = Self::get_u64(obj, "year")
            .or_else(|| Self::get_u64(obj, "msgYear"))
            .or_else(|| {
                // Try to extract year from message number like "001/26"
                message_number.as_ref().and_then(|mn| {
                    mn.split('/').nth(1).and_then(|y| y.parse::<u64>().ok())
                })
            });

        let area = Self::parse_warning_area(obj);

        let subarea = Self::get_str(obj, "subarea")
            .or_else(|| Self::get_str(obj, "subregion"))
            .or_else(|| Self::get_str(obj, "navArea"))
            .map(|s| s.to_string());

        let text = Self::get_str(obj, "text")
            .or_else(|| Self::get_str(obj, "message"))
            .or_else(|| Self::get_str(obj, "description"))
            .or_else(|| Self::get_str(obj, "content"))
            .unwrap_or("")
            .to_string();

        let status = Self::get_str(obj, "status")
            .unwrap_or("A")
            .to_string();

        let issue_date = Self::get_str(obj, "issueDate")
            .or_else(|| Self::get_str(obj, "issue_date"))
            .or_else(|| Self::get_str(obj, "date"))
            .or_else(|| Self::get_str(obj, "published"))
            .map(|s| s.to_string());

        let cancel_date = Self::get_str(obj, "cancelDate")
            .or_else(|| Self::get_str(obj, "cancel_date"))
            .or_else(|| Self::get_str(obj, "expiry"))
            .or_else(|| Self::get_str(obj, "expires"))
            .map(|s| s.to_string());

        let authority = Self::get_str(obj, "authority")
            .or_else(|| Self::get_str(obj, "issuer"))
            .or_else(|| Self::get_str(obj, "source"))
            .map(|s| s.to_string());

        let warning_type = Self::parse_warning_type(obj);

        Ok(MaritimeWarning {
            id,
            message_number,
            year,
            area,
            subarea,
            text,
            status,
            issue_date,
            cancel_date,
            authority,
            warning_type,
        })
    }

    /// Parse warning type from various fields
    fn parse_warning_type(obj: &Value) -> WarningType {
        let type_str = Self::get_str(obj, "type")
            .or_else(|| Self::get_str(obj, "warningType"))
            .or_else(|| Self::get_str(obj, "category"))
            .unwrap_or("");

        match type_str.to_uppercase().as_str() {
            s if s.contains("HYDROLANT") => WarningType::HYDROLANT,
            s if s.contains("HYDROPAC") => WarningType::HYDROPAC,
            s if s.contains("NAVAREA") => WarningType::NAVAREA,
            s if s.contains("COASTAL") => WarningType::Coastal,
            s if s.contains("LOCAL") => WarningType::Local,
            _ => {
                // Try to infer from other fields
                if let Some(area) = Self::get_str(obj, "area") {
                    match area.to_uppercase().as_str() {
                        s if s.contains("ATLANTIC") => WarningType::HYDROLANT,
                        s if s.contains("PACIFIC") => WarningType::HYDROPAC,
                        s if s.contains("NAVAREA") => WarningType::NAVAREA,
                        _ => WarningType::NAVAREA,
                    }
                } else {
                    WarningType::NAVAREA
                }
            }
        }
    }

    /// Parse warning area from various fields
    fn parse_warning_area(obj: &Value) -> WarningArea {
        let area_str = Self::get_str(obj, "area")
            .or_else(|| Self::get_str(obj, "region"))
            .or_else(|| Self::get_str(obj, "navArea"))
            .unwrap_or("");

        match area_str.to_uppercase().as_str() {
            s if s.contains("ATLANTIC") || s.contains("HYDROLANT") => WarningArea::Atlantic,
            s if s.contains("PACIFIC") || s.contains("HYDROPAC") => WarningArea::Pacific,
            s if s.contains("MEDITERRANEAN") || s.contains("MED") => WarningArea::Mediterranean,
            s if s.contains("INDIAN") => WarningArea::Indian,
            s if s.contains("ARCTIC") => WarningArea::Arctic,
            s if s.contains("CARIBBEAN") => WarningArea::Caribbean,
            s if s.contains("GULF") && s.contains("MEXICO") => WarningArea::GulfOfMexico,
            s if s.contains("BALTIC") => WarningArea::Baltic,
            s if s.contains("BLACK") => WarningArea::BlackSea,
            s if s.contains("RED") => WarningArea::RedSea,
            s if s.contains("PERSIAN") || s.contains("ARABIAN") => WarningArea::PersianGulf,
            _ => WarningArea::Global,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = if let Some(msg) = error.as_str() {
                msg.to_string()
            } else if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
                msg.to_string()
            } else {
                "Unknown error".to_string()
            };

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

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| {
            v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NGA MARITIME WARNINGS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Maritime broadcast warning
#[derive(Debug, Clone)]
pub struct MaritimeWarning {
    /// Warning ID or reference number
    pub id: String,
    /// Message number (e.g., "001/26")
    pub message_number: Option<String>,
    /// Year of the warning
    pub year: Option<u64>,
    /// Geographic area
    pub area: WarningArea,
    /// Subarea or NAVAREA designation
    pub subarea: Option<String>,
    /// Warning text/message content
    pub text: String,
    /// Status (A=Active, C=Cancelled, etc.)
    pub status: String,
    /// Issue date (ISO 8601 or other format)
    pub issue_date: Option<String>,
    /// Cancel/expiry date
    pub cancel_date: Option<String>,
    /// Issuing authority
    pub authority: Option<String>,
    /// Type of warning
    pub warning_type: WarningType,
}

/// Type of maritime warning
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningType {
    /// HYDROLANT - Hydrographic warnings for Atlantic
    HYDROLANT,
    /// HYDROPAC - Hydrographic warnings for Pacific
    HYDROPAC,
    /// NAVAREA - Navigational area warnings
    NAVAREA,
    /// Coastal warnings
    Coastal,
    /// Local warnings
    Local,
}

/// Geographic area for maritime warnings
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningArea {
    /// Atlantic Ocean
    Atlantic,
    /// Pacific Ocean
    Pacific,
    /// Mediterranean Sea
    Mediterranean,
    /// Indian Ocean
    Indian,
    /// Arctic Ocean
    Arctic,
    /// Caribbean Sea
    Caribbean,
    /// Gulf of Mexico
    GulfOfMexico,
    /// Baltic Sea
    Baltic,
    /// Black Sea
    BlackSea,
    /// Red Sea
    RedSea,
    /// Persian Gulf / Arabian Gulf
    PersianGulf,
    /// Global or unspecified
    Global,
}
