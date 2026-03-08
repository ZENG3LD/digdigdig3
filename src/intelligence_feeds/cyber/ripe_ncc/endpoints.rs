//! RIPE NCC API endpoints

/// Base URLs for RIPE NCC API
pub struct RipeNccEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for RipeNccEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://stat.ripe.net/data",
            ws_base: None, // RIPE NCC does not support WebSocket
        }
    }
}

/// RIPE NCC API endpoint enum
#[derive(Debug, Clone)]
pub enum RipeNccEndpoint {
    /// Get country internet resources statistics
    CountryResourceStats,
    /// Get ASN overview
    AsOverview,
    /// Get routing status for prefix/IP
    RoutingStatus,
    /// Get BGP state for prefix/IP
    BgpState,
    /// Get announced prefixes by ASN
    AnnouncedPrefixes,
    /// Get ASN neighbors/peers
    AsnNeighbours,
    /// Get network info for IP address
    NetworkInfo,
    /// Get RIR allocation stats by country
    RirStatsCountry,
    /// Get full list of country resources
    CountryResourceList,
    /// Get abuse contact for IP/prefix
    AbuseContactFinder,
}

impl RipeNccEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::CountryResourceStats => "/country-resource-stats/data.json",
            Self::AsOverview => "/as-overview/data.json",
            Self::RoutingStatus => "/routing-status/data.json",
            Self::BgpState => "/bgp-state/data.json",
            Self::AnnouncedPrefixes => "/announced-prefixes/data.json",
            Self::AsnNeighbours => "/asn-neighbours/data.json",
            Self::NetworkInfo => "/network-info/data.json",
            Self::RirStatsCountry => "/rir-stats-country/data.json",
            Self::CountryResourceList => "/country-resource-list/data.json",
            Self::AbuseContactFinder => "/abuse-contact-finder/data.json",
        }
    }
}
