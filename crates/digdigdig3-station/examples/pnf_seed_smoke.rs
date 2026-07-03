//! Live e2e: PnF deep seed via `add_with_warm` — mirrors the mlc bridge's
//! `spawn_pnf_sub` flow exactly (subscribe with warm override → ring peek by
//! the report.ok key → drain handle events). Discriminates between
//! "kline-approx seed broken in shared logic" vs "wasm-transport-only".
//!
//! Run:
//! ```bash
//! cd digdigdig3
//! cargo run --release --example pnf_seed_smoke -p digdigdig3-station
//! ```

use std::collections::BTreeSet;
use std::time::Duration;

use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3_station::data::PnfColumnPoint;
use digdigdig3_station::{Event, Station, SubscriptionSet, Stream};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let station = Station::builder().warm_start(5000).build().await?;

    // Mirror the UI defaults: box_size 10.0 (×1e8 wire units), reversal 3.
    let box_units = (10.0f64 * 1e8) as u64;
    // wasm-depth warm override (10_000) — the exact value from the repro.
    let set = SubscriptionSet::new().add_with_warm(
        ExchangeId::Binance,
        "BTCUSDT",
        AccountType::Spot,
        [Stream::PnfBar(box_units, 3)],
        10_000,
    );

    let t0 = std::time::Instant::now();
    let mut report = station.subscribe(set).await?;
    println!(
        "subscribe returned in {:?}; ok={} failed={}",
        t0.elapsed(),
        report.ok.len(),
        report.failed.len()
    );
    for f in &report.failed {
        println!("FAILED: {} {:?} {:?}: {}", f.symbol, f.exchange, f.stream, f.error);
    }

    // Bridge-mirror ring peek (spawn_pnf_sub does exactly this, right after
    // subscribe returns).
    peek(&station, &report.ok, "PEEK#1 (right after subscribe)").await;

    // Drain handle events for 20s.
    let mut n_events = 0usize;
    let mut distinct: BTreeSet<u64> = BTreeSet::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        let ev = tokio::select! {
            _ = tokio::time::sleep_until(deadline) => break,
            ev = report.handle.recv() => ev,
        };
        match ev {
            Some(Event::PnfBar { point, .. }) => {
                n_events += 1;
                distinct.insert(point.column_id);
            }
            Some(_) => {}
            None => {
                println!("handle closed early");
                break;
            }
        }
    }
    println!("HANDLE after 20s: pnf events={n_events} distinct columns={}", distinct.len());

    peek(&station, &report.ok, "PEEK#2 (after 20s)").await;
    Ok(())
}

async fn peek(station: &Station, ok: &[digdigdig3_station::SeriesKey], label: &str) {
    let Some(key) = ok.first() else {
        println!("{label}: no ok key");
        return;
    };
    match station.series::<PnfColumnPoint>(key) {
        Some(s) => {
            let pts = s.read().await.snapshot();
            println!("{label}: ring n={}", pts.len());
            if let (Some(f), Some(l)) = (pts.first(), pts.last()) {
                let span_h = (l.open_time - f.open_time) as f64 / 3.6e6;
                let bmin = pts.iter().map(|p| p.bottom).fold(f64::INFINITY, f64::min);
                let tmax = pts.iter().map(|p| p.top).fold(f64::NEG_INFINITY, f64::max);
                println!(
                    "  first id={} ts={}  last id={} ts={}  span={:.1}h  price=[{:.0}..{:.0}]",
                    f.column_id, f.open_time, l.column_id, l.open_time, span_h, bmin, tmax
                );
            }
        }
        None => println!("{label}: station.series() = NONE — key mismatch / not registered (key={key:?})"),
    }
}
