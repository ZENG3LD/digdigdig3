//! Global Forest Watch API endpoints

/// Base URLs for GFW API
pub struct GfwEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for GfwEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://data-api.globalforestwatch.org",
            ws_base: None, // GFW does not support WebSocket
        }
    }
}

/// GFW API endpoint enum
#[derive(Debug, Clone)]
pub enum GfwEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DATASET ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List all datasets
    Datasets,
    /// Get dataset details by ID
    Dataset,
    /// Get latest version of a dataset
    DatasetLatest,
    /// Query a specific dataset version
    DatasetQuery,

    // ═══════════════════════════════════════════════════════════════════════
    // FOREST CHANGE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get forest change statistics by region
    ForestChangeStatistics,
    /// Get tree cover loss data
    TreeCoverLoss,
    /// Get tree cover gain data
    TreeCoverGain,
    /// Get fire alerts
    FireAlerts,
    /// Get deforestation alerts
    DeforestationAlerts,
}

impl GfwEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Datasets
            Self::Datasets => "/dataset".to_string(),
            Self::Dataset => "/dataset/{id}".to_string(),
            Self::DatasetLatest => "/dataset/{id}/latest".to_string(),
            Self::DatasetQuery => "/dataset/{dataset_id}/{version}/query".to_string(),

            // Forest change
            Self::ForestChangeStatistics => "/forest-change/statistics".to_string(),
            Self::TreeCoverLoss => "/forest-change/loss".to_string(),
            Self::TreeCoverGain => "/forest-change/gain".to_string(),
            Self::FireAlerts => "/fire-alerts".to_string(),
            Self::DeforestationAlerts => "/deforestation-alerts".to_string(),
        }
    }

    /// Replace path parameters
    pub fn with_params(&self, id: Option<&str>, dataset_id: Option<&str>, version: Option<&str>) -> String {
        let mut path = self.path();

        if let Some(id_val) = id {
            path = path.replace("{id}", id_val);
        }
        if let Some(dataset_val) = dataset_id {
            path = path.replace("{dataset_id}", dataset_val);
        }
        if let Some(version_val) = version {
            path = path.replace("{version}", version_val);
        }

        path
    }
}
