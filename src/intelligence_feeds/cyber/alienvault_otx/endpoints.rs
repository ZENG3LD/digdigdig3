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

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get a specific pulse by ID
    PulseById { pulse_id: String },
    /// Create a new pulse (POST endpoint)
    PulseCreate,
    /// Get pulses created by a specific user
    UserPulses { username: String },
    /// Full-text search across OTX pulses
    PulseSearch,
    /// Get pulses created by the authenticated user
    MyPulses,
    /// Get all indicators for an IPv4 address (all sections)
    Ipv4Indicators { ip: String, section: String },
    /// Get all indicators for a domain (all sections)
    DomainIndicators { domain: String, section: String },
    /// Get all indicators for a hostname (all sections)
    HostnameIndicators { hostname: String, section: String },
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

            // C7 additions
            Self::PulseById { pulse_id } => format!("/pulses/{}", pulse_id),
            Self::PulseCreate => "/pulses/create".to_string(),
            Self::UserPulses { username } => format!("/users/{}/pulses", username),
            Self::PulseSearch => "/search/pulses".to_string(),
            Self::MyPulses => "/pulses/my".to_string(),
            Self::Ipv4Indicators { ip, section } => format!("/indicators/IPv4/{}/{}", ip, section),
            Self::DomainIndicators { domain, section } => format!("/indicators/domain/{}/{}", domain, section),
            Self::HostnameIndicators { hostname, section } => format!("/indicators/hostname/{}/{}", hostname, section),
        }
    }
}
