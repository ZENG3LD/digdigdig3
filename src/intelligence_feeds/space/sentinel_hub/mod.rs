//! # Copernicus Sentinel Hub API Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: OAuth2 Client Credentials
//! - Free tier: Yes (30,000 processing units/month)
//!
//! ## Data Types
//! - Satellite Imagery: Sentinel-1, Sentinel-2, Sentinel-3, Landsat, MODIS
//! - STAC Catalog: Search and discover satellite imagery
//! - Statistical Analysis: Time-series statistics from imagery
//! - Image Processing: Custom processing of satellite data
//!
//! ## Key Endpoints
//! - /api/v1/catalog/search - STAC catalog search
//! - /api/v1/statistical - Statistical analysis
//! - /api/v1/process - Image processing
//! - /oauth/token - OAuth2 authentication
//!
//! ## Rate Limits
//! - Free tier: 30,000 processing units per month
//! - Paid tier: Custom limits based on subscription

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{SentinelHubEndpoint, SentinelHubEndpoints};
pub use auth::SentinelHubAuth;
pub use parser::{
    SentinelHubParser, SentinelCatalogResult, SentinelFeature, SentinelContext,
    SentinelStatistical, StatisticalBand, BandStats,
};
pub use connector::SentinelHubConnector;
