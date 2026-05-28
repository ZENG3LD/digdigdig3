//! Raw HTX index kline WS test — bypasses digdigdig3 transport entirely.
//! Connects directly to HTX USDT swap WS, subscribes to
//! market.BTC-USDT.index.1min, reads 10 frames or 30 seconds.
//!
//! Usage:
//!   cargo run --example htx_index_raw --release

use std::io::Read as IoRead;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use futures_util::{SinkExt, StreamExt};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn decode_gzip(bytes: &[u8]) -> String {
    let mut decoder = GzDecoder::new(bytes);
    let mut s = String::new();
    decoder.read_to_string(&mut s).unwrap_or(0);
    s
}

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // HTX index kline is on a separate WS endpoint: ws_index
    let url = "wss://api.hbdm.com/ws_index";
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

    // Try index kline via ws_index endpoint
    // HTX ws_index endpoint uses a different symbol format potentially
    let sub = serde_json::json!({
        "sub": "market.BTC-USDT.index.1min",
        "id": "id1"
    });

    let sub_spot_format = serde_json::json!({
        "sub": "market.BTC-USD.index.1min",
        "id": "id2"
    });
    let sub_text = sub.to_string();
    eprintln!("[{}] send subscribe: {}", now_ms(), sub_text);
    write_half.send(Message::Text(sub_text)).await.expect("send subscribe");

    // Also try the spot format to diagnose
    tokio::time::sleep(Duration::from_millis(500)).await;
    let sub_spot_text = sub_spot_format.to_string();
    eprintln!("[{}] send spot-format subscribe: {}", now_ms(), sub_spot_text);
    write_half.send(Message::Text(sub_spot_text)).await.expect("send sub2");

    let start = Instant::now();
    let deadline = Duration::from_secs(30);
    let mut frame_count = 0;

    eprintln!("[{}] waiting for frames...", now_ms());

    loop {
        if start.elapsed() >= deadline {
            eprintln!("[{}] 30s deadline, {} frames seen", now_ms(), frame_count);
            break;
        }

        let timeout = tokio::time::timeout(Duration::from_secs(35), read_half.next());
        match timeout.await {
            Ok(Some(Ok(Message::Text(text)))) => {
                frame_count += 1;
                eprintln!("[{}] TEXT #{}: {}", now_ms(), frame_count, &text[..text.len().min(200)]);
                if frame_count >= 5 { break; }
            }
            Ok(Some(Ok(Message::Binary(b)))) => {
                frame_count += 1;
                let decoded = decode_gzip(&b);
                eprintln!("[{}] BINARY #{}: {}", now_ms(), frame_count, &decoded[..decoded.len().min(300)]);
                // Handle pings
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&decoded) {
                    if let Some(ping) = v.get("ping").and_then(|p| p.as_i64()) {
                        let pong = serde_json::json!({"pong": ping}).to_string();
                        eprintln!("[{}] sending pong for ping={}", now_ms(), ping);
                        write_half.send(Message::Text(pong)).await.ok();
                        frame_count -= 1; // don't count ping as data
                    }
                }
                if frame_count >= 5 { break; }
            }
            Ok(Some(Ok(Message::Ping(p)))) => {
                write_half.send(Message::Pong(p)).await.ok();
            }
            Ok(Some(Ok(Message::Close(f)))) => {
                eprintln!("[{}] CLOSE: {:?}", now_ms(), f);
                break;
            }
            Ok(Some(Err(e))) => {
                eprintln!("[{}] ERROR: {}", now_ms(), e);
                break;
            }
            Ok(None) => {
                eprintln!("[{}] stream ended", now_ms());
                break;
            }
            Err(_) => {
                eprintln!("[{}] read timeout", now_ms());
                break;
            }
            _ => {}
        }
    }

    eprintln!("total data frames: {}", frame_count);
}
