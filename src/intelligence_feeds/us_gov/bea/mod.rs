//! BEA (Bureau of Economic Analysis) API connector
//!
//! Provides access to U.S. economic data including GDP, national income,
//! international transactions, and regional economic statistics.

pub mod auth;
pub mod endpoints;
pub mod parser;
pub mod connector;

pub use auth::BeaAuth;
pub use endpoints::{BeaEndpoints, BeaEndpoint, format_dataset_name, parse_dataset_name};
pub use connector::BeaConnector;
pub use parser::{
    BeaParser, BeaDataPoint, BeaDataset, BeaParameter, BeaParameterValue,
};
