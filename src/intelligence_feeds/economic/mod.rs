//! International economic data sources (central banks, statistical offices)

pub mod fred;
pub mod worldbank;
pub mod dbnomics;
pub mod oecd;
pub mod eurostat;
pub mod ecb;
pub mod imf;
pub mod bis;
pub mod bundesbank;
pub mod ecos;
pub mod cbr;
pub mod boe;

pub use fred::{FredConnector, FredAuth, FredParser};
pub use worldbank::{WorldBankConnector, WorldBankAuth};
pub use dbnomics::{DBnomicsConnector, DBnomicsAuth, DBnomicsParser};
pub use oecd::{OecdConnector, OecdAuth, OecdParser};
pub use eurostat::{EurostatConnector, EurostatAuth, EurostatParser};
pub use ecb::{EcbConnector, EcbAuth};
pub use imf::{ImfConnector, ImfAuth};
pub use bis::{BisConnector, BisAuth};
pub use bundesbank::{BundesbankConnector, BundesbankAuth};
pub use ecos::{EcosConnector, EcosAuth};
pub use cbr::{CbrConnector, CbrAuth};
pub use boe::{BoeConnector, BoeAuth};
