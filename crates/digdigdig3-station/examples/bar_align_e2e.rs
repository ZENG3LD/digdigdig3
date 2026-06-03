//! Live e2e for the bar-aligned non-OHLCV loader (the mlq data-handoff contract).
//!
//! Connects Binance USDⓈ-M futures and pulls each REST-historical non-OHLCV
//! stream as a bar-aligned series, asserting:
//!   * non-empty
//!   * every bar_open_time is aligned to the interval grid (t % step == 0)
//!   * timestamps strictly increasing
//! Flow streams (Liquidation/AggTrade) are asserted to honestly report the
//! daemon requirement (negative check), not silently pass.
//!
//! Exit code 0 only if every Tier-1/2 series passes. Run:
//!   cargo run -p digdigdig3-station --example bar_align_e2e --release

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3_station::{bar_align::load_bar_aligned, BarAlignedSeries};
use digdigdig3_station::series::Kind;

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

fn step_ms(iv: &str) -> i64 {
    let (n, u) = iv.split_at(iv.len() - 1);
    let n: i64 = n.parse().unwrap();
    n * match u {
        "m" => 60_000,
        "h" => 3_600_000,
        "d" => 86_400_000,
        _ => panic!("unexpected interval in test"),
    }
}

/// Validate one bar-aligned series. Returns Ok(count) or Err(reason).
fn check(label: &str, iv: &str, series: &BarAlignedSeries) -> std::result::Result<usize, String> {
    let step = step_ms(iv);
    let times: Vec<i64> = match series {
        BarAlignedSeries::Klines(v) => v.iter().map(|b| b.open_time).collect(),
        BarAlignedSeries::Scalar(v) => v.iter().map(|b| b.bar_open_time).collect(),
    };
    if times.is_empty() {
        return Err(format!("{label}: EMPTY series"));
    }
    for w in times.windows(2) {
        if w[1] <= w[0] {
            return Err(format!("{label}: non-monotonic ts {} -> {}", w[0], w[1]));
        }
    }
    for t in &times {
        if t % step != 0 {
            return Err(format!("{label}: bar {t} not aligned to {iv} grid (step {step})"));
        }
    }
    Ok(times.len())
}

#[tokio::main]
async fn main() {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::Binance, &[AccountType::FuturesCross], false)
        .await
        .expect("connect Binance futures");

    let sym = "BTCUSDT";
    let acct = AccountType::FuturesCross;
    let iv = "1h";
    let interval = KlineInterval::new(iv);
    let end = now_ms();
    let start = end - 3 * 86_400_000; // 3 days — inside the 30d OI/LSR cap

    // (label, kind) — Tier-1/2 REST-historical, all must pass.
    let required = vec![
        ("mark_price_klines", Kind::MarkPriceKline(interval.clone())),
        ("index_price_klines", Kind::IndexPriceKline(interval.clone())),
        ("premium_index_klines", Kind::PremiumIndexKline(interval.clone())),
        ("funding_rate", Kind::FundingRate),
        ("open_interest", Kind::OpenInterest),
        ("long_short_ratio", Kind::LongShortRatio),
        ("mark_price_scalar", Kind::MarkPrice),
    ];

    let mut failures = Vec::new();
    println!("\n=== bar-align e2e — Binance USDⓈ-M {sym} {iv}, {} bars window ===", (end - start) / step_ms(iv));

    for (label, kind) in &required {
        match load_bar_aligned(&hub, ExchangeId::Binance, acct, sym, kind, &interval, start, end).await {
            Ok(series) => match check(label, iv, &series) {
                Ok(n) => {
                    let (first, last) = match &series {
                        BarAlignedSeries::Klines(v) => (
                            format!("close={:.2}", v.first().unwrap().close),
                            format!("close={:.2}", v.last().unwrap().close),
                        ),
                        BarAlignedSeries::Scalar(v) => (
                            format!("v={:.6} filled={}", v.first().unwrap().value, v.first().unwrap().filled),
                            format!("v={:.6} filled={}", v.last().unwrap().value, v.last().unwrap().filled),
                        ),
                    };
                    println!("  OK   {label:<22} {n:>4} bars | first[{first}] last[{last}]");
                }
                Err(e) => {
                    println!("  FAIL {label:<22} {e}");
                    failures.push(e);
                }
            },
            Err(e) => {
                println!("  FAIL {label:<22} loader error: {e}");
                failures.push(format!("{label}: {e}"));
            }
        }
    }

    // Negative check: flow streams must honestly report the daemon requirement.
    for (label, kind) in [("liquidation", Kind::Liquidation), ("agg_trade", Kind::AggTrade)] {
        match load_bar_aligned(&hub, ExchangeId::Binance, acct, sym, &kind, &interval, start, end).await {
            Err(e) if e.is_not_supported() => {
                println!("  OK   {label:<22} correctly daemon-gated ({e})");
            }
            Err(e) => {
                println!("  FAIL {label:<22} wrong error: {e}");
                failures.push(format!("{label}: wrong error {e}"));
            }
            Ok(s) => {
                println!("  FAIL {label:<22} unexpectedly returned {} bars (should be daemon-gated)", s.len());
                failures.push(format!("{label}: not daemon-gated"));
            }
        }
    }

    if failures.is_empty() {
        println!("\nRESULT: PASS — all bar-aligned series valid\n");
    } else {
        eprintln!("\nRESULT: FAIL — {} issue(s):", failures.len());
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}
