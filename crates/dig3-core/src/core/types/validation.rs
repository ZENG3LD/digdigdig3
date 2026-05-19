//! # ValidationStamp
//!
//! Empirical validation evidence for a connector — populated by the `e2e_smoke` harness.
//! Replaces "declared" capabilities with observed truth: "this method returned real data on date X".

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Empirical validation result for a single REST method or WS stream.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldValidation {
    /// Method/stream returned real data; listed fields were non-zero/non-default.
    Validated { fields_populated: Vec<String> },
    /// Method/stream connected and returned data, but values were zero or default — parser bug.
    PopulatedButEmpty { missing_fields: Vec<String> },
    /// Method/stream did not work (auth error, symbol error, timeout, silent stream).
    Failed { reason: String },
    /// Not tested in this harness run.
    NotTested,
}

impl FieldValidation {
    /// Returns `true` if this is a full `Validated` result.
    pub fn is_validated(&self) -> bool {
        matches!(self, Self::Validated { .. })
    }

    /// Returns `true` if the result is non-failing (validated or partially populated).
    pub fn is_working(&self) -> bool {
        matches!(self, Self::Validated { .. } | Self::PopulatedButEmpty { .. })
    }
}

/// Empirical validation stamp for one connector, emitted by the `e2e_smoke` harness.
///
/// Attached to a connector via `HasCapabilities::validation_status()`.
/// Consumers can call `hub.connect_full_validated(id, ...)` to refuse connectors
/// with no stamp (never smoke-tested) or stale stamps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationStamp {
    /// ISO date (YYYY-MM-DD) when last validated against live exchange data.
    pub tested_on: String,
    /// Harness identifier (e.g. "e2e_smoke v1").
    pub harness_version: String,
    /// REST validation results, keyed by method name (e.g. "get_ticker").
    pub rest: BTreeMap<String, FieldValidation>,
    /// WS stream validation results, keyed by `StreamKind` name (e.g. "Ticker").
    pub ws: BTreeMap<String, FieldValidation>,
}

impl ValidationStamp {
    /// Returns `true` if all tested REST methods passed validation.
    pub fn rest_fully_validated(&self) -> bool {
        self.rest.values().all(FieldValidation::is_validated)
    }

    /// Returns `true` if all tested WS streams passed validation.
    pub fn ws_fully_validated(&self) -> bool {
        self.ws.values().all(FieldValidation::is_validated)
    }

    /// Returns `true` if both REST and WS are fully validated.
    pub fn fully_validated(&self) -> bool {
        self.rest_fully_validated() && self.ws_fully_validated()
    }

    /// Count of validated REST methods.
    pub fn rest_ok_count(&self) -> usize {
        self.rest.values().filter(|v| v.is_validated()).count()
    }

    /// Count of validated WS streams.
    pub fn ws_ok_count(&self) -> usize {
        self.ws.values().filter(|v| v.is_validated()).count()
    }
}
