//! Live e2e matrix: cold-start seed depth for EVERY derived chart kind,
//! measured exactly the way the mlc bridge sees it — `add_with_warm(10_000)`
//! (wasm depth) then an immediate `Station::series::<T>()` ring peek (valid
//! since the seed_done gate landed: subscribe returns only after seeding).
//!
//! Run:
//! ```bash
//! cd digdigdig3
//! cargo run --release --example seed_matrix_smoke -p digdigdig3-station
//! ```

use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3_station::data::{
    BarPoint, FootprintPoint, KagiSegmentPoint, PnfColumnPoint, RenkoBrickPoint, ScalarBarPoint,
    ThreeLineBreakLinePoint, TpoSessionPoint,
};
use digdigdig3_station::series::DataPoint;
use digdigdig3_station::{Station, Stream, SubscriptionSet, TpoSource};

const WARM_N: usize = 10_000; // wasm depth — the exact repro value

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kinds: Vec<(&str, Stream)> = vec![
        ("range_bar(25)", Stream::RangeBar((25.0f64 * 1e8) as u64)),
        ("tick_bar(500)", Stream::TickBar(500)),
        ("volume_bar(10)", Stream::VolumeBar((10.0f64 * 1e8) as u64)),
        ("dollar_bar(500k)", Stream::DollarBar { dollar_threshold: 500_000 }),
        ("tick_imbalance", Stream::TickImbalanceBar { alpha_x100: 95, min_ticks: 100 }),
        ("volume_imbalance", Stream::VolumeImbalanceBar { alpha_x100: 95, min_ticks: 100 }),
        ("run_bar", Stream::RunBar { alpha_x100: 95, min_ticks: 100 }),
        ("footprint(1m)", Stream::Footprint(KlineInterval::new("1m"))),
        ("cvd_line", Stream::CvdLine),
        ("tpo(30m,kline1m)", Stream::TpoProfile(30, TpoSource::Kline1m)),
        ("renko(10,3)", Stream::RenkoBar((10.0f64 * 1e8) as u64, 3)),
        ("pnf(10,3)", Stream::PnfBar((10.0f64 * 1e8) as u64, 3)),
        ("kagi(50)", Stream::KagiBar((50.0f64 * 1e8) as u64)),
        ("3lb", Stream::ThreeLineBreak { lines_back: 3 }),
    ];

    println!("kind | subscribe_ms | ring_n | span_h | detail");
    println!("-----|--------------|--------|--------|-------");

    for (label, stream) in kinds {
        // Fresh station per kind — clean first-spawn path every time (the
        // exact path a fresh chart-open takes), no cross-kind upstream reuse.
        let station = Station::builder().warm_start(5000).build().await?;
        let set = SubscriptionSet::new().add_with_warm(
            ExchangeId::Binance,
            "BTCUSDT",
            AccountType::Spot,
            [stream.clone()],
            WARM_N,
        );
        let t0 = std::time::Instant::now();
        let report = match station.subscribe(set).await {
            Ok(r) => r,
            Err(e) => {
                println!("{label} | SUBSCRIBE ERR: {e}");
                continue;
            }
        };
        let ms = t0.elapsed().as_millis();
        if let Some(f) = report.failed.first() {
            println!("{label} | {ms} | FAILED: {}", f.error);
            continue;
        }
        let Some(key) = report.ok.first() else {
            println!("{label} | {ms} | no ok key");
            continue;
        };

        // Typed ring peek per output kind (mirror of each spawn_*_sub).
        let (n, span_h, detail) = match &stream {
            Stream::RangeBar(_)
            | Stream::TickBar(_)
            | Stream::VolumeBar(_)
            | Stream::DollarBar { .. }
            | Stream::TickImbalanceBar { .. }
            | Stream::VolumeImbalanceBar { .. }
            | Stream::RunBar { .. } => peek::<BarPoint>(&station, key, |p| p.open_time).await,
            Stream::Footprint(_) => peek::<FootprintPoint>(&station, key, |p| p.open_time).await,
            Stream::CvdLine => peek::<ScalarBarPoint>(&station, key, |p| p.ts_ms).await,
            Stream::TpoProfile(_, _) => {
                peek::<TpoSessionPoint>(&station, key, |p| p.open_time).await
            }
            Stream::RenkoBar(_, _) => peek::<RenkoBrickPoint>(&station, key, |p| p.open_time).await,
            Stream::PnfBar(_, _) => peek::<PnfColumnPoint>(&station, key, |p| p.open_time).await,
            Stream::KagiBar(_) => peek::<KagiSegmentPoint>(&station, key, |p| p.open_time).await,
            Stream::ThreeLineBreak { .. } => {
                peek::<ThreeLineBreakLinePoint>(&station, key, |p| p.ts_open).await
            }
            _ => (usize::MAX, 0.0, "unhandled".to_string()),
        };
        println!("{label} | {ms} | {n} | {span_h:.1} | {detail}");
    }
    Ok(())
}

async fn peek<T: DataPoint>(
    station: &Station,
    key: &digdigdig3_station::SeriesKey,
    ts_of: impl Fn(&T) -> i64,
) -> (usize, f64, String) {
    match station.series::<T>(key) {
        Some(s) => {
            let pts = s.read().await.snapshot();
            let n = pts.len();
            if let (Some(f), Some(l)) = (pts.first(), pts.last()) {
                let span_h = (ts_of(l) - ts_of(f)) as f64 / 3.6e6;
                (n, span_h, String::new())
            } else {
                (0, 0.0, "empty ring".to_string())
            }
        }
        None => (0, 0.0, "series()=NONE (not registered / key mismatch)".to_string()),
    }
}
