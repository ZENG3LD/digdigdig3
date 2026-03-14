//! # gRPC Client
//!
//! Connection manager for gRPC endpoints using `tonic`.
//! Each connector that needs gRPC holds a `GrpcClient` and calls `channel()`
//! to get a `tonic::Channel` for building typed service clients.
//!
//! ## Usage
//!
//! ```rust,no_run
//! # #[cfg(feature = "grpc")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use digdigdig3::core::grpc::GrpcClient;
//!
//! // Connect with TLS
//! let client = GrpcClient::connect("https://invest-public-api.tinkoff.ru:443").await?;
//!
//! // Use the channel to create a typed stub
//! // let mut stub = SomeServiceClient::new(client.channel());
//! # Ok(())
//! # }
//! ```

use tonic::transport::{Channel, ClientTlsConfig, Endpoint};

use crate::core::types::{ExchangeError, ExchangeResult};

/// gRPC connection manager.
///
/// Wraps a `tonic::Channel` and exposes it for building typed service clients.
/// Each connector that uses gRPC creates one `GrpcClient` and clones the inner
/// channel (cheap — `Channel` is backed by an `Arc`) per service stub.
pub struct GrpcClient {
    channel: Channel,
}

impl GrpcClient {
    /// Connect to a gRPC endpoint with TLS using native roots.
    ///
    /// `url` must be a full URI including scheme, host, and port,
    /// e.g. `"https://invest-public-api.tinkoff.ru:443"`.
    pub async fn connect(url: &str) -> ExchangeResult<Self> {
        let tls = ClientTlsConfig::new().with_native_roots();

        let channel = Endpoint::from_shared(url.to_string())
            .map_err(|e| ExchangeError::Network(format!("Invalid gRPC URL: {}", e)))?
            .tls_config(tls)
            .map_err(|e| ExchangeError::Network(format!("TLS config error: {}", e)))?
            .connect()
            .await
            .map_err(|e| ExchangeError::Network(format!("gRPC connect failed: {}", e)))?;

        Ok(Self { channel })
    }

    /// Connect to a gRPC endpoint; the returned client is suitable for use
    /// with a per-call bearer token interceptor.
    ///
    /// The token is NOT automatically injected here — callers attach it via a
    /// `tonic::service::interceptor` when constructing the service stub:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "grpc")]
    /// # async fn example(token: String) -> Result<(), Box<dyn std::error::Error>> {
    /// use digdigdig3::core::grpc::GrpcClient;
    /// use tonic::metadata::AsciiMetadataValue;
    ///
    /// let client = GrpcClient::connect_with_token(
    ///     "https://invest-public-api.tinkoff.ru:443",
    ///     &token,
    /// ).await?;
    ///
    /// // let auth_value = format!("Bearer {}", token).parse::<AsciiMetadataValue>()?;
    /// // let stub = SomeServiceClient::with_interceptor(client.channel(), move |mut req| {
    /// //     req.metadata_mut().insert("authorization", auth_value.clone());
    /// //     Ok(req)
    /// // });
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_token(url: &str, _token: &str) -> ExchangeResult<Self> {
        // Transport is identical to `connect`; token is attached per-request
        // by the connector via a tonic interceptor, not at channel level.
        Self::connect(url).await
    }

    /// Return a clone of the underlying `tonic::Channel`.
    ///
    /// `Channel` is cheaply cloneable (`Arc`-backed). Pass the clone to a
    /// generated tonic service client:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "grpc")]
    /// # async fn example(grpc: digdigdig3::core::grpc::GrpcClient) {
    /// // let stub = SomeServiceClient::new(grpc.channel());
    /// # }
    /// ```
    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }
}
