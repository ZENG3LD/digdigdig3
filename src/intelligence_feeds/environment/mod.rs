//! Environment, weather, and natural hazard data

pub mod noaa;
pub mod openaq;
pub mod open_weather_map;
pub mod nasa_firms;
pub mod nasa_eonet;
pub mod global_forest_watch;
pub mod usgs_earthquake;
pub mod gdacs;
pub mod nws_alerts;

pub use noaa::{NoaaConnector, NoaaAuth};
pub use openaq::{OpenAqConnector, OpenAqAuth, OpenAqParser};
pub use open_weather_map::{OpenWeatherMapConnector, OpenWeatherMapAuth};
pub use nasa_firms::{NasaFirmsConnector, NasaFirmsAuth, NasaFirmsParser, FireHotspot, FireSummary, CountryFireCount};
pub use nasa_eonet::{NasaEonetConnector, NasaEonetAuth, NasaEonetParser, NaturalEvent, EventCategory, EventSource, EventGeometry};
pub use global_forest_watch::{GfwConnector, GfwAuth};
pub use usgs_earthquake::{UsgsEarthquakeConnector, UsgsEarthquakeAuth};
pub use gdacs::{GdacsConnector, GdacsAuth, GdacsParser, DisasterEvent, DisasterType, AlertLevel};
pub use nws_alerts::{NwsAlertsConnector, NwsAlertsAuth, NwsAlertsParser, WeatherAlert, Severity, Certainty, Urgency};
