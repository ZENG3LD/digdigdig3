//! # gRPC Transport
//!
//! Feature-gated behind `grpc` — only compiled when the `grpc` feature is enabled.
//!
//! Provides `GrpcClient`, a thin connection manager around `tonic::Channel`.
//! Connectors that use gRPC (e.g., Tinkoff Invest API) create a `GrpcClient`,
//! call `channel()`, then build typed tonic service stubs from the channel.
//!
//! ## Enable
//!
//! ```toml
//! [dependencies]
//! digdigdig3 = { version = "...", features = ["grpc"] }
//! ```

mod client;

pub use client::GrpcClient;
