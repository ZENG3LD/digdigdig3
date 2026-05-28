//! Runtime abstraction layer for native vs wasm32 targets.
//!
//! Three traits paper over the difference between `tokio` (native) and
//! browser primitives (wasm32-unknown-unknown):
//!
//! - [`Spawn`]: spawn a `'static` future on the current executor.
//!   Native = `tokio::spawn` (requires `Send`).
//!   Wasm = `wasm_bindgen_futures::spawn_local` (single-threaded, `!Send` ok).
//!
//! - [`Timer`]: sleep for a duration.
//!   Native = `tokio::time::sleep`.
//!   Wasm = `gloo_timers::future::sleep`.
//!
//! - [`WsConnector`] / [`WsConn`]: open and use a WebSocket connection.
//!   Native = `tokio_tungstenite`.
//!   Wasm = `web_sys::WebSocket` behind a channel actor (per websys-actor-design.md).
//!
//! ## Send-bound policy
//!
//! On native, `Spawn::spawn` requires `Future: Send + 'static` because
//! `tokio::spawn` requires it (multi-threaded scheduler). On wasm, the
//! browser event loop is single-threaded so `Future: 'static` suffices —
//! no `Send` bound. The same split applies to `Timer::sleep`'s return type
//! and `WsConnector`/`WsConn`'s async-trait bound.
//!
//! The `Runtime` façade exposes two spawn methods to avoid `unsafe transmute`:
//! - `spawn_send` — native-only, takes `Future + Send + 'static`
//! - `spawn_local` — wasm-only, takes `Future + 'static`
//!
//! `UniversalWsTransport` (Phase 3) uses cfg to call the appropriate one.
//! This avoids any `unsafe` code while keeping a single `Runtime` type.
//!
//! ## Usage
//!
//! ```ignore
//! let rt = default_runtime();
//! let conn = rt.connect_ws("wss://example.com", Duration::from_secs(10)).await?;
//! ```

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

// ─── Spawn ────────────────────────────────────────────────────────────────────

/// Spawn a future on the current runtime. See module docs for Send-bound policy.
#[cfg(not(target_arch = "wasm32"))]
pub trait Spawn: Send + Sync + 'static {
    fn spawn(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>);
}

/// Spawn a future on the browser event loop. No Send bound required.
#[cfg(target_arch = "wasm32")]
pub trait Spawn: 'static {
    fn spawn(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>);
}

// ─── Timer ────────────────────────────────────────────────────────────────────

/// Async sleep abstraction.
#[cfg(not(target_arch = "wasm32"))]
pub trait Timer: Send + Sync + 'static {
    fn sleep(&self, dur: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}

/// Async sleep abstraction (wasm — no Send bound).
#[cfg(target_arch = "wasm32")]
pub trait Timer: 'static {
    fn sleep(&self, dur: Duration) -> Pin<Box<dyn Future<Output = ()> + 'static>>;
}

// ─── WsRtError / WsFrame ─────────────────────────────────────────────────────

/// Error type for runtime-level WebSocket operations.
#[derive(Debug, thiserror::Error)]
pub enum WsRtError {
    #[error("connect: {0}")]
    Connect(String),
    #[error("send: {0}")]
    Send(String),
    #[error("recv: {0}")]
    Recv(String),
    #[error("closed by peer")]
    Closed,
    #[error("timeout")]
    Timeout,
}

/// A WebSocket frame at the runtime abstraction level.
#[derive(Debug, Clone, PartialEq)]
pub enum WsFrame {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

// ─── WsConnector / WsConn ────────────────────────────────────────────────────

/// Open a WebSocket connection. Returns a boxed [`WsConn`].
#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait WsConnector: Send + Sync + 'static {
    async fn connect(&self, url: &str, timeout: Duration) -> Result<Box<dyn WsConn>, WsRtError>;
}

/// Open a WebSocket connection (wasm — `?Send`).
#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait WsConnector: 'static {
    async fn connect(&self, url: &str, timeout: Duration) -> Result<Box<dyn WsConn>, WsRtError>;
}

/// An open WebSocket connection.
#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait WsConn: Send + 'static {
    async fn send(&mut self, frame: WsFrame) -> Result<(), WsRtError>;
    async fn next_frame(&mut self) -> Option<Result<WsFrame, WsRtError>>;
    async fn close(&mut self) -> Result<(), WsRtError>;
}

/// An open WebSocket connection (wasm — `?Send`).
#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait WsConn: 'static {
    async fn send(&mut self, frame: WsFrame) -> Result<(), WsRtError>;
    async fn next_frame(&mut self) -> Option<Result<WsFrame, WsRtError>>;
    async fn close(&mut self) -> Result<(), WsRtError>;
}

// ─── Concrete runtime impls ───────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Monotonic clock: `std::time::Instant` on native, `instant::Instant` on wasm32.
pub mod clock;

// ─── Runtime façade ───────────────────────────────────────────────────────────

/// Default runtime for the current target.
///
/// Constructed via [`default_runtime()`]. Holds the target-conditional
/// concrete impl (`TokioRuntime` on native, `WasmRuntime` on wasm).
///
/// Two spawn methods exist to avoid `unsafe` transmute between Send-bound variants:
/// - [`Runtime::spawn_send`] — native-only; takes a `Send + 'static` future.
/// - [`Runtime::spawn_local`] — wasm-only; takes a `'static` future (no Send).
///
/// Phase 3 (`UniversalWsTransport` migration) calls the right one via cfg.
pub struct Runtime(
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) native::TokioRuntime,
    #[cfg(target_arch = "wasm32")]
    pub(crate) wasm::WasmRuntime,
);

impl Runtime {
    /// Spawn a `Send + 'static` future. Available on native targets only.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        self.0.spawn(fut);
    }

    /// Spawn a `'static` future (no Send) on the browser microtask queue.
    /// Available on wasm32 targets only.
    #[cfg(target_arch = "wasm32")]
    pub fn spawn_local(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        self.0.spawn(fut);
    }

    /// Sleep for `dur`. Returns a `'static` future (Send on native, !Send on wasm).
    /// For use in cfg-conditional transport code.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn sleep(&self, dur: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        self.0.sleep(dur)
    }

    /// Sleep for `dur` (wasm — no Send bound).
    #[cfg(target_arch = "wasm32")]
    pub fn sleep(&self, dur: Duration) -> Pin<Box<dyn Future<Output = ()> + 'static>> {
        self.0.sleep(dur)
    }

    /// Open a WebSocket connection to `url` with `timeout`.
    pub async fn connect_ws(
        &self,
        url: &str,
        timeout: Duration,
    ) -> Result<Box<dyn WsConn>, WsRtError> {
        self.0.connect(url, timeout).await
    }
}

/// Construct the default runtime for the current compile target.
pub fn default_runtime() -> Runtime {
    #[cfg(not(target_arch = "wasm32"))]
    return Runtime(native::TokioRuntime);
    #[cfg(target_arch = "wasm32")]
    return Runtime(wasm::WasmRuntime);
}
