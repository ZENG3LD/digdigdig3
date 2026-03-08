//! UNHCR API endpoints

/// Base URLs for UNHCR API
pub struct UnhcrEndpoints {
    pub rest_base: &'static str,
    pub testnet_base: Option<&'static str>,
}

impl UnhcrEndpoints {
    pub fn new(_testnet: bool) -> Self {
        Self {
            rest_base: "https://api.unhcr.org/population/v1",
            testnet_base: None,
        }
    }

    pub fn url(&self, endpoint: &UnhcrEndpoint) -> String {
        format!("{}{}", self.rest_base, endpoint.path())
    }
}

impl Default for UnhcrEndpoints {
    fn default() -> Self {
        Self::new(false)
    }
}

/// UNHCR API endpoint enum
#[derive(Debug, Clone)]
pub enum UnhcrEndpoint {
    /// Get refugee population statistics
    Population,
    /// Get demographic breakdowns
    Demographics,
    /// Get durable solutions data
    Solutions,
    /// Get asylum decisions data
    AsylumDecisions,
    /// Get country list
    Countries,
}

impl UnhcrEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Population => "/population/".to_string(),
            Self::Demographics => "/demographics/".to_string(),
            Self::Solutions => "/solutions/".to_string(),
            Self::AsylumDecisions => "/asylum-decisions/".to_string(),
            Self::Countries => "/countries/".to_string(),
        }
    }
}

/// Format query parameters for UNHCR API
///
/// # Arguments
/// - `year` - Optional year
/// - `country_origin` - Optional country of origin
/// - `country_asylum` - Optional country of asylum
/// - `page` - Optional page number
/// - `limit` - Optional limit (page size)
pub fn format_params(
    year: Option<u32>,
    country_origin: Option<&str>,
    country_asylum: Option<&str>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();

    if let Some(y) = year {
        params.push(("year".to_string(), y.to_string()));
    }
    if let Some(origin) = country_origin {
        params.push(("country_of_origin".to_string(), origin.to_string()));
    }
    if let Some(asylum) = country_asylum {
        params.push(("country_of_asylum".to_string(), asylum.to_string()));
    }
    if let Some(p) = page {
        params.push(("page".to_string(), p.to_string()));
    }
    if let Some(l) = limit {
        params.push(("limit".to_string(), l.to_string()));
    }

    params
}
