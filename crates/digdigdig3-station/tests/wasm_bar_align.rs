//! # wasm_bar_align — the non-OHLCV bar-aligned loader, in-browser.
//!
//! Proves `bar_align::load_bar_aligned` runs in a wasm32 browser context and
//! produces bar-grid-aligned series from a real connector + REST fetch.
//!
//! Uses **Lighter** as the data source: it sends `Access-Control-Allow-Origin: *`
//! on its public market endpoints (confirmed in `wasm_rest_parity.rs`), so the
//! browser dials it directly — no shared CORS proxy whose IP Binance et al.
//! rate-limit/ban. Covers a kline-family stream (mark price klines) and a
//! scalar state stream (funding rate, forward-filled).
//!
//! ## Run (Windows, username with a space → use 8.3 short paths to avoid the
//! wasm-bindgen-test-runner path-quoting bug):
//! ```sh
//! CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER='C:\Users\VAPC~1\AppData\Local\.wasm-pack\wasm-bindgen-<hash>\wasm-bindgen-test-runner.exe' \
//! CHROMEDRIVER='C:\Users\VAPC~1\.cache\dig2browser\drivers\msedgedriver\<ver>\msedgedriver.exe' \
//! WASM_BINDGEN_TEST_TIMEOUT=600 \
//!   cargo test --target wasm32-unknown-unknown -p digdigdig3-station --test wasm_bar_align
//! ```

#![cfg(target_arch = "wasm32")]

use std::sync::Arc;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3_station::bar_align::load_bar_aligned;
use digdigdig3_station::series::Kind;
use digdigdig3_station::BarAlignedSeries;

// Recent 2-day window ending "now" (browser clock); Lighter returns recent
// candles/funding so a relative window keeps data fresh regardless of run date.
fn window() -> (i64, i64) {
    let now = js_sys::Date::now() as i64;
    (now - 2 * 86_400_000, now)
}

fn times(series: &BarAlignedSeries) -> Vec<i64> {
    match series {
        BarAlignedSeries::Klines(v) => v.iter().map(|b| b.open_time).collect(),
        BarAlignedSeries::Scalar(v) => v.iter().map(|b| b.bar_open_time).collect(),
    }
}

async fn lighter_hub() -> Arc<ExchangeHub> {
    let hub = Arc::new(ExchangeHub::new());
    // Lighter sends ACAO:* on public market data — dial directly, no proxy.
    hub.connect_public(ExchangeId::Lighter, false)
        .await
        .expect("Lighter connect_public (direct, CORS *)");
    hub
}

#[wasm_bindgen_test]
async fn wasm_mark_price_klines() {
    let hub = lighter_hub().await;
    let iv = KlineInterval::new("1h");
    let (start, end) = window();
    let series = load_bar_aligned(
        &hub, ExchangeId::Lighter, AccountType::FuturesCross, "BTC",
        &Kind::MarkPriceKline(iv.clone()), &iv, start, end,
    )
    .await
    .expect("Lighter mark price klines (wasm, direct)");

    let t = times(&series);
    web_sys::console::log_1(&format!("wasm Lighter mark klines: {} bars", t.len()).into());
    assert!(!t.is_empty(), "expected ≥1 bar");
    assert!(t.windows(2).all(|w| w[1] > w[0]), "timestamps must strictly increase");
    assert!(t.iter().all(|x| x % 3_600_000 == 0), "bars must be 1h-grid-aligned");
}

#[wasm_bindgen_test]
async fn wasm_funding_rate_scalar() {
    let hub = lighter_hub().await;
    let iv = KlineInterval::new("1h");
    let (start, end) = window();
    let series = load_bar_aligned(
        &hub, ExchangeId::Lighter, AccountType::FuturesCross, "BTC",
        &Kind::FundingRate, &iv, start, end,
    )
    .await
    .expect("Lighter funding rate (wasm, direct)");

    let t = times(&series);
    web_sys::console::log_1(&format!("wasm Lighter funding: {} bars", t.len()).into());
    assert!(!t.is_empty(), "expected ≥1 funding bar");
    assert!(t.iter().all(|x| x % 3_600_000 == 0), "funding bars must be 1h-grid-aligned");
}
