//! Censys API endpoints

/// Base URLs for Censys API
pub struct CensysEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for CensysEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://search.censys.io/api/v2",
            ws_base: None, // Censys does not support WebSocket
        }
    }
}

/// Censys API endpoint enum
#[derive(Debug, Clone)]
pub enum CensysEndpoint {
    /// Search hosts (POST)
    HostsSearch,
    /// View host details
    HostView { ip: String },
    /// Aggregate host data
    HostsAggregate,
    /// Compare host snapshots
    HostsDiff,
    /// Search certificates (POST)
    CertificatesSearch,

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get historical observations for a host (events over time)
    HostEvents { ip: String },
    /// Get hostnames for a host (reverse DNS and SANs)
    HostNames { ip: String },
    /// View a specific certificate by SHA-256 fingerprint
    ViewCertificate { fingerprint: String },
    /// Get hosts that present a specific certificate
    HostsByCert { fingerprint: String },
    /// Aggregate certificate data by field
    AggregateCerts,
    /// Get current account info (quota usage, rate limits)
    AccountInfo,
}

impl CensysEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::HostsSearch => "/hosts/search".to_string(),
            Self::HostView { ip } => format!("/hosts/{}", ip),
            Self::HostsAggregate => "/hosts/aggregate".to_string(),
            Self::HostsDiff => "/hosts/diff".to_string(),
            Self::CertificatesSearch => "/certificates/search".to_string(),

            // C7 additions
            Self::HostEvents { ip } => format!("/hosts/{}/observations", ip),
            Self::HostNames { ip } => format!("/hosts/{}/names", ip),
            Self::ViewCertificate { fingerprint } => format!("/certificates/{}", fingerprint),
            Self::HostsByCert { fingerprint } => format!("/certificates/{}/hosts", fingerprint),
            Self::AggregateCerts => "/certificates/aggregate".to_string(),
            Self::AccountInfo => "/account".to_string(),
        }
    }
}
