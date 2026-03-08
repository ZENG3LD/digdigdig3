//! EIA API endpoints

/// Base URLs for EIA API v2
pub struct EiaEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for EiaEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.eia.gov/v2",
            ws_base: None, // EIA does not support WebSocket
        }
    }
}

/// EIA API endpoint enum
#[derive(Debug, Clone)]
pub enum EiaEndpoint {
    /// Get series data from a route
    /// Pattern: /{route}/data/
    SeriesData { route: String },

    /// Get metadata for a route
    /// Pattern: /{route}/
    RouteMetadata { route: String },

    /// Get available facets for a route
    /// Pattern: /{route}/facets/
    Facets { route: String },
}

impl EiaEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::SeriesData { route } => format!("/{}/data/", route),
            Self::RouteMetadata { route } => format!("/{}/", route),
            Self::Facets { route } => format!("/{}/facets/", route),
        }
    }
}

/// EIA route constants for hierarchical data organization
pub mod routes {
    // ═══════════════════════════════════════════════════════════════════════
    // PETROLEUM
    // ═══════════════════════════════════════════════════════════════════════
    pub const PETROLEUM: &str = "petroleum";
    pub const PETROLEUM_SPOT_PRICES: &str = "petroleum/pri/spt";
    pub const PETROLEUM_WEEKLY_STOCKS: &str = "petroleum/stoc/wstk";
    pub const PETROLEUM_CRUDE_PRODUCTION: &str = "petroleum/crd/crpdn";
    pub const PETROLEUM_IMPORTS: &str = "petroleum/move/imp";

    // ═══════════════════════════════════════════════════════════════════════
    // NATURAL GAS
    // ═══════════════════════════════════════════════════════════════════════
    pub const NATURAL_GAS: &str = "natural-gas";
    pub const NATURAL_GAS_PRICES: &str = "natural-gas/pri/sum";
    pub const NATURAL_GAS_WEEKLY_STORAGE: &str = "natural-gas/stor/wkly";

    // ═══════════════════════════════════════════════════════════════════════
    // ELECTRICITY
    // ═══════════════════════════════════════════════════════════════════════
    pub const ELECTRICITY: &str = "electricity";
    pub const ELECTRICITY_RETAIL_SALES: &str = "electricity/retail-sales";
    pub const ELECTRICITY_GENERATION_BY_FUEL: &str = "electricity/rto/fuel-type-data";

    // ═══════════════════════════════════════════════════════════════════════
    // COAL
    // ═══════════════════════════════════════════════════════════════════════
    pub const COAL: &str = "coal";

    // ═══════════════════════════════════════════════════════════════════════
    // TOTAL ENERGY
    // ═══════════════════════════════════════════════════════════════════════
    pub const TOTAL_ENERGY: &str = "total-energy";

    // ═══════════════════════════════════════════════════════════════════════
    // OUTLOOKS & FORECASTS
    // ═══════════════════════════════════════════════════════════════════════
    pub const STEO: &str = "steo"; // Short-Term Energy Outlook (forecasts!)
    pub const AEO: &str = "aeo"; // Annual Energy Outlook

    // ═══════════════════════════════════════════════════════════════════════
    // INTERNATIONAL
    // ═══════════════════════════════════════════════════════════════════════
    pub const INTERNATIONAL: &str = "international";

    // ═══════════════════════════════════════════════════════════════════════
    // STATE ENERGY DATA
    // ═══════════════════════════════════════════════════════════════════════
    pub const SEDS: &str = "seds"; // State Energy Data System

    // ═══════════════════════════════════════════════════════════════════════
    // OTHER
    // ═══════════════════════════════════════════════════════════════════════
    pub const DENSIFIED_BIOMASS: &str = "densified-biomass";
    pub const CO2_EMISSIONS: &str = "co2-emissions";
}

/// Common product codes for petroleum spot prices
pub mod products {
    pub const BRENT_CRUDE: &str = "EPCBRENT"; // Europe Brent Spot Price FOB
    pub const WTI_CRUDE: &str = "EPCWTI";    // Cushing, OK WTI Spot Price FOB
}

/// Frequency codes for EIA data
#[derive(Debug, Clone, Copy)]
pub enum Frequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annual,
}

impl Frequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::Annual => "annual",
        }
    }
}

/// Sort order for data retrieval
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}
