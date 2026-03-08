//! # FeedId
//!
//! Enum identifying each intelligence feed connector.
//! 88 variants covering all supported data providers.

use serde::{Deserialize, Serialize};

/// Unique identifier for each intelligence feed connector.
///
/// Used by [`super::FeedRegistry`] for metadata lookup and by
/// [`super::FeedFactory`] to instantiate concrete connectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedId {
    // ── academic (2) ──────────────────────────────────────────────────────────
    Arxiv,
    SemanticScholar,

    // ── aviation (4) ──────────────────────────────────────────────────────────
    AdsbExchange,
    AviationStack,
    Opensky,
    Wingbits,

    // ── c2intel (1) ───────────────────────────────────────────────────────────
    C2IntelFeeds,

    // ── conflict (5) ──────────────────────────────────────────────────────────
    Acled,
    Gdelt,
    Ucdp,
    ReliefWeb,
    Unhcr,

    // ── corporate (3) ─────────────────────────────────────────────────────────
    Gleif,
    OpenCorporates,
    UkCompaniesHouse,

    // ── crypto (5) ────────────────────────────────────────────────────────────
    Bitquery,
    CoinGecko,
    Coinglass,
    Etherscan,
    WhaleAlert,

    // ── cyber (9) ─────────────────────────────────────────────────────────────
    AbuseIpdb,
    OtxAlienVault,
    Censys,
    CloudflareRadar,
    Nvd,
    RipeNcc,
    Shodan,
    Urlhaus,
    VirusTotal,

    // ── demographics (4) ──────────────────────────────────────────────────────
    UnOcha,
    UnPopulation,
    Who,
    Wikipedia,

    // ── economic (12) ─────────────────────────────────────────────────────────
    Bis,
    Boe,
    Bundesbank,
    Cbr,
    DBnomics,
    Ecb,
    Ecos,
    Eurostat,
    Fred,
    Imf,
    Oecd,
    WorldBank,

    // ── environment (9) ───────────────────────────────────────────────────────
    Gdacs,
    GlobalForestWatch,
    NasaEonet,
    NasaFirms,
    Noaa,
    NwsAlerts,
    OpenWeatherMap,
    OpenAq,
    UsgsEarthquake,

    // ── faa (1) ───────────────────────────────────────────────────────────────
    FaaStatus,

    // ── feodo (1) ─────────────────────────────────────────────────────────────
    FeodoTracker,

    // ── financial (4) ─────────────────────────────────────────────────────────
    AlphaVantage,
    Finnhub,
    NewsApi,
    OpenFigi,

    // ── governance (2) ────────────────────────────────────────────────────────
    EuParliament,
    UkParliament,

    // ── hacker_news (1) ───────────────────────────────────────────────────────
    HackerNews,

    // ── maritime (4) ──────────────────────────────────────────────────────────
    Ais,
    AisStream,
    ImfPortWatch,
    NgaWarnings,

    // ── prediction (1) ────────────────────────────────────────────────────────
    PredictIt,

    // ── rss (1) ───────────────────────────────────────────────────────────────
    RssProxy,

    // ── sanctions (3) ─────────────────────────────────────────────────────────
    Interpol,
    Ofac,
    OpenSanctions,

    // ── space (5) ─────────────────────────────────────────────────────────────
    LaunchLibrary,
    Nasa,
    SentinelHub,
    SpaceTrack,
    SpaceX,

    // ── trade (2) ─────────────────────────────────────────────────────────────
    Comtrade,
    EuTed,

    // ── us_gov (9) ────────────────────────────────────────────────────────────
    Bea,
    Bls,
    Census,
    Congress,
    Eia,
    FbiCrime,
    SamGov,
    SecEdgar,
    UsaSpending,
}

impl FeedId {
    /// Human-readable display name for the feed.
    pub fn name(&self) -> &'static str {
        match self {
            // academic
            Self::Arxiv => "arXiv",
            Self::SemanticScholar => "Semantic Scholar",
            // aviation
            Self::AdsbExchange => "ADS-B Exchange",
            Self::AviationStack => "AviationStack",
            Self::Opensky => "OpenSky Network",
            Self::Wingbits => "Wingbits",
            // c2intel
            Self::C2IntelFeeds => "C2 Intel Feeds",
            // conflict
            Self::Acled => "ACLED",
            Self::Gdelt => "GDELT",
            Self::Ucdp => "UCDP",
            Self::ReliefWeb => "ReliefWeb",
            Self::Unhcr => "UNHCR",
            // corporate
            Self::Gleif => "GLEIF",
            Self::OpenCorporates => "OpenCorporates",
            Self::UkCompaniesHouse => "UK Companies House",
            // crypto
            Self::Bitquery => "Bitquery",
            Self::CoinGecko => "CoinGecko",
            Self::Coinglass => "Coinglass",
            Self::Etherscan => "Etherscan",
            Self::WhaleAlert => "Whale Alert",
            // cyber
            Self::AbuseIpdb => "AbuseIPDB",
            Self::OtxAlienVault => "AlienVault OTX",
            Self::Censys => "Censys",
            Self::CloudflareRadar => "Cloudflare Radar",
            Self::Nvd => "NVD",
            Self::RipeNcc => "RIPE NCC",
            Self::Shodan => "Shodan",
            Self::Urlhaus => "URLhaus",
            Self::VirusTotal => "VirusTotal",
            // demographics
            Self::UnOcha => "UN OCHA",
            Self::UnPopulation => "UN Population",
            Self::Who => "WHO",
            Self::Wikipedia => "Wikipedia",
            // economic
            Self::Bis => "BIS",
            Self::Boe => "Bank of England",
            Self::Bundesbank => "Deutsche Bundesbank",
            Self::Cbr => "Bank of Russia",
            Self::DBnomics => "DBnomics",
            Self::Ecb => "ECB",
            Self::Ecos => "ECOS (Bank of Korea)",
            Self::Eurostat => "Eurostat",
            Self::Fred => "FRED",
            Self::Imf => "IMF",
            Self::Oecd => "OECD",
            Self::WorldBank => "World Bank",
            // environment
            Self::Gdacs => "GDACS",
            Self::GlobalForestWatch => "Global Forest Watch",
            Self::NasaEonet => "NASA EONET",
            Self::NasaFirms => "NASA FIRMS",
            Self::Noaa => "NOAA",
            Self::NwsAlerts => "NWS Alerts",
            Self::OpenWeatherMap => "OpenWeatherMap",
            Self::OpenAq => "OpenAQ",
            Self::UsgsEarthquake => "USGS Earthquake",
            // faa
            Self::FaaStatus => "FAA Status",
            // feodo
            Self::FeodoTracker => "Feodo Tracker",
            // financial
            Self::AlphaVantage => "Alpha Vantage",
            Self::Finnhub => "Finnhub",
            Self::NewsApi => "NewsAPI",
            Self::OpenFigi => "OpenFIGI",
            // governance
            Self::EuParliament => "EU Parliament",
            Self::UkParliament => "UK Parliament",
            // hacker_news
            Self::HackerNews => "Hacker News",
            // maritime
            Self::Ais => "AIS (Datalastic)",
            Self::AisStream => "AISStream",
            Self::ImfPortWatch => "IMF PortWatch",
            Self::NgaWarnings => "NGA Maritime Warnings",
            // prediction
            Self::PredictIt => "PredictIt",
            // rss
            Self::RssProxy => "RSS Proxy",
            // sanctions
            Self::Interpol => "INTERPOL",
            Self::Ofac => "OFAC",
            Self::OpenSanctions => "OpenSanctions",
            // space
            Self::LaunchLibrary => "Launch Library 2",
            Self::Nasa => "NASA",
            Self::SentinelHub => "Sentinel Hub",
            Self::SpaceTrack => "Space-Track",
            Self::SpaceX => "SpaceX",
            // trade
            Self::Comtrade => "UN COMTRADE",
            Self::EuTed => "EU TED",
            // us_gov
            Self::Bea => "BEA",
            Self::Bls => "BLS",
            Self::Census => "US Census",
            Self::Congress => "Congress.gov",
            Self::Eia => "EIA",
            Self::FbiCrime => "FBI Crime Data",
            Self::SamGov => "SAM.gov",
            Self::SecEdgar => "SEC EDGAR",
            Self::UsaSpending => "USASpending.gov",
        }
    }

    /// Lowercase identifier string, suitable for serialization/filenames.
    pub fn as_str(&self) -> &'static str {
        match self {
            // academic
            Self::Arxiv => "arxiv",
            Self::SemanticScholar => "semantic_scholar",
            // aviation
            Self::AdsbExchange => "adsb_exchange",
            Self::AviationStack => "aviation_stack",
            Self::Opensky => "opensky",
            Self::Wingbits => "wingbits",
            // c2intel
            Self::C2IntelFeeds => "c2intel_feeds",
            // conflict
            Self::Acled => "acled",
            Self::Gdelt => "gdelt",
            Self::Ucdp => "ucdp",
            Self::ReliefWeb => "reliefweb",
            Self::Unhcr => "unhcr",
            // corporate
            Self::Gleif => "gleif",
            Self::OpenCorporates => "opencorporates",
            Self::UkCompaniesHouse => "uk_companies_house",
            // crypto
            Self::Bitquery => "bitquery",
            Self::CoinGecko => "coingecko",
            Self::Coinglass => "coinglass",
            Self::Etherscan => "etherscan",
            Self::WhaleAlert => "whale_alert",
            // cyber
            Self::AbuseIpdb => "abuseipdb",
            Self::OtxAlienVault => "otx_alienvault",
            Self::Censys => "censys",
            Self::CloudflareRadar => "cloudflare_radar",
            Self::Nvd => "nvd",
            Self::RipeNcc => "ripe_ncc",
            Self::Shodan => "shodan",
            Self::Urlhaus => "urlhaus",
            Self::VirusTotal => "virustotal",
            // demographics
            Self::UnOcha => "un_ocha",
            Self::UnPopulation => "un_population",
            Self::Who => "who",
            Self::Wikipedia => "wikipedia",
            // economic
            Self::Bis => "bis",
            Self::Boe => "boe",
            Self::Bundesbank => "bundesbank",
            Self::Cbr => "cbr",
            Self::DBnomics => "dbnomics",
            Self::Ecb => "ecb",
            Self::Ecos => "ecos",
            Self::Eurostat => "eurostat",
            Self::Fred => "fred",
            Self::Imf => "imf",
            Self::Oecd => "oecd",
            Self::WorldBank => "world_bank",
            // environment
            Self::Gdacs => "gdacs",
            Self::GlobalForestWatch => "global_forest_watch",
            Self::NasaEonet => "nasa_eonet",
            Self::NasaFirms => "nasa_firms",
            Self::Noaa => "noaa",
            Self::NwsAlerts => "nws_alerts",
            Self::OpenWeatherMap => "open_weather_map",
            Self::OpenAq => "openaq",
            Self::UsgsEarthquake => "usgs_earthquake",
            // faa
            Self::FaaStatus => "faa_status",
            // feodo
            Self::FeodoTracker => "feodo_tracker",
            // financial
            Self::AlphaVantage => "alpha_vantage",
            Self::Finnhub => "finnhub",
            Self::NewsApi => "newsapi",
            Self::OpenFigi => "openfigi",
            // governance
            Self::EuParliament => "eu_parliament",
            Self::UkParliament => "uk_parliament",
            // hacker_news
            Self::HackerNews => "hacker_news",
            // maritime
            Self::Ais => "ais",
            Self::AisStream => "aisstream",
            Self::ImfPortWatch => "imf_portwatch",
            Self::NgaWarnings => "nga_warnings",
            // prediction
            Self::PredictIt => "predictit",
            // rss
            Self::RssProxy => "rss_proxy",
            // sanctions
            Self::Interpol => "interpol",
            Self::Ofac => "ofac",
            Self::OpenSanctions => "opensanctions",
            // space
            Self::LaunchLibrary => "launch_library",
            Self::Nasa => "nasa",
            Self::SentinelHub => "sentinel_hub",
            Self::SpaceTrack => "space_track",
            Self::SpaceX => "spacex",
            // trade
            Self::Comtrade => "comtrade",
            Self::EuTed => "eu_ted",
            // us_gov
            Self::Bea => "bea",
            Self::Bls => "bls",
            Self::Census => "census",
            Self::Congress => "congress",
            Self::Eia => "eia",
            Self::FbiCrime => "fbi_crime",
            Self::SamGov => "sam_gov",
            Self::SecEdgar => "sec_edgar",
            Self::UsaSpending => "usaspending",
        }
    }
}
