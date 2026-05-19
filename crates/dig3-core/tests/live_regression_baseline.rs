//! Phase ι.3 — Baseline regression detector.
//!
//! Collects 60s WS event rates for top-7 exchanges × (Trade + Ticker)
//! and asserts each stream meets at least 50% of the baseline in
//! `data/expected_event_rates.json`.
//!
//! Run with: cargo test --test live_regression_baseline -- --nocapture --ignored

#[path = "common/mod.rs"]
mod common;

use common::{run_jobs, JobOutcome};
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

use digdigdig3_core::connector_manager::ExchangeHub;
use digdigdig3_core::core::types::{AccountType, ExchangeId, StreamType, Symbol, SubscriptionRequest};

const BASELINE_JSON: &str = include_str!("../../../data/expected_event_rates.json");

/// 50% tolerance: accept if actual >= baseline * 0.5
const TOLERANCE: f64 = 0.5;
const COLLECT_DURATION: Duration = Duration::from_secs(60);

#[derive(Deserialize)]
struct StreamBaseline {
    min_events_per_minute: u64,
    captured_on: String,
}

// Baseline shape: exchange → account_type → stream_kind → baseline
type Baseline = HashMap<String, HashMap<String, HashMap<String, StreamBaseline>>>;

// (ExchangeId, AccountType, raw_symbol, stream_types)
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

#[tokio::test]
#[ignore] // live API — run with --ignored
async fn ws_event_rates_meet_baseline() {
    let baseline: Baseline =
        serde_json::from_str(BASELINE_JSON).expect("data/expected_event_rates.json is malformed");

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
                    Err(_) => {}
                }
            }

            Ok(count)
        },
    )
    .await;

    println!("\n=== ws_event_rates_meet_baseline (tolerance {:.0}%) ===", TOLERANCE * 100.0);
    println!(
        "{:<30} {:>14} {:>14} {:>14} {}",
        "label", "actual/min", "baseline/min", "floor/min", "status"
    );
    println!("{}", "-".repeat(85));

    let mut regressions: Vec<String> = Vec::new();

    for r in &results {
        // Parse label back to exchange / account_type / stream_type strings.
        let mut parts = r.label.splitn(2, ':');
        let id_str = parts.next().unwrap_or("");
        let st_str = parts.next().unwrap_or("");
        let acct_str = "Spot"; // all targets are Spot

        let baseline_entry = baseline
            .get(id_str)
            .and_then(|a| a.get(acct_str))
            .and_then(|s| s.get(st_str));

        match &r.outcome {
            JobOutcome::Ok(count) => {
                let actual_per_min = count * 60 / COLLECT_DURATION.as_secs();
                if let Some(b) = baseline_entry {
                    let floor = (b.min_events_per_minute as f64 * TOLERANCE) as u64;
                    let status = if actual_per_min >= floor { "OK" } else { "REGRESSION" };
                    println!(
                        "{:<30} #{:>3} {:>14} {:>14} {:>14} {} (baseline from {})",
                        r.label, r.job_id.0, actual_per_min, b.min_events_per_minute, floor,
                        status, b.captured_on
                    );
                    if actual_per_min < floor {
                        regressions.push(format!(
                            "{}: actual={}/min < floor={}/min (baseline={}/min, tolerance={:.0}%)",
                            r.label,
                            actual_per_min,
                            floor,
                            b.min_events_per_minute,
                            TOLERANCE * 100.0,
                        ));
                    }
                } else {
                    println!(
                        "{:<30} #{:>3} {:>14} {:>14} {:>14} NO_BASELINE",
                        r.label, r.job_id.0, actual_per_min, "-", "-"
                    );
                }
            }
            JobOutcome::TimedOut => {
                println!("{:<30} #{:>3} {:>14}", r.label, r.job_id.0, "TIMEOUT");
                regressions.push(format!("{}: timed out", r.label));
            }
            JobOutcome::Failed(e) => {
                println!("{:<30} #{:>3} FAIL: {e}", r.label, r.job_id.0);
                // Connection failures are not regressions against the rate baseline.
            }
        }
    }

    assert!(
        regressions.is_empty(),
        "Regressions vs baseline (tolerance {:.0}%):\n{}",
        TOLERANCE * 100.0,
        regressions.join("\n")
    );

    println!("\nAll streams meet baseline — PASS");
}
