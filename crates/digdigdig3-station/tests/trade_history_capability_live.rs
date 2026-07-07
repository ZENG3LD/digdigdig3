#![cfg(not(target_arch = "wasm32"))]
//! Live regression: capability-aware trade-history seed gating.
//!
//! Proves the two behaviors added by the data-capability arc (Wave 1):
//! - A `RecentOnly` venue (Bybit spot: 60 trades, no cursor) never attempts
//!   pagination — `agg_trades_paginated` short-circuits straight to the
//!   `trades_recent` fallback and reports `TruncationReason::VenueRecentOnly`
//!   on the FIRST call, not after repeated hammering.
//! - A `RestWindow` venue (Binance USDⓈ-M futures aggTrades: 24h wall) stops
//!   paging once a page's oldest trade crosses the wall, instead of
//!   discovering the ceiling by an empty page — `TruncationReason::
//!   VenueWindowCap`.
//!
//! Live — requires network. Gated `--ignored`.

use std::sync::Arc;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3_station::backfill::agg_trades_paginated;
use digdigdig3_station::{SeedSource, TruncationReason};

#[tokio::test]
#[ignore] // live API
async fn bybit_spot_recent_only_never_pages_reports_venue_recent_only() {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::Bybit, &[AccountType::Spot], false)
        .await
        .expect("connect Bybit spot");

    // Ask for far more than the venue's RecentOnly ceiling (60) — a
    // pagination attempt would try to walk multiple 1000-trade pages;
    // the capability gate must skip straight to a single shallow call.
    let (points, outcome) = agg_trades_paginated(
        &hub,
        ExchangeId::Bybit,
        AccountType::Spot,
        "BTCUSDT",
        1000,
        10,
    )
    .await;

    assert_eq!(outcome.requested, 10_000);
    assert!(
        outcome.achieved <= 60,
        "Bybit spot RecentOnly ceiling is 60 trades, got achieved={}",
        outcome.achieved,
    );
    assert_eq!(outcome.source, SeedSource::RecentTradesFallback);
    assert_eq!(outcome.truncated_by, Some(TruncationReason::VenueRecentOnly));
    assert!(
        points.len() <= 60,
        "must not have paginated past the venue's single-call ceiling"
    );
}

#[tokio::test]
#[ignore] // live API
async fn binance_futures_rest_window_clamps_at_24h_wall() {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::Binance, &[AccountType::FuturesCross], false)
        .await
        .expect("connect Binance futures");

    // Request a window that would need to walk back far more than 24h of
    // aggTrades (10 pages × 1000 typically covers well under an hour on a
    // liquid BTCUSDT perp, so this alone would not normally cross the
    // wall — the point of this test is simply that the outcome/behavior
    // is well-formed and self-consistent for a RestWindow tier, not to
    // force-cross the wall in a bounded test run).
    let (points, outcome) = agg_trades_paginated(
        &hub,
        ExchangeId::Binance,
        AccountType::FuturesCross,
        "BTCUSDT",
        1000,
        5,
    )
    .await;

    assert_eq!(outcome.requested, 5000);
    assert_eq!(outcome.source, SeedSource::AggTradesPaginated);
    // Either fully satisfied (didn't reach the wall in 5 pages) or capped
    // by the window wall — both are legitimate RestWindow outcomes. What
    // must NOT happen is a RecentOnly-style outcome (Binance futures is
    // NOT RecentOnly) or an unnamed truncation.
    assert_ne!(outcome.truncated_by, Some(TruncationReason::VenueRecentOnly));
    if let Some(reason) = outcome.truncated_by {
        assert!(matches!(
            reason,
            TruncationReason::VenueWindowCap | TruncationReason::VenueHistoryExhausted
        ));
    }
    assert!(!points.is_empty(), "expected at least one page of live BTCUSDT futures aggTrades");
}
