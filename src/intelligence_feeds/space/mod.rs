//! Space, satellite, and Earth observation data

pub mod launch_library;
pub mod spacex;
pub mod space_track;
pub mod nasa;
pub mod sentinel_hub;

pub use launch_library::{LaunchLibraryConnector, LaunchLibraryAuth};
pub use spacex::{SpaceXConnector, SpaceXAuth, SpaceXParser, SpaceXLaunch, SpaceXRocket, SpaceXCrew, SpaceXStarlink};
pub use space_track::{SpaceTrackConnector, SpaceTrackAuth, SpaceTrackParser, Satellite, DecayPrediction, TleData};
pub use nasa::{NasaConnector, NasaAuth, NasaParser};
pub use sentinel_hub::{SentinelHubConnector, SentinelHubAuth, SentinelHubParser, SentinelCatalogResult, SentinelFeature, SentinelStatistical};
