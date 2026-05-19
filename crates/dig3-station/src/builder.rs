use crate::{Result, Station};

/// Fluent builder for [`Station`].
///
/// Phase 1: all configuration options are no-ops; `build()` constructs a
/// minimal `Station` with a fresh `ExchangeHub`. Persistence / cache /
/// multiplex / reconnect feature gates wire in step 7+.
#[derive(Debug, Default)]
pub struct StationBuilder {
    _priv: (),
}

impl StationBuilder {
    pub fn new() -> Self {
        Self { _priv: () }
    }

    pub async fn build(self) -> Result<Station> {
        Station::from_builder().await
    }
}
