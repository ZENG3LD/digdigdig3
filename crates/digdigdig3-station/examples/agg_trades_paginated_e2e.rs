//! Live e2e: verify backfill::agg_trades_paginated deepens history vs
//! agg_trades_recent against Binance BTCUSDT futures.
//!
//! Run:
//! ```bash
//! cd digdigdig3
//! cargo run --release --example agg_trades_paginated_e2e -p digdigdig3-station
//! ```

use std::sync::Arc;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::Binance, &[AccountType::FuturesCross], false).await?;

    let symbol = "BTCUSDT";
    let account = AccountType::FuturesCross;

    println!("=== Phase 1: agg_trades_recent (old behaviour, 1 page × 1000) ===");
    let recent = digdigdig3_station::backfill::agg_trades_recent(
        &hub,
        ExchangeId::Binance,
        account,
        symbol,
        1000,
    )
    .await;
    summarise("recent", &recent);

    for n_pages in [2usize, 5, 10] {
        println!("\n=== Phase 2: agg_trades_paginated (1000 × {n_pages} pages) ===");
        let paged = digdigdig3_station::backfill::agg_trades_paginated(
            &hub,
            ExchangeId::Binance,
            account,
            symbol,
            1000,
            n_pages,
        )
        .await;
        summarise(&format!("paged_{n_pages}"), &paged);
    }

    Ok(())
}

fn summarise(label: &str, agg: &[digdigdig3_station::data::AggTradePoint]) {
    if agg.is_empty() {
        println!("[{label}] EMPTY");
        return;
    }
    let oldest = agg.first().map(|a| a.ts_ms).unwrap_or(0);
    let newest = agg.last().map(|a| a.ts_ms).unwrap_or(0);
    let span_ms = newest - oldest;
    let span_secs = span_ms as f64 / 1000.0;
    let buys = agg.iter().filter(|a| a.side == 0).count();
    let sells = agg.iter().filter(|a| a.side == 1).count();
    let min_id = agg.iter().map(|a| a.agg_id).min().unwrap_or(0);
    let max_id = agg.iter().map(|a| a.agg_id).max().unwrap_or(0);
    let price_min = agg.iter().map(|a| a.price).fold(f64::INFINITY, f64::min);
    let price_max = agg.iter().map(|a| a.price).fold(f64::NEG_INFINITY, f64::max);
    println!(
        "[{label}] count={}  span={:.1}s  ts=[{}..{}]  agg_id=[{}..{}]  side=[buy={}/sell={}]  price=[{:.2}..{:.2}]",
        agg.len(), span_secs, oldest, newest, min_id, max_id, buys, sells, price_min, price_max
    );
}
