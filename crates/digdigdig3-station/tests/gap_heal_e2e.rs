//! E2E for the pure gap-heal decision + window-selection logic.
//!
//! These tests simulate the exact sequence the live forwarder follows:
//! - some live points arrive,
//! - a long-jump live point comes in,
//! - `should_heal` returns true,
//! - the forwarder calls REST (mocked here as a Vec),
//! - `select_heal_window` returns ONLY the truly-missing points,
//! - replay the sequence and confirm the consumer would see a continuous
//!   timeline.

use std::time::Duration;

use digdigdig3_station::data::{BarPoint, TradePoint};
use digdigdig3_station::gap_heal::{select_heal_window, should_heal, GapHealConfig};
use digdigdig3_station::{DataPoint, Kind};

fn trade(ts_ms: i64, price: f64) -> TradePoint {
    TradePoint { ts_ms, price, quantity: 1.0, side: 0, trade_id_hash: ts_ms as u64 }
}
fn bar(open_time: i64, close: f64) -> BarPoint {
    BarPoint {
        open_time,
        open: close, high: close, low: close, close,
        volume: 1.0, quote_volume: 0.0, trades_count: 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// should_heal decision matrix
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn should_heal_off_when_config_disabled() {
    let cfg = GapHealConfig::default();
    assert!(!cfg.enabled);
    // Even a huge gap doesn't trigger when off.
    assert!(!should_heal(&Kind::Trade, 1_000, 1_000_000, &cfg));
}

#[test]
fn should_heal_off_on_first_event() {
    let cfg = GapHealConfig::on();
    // last_seen_ms == 0 means we never received a live event yet.
    assert!(!should_heal(&Kind::Trade, 0, 1_700_000_000_000, &cfg));
}

#[test]
fn should_heal_off_on_clock_skew_or_duplicate() {
    let cfg = GapHealConfig::on();
    // now <= last_seen: should NOT heal (would loop forever).
    assert!(!should_heal(&Kind::Trade, 1_000, 1_000, &cfg));
    assert!(!should_heal(&Kind::Trade, 1_000, 999, &cfg));
}

#[test]
fn should_heal_trade_below_threshold() {
    let cfg = GapHealConfig::on().trade_gap(Duration::from_secs(10));
    // 5-second gap < 10s threshold.
    assert!(!should_heal(&Kind::Trade, 1_000, 1_000 + 5_000, &cfg));
    // Exactly equal — not a gap (>, not >=).
    assert!(!should_heal(&Kind::Trade, 1_000, 1_000 + 10_000, &cfg));
}

#[test]
fn should_heal_trade_above_threshold() {
    let cfg = GapHealConfig::on().trade_gap(Duration::from_secs(10));
    // 13-second gap > 10s.
    assert!(should_heal(&Kind::Trade, 1_000, 1_000 + 13_000, &cfg));
    // 30-second gap.
    assert!(should_heal(&Kind::Trade, 1_000, 1_000 + 30_000, &cfg));
}

#[test]
fn should_heal_kline_uses_interval_multiplier() {
    let cfg = GapHealConfig::on().kline_intervals(3);
    // 1m kline: threshold = 1m * 3 = 180s.
    let kind = Kind::Kline("1m".into());
    assert!(!should_heal(&kind, 1_000, 1_000 + 120_000, &cfg)); // 2min gap < 3min
    assert!(should_heal(&kind, 1_000, 1_000 + 200_000, &cfg)); // 3.3min gap > 3min
}

#[test]
fn should_heal_kline_malformed_interval_returns_false() {
    let cfg = GapHealConfig::on();
    let kind = Kind::Kline("foobar".into());
    assert!(!should_heal(&kind, 1_000, 1_000_000_000, &cfg));
}

#[test]
fn should_heal_off_for_kinds_without_rest_history() {
    let cfg = GapHealConfig::on();
    // None of these has a public REST history endpoint useful for backfill.
    for k in [
        Kind::Ticker,
        Kind::Orderbook,
        Kind::MarkPrice,
        Kind::FundingRate,
        Kind::OpenInterest,
        Kind::Liquidation,
        Kind::AggTrade,
    ] {
        assert!(!should_heal(&k, 1_000, 1_000_000_000, &cfg), "{:?} must not heal", k);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// select_heal_window: filter + sort + dedup
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn select_window_drops_already_seen() {
    let last_seen = 1_000;
    let pulled = vec![
        trade(900, 1.0),  // already seen — drop
        trade(1_000, 2.0), // exactly last_seen — drop
        trade(1_100, 3.0), // new — keep
        trade(1_200, 4.0), // new — keep
    ];
    let out = select_heal_window(pulled, last_seen);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].ts_ms, 1_100);
    assert_eq!(out[1].ts_ms, 1_200);
}

#[test]
fn select_window_sorts_oldest_first() {
    let pulled = vec![trade(3_000, 1.0), trade(1_500, 2.0), trade(2_000, 3.0)];
    let out = select_heal_window(pulled, 1_000);
    assert_eq!(out.iter().map(|p| p.ts_ms).collect::<Vec<_>>(), vec![1_500, 2_000, 3_000]);
}

#[test]
fn select_window_dedups_by_ts() {
    let pulled = vec![
        trade(1_500, 1.0),
        trade(1_500, 2.0), // duplicate ts after sort
        trade(2_000, 3.0),
    ];
    let out = select_heal_window(pulled, 1_000);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].ts_ms, 1_500);
    assert_eq!(out[1].ts_ms, 2_000);
}

#[test]
fn select_window_empty_when_all_seen() {
    let out = select_heal_window(vec![trade(500, 1.0), trade(900, 2.0)], 1_000);
    assert!(out.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// E2E SIMULATION — replays the forwarder's decision loop without a Station.
// Demonstrates that a 13s gap between live events gets filled by REST.
// ─────────────────────────────────────────────────────────────────────────────

/// Simulates the forwarder logic for one stream key. Returns the FULL sequence
/// of points the consumer would see (warm-start + live + healed).
fn simulate_consumer_view<T: DataPoint>(
    kind: Kind,
    cfg: &GapHealConfig,
    live_events: Vec<T>,
    rest_responder: impl Fn(i64) -> Vec<T>,
) -> Vec<T> {
    let mut consumer: Vec<T> = Vec::new();
    let mut last_seen_ms: i64 = 0;

    for live in live_events {
        let now_ts = live.timestamp_ms();
        if should_heal(&kind, last_seen_ms, now_ts, cfg) {
            let pulled = rest_responder(last_seen_ms);
            let healed = select_heal_window(pulled, last_seen_ms);
            for h in healed {
                let h_ts = h.timestamp_ms();
                consumer.push(h);
                last_seen_ms = last_seen_ms.max(h_ts);
            }
        }
        last_seen_ms = last_seen_ms.max(now_ts);
        consumer.push(live);
    }
    consumer
}

#[test]
fn e2e_trade_13s_gap_filled_by_rest() {
    let cfg = GapHealConfig::on().trade_gap(Duration::from_secs(10));

    // Live: t=1000, t=2000, then jump to t=15000 (13s gap > 10s threshold).
    let live = vec![trade(1_000, 70_000.0), trade(2_000, 70_010.0), trade(15_000, 70_500.0)];

    // REST returns: trades at 3000, 5000, 8000, 12000 (covers most of the gap)
    // plus duplicates of already-seen points (REST `get_recent_trades` returns
    // the latest N regardless of time window).
    let rest_responder = |_since_ms: i64| {
        vec![
            trade(1_000, 70_000.0),  // duplicate of live[0]
            trade(2_000, 70_010.0),  // duplicate of live[1]
            trade(3_000, 70_050.0),  // GAP fill
            trade(5_000, 70_100.0),  // GAP fill
            trade(8_000, 70_200.0),  // GAP fill
            trade(12_000, 70_400.0), // GAP fill
        ]
    };

    let view = simulate_consumer_view(Kind::Trade, &cfg, live, rest_responder);
    let timestamps: Vec<i64> = view.iter().map(|t| t.ts_ms).collect();

    // Consumer must see: 1000, 2000, [3000, 5000, 8000, 12000 from REST], 15000.
    // No duplicates of 1000/2000.
    assert_eq!(timestamps, vec![1_000, 2_000, 3_000, 5_000, 8_000, 12_000, 15_000]);
}

#[test]
fn e2e_trade_no_gap_no_heal_calls() {
    let cfg = GapHealConfig::on().trade_gap(Duration::from_secs(10));

    // All live events within 1s of each other — no gap.
    let live = vec![trade(1_000, 1.0), trade(1_500, 2.0), trade(2_000, 3.0), trade(2_300, 4.0)];

    use std::cell::Cell;
    let calls = Cell::new(0usize);
    let view = simulate_consumer_view::<TradePoint>(Kind::Trade, &cfg, live, |_since_ms: i64| {
        calls.set(calls.get() + 1);
        vec![trade(1_700, 99.0)] // would be wrongly inserted if heal triggered
    });

    let timestamps: Vec<i64> = view.iter().map(|t| t.ts_ms).collect();
    assert_eq!(timestamps, vec![1_000, 1_500, 2_000, 2_300]);
    assert_eq!(calls.get(), 0, "REST must not be called when no gap detected");
}

#[test]
fn e2e_kline_3m_gap_filled() {
    // 1m kline, threshold = 3 intervals = 180s gap.
    let cfg = GapHealConfig::on().kline_intervals(3);
    let kind = Kind::Kline("1m".into());

    // Live: bars at t=60000, 120000, then jump to 360000 (4 minutes gap).
    let live = vec![bar(60_000, 70_000.0), bar(120_000, 70_100.0), bar(360_000, 70_500.0)];

    // REST returns last 6 minute bars (some overlap with live).
    let rest_responder = |_: i64| {
        vec![
            bar(60_000, 70_000.0),  // duplicate
            bar(120_000, 70_100.0), // duplicate
            bar(180_000, 70_200.0), // GAP
            bar(240_000, 70_300.0), // GAP
            bar(300_000, 70_400.0), // GAP
        ]
    };

    let view = simulate_consumer_view(kind, &cfg, live, rest_responder);
    let timestamps: Vec<i64> = view.iter().map(|b| b.open_time).collect();
    assert_eq!(timestamps, vec![60_000, 120_000, 180_000, 240_000, 300_000, 360_000]);
}

#[test]
fn e2e_kline_under_threshold_does_not_heal() {
    let cfg = GapHealConfig::on().kline_intervals(3);
    let kind = Kind::Kline("1m".into());

    // Bars 60s apart — no gap.
    let live = vec![bar(60_000, 1.0), bar(120_000, 2.0), bar(180_000, 3.0)];

    use std::cell::Cell;
    let calls = Cell::new(0usize);
    let view = simulate_consumer_view(kind, &cfg, live, |_| {
        calls.set(calls.get() + 1);
        vec![]
    });

    assert_eq!(view.iter().map(|b| b.open_time).collect::<Vec<_>>(),
               vec![60_000, 120_000, 180_000]);
    assert_eq!(calls.get(), 0);
}

#[test]
fn e2e_rest_empty_response_does_not_break_live() {
    let cfg = GapHealConfig::on().trade_gap(Duration::from_secs(10));
    let live = vec![trade(1_000, 1.0), trade(20_000, 2.0)];

    // REST is down / returns nothing. Consumer must still see live events,
    // just without gap filling.
    let view = simulate_consumer_view(Kind::Trade, &cfg, live, |_| Vec::new());
    assert_eq!(view.iter().map(|t| t.ts_ms).collect::<Vec<_>>(), vec![1_000, 20_000]);
}

#[test]
fn e2e_rest_returns_only_already_seen_no_op() {
    let cfg = GapHealConfig::on().trade_gap(Duration::from_secs(10));
    let live = vec![trade(1_000, 1.0), trade(20_000, 2.0)];

    // REST returns only stale data — should be filtered out by select_heal_window.
    let view = simulate_consumer_view(Kind::Trade, &cfg, live, |_| {
        vec![trade(500, 0.0), trade(1_000, 1.0)]
    });
    assert_eq!(view.iter().map(|t| t.ts_ms).collect::<Vec<_>>(), vec![1_000, 20_000]);
}

#[test]
fn e2e_back_to_back_gaps() {
    // Two gaps in sequence — each must trigger an independent heal.
    let cfg = GapHealConfig::on().trade_gap(Duration::from_secs(5));
    let live = vec![
        trade(1_000, 1.0),
        trade(10_000, 2.0), // 9s gap > 5s
        trade(25_000, 3.0), // 15s gap > 5s
    ];

    use std::cell::Cell;
    let calls = Cell::new(0usize);
    let view = simulate_consumer_view(Kind::Trade, &cfg, live, |since| {
        calls.set(calls.get() + 1);
        // Return points in the gap for whichever heal call.
        if since < 5_000 {
            vec![trade(3_000, 1.5), trade(7_000, 1.8)]
        } else {
            vec![trade(13_000, 2.3), trade(20_000, 2.7)]
        }
    });

    assert_eq!(calls.get(), 2, "two distinct gaps should yield two REST calls");
    assert_eq!(
        view.iter().map(|t| t.ts_ms).collect::<Vec<_>>(),
        vec![1_000, 3_000, 7_000, 10_000, 13_000, 20_000, 25_000]
    );
}
