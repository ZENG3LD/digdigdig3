//! # ECB (European Central Bank) Connector
//!
//! Category: data_feeds
//! Type: Data Provider
//!
//! ## Features
//! - REST API: Yes (SDMX 2.1)
//! - WebSocket: No
//! - Authentication: None (public API)
//! - Free tier: Yes (completely free)
//!
//! ## Data Types
//! - Price data: No (economic data only)
//! - Historical data: Yes (extensive time series)
//! - Macro data: Yes (primary use case)
//! - Economic indicators: Yes
//! - Statistical data: Yes (EU/eurozone statistics)
//!
//! ## Key Endpoints
//! - /data/{dataflow}/{key} - Get time series data (CORE endpoint)
//! - /dataflow/ECB - List all available dataflows
//! - /dataflow/ECB/{id}/latest - Get dataflow metadata
//! - /datastructure/ECB/{id} - Get data structure definition
//! - /codelist/ECB/{id} - Get codelist (dimension values)
//!
//! ## Rate Limits
//! - No documented rate limits
//! - Fair use policy applies
//!
//! ## Data Coverage
//! - European Central Bank official statistics
//! - Eurozone and EU economic indicators
//! - Exchange rates, interest rates, monetary aggregates
//! - Balance sheets, securities statistics
//! - National accounts, prices, external sector
//!
//! ## Key Dataflows
//! - **EXR**: Exchange rates (e.g., EUR/USD)
//! - **FM**: Financial market data (interest rates)
//! - **BSI**: Balance sheet items (money supply)
//! - **ICP**: Index of consumer prices (HICP inflation)
//! - **MNA**: Main national accounts (GDP)
//! - **BOP**: Balance of payments
//! - **GFS**: Government finance statistics
//! - **SEC**: Securities statistics
//!
//! ## Example Keys
//! - Exchange rates: `D.USD.EUR.SP00.A` (daily USD/EUR spot rate)
//!   - D = Daily frequency
//!   - USD = Currency (US Dollar)
//!   - EUR = Currency (Euro)
//!   - SP00 = Exchange rate type (Spot)
//!   - A = Variation (Average)
//!
//! ## Usage Restrictions
//! - Free for all use
//! - Attribution recommended
//! - No redistribution restrictions for non-commercial use

mod endpoints;
mod auth;
mod parser;
mod connector;

pub use endpoints::{EcbEndpoint, EcbEndpoints};
pub use auth::EcbAuth;
pub use parser::{
    EcbParser, SdmxObservation, SdmxDataflow, SdmxDataStructure,
    SdmxCodelist, SdmxCode,
};
pub use connector::EcbConnector;
