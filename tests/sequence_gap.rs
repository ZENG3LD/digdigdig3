//! Phase ι.4 — orderbook delta continuity / sequence-gap test.
//!
//! Validates that orderbook delta streams produce contiguous sequence numbers.
//! Gap detection uses `OrderbookDelta::last_update_id` (u64, Option).
//!
//! Per-exchange sequence field mapping:
//!   Binance: last_update_id (u) — final update ID of delta message.
//!   OKX:     last_update_id — seq_id exposed as last_update_id in parser.
//!   Bybit:   last_update_id — update_id / u exposed as last_update_id in parser.
//!
//! If an exchange parser does not fill last_update_id, total > 0 is asserted only
//! and gap detection is skipped (documented as deferred in that exchange's parser).
//!
//! Run with: cargo test --test sequence_gap -- --ignored --nocapture

#[path = "common/mod.rs"]
mod common;

use common::{run_jobs, JobOutcome};
use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::{AccountType, ExchangeId, StreamEvent};
use futures_util::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

/// (exchange, account_type, exchange-native symbol)
const TARGETS: &[(ExchangeId, AccountType, &str)] = &[
    (ExchangeId::Binance, AccountType::Spot, "BTCUSDT"),
    (ExchangeId::OKX, AccountType::Spot, "BTC-USDT"),
    (ExchangeId::Bybit, AccountType::Spot, "BTCUSDT"),
];

#[tokio::test]
#[ignore] // live API — run with --ignored
async fn orderbook_delta_no_gaps() {
    let labels: Vec<String> = TARGETS
        .iter()
        .map(|(id, _, _)| format!("{id:?}"))
        .collect();

    let results = run_jobs(
        labels,
        Duration::from_secs(45),
        |_job_id, label| async move {
            let (id, acct, sym_str) = TARGETS
                .iter()
                .find(|(id, _, _)| format!("{id:?}") == label)
                .ok_or_else(|| format!("unknown target: {label}"))?;

            let hub = ExchangeHub::new();
            hub.connect_full(*id, &[*acct], false)
                .await
                .map_err(|e| e.to_string())?;

            let ws = hub
                .ws(*id, *acct)
                .ok_or_else(|| format!("{label}: no ws handle"))?;

            // Subscribe to orderbook delta stream
            ws.subscribe(digdigdig3::SubscriptionRequest {
                symbol: digdigdig3::Symbol::with_raw("", "", sym_str.to_string()),
                stream_type: digdigdig3::StreamType::OrderbookDelta,
                account_type: *acct,
                depth: None,
                update_speed_ms: None,
            })
            .await
            .map_err(|e| format!("{label}: subscribe failed: {e}"))?;

            let mut stream = ws.event_stream();
            let mut last_seq: Option<u64> = None;
            let mut gaps = 0u64;
            let mut total = 0u64;
            // Track whether last_update_id was ever populated
            let mut seq_available = false;

            let deadline = tokio::time::Instant::now() + Duration::from_secs(25);
            while tokio::time::Instant::now() < deadline {
                let remaining =
                    deadline.saturating_duration_since(tokio::time::Instant::now());
                match timeout(remaining.max(Duration::from_millis(100)), stream.next()).await {
                    Ok(Some(Ok(StreamEvent::OrderbookDelta(d)))) => {
                        total += 1;
                        // last_update_id is the canonical sequence field populated by
                        // parsers for Binance (U/u fields), OKX (seqId), Bybit (u).
                        if let Some(seq) = d.last_update_id {
                            seq_available = true;
                            if let Some(prev) = last_seq {
                                // Allow +1 contiguity; allow skip if first_update_id > prev+1
                                // (some exchanges may legitimately skip on snapshot boundary).
                                let first = d.first_update_id.unwrap_or(seq);
                                if first > prev + 1 {
                                    gaps += 1;
                                }
                            }
                            last_seq = Some(seq);
                        }
                    }
                    Ok(Some(Ok(_))) => {} // other event types, ignore
                    Ok(Some(Err(_))) | Ok(None) => {}
                    Err(_) => break,
                }
            }

            Ok((total, gaps, seq_available))
        },
    )
    .await;

    println!("\n=== orderbook_delta_no_gaps ===");
    for r in &results {
        match &r.outcome {
            JobOutcome::Ok((total, gaps, seq_available)) => {
                if *seq_available {
                    let gap_pct = if *total > 0 { gaps * 100 / total } else { 0 };
                    println!(
                        "{:<15} job#{:>3} total={total} gaps={gaps} gap%={gap_pct}%",
                        r.label, r.job_id.0
                    );
                    assert!(
                        *total > 0,
                        "{}: zero orderbook deltas received in 25s",
                        r.label
                    );
                    assert!(
                        gap_pct < 5,
                        "{}: gap rate too high ({gaps}/{total} = {gap_pct}%)",
                        r.label
                    );
                } else {
                    // Sequence field not populated by parser — assert events flow only.
                    // Gap detection deferred: parser does not fill last_update_id for this exchange.
                    println!(
                        "{:<15} job#{:>3} total={total} seq=N/A (gap detection deferred)",
                        r.label, r.job_id.0
                    );
                    assert!(
                        *total > 0,
                        "{}: zero orderbook deltas received in 25s (seq detection deferred)",
                        r.label
                    );
                }
            }
            JobOutcome::TimedOut => {
                panic!("{}: timed out waiting for orderbook deltas", r.label)
            }
            JobOutcome::Failed(e) => {
                panic!("{}: {e}", r.label)
            }
        }
    }
}
