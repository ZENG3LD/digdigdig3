//! # Auth — Credential-passing traits and types
//!
//! Auth is an INTERNAL implementation detail.
//! No signing logic is exposed in the public trait surface.
//!
//! Connectors that require authentication hold credentials internally
//! and sign requests in their own private methods.
//!
//! ## Design
//! - `Authenticated` — marks a connector as credential-aware
//! - `CredentialKind` — enum of all auth schemes (for capability reporting)
//! - `ExchangeCredentials` is in `types/trading.rs` (the data container)
//! - `Credentials` — simple API key + secret struct (used by connector constructors)

use std::collections::HashMap;
use crate::core::types::ExchangeResult;

// ═══════════════════════════════════════════════════════════════════════════════
// CREDENTIALS (backward compat — used by connector constructors)
// ═══════════════════════════════════════════════════════════════════════════════

/// Simple API key + secret credentials.
///
/// Used by connector constructors for the common HMAC-SHA256 case.
/// For the full multi-scheme credential model, use `ExchangeCredentials`.
#[derive(Clone)]
pub struct Credentials {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>,
    /// Whether to connect to the exchange's testnet/sandbox environment.
    ///
    /// Defaults to `false` (production). Set to `true` to use testnet endpoints.
    pub testnet: bool,
}

impl Credentials {
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            passphrase: None,
            testnet: false,
        }
    }

    pub fn with_passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    /// Set testnet mode.
    ///
    /// When `true`, the connector will use the exchange's testnet/sandbox endpoints.
    pub fn with_testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUTH REQUEST (backward compat — used by connector auth implementations)
// ═══════════════════════════════════════════════════════════════════════════════

/// Request structure for signing.
pub struct AuthRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub body: Option<&'a str>,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl<'a> AuthRequest<'a> {
    pub fn new(method: &'a str, path: &'a str) -> Self {
        Self {
            method,
            path,
            query: None,
            body: None,
            headers: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    pub fn with_query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    pub fn with_body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }
}

/// Where the signature goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureLocation {
    /// In headers (KuCoin, OKX, Bybit, Gate.io)
    Headers,
    /// In query params (Binance)
    QueryParams,
}

/// Exchange auth trait — each exchange implements its own signing logic.
///
/// This is the connector-internal trait for request signing.
/// It is NOT exposed in the public API surface.
pub trait ExchangeAuth: Send + Sync {
    fn sign_request(
        &self,
        credentials: &Credentials,
        req: &mut AuthRequest<'_>,
    ) -> ExchangeResult<()>;

    fn signature_location(&self) -> SignatureLocation {
        SignatureLocation::Headers
    }
}

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
    /// 1/43: HyperLiquid.
    EthereumWallet,

    /// Solana Ed25519 keypair.
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
