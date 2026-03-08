//! US Treasury OFAC Sanctions List API connector
//!
//! Provides access to OFAC sanctions data via ofac-api.com service.
//!
//! # Features
//! - Search sanctioned entities by name
//! - Screen names/entities against SDN list
//! - List available sanction sources
//! - Access Specially Designated Nationals (SDN) list
//!
//! # Authentication
//! Set the `OFAC_API_KEY` environment variable with your API key from ofac-api.com
//!
//! # Rate Limits
//! Free tier available with limited requests. Paid tiers offer higher limits.
//!
//! # Example
//! ```ignore
//! use connectors_v5::data_feeds::ofac::OfacConnector;
//!
//! let connector = OfacConnector::from_env();
//!
//! // Search for sanctioned entities
//! let results = connector.search("Putin", None, None, None, None).await?;
//!
//! // Screen a name against SDN list
//! let screen = connector.screen("John Smith", None).await?;
//!
//! // Get available sanction sources
//! let sources = connector.get_sources().await?;
//! ```

pub mod auth;
pub mod connector;
pub mod endpoints;
pub mod parser;

pub use auth::OfacAuth;
pub use connector::OfacConnector;
pub use endpoints::{OfacEndpoint, OfacEndpoints};
pub use parser::{OfacEntity, OfacParser, OfacScreenResult, OfacSearchResult, OfacSource};
