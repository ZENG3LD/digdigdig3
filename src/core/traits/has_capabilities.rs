//! Per-connector capability declaration.
//!
//! Every connector MUST explicitly implement `capabilities()` — there is no
//! default impl. This prevents silent all-false declarations and forces
//! conscious surface mapping.

use crate::core::types::{ConnectorCapabilities, ValidationStamp};

/// Declare the full capability surface of a connector.
///
/// Examined by the pool to filter connectors that can satisfy a given
/// operation before calling. Every `CoreConnector` implementor must provide
/// a fully explicit declaration — the trait intentionally has no default impl.
pub trait HasCapabilities: Send + Sync {
    /// Return a declarative map of what this connector supports.
    fn capabilities(&self) -> ConnectorCapabilities;

    /// Empirical validation results from the last `deep_smoke` harness run.
    ///
    /// `None` = connector was never smoke-tested with live exchange data.
    /// Connectors override this with a 1-liner delegating to `validation_snapshot::validation_for`.
    fn validation_status(&self) -> Option<&'static ValidationStamp> {
        None
    }
}
