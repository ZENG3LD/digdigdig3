//! Aviation and flight tracking data

pub mod adsb_exchange;
pub mod opensky;
pub mod aviationstack;
pub mod wingbits;

pub use adsb_exchange::{AdsbExchangeConnector, AdsbExchangeAuth};
pub use opensky::{OpenskyConnector, OpenskyAuth};
pub use aviationstack::{AviationStackConnector, AviationStackAuth, AviationStackParser, AvFlight, AvAirport, AvAirline, AvFlightInfo, AvRoute};
pub use wingbits::{WingbitsConnector, WingbitsAuth, WingbitsParser, AircraftDetails, AircraftCategory};
