//! AlienVault OTX API endpoints

/// Base URLs for AlienVault OTX API
pub struct OtxEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OtxEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://otx.alienvault.com/api/v1",
            ws_base: None, // OTX does not support WebSocket
        }
    }
}

/// AlienVault OTX API endpoint enum
#[derive(Debug, Clone)]
pub enum OtxEndpoint {
    /// Get subscribed threat intelligence pulses
    SubscribedPulses,
    /// Get recent pulse activity
    PulseActivity,
    /// Get IP reputation for an IPv4 address
    IpReputation { ip: String },
    /// Get domain reputation
    DomainReputation { domain: String },
    /// Get hostname reputation
    HostnameReputation { hostname: String },
    /// Get file hash reputation
    FileReputation { hash: String },
    /// Get URL reputation
    UrlReputation { url: String },
}

impl OtxEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::SubscribedPulses => "/pulses/subscribed".to_string(),
            Self::PulseActivity => "/pulses/activity".to_string(),
            Self::IpReputation { ip } => format!("/indicators/IPv4/{}/general", ip),
            Self::DomainReputation { domain } => format!("/indicators/domain/{}/general", domain),
            Self::HostnameReputation { hostname } => format!("/indicators/hostname/{}/general", hostname),
            Self::FileReputation { hash } => format!("/indicators/file/{}/general", hash),
            Self::UrlReputation { url } => format!("/indicators/url/{}/general", url),
        }
    }
}
