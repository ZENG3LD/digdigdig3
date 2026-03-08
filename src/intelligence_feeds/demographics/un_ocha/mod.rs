//! # UN OCHA HAPI (Humanitarian API) Connector
//!
//! Category: data_feeds/demographics
//! Type: Humanitarian Data Provider
//!
//! ## Features
//! - REST API: Yes
//! - WebSocket: No
//! - Authentication: Optional (app identifier header)
//! - Free tier: Yes (public API with recommended limits)
//!
//! ## Data Types
//! - Population data: Yes (by location, with demographic breakdowns)
//! - Food security: Yes (IPC classification with phases 1-5)
//! - Humanitarian needs: Yes (by sector - Health, Shelter, Food, WASH, Protection)
//! - Displacement: Yes (refugees, IDPs, returnees, asylum seekers, stateless)
//! - Operational presence: Yes (humanitarian organizations by location and sector)
//! - Funding: Yes (requirements, received, gaps, percent funded)
//!
//! ## Key Endpoints
//! - /api/v1/population - Population statistics by location
//! - /api/v1/food-security - IPC food security phases
//! - /api/v1/humanitarian-needs - Sector-based needs assessments
//! - /api/v1/operational-presence - Organization presence mapping
//! - /api/v1/funding - Humanitarian funding data
//! - /api/v1/refugees - Refugee statistics by origin and asylum
//! - /api/v1/idps - Internally Displaced Persons data
//! - /api/v1/returnees - Returnee statistics
//!
//! ## Rate Limits
//! - Recommended: Be respectful, no hard limits documented
//! - Burst: Avoid excessive parallel requests
//! - Consider using app identifier for tracking
//!
//! ## Data Coverage
//! - Geographic: Global, with focus on crisis-affected countries
//! - Temporal: Varies by dataset (typically current + recent years)
//! - Updates: Regular updates from UN agencies, NGOs, and government sources
//! - Granularity: Country and admin-level 1/2 where available
//!
//! ## Data Sources
//! - UNHCR (refugees, asylum, IDPs)
//! - IPC (food security classifications)
//! - OCHA (humanitarian needs, operational presence, funding)
//! - National governments
//! - UN agencies (UNICEF, WFP, WHO, etc.)
//!
//! ## Use Cases
//! - Crisis monitoring and early warning
//! - Humanitarian resource allocation
//! - Displacement tracking
//! - Food security monitoring
//! - Funding gap analysis
//! - Operational coordination
//!
//! ## Location Codes
//! - ISO-3 country codes: "AFG" (Afghanistan), "SYR" (Syria), "SOM" (Somalia)
//! - P-codes: Admin-level codes for sub-national areas
//!
//! ## IPC Phases (Food Security)
//! - Phase 1: Minimal - Food secure
//! - Phase 2: Stressed - Marginally food secure
//! - Phase 3: Crisis - Acute food insecurity
//! - Phase 4: Emergency - Humanitarian emergency
//! - Phase 5: Catastrophe/Famine - Extreme crisis
//!
//! ## Common Sectors
//! - Food Security
//! - Health
//! - Shelter
//! - WASH (Water, Sanitation, Hygiene)
//! - Protection
//! - Education
//! - Nutrition
//! - Camp Coordination
//!
//! ## Usage Restrictions
//! - Free for humanitarian, research, and operational use
//! - Attribution requested
//! - Respect rate limits
//! - Optional app identifier recommended for tracking

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{UnOchaEndpoint, UnOchaEndpoints};
pub use auth::UnOchaAuth;
pub use parser::{
    UnOchaParser, PopulationData, FoodSecurityData, HumanitarianNeeds,
    OperationalPresence, FundingData, DisplacementData,
};
pub use connector::UnOchaConnector;
