//! Wasm smoke test — opens a real WebSocket to Binance from a headless browser.
//!
//! Run with:
//!   wasm-pack test --headless --firefox crates/digdigdig3 --test wasm_smoke
//!
//! Requires wasm-pack + Firefox + geckodriver installed on the test host.
//!
//! These tests are gated to `target_arch = "wasm32"` so they are silently
//! skipped on native builds.

#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Verify that a plain wss:// handshake to Binance succeeds from the browser.
///
/// BTCUSDT trade stream fires ~10-50 frames/second — the connection itself is
/// the signal we care about here.
#[wasm_bindgen_test]
async fn binance_ws_handshake_succeeds() {
    use digdigdig3::core::rt::default_runtime;

    let rt = default_runtime();
    let conn = rt
        .connect_ws(
            "wss://stream.binance.com:9443/ws/btcusdt@trade",
            Duration::from_secs(10),
        )
        .await
        .expect("WebSocket handshake to Binance must succeed from the browser");

    drop(conn);
}

/// Verify that at least one data frame arrives within 5 seconds.
///
/// BTCUSDT is among the most active pairs on Binance — a trade frame should
/// arrive well within 1 second under normal conditions.
#[wasm_bindgen_test]
async fn binance_ws_first_frame() {
    use digdigdig3::core::rt::{default_runtime, WsFrame};

    let rt = default_runtime();
    let mut conn = rt
        .connect_ws(
            "wss://stream.binance.com:9443/ws/btcusdt@trade",
            Duration::from_secs(10),
        )
        .await
        .expect("connect");

    // Poll up to 50 frames; BTCUSDT fires fast enough that we expect a hit
    // in the first few iterations.
    let mut got_frame = false;
    for _ in 0..50 {
        match conn.next_frame().await {
            Some(Ok(WsFrame::Text(_))) | Some(Ok(WsFrame::Binary(_))) => {
                got_frame = true;
                break;
            }
            Some(Ok(_)) => continue, // Ping/Pong/Close — keep polling
            Some(Err(e)) => panic!("WS error reading frame: {:?}", e),
            None => break, // Connection closed
        }
    }

    assert!(
        got_frame,
        "expected at least 1 data frame from Binance BTCUSDT trade stream within 50 polls"
    );
}
