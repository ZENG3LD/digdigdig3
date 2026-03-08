//! UN OCHA HAPI API endpoints

/// Base URLs for UN OCHA HAPI API
pub struct UnOchaEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for UnOchaEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://hapi.humdata.org/api/v1",
            ws_base: None, // HAPI does not support WebSocket
        }
    }
}

/// UN OCHA HAPI API endpoint enum
#[derive(Debug, Clone)]
pub enum UnOchaEndpoint {
    /// Population data by location
    Population,
    /// Food security data (IPC phases)
    FoodSecurity,
    /// Humanitarian needs by sector and location
    HumanitarianNeeds,
    /// Operational presence of organizations
    OperationalPresence,
    /// Humanitarian funding data
    Funding,
    /// Refugee data by country of origin and asylum
    Refugees,
    /// Internally Displaced Persons data
    Idps,
    /// Returnees data
    Returnees,
}

impl UnOchaEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Population => "/population",
            Self::FoodSecurity => "/food-security",
            Self::HumanitarianNeeds => "/humanitarian-needs",
            Self::OperationalPresence => "/operational-presence",
            Self::Funding => "/funding",
            Self::Refugees => "/refugees",
            Self::Idps => "/idps",
            Self::Returnees => "/returnees",
        }
    }
}
