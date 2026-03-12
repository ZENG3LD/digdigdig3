//! # AuthV2 — Minimal credential-passing trait (V2 architecture)
//!
//! Auth is an INTERNAL implementation detail.
//! No `sign_request` is exposed in the public trait surface.
//!
//! Connectors that require authentication hold credentials internally
//! and sign requests in their own private methods.
//!
//! ## Design
//! - `Authenticated` — marks a connector as credential-aware
//! - `CredentialKind` — enum of all auth schemes (for capability reporting)
//! - `ExchangeCredentials` is in `types/trading_v2.rs` (the data container)

/// Enum of all authentication scheme kinds across 24 exchanges.
///
/// Used for capability discovery and client-side validation
/// (e.g. "does this connector accept Ethereum wallet credentials?").
///
/// This is a pure descriptor — it contains no credentials data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKind {
    /// HMAC-SHA256 with API key + secret.
    ///
    /// 12/24: Binance, Bybit, GateIO, Bitfinex, Bitstamp, Gemini, MEXC,
    /// HTX, BingX, Phemex, CryptoCom, Upbit.
    HmacSha256,

    /// HMAC-SHA256 with API key + secret + passphrase.
    ///
    /// 3/24: OKX, KuCoin, Bitget.
    HmacWithPassphrase,

    /// HMAC-SHA512.
    ///
    /// 1/24: Kraken.
    HmacSha512,

    /// HMAC-SHA384.
    ///
    /// 1/24: Deribit (some endpoints).
    HmacSha384,

    /// JWT signed with EC P-256 private key (ES256).
    ///
    /// 1/24: Coinbase Advanced Trade.
    JwtEs256,

    /// JWT signed with HMAC-SHA256 secret.
    ///
    /// 1/24: Paradex.
    JwtHmac,

    /// OAuth 2.0 bearer token.
    ///
    /// 1/24: Upstox, some Indian brokers.
    OAuth2,

    /// Ethereum ECDSA wallet signing (EIP-712).
    ///
    /// 2/24: HyperLiquid, GMX.
    EthereumWallet,

    /// Solana Ed25519 keypair.
    ///
    /// 1/24: Jupiter, Raydium.
    SolanaKeypair,

    /// StarkEx / StarkNet STARK key.
    ///
    /// 2/24: Lighter, Paradex.
    StarkKey,

    /// Cosmos SDK wallet (Tendermint).
    ///
    /// 1/24: dYdX v4.
    CosmosWallet,
}

/// Marks a connector as credential-aware and capable of authenticated requests.
///
/// Connectors that ONLY support public endpoints (e.g. read-only data feeds)
/// do NOT implement this trait.
///
/// # Auth is internal
/// Connectors sign requests internally. This trait only controls
/// credential storage and the ability to check authentication state.
/// Callers use `set_credentials` once at construction time and then
/// call the trading/account trait methods normally.
pub trait Authenticated: Send + Sync {
    /// Store credentials in the connector for use in all subsequent requests.
    ///
    /// Calling this replaces any previously stored credentials.
    /// The connector validates the credential variant matches its expected
    /// type and returns `ExchangeError::InvalidCredentials` if mismatched.
    fn set_credentials(&mut self, creds: crate::core::types::ExchangeCredentials);

    /// Returns `true` if credentials have been set and the connector
    /// is ready to make authenticated requests.
    fn is_authenticated(&self) -> bool;

    /// Returns the credential scheme this connector accepts, or `None`
    /// if the connector only supports public (unauthenticated) requests.
    fn credential_type(&self) -> Option<CredentialKind>;
}
