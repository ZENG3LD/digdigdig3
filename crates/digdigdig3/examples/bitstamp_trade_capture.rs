//! # bitstamp_trade_capture — raw WS frame debug harness for Bitstamp live_trades_btcusd.
//!
//! Connects directly to wss://ws.bitstamp.net, sends subscribe, dumps every
//! raw frame to stdout for 120 seconds, then prints a summary.
//!
//! Usage:
//! ```
//! cargo run --example bitstamp_trade_capture --release
//! ```

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let url = "wss://ws.bitstamp.net";
    eprintln!("[bitstamp_trade_capture] connecting to {}", url);

    let (ws_stream, _) = connect_async(url).await.expect("WS connect failed");
    eprintln!("[bitstamp_trade_capture] connected");

    let (mut writer, mut reader) = ws_stream.split();

    // Send subscribe for live_trades_btcusd
    let sub = r#"{"event":"bts:subscribe","data":{"channel":"live_trades_btcusd"}}"#;
    eprintln!("[bitstamp_trade_capture] sending: {}", sub);
    writer
        .send(Message::Text(sub.to_string()))
        .await
        .expect("send failed");

    let mut frame_count = 0usize;
    let mut trade_count = 0usize;
    let mut other_count = 0usize;
    let capture_secs = 120u64;

    eprintln!(
        "[bitstamp_trade_capture] capturing {} seconds...",
        capture_secs
    );

    let result = timeout(Duration::from_secs(capture_secs), async {
        while let Some(msg) = reader.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    frame_count += 1;
                    // Parse to inspect event type
                    let parsed: serde_json::Value =
                        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
                    let event = parsed
                        .get("event")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let channel = parsed
                        .get("channel")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");

                    if event == "trade" {
                        trade_count += 1;
                        // Print first 5 trade frames in full
                        if trade_count <= 5 {
                            eprintln!(
                                "[TRADE frame #{}] event={} channel={} raw={}",
                                trade_count, event, channel, text
                            );
                        }
                        // For every trade, also inspect the data field to see if
                        // it's a string or object (Pusher double-encode check)
                        let data_field = parsed.get("data");
                        if trade_count == 1 {
                            eprintln!(
                                "[TRADE data field type] is_string={} is_object={} raw_data={:?}",
                                data_field.map(|v| v.is_string()).unwrap_or(false),
                                data_field.map(|v| v.is_object()).unwrap_or(false),
                                data_field
                            );
                        }
                    } else {
                        other_count += 1;
                        eprintln!(
                            "[OTHER frame #{}] event={} channel={} raw={}",
                            other_count, event, channel, &text[..text.len().min(300)]
                        );
                    }
                }
                Ok(Message::Ping(d)) => {
                    eprintln!("[bitstamp_trade_capture] recv Ping, sending Pong");
                    let _ = writer.send(Message::Pong(d)).await;
                }
                Ok(Message::Close(_)) => {
                    eprintln!("[bitstamp_trade_capture] server closed connection");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[bitstamp_trade_capture] WS error: {}", e);
                    break;
                }
            }
        }
    })
    .await;

    if result.is_err() {
        eprintln!(
            "[bitstamp_trade_capture] {} second window elapsed",
            capture_secs
        );
    }

    println!("=== SUMMARY ===");
    println!("total_frames:  {}", frame_count);
    println!("trade_frames:  {}", trade_count);
    println!("other_frames:  {}", other_count);
    if trade_count == 0 {
        println!("DIAGNOSIS: zero trade frames — check subscribe ack and channel name");
    } else {
        println!("DIAGNOSIS: trades flowing OK");
    }
}
