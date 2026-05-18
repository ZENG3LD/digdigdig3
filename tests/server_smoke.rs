//! Phase ο: server smoke tests.
//!
//! These tests require `--features server` and network access for live exchange
//! tests. All live/network tests are marked `#[ignore]`.
//!
//! Run offline tests:
//!   cargo test --test server_smoke --features server
//!
//! Run all (requires live exchanges):
//!   cargo test --test server_smoke --features server -- --include-ignored

#![cfg(feature = "server")]

use std::time::Duration;

use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::storage::{StorageConfig, StorageManager};
use digdigdig3::server::{bus::EventBus, bus::BusEvent, state::ServerState};

// ── EventBus unit tests ───────────────────────────────────────────────────────

#[test]
fn bus_publish_receive() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe();

    let ev = BusEvent {
        exchange: "binance".into(),
        account: "spot".into(),
        symbol: "btcusdt".into(),
        stream_kind: "ticker".into(),
        timestamp_ms: 1_700_000_000_000,
        event_type: "Ticker".into(),
        payload_json: b"{}".to_vec(),
    };
    bus.publish(ev.clone());

    let received = rx.try_recv().expect("should receive published event");
    assert_eq!(received.exchange, "binance");
    assert_eq!(received.symbol, "btcusdt");
}

#[test]
fn bus_active_subscriptions_count() {
    let bus = EventBus::new();
    assert_eq!(bus.active_subscriptions(), 0);

    let _rx1 = bus.subscribe();
    assert_eq!(bus.active_subscriptions(), 1);

    let _rx2 = bus.subscribe();
    assert_eq!(bus.active_subscriptions(), 2);

    bus.unsubscribe();
    assert_eq!(bus.active_subscriptions(), 1);
}

// ── ServerState unit tests ────────────────────────────────────────────────────

#[test]
fn server_state_uptime() {
    let storage_cfg = StorageConfig {
        root: std::env::temp_dir().join("dig3_server_smoke_test"),
        ..Default::default()
    };
    let storage = StorageManager::new(storage_cfg).expect("storage init");
    let hub = ExchangeHub::new();
    let state = ServerState::new(hub, storage);

    // Uptime should be near 0 immediately after creation.
    assert!(state.uptime_secs() < 5);
    assert_eq!(state.hub.len_rest(), 0);
    assert_eq!(state.bus.active_subscriptions(), 0);
}

// ── Health service unit test ──────────────────────────────────────────────────

#[tokio::test]
async fn health_response_zero_state() {
    use digdigdig3::server::health::HealthService;
    use digdigdig3::server::proto::{health_server::Health, HealthRequest};
    use tonic::Request;

    let storage_cfg = StorageConfig {
        root: std::env::temp_dir().join("dig3_server_smoke_health"),
        ..Default::default()
    };
    let storage = StorageManager::new(storage_cfg).expect("storage init");
    let hub = ExchangeHub::new();
    let state = ServerState::new(hub, storage);
    let svc = HealthService { state };

    let resp = svc
        .status(Request::new(HealthRequest {}))
        .await
        .expect("health ok");
    let inner = resp.into_inner();

    assert_eq!(inner.connected_exchanges, 0);
    assert_eq!(inner.active_subscriptions, 0);
    assert!(inner.uptime_secs >= 0);
}

// ── Full server integration test (requires network) ───────────────────────────

/// Start a server, connect a tonic client, call Health::Status.
#[tokio::test]
#[ignore]
async fn server_grpc_health_roundtrip() {
    use std::net::SocketAddr;

    use digdigdig3::server::health::HealthService;
    use digdigdig3::server::live_events::LiveEventsService;
    use digdigdig3::server::proto::{
        health_client::HealthClient, health_server::HealthServer,
        live_events_server::LiveEventsServer, rest_proxy_server::RestProxyServer,
        storage_read_server::StorageReadServer, HealthRequest,
    };
    use digdigdig3::server::rest_proxy::RestProxyService;
    use digdigdig3::server::storage_read::StorageReadService;
    use tonic::transport::Server;

    let addr: SocketAddr = "127.0.0.1:18269".parse().unwrap();

    let storage_cfg = StorageConfig {
        root: std::env::temp_dir().join("dig3_server_smoke_roundtrip"),
        ..Default::default()
    };
    let storage = StorageManager::new(storage_cfg).expect("storage");
    let hub = ExchangeHub::new();
    let state = ServerState::new(hub, storage);

    // Spawn server
    tokio::spawn(
        Server::builder()
            .add_service(LiveEventsServer::new(LiveEventsService {
                state: state.clone(),
            }))
            .add_service(RestProxyServer::new(RestProxyService {
                state: state.clone(),
            }))
            .add_service(StorageReadServer::new(StorageReadService {
                state: state.clone(),
            }))
            .add_service(HealthServer::new(HealthService { state }))
            .serve(addr),
    );

    // Small grace period for server to bind
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut client = HealthClient::connect(format!("http://{}", addr))
        .await
        .expect("connect");

    let resp = timeout(
        Duration::from_secs(5),
        client.status(tonic::Request::new(HealthRequest {})),
    )
    .await
    .expect("no timeout")
    .expect("rpc ok");

    let inner = resp.into_inner();
    assert_eq!(inner.connected_exchanges, 0);
    assert_eq!(inner.active_subscriptions, 0);
}

/// Subscribe to live events from Binance (requires real network).
#[tokio::test]
#[ignore]
async fn server_live_events_subscribe() {
    use futures_util::StreamExt as _;

    use digdigdig3::server::proto::{
        live_events_client::LiveEventsClient, SubscribeRequest,
    };

    // Assumes a dig3-server is already running on 127.0.0.1:18260
    let mut client = LiveEventsClient::connect("http://127.0.0.1:18260")
        .await
        .expect("connect to running dig3-server");

    let mut stream = client
        .subscribe(tonic::Request::new(SubscribeRequest {
            exchange: "binance".into(),
            account: "spot".into(),
            symbol: "btcusdt".into(),
            stream_kind: "ticker".into(),
        }))
        .await
        .expect("subscribe")
        .into_inner();

    let event = timeout(Duration::from_secs(10), stream.next())
        .await
        .expect("no timeout")
        .expect("stream not ended")
        .expect("rpc ok");

    assert!(!event.event_type.is_empty());
    assert!(!event.payload_json.is_empty());
}
