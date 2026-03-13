//! Central Bank of Russia (CBR) API endpoints

/// Base URLs for CBR API
pub struct CbrEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for CbrEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.cbr.ru",
            ws_base: None, // CBR does not support WebSocket
        }
    }
}

/// CBR API endpoint enum
#[derive(Debug, Clone)]
pub enum CbrEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // JSON API (v1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get current and historical key rate
    KeyRate,
    /// Get daily exchange rates (JSON)
    DailyJson,

    // ═══════════════════════════════════════════════════════════════════════
    // XML/LEGACY API
    // ═══════════════════════════════════════════════════════════════════════
    /// Get daily exchange rates (XML)
    DailyXml,
    /// List all currencies
    CurrencyList,
    /// Get historical exchange rate for a currency
    ExchangeRateDynamic,
    /// Get precious metal prices
    MetalPrices,
    /// Get repo auction rates
    RepoRates,
    /// Get international reserves (ostat)
    InternationalReserves,
    /// Get monetary base
    MonetaryBase,
    /// Get interbank rates (MKR)
    InterbankRates,

    // ═══════════════════════════════════════════════════════════════════════
    // NEW ENDPOINTS (C6 additions)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get RUONIA overnight rate (Ruble Overnight Index Average)
    RuoniaRate,
    /// Get CBR deposit rates
    DepositRates,
    /// Get historical refinancing rate series
    RefinancingRateHistory,
}

impl CbrEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // JSON API
            Self::KeyRate => "/api/v1/press/keyrate",
            Self::DailyJson => "/api/v1/daily_json",

            // XML/Legacy API
            Self::DailyXml => "/scripts/XML_daily.asp",
            Self::CurrencyList => "/scripts/XML_val.asp",
            Self::ExchangeRateDynamic => "/scripts/XML_dynamic.asp",
            Self::MetalPrices => "/scripts/xml_metall.asp",
            Self::RepoRates => "/scripts/XML_repo.asp",
            Self::InternationalReserves => "/scripts/XML_ostat.asp",
            Self::MonetaryBase => "/scripts/XML_bic.asp",
            Self::InterbankRates => "/scripts/XML_mkr.asp",

            // C6 additions
            Self::RuoniaRate => "/scripts/XML_RuoniaRate.asp",
            Self::DepositRates => "/scripts/XML_DP.asp",
            Self::RefinancingRateHistory => "/api/v1/press/keyrate",
        }
    }
}

/// Format date to CBR format (DD/MM/YYYY)
pub fn format_date_cbr(date: &str) -> String {
    // If input is YYYY-MM-DD, convert to DD/MM/YYYY
    if date.len() == 10 && date.chars().nth(4) == Some('-') {
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
        }
    }
    // Otherwise assume it's already in correct format
    date.to_string()
}

/// Parse date from CBR format (DD/MM/YYYY or DD.MM.YYYY) to YYYY-MM-DD
pub fn parse_date_cbr(date: &str) -> String {
    // Handle both DD/MM/YYYY and DD.MM.YYYY
    let date_clean = date.replace('.', "/");
    let parts: Vec<&str> = date_clean.split('/').collect();
    if parts.len() == 3 {
        return format!("{}-{}-{}", parts[2], parts[1], parts[0]);
    }
    // Fallback
    date.to_string()
}
