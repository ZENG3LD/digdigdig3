//! AbuseIPDB API endpoints

/// Base URLs for AbuseIPDB API
pub struct AbuseIpdbEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for AbuseIpdbEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.abuseipdb.com/api/v2",
            ws_base: None, // AbuseIPDB does not support WebSocket
        }
    }
}

/// AbuseIPDB API endpoint enum
#[derive(Debug, Clone)]
pub enum AbuseIpdbEndpoint {
    /// Check IP address for abuse reports
    Check,
    /// Get blacklist of malicious IP addresses
    Blacklist,
    /// Report an IP address for abuse
    Report,
    /// Check a network block (CIDR notation)
    CheckBlock,
    /// Submit multiple IP reports in bulk
    BulkReport,
    /// Clear own IP address from reports
    ClearAddress,
    /// Get list of abuse categories
    Categories,
}

impl AbuseIpdbEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Check => "/check",
            Self::Blacklist => "/blacklist",
            Self::Report => "/report",
            Self::CheckBlock => "/check-block",
            Self::BulkReport => "/bulk-report",
            Self::ClearAddress => "/clear-address",
            Self::Categories => "/categories",
        }
    }
}
