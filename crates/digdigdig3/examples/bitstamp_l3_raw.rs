//! Raw WS probe: subscribe live_orders_btcusd, print full first frame.
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("wss://ws.bitstamp.net").await.expect("connect");
    let (mut writer, mut reader) = ws_stream.split();
    let sub = r#"{"event":"bts:subscribe","data":{"channel":"live_orders_btcusd"}}"#;
    writer.send(Message::Text(sub.to_string())).await.expect("send");
    eprintln!("subscribed to live_orders_btcusd");

    let mut count = 0usize;
    let _ = timeout(Duration::from_secs(15), async {
        while let Some(Ok(Message::Text(text))) = reader.next().await {
            let v: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let event = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
            if event == "order_created" || event == "order_changed" || event == "order_deleted" {
                count += 1;
                // Print first full frame
                if count == 1 {
                    eprintln!("FULL FRAME: {}", text);
                    let data = v.get("data");
                    if let Some(d) = data {
                        eprintln!("price field type: {:?}", d.get("price"));
                        eprintln!("amount field type: {:?}", d.get("amount"));
                        eprintln!("order_type field: {:?}", d.get("order_type"));
                        eprintln!("id field: {:?}", d.get("id"));
                        eprintln!("microtimestamp field: {:?}", d.get("microtimestamp"));
                    }
                }
                if count >= 3 { break; }
            }
        }
    }).await;
    eprintln!("done. total_l3_frames={count}");
}
