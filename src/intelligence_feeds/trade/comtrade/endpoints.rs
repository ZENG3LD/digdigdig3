//! UN COMTRADE API endpoints

/// Base URLs for COMTRADE API
pub struct ComtradeEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ComtradeEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://comtradeapi.un.org",
            ws_base: None, // COMTRADE does not support WebSocket
        }
    }
}

/// COMTRADE API endpoint enum
#[derive(Debug, Clone)]
pub enum ComtradeEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get trade data (authenticated)
    /// Path: /data/v1/get/{typeCode}/{freqCode}/{clCode}
    GetTradeData {
        type_code: String,
        freq_code: String,
        cl_code: String,
    },
    /// Preview trade data (no auth needed)
    /// Path: /data/v1/preview/{typeCode}/{freqCode}/{clCode}
    PreviewTradeData {
        type_code: String,
        freq_code: String,
        cl_code: String,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS (6)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get list of reporter countries
    /// Path: /public/v1/getLOV/reporterCode
    GetReporters,
    /// Get list of partner countries
    /// Path: /public/v1/getLOV/partnerCode
    GetPartners,
    /// Get commodity codes for a classification system
    /// Path: /public/v1/getLOV/cmdCode/{classification}
    GetCommodityCodes { classification: String },
    /// Get flow codes (import/export/re-export)
    /// Path: /public/v1/getLOV/flowCode
    GetFlowCodes,
    /// Get type codes (commodities/services)
    /// Path: /public/v1/getLOV/typeCode
    GetTypeCodes,
    /// Get frequency codes (annual/monthly)
    /// Path: /public/v1/getLOV/freqCode
    GetFreqCodes,
}

impl ComtradeEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Data
            Self::GetTradeData {
                type_code,
                freq_code,
                cl_code,
            } => format!("/data/v1/get/{}/{}/{}", type_code, freq_code, cl_code),
            Self::PreviewTradeData {
                type_code,
                freq_code,
                cl_code,
            } => format!(
                "/data/v1/preview/{}/{}/{}",
                type_code, freq_code, cl_code
            ),

            // Metadata
            Self::GetReporters => "/public/v1/getLOV/reporterCode".to_string(),
            Self::GetPartners => "/public/v1/getLOV/partnerCode".to_string(),
            Self::GetCommodityCodes { classification } => {
                format!("/public/v1/getLOV/cmdCode/{}", classification)
            }
            Self::GetFlowCodes => "/public/v1/getLOV/flowCode".to_string(),
            Self::GetTypeCodes => "/public/v1/getLOV/typeCode".to_string(),
            Self::GetFreqCodes => "/public/v1/getLOV/freqCode".to_string(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Type codes
pub const TYPE_COMMODITIES: &str = "C";
pub const TYPE_SERVICES: &str = "S";

/// Frequency codes
pub const FREQ_ANNUAL: &str = "A";
pub const FREQ_MONTHLY: &str = "M";

/// Classification codes
pub const CL_HS: &str = "HS"; // Harmonized System
pub const CL_SITC: &str = "SITC"; // Standard International Trade Classification

/// Flow codes
pub const FLOW_EXPORT: &str = "X";
pub const FLOW_IMPORT: &str = "M";
pub const FLOW_REEXPORT: &str = "RX";

/// Key country codes
pub const COUNTRY_US: u32 = 842;
pub const COUNTRY_CHINA: u32 = 156;
pub const COUNTRY_GERMANY: u32 = 276;
pub const COUNTRY_JAPAN: u32 = 392;
pub const COUNTRY_UK: u32 = 826;
pub const COUNTRY_RUSSIA: u32 = 643;
pub const COUNTRY_INDIA: u32 = 356;
pub const COUNTRY_KOREA: u32 = 410;

/// Key HS commodity codes
pub const HS_MINERAL_FUELS: &str = "27"; // Oil, gas
pub const HS_MACHINERY: &str = "84";
pub const HS_ELECTRONICS: &str = "85";
pub const HS_VEHICLES: &str = "87";
pub const HS_PRECIOUS_METALS: &str = "71";
