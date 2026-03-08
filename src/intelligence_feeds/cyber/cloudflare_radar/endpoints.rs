//! Cloudflare Radar API endpoints

/// Base URLs for Cloudflare Radar API
pub struct CloudflareRadarEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for CloudflareRadarEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.cloudflare.com/client/v4/radar",
            ws_base: None, // Cloudflare Radar does not support WebSocket
        }
    }
}

/// Cloudflare Radar API endpoint enum
#[derive(Debug, Clone)]
pub enum CloudflareRadarEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // HTTP TRAFFIC ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get top locations by HTTP requests
    HttpTopLocations,
    /// Get top ASes by traffic
    HttpTopAses,
    /// Get bot vs human traffic summary
    HttpSummaryBotClass,
    /// Get device type breakdown
    HttpSummaryDeviceType,
    /// Get HTTP protocol versions
    HttpSummaryHttpProtocol,
    /// Get OS breakdown
    HttpSummaryOs,
    /// Get browser breakdown
    HttpSummaryBrowser,
    /// Get HTTP traffic time series
    HttpTimeseries,

    // ═══════════════════════════════════════════════════════════════════════
    // DDOS ATTACK ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get Layer 3 DDoS attack summary
    AttacksLayer3Summary,
    /// Get Layer 7 DDoS attack summary
    AttacksLayer7Summary,
    /// Get Layer 3 attack time series
    AttacksLayer3Timeseries,

    // ═══════════════════════════════════════════════════════════════════════
    // DNS ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get DNS query top locations
    DnsTopLocations,

    // ═══════════════════════════════════════════════════════════════════════
    // RANKING ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get top domains ranking
    RankingTop,
}

impl CloudflareRadarEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // HTTP Traffic
            Self::HttpTopLocations => "/http/top/locations",
            Self::HttpTopAses => "/http/top/ases",
            Self::HttpSummaryBotClass => "/http/summary/bot_class",
            Self::HttpSummaryDeviceType => "/http/summary/device_type",
            Self::HttpSummaryHttpProtocol => "/http/summary/http_protocol",
            Self::HttpSummaryOs => "/http/summary/os",
            Self::HttpSummaryBrowser => "/http/summary/browser",
            Self::HttpTimeseries => "/http/timeseries",

            // DDoS Attacks
            Self::AttacksLayer3Summary => "/attacks/layer3/summary",
            Self::AttacksLayer7Summary => "/attacks/layer7/summary",
            Self::AttacksLayer3Timeseries => "/attacks/layer3/timeseries",

            // DNS
            Self::DnsTopLocations => "/dns/top/locations",

            // Ranking
            Self::RankingTop => "/ranking/top",
        }
    }
}
