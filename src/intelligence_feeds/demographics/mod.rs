//! Demographics, health, and population data

pub mod un_population;
pub mod who;
pub mod wikipedia;
pub mod un_ocha;

pub use un_population::{UnPopConnector, UnPopAuth};
pub use who::{WhoConnector, WhoAuth};
pub use wikipedia::{WikipediaConnector, WikipediaAuth, WikipediaParser};
pub use un_ocha::{
    UnOchaConnector, UnOchaAuth, UnOchaParser,
    PopulationData, FoodSecurityData, HumanitarianNeeds,
    OperationalPresence, FundingData, DisplacementData,
};
