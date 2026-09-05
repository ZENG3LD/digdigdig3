//! Live regression: `OkxConnector::get_liquidation_history` must clamp
//! `limit` to the venue cap (100) instead of forwarding the caller's raw
//! request and getting rejected with "Parameter limit error".
//!
//! Run with: cargo test --release -p digdigdig3 --test okx_liquidation_history_live -- --ignored --nocapture

use digdigdig3::core::traits::MarketDataPublic;
use digdigdig3::core::types::{AccountType, SymbolInput};
use digdigdig3::l3::open::crypto::cex::okx::OkxConnector;

#[tokio::test]
#[ignore]
async fn live_okx_liquidation_history_accepts_large_limit_request() {
    let conn = OkxConnector::public(false, None)
        .await
        .expect("public OKX connector construction must not fail");

    let now_ms = chrono::Utc::now().timestamp_millis();
    let seven_days_ms = 7 * 24 * 60 * 60 * 1000;
    let start_ms = now_ms - seven_days_ms;

    // Mirror the exact live regression: mlq-warmup's bar_align loader asks
    // for up to 1000 rows. Before the clamp fix this was forwarded verbatim
    // to /api/v5/public/liquidation-orders and rejected with "Parameter
    // limit error". After the fix it must page transparently and return at
    // least one row.
    let rows = conn
        .get_liquidation_history(
            Some(SymbolInput::Raw("BTC-USDT-SWAP")),
            Some(start_ms),
            Some(now_ms),
            Some(1000),
            AccountType::FuturesCross,
        )
        .await
        .expect("OKX must not reject the request with a venue parameter error");

    assert!(!rows.is_empty(), "expected at least 1 liquidation row over the last 7 days for BTC-USDT-SWAP");

    let first_ts = rows.iter().map(|r| r.timestamp).min().expect("non-empty");
    let last_ts = rows.iter().map(|r| r.timestamp).max().expect("non-empty");
    println!(
        "OKX BTC-USDT-SWAP liquidation-orders (last 7d, limit=1000 requested): {} rows, first_ts={} last_ts={}",
        rows.len(),
        first_ts,
        last_ts
    );
}
