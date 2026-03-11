//! # Feed Factory
//!
//! Factory for instantiating intelligence feed connectors by [`FeedId`].
//!
//! ## Overview
//!
//! Because intelligence feeds have no shared trait, the factory returns
//! `Box<dyn Any>` which the caller downcasts to the concrete connector type.
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::intelligence_feeds::feed_manager::{FeedFactory, FeedId, FeedCredentials};
//!
//! // Create a public (no-auth) feed
//! let any = FeedFactory::create_public(FeedId::Arxiv)?;
//! let arxiv = any.downcast::<ArxivConnector>().unwrap();
//!
//! // Create an authenticated feed
//! let any = FeedFactory::create_authenticated(
//!     FeedId::Fred,
//!     FeedCredentials::ApiKey("my_fred_key".into()),
//! ).await?;
//! let fred = any.downcast::<FredConnector>().unwrap();
//! ```
//!
//! ## Credential Variants
//!
//! | Variant | Used by |
//! |---------|---------|
//! | `None` | Feeds with `FeedAuthType::None` |
//! | `ApiKey` | Most single-key feeds |
//! | `ApiKeyPair` | Censys (API ID + API Secret) |
//! | `Credentials` | Bitquery, Coinglass |
//! | `CredentialsWithRateLimit` | Coinglass (explicit rate limit) |
//! | `UsernamePassword` | SpaceTrack, OpenSky |

use std::any::Any;

use super::feed_id::FeedId;
use crate::core::{Credentials, ExchangeError, ExchangeResult};

// ── Credential type ──────────────────────────────────────────────────────────

/// Credentials accepted by [`FeedFactory`].
pub enum FeedCredentials {
    /// No authentication required.
    None,
    /// Single API key.
    ApiKey(String),
    /// Two-key pair (Censys: API ID + API Secret).
    ApiKeyPair {
        api_id: String,
        api_secret: String,
    },
    /// Core [`Credentials`] struct (Bitquery, Coinglass).
    Credentials(Credentials),
    /// [`Credentials`] with explicit rate-limit (Coinglass preferred variant).
    CredentialsWithRateLimit {
        creds: Credentials,
        rate_limit_per_min: u32,
    },
    /// Username + password (SpaceTrack, OpenSky).
    UsernamePassword {
        username: String,
        password: String,
    },
}

// ── Factory ───────────────────────────────────────────────────────────────────

/// Factory for creating intelligence feed connectors.
///
/// Returns `Box<dyn Any>` — the caller must downcast to the concrete type.
pub struct FeedFactory;

impl FeedFactory {
    /// Create a feed connector that does **not** require authentication.
    ///
    /// Works for feeds with `FeedAuthType::None` (33 feeds) and
    /// `FeedAuthType::Optional` (10 feeds) where the connector provides a
    /// no-arg or `anonymous()`/`public()` constructor.
    ///
    /// Returns `Err(Auth)` for feeds that require credentials.
    pub fn create_public(id: FeedId) -> ExchangeResult<Box<dyn Any>> {
        match id {
            // ── academic ──────────────────────────────────────────────────
            FeedId::Arxiv => {
                Ok(Box::new(crate::intelligence_feeds::ArxivConnector::new()))
            }
            FeedId::SemanticScholar => {
                use crate::intelligence_feeds::SemanticScholarAuth;
                Ok(Box::new(
                    crate::intelligence_feeds::SemanticScholarConnector::new(
                        SemanticScholarAuth::unauthenticated(),
                    ),
                ))
            }
            // ── aviation ──────────────────────────────────────────────────
            FeedId::Opensky => {
                use crate::intelligence_feeds::aviation::opensky::{OpenskyConnector, OpenskyAuth};
                Ok(Box::new(OpenskyConnector::new(
                    OpenskyAuth::anonymous(),
                )))
            }
            // ── c2intel ───────────────────────────────────────────────────
            FeedId::C2IntelFeeds => {
                Ok(Box::new(crate::intelligence_feeds::C2IntelFeedsConnector::new()))
            }
            // ── conflict ──────────────────────────────────────────────────
            FeedId::Gdelt => {
                Ok(Box::new(crate::intelligence_feeds::GdeltConnector::new()))
            }
            FeedId::Ucdp => {
                Ok(Box::new(crate::intelligence_feeds::UcdpConnector::new()))
            }
            FeedId::ReliefWeb => {
                use crate::intelligence_feeds::conflict::reliefweb::{ReliefWebConnector, ReliefWebAuth};
                Ok(Box::new(ReliefWebConnector::new(ReliefWebAuth::anonymous())))
            }
            FeedId::Unhcr => {
                Ok(Box::new(crate::intelligence_feeds::UnhcrConnector::new(false)))
            }
            // ── corporate ─────────────────────────────────────────────────
            FeedId::Gleif => {
                Ok(Box::new(crate::intelligence_feeds::GleifConnector::new()))
            }
            FeedId::OpenCorporates => {
                use crate::intelligence_feeds::corporate::opencorporates::{OpenCorporatesConnector, OpenCorporatesAuth};
                Ok(Box::new(OpenCorporatesConnector::new(
                    OpenCorporatesAuth::anonymous(),
                )))
            }
            // ── crypto ────────────────────────────────────────────────────
            FeedId::CoinGecko => {
                use crate::intelligence_feeds::crypto::coingecko::CoinGeckoAuth;
                Ok(Box::new(crate::intelligence_feeds::CoinGeckoConnector::new(
                    CoinGeckoAuth::free(),
                )))
            }
            FeedId::Etherscan => {
                // Etherscan works without a key (lower rate limits — 5 req/sec)
                use crate::onchain::ethereum::etherscan::{EtherscanConnector, EtherscanAuth};
                Ok(Box::new(EtherscanConnector::new(
                    EtherscanAuth { api_key: None },
                )))
            }
            // ── cyber ─────────────────────────────────────────────────────
            FeedId::Nvd => {
                use crate::intelligence_feeds::cyber::nvd::{NvdConnector, NvdAuth};
                Ok(Box::new(NvdConnector::new(NvdAuth::public())))
            }
            FeedId::RipeNcc => {
                Ok(Box::new(crate::intelligence_feeds::RipeNccConnector::new()))
            }
            // ── demographics ──────────────────────────────────────────────
            FeedId::UnOcha => {
                use crate::intelligence_feeds::demographics::un_ocha::UnOchaConnector;
                Ok(Box::new(UnOchaConnector::public()))
            }
            FeedId::UnPopulation => {
                Ok(Box::new(crate::intelligence_feeds::UnPopConnector::new(false)))
            }
            FeedId::Who => {
                use crate::intelligence_feeds::demographics::who::WhoConnector;
                Ok(Box::new(WhoConnector::new()))
            }
            FeedId::Wikipedia => {
                // WikipediaAuth::new() uses default User-Agent (no credentials needed)
                use crate::intelligence_feeds::{WikipediaConnector, WikipediaAuth};
                Ok(Box::new(WikipediaConnector::new(WikipediaAuth::new())))
            }
            // ── economic ──────────────────────────────────────────────────
            FeedId::Bis => {
                Ok(Box::new(crate::intelligence_feeds::BisConnector::new()))
            }
            FeedId::Boe => {
                Ok(Box::new(crate::intelligence_feeds::BoeConnector::new()))
            }
            FeedId::Bundesbank => {
                Ok(Box::new(crate::intelligence_feeds::BundesbankConnector::new()))
            }
            FeedId::Cbr => {
                Ok(Box::new(crate::intelligence_feeds::CbrConnector::new()))
            }
            FeedId::DBnomics => {
                Ok(Box::new(crate::intelligence_feeds::DBnomicsConnector::new()))
            }
            FeedId::Ecb => {
                Ok(Box::new(crate::intelligence_feeds::EcbConnector::new()))
            }
            FeedId::Eurostat => {
                Ok(Box::new(crate::intelligence_feeds::EurostatConnector::new()))
            }
            FeedId::Imf => {
                Ok(Box::new(crate::intelligence_feeds::ImfConnector::new()))
            }
            FeedId::Oecd => {
                Ok(Box::new(crate::intelligence_feeds::OecdConnector::new()))
            }
            FeedId::WorldBank => {
                Ok(Box::new(crate::intelligence_feeds::WorldBankConnector::new()))
            }
            // ── environment ───────────────────────────────────────────────
            FeedId::Gdacs => {
                Ok(Box::new(crate::intelligence_feeds::GdacsConnector::new()))
            }
            FeedId::NasaEonet => {
                Ok(Box::new(crate::intelligence_feeds::NasaEonetConnector::new()))
            }
            FeedId::NwsAlerts => {
                use crate::intelligence_feeds::environment::nws_alerts::NwsAlertsConnector;
                Ok(Box::new(NwsAlertsConnector::new()))
            }
            FeedId::OpenAq => {
                use crate::intelligence_feeds::environment::openaq::{OpenAqConnector, OpenAqAuth};
                Ok(Box::new(OpenAqConnector::new(OpenAqAuth::public())))
            }
            FeedId::UsgsEarthquake => {
                Ok(Box::new(crate::intelligence_feeds::UsgsEarthquakeConnector::new()))
            }
            // ── faa ───────────────────────────────────────────────────────
            FeedId::FaaStatus => {
                Ok(Box::new(crate::intelligence_feeds::FaaStatusConnector::new()))
            }
            // ── feodo ─────────────────────────────────────────────────────
            FeedId::FeodoTracker => {
                Ok(Box::new(crate::intelligence_feeds::FeodoTrackerConnector::new()))
            }
            // ── financial ─────────────────────────────────────────────────
            FeedId::OpenFigi => {
                use crate::intelligence_feeds::financial::openfigi::{OpenFigiConnector, OpenFigiAuth};
                Ok(Box::new(OpenFigiConnector::new(OpenFigiAuth::no_auth())))
            }
            // ── governance ────────────────────────────────────────────────
            FeedId::EuParliament => {
                Ok(Box::new(crate::intelligence_feeds::EuParliamentConnector::new()))
            }
            FeedId::UkParliament => {
                Ok(Box::new(crate::intelligence_feeds::UkParliamentConnector::new()))
            }
            // ── hacker_news ───────────────────────────────────────────────
            FeedId::HackerNews => {
                Ok(Box::new(crate::intelligence_feeds::HackerNewsConnector::new()))
            }
            // ── maritime ──────────────────────────────────────────────────
            FeedId::ImfPortWatch => {
                Ok(Box::new(crate::intelligence_feeds::ImfPortWatchConnector::new()))
            }
            FeedId::NgaWarnings => {
                use crate::intelligence_feeds::maritime::nga_warnings::NgaWarningsConnector;
                Ok(Box::new(NgaWarningsConnector::new()))
            }
            // ── prediction ────────────────────────────────────────────────
            FeedId::PredictIt => {
                Ok(Box::new(crate::intelligence_feeds::PredictItConnector::new()))
            }
            // ── rss ───────────────────────────────────────────────────────
            FeedId::RssProxy => {
                Ok(Box::new(crate::intelligence_feeds::RssProxyConnector::new()))
            }
            // ── sanctions ─────────────────────────────────────────────────
            FeedId::Interpol => {
                use crate::intelligence_feeds::sanctions::interpol::{InterpolConnector, InterpolAuth};
                Ok(Box::new(InterpolConnector::new(InterpolAuth::new())))
            }
            FeedId::OpenSanctions => {
                use crate::intelligence_feeds::sanctions::opensanctions::{OpenSanctionsConnector, OpenSanctionsAuth};
                Ok(Box::new(OpenSanctionsConnector::new(
                    OpenSanctionsAuth::anonymous(),
                )))
            }
            // ── space ─────────────────────────────────────────────────────
            FeedId::LaunchLibrary => {
                Ok(Box::new(crate::intelligence_feeds::LaunchLibraryConnector::new()))
            }
            FeedId::Nasa => {
                use crate::intelligence_feeds::space::nasa::{NasaConnector, NasaAuth};
                // Use DEMO_KEY (30 req/hour) as the public fallback
                Ok(Box::new(NasaConnector::new(NasaAuth::from_env())))
            }
            FeedId::SpaceX => {
                Ok(Box::new(crate::intelligence_feeds::SpaceXConnector::new()))
            }
            // ── trade ─────────────────────────────────────────────────────
            FeedId::EuTed => {
                use crate::intelligence_feeds::trade::eu_ted::{EuTedConnector, EuTedAuth};
                Ok(Box::new(EuTedConnector::new(EuTedAuth::public())))
            }
            // ── us_gov ────────────────────────────────────────────────────
            FeedId::SecEdgar => {
                Ok(Box::new(crate::intelligence_feeds::SecEdgarConnector::new()))
            }
            FeedId::UsaSpending => {
                Ok(Box::new(crate::intelligence_feeds::UsaSpendingConnector::new()))
            }
            // ── auth-required feeds ───────────────────────────────────────
            other => Err(ExchangeError::Auth(format!(
                "{:?} requires authentication — use FeedFactory::create_authenticated()",
                other
            ))),
        }
    }

    /// Create an authenticated feed connector.
    ///
    /// The `creds` variant must match the feed's expected auth type; passing the
    /// wrong variant returns `Err(Auth)`.
    pub async fn create_authenticated(
        id: FeedId,
        creds: FeedCredentials,
    ) -> ExchangeResult<Box<dyn Any>> {
        match (id, creds) {
            // ── academic ──────────────────────────────────────────────────
            (FeedId::SemanticScholar, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::SemanticScholarAuth;
                Ok(Box::new(
                    crate::intelligence_feeds::SemanticScholarConnector::new(
                        SemanticScholarAuth::new(key),
                    ),
                ))
            }
            // ── aviation ──────────────────────────────────────────────────
            (FeedId::AdsbExchange, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::AdsbExchangeAuth;
                Ok(Box::new(
                    crate::intelligence_feeds::AdsbExchangeConnector::new(
                        AdsbExchangeAuth::new(key),
                    ),
                ))
            }
            (FeedId::AviationStack, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::AviationStackAuth;
                Ok(Box::new(
                    crate::intelligence_feeds::AviationStackConnector::new(
                        AviationStackAuth::new(key),
                    ),
                ))
            }
            (FeedId::Opensky, FeedCredentials::UsernamePassword { username, password }) => {
                use crate::intelligence_feeds::aviation::opensky::{OpenskyConnector, OpenskyAuth};
                Ok(Box::new(OpenskyConnector::new(OpenskyAuth::new(
                    username, password,
                ))))
            }
            (FeedId::Wingbits, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::aviation::wingbits::WingbitsConnector;
                Ok(Box::new(WingbitsConnector::new(key)))
            }
            // ── conflict ──────────────────────────────────────────────────
            (FeedId::Acled, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::AcledAuth;
                // ACLED requires key + email; use key for both fields when only key given
                Ok(Box::new(crate::intelligence_feeds::AcledConnector::new(
                    AcledAuth::new(key.clone(), key),
                )))
            }
            (FeedId::ReliefWeb, FeedCredentials::ApiKey(appname)) => {
                use crate::intelligence_feeds::conflict::reliefweb::{ReliefWebConnector, ReliefWebAuth};
                Ok(Box::new(ReliefWebConnector::new(ReliefWebAuth::new(appname))))
            }
            // ── corporate ─────────────────────────────────────────────────
            (FeedId::OpenCorporates, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::corporate::opencorporates::{OpenCorporatesConnector, OpenCorporatesAuth};
                Ok(Box::new(OpenCorporatesConnector::new(
                    OpenCorporatesAuth::new(key),
                )))
            }
            (FeedId::UkCompaniesHouse, FeedCredentials::ApiKey(key)) => {
                Ok(Box::new(
                    crate::intelligence_feeds::UkCompaniesHouseConnector::new(key),
                ))
            }
            // ── crypto ────────────────────────────────────────────────────
            (FeedId::Bitquery, FeedCredentials::Credentials(c)) => {
                use crate::onchain::analytics::bitquery::BitqueryConnector;
                Ok(Box::new(BitqueryConnector::new(c).await?))
            }
            (FeedId::CoinGecko, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::crypto::coingecko::CoinGeckoAuth;
                Ok(Box::new(crate::intelligence_feeds::CoinGeckoConnector::new(
                    CoinGeckoAuth::new(key),
                )))
            }
            (FeedId::Coinglass, FeedCredentials::Credentials(c)) => {
                Ok(Box::new(
                    crate::intelligence_feeds::CoinglassConnector::new(c, 30).await?,
                ))
            }
            (FeedId::Coinglass, FeedCredentials::CredentialsWithRateLimit { creds, rate_limit_per_min }) => {
                Ok(Box::new(
                    crate::intelligence_feeds::CoinglassConnector::new(creds, rate_limit_per_min)
                        .await?,
                ))
            }
            (FeedId::Etherscan, FeedCredentials::ApiKey(key)) => {
                use crate::onchain::ethereum::etherscan::{EtherscanConnector, EtherscanAuth};
                Ok(Box::new(EtherscanConnector::new(EtherscanAuth::new(key))))
            }
            (FeedId::WhaleAlert, FeedCredentials::ApiKey(key)) => {
                use crate::onchain::analytics::whale_alert::{WhaleAlertConnector, WhaleAlertAuth};
                Ok(Box::new(WhaleAlertConnector::new(WhaleAlertAuth::new(key))))
            }
            // ── cyber ─────────────────────────────────────────────────────
            (FeedId::AbuseIpdb, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::cyber::abuseipdb::AbuseIpdbConnector;
                Ok(Box::new(AbuseIpdbConnector::new(key)))
            }
            (FeedId::OtxAlienVault, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::cyber::alienvault_otx::OtxConnector;
                Ok(Box::new(OtxConnector::new(key)))
            }
            (FeedId::Censys, FeedCredentials::ApiKeyPair { api_id, api_secret }) => {
                // CensysConnector::new takes api_id and api_secret directly
                Ok(Box::new(crate::intelligence_feeds::CensysConnector::new(
                    api_id, api_secret,
                )))
            }
            (FeedId::CloudflareRadar, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::cyber::cloudflare_radar::{CloudflareRadarConnector, CloudflareRadarAuth};
                Ok(Box::new(CloudflareRadarConnector::new(
                    CloudflareRadarAuth::new(key),
                )))
            }
            (FeedId::Nvd, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::cyber::nvd::{NvdConnector, NvdAuth};
                Ok(Box::new(NvdConnector::new(NvdAuth::new(key))))
            }
            (FeedId::Shodan, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::cyber::shodan::ShodanConnector;
                Ok(Box::new(ShodanConnector::new(key)))
            }
            (FeedId::Urlhaus, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::cyber::urlhaus::UrlhausConnector;
                Ok(Box::new(UrlhausConnector::new(key)))
            }
            (FeedId::VirusTotal, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::cyber::virustotal::VirusTotalConnector;
                Ok(Box::new(VirusTotalConnector::new(key)))
            }
            // ── demographics ──────────────────────────────────────────────
            (FeedId::Wikipedia, FeedCredentials::ApiKey(key)) => {
                // Wikipedia uses User-Agent header, not an API key; treat key as user_agent
                use crate::intelligence_feeds::demographics::wikipedia::{WikipediaConnector, WikipediaAuth};
                Ok(Box::new(WikipediaConnector::new(WikipediaAuth::with_user_agent(key))))
            }
            // ── economic ──────────────────────────────────────────────────
            (FeedId::Ecos, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::EcosAuth;
                Ok(Box::new(crate::intelligence_feeds::EcosConnector::new(
                    EcosAuth::new(key),
                )))
            }
            (FeedId::Fred, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::FredAuth;
                Ok(Box::new(crate::intelligence_feeds::FredConnector::new(
                    FredAuth::new(key),
                )))
            }
            // ── environment ───────────────────────────────────────────────
            (FeedId::GlobalForestWatch, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::environment::global_forest_watch::{GfwConnector, GfwAuth};
                Ok(Box::new(GfwConnector::new(GfwAuth::new(key))))
            }
            (FeedId::NasaFirms, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::NasaFirmsAuth;
                Ok(Box::new(crate::intelligence_feeds::NasaFirmsConnector::new(
                    NasaFirmsAuth::new(key),
                )))
            }
            (FeedId::Noaa, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::NoaaAuth;
                Ok(Box::new(crate::intelligence_feeds::NoaaConnector::new(
                    NoaaAuth::new(key),
                )))
            }
            (FeedId::OpenAq, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::environment::openaq::{OpenAqConnector, OpenAqAuth};
                Ok(Box::new(OpenAqConnector::new(OpenAqAuth::new(key))))
            }
            (FeedId::OpenWeatherMap, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::OpenWeatherMapAuth;
                Ok(Box::new(crate::intelligence_feeds::OpenWeatherMapConnector::new(
                    OpenWeatherMapAuth::new(key),
                )))
            }
            // ── financial ─────────────────────────────────────────────────
            (FeedId::AlphaVantage, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::AlphaVantageAuth;
                Ok(Box::new(crate::intelligence_feeds::AlphaVantageConnector::new(
                    AlphaVantageAuth::new(key),
                )))
            }
            (FeedId::Finnhub, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::FinnhubAuth;
                Ok(Box::new(crate::intelligence_feeds::FinnhubConnector::new(
                    FinnhubAuth::new(key),
                )))
            }
            (FeedId::NewsApi, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::NewsApiAuth;
                Ok(Box::new(crate::intelligence_feeds::NewsApiConnector::new(
                    NewsApiAuth::new(key),
                )))
            }
            (FeedId::OpenFigi, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::financial::openfigi::{OpenFigiConnector, OpenFigiAuth};
                Ok(Box::new(OpenFigiConnector::new(OpenFigiAuth::new(key))))
            }
            // ── maritime ──────────────────────────────────────────────────
            (FeedId::Ais, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::AisAuth;
                Ok(Box::new(crate::intelligence_feeds::AisConnector::new(
                    AisAuth::new(key),
                )))
            }
            (FeedId::AisStream, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::AisStreamAuth;
                Ok(Box::new(crate::intelligence_feeds::AisStreamConnector::new(
                    AisStreamAuth::new(key),
                )))
            }
            // ── sanctions ─────────────────────────────────────────────────
            (FeedId::Ofac, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::OfacAuth;
                Ok(Box::new(crate::intelligence_feeds::OfacConnector::new(
                    OfacAuth::new(key),
                )))
            }
            (FeedId::OpenSanctions, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::sanctions::opensanctions::{OpenSanctionsConnector, OpenSanctionsAuth};
                Ok(Box::new(OpenSanctionsConnector::new(
                    OpenSanctionsAuth::new(key),
                )))
            }
            // ── space ─────────────────────────────────────────────────────
            (FeedId::Nasa, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::space::nasa::{NasaConnector, NasaAuth};
                Ok(Box::new(NasaConnector::new(NasaAuth::new(key))))
            }
            (FeedId::SentinelHub, FeedCredentials::ApiKeyPair { api_id, api_secret }) => {
                use crate::intelligence_feeds::SentinelHubAuth;
                Ok(Box::new(crate::intelligence_feeds::SentinelHubConnector::new(
                    SentinelHubAuth::new(api_id, api_secret),
                )))
            }
            (FeedId::SpaceTrack, FeedCredentials::UsernamePassword { username, password }) => {
                use crate::intelligence_feeds::SpaceTrackAuth;
                Ok(Box::new(crate::intelligence_feeds::SpaceTrackConnector::new(
                    SpaceTrackAuth::new(username, password),
                )))
            }
            // ── trade ─────────────────────────────────────────────────────
            (FeedId::Comtrade, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::ComtradeAuth;
                Ok(Box::new(crate::intelligence_feeds::ComtradeConnector::new(
                    ComtradeAuth::new(key),
                )))
            }
            (FeedId::EuTed, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::trade::eu_ted::{EuTedConnector, EuTedAuth};
                Ok(Box::new(EuTedConnector::new(EuTedAuth::new(key))))
            }
            // ── us_gov ────────────────────────────────────────────────────
            (FeedId::Bea, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::BeaAuth;
                Ok(Box::new(crate::intelligence_feeds::BeaConnector::new(
                    BeaAuth::new(key),
                )))
            }
            (FeedId::Bls, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::BlsAuth;
                Ok(Box::new(crate::intelligence_feeds::BlsConnector::new(
                    BlsAuth::new(key),
                )))
            }
            (FeedId::Census, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::CensusAuth;
                Ok(Box::new(crate::intelligence_feeds::CensusConnector::new(
                    CensusAuth::new(key),
                )))
            }
            (FeedId::Congress, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::CongressAuth;
                Ok(Box::new(crate::intelligence_feeds::CongressConnector::new(
                    CongressAuth::new(key),
                )))
            }
            (FeedId::Eia, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::EiaAuth;
                Ok(Box::new(crate::intelligence_feeds::EiaConnector::new(
                    EiaAuth::new(key),
                )))
            }
            (FeedId::FbiCrime, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::FbiCrimeAuth;
                Ok(Box::new(crate::intelligence_feeds::FbiCrimeConnector::new(
                    FbiCrimeAuth::new(key),
                )))
            }
            (FeedId::SamGov, FeedCredentials::ApiKey(key)) => {
                use crate::intelligence_feeds::SamGovAuth;
                Ok(Box::new(crate::intelligence_feeds::SamGovConnector::new(
                    SamGovAuth::new(key),
                )))
            }
            // ── mismatch / missing credentials ────────────────────────────
            (id, _) => Err(ExchangeError::Auth(format!(
                "Invalid or mismatched credentials for {:?}",
                id
            ))),
        }
    }
}
