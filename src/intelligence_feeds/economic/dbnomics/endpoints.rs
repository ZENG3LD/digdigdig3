//! DBnomics API endpoints

/// Base URLs for DBnomics API
pub struct DBnomicsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for DBnomicsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.db.nomics.world/v22",
            ws_base: None, // DBnomics does not support WebSocket
        }
    }
}

/// DBnomics API endpoint enum
#[derive(Debug, Clone)]
pub enum DBnomicsEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // PROVIDER ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all data providers (IMF, World Bank, ECB, OECD, etc.)
    Providers,
    /// Get a specific provider by code
    Provider,

    // ═══════════════════════════════════════════════════════════════════════
    // DATASET ENDPOINTS (3)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get datasets for a provider
    Datasets,
    /// Get a specific dataset
    Dataset,
    /// Search for datasets
    SearchDatasets,

    // ═══════════════════════════════════════════════════════════════════════
    // SERIES ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get a specific series (with observations)
    Series,
    /// List series in a dataset
    SeriesList,
    /// Search for series
    SearchSeries,
    /// Convert/resolve series ID
    ConvertSeriesId,

    // ═══════════════════════════════════════════════════════════════════════
    // UPDATES ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get last updates
    LastUpdates,
}

impl DBnomicsEndpoint {
    /// Get endpoint path
    ///
    /// Note: Some paths contain dynamic segments (e.g., {provider_code})
    /// which must be replaced at request time
    pub fn path(&self) -> &'static str {
        match self {
            // Providers
            Self::Providers => "/providers",
            Self::Provider => "/providers/{provider_code}",

            // Datasets
            Self::Datasets => "/datasets/{provider_code}",
            Self::Dataset => "/datasets/{provider_code}/{dataset_code}",
            Self::SearchDatasets => "/search/datasets",

            // Series
            Self::Series => "/series/{provider}/{dataset}/{series}",
            Self::SeriesList => "/series/{provider}/{dataset}",
            Self::SearchSeries => "/search/series",
            Self::ConvertSeriesId => "/series",

            // Updates
            Self::LastUpdates => "/last-updates",
        }
    }

    /// Build full path with substitutions
    ///
    /// Replace path parameters with actual values.
    /// Parameters are specified in curly braces: {param_name}
    pub fn build_path(&self, params: &[(&str, &str)]) -> String {
        let mut path = self.path().to_string();

        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            path = path.replace(&placeholder, value);
        }

        path
    }
}

/// Major data providers on DBnomics
///
/// These are the most commonly used providers for economic data.
/// See https://db.nomics.world/ for the complete list.
#[derive(Debug, Clone, Copy)]
pub enum DBnomicsProvider {
    /// International Monetary Fund
    IMF,
    /// World Bank
    WB,
    /// European Central Bank
    ECB,
    /// Organisation for Economic Co-operation and Development
    OECD,
    /// Eurostat (European Union statistics)
    Eurostat,
    /// Bank for International Settlements
    BIS,
    /// International Labour Organization
    ILO,
}

impl DBnomicsProvider {
    /// Get provider code for API requests
    pub fn code(&self) -> &'static str {
        match self {
            Self::IMF => "IMF",
            Self::WB => "WB",
            Self::ECB => "ECB",
            Self::OECD => "OECD",
            Self::Eurostat => "Eurostat",
            Self::BIS => "BIS",
            Self::ILO => "ILO",
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::IMF => "International Monetary Fund",
            Self::WB => "World Bank",
            Self::ECB => "European Central Bank",
            Self::OECD => "Organisation for Economic Co-operation and Development",
            Self::Eurostat => "Eurostat (European Union)",
            Self::BIS => "Bank for International Settlements",
            Self::ILO => "International Labour Organization",
        }
    }
}
