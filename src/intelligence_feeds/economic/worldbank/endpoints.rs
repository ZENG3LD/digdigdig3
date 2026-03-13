//! World Bank API endpoints

/// Base URLs for World Bank API
pub struct WorldBankEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for WorldBankEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.worldbank.org/v2",
            ws_base: None, // World Bank does not support WebSocket
        }
    }
}

/// World Bank API endpoint enum
#[derive(Debug, Clone)]
pub enum WorldBankEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // INDICATOR ENDPOINTS (Core data access)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get indicator data for a country - CORE endpoint
    IndicatorData { country: String, indicator: String },
    /// Get indicator metadata
    Indicator { id: String },
    /// Search for indicators
    IndicatorSearch,
    /// List all indicators (paginated)
    Indicators,
    /// Get indicators by topic
    TopicIndicators { topic_id: String },
    /// Get multiple indicators at once
    MultiIndicatorData { country: String, indicators: String },

    // ═══════════════════════════════════════════════════════════════════════
    // COUNTRY ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get country metadata
    Country { code: String },
    /// List all countries (paginated)
    Countries,
    /// Get countries by income level
    IncomeCountries { level: String },
    /// Get countries by lending type
    LendingCountries { lending_type: String },

    // ═══════════════════════════════════════════════════════════════════════
    // CLASSIFICATION ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get topic metadata
    Topic { id: String },
    /// List all topics
    Topics,
    /// Get source metadata
    Source { id: String },
    /// List all sources
    Sources,
    /// Get income levels
    IncomeLevels,
    /// Get lending types
    LendingTypes,

    // ═══════════════════════════════════════════════════════════════════════
    // C6 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Fetch an indicator for multiple countries in one request using
    /// semicolon-separated country codes: e.g. "US;GB;DE"
    MultiCountryBatch { countries: String, indicator: String },
    /// Fetch sub-national / regional data for a country
    /// Uses the same indicator endpoint but with admin region codes
    SubNationalData { country: String, indicator: String },
}

impl WorldBankEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Indicator endpoints
            Self::IndicatorData { country, indicator } => {
                format!("/country/{}/indicator/{}", country, indicator)
            }
            Self::Indicator { id } => format!("/indicator/{}", id),
            Self::IndicatorSearch => "/indicator".to_string(),
            Self::Indicators => "/indicator".to_string(),
            Self::TopicIndicators { topic_id } => format!("/topic/{}/indicator", topic_id),
            Self::MultiIndicatorData { country, indicators } => {
                format!("/country/{}/indicator/{}", country, indicators)
            }

            // Country endpoints
            Self::Country { code } => format!("/country/{}", code),
            Self::Countries => "/country".to_string(),
            Self::IncomeCountries { level } => format!("/incomelevel/{}/country", level),
            Self::LendingCountries { lending_type } => format!("/lendingtype/{}/country", lending_type),

            // Classification endpoints
            Self::Topic { id } => format!("/topic/{}", id),
            Self::Topics => "/topic".to_string(),
            Self::Source { id } => format!("/source/{}", id),
            Self::Sources => "/source".to_string(),
            Self::IncomeLevels => "/incomelevel".to_string(),
            Self::LendingTypes => "/lendingtype".to_string(),

            // C6 additions
            Self::MultiCountryBatch { countries, indicator } => {
                format!("/country/{}/indicator/{}", countries, indicator)
            }
            Self::SubNationalData { country, indicator } => {
                format!("/country/{}/indicator/{}", country, indicator)
            }
        }
    }
}

/// Common World Bank indicator IDs (for reference)
pub mod indicators {
    /// GDP (current US$)
    pub const GDP: &str = "NY.GDP.MKTP.CD";
    /// GDP growth (annual %)
    pub const GDP_GROWTH: &str = "NY.GDP.MKTP.KD.ZG";
    /// Inflation, consumer prices (annual %)
    pub const INFLATION: &str = "FP.CPI.TOTL.ZG";
    /// Unemployment, total (% of total labor force)
    pub const UNEMPLOYMENT: &str = "SL.UEM.TOTL.ZS";
    /// Population, total
    pub const POPULATION: &str = "SP.POP.TOTL";
    /// GNI per capita, Atlas method (current US$)
    pub const GNI_PER_CAPITA: &str = "NY.GNP.PCAP.CD";
    /// Trade (% of GDP)
    pub const TRADE_GDP: &str = "NE.TRD.GNFS.ZS";
    /// Foreign direct investment, net inflows (% of GDP)
    pub const FDI: &str = "BX.KLT.DINV.WD.GD.ZS";
    /// Exports of goods and services (% of GDP)
    pub const EXPORTS_GDP: &str = "NE.EXP.GNFS.ZS";
    /// Imports of goods and services (% of GDP)
    pub const IMPORTS_GDP: &str = "NE.IMP.GNFS.ZS";
}
