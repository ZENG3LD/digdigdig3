//! `digdigdig3-proxy` library surface — exposed so integration tests can
//! drive the router with `tower::ServiceExt::oneshot` without a live socket.
//! The binary (`src/main.rs`) is a thin wrapper: parse CLI args, bind, serve.

pub mod error;
pub mod forward;
pub mod router;
pub mod target;
