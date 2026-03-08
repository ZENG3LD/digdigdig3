//! EIA connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{EiaParser, EiaObservation, EiaMetadata, EiaFacet};

/// EIA (U.S. Energy Information Administration) connector
///
/// Provides access to U.S. energy data including petroleum, natural gas, electricity, coal, and forecasts.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::eia::EiaConnector;
///
/// let connector = EiaConnector::from_env();
///
/// // Get crude oil prices
/// let prices = connector.get_crude_oil_prices(Some("2024-01-01"), Some("2024-12-31")).await?;
///
/// // Get natural gas storage data
/// let storage = connector.get_gas_storage(None, None).await?;
///
/// // Get custom series data
/// let data = connector.get_series_data("petroleum/pri/spt", None, None, None, None, None, None, None).await?;
/// ```
pub struct EiaConnector {
    client: Client,
    auth: EiaAuth,
    endpoints: EiaEndpoints,
}

impl EiaConnector {
    /// Create new EIA connector with authentication
    pub fn new(auth: EiaAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: EiaEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `EIA_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(EiaAuth::from_env())
    }

    /// Internal: Make GET request to EIA API
    async fn get(
        &self,
        endpoint: EiaEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .query(&params)
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

        // Check for EIA API errors
        EiaParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EIA-SPECIFIC METHODS (Core API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get series data from a route
    ///
    /// This is the CORE endpoint for retrieving time series data from EIA.
    ///
    /// # Arguments
    /// - `route` - Data route (e.g., "petroleum/pri/spt", "natural-gas/pri/sum")
    /// - `frequency` - Optional data frequency (hourly, daily, weekly, monthly, quarterly, annual)
    /// - `data_columns` - Optional data columns to retrieve (e.g., vec!["value"])
    /// - `facets` - Optional facet filters (e.g., HashMap from facet name to values)
    /// - `start` - Optional start date (YYYY-MM-DD format)
    /// - `end` - Optional end date (YYYY-MM-DD format)
    /// - `sort` - Optional sort order (ascending or descending)
    /// - `length` - Optional limit on number of records
    ///
    /// # Example
    /// ```ignore
    /// let mut facets = HashMap::new();
    /// facets.insert("product".to_string(), vec!["EPCBRENT".to_string()]);
    /// let data = connector.get_series_data(
    ///     "petroleum/pri/spt",
    ///     Some(Frequency::Daily),
    ///     None,
    ///     Some(facets),
    ///     Some("2024-01-01"),
    ///     Some("2024-12-31"),
    ///     None,
    ///     None
    /// ).await?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub async fn get_series_data(
        &self,
        route: &str,
        frequency: Option<Frequency>,
        data_columns: Option<Vec<&str>>,
        facets: Option<HashMap<String, Vec<String>>>,
        start: Option<&str>,
        end: Option<&str>,
        sort: Option<SortOrder>,
        length: Option<u32>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        let mut params = HashMap::new();

        if let Some(freq) = frequency {
            params.insert("frequency".to_string(), freq.as_str().to_string());
        }

        if let Some(cols) = data_columns {
            for col in cols {
                params.insert("data[]".to_string(), col.to_string());
            }
        }

        if let Some(facet_map) = facets {
            for (facet_name, values) in facet_map {
                for value in values {
                    let key = format!("facets[{}][]", facet_name);
                    params.insert(key, value);
                }
            }
        }

        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }

        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }

        if let Some(sort_order) = sort {
            params.insert("sort".to_string(), sort_order.as_str().to_string());
        }

        if let Some(len) = length {
            params.insert("length".to_string(), len.to_string());
        }

        let endpoint = EiaEndpoint::SeriesData {
            route: route.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        EiaParser::parse_data(&response)
    }

    /// Get metadata for a route
    ///
    /// Returns information about available frequencies, data columns, facets, etc.
    pub async fn get_route_metadata(&self, route: &str) -> ExchangeResult<EiaMetadata> {
        let params = HashMap::new();
        let endpoint = EiaEndpoint::RouteMetadata {
            route: route.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        EiaParser::parse_metadata(&response)
    }

    /// Get available facets for a route
    ///
    /// Facets are filter dimensions (e.g., product types, regions, fuel types)
    pub async fn get_facets(&self, route: &str) -> ExchangeResult<Vec<EiaFacet>> {
        let params = HashMap::new();
        let endpoint = EiaEndpoint::Facets {
            route: route.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        EiaParser::parse_facets(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS (Common Data Series)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get crude oil spot prices (Brent and WTI)
    ///
    /// Default: Daily frequency, all products (Brent + WTI)
    pub async fn get_crude_oil_prices(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::PETROLEUM_SPOT_PRICES,
            Some(Frequency::Daily),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get natural gas prices
    ///
    /// Default: Monthly frequency
    pub async fn get_natural_gas_prices(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::NATURAL_GAS_PRICES,
            Some(Frequency::Monthly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get weekly petroleum stocks
    pub async fn get_weekly_petroleum_stocks(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::PETROLEUM_WEEKLY_STOCKS,
            Some(Frequency::Weekly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get crude oil production data
    pub async fn get_crude_production(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::PETROLEUM_CRUDE_PRODUCTION,
            Some(Frequency::Monthly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get natural gas storage data (weekly)
    pub async fn get_gas_storage(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::NATURAL_GAS_WEEKLY_STORAGE,
            Some(Frequency::Weekly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get electricity generation by fuel type
    pub async fn get_electricity_generation(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::ELECTRICITY_GENERATION_BY_FUEL,
            Some(Frequency::Hourly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get Short-Term Energy Outlook (STEO) forecasts
    ///
    /// STEO contains EIA's official forecasts for energy markets
    pub async fn get_steo_forecast(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::STEO,
            Some(Frequency::Monthly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get CO2 emissions data
    pub async fn get_co2_emissions(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::CO2_EMISSIONS,
            Some(Frequency::Annual),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get international energy data
    ///
    /// Requires sub-route specification (e.g., "international/petroleum")
    pub async fn get_international_data(
        &self,
        route: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        let full_route = if route.starts_with("international/") {
            route.to_string()
        } else {
            format!("international/{}", route)
        };

        self.get_series_data(
            &full_route,
            Some(Frequency::Monthly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }

    /// Get coal data
    pub async fn get_coal_data(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<EiaObservation>> {
        self.get_series_data(
            routes::COAL,
            Some(Frequency::Monthly),
            None,
            None,
            start,
            end,
            None,
            None,
        )
        .await
    }
}
