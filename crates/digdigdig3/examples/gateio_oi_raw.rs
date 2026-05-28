//! Raw GateIO contract_stats WS test — bypasses digdigdig3 transport entirely.
//! Connects directly to wss://fx-ws.gateio.ws/v4/ws/usdt, subscribes to
//! futures.contract_stats for BTC_USDT, reads 30 frames or 25 seconds.
//!
//! Usage:
//!   cargo run --example gateio_oi_raw --release

use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn now_sec() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tokio::main]
async fn main() {
    // Install rustls crypto provider (required when using tokio-tungstenite with rustls)
    let _ = rustls::crypto::ring::default_provider().install_default();

    let url = "wss://fx-ws.gateio.ws/v4/ws/usdt";
    eprintln!("[{}] connecting to {}", now_ms(), url);

    let (ws_stream, _response) = match connect_async(url).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("CONNECT FAILED: {}", e);
            std::process::exit(1);
        }
    };
    eprintln!("[{}] connected OK", now_ms());

    let (mut write_half, mut read_half) = ws_stream.split();

    // Send subscribe for contract_stats with 1m interval (test both 1min and 10s)
    // GateIO docs say valid intervals are "10" (10s) or "1m"
    let sub = serde_json::json!({
        "time": now_sec(),
        "channel": "futures.contract_stats",
        "event": "subscribe",
        "payload": ["BTC_USDT", "1m"]
    });
    let sub_text = sub.to_string();
    eprintln!("[{}] send subscribe: {}", now_ms(), sub_text);
    write_half.send(Message::Text(sub_text)).await.expect("send subscribe");

    let start = Instant::now();
    let deadline = Duration::from_secs(75);
    let mut frame_count = 0;

    // Also send GateIO ping every 20s
    let mut ping_ticker = tokio::time::interval(Duration::from_secs(20));
    ping_ticker.tick().await; // consume first

    eprintln!("[{}] waiting for frames...", now_ms());

    loop {
        if start.elapsed() >= deadline {
            eprintln!("[{}] 25s deadline reached, {} frames seen", now_ms(), frame_count);
            break;
        }

        tokio::select! {
            msg = read_half.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        frame_count += 1;
                        eprintln!("[{}] FRAME #{}: {}", now_ms(), frame_count, text);
                        if frame_count >= 5 { break; }
                    }
                    Some(Ok(Message::Binary(b))) => {
                        eprintln!("[{}] BINARY {} bytes", now_ms(), b.len());
                    }
                    Some(Ok(Message::Ping(p))) => {
                        eprintln!("[{}] WS Ping", now_ms());
                        write_half.send(Message::Pong(p)).await.ok();
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(f))) => {
                        eprintln!("[{}] CLOSE: {:?}", now_ms(), f);
                        break;
                    }
                    Some(Ok(Message::Frame(_))) => {}
                    Some(Err(e)) => {
                        eprintln!("[{}] ERROR: {}", now_ms(), e);
                        break;
                    }
                    None => {
                        eprintln!("[{}] stream ended", now_ms());
                        break;
                    }
                }
            }
            _ = ping_ticker.tick() => {
                let ping = serde_json::json!({
                    "time": now_sec(),
                    "channel": "futures.ping"
                }).to_string();
                eprintln!("[{}] send ping", now_ms());
                write_half.send(Message::Text(ping)).await.ok();
            }
        }
    }

    eprintln!("total frames: {}", frame_count);
}
