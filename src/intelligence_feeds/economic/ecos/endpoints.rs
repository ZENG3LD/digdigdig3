//! Bank of Korea ECOS API endpoints

/// Base URLs for ECOS API
pub struct EcosEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for EcosEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://ecos.bok.or.kr/api",
            ws_base: None, // ECOS does not support WebSocket
        }
    }
}

/// ECOS API endpoint enum
#[derive(Debug, Clone)]
pub enum EcosEndpoint {
    /// Get statistical data by stat code, cycle, and date range
    StatisticSearch,
    /// Get list of key statistics
    KeyStatisticList,
    /// Get statistical table list by stat code
    StatisticTableList,
    /// Get statistical item list by stat code
    StatisticItemList,
    /// Search statistics by keyword
    StatisticWord,
    /// Get statistical metadata by data name
    StatMeta,
}

impl EcosEndpoint {
    /// Get endpoint path segment (without API key)
    pub fn service_name(&self) -> &'static str {
        match self {
            Self::StatisticSearch => "StatisticSearch",
            Self::KeyStatisticList => "KeyStatisticList",
            Self::StatisticTableList => "StatisticTableList",
            Self::StatisticItemList => "StatisticItemList",
            Self::StatisticWord => "StatisticWord",
            Self::StatMeta => "StatMeta",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// COMMON STAT CODES
// ═══════════════════════════════════════════════════════════════════════

/// Gross Domestic Product
pub const STAT_GDP: &str = "200Y001";

/// Consumer Price Index
pub const STAT_CPI: &str = "901Y009";

/// Base Rate (Policy Rate)
pub const STAT_POLICY_RATE: &str = "722Y001";

/// Exchange Rates
pub const STAT_EXCHANGE_RATES: &str = "731Y001";

/// Employment
pub const STAT_EMPLOYMENT: &str = "901Y027";

/// Monetary Aggregates (Money Supply)
pub const STAT_MONEY_SUPPLY: &str = "101Y004";

/// Trade Balance
pub const STAT_TRADE: &str = "403Y003";

/// Industrial Production Index
pub const STAT_INDUSTRIAL_PRODUCTION: &str = "901Y033";

// ═══════════════════════════════════════════════════════════════════════
// CYCLE CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Annual frequency
pub const CYCLE_ANNUAL: &str = "A";

/// Quarterly frequency
pub const CYCLE_QUARTERLY: &str = "Q";

/// Monthly frequency
pub const CYCLE_MONTHLY: &str = "M";

/// Daily frequency
pub const CYCLE_DAILY: &str = "D";
