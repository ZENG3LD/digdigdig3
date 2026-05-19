//! Phase ι.1 — 60-second WS event-rate catcher, top-7 exchanges × (Trade + Ticker).
//!
//! Asserts that at least 70% of (exchange, stream) pairs have non-zero event rate.
//! Run with: cargo test --test live_ws_event_rates -- --nocapture --ignored

#[path = "common/mod.rs"]
mod common;

use common::{run_jobs, JobOutcome};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, StreamType, Symbol, SubscriptionRequest};

// (ExchangeId, AccountType, raw_symbol, stream_types)
// Ticker + Trade for each exchange, using exchange-native symbols.
const TARGETS: &[(ExchangeId, AccountType, &str, &[StreamType])] = &[
    (
        ExchangeId::Binance,
        AccountType::Spot,
        "BTCUSDT",
        &[StreamType::Trade, StreamType::Ticker],
    ),
    (
        ExchangeId::Bybit,
        AccountType::Spot,
        "BTCUSDT",
        &[StreamType::Trade, StreamType::Ticker],
    ),
    (
        ExchangeId::OKX,
        AccountType::Spot,
        "BTC-USDT",
        &[StreamType::Trade, StreamType::Ticker],
    ),
    (
        ExchangeId::Bitget,
        AccountType::Spot,
        "BTCUSDT",
        &[StreamType::Trade, StreamType::Ticker],
    ),
    (
        ExchangeId::KuCoin,
        AccountType::Spot,
        "BTC-USDT",
        &[StreamType::Trade, StreamType::Ticker],
    ),
    (
        ExchangeId::GateIO,
        AccountType::Spot,
        "BTC_USDT",
        &[StreamType::Trade, StreamType::Ticker],
    ),
    (
        ExchangeId::MEXC,
        AccountType::Spot,
        "BTCUSDT",
        &[StreamType::Trade, StreamType::Ticker],
    ),
];

const COLLECT_DURATION: Duration = Duration::from_secs(60);

#[tokio::test]
#[ignore] // live API — run with --ignored
async fn ws_event_rates_top_7() {
    // Build one label per (exchange × stream_type) pair.
    let labels: Vec<String> = TARGETS
        .iter()
        .flat_map(|(id, _, _, stream_types)| {
            stream_types
                .iter()
                .map(move |st| format!("{id:?}:{st:?}"))
        })
        .collect();

    let results = run_jobs(
        labels,
        COLLECT_DURATION + Duration::from_secs(15),
        |_job_id, label| async move {
            // Parse label back to (ExchangeId, AccountType, symbol, StreamType).
            let mut parts = label.splitn(2, ':');
            let id_str = parts.next().ok_or("bad label: missing exchange")?;
            let st_str = parts.next().ok_or("bad label: missing stream_type")?;

            let (id, acct, raw_sym, _) = TARGETS
                .iter()
                .find(|(id, _, _, _)| format!("{id:?}") == id_str)
                .ok_or_else(|| format!("no target for exchange {id_str}"))?;

            let stream_type = match st_str {
                "Trade" => StreamType::Trade,
                "Ticker" => StreamType::Ticker,
                other => return Err(format!("unknown stream_type {other}")),
            };

            let hub = ExchangeHub::new();
            hub.connect_full(*id, &[*acct], false)
                .await
                .map_err(|e| e.to_string())?;

            let ws = hub
                .ws(*id, *acct)
                .ok_or_else(|| format!("{id:?}: no WS connector after connect_full"))?;

            ws.connect(*acct)
                .await
                .map_err(|e| format!("ws.connect: {e}"))?;

            let sym = Symbol::with_raw("", "", raw_sym.to_string());
            let req = match stream_type {
                StreamType::Ticker => SubscriptionRequest::ticker_for(sym, *acct),
                StreamType::Trade => SubscriptionRequest::trade_for(sym, *acct),
                other => SubscriptionRequest {
                    symbol: Symbol::with_raw("", "", raw_sym.to_string()),
                    stream_type: other,
                    account_type: *acct,
                    depth: None,
                    update_speed_ms: None,
                },
            };
            ws.subscribe(req)
                .await
                .map_err(|e| format!("subscribe: {e}"))?;

            let mut stream = ws.event_stream();
            let mut count = 0u64;
            let deadline = tokio::time::Instant::now() + COLLECT_DURATION;

            loop {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                if remaining.is_zero() {
                    break;
                }
                match timeout(remaining.min(Duration::from_millis(500)), stream.next()).await {
                    Ok(Some(Ok(_))) => count += 1,
                    Ok(Some(Err(_))) | Ok(None) => break,
                    Err(_) => {
                        // poll timeout — check deadline again
                    }
                }
            }

            Ok(count)
        },
    )
    .await;

    println!("\n=== ws_event_rates_top_7 (60s window) ===");
    println!("{:<30} {:>6} {:>14} {}", "label", "job#", "events/min", "status");
    println!("{}", "-".repeat(60));

    let mut totals: HashMap<String, u64> = HashMap::new();
    for r in &results {
        match &r.outcome {
            JobOutcome::Ok(n) => {
                let per_min = n * 60 / COLLECT_DURATION.as_secs();
                let status = if *n == 0 { "SILENT" } else { "OK" };
                println!(
                    "{:<30} {:>6} {:>14} {}",
                    r.label, r.job_id.0, per_min, status
                );
                totals.insert(r.label.clone(), per_min);
            }
            JobOutcome::TimedOut => {
                println!("{:<30} {:>6} {:>14}", r.label, r.job_id.0, "TIMEOUT");
            }
            JobOutcome::Failed(e) => {
                println!("{:<30} {:>6} FAIL: {e}", r.label, r.job_id.0);
            }
        }
    }

    let total = results.len();
    let nonzero = totals.values().filter(|&&v| v > 0).count();

    assert!(
        total > 0,
        "No results at all — check connectivity or ExchangeHub setup"
    );
    assert!(
        nonzero * 100 / total >= 70,
        "Only {nonzero}/{total} streams flowing — at least 70% must have non-zero event rate. \
         Check WS subscription logic for silent exchanges."
    );

    println!("\n{nonzero}/{total} streams flowing (>= 70% required) — PASS");
}
