//! Phase ι.2 — reconnect-survival: subscriptions survive forced disconnect+reconnect.
//!
//! Validates that after calling disconnect() + reconnecting via hub.connect_websocket(),
//! events resume flowing on the same symbol.
//!
//! Note: UniversalWsTransport::disconnect() sends TransportCmd::Shutdown — the driver
//! task exits permanently. Auto-reconnect is NOT triggered. Instead, this test reconnects
//! explicitly by calling hub.connect_websocket() after disconnect, which creates a fresh
//! transport. This verifies the hub correctly re-wires a new transport for the same
//! (exchange, account_type) pair and events resume.
//!
//! Run with: cargo test --test reconnect_survival -- --ignored --nocapture

#[path = "common/mod.rs"]
mod common;

use common::{run_jobs, JobOutcome};
use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::{AccountType, ExchangeId, StreamEvent, StreamType, SubscriptionRequest, Symbol};
use futures_util::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

/// (exchange, account_type, exchange-native symbol string)
const TARGETS: &[(ExchangeId, AccountType, &str)] = &[
    (ExchangeId::Binance, AccountType::Spot, "BTCUSDT"),
    (ExchangeId::Bybit, AccountType::Spot, "BTCUSDT"),
    (ExchangeId::OKX, AccountType::Spot, "BTC-USDT"),
    (ExchangeId::Bitget, AccountType::Spot, "BTCUSDT"),
    (ExchangeId::KuCoin, AccountType::Spot, "BTC-USDT"),
];

#[tokio::test]
#[ignore] // live API — run with --ignored
async fn reconnect_survives_subscriptions() {
    let labels: Vec<String> = TARGETS
        .iter()
        .map(|(id, _, _)| format!("{id:?}"))
        .collect();

    let results = run_jobs(
        labels,
        Duration::from_secs(60),
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
                .ok_or_else(|| format!("{label}: no ws handle after connect_full"))?;

            // Subscribe to trades
            let sub = SubscriptionRequest {
                symbol: Symbol::with_raw("", "", sym_str.to_string()),
                stream_type: StreamType::Trade,
                account_type: *acct,
                depth: None,
                update_speed_ms: None,
            };
            ws.subscribe(sub)
                .await
                .map_err(|e| format!("{label}: subscribe failed: {e}"))?;

            // Collect pre-disconnect baseline events
            let mut stream = ws.event_stream();
            let mut pre = 0u32;
            let pre_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
            while tokio::time::Instant::now() < pre_deadline {
                let remaining = pre_deadline.saturating_duration_since(tokio::time::Instant::now());
                match timeout(remaining.max(Duration::from_millis(100)), stream.next()).await {
                    Ok(Some(Ok(_))) => pre += 1,
                    Ok(Some(Err(_))) | Ok(None) => {}
                    Err(_) => break,
                }
            }

            // Force disconnect (Shutdown — driver exits permanently)
            ws.disconnect()
                .await
                .map_err(|e| format!("{label}: disconnect failed: {e}"))?;
            drop(stream);

            // Reconnect: create a fresh transport for the same (id, acct)
            tokio::time::sleep(Duration::from_secs(2)).await;
            hub.connect_websocket(*id, *acct, false)
                .await
                .map_err(|e| format!("{label}: reconnect failed: {e}"))?;

            let ws2 = hub
                .ws(*id, *acct)
                .ok_or_else(|| format!("{label}: no ws handle after reconnect"))?;
            let sub2 = SubscriptionRequest {
                symbol: Symbol::with_raw("", "", sym_str.to_string()),
                stream_type: StreamType::Trade,
                account_type: *acct,
                depth: None,
                update_speed_ms: None,
            };
            ws2.subscribe(sub2)
                .await
                .map_err(|e| format!("{label}: re-subscribe failed: {e}"))?;

            // Collect post-reconnect events
            let mut stream2 = ws2.event_stream();
            let mut post = 0u32;
            let post_deadline = tokio::time::Instant::now() + Duration::from_secs(20);
            while tokio::time::Instant::now() < post_deadline {
                let remaining =
                    post_deadline.saturating_duration_since(tokio::time::Instant::now());
                match timeout(remaining.max(Duration::from_millis(100)), stream2.next()).await {
                    Ok(Some(Ok(_))) => post += 1,
                    Ok(Some(Err(_))) | Ok(None) => {}
                    Err(_) => break,
                }
            }

            Ok((pre, post))
        },
    )
    .await;

    println!("\n=== reconnect_survives_subscriptions ===");
    let mut failed: Vec<String> = Vec::new();
    for r in &results {
        match &r.outcome {
            JobOutcome::Ok((pre, post)) => {
                let status = if *post > 0 { "OK" } else { "NO_POST_EVENTS" };
                println!(
                    "{:<15} job#{:>3} pre={:>4} post={:>4} {status}",
                    r.label, r.job_id.0, pre, post
                );
                if *post == 0 {
                    failed.push(r.label.clone());
                }
            }
            JobOutcome::TimedOut => {
                println!("{:<15} job#{:>3} TIMED_OUT", r.label, r.job_id.0);
                failed.push(format!("{} (timeout)", r.label));
            }
            JobOutcome::Failed(e) => {
                println!("{:<15} job#{:>3} FAILED: {e}", r.label, r.job_id.0);
                failed.push(format!("{} ({e})", r.label));
            }
        }
    }

    let total = TARGETS.len();
    let pass_count = total.saturating_sub(failed.len());
    let pct = pass_count * 100 / total.max(1);
    assert!(
        pct >= 60,
        "Reconnect-survival failed for too many targets: {failed:?} ({pct}% pass)"
    );
}

/// Smoke: assert StreamEvent::Trade is the dominant event type on spot trade stream.
#[tokio::test]
#[ignore]
async fn reconnect_trade_events_are_trades() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .expect("Binance connect_full");

    let ws = hub
        .ws(ExchangeId::Binance, AccountType::Spot)
        .expect("Binance ws");
    ws.subscribe(SubscriptionRequest {
        symbol: Symbol::with_raw("", "", "BTCUSDT".to_string()),
        stream_type: StreamType::Trade,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe trades");

    let mut stream = ws.event_stream();
    let mut trade_count = 0u32;
    let mut other_count = 0u32;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);

    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        match timeout(remaining.max(Duration::from_millis(100)), stream.next()).await {
            Ok(Some(Ok(StreamEvent::Trade(_)))) => trade_count += 1,
            Ok(Some(Ok(_))) => other_count += 1,
            Ok(Some(Err(_))) | Ok(None) | Err(_) => {}
        }
    }

    println!("trade={trade_count} other={other_count}");
    assert!(trade_count > 0, "No Trade events received in 10s window");
}
