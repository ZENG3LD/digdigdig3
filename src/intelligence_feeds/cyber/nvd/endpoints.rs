//! NVD API endpoints

/// Base URLs for NVD API
pub struct NvdEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for NvdEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://services.nvd.nist.gov/rest/json",
            ws_base: None, // NVD does not support WebSocket
        }
    }
}

/// NVD API endpoint enum
#[derive(Debug, Clone)]
pub enum NvdEndpoint {
    /// Search CVEs (Common Vulnerabilities and Exposures)
    CvesSearch,
    /// Search CPEs (Common Platform Enumerations)
    CpesSearch,
    /// CPE match strings
    CpeMatch,
}

impl NvdEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::CvesSearch => "/cves/2.0",
            Self::CpesSearch => "/cpes/2.0",
            Self::CpeMatch => "/cpematch/2.0",
        }
    }
}
