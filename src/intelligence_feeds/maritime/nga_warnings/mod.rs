//! # NGA Maritime Warnings (NGA MSI) Connector
//!
//! Category: data_feeds/maritime
//! Type: Maritime Safety Information Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: None required
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Broadcast warnings: Yes (HYDROLANT, HYDROPAC)
//! - Navigational warnings: Yes (NAVAREA)
//! - Coastal warnings: Yes
//! - Local warnings: Yes
//! - Geographic filtering: Yes
//!
//! ## Key Endpoints
//! - /api/publications/broadcast-warn - Broadcast warnings (HYDROLANT, HYDROPAC)
//! - /api/publications/navwarn - Navigational warnings (NAVAREA)
//! - /api/publications/warn/{id} - Specific warning by ID
//!
//! ## Rate Limits
//! - No documented rate limits
//! - Respect the API with reasonable request intervals
//!
//! ## Data Coverage
//! - Global maritime warnings
//! - Areas: Atlantic, Pacific, Mediterranean, Indian, Arctic, Caribbean, etc.
//! - Types: Hydrographic, Navigational, Coastal, Local
//! - Historical depth: Active warnings (status=A)
//! - Update frequency: Real-time as warnings are issued
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Public data provided by NGA
//! - No authentication required
//!
//! ## Example Usage
//! ```ignore
//! use connectors_v5::data_feeds::maritime::nga_warnings::NgaWarningsConnector;
//!
//! let connector = NgaWarningsConnector::new();
//!
//! // Get all active warnings
//! let warnings = connector.get_active_warnings().await?;
//!
//! // Get warnings for specific area
//! let atlantic = connector.get_warnings_by_area("Atlantic").await?;
//!
//! // Get HYDROLANT warnings
//! let hydrolant = connector.get_hydrolant_warnings().await?;
//!
//! // Get HYDROPAC warnings
//! let hydropac = connector.get_hydropac_warnings().await?;
//!
//! // Get NAVAREA warnings
//! let navarea = connector.get_navarea_warnings().await?;
//! ```

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{NgaWarningsEndpoint, NgaWarningsEndpoints};
pub use auth::NgaWarningsAuth;
pub use parser::{
    NgaWarningsParser, MaritimeWarning, WarningType, WarningArea,
};
pub use connector::NgaWarningsConnector;
