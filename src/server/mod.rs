//! Phase ο — dig3-server gRPC IPC daemon.
//!
//! Feature-gated behind `server` (which implies `grpc`).
//! Exposes 4 gRPC services:
//!
//! - `LiveEvents`   — server-streaming: subscribe to live WS events
//! - `RestProxy`    — unary: get_ticker / get_klines / get_orderbook via hub
//! - `StorageRead`  — server-streaming: read_range from EventLog
//! - `Health`       — unary: uptime + capabilities + sub count

pub mod bus;
pub mod config;
pub mod health;
pub mod live_events;
pub mod proto;
pub mod rest_proxy;
pub mod state;
pub mod storage_read;
pub mod subscriber;

pub use bus::EventBus;
pub use config::ServerConfig;
pub use state::ServerState;
