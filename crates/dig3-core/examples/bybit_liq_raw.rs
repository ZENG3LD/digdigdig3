//! Bybit liquidation raw capture — bypasses ALL digdigdig3 transport code.
//!
//! Connects directly via tokio-tungstenite to wss://stream.bybit.com/v5/public/linear,
//! subscribes to allLiquidation.* + publicTrade.BTCUSDT, reads every frame for 300s,
//! prints raw text with timestamps, and reports per-topic counts.
//!
//! Usage:
//!   cargo run --example bybit_liq_raw --release
//!   cargo run --example bybit_liq_raw --release -- --duration 60

use std::collections::HashMap;
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

fn ts_str() -> String {
    format!("[{}]", now_ms())
}

#[tokio::main]
async fn main() {
    // Parse optional --duration N argument
    let duration_secs: u64 = std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|w| {
            if w[0] == "--duration" {
                w[1].parse().ok()
            } else {
                None
            }
        })
        .unwrap_or(300);

    let url = "wss://stream.bybit.com/v5/public/linear";
    eprintln!("{} connecting to {}", ts_str(), url);

    let (ws_stream, _response) = match connect_async(url).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} CONNECT FAILED: {}", ts_str(), e);
            std::process::exit(1);
        }
    };

    eprintln!("{} connected OK", ts_str());
    let (mut write_half, mut read_half) = ws_stream.split();

    // Subscribe: allLiquidation for 5 high-volume symbols + publicTrade.BTCUSDT as baseline
    let sub = serde_json::json!({
        "op": "subscribe",
        "args": [
            "allLiquidation.BTCUSDT",
            "allLiquidation.ETHUSDT",
            "allLiquidation.SOLUSDT",
            "allLiquidation.DOGEUSDT",
            "allLiquidation.XRPUSDT",
            "publicTrade.BTCUSDT"
        ]
    });
    let sub_text = sub.to_string();
    eprintln!("{} sending subscribe: {}", ts_str(), sub_text);

    if let Err(e) = write_half.send(Message::Text(sub_text)).await {
        eprintln!("{} SUBSCRIBE SEND FAILED: {}", ts_str(), e);
        std::process::exit(1);
    }

    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut liq_frames: Vec<String> = Vec::new();
    let start = Instant::now();
    let deadline = Duration::from_secs(duration_secs);

    // Ping every 20s to keep connection alive
    let mut ping_ticker = tokio::time::interval(Duration::from_secs(20));
    ping_ticker.tick().await; // consume first immediate tick

    eprintln!("{} reading frames for {}s ...", ts_str(), duration_secs);

    loop {
        if start.elapsed() >= deadline {
            break;
        }

        tokio::select! {
            msg = read_half.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Extract topic prefix for counting
                        let topic = extract_topic_prefix(&text);
                        *counts.entry(topic.clone()).or_insert(0) += 1;

                        // Print every frame (stdout for logging)
                        println!("{} TOPIC={} RAW={}", ts_str(), topic, text);

                        // Keep first 3 liq frames as samples
                        if topic.starts_with("allLiquidation") && liq_frames.len() < 3 {
                            liq_frames.push(text.clone());
                        }
                    }
                    Some(Ok(Message::Binary(b))) => {
                        eprintln!("{} BINARY frame {} bytes", ts_str(), b.len());
                    }
                    Some(Ok(Message::Ping(_))) => {
                        eprintln!("{} WS Ping received (tungstenite handles pong)", ts_str());
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(f))) => {
                        eprintln!("{} CLOSE received: {:?}", ts_str(), f);
                        break;
                    }
                    Some(Ok(Message::Frame(_))) => {}
                    Some(Err(e)) => {
                        eprintln!("{} WS ERROR: {}", ts_str(), e);
                        break;
                    }
                    None => {
                        eprintln!("{} stream ended", ts_str());
                        break;
                    }
                }
            }
            _ = ping_ticker.tick() => {
                let ping = serde_json::json!({"op": "ping"}).to_string();
                eprintln!("{} sending ping", ts_str());
                if let Err(e) = write_half.send(Message::Text(ping)).await {
                    eprintln!("{} ping send failed: {}", ts_str(), e);
                    break;
                }
            }
        }
    }

    let elapsed = start.elapsed();

    eprintln!("\n=== RAW CAPTURE SUMMARY ===");
    eprintln!("duration: {:.1}s", elapsed.as_secs_f64());
    eprintln!("per-topic counts:");
    let mut sorted: Vec<(String, u64)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    for (topic, count) in &sorted {
        eprintln!("  {:45} {}", topic, count);
    }

    let liq_count: u64 = sorted.iter()
        .filter(|(t, _)| t.starts_with("allLiquidation"))
        .map(|(_, c)| c)
        .sum();
    let trade_count: u64 = sorted.iter()
        .filter(|(t, _)| t.starts_with("publicTrade"))
        .map(|(_, c)| c)
        .sum();

    eprintln!("\nallLiquidation frames total: {}", liq_count);
    eprintln!("publicTrade frames total:    {}", trade_count);

    if liq_frames.is_empty() {
        eprintln!("\nSAMPLE LIQ FRAMES: none");
    } else {
        eprintln!("\nSAMPLE LIQ FRAMES ({}):", liq_frames.len());
        for f in &liq_frames {
            eprintln!("  {}", f);
        }
    }

    if liq_count == 0 && trade_count > 0 {
        eprintln!("\nDIAGNOSIS: publicTrade flows ({} frames) but allLiquidation = 0.", trade_count);
        eprintln!("  → Bybit is NOT pushing allLiquidation frames on this endpoint.");
        eprintln!("  → Possible causes: channel throttled, symbol format wrong, or Bybit side issue.");
    } else if liq_count == 0 && trade_count == 0 {
        eprintln!("\nDIAGNOSIS: ZERO frames on both channels. Network/TLS/sub format broken.");
    } else if liq_count > 0 {
        eprintln!("\nDIAGNOSIS: allLiquidation frames DO arrive via raw connection.");
        eprintln!("  → Bug is inside our digdigdig3 transport/protocol pipeline.");
    }
}

/// Extract a topic key for counting from a raw JSON text frame.
/// Returns the topic value, or a meta label for control frames.
fn extract_topic_prefix(text: &str) -> String {
    // Try fast JSON parse
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(topic) = v.get("topic").and_then(|t| t.as_str()) {
            return topic.to_string();
        }
        if let Some(op) = v.get("op").and_then(|t| t.as_str()) {
            return format!("__op:{}", op);
        }
        if v.get("success").is_some() {
            return "__ack".to_string();
        }
    }
    "__unparsed".to_string()
}
