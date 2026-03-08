//! # Wingbits Aircraft Enrichment Connector
//!
//! Category: data_feeds/aviation
//! Type: Aircraft Data Enrichment
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: API Key (header)
//! - Free tier: No (API key required)
//!
//! ## Data Types
//! - Aircraft registration: Yes
//! - Manufacturer details: Yes
//! - Operator information: Yes
//! - Owner information: Yes
//! - Aircraft type/model: Yes
//! - Category classification: Yes
//! - Military detection: Yes
//!
//! ## Key Endpoints
//! - /api/wingbits/details/{icao24} - Single aircraft lookup
//! - /api/wingbits/details/batch - Batch aircraft lookup (POST)
//!
//! ## Rate Limits
//! - Circuit breaker: 5 consecutive failures
//! - Client-side cache: 1 hour TTL, 2000 max entries
//!
//! ## Data Coverage
//! - Global aircraft registry data
//! - Owner and operator information
//! - Manufacturer and type details
//! - Military aircraft classification
//!
//! ## Usage Restrictions
//! - API key required (env: WINGBITS_API_KEY)
//! - Batch requests limited to ~100 aircraft per call
//!
//! ## Example
//! ```ignore
//! use connectors_v5::data_feeds::aviation::wingbits::WingbitsConnector;
//!
//! let connector = WingbitsConnector::from_env();
//!
//! // Single lookup
//! let details = connector.get_aircraft_details("a12345").await?;
//! println!("Registration: {:?}", details.registration);
//! println!("Operator: {:?}", details.operator);
//!
//! // Check if military
//! if connector.is_military(&details) {
//!     println!("Military aircraft detected");
//! }
//!
//! // Batch lookup
//! let icao24s = vec!["a12345", "a67890"];
//! let batch = connector.get_batch_details(&icao24s).await?;
//! for aircraft in batch {
//!     println!("{}: {:?}", aircraft.icao24, aircraft.registration);
//! }
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{WingbitsEndpoint, WingbitsEndpoints};
pub use auth::WingbitsAuth;
pub use parser::{
    WingbitsParser, AircraftDetails, AircraftCategory,
};
pub use connector::WingbitsConnector;
