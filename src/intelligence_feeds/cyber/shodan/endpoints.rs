//! Shodan API endpoints

/// Base URLs for Shodan API
pub struct ShodanEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ShodanEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.shodan.io",
            ws_base: None, // Shodan does not support WebSocket
        }
    }
}

/// Shodan API endpoint enum
#[derive(Debug, Clone)]
pub enum ShodanEndpoint {
    /// Get host information for an IP address
    HostInfo { ip: String },
    /// Count results for a search query
    HostCount,
    /// Search Shodan for hosts
    HostSearch,
    /// Resolve hostnames to IP addresses
    DnsResolve,
    /// Reverse DNS lookup for IP addresses
    DnsReverse,
    /// Get your current IP address
    MyIp,
    /// Get API plan information
    ApiInfo,
    /// List of ports Shodan crawls
    Ports,
    /// List of protocols Shodan crawls
    Protocols,
}

impl ShodanEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::HostInfo { ip } => format!("/shodan/host/{}", ip),
            Self::HostCount => "/shodan/host/count".to_string(),
            Self::HostSearch => "/shodan/host/search".to_string(),
            Self::DnsResolve => "/dns/resolve".to_string(),
            Self::DnsReverse => "/dns/reverse".to_string(),
            Self::MyIp => "/tools/myip".to_string(),
            Self::ApiInfo => "/api-info".to_string(),
            Self::Ports => "/shodan/ports".to_string(),
            Self::Protocols => "/shodan/protocols".to_string(),
        }
    }
}
