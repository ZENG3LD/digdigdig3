//! CapabilityProvider trait — query a connector's WebSocket stream capabilities.

use crate::core::types::AccountType;

use super::{stream_kind::StreamKind, support_level::SupportLevel};

/// Query a connector's WebSocket stream capabilities.
///
/// Implemented by UniversalWsTransport<P> via derivation from the TopicRegistry.
pub trait CapabilityProvider: Send + Sync {
    /// What level of support exists for (kind, account_type)?
    ///
    /// - `Native` ← registry has a parser entry for (kind, account)
    /// - `UnsupportedByExchange` ← no registry entry AND exchange impl
    ///   explicitly tagged this kind as "exchange has no channel"
    /// - `NotImplemented` ← no registry entry AND no explicit tag
    ///   (dig3 hasn't wired it yet)
    /// - `RequiresAuth` ← registry entry tagged with RequiresAuth
    fn supports(&self, kind: &StreamKind, account: AccountType) -> SupportLevel;

    /// Convenience: returns true iff supports() == Native.
    fn is_native(&self, kind: &StreamKind, account: AccountType) -> bool {
        self.supports(kind, account) == SupportLevel::Native
    }
}
