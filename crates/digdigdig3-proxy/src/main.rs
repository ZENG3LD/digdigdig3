//! `digdigdig3-proxy` — CORS proxy server for the digdigdig3 wasm build.
//!
//! Forwards browser REST requests to the curated 20-venue exchange
//! allowlist (`digdigdig3::core::proxy_allowlist`). Two supported request
//! shapes, both host-validated before forwarding:
//!
//! * **Prefix mode**: `GET /{host}/{*path}?query` — first path segment is
//!   the destination host.
//! * **Encoded mode**: `GET /proxy?url=<percent-encoded full URL>`.
//!
//! No auth — intended to bind to loopback only by default. Read-only
//! (GET/HEAD); anything else is rejected with 405.

use std::net::SocketAddr;

use clap::Parser;

/// Default bind port. See `nemo/d-watchdog/infra-cloudflared-docs/README.md`
/// port table — dig3 range 17800-17899 (digdigdig3-proxy = first entry).
const DEFAULT_PORT: u16 = 17800;

#[derive(Parser, Debug)]
#[command(name = "digdigdig3-proxy", version, about = "CORS proxy for digdigdig3 wasm REST calls")]
struct Cli {
    /// Bind address. Defaults to loopback-only — this proxy has no auth,
    /// do not expose it beyond localhost / a trusted reverse proxy.
    #[arg(long, env = "DIGDIGDIG3_PROXY_BIND")]
    bind: Option<SocketAddr>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let addr = cli
        .bind
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], DEFAULT_PORT)));

    let app = digdigdig3_proxy::router::build_router();

    tracing::info!(
        %addr,
        allowlist_size = digdigdig3::core::proxy_allowlist::REST_HOST_ALLOWLIST.len(),
        "digdigdig3-proxy listening"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
