//! dig3-server — gRPC IPC daemon for ExchangeHub + StorageManager.
//!
//! Build: `cargo build --bin dig3-server --features server --release`

#[cfg(feature = "server")]
mod inner {
    use std::path::PathBuf;

    use tonic::transport::Server;
    use tracing::info;
    use tracing_subscriber::EnvFilter;

    use digdigdig3::connector_manager::ExchangeHub;
    use digdigdig3::core::storage::StorageConfig;
    use digdigdig3::server::{
        config::ServerConfig,
        health::HealthService,
        live_events::LiveEventsService,
        proto::{
            health_server::HealthServer, live_events_server::LiveEventsServer,
            rest_proxy_server::RestProxyServer, storage_read_server::StorageReadServer,
        },
        rest_proxy::RestProxyService,
        state::ServerState,
        storage_read::StorageReadService,
        subscriber,
    };

    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        // ── Tracing ───────────────────────────────────────────────────────────
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info,dig3=debug")),
            )
            .init();

        // ── CLI ───────────────────────────────────────────────────────────────
        let args: Vec<String> = std::env::args().collect();
        let mut config_path: Option<PathBuf> = None;
        let mut grpc_override: Option<String> = None;
        let mut storage_override: Option<PathBuf> = None;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--config" => {
                    i += 1;
                    config_path = args.get(i).map(PathBuf::from);
                }
                "--grpc-addr" => {
                    i += 1;
                    grpc_override = args.get(i).cloned();
                }
                "--storage-root" => {
                    i += 1;
                    storage_override = args.get(i).map(PathBuf::from);
                }
                _ => {}
            }
            i += 1;
        }

        // ── Config ────────────────────────────────────────────────────────────
        let mut cfg = match config_path {
            Some(ref p) => ServerConfig::from_file(p)?,
            None => ServerConfig::default(),
        };
        if let Some(addr) = grpc_override {
            cfg.grpc_addr = addr;
        }
        if let Some(root) = storage_override {
            cfg.storage_root = root;
        }

        let bind: std::net::SocketAddr = cfg.grpc_addr.parse()?;
        info!("dig3-server starting on {}", bind);

        // ── Hub ───────────────────────────────────────────────────────────────
        let hub = ExchangeHub::new();
        let mut ws_pairs: Vec<(digdigdig3::core::types::ExchangeId, digdigdig3::core::types::AccountType)> =
            Vec::new();

        for entry in &cfg.exchanges {
            info!("connecting {:?} (testnet={})", entry.id, entry.testnet);
            if let Err(e) = hub
                .connect_full(entry.id, &entry.account_types, entry.testnet)
                .await
            {
                tracing::warn!("connect {:?}: {}", entry.id, e);
            } else {
                for &acct in &entry.account_types {
                    ws_pairs.push((entry.id, acct));
                }
            }
        }

        // ── Storage ───────────────────────────────────────────────────────────
        let storage_cfg = StorageConfig {
            root: cfg.storage_root.clone(),
            ..Default::default()
        };
        let storage = ServerState::build_storage(storage_cfg)?;

        // ── State ─────────────────────────────────────────────────────────────
        let state = ServerState::new(hub, storage);

        // ── WS subscriber tasks ───────────────────────────────────────────────
        subscriber::spawn_all(&state, &ws_pairs).await;

        // ── gRPC server ───────────────────────────────────────────────────────
        let live_events = LiveEventsServer::new(LiveEventsService {
            state: state.clone(),
        });
        let rest_proxy = RestProxyServer::new(RestProxyService {
            state: state.clone(),
        });
        let storage_read = StorageReadServer::new(StorageReadService {
            state: state.clone(),
        });
        let health = HealthServer::new(HealthService { state });

        info!("gRPC listening on {}", bind);

        Server::builder()
            .add_service(live_events)
            .add_service(rest_proxy)
            .add_service(storage_read)
            .add_service(health)
            .serve_with_shutdown(bind, shutdown_signal())
            .await?;

        info!("dig3-server stopped");
        Ok(())
    }

    async fn shutdown_signal() {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("ctrl-c received, shutting down");
    }
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::run().await
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!(
        "dig3-server requires `server` feature:\n  \
         cargo build --bin dig3-server --features server --release"
    );
    std::process::exit(1);
}
