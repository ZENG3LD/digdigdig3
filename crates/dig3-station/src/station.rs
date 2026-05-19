use crate::{Result, StationBuilder, SubscriptionHandle, SubscriptionSet};

/// Phase 1 stub `Station`. Full design lives in `docs/plans/station-architecture.md`.
#[derive(Debug)]
pub struct Station {
    _priv: (),
}

impl Station {
    pub fn builder() -> StationBuilder {
        StationBuilder::new()
    }

    pub(crate) async fn new_stub() -> Result<Self> {
        Ok(Self { _priv: () })
    }

    pub async fn subscribe(&self, _set: SubscriptionSet) -> Result<SubscriptionHandle> {
        Ok(SubscriptionHandle::stub())
    }
}
