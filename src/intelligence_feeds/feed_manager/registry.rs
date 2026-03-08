//! # Feed Registry
//!
//! Static metadata registry for all 88 intelligence feed connectors.
//! Provides O(1) lookup by FeedId and filtering by category/auth type.

use super::feed_id::FeedId;

// ═══════════════════════════════════════════════════════════════════════════════
// SUPPORTING TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Thematic category for grouping feeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedCategory {
    Academic,
    Aviation,
    C2Intel,
    Conflict,
    Corporate,
    Crypto,
    Cyber,
    Demographics,
    Economic,
    Environment,
    Faa,
    Feodo,
    Financial,
    Governance,
    HackerNews,
    Maritime,
    Prediction,
    Rss,
    Sanctions,
    Space,
    Trade,
    UsGov,
}

/// Authentication requirement for a feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedAuthType {
    /// No authentication required (33 feeds).
    None,
    /// Has an anonymous/free/public fallback but benefits from a key (10 feeds).
    Optional,
    /// Requires a single API key.
    ApiKey,
    /// Requires two keys: API ID + API Secret (Censys only).
    ApiKeyPair,
    /// Uses [`crate::core::Credentials`] — Bitquery, Coinglass.
    Credentials,
    /// Username + password — SpaceTrack, OpenSky.
    UsernamePassword,
}

/// Static metadata describing a single intelligence feed connector.
pub struct FeedMetadata {
    /// Unique identifier.
    pub id: FeedId,
    /// Human-readable name.
    pub name: &'static str,
    /// Thematic category.
    pub category: FeedCategory,
    /// Authentication requirement.
    pub auth_type: FeedAuthType,
    /// Short description of what this feed provides.
    pub description: &'static str,
    /// Primary REST API base URL.
    pub base_url: &'static str,
}

// ═══════════════════════════════════════════════════════════════════════════════
// STATIC REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Complete static array of all 88 feed metadata entries.
pub const FEEDS: &[FeedMetadata] = &[
    // ── academic ──────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Arxiv,
        name: "arXiv",
        category: FeedCategory::Academic,
        auth_type: FeedAuthType::None,
        description: "arXiv preprint server — 2M+ research papers in physics, math, CS, finance",
        base_url: "https://export.arxiv.org/api",
    },
    FeedMetadata {
        id: FeedId::SemanticScholar,
        name: "Semantic Scholar",
        category: FeedCategory::Academic,
        auth_type: FeedAuthType::Optional,
        description: "Semantic Scholar academic graph — 200M+ papers with citations and embeddings",
        base_url: "https://api.semanticscholar.org/graph/v1",
    },
    // ── aviation ──────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::AdsbExchange,
        name: "ADS-B Exchange",
        category: FeedCategory::Aviation,
        auth_type: FeedAuthType::ApiKey,
        description: "ADS-B Exchange real-time aircraft positions via RapidAPI",
        base_url: "https://adsbexchange-com1.p.rapidapi.com/v2",
    },
    FeedMetadata {
        id: FeedId::AviationStack,
        name: "AviationStack",
        category: FeedCategory::Aviation,
        auth_type: FeedAuthType::ApiKey,
        description: "AviationStack real-time flight data, airport info, airline schedules",
        base_url: "https://api.aviationstack.com/v1",
    },
    FeedMetadata {
        id: FeedId::Opensky,
        name: "OpenSky Network",
        category: FeedCategory::Aviation,
        auth_type: FeedAuthType::Optional,
        description: "OpenSky Network free ADS-B data — anonymous or authenticated access",
        base_url: "https://opensky-network.org/api",
    },
    FeedMetadata {
        id: FeedId::Wingbits,
        name: "Wingbits",
        category: FeedCategory::Aviation,
        auth_type: FeedAuthType::ApiKey,
        description: "Wingbits decentralized ADS-B network with token rewards",
        base_url: "https://api.wingbits.com/v1",
    },
    // ── c2intel ───────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::C2IntelFeeds,
        name: "C2 Intel Feeds",
        category: FeedCategory::C2Intel,
        auth_type: FeedAuthType::None,
        description: "C2IntelFeeds GitHub — community-sourced C2 server indicators",
        base_url: "https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master",
    },
    // ── conflict ──────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Acled,
        name: "ACLED",
        category: FeedCategory::Conflict,
        auth_type: FeedAuthType::ApiKey,
        description: "Armed Conflict Location & Event Data — global conflict events database",
        base_url: "https://api.acleddata.com",
    },
    FeedMetadata {
        id: FeedId::Gdelt,
        name: "GDELT",
        category: FeedCategory::Conflict,
        auth_type: FeedAuthType::None,
        description: "GDELT global event database — geopolitical events from news worldwide",
        base_url: "https://api.gdeltproject.org/api/v2",
    },
    FeedMetadata {
        id: FeedId::Ucdp,
        name: "UCDP",
        category: FeedCategory::Conflict,
        auth_type: FeedAuthType::None,
        description: "Uppsala Conflict Data Program — armed conflict and peace data",
        base_url: "https://ucdpapi.pcr.uu.se/api",
    },
    FeedMetadata {
        id: FeedId::ReliefWeb,
        name: "ReliefWeb",
        category: FeedCategory::Conflict,
        auth_type: FeedAuthType::Optional,
        description: "ReliefWeb humanitarian reports, disasters, and crisis updates",
        base_url: "https://api.reliefweb.int/v1",
    },
    FeedMetadata {
        id: FeedId::Unhcr,
        name: "UNHCR",
        category: FeedCategory::Conflict,
        auth_type: FeedAuthType::None,
        description: "UNHCR refugee and displaced population statistics",
        base_url: "https://api.unhcr.org/v1",
    },
    // ── corporate ─────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Gleif,
        name: "GLEIF",
        category: FeedCategory::Corporate,
        auth_type: FeedAuthType::None,
        description: "GLEIF LEI registry — legal entity identifiers for 2M+ organizations",
        base_url: "https://api.gleif.org/api/v1",
    },
    FeedMetadata {
        id: FeedId::OpenCorporates,
        name: "OpenCorporates",
        category: FeedCategory::Corporate,
        auth_type: FeedAuthType::Optional,
        description: "OpenCorporates — 200M+ company records from 140+ jurisdictions",
        base_url: "https://api.opencorporates.com/v0.4",
    },
    FeedMetadata {
        id: FeedId::UkCompaniesHouse,
        name: "UK Companies House",
        category: FeedCategory::Corporate,
        auth_type: FeedAuthType::ApiKey,
        description: "UK Companies House — UK company registrations, filings, and officers",
        base_url: "https://api.company-information.service.gov.uk",
    },
    // ── crypto ────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Bitquery,
        name: "Bitquery",
        category: FeedCategory::Crypto,
        auth_type: FeedAuthType::Credentials,
        description: "Bitquery GraphQL blockchain data — DEX trades, token transfers, on-chain analytics",
        base_url: "https://streaming.bitquery.io/eap",
    },
    FeedMetadata {
        id: FeedId::CoinGecko,
        name: "CoinGecko",
        category: FeedCategory::Crypto,
        auth_type: FeedAuthType::Optional,
        description: "CoinGecko crypto market data — prices, market caps, volumes for 10K+ coins",
        base_url: "https://api.coingecko.com/api/v3",
    },
    FeedMetadata {
        id: FeedId::Coinglass,
        name: "Coinglass",
        category: FeedCategory::Crypto,
        auth_type: FeedAuthType::Credentials,
        description: "Coinglass derivatives analytics — liquidations, open interest, funding rates",
        base_url: "https://open-api-v4.coinglass.com/api",
    },
    FeedMetadata {
        id: FeedId::Etherscan,
        name: "Etherscan",
        category: FeedCategory::Crypto,
        auth_type: FeedAuthType::Optional,
        description: "Etherscan Ethereum block explorer API — transactions, tokens, contracts",
        base_url: "https://api.etherscan.io/api",
    },
    FeedMetadata {
        id: FeedId::WhaleAlert,
        name: "Whale Alert",
        category: FeedCategory::Crypto,
        auth_type: FeedAuthType::ApiKey,
        description: "Whale Alert — large crypto transaction monitoring across blockchains",
        base_url: "https://api.whale-alert.io/v1",
    },
    // ── cyber ─────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::AbuseIpdb,
        name: "AbuseIPDB",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::ApiKey,
        description: "AbuseIPDB — IP address abuse reports and reputation database",
        base_url: "https://api.abuseipdb.com/api/v2",
    },
    FeedMetadata {
        id: FeedId::OtxAlienVault,
        name: "AlienVault OTX",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::ApiKey,
        description: "AlienVault OTX threat intelligence — pulses, indicators, malware hashes",
        base_url: "https://otx.alienvault.com/api/v1",
    },
    FeedMetadata {
        id: FeedId::Censys,
        name: "Censys",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::ApiKeyPair,
        description: "Censys internet-wide scan data — hosts, certificates, open ports",
        base_url: "https://search.censys.io/api/v2",
    },
    FeedMetadata {
        id: FeedId::CloudflareRadar,
        name: "Cloudflare Radar",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::ApiKey,
        description: "Cloudflare Radar — internet traffic trends, BGP, DNS, attack maps",
        base_url: "https://api.cloudflare.com/client/v4/radar",
    },
    FeedMetadata {
        id: FeedId::Nvd,
        name: "NVD",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::Optional,
        description: "NIST NVD — CVE vulnerability database with CVSS scores",
        base_url: "https://services.nvd.nist.gov/rest/json/cves/2.0",
    },
    FeedMetadata {
        id: FeedId::RipeNcc,
        name: "RIPE NCC",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::None,
        description: "RIPE NCC STAT — IP routing, BGP, geolocation, ASN data",
        base_url: "https://stat.ripe.net/data",
    },
    FeedMetadata {
        id: FeedId::Shodan,
        name: "Shodan",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::ApiKey,
        description: "Shodan internet-connected device search engine",
        base_url: "https://api.shodan.io",
    },
    FeedMetadata {
        id: FeedId::Urlhaus,
        name: "URLhaus",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::ApiKey,
        description: "URLhaus (abuse.ch) — malicious URL database and malware distribution sites",
        base_url: "https://urlhaus-api.abuse.ch/v1",
    },
    FeedMetadata {
        id: FeedId::VirusTotal,
        name: "VirusTotal",
        category: FeedCategory::Cyber,
        auth_type: FeedAuthType::ApiKey,
        description: "VirusTotal file, URL, and IP reputation scanning via 70+ AV engines",
        base_url: "https://www.virustotal.com/api/v3",
    },
    // ── demographics ──────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::UnOcha,
        name: "UN OCHA",
        category: FeedCategory::Demographics,
        auth_type: FeedAuthType::None,
        description: "UN OCHA humanitarian data — population, food security, displacement",
        base_url: "https://data.humdata.org/api/3",
    },
    FeedMetadata {
        id: FeedId::UnPopulation,
        name: "UN Population",
        category: FeedCategory::Demographics,
        auth_type: FeedAuthType::None,
        description: "UN Population Division — demographic projections and estimates",
        base_url: "https://population.un.org/dataportalapi/api/v1",
    },
    FeedMetadata {
        id: FeedId::Who,
        name: "WHO",
        category: FeedCategory::Demographics,
        auth_type: FeedAuthType::None,
        description: "WHO Global Health Observatory — global health statistics and indicators",
        base_url: "https://ghoapi.azureedge.net/api",
    },
    FeedMetadata {
        id: FeedId::Wikipedia,
        name: "Wikipedia",
        category: FeedCategory::Demographics,
        auth_type: FeedAuthType::Optional,
        description: "Wikipedia Pageviews API — article traffic and attention signals",
        base_url: "https://wikimedia.org/api/rest_v1",
    },
    // ── economic ──────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Bis,
        name: "BIS",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "Bank for International Settlements — financial stability and monetary data",
        base_url: "https://stats.bis.org/api/v2",
    },
    FeedMetadata {
        id: FeedId::Boe,
        name: "Bank of England",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "Bank of England statistics — UK monetary, financial, and banking data",
        base_url: "https://www.bankofengland.co.uk/boeapps/database",
    },
    FeedMetadata {
        id: FeedId::Bundesbank,
        name: "Deutsche Bundesbank",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "Deutsche Bundesbank time series — German and European financial statistics",
        base_url: "https://api.bundesbank.de/service/v1",
    },
    FeedMetadata {
        id: FeedId::Cbr,
        name: "Bank of Russia",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "Bank of Russia — key rate, FX rates, and Russian financial statistics",
        base_url: "https://www.cbr.ru/scripts",
    },
    FeedMetadata {
        id: FeedId::DBnomics,
        name: "DBnomics",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "DBnomics — aggregator for 70+ statistical providers in one API",
        base_url: "https://api.db.nomics.world/v22",
    },
    FeedMetadata {
        id: FeedId::Ecb,
        name: "ECB",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "European Central Bank — euro area monetary and financial statistics",
        base_url: "https://data-api.ecb.europa.eu/service/data",
    },
    FeedMetadata {
        id: FeedId::Ecos,
        name: "ECOS (Bank of Korea)",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::ApiKey,
        description: "Bank of Korea ECOS — Korean economic and monetary statistics",
        base_url: "https://ecos.bok.or.kr/api",
    },
    FeedMetadata {
        id: FeedId::Eurostat,
        name: "Eurostat",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "Eurostat — EU statistical data on economy, trade, environment, society",
        base_url: "https://ec.europa.eu/eurostat/api/dissemination/statistics/1.0/data",
    },
    FeedMetadata {
        id: FeedId::Fred,
        name: "FRED",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::ApiKey,
        description: "Federal Reserve Economic Data — 800K+ US and international time series",
        base_url: "https://api.stlouisfed.org/fred",
    },
    FeedMetadata {
        id: FeedId::Imf,
        name: "IMF",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "International Monetary Fund data — global macro, balance of payments",
        base_url: "https://www.imf.org/external/datamapper/api/v1",
    },
    FeedMetadata {
        id: FeedId::Oecd,
        name: "OECD",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "OECD SDMX — economic and social statistics for 38 member countries",
        base_url: "https://sdmx.oecd.org/public/rest/data",
    },
    FeedMetadata {
        id: FeedId::WorldBank,
        name: "World Bank",
        category: FeedCategory::Economic,
        auth_type: FeedAuthType::None,
        description: "World Bank Open Data — development indicators for 200+ countries",
        base_url: "https://api.worldbank.org/v2",
    },
    // ── environment ───────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Gdacs,
        name: "GDACS",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::None,
        description: "GDACS global disaster alert and coordination system",
        base_url: "https://www.gdacs.org/gdacsapi/api/events",
    },
    FeedMetadata {
        id: FeedId::GlobalForestWatch,
        name: "Global Forest Watch",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::ApiKey,
        description: "Global Forest Watch — forest loss alerts, fire, deforestation data",
        base_url: "https://data-api.globalforestwatch.org",
    },
    FeedMetadata {
        id: FeedId::NasaEonet,
        name: "NASA EONET",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::None,
        description: "NASA EONET — Earth natural event tracker (wildfires, storms, floods)",
        base_url: "https://eonet.gsfc.nasa.gov/api/v3",
    },
    FeedMetadata {
        id: FeedId::NasaFirms,
        name: "NASA FIRMS",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::ApiKey,
        description: "NASA FIRMS — active fire/hotspot data from MODIS and VIIRS satellites",
        base_url: "https://firms.modaps.eosdis.nasa.gov/api",
    },
    FeedMetadata {
        id: FeedId::Noaa,
        name: "NOAA",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::ApiKey,
        description: "NOAA Climate Data Online — weather observations and climate records",
        base_url: "https://www.ncdc.noaa.gov/cdo-web/api/v2",
    },
    FeedMetadata {
        id: FeedId::NwsAlerts,
        name: "NWS Alerts",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::None,
        description: "National Weather Service alerts — US weather warnings and watches",
        base_url: "https://api.weather.gov",
    },
    FeedMetadata {
        id: FeedId::OpenWeatherMap,
        name: "OpenWeatherMap",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::ApiKey,
        description: "OpenWeatherMap — global weather data, forecasts, and historical records",
        base_url: "https://api.openweathermap.org/data/2.5",
    },
    FeedMetadata {
        id: FeedId::OpenAq,
        name: "OpenAQ",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::Optional,
        description: "OpenAQ — open air quality data from 10K+ monitoring stations worldwide",
        base_url: "https://api.openaq.org/v3",
    },
    FeedMetadata {
        id: FeedId::UsgsEarthquake,
        name: "USGS Earthquake",
        category: FeedCategory::Environment,
        auth_type: FeedAuthType::None,
        description: "USGS Earthquake Hazards — real-time seismic event data",
        base_url: "https://earthquake.usgs.gov/fdsnws/event/1",
    },
    // ── faa ───────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::FaaStatus,
        name: "FAA Status",
        category: FeedCategory::Faa,
        auth_type: FeedAuthType::None,
        description: "FAA NASSTATUS — US airport delay and ground stop information",
        base_url: "https://nasstatus.faa.gov/api",
    },
    // ── feodo ─────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::FeodoTracker,
        name: "Feodo Tracker",
        category: FeedCategory::Feodo,
        auth_type: FeedAuthType::None,
        description: "Feodo Tracker (abuse.ch) — botnet C2 server blocklists",
        base_url: "https://feodotracker.abuse.ch/downloads",
    },
    // ── financial ─────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::AlphaVantage,
        name: "Alpha Vantage",
        category: FeedCategory::Financial,
        auth_type: FeedAuthType::ApiKey,
        description: "Alpha Vantage — stock prices, forex, crypto, and technical indicators",
        base_url: "https://www.alphavantage.co/query",
    },
    FeedMetadata {
        id: FeedId::Finnhub,
        name: "Finnhub",
        category: FeedCategory::Financial,
        auth_type: FeedAuthType::ApiKey,
        description: "Finnhub — real-time stock quotes, fundamentals, earnings, and news",
        base_url: "https://finnhub.io/api/v1",
    },
    FeedMetadata {
        id: FeedId::NewsApi,
        name: "NewsAPI",
        category: FeedCategory::Financial,
        auth_type: FeedAuthType::ApiKey,
        description: "NewsAPI — news headlines and articles from 80K+ sources worldwide",
        base_url: "https://newsapi.org/v2",
    },
    FeedMetadata {
        id: FeedId::OpenFigi,
        name: "OpenFIGI",
        category: FeedCategory::Financial,
        auth_type: FeedAuthType::Optional,
        description: "OpenFIGI — financial instrument identifier mapping across exchanges",
        base_url: "https://api.openfigi.com/v3",
    },
    // ── governance ────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::EuParliament,
        name: "EU Parliament",
        category: FeedCategory::Governance,
        auth_type: FeedAuthType::None,
        description: "EU Parliament open data — MEPs, votes, plenary sessions, legislation",
        base_url: "https://data.europarl.europa.eu/api/v2",
    },
    FeedMetadata {
        id: FeedId::UkParliament,
        name: "UK Parliament",
        category: FeedCategory::Governance,
        auth_type: FeedAuthType::None,
        description: "UK Parliament — MPs, Lords, bills, votes, and Hansard data",
        base_url: "https://members-api.parliament.uk/api",
    },
    // ── hacker_news ───────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::HackerNews,
        name: "Hacker News",
        category: FeedCategory::HackerNews,
        auth_type: FeedAuthType::None,
        description: "Hacker News Firebase API — top stories, jobs, ask, show, and comments",
        base_url: "https://hacker-news.firebaseio.com/v0",
    },
    // ── maritime ──────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Ais,
        name: "AIS (Datalastic)",
        category: FeedCategory::Maritime,
        auth_type: FeedAuthType::ApiKey,
        description: "AIS Datalastic — vessel positions, routes, and port calls",
        base_url: "https://api.datalastic.com/api/v0",
    },
    FeedMetadata {
        id: FeedId::AisStream,
        name: "AISStream",
        category: FeedCategory::Maritime,
        auth_type: FeedAuthType::ApiKey,
        description: "AISStream WebSocket — real-time AIS vessel tracking",
        base_url: "wss://stream.aisstream.io/v0/stream",
    },
    FeedMetadata {
        id: FeedId::ImfPortWatch,
        name: "IMF PortWatch",
        category: FeedCategory::Maritime,
        auth_type: FeedAuthType::None,
        description: "IMF PortWatch — global port activity and trade flow indicators",
        base_url: "https://portwatch.imf.org/datasets/portwatch",
    },
    FeedMetadata {
        id: FeedId::NgaWarnings,
        name: "NGA Maritime Warnings",
        category: FeedCategory::Maritime,
        auth_type: FeedAuthType::None,
        description: "NGA MSI — maritime safety information, NAVAREA warnings",
        base_url: "https://msi.nga.mil/api/publications",
    },
    // ── prediction ────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::PredictIt,
        name: "PredictIt",
        category: FeedCategory::Prediction,
        auth_type: FeedAuthType::None,
        description: "PredictIt prediction markets — political and economic event contracts",
        base_url: "https://www.predictit.org/api/marketdata",
    },
    // ── rss ───────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::RssProxy,
        name: "RSS Proxy",
        category: FeedCategory::Rss,
        auth_type: FeedAuthType::None,
        description: "RSS feed proxy — fetch and parse any RSS/Atom feed",
        base_url: "https://rss2json.com/api.json",
    },
    // ── sanctions ─────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Interpol,
        name: "INTERPOL",
        category: FeedCategory::Sanctions,
        auth_type: FeedAuthType::None,
        description: "INTERPOL public API — red notices, yellow notices, wanted persons",
        base_url: "https://ws-public.interpol.int",
    },
    FeedMetadata {
        id: FeedId::Ofac,
        name: "OFAC",
        category: FeedCategory::Sanctions,
        auth_type: FeedAuthType::ApiKey,
        description: "US Treasury OFAC — sanctions list screening and entity search",
        base_url: "https://api.ofac-api.com/v4",
    },
    FeedMetadata {
        id: FeedId::OpenSanctions,
        name: "OpenSanctions",
        category: FeedCategory::Sanctions,
        auth_type: FeedAuthType::Optional,
        description: "OpenSanctions — unified sanctions and watchlist data from 100+ sources",
        base_url: "https://api.opensanctions.org",
    },
    // ── space ─────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::LaunchLibrary,
        name: "Launch Library 2",
        category: FeedCategory::Space,
        auth_type: FeedAuthType::None,
        description: "Launch Library 2 — space launch schedule and mission database",
        base_url: "https://ll.thespacedevs.com/2.2.0",
    },
    FeedMetadata {
        id: FeedId::Nasa,
        name: "NASA",
        category: FeedCategory::Space,
        auth_type: FeedAuthType::Optional,
        description: "NASA APIs — APOD, NEO asteroids, Mars Rover, GIBS imagery",
        base_url: "https://api.nasa.gov",
    },
    FeedMetadata {
        id: FeedId::SentinelHub,
        name: "Sentinel Hub",
        category: FeedCategory::Space,
        auth_type: FeedAuthType::ApiKey,
        description: "Copernicus Sentinel Hub — satellite imagery and geospatial processing",
        base_url: "https://services.sentinel-hub.com",
    },
    FeedMetadata {
        id: FeedId::SpaceTrack,
        name: "Space-Track",
        category: FeedCategory::Space,
        auth_type: FeedAuthType::UsernamePassword,
        description: "Space-Track.org — USSPACECOM TLE orbital data for 25K+ objects",
        base_url: "https://www.space-track.org",
    },
    FeedMetadata {
        id: FeedId::SpaceX,
        name: "SpaceX",
        category: FeedCategory::Space,
        auth_type: FeedAuthType::None,
        description: "SpaceX REST API — launches, rockets, crew, capsules, Starlink data",
        base_url: "https://api.spacexdata.com/v5",
    },
    // ── trade ─────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Comtrade,
        name: "UN COMTRADE",
        category: FeedCategory::Trade,
        auth_type: FeedAuthType::ApiKey,
        description: "UN COMTRADE — international merchandise and services trade statistics",
        base_url: "https://comtradeapi.un.org/data/v1",
    },
    FeedMetadata {
        id: FeedId::EuTed,
        name: "EU TED",
        category: FeedCategory::Trade,
        auth_type: FeedAuthType::Optional,
        description: "EU TED — Tenders Electronic Daily, EU public procurement notices",
        base_url: "https://api.ted.europa.eu/v3",
    },
    // ── us_gov ────────────────────────────────────────────────────────────────
    FeedMetadata {
        id: FeedId::Bea,
        name: "BEA",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::ApiKey,
        description: "Bureau of Economic Analysis — US GDP, national income, trade in services",
        base_url: "https://apps.bea.gov/api/data",
    },
    FeedMetadata {
        id: FeedId::Bls,
        name: "BLS",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::ApiKey,
        description: "Bureau of Labor Statistics — US CPI, unemployment, wages, PPI",
        base_url: "https://api.bls.gov/publicAPI/v2",
    },
    FeedMetadata {
        id: FeedId::Census,
        name: "US Census",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::ApiKey,
        description: "US Census Bureau — demographic, economic, and geographic data",
        base_url: "https://api.census.gov/data",
    },
    FeedMetadata {
        id: FeedId::Congress,
        name: "Congress.gov",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::ApiKey,
        description: "Congress.gov API — US legislation, members, committees, votes",
        base_url: "https://api.congress.gov/v3",
    },
    FeedMetadata {
        id: FeedId::Eia,
        name: "EIA",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::ApiKey,
        description: "US Energy Information Administration — energy production, consumption, prices",
        base_url: "https://api.eia.gov/v2",
    },
    FeedMetadata {
        id: FeedId::FbiCrime,
        name: "FBI Crime Data",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::ApiKey,
        description: "FBI Crime Data Explorer — US crime statistics by agency and geography",
        base_url: "https://api.usa.gov/crime/fbi/cde",
    },
    FeedMetadata {
        id: FeedId::SamGov,
        name: "SAM.gov",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::ApiKey,
        description: "SAM.gov — US federal contracts, grants, entity registrations",
        base_url: "https://api.sam.gov/entity-information/v3/entities",
    },
    FeedMetadata {
        id: FeedId::SecEdgar,
        name: "SEC EDGAR",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::None,
        description: "SEC EDGAR — company filings, financial statements, 8-K, 10-K, 10-Q",
        base_url: "https://data.sec.gov",
    },
    FeedMetadata {
        id: FeedId::UsaSpending,
        name: "USASpending.gov",
        category: FeedCategory::UsGov,
        auth_type: FeedAuthType::None,
        description: "USASpending.gov — US federal spending, contracts, grants, agencies",
        base_url: "https://api.usaspending.gov/api/v2",
    },
];

// ═══════════════════════════════════════════════════════════════════════════════
// FEED REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Static registry providing lookup and filtering over all feed metadata.
///
/// All methods are zero-cost — they operate on the compile-time `FEEDS` slice.
pub struct FeedRegistry;

impl FeedRegistry {
    /// Return the complete list of all 88 feeds.
    pub fn all() -> &'static [FeedMetadata] {
        FEEDS
    }

    /// Look up metadata for a specific feed by ID.
    ///
    /// Returns `None` only if the registry has a bug (all variants must be
    /// present in `FEEDS`).
    pub fn get(id: FeedId) -> Option<&'static FeedMetadata> {
        FEEDS.iter().find(|m| m.id == id)
    }

    /// Return all feeds belonging to a given category.
    pub fn by_category(cat: FeedCategory) -> Vec<&'static FeedMetadata> {
        FEEDS.iter().filter(|m| m.category == cat).collect()
    }

    /// Return feeds that do not require authentication
    /// (i.e. [`FeedAuthType::None`] or [`FeedAuthType::Optional`]).
    pub fn public_feeds() -> Vec<&'static FeedMetadata> {
        FEEDS
            .iter()
            .filter(|m| {
                matches!(m.auth_type, FeedAuthType::None | FeedAuthType::Optional)
            })
            .collect()
    }

    /// Total number of registered feeds.
    pub fn count() -> usize {
        FEEDS.len()
    }
}
