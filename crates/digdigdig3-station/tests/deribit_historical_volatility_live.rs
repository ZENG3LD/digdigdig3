#![cfg(not(target_arch = "wasm32"))]
//! Live integration test: `Kind::HistoricalVolatility` bar-aligned loader
//! against Deribit's PUBLIC `public/get_historical_volatility?currency=BTC`
//! endpoint (no API key needed).
//!
//! Loads `BTC-PERPETUAL` on H1 over the trailing 30 days and asserts at
//! least one non-NaN bar. Gated with `--ignored` (hits the live network).
//! Run:
//!   cargo test --release -p digdigdig3-station --test deribit_historical_volatility_live -- --ignored --nocapture

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3_station::bar_align::load_bar_aligned;
use digdigdig3_station::series::Kind;
use digdigdig3_station::BarAlignedSeries;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_millis() as i64
}

#[tokio::test]
#[ignore]
async fn deribit_historical_volatility_btc_perpetual_h1_30d() {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::Deribit, &[AccountType::FuturesCross], false)
        .await
        .expect("connect Deribit");

    let interval = KlineInterval::new("1h");
    let end = now_ms();
    let start = end - 30 * 86_400_000; // trailing 30 days

    let series = load_bar_aligned(
        &hub,
        ExchangeId::Deribit,
        AccountType::FuturesCross,
        "BTC-PERPETUAL",
        &Kind::HistoricalVolatility,
        &interval,
        start,
        end,
    )
    .await
    .expect("load_bar_aligned(HistoricalVolatility) against live Deribit");

    let bars = match series {
        BarAlignedSeries::Scalar(bars) => bars,
        BarAlignedSeries::Klines(_) => panic!("HistoricalVolatility must yield Scalar bars, got Klines"),
    };

    let non_nan: Vec<_> = bars.iter().filter(|b| !b.value.is_nan()).collect();
    println!(
        "deribit HistoricalVolatility BTC-PERPETUAL 1h/30d: {} bars total, {} non-NaN",
        bars.len(),
        non_nan.len()
    );
    if let (Some(first), Some(last)) = (non_nan.first(), non_nan.last()) {
        println!(
            "first bar_open_time={} value={}, last bar_open_time={} value={}",
            first.bar_open_time, first.value, last.bar_open_time, last.value
        );
    }

    assert!(
        !non_nan.is_empty(),
        "expected >= 1 non-NaN HistoricalVolatility bar from live Deribit, got 0 of {}",
        bars.len()
    );
}
