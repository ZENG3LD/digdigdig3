use crate::{Result, Station};

/// Fluent builder for [`Station`].
///
/// Phase 1 stub: all configuration options are no-ops; `build()` constructs a
/// minimal `Station` with default `ExchangeHub`.
#[derive(Debug, Default)]
pub struct StationBuilder {
    _priv: (),
}

impl StationBuilder {
    pub fn new() -> Self {
        Self { _priv: () }
    }

    pub async fn build(self) -> Result<Station> {
        Station::new_stub().await
    }
}
