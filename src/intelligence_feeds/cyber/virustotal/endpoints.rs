//! VirusTotal API v3 endpoints

/// Base URLs for VirusTotal API
pub struct VirusTotalEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for VirusTotalEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.virustotal.com/api/v3",
            ws_base: None, // VirusTotal does not support WebSocket
        }
    }
}

/// VirusTotal API endpoint enum
#[derive(Debug, Clone)]
pub enum VirusTotalEndpoint {
    /// Get file report by hash (MD5/SHA1/SHA256)
    FileReport { hash: String },
    /// Get URL scan report (id = base64url of URL)
    UrlReport { id: String },
    /// Get domain report
    DomainReport { domain: String },
    /// Get IP address report
    IpReport { ip: String },
    /// Search for files, URLs, domains, IPs
    Search,
}

impl VirusTotalEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::FileReport { hash } => format!("/files/{}", hash),
            Self::UrlReport { id } => format!("/urls/{}", id),
            Self::DomainReport { domain } => format!("/domains/{}", domain),
            Self::IpReport { ip } => format!("/ip_addresses/{}", ip),
            Self::Search => "/search".to_string(),
        }
    }
}
