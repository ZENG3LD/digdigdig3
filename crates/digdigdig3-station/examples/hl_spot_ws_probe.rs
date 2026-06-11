//! Live probe: HyperLiquid SPOT + Perp WS subscriptions via Station.
//!
//! Validates:
//!  1. SPOT Trade subscription for "HYPE/USDC" (display name via add_raw) —
//!     station's Part-B seam resolves to "@N" wire; first real trade event within 90s.
//!  2. SPOT Kline 1m subscription for "HYPE/USDC" — first candle event within 90s.
//!  3. Perp regression: BTC FuturesCross Trade subscription must still receive events.
//!
//! Run:
//!   cargo run -p digdigdig3-station --example hl_spot_ws_probe

use std::time::Duration;

use digdigdig3::core::websocket::KlineInterval;
use digdigdig3::{AccountType, ExchangeId};
use digdigdig3_station::{Station, SubscriptionSet, Stream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut pass = 0usize;
    let mut fail = 0usize;

    let timeout = Duration::from_secs(90);

    // Build Station — subscribe() lazily calls connect_public + connect_websocket.
    let station = Station::builder().build().await?;

    // ─── 1. SPOT Trade for "HYPE/USDC" ───────────────────────────────────────
    println!("[1] Subscribing SPOT Trade for HYPE/USDC (display name via add_raw)...");
    let set = SubscriptionSet::new().add_raw(
        ExchangeId::HyperLiquid,
        "HYPE/USDC",
        AccountType::Spot,
        [Stream::Trade],
    );

    let report = station.subscribe(set).await?;
    if !report.failed.is_empty() {
        println!("FAIL SPOT Trade subscribe failed: {:?}", report.failed);
        fail += 1;
    } else {
        let mut handle = report.handle;
        match tokio::time::timeout(timeout, handle.recv()).await {
            Ok(Some(ev)) => {
                println!(
                    "PASS SPOT Trade HYPE/USDC: event symbol=\"{}\"",
                    ev.symbol()
                );
                pass += 1;
            }
            Ok(None) => {
                println!("FAIL SPOT Trade HYPE/USDC: channel closed with no event");
                fail += 1;
            }
            Err(_) => {
                println!(
                    "FAIL SPOT Trade HYPE/USDC: timeout after {}s (wire subscribe frame may not have carried wire id)",
                    timeout.as_secs()
                );
                fail += 1;
            }
        }
    }

    // ─── 2. SPOT Kline 1m for "HYPE/USDC" ────────────────────────────────────
    println!("[2] Subscribing SPOT Kline 1m for HYPE/USDC...");
    let set2 = SubscriptionSet::new().add_raw(
        ExchangeId::HyperLiquid,
        "HYPE/USDC",
        AccountType::Spot,
        [Stream::Kline(KlineInterval::new("1m"))],
    );

    let report2 = station.subscribe(set2).await?;
    if !report2.failed.is_empty() {
        println!("FAIL SPOT Kline 1m subscribe failed: {:?}", report2.failed);
        fail += 1;
    } else {
        let mut handle2 = report2.handle;
        match tokio::time::timeout(timeout, handle2.recv()).await {
            Ok(Some(ev)) => {
                println!(
                    "PASS SPOT Kline 1m HYPE/USDC: event symbol=\"{}\"",
                    ev.symbol()
                );
                pass += 1;
            }
            Ok(None) => {
                println!("FAIL SPOT Kline 1m HYPE/USDC: channel closed");
                fail += 1;
            }
            Err(_) => {
                println!(
                    "FAIL SPOT Kline 1m HYPE/USDC: timeout after {}s",
                    timeout.as_secs()
                );
                fail += 1;
            }
        }
    }

    // ─── 3. Perp regression: BTC FuturesCross Trade ──────────────────────────
    println!("[3] Subscribing Perp Trade BTC FuturesCross (perp regression)...");
    let set3 = SubscriptionSet::new().add_raw(
        ExchangeId::HyperLiquid,
        "BTC",
        AccountType::FuturesCross,
        [Stream::Trade],
    );

    let report3 = station.subscribe(set3).await?;
    if !report3.failed.is_empty() {
        println!("FAIL Perp Trade BTC subscribe failed: {:?}", report3.failed);
        fail += 1;
    } else {
        let mut handle3 = report3.handle;
        match tokio::time::timeout(timeout, handle3.recv()).await {
            Ok(Some(ev)) => {
                println!("PASS Perp Trade BTC: event symbol=\"{}\"", ev.symbol());
                pass += 1;
            }
            Ok(None) => {
                println!("FAIL Perp Trade BTC: channel closed");
                fail += 1;
            }
            Err(_) => {
                println!(
                    "FAIL Perp Trade BTC: timeout after {}s",
                    timeout.as_secs()
                );
                fail += 1;
            }
        }
    }

    println!("\n--- SUMMARY: {pass} PASS / {fail} FAIL ---");
    if fail > 0 {
        Err(format!("{fail} probe(s) failed").into())
    } else {
        Ok(())
    }
}
