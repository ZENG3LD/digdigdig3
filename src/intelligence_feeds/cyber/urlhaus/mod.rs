//! # URLhaus (abuse.ch) Connector
//!
//! Category: data_feeds
//! Type: Cybersecurity & Threat Intelligence
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header)
//! - Free tier: Yes
//!
//! ## Data Types
//! - Malicious URLs: Yes
//! - URL status (online/offline): Yes
//! - Threat classification: Yes (malware_download, phishing, crypto_mining)
//! - Malware payloads: Yes (SHA256 hashes, file info)
//! - Host reputation: Yes
//! - Tags: Yes (malware families, campaigns)
//!
//! ## Key Endpoints
//! - /urls/recent/limit/{limit}/ - Get recent malicious URLs (GET)
//! - /url/ - Lookup URL information (POST)
//! - /host/ - Lookup host information (POST)
//! - /payload/ - Lookup payload by SHA256 hash (POST)
//! - /tag/ - Lookup URLs by tag (POST)
//!
//! ## Rate Limits
//! - Free tier: Available with API key
//! - Rate limits: Reasonable for research purposes
//! - Max recent URLs: 1000 per request
//!
//! ## Data Coverage
//! - Global malicious URL database
//! - Malware distribution sites
//! - Phishing URLs
//! - Crypto mining sites
//! - Associated malware samples (hashes)
//! - URL status tracking (online/offline/unknown)
//!
//! ## Threat Types
//! - malware_download: Malware distribution
//! - phishing: Phishing campaigns
//! - crypto_mining: Cryptocurrency mining scripts
//!
//! ## Usage Restrictions
//! - API key recommended (env: URLHAUS_AUTH_KEY)
//! - Some endpoints work without authentication (recent URLs)
//! - POST endpoints may require authentication

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UrlhausEndpoint, UrlhausEndpoints};
pub use auth::UrlhausAuth;
pub use parser::{
    UrlhausParser, UrlhausEntry, UrlhausThreatType, UrlhausUrlInfo,
    UrlhausHostInfo, UrlhausPayload,
};
pub use connector::UrlhausConnector;
