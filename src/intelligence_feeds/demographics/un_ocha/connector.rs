//! UN OCHA HAPI connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    UnOchaParser, PopulationData, FoodSecurityData, HumanitarianNeeds,
    OperationalPresence, FundingData, DisplacementData,
};

/// UN OCHA HAPI (Humanitarian API) connector
///
/// Provides access to humanitarian data including population statistics,
/// food security assessments, displacement data, and humanitarian needs.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::un_ocha::UnOchaConnector;
///
/// let connector = UnOchaConnector::new(None);
///
/// // Get population data for a location
/// let population = connector.get_population("AFG").await?;
///
/// // Get food security data
/// let food_security = connector.get_food_security("SOM").await?;
///
/// // Get humanitarian needs
/// let needs = connector.get_humanitarian_needs("SYR").await?;
///
/// // Get refugee data
/// let refugees = connector.get_refugees(Some(2023)).await?;
/// ```
pub struct UnOchaConnector {
    client: Client,
    auth: UnOchaAuth,
    endpoints: UnOchaEndpoints,
}

impl UnOchaConnector {
    /// Create new UN OCHA HAPI connector
    ///
    /// # Arguments
    /// - `app_identifier` - Optional application identifier for tracking
    pub fn new(app_identifier: Option<String>) -> Self {
        Self {
            client: Client::new(),
            auth: UnOchaAuth::new(app_identifier),
            endpoints: UnOchaEndpoints::default(),
        }
    }

    /// Create connector for public access (no app identifier)
    pub fn public() -> Self {
        Self::new(None)
    }

    /// Internal: Make GET request to HAPI API
    async fn get(
        &self,
        endpoint: UnOchaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<String> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Build headers
        let mut headers = HashMap::new();
        self.auth.add_headers(&mut headers);

        // Build reqwest headers
        let mut req_headers = reqwest::header::HeaderMap::new();
        for (key, value) in headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| ExchangeError::Parse(format!("Invalid header name: {}", e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|e| ExchangeError::Parse(format!("Invalid header value: {}", e)))?;
            req_headers.insert(header_name, header_value);
        }

        let response = self
            .client
            .get(&url)
            .headers(req_headers)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {} - {}", status, error_text),
            });
        }

        let json = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(json)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get population data by location
    ///
    /// # Arguments
    /// - `location` - Location code (ISO-3 country code or P-code)
    ///   - Country examples: "AFG" (Afghanistan), "SOM" (Somalia), "SYR" (Syria)
    ///   - Can also use admin-level P-codes for sub-national data
    ///
    /// # Returns
    /// Vector of population data records with demographic breakdowns
    pub async fn get_population(&self, location: &str) -> ExchangeResult<Vec<PopulationData>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        let json = self.get(UnOchaEndpoint::Population, params).await?;
        UnOchaParser::parse_population(&json)
    }

    /// Get food security data by location (IPC classification)
    ///
    /// # Arguments
    /// - `location` - Location code (ISO-3 country code or P-code)
    ///
    /// # Returns
    /// Vector of food security assessments with IPC phases:
    /// - Phase 1: Minimal
    /// - Phase 2: Stressed
    /// - Phase 3: Crisis
    /// - Phase 4: Emergency
    /// - Phase 5: Catastrophe/Famine
    pub async fn get_food_security(
        &self,
        location: &str,
    ) -> ExchangeResult<Vec<FoodSecurityData>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        let json = self.get(UnOchaEndpoint::FoodSecurity, params).await?;
        UnOchaParser::parse_food_security(&json)
    }

    /// Get humanitarian needs by location
    ///
    /// # Arguments
    /// - `location` - Location code (ISO-3 country code)
    ///
    /// # Returns
    /// Vector of humanitarian needs by sector (Health, Shelter, Food, WASH, etc.)
    /// with people in need, targeted, and reached
    pub async fn get_humanitarian_needs(
        &self,
        location: &str,
    ) -> ExchangeResult<Vec<HumanitarianNeeds>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        let json = self.get(UnOchaEndpoint::HumanitarianNeeds, params).await?;
        UnOchaParser::parse_humanitarian_needs(&json)
    }

    /// Get humanitarian needs for a specific sector
    ///
    /// # Arguments
    /// - `location` - Location code
    /// - `sector` - Sector name (e.g., "Health", "Shelter", "Food Security", "WASH", "Protection")
    pub async fn get_needs_by_sector(
        &self,
        location: &str,
        sector: &str,
    ) -> ExchangeResult<Vec<HumanitarianNeeds>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());
        params.insert("sector".to_string(), sector.to_string());

        let json = self.get(UnOchaEndpoint::HumanitarianNeeds, params).await?;
        UnOchaParser::parse_humanitarian_needs(&json)
    }

    /// Get operational presence of humanitarian organizations
    ///
    /// # Arguments
    /// - `location` - Location code
    ///
    /// # Returns
    /// Vector of organizations operating in the location by sector
    pub async fn get_operational_presence(
        &self,
        location: &str,
    ) -> ExchangeResult<Vec<OperationalPresence>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        let json = self.get(UnOchaEndpoint::OperationalPresence, params).await?;
        UnOchaParser::parse_operational_presence(&json)
    }

    /// Get humanitarian funding data
    ///
    /// # Arguments
    /// - `location` - Location code
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// Vector of funding data including requirements, received, and gaps
    pub async fn get_funding(
        &self,
        location: &str,
        year: Option<u32>,
    ) -> ExchangeResult<Vec<FundingData>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        if let Some(y) = year {
            params.insert("year".to_string(), y.to_string());
        }

        let json = self.get(UnOchaEndpoint::Funding, params).await?;
        UnOchaParser::parse_funding(&json)
    }

    /// Get refugee data
    ///
    /// # Arguments
    /// - `year` - Optional year filter (defaults to most recent)
    ///
    /// # Returns
    /// Vector of refugee statistics by country of origin and asylum
    pub async fn get_refugees(&self, year: Option<u32>) -> ExchangeResult<Vec<DisplacementData>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("year".to_string(), y.to_string());
        }

        let json = self.get(UnOchaEndpoint::Refugees, params).await?;
        UnOchaParser::parse_displacement(&json)
    }

    /// Get refugee data for a specific origin country
    ///
    /// # Arguments
    /// - `origin_country` - ISO-3 country code of origin (e.g., "SYR", "AFG", "SSD")
    /// - `year` - Optional year filter
    pub async fn get_refugees_by_origin(
        &self,
        origin_country: &str,
        year: Option<u32>,
    ) -> ExchangeResult<Vec<DisplacementData>> {
        let mut params = HashMap::new();
        params.insert("origin_location_code".to_string(), origin_country.to_string());

        if let Some(y) = year {
            params.insert("year".to_string(), y.to_string());
        }

        let json = self.get(UnOchaEndpoint::Refugees, params).await?;
        UnOchaParser::parse_displacement(&json)
    }

    /// Get Internally Displaced Persons (IDP) data
    ///
    /// # Arguments
    /// - `location` - Location code where people are displaced
    ///
    /// # Returns
    /// Vector of IDP statistics by location
    pub async fn get_idps(&self, location: &str) -> ExchangeResult<Vec<DisplacementData>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        let json = self.get(UnOchaEndpoint::Idps, params).await?;
        UnOchaParser::parse_displacement(&json)
    }

    /// Get returnees data (people returning to their homes)
    ///
    /// # Arguments
    /// - `location` - Location code where people are returning
    ///
    /// # Returns
    /// Vector of returnee statistics
    pub async fn get_returnees(&self, location: &str) -> ExchangeResult<Vec<DisplacementData>> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        let json = self.get(UnOchaEndpoint::Returnees, params).await?;
        UnOchaParser::parse_displacement(&json)
    }

    /// Get comprehensive displacement overview for a country
    ///
    /// Combines refugees (both from and to), IDPs, and returnees data.
    ///
    /// # Arguments
    /// - `location` - Location code
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// Tuple of (refugees_out, refugees_in, idps, returnees)
    pub async fn get_displacement_overview(
        &self,
        location: &str,
        year: Option<u32>,
    ) -> ExchangeResult<(
        Vec<DisplacementData>,
        Vec<DisplacementData>,
        Vec<DisplacementData>,
        Vec<DisplacementData>,
    )> {
        // Get refugees from this country
        let refugees_out = self.get_refugees_by_origin(location, year).await?;

        // Get refugees in this country (asylum)
        let mut params_in = HashMap::new();
        params_in.insert("asylum_location_code".to_string(), location.to_string());
        if let Some(y) = year {
            params_in.insert("year".to_string(), y.to_string());
        }
        let json_in = self.get(UnOchaEndpoint::Refugees, params_in).await?;
        let refugees_in = UnOchaParser::parse_displacement(&json_in)?;

        // Get IDPs
        let idps = self.get_idps(location).await?;

        // Get returnees
        let returnees = self.get_returnees(location).await?;

        Ok((refugees_out, refugees_in, idps, returnees))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get conflict events data for a location
    ///
    /// Returns conflict event records (battles, explosions, etc.) via HAPI.
    /// Powered by ACLED data integrated into OCHA's Humanitarian API.
    ///
    /// # Arguments
    /// - `location` - ISO-3 country code (e.g., "SYR", "SDN")
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// Conflict event records as raw JSON
    pub async fn get_conflict_events(
        &self,
        location: &str,
        year: Option<u32>,
    ) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());
        if let Some(y) = year {
            params.insert("year".to_string(), y.to_string());
        }

        self.get(UnOchaEndpoint::ConflictEvents, params).await
    }

    /// Get national risk data for a country
    ///
    /// Returns the INFORM risk index and component scores for a country.
    ///
    /// # Arguments
    /// - `location` - ISO-3 country code
    ///
    /// # Returns
    /// National risk assessment as raw JSON
    pub async fn get_national_risk(&self, location: &str) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());

        self.get(UnOchaEndpoint::NationalRisk, params).await
    }

    /// Get food prices data for a country (WFP VAM market monitoring)
    ///
    /// Returns food commodity prices from local markets collected as part
    /// of WFP's food security monitoring.
    ///
    /// # Arguments
    /// - `location` - ISO-3 country code
    /// - `commodity` - Optional commodity name filter
    ///
    /// # Returns
    /// Food price records as raw JSON
    pub async fn get_food_prices(
        &self,
        location: &str,
        commodity: Option<&str>,
    ) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("location_code".to_string(), location.to_string());
        if let Some(c) = commodity {
            params.insert("commodity_name".to_string(), c.to_string());
        }

        self.get(UnOchaEndpoint::FoodPrices, params).await
    }
}

impl Default for UnOchaConnector {
    fn default() -> Self {
        Self::public()
    }
}
