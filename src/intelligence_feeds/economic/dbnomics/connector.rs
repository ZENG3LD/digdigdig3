//! DBnomics connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{DBnomicsParser, Provider, Dataset, Series, Observation, LastUpdate};

/// DBnomics connector
///
/// Provides access to economic data from multiple international providers:
/// - IMF (International Monetary Fund)
/// - World Bank
/// - ECB (European Central Bank)
/// - OECD
/// - Eurostat
/// - BIS (Bank for International Settlements)
/// - ILO (International Labour Organization)
/// - And many more
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::dbnomics::DBnomicsConnector;
///
/// let connector = DBnomicsConnector::new();
///
/// // Get all providers
/// let providers = connector.list_providers().await?;
///
/// // Get datasets from IMF
/// let datasets = connector.list_datasets("IMF", None, None).await?;
///
/// // Get a specific series with data
/// let series = connector.get_series("IMF", "IFS", "A.US.PCPI_IX").await?;
/// ```
pub struct DBnomicsConnector {
    client: Client,
    auth: DBnomicsAuth,
    endpoints: DBnomicsEndpoints,
}

impl DBnomicsConnector {
    /// Create new DBnomics connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: DBnomicsAuth::new(),
            endpoints: DBnomicsEndpoints::default(),
        }
    }

    /// Internal: Make GET request to DBnomics API
    async fn get(
        &self,
        endpoint: DBnomicsEndpoint,
        path_params: &[(&str, &str)],
        mut query_params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for DBnomics)
        self.auth.sign_query(&mut query_params);

        // Build path with substitutions
        let path = endpoint.build_path(path_params);
        let url = format!("{}{}", self.endpoints.rest_base, path);

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for DBnomics API errors
        DBnomicsParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PROVIDER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all data providers
    ///
    /// Returns list of available providers (IMF, World Bank, ECB, etc.)
    ///
    /// # Example
    /// ```ignore
    /// let providers = connector.list_providers().await?;
    /// for provider in providers {
    ///     println!("{}: {}", provider.code, provider.name);
    /// }
    /// ```
    pub async fn list_providers(&self) -> ExchangeResult<Vec<Provider>> {
        let response = self.get(DBnomicsEndpoint::Providers, &[], HashMap::new()).await?;
        DBnomicsParser::parse_providers(&response)
    }

    /// Get a specific provider by code
    ///
    /// # Arguments
    /// - `provider_code` - Provider code (e.g., "IMF", "WB", "ECB")
    ///
    /// # Example
    /// ```ignore
    /// let provider = connector.get_provider("IMF").await?;
    /// println!("Provider: {}", provider.name);
    /// ```
    pub async fn get_provider(&self, provider_code: &str) -> ExchangeResult<Provider> {
        let response = self
            .get(
                DBnomicsEndpoint::Provider,
                &[("provider_code", provider_code)],
                HashMap::new(),
            )
            .await?;
        DBnomicsParser::parse_provider(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATASET ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// List datasets for a provider
    ///
    /// # Arguments
    /// - `provider_code` - Provider code (e.g., "IMF", "WB")
    /// - `limit` - Optional limit (default 50, max 1000)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Example
    /// ```ignore
    /// let datasets = connector.list_datasets("IMF", Some(10), None).await?;
    /// ```
    pub async fn list_datasets(
        &self,
        provider_code: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Dataset>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self
            .get(
                DBnomicsEndpoint::Datasets,
                &[("provider_code", provider_code)],
                params,
            )
            .await?;

        DBnomicsParser::parse_datasets(&response)
    }

    /// Get a specific dataset
    ///
    /// # Arguments
    /// - `provider_code` - Provider code (e.g., "IMF")
    /// - `dataset_code` - Dataset code (e.g., "IFS")
    ///
    /// # Example
    /// ```ignore
    /// let dataset = connector.get_dataset("IMF", "IFS").await?;
    /// ```
    pub async fn get_dataset(
        &self,
        provider_code: &str,
        dataset_code: &str,
    ) -> ExchangeResult<Dataset> {
        let response = self
            .get(
                DBnomicsEndpoint::Dataset,
                &[("provider_code", provider_code), ("dataset_code", dataset_code)],
                HashMap::new(),
            )
            .await?;

        DBnomicsParser::parse_dataset(&response)
    }

    /// Search for datasets
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `limit` - Optional limit (default 50, max 1000)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Example
    /// ```ignore
    /// let datasets = connector.search_datasets("inflation", Some(10), None).await?;
    /// ```
    pub async fn search_datasets(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Dataset>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(DBnomicsEndpoint::SearchDatasets, &[], params).await?;
        DBnomicsParser::parse_datasets(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SERIES ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get a series with observations
    ///
    /// # Arguments
    /// - `provider` - Provider code (e.g., "IMF")
    /// - `dataset` - Dataset code (e.g., "IFS")
    /// - `series` - Series code (e.g., "A.US.PCPI_IX")
    ///
    /// # Example
    /// ```ignore
    /// let series = connector.get_series("IMF", "IFS", "A.US.PCPI_IX").await?;
    /// for obs in series.observations {
    ///     println!("{}: {:?}", obs.period, obs.value);
    /// }
    /// ```
    pub async fn get_series(
        &self,
        provider: &str,
        dataset: &str,
        series: &str,
    ) -> ExchangeResult<Series> {
        let response = self
            .get(
                DBnomicsEndpoint::Series,
                &[("provider", provider), ("dataset", dataset), ("series", series)],
                HashMap::new(),
            )
            .await?;

        DBnomicsParser::parse_series(&response)
    }

    /// List series in a dataset
    ///
    /// # Arguments
    /// - `provider` - Provider code
    /// - `dataset` - Dataset code
    /// - `limit` - Optional limit (default 50, max 1000)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Example
    /// ```ignore
    /// let series_list = connector.list_series("IMF", "IFS", Some(20), None).await?;
    /// ```
    pub async fn list_series(
        &self,
        provider: &str,
        dataset: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Series>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self
            .get(
                DBnomicsEndpoint::SeriesList,
                &[("provider", provider), ("dataset", dataset)],
                params,
            )
            .await?;

        DBnomicsParser::parse_series_list(&response)
    }

    /// Search for series
    ///
    /// # Arguments
    /// - `query` - Search query
    /// - `limit` - Optional limit (default 50, max 1000)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Example
    /// ```ignore
    /// let series = connector.search_series("GDP growth", Some(10), None).await?;
    /// ```
    pub async fn search_series(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Series>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(DBnomicsEndpoint::SearchSeries, &[], params).await?;
        DBnomicsParser::parse_series_list(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LAST UPDATES ENDPOINT
    // ═══════════════════════════════════════════════════════════════════════

    /// Get last updates
    ///
    /// Returns recently updated series/datasets
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default 50, max 1000)
    ///
    /// # Example
    /// ```ignore
    /// let updates = connector.get_last_updates(Some(20)).await?;
    /// ```
    pub async fn get_last_updates(&self, limit: Option<u32>) -> ExchangeResult<Vec<LastUpdate>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(DBnomicsEndpoint::LastUpdates, &[], params).await?;
        DBnomicsParser::parse_last_updates(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get latest value from a series
    ///
    /// Returns the most recent observation value.
    ///
    /// # Example
    /// ```ignore
    /// let latest = connector.get_indicator_value("IMF", "IFS", "A.US.PCPI_IX").await?;
    /// ```
    pub async fn get_indicator_value(
        &self,
        provider: &str,
        dataset: &str,
        series: &str,
    ) -> ExchangeResult<Option<f64>> {
        let series_data = self.get_series(provider, dataset, series).await?;

        // Return the last observation value
        Ok(series_data.observations.last().and_then(|obs| obs.value))
    }

    /// Get time series data with date filtering
    ///
    /// Returns observations for a series, optionally filtered by date range.
    ///
    /// # Arguments
    /// - `provider` - Provider code
    /// - `dataset` - Dataset code
    /// - `series` - Series code
    /// - `start_period` - Optional start period (e.g., "2020-01-01")
    /// - `end_period` - Optional end period (e.g., "2023-12-31")
    ///
    /// # Example
    /// ```ignore
    /// let observations = connector
    ///     .get_time_series("IMF", "IFS", "A.US.PCPI_IX", Some("2020-01-01"), Some("2023-12-31"))
    ///     .await?;
    /// ```
    pub async fn get_time_series(
        &self,
        provider: &str,
        dataset: &str,
        series: &str,
        start_period: Option<&str>,
        end_period: Option<&str>,
    ) -> ExchangeResult<Vec<Observation>> {
        let series_data = self.get_series(provider, dataset, series).await?;

        // Filter observations by date range if specified
        let observations = if start_period.is_some() || end_period.is_some() {
            series_data
                .observations
                .into_iter()
                .filter(|obs| {
                    let period_str = &obs.period;

                    let after_start = start_period
                        .map(|start| period_str.as_str() >= start)
                        .unwrap_or(true);

                    let before_end = end_period
                        .map(|end| period_str.as_str() <= end)
                        .unwrap_or(true);

                    after_start && before_end
                })
                .collect()
        } else {
            series_data.observations
        };

        Ok(observations)
    }
}

impl Default for DBnomicsConnector {
    fn default() -> Self {
        Self::new()
    }
}
