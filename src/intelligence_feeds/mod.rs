//! # Intelligence Feeds Module
//!
//! Data providers, analytics platforms, and economic data sources.
//! Organized into thematic groups covering crypto, economics, government,
//! finance, trade, conflict, maritime, aviation, space, environment, cyber,
//! governance, sanctions, corporate, demographics, academic, and prediction data.

// === Thematic Groups ===

/// Crypto/Market data providers (Coinglass, Bitquery, WhaleAlert, CoinGecko, Etherscan)
pub mod crypto;

/// International economic data (FRED, World Bank, DBnomics, OECD, Eurostat, ECB, IMF, BIS, Bundesbank, ECOS, CBR, BoE)
pub mod economic;

/// U.S. Government data (EIA, BLS, BEA, Census, SEC EDGAR, Congress, FBI Crime, USASpending, SAM.gov)
pub mod us_gov;

/// Financial data providers (Alpha Vantage, Finnhub, NewsAPI, OpenFIGI)
pub mod financial;

/// International trade and procurement (UN COMTRADE, EU TED)
pub mod trade;

/// Conflict, humanitarian, and crisis data (ACLED, GDELT, UCDP, ReliefWeb, UNHCR)
pub mod conflict;

/// Maritime and vessel tracking (AISStream, IMF PortWatch, AIS/Datalastic)
pub mod maritime;

/// Aviation and flight tracking (ADS-B Exchange, OpenSky, AviationStack)
pub mod aviation;

/// FAA Airport Status (NASSTATUS)
pub mod faa_status;

/// Space, satellite, and Earth observation (Launch Library, SpaceX, Space-Track, NASA, Sentinel Hub)
pub mod space;

/// Environment, weather, and natural hazards (NOAA, OpenAQ, OpenWeatherMap, NASA FIRMS, Global Forest Watch, USGS Earthquake, GDACS)
pub mod environment;

/// Cybersecurity, threat intelligence, and internet infrastructure (Shodan, Censys, VirusTotal, NVD, AlienVault OTX, Cloudflare Radar, RIPE NCC)
pub mod cyber;

/// C2 (Command & Control) threat intelligence feeds
pub mod c2intel_feeds;

/// Feodo Tracker botnet C2 feeds
pub mod feodo_tracker;

// National Weather Service alerts
// pub mod nws_alerts; // TODO: Implement

/// Parliamentary and legislative data (UK Parliament, EU Parliament)
pub mod governance;

/// Sanctions, watchlists, and law enforcement (OpenSanctions, OFAC, INTERPOL)
pub mod sanctions;

/// Corporate ownership, registries, and entity data (GLEIF, OpenCorporates, UK Companies House)
pub mod corporate;

/// Demographics, health, and population data (UN Population, WHO, Wikipedia)
pub mod demographics;

/// Academic research and papers (arXiv, Semantic Scholar)
pub mod academic;

/// News and social content feeds (Hacker News)
pub mod hacker_news;

/// RSS Feed Proxy (news aggregator)
pub mod rss_proxy;

/// Prediction markets (PredictIt, Polymarket)
pub mod prediction;

/// Feed registry, metadata, and factory for all 88 intelligence feeds
pub mod feed_manager;

// === Re-exports ===
// All types are re-exported from their thematic group modules so consumers
// can use either `data_feeds::crypto::CoinglassConnector` or `data_feeds::CoinglassConnector`.

// Crypto/Market
pub use crypto::{CoinglassConnector, CoinglassAuth, CoinglassParser};
pub use crypto::{CoinGeckoConnector, CoinGeckoAuth};

// Economic Data
pub use economic::{FredConnector, FredAuth, FredParser};
pub use economic::{WorldBankConnector, WorldBankAuth};
pub use economic::{DBnomicsConnector, DBnomicsAuth, DBnomicsParser};
pub use economic::{OecdConnector, OecdAuth, OecdParser};
pub use economic::{EurostatConnector, EurostatAuth, EurostatParser};
pub use economic::{EcbConnector, EcbAuth};
pub use economic::{ImfConnector, ImfAuth};
pub use economic::{BisConnector, BisAuth};
pub use economic::{BundesbankConnector, BundesbankAuth};
pub use economic::{EcosConnector, EcosAuth};
pub use economic::{CbrConnector, CbrAuth};
pub use economic::{BoeConnector, BoeAuth};

// US Government Data
pub use us_gov::{EiaConnector, EiaAuth};
pub use us_gov::{BlsConnector, BlsAuth};
pub use us_gov::{BeaConnector, BeaAuth};
pub use us_gov::{CensusConnector, CensusAuth};
pub use us_gov::{SecEdgarConnector, SecEdgarAuth};
pub use us_gov::{CongressConnector, CongressAuth};
pub use us_gov::{FbiCrimeConnector, FbiCrimeAuth, FbiCrimeParser, CrimeEstimate, CrimeAgency, NibrsData};
pub use us_gov::{UsaSpendingConnector, UsaSpendingAuth, UsaSpendingParser, UsaSpendingAward, UsaSpendingAgency, UsaSpendingState};
pub use us_gov::{SamGovConnector, SamGovAuth, SamGovParser, SamEntity, SamOpportunity, SamAddress};

// Financial Data Providers
pub use financial::{AlphaVantageConnector, AlphaVantageAuth};
pub use financial::{FinnhubConnector, FinnhubAuth};
pub use financial::{NewsApiConnector, NewsApiAuth};
pub use financial::{OpenFigiConnector, OpenFigiAuth};

// International Trade
pub use trade::{ComtradeConnector, ComtradeAuth};
pub use trade::{EuTedConnector, EuTedAuth, EuTedParser, TedNotice, TedEntity, TedSearchResult};

// Conflict/Humanitarian
pub use conflict::{AcledConnector, AcledAuth, AcledParser, AcledEvent, AcledResponse};
pub use conflict::{GdeltConnector, GdeltAuth};
pub use conflict::{UcdpConnector, UcdpAuth};
pub use conflict::{ReliefWebConnector, ReliefWebAuth, ReliefWebParser, ReliefWebReport, ReliefWebDisaster, ReliefWebCountry, ReliefWebSearchResult};
pub use conflict::{UnhcrConnector, UnhcrAuth};

// Maritime
pub use maritime::{AisStreamConnector, AisStreamAuth};
pub use maritime::{ImfPortWatchConnector, ImfPortWatchAuth};
pub use maritime::{AisConnector, AisAuth};

// Aviation
pub use aviation::{AdsbExchangeConnector, AdsbExchangeAuth};
pub use aviation::{OpenskyConnector, OpenskyAuth};
pub use aviation::{AviationStackConnector, AviationStackAuth, AviationStackParser, AvFlight, AvAirport, AvAirline, AvFlightInfo, AvRoute};
pub use aviation::{WingbitsConnector, WingbitsAuth, WingbitsParser, AircraftDetails, AircraftCategory};
pub use faa_status::{FaaStatusConnector, FaaStatusAuth, FaaStatusParser, AirportDelay, AirportStatus, DelayType, DelaySeverity};

// Space
pub use space::{LaunchLibraryConnector, LaunchLibraryAuth};
pub use space::{SpaceXConnector, SpaceXAuth, SpaceXParser, SpaceXLaunch, SpaceXRocket, SpaceXCrew, SpaceXStarlink};
pub use space::{SpaceTrackConnector, SpaceTrackAuth, SpaceTrackParser, Satellite, DecayPrediction, TleData};
pub use space::{NasaConnector, NasaAuth, NasaParser};
pub use space::{SentinelHubConnector, SentinelHubAuth, SentinelHubParser, SentinelCatalogResult, SentinelFeature, SentinelStatistical};

// Environment
pub use environment::{NoaaConnector, NoaaAuth};
pub use environment::{OpenAqConnector, OpenAqAuth, OpenAqParser};
pub use environment::{OpenWeatherMapConnector, OpenWeatherMapAuth};
pub use environment::{NasaFirmsConnector, NasaFirmsAuth, NasaFirmsParser, FireHotspot, FireSummary, CountryFireCount};
pub use environment::{NasaEonetConnector, NasaEonetAuth, NasaEonetParser, NaturalEvent, EventCategory, EventSource, EventGeometry};
pub use environment::{GfwConnector, GfwAuth};
pub use environment::{UsgsEarthquakeConnector, UsgsEarthquakeAuth};
pub use environment::{GdacsConnector, GdacsAuth, GdacsParser, DisasterEvent, DisasterType, AlertLevel};

// Cyber
pub use cyber::{ShodanConnector, ShodanAuth, ShodanParser, ShodanHost, ShodanSearchResult, ShodanService, ShodanApiInfo, ShodanDnsResult};
pub use cyber::{CensysConnector, CensysAuth, CensysParser, CensysHost, CensysSearchResult, CensysService, CensysLocation};
pub use cyber::{VirusTotalConnector, VirusTotalAuth, VirusTotalParser, VtFileReport, VtAnalysisStats, VtDomainReport, VtIpReport};
pub use cyber::{NvdConnector, NvdAuth, NvdParser, NvdCve, NvdSearchResult};
pub use cyber::{OtxConnector, OtxAuth, OtxParser, OtxPulse, OtxIndicator, OtxIpReputation};
pub use cyber::{CloudflareRadarConnector, CloudflareRadarAuth};
pub use cyber::{RipeNccConnector, RipeNccAuth};

// C2 / Threat Intelligence Feeds
pub use c2intel_feeds::{C2IntelFeedsConnector, C2IntelFeedsAuth, C2IntelFeedsParser, C2Indicator, IndicatorType};

// Feodo Tracker
pub use feodo_tracker::{FeodoTrackerConnector, FeodoTrackerAuth, FeodoTrackerParser, C2Server, C2Status, BlocklistStats};

// NWS Alerts
// pub use nws_alerts::{NwsAlertsConnector, NwsAlertsAuth, NwsAlertsParser}; // TODO: Implement

// Governance
pub use governance::{UkParliamentConnector, UkParliamentAuth};
pub use governance::{EuParliamentConnector, EuParliamentAuth, EuParliamentParser};

// Sanctions
pub use sanctions::{OpenSanctionsConnector, OpenSanctionsAuth};
pub use sanctions::{OfacConnector, OfacAuth, OfacParser, OfacEntity, OfacSearchResult, OfacScreenResult, OfacSource};
pub use sanctions::{InterpolConnector, InterpolAuth, InterpolParser, InterpolNotice, InterpolSearchResult, ArrestWarrant, InterpolImage};

// Corporate
pub use corporate::{GleifConnector, GleifAuth, GleifParser, GleifEntity, GleifRelationship, GleifOwnershipChain};
pub use corporate::{OpenCorporatesConnector, OpenCorporatesAuth, OcCompany, OcOfficer, OcCompanyRef, OcFiling, OcSearchResult};
pub use corporate::{UkCompaniesHouseConnector, UkCompaniesHouseAuth, UkCompaniesHouseParser, ChCompany, ChOfficer, ChPsc, ChFiling, ChSearchResult};

// Demographics
pub use demographics::{UnPopConnector, UnPopAuth};
pub use demographics::{WhoConnector, WhoAuth};
pub use demographics::{WikipediaConnector, WikipediaAuth, WikipediaParser};
pub use demographics::{UnOchaConnector, UnOchaAuth, UnOchaParser, PopulationData, FoodSecurityData, HumanitarianNeeds, OperationalPresence, FundingData, DisplacementData};

// Academic
pub use academic::{ArxivConnector, ArxivAuth};
pub use academic::{SemanticScholarConnector, SemanticScholarAuth};

// News & Social
pub use hacker_news::{HackerNewsConnector, HackerNewsAuth, HackerNewsParser, HnStory, HnUser, HnItemType, HnUpdates};

// RSS Feed Proxy
pub use rss_proxy::{RssProxyConnector, RssProxyAuth, RssProxyParser, RssFeed, RssFeedItem};

// Prediction
pub use prediction::{PredictItConnector, PredictItAuth};
