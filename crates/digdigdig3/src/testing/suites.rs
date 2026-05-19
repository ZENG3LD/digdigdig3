//! # Test Suites
//!
//! Each submodule contains test functions for a specific core trait.
//! Test functions take a connector reference and validate its behaviour
//! against real exchange responses.

#[path = "suites/market_data.rs"]
pub mod market_data;
#[path = "suites/trading.rs"]
pub mod trading;
#[path = "suites/account.rs"]
pub mod account;
#[path = "suites/positions.rs"]
pub mod positions;
#[path = "suites/operations.rs"]
pub mod operations;

// ═══════════════════════════════════════════════════════════════════════════════
// SHARED RESULT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

use std::fmt;

/// Outcome of a single test function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    /// Test completed and all assertions passed.
    Passed,
    /// Test completed but an assertion failed (data was wrong).
    Failed,
    /// The connector returned `UnsupportedOperation` — feature not present.
    Skipped,
    /// A network, authentication, or infrastructure error occurred.
    Error,
}

impl fmt::Display for TestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Passed  => write!(f, "PASS"),
            Self::Failed  => write!(f, "FAIL"),
            Self::Skipped => write!(f, "SKIP"),
            Self::Error   => write!(f, "ERR "),
        }
    }
}

/// Result of a single test function execution.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Name of the test function (e.g. `"test_get_price"`).
    pub test_name: String,
    /// Exchange name returned by `ExchangeIdentity::exchange_name()`.
    pub exchange: String,
    /// Pass / Fail / Skip / Error.
    pub status: TestStatus,
    /// Human-readable detail — assertion failure reason, error text, or skip reason.
    pub message: Option<String>,
    /// Wall-clock duration of the test in milliseconds.
    pub duration_ms: u64,
}

impl TestResult {
    /// Convenience constructor for a passing test.
    pub fn pass(test_name: impl Into<String>, exchange: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            test_name: test_name.into(),
            exchange: exchange.into(),
            status: TestStatus::Passed,
            message: None,
            duration_ms,
        }
    }

    /// Convenience constructor for a failing test.
    pub fn fail(
        test_name: impl Into<String>,
        exchange: impl Into<String>,
        duration_ms: u64,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            test_name: test_name.into(),
            exchange: exchange.into(),
            status: TestStatus::Failed,
            message: Some(reason.into()),
            duration_ms,
        }
    }

    /// Convenience constructor for a skipped (UnsupportedOperation) test.
    pub fn skip(
        test_name: impl Into<String>,
        exchange: impl Into<String>,
        duration_ms: u64,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            test_name: test_name.into(),
            exchange: exchange.into(),
            status: TestStatus::Skipped,
            message: Some(reason.into()),
            duration_ms,
        }
    }

    /// Convenience constructor for an error (network / auth) result.
    pub fn error(
        test_name: impl Into<String>,
        exchange: impl Into<String>,
        duration_ms: u64,
        err: impl Into<String>,
    ) -> Self {
        Self {
            test_name: test_name.into(),
            exchange: exchange.into(),
            status: TestStatus::Error,
            message: Some(err.into()),
            duration_ms,
        }
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}/{}", self.status, self.exchange, self.test_name)?;
        if let Some(ref msg) = self.message {
            write!(f, " — {}", msg)?;
        }
        write!(f, " ({}ms)", self.duration_ms)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

use crate::core::types::ExchangeError;

/// Returns `true` if `err` represents an unsupported operation.
///
/// Used throughout the suites to translate `UnsupportedOperation` → `Skipped`.
pub fn is_unsupported(err: &ExchangeError) -> bool {
    matches!(err, ExchangeError::UnsupportedOperation(_) | ExchangeError::NotSupported(_))
}

/// Returns `true` if `err` is an authentication or credentials error.
pub fn is_auth_error(err: &ExchangeError) -> bool {
    matches!(
        err,
        ExchangeError::Auth(_)
            | ExchangeError::InvalidCredentials(_)
            | ExchangeError::PermissionDenied(_)
    )
}

// Re-export assertion helpers so callers can do `use crate::testing::suites::*`
pub use crate::testing::assertions::{
    assert_balance_sane, assert_kline_sane, assert_orderbook_sane,
    assert_position_sane, assert_price_sane, assert_ticker_sane,
};
pub use crate::testing::harness::TestHarness;
