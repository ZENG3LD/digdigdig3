//! server_client_demo — demonstrates the dig3-server gRPC IPC pattern.
//!
//! This example requires a running dig3-server:
//!   cargo run --bin dig3-server --features server -- --grpc-addr 127.0.0.1:18260
//!
//! Then in another terminal:
//!   cargo run --example server_client_demo --features server
//!
//! It connects to the gRPC server, calls Health::Status, subscribes to the
//! BTC/USDT ticker stream, prints 5 events, then disconnects.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "server")]
    {
        run().await?;
    }
    #[cfg(not(feature = "server"))]
    {
        eprintln!(
            "server_client_demo requires `server` feature:\n  \
             cargo run --example server_client_demo --features server"
        );
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(feature = "server")]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;

    use futures_util::StreamExt as _;
    use tokio::time::timeout;

    use digdigdig3::server::proto::{
        health_client::HealthClient,
        live_events_client::LiveEventsClient,
        HealthRequest, SubscribeRequest,
    };

    let addr = "http://127.0.0.1:18260";

    // ── Health check ──────────────────────────────────────────────────────────
    let mut health = HealthClient::connect(addr).await?;
    let status = health
        .status(tonic::Request::new(HealthRequest {}))
        .await?
        .into_inner();

    println!("=== dig3-server health ===");
    println!("  uptime_secs:          {}", status.uptime_secs);
    println!("  connected_exchanges:  {}", status.connected_exchanges);
    println!("  active_subscriptions: {}", status.active_subscriptions);
    println!();

    // ── Live events — subscribe to BTC/USDT ticker ───────────────────────────
    let mut live = LiveEventsClient::connect(addr).await?;
    let mut stream = live
        .subscribe(tonic::Request::new(SubscribeRequest {
            exchange: "binance".into(),
            account: "spot".into(),
            symbol: "btcusdt".into(),
            stream_kind: "ticker".into(),
        }))
        .await?
        .into_inner();

    println!("=== Waiting for up to 5 live BTC/USDT ticker events ===");
    let mut count = 0usize;

    while count < 5 {
        match timeout(Duration::from_secs(15), stream.next()).await {
            Ok(Some(Ok(ev))) => {
                let payload = String::from_utf8_lossy(&ev.payload_json);
                println!(
                    "[{}] type={} ts={} payload={}",
                    count + 1,
                    ev.event_type,
                    ev.timestamp_ms,
                    &payload[..payload.len().min(120)]
                );
                count += 1;
            }
            Ok(Some(Err(e))) => {
                eprintln!("stream error: {}", e);
                break;
            }
            Ok(None) => {
                println!("stream ended");
                break;
            }
            Err(_) => {
                println!("timeout waiting for event");
                break;
            }
        }
    }

    println!("\nReceived {} events. Done.", count);
    Ok(())
}
