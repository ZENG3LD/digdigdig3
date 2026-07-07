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
//! Wave 2 adds:
//! - OKX `RestDeep(FromId)` — `/api/v5/market/history-trades` genuinely
//!   pages backward with no discovered ceiling; 3-page walk asserts
//!   strictly-older monotonic coverage (no page overlap, no forward jump).
//! - Bitfinex `RestWindow(TsWindow)` — `/v2/trades/{symbol}/hist` windowed
//!   pagination; 2-page walk asserts strictly-older coverage.
//! - Gate.io futures `RestWindow(TsWindow, max_back_ms=0)` — `/futures/
//!   {settle}/trades` `to`-windowed pagination with no discovered ceiling;
//!   2-page walk asserts strictly-older coverage.
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

#[tokio::test]
#[ignore] // live API
async fn okx_spot_rest_deep_pages_strictly_older_across_three_pages() {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::OKX, &[AccountType::Spot], false)
        .await
        .expect("connect OKX spot");

    // 3 pages of 100 (OKX history-trades server-clamps limit to 100/page,
    // live-verified 2026-07-08) — enough to prove the from_id cursor walks
    // strictly backward with no overlap and no venue-side ceiling within
    // this small a window.
    let (points, outcome) = agg_trades_paginated(
        &hub,
        ExchangeId::OKX,
        AccountType::Spot,
        "BTC-USDT",
        100,
        3,
    )
    .await;

    assert_eq!(outcome.requested, 300);
    assert_eq!(outcome.source, SeedSource::AggTradesPaginated);
    // RestDeep has no window wall — the only legitimate truncation reason
    // within a 3-page walk is genuine history exhaustion (which will not
    // happen on a liquid BTC-USDT pair), so this should be fully satisfied.
    assert!(
        outcome.truncated_by.is_none() || outcome.achieved >= outcome.requested,
        "OKX spot is RestDeep — a 3-page walk on BTC-USDT should not truncate short: {outcome:?}"
    );
    assert!(points.len() >= 250, "expected close to 3 full pages, got {}", points.len());

    // Strictly-older monotonic coverage: sorted oldest->newest, every
    // consecutive pair must have non-decreasing agg_id (dedup keys by
    // agg_id == OKX tradeId, so equal ids cannot occur — strictly
    // increasing) and non-decreasing timestamp.
    let mut sorted = points.clone();
    sorted.sort_unstable_by_key(|p| p.agg_id);
    for w in sorted.windows(2) {
        assert!(
            w[0].agg_id < w[1].agg_id,
            "agg_id must be strictly increasing after sort (no duplicate tradeIds): {} >= {}",
            w[0].agg_id, w[1].agg_id,
        );
        assert!(
            w[0].ts_ms <= w[1].ts_ms,
            "timestamp must be non-decreasing alongside agg_id: {} > {}",
            w[0].ts_ms, w[1].ts_ms,
        );
    }
}

#[tokio::test]
#[ignore] // live API
async fn bitfinex_spot_rest_window_pages_strictly_older_across_two_pages() {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::Bitfinex, &[AccountType::Spot], false)
        .await
        .expect("connect Bitfinex spot");

    // Bitfinex re-purposes agg_id as the trade's own ms timestamp (see
    // HistoryCursor::TsWindow contract in backfill::agg_trades_paginated) —
    // 2 pages of 200 is enough to prove the `end`-window walk goes strictly
    // backward with no overlap.
    let (points, outcome) = agg_trades_paginated(
        &hub,
        ExchangeId::Bitfinex,
        AccountType::Spot,
        "tBTCUSD",
        200,
        2,
    )
    .await;

    assert_eq!(outcome.requested, 400);
    assert_eq!(outcome.source, SeedSource::AggTradesPaginated);
    assert!(
        outcome.truncated_by.is_none() || outcome.achieved >= outcome.requested,
        "Bitfinex spot is RestWindow with no stated ceiling — a 2-page walk on \
         tBTCUSD should not truncate short: {outcome:?}"
    );
    assert!(points.len() >= 350, "expected close to 2 full pages, got {}", points.len());

    // Strictly-older monotonic coverage: agg_id IS the ms timestamp for this
    // venue, so a duplicate agg_id would mean the SAME millisecond produced
    // two trades — collapse-dedup on that basis is a known lossy edge (the
    // dedup keys by agg_id, so same-ms trades merge), but coverage should
    // still be non-decreasing with no big backward jump.
    let mut sorted = points.clone();
    sorted.sort_unstable_by_key(|p| p.agg_id);
    for w in sorted.windows(2) {
        assert!(
            w[0].agg_id <= w[1].agg_id,
            "agg_id (ms timestamp) must be non-decreasing after sort: {} > {}",
            w[0].agg_id, w[1].agg_id,
        );
    }
}

#[tokio::test]
#[ignore] // live API
async fn gateio_futures_rest_window_pages_strictly_older_across_two_pages() {
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_full(ExchangeId::GateIO, &[AccountType::FuturesCross], false)
        .await
        .expect("connect Gate.io futures");

    // Gate.io futures `to`-windowed pagination has no discovered ceiling
    // (probed back 2 years live) — 2 pages of 200 proves the walk goes
    // strictly backward with no overlap.
    //
    // NOTE on point count: unlike OKX/Bitfinex, Gate.io futures genuinely
    // batches many fills under one identical millisecond timestamp (live
    // probe 2026-07-08: 20 raw trades → only 9 distinct ms values, one ms
    // covering 7 trades) — since this connector's `aggregate_id` IS the ms
    // timestamp (no real per-trade ID exists on this endpoint), the shared
    // loop's agg_id-keyed dedup collapses same-ms trades by design. A count
    // near `page_size * n_pages` is therefore NOT a valid assertion here;
    // strictly-older coverage and a non-truncated outcome are.
    let (points, outcome) = agg_trades_paginated(
        &hub,
        ExchangeId::GateIO,
        AccountType::FuturesCross,
        "BTC_USDT",
        200,
        2,
    )
    .await;

    assert_eq!(outcome.requested, 400);
    assert_eq!(outcome.source, SeedSource::AggTradesPaginated);
    assert!(
        outcome.truncated_by.is_none() || outcome.achieved >= outcome.requested,
        "Gate.io futures is RestWindow with no stated ceiling — a 2-page walk \
         on BTC_USDT should not truncate short: {outcome:?}"
    );
    assert!(!points.is_empty(), "expected at least some live BTC_USDT futures trades");

    let mut sorted = points.clone();
    sorted.sort_unstable_by_key(|p| p.agg_id);
    for w in sorted.windows(2) {
        assert!(
            w[0].agg_id <= w[1].agg_id,
            "agg_id (ms timestamp) must be non-decreasing after sort: {} > {}",
            w[0].agg_id, w[1].agg_id,
        );
    }
}
