#![cfg(not(target_arch = "wasm32"))]
//! E2E for the pure auto-heal logic — kline only.
//!
//! Auto-heal model:
//!   - WS disconnect (Err/None) triggers heal IMMEDIATELY, no timestamp threshold.
//!   - REST `get_klines(limit=N)` where N = max(default_limit, ceil(gap/interval)).
//!   - ALL pulled bars upsert (last-write-wins by open_time) into memory + disk.
//!   - Only bars strictly newer than last_emitted_ms get re-emitted to consumer.
//!
//! Trade/OB/Ticker/Mark/Funding/OI/Liquidation: live-only, no REST analog,
//! disconnect = data loss for those streams.

use digdigdig3_station::data::BarPoint;
use digdigdig3_station::gap_heal::{heal_limit, select_heal_window, GapHealConfig};
use digdigdig3_station::{DataPoint, Series};

fn bar(open_time: i64, close: f64) -> BarPoint {
    BarPoint {
        open_time, open: close, high: close, low: close, close,
        volume: 1.0, quote_volume: 0.0, trades_count: 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// heal_limit sizing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn heal_limit_uses_default_when_short_gap() {
    let cfg = GapHealConfig::on().default_limit(300).max_limit(1000);
    // 5 minutes since last bar, interval 1m: need = 5, default = 300 → default wins.
    let n = heal_limit(&cfg, "1m", 1_000_000, 1_000_000 + 5 * 60 * 1000);
    assert_eq!(n, 300);
}

#[test]
fn heal_limit_scales_to_gap_when_long() {
    let cfg = GapHealConfig::on().default_limit(300).max_limit(1000);
    // 500 minutes since last bar at 1m: need = 500, > 300 → 500.
    let n = heal_limit(&cfg, "1m", 1_000_000, 1_000_000 + 500 * 60 * 1000);
    assert_eq!(n, 500);
}

#[test]
fn heal_limit_capped_at_max() {
    let cfg = GapHealConfig::on().default_limit(300).max_limit(1000);
    // 5000 minutes at 1m: need = 5000, capped at max=1000.
    let n = heal_limit(&cfg, "1m", 1_000_000, 1_000_000 + 5000 * 60 * 1000);
    assert_eq!(n, 1000);
}

#[test]
fn heal_limit_fallback_to_default_when_last_unknown() {
    let cfg = GapHealConfig::on().default_limit(300).max_limit(1000);
    // First-ever heal (last_written = 0): no scaling, just default.
    let n = heal_limit(&cfg, "1m", 0, 1_700_000_000_000);
    assert_eq!(n, 300);
}

#[test]
fn heal_limit_handles_malformed_interval() {
    let cfg = GapHealConfig::on().default_limit(300).max_limit(1000);
    let n = heal_limit(&cfg, "foobar", 1_000_000, 1_000_000 + 5 * 60 * 1000);
    assert_eq!(n, 300, "malformed interval falls back to default");
}

#[test]
fn heal_limit_5m_interval_correctly_scaled() {
    let cfg = GapHealConfig::on().default_limit(50).max_limit(1000);
    // 1 hour gap at 5m interval: 60 / 5 = 12 bars. Below default 50 → default.
    let n = heal_limit(&cfg, "5m", 1_000_000, 1_000_000 + 60 * 60 * 1000);
    assert_eq!(n, 50);
    // 5 hour gap: 60 bars > 50 → 60.
    let n2 = heal_limit(&cfg, "5m", 1_000_000, 1_000_000 + 5 * 60 * 60 * 1000);
    assert_eq!(n2, 60);
}

// ─────────────────────────────────────────────────────────────────────────────
// select_heal_window: filter + sort + dedup
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn window_drops_already_seen() {
    let out = select_heal_window(
        vec![bar(900, 1.0), bar(1_000, 2.0), bar(1_100, 3.0), bar(1_200, 4.0)],
        1_000,
    );
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].open_time, 1_100);
    assert_eq!(out[1].open_time, 1_200);
}

#[test]
fn window_sorts_and_dedups() {
    let out = select_heal_window(
        vec![bar(3_000, 1.0), bar(1_500, 2.0), bar(1_500, 9.9), bar(2_000, 3.0)],
        1_000,
    );
    assert_eq!(out.iter().map(|b| b.open_time).collect::<Vec<_>>(), vec![1_500, 2_000, 3_000]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Series::upsert_by_ts — last-write-wins for klines
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn upsert_replaces_broken_live_bar_with_rest() {
    let mut s: Series<BarPoint> = Series::new(10);
    // Live mid-candle update — close looks wrong (low data point).
    s.push(bar(120_000, 50.0));
    // REST returns the canonical bar with the correct close.
    s.upsert_by_ts(bar(120_000, 115.5));
    assert_eq!(s.len(), 1);
    assert_eq!(s.last().unwrap().close, 115.5);
}

#[test]
fn upsert_appends_when_ts_absent() {
    let mut s: Series<BarPoint> = Series::new(10);
    s.push(bar(60_000, 100.0));
    s.upsert_by_ts(bar(120_000, 110.0));
    assert_eq!(s.len(), 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Full forwarder simulation: live + disconnect + heal
// ─────────────────────────────────────────────────────────────────────────────

/// Simulates the auto-heal forwarder. `events` is a sequence of either a
/// live bar (Ok) or a disconnect signal (Err). On disconnect, the simulated
/// REST returns whatever `rest_responder` says.
enum E { Live(BarPoint), Disconnect }

fn simulate(
    cfg: &GapHealConfig,
    interval: &str,
    events: Vec<E>,
    rest_responder: impl Fn(i64) -> Vec<BarPoint>,
) -> (Vec<BarPoint>, Series<BarPoint>) {
    let mut consumer: Vec<BarPoint> = Vec::new();
    let mut series: Series<BarPoint> = Series::new(128);
    let mut last_emitted_ms: i64 = 0;

    for e in events {
        match e {
            E::Live(b) => {
                let ts = b.timestamp_ms();
                series.upsert_by_ts(b.clone());
                last_emitted_ms = last_emitted_ms.max(ts);
                consumer.push(b);
            }
            E::Disconnect => {
                if !cfg.enabled { continue; }
                // Same logic as run_kline_heal:
                let now_ms = last_emitted_ms + 120_000; // simulate "now"
                let limit = heal_limit(cfg, interval, last_emitted_ms, now_ms);
                let pulled = rest_responder(last_emitted_ms);
                let _ = limit; // limit used only for REST sizing; mock ignores
                let new_to_emit = select_heal_window(pulled.clone(), last_emitted_ms);
                for p in pulled {
                    series.upsert_by_ts(p);
                }
                for p in new_to_emit {
                    let ts = p.timestamp_ms();
                    consumer.push(p);
                    last_emitted_ms = last_emitted_ms.max(ts);
                }
            }
        }
    }
    (consumer, series)
}

#[test]
fn disconnect_auto_heals_with_rest_bars() {
    let cfg = GapHealConfig::on();
    let events = vec![
        E::Live(bar(60_000, 100.0)),
        E::Live(bar(120_000, 110.0)),
        E::Disconnect, // WS dropped here — heal triggers
        E::Live(bar(360_000, 200.0)), // post-reconnect
    ];

    // REST returns 5 most-recent bars including the gap fill.
    let rest = |_: i64| {
        vec![
            bar(60_000, 100.0),   // dup
            bar(120_000, 110.0),  // dup
            bar(180_000, 130.0),  // gap fill
            bar(240_000, 140.0),  // gap fill
            bar(300_000, 150.0),  // gap fill
        ]
    };

    let (view, _series) = simulate(&cfg, "1m", events, rest);
    let timestamps: Vec<i64> = view.iter().map(|b| b.open_time).collect();
    assert_eq!(timestamps, vec![60_000, 120_000, 180_000, 240_000, 300_000, 360_000]);
}

#[test]
fn disconnect_with_empty_rest_still_re_attaches() {
    let cfg = GapHealConfig::on();
    let events = vec![
        E::Live(bar(60_000, 100.0)),
        E::Disconnect,
        E::Live(bar(300_000, 200.0)),
    ];
    let (view, _) = simulate(&cfg, "1m", events, |_| Vec::new());
    // No REST fill, but post-disconnect live event still flows.
    let timestamps: Vec<i64> = view.iter().map(|b| b.open_time).collect();
    assert_eq!(timestamps, vec![60_000, 300_000]);
}

#[test]
fn rest_with_corrected_canonical_bar_overwrites_broken_live() {
    let cfg = GapHealConfig::on();
    let events = vec![
        E::Live(bar(60_000, 50.0)),   // broken live: close=50 (wrong)
        E::Disconnect,
        E::Live(bar(120_000, 110.0)),
    ];
    // REST returns the canonical bar at 60_000 with the right close=99.5.
    let rest = |_: i64| vec![bar(60_000, 99.5)];

    let (_view, series) = simulate(&cfg, "1m", events, rest);
    // The series should now hold the canonical bar at 60_000.
    let snap = series.snapshot();
    let bar_60 = snap.into_iter().find(|b| b.open_time == 60_000).unwrap();
    assert_eq!(bar_60.close, 99.5, "REST canonical bar must overwrite broken live");
}

#[test]
fn back_to_back_disconnects_each_trigger_heal() {
    let cfg = GapHealConfig::on();
    let events = vec![
        E::Live(bar(60_000, 1.0)),
        E::Disconnect,
        E::Live(bar(240_000, 2.0)),
        E::Disconnect,
        E::Live(bar(480_000, 3.0)),
    ];

    use std::cell::Cell;
    let calls = Cell::new(0usize);
    let (view, _) = simulate(&cfg, "1m", events, |since| {
        calls.set(calls.get() + 1);
        if since < 100_000 {
            vec![bar(120_000, 1.3), bar(180_000, 1.6)]
        } else {
            vec![bar(300_000, 2.3), bar(360_000, 2.6), bar(420_000, 2.9)]
        }
    });
    assert_eq!(calls.get(), 2, "each disconnect must trigger a REST call");
    let timestamps: Vec<i64> = view.iter().map(|b| b.open_time).collect();
    assert_eq!(
        timestamps,
        vec![60_000, 120_000, 180_000, 240_000, 300_000, 360_000, 420_000, 480_000]
    );
}

#[test]
fn heal_disabled_means_disconnect_is_no_op() {
    let cfg = GapHealConfig::default(); // enabled = false
    let events = vec![
        E::Live(bar(60_000, 1.0)),
        E::Disconnect,
        E::Live(bar(300_000, 2.0)),
    ];

    use std::cell::Cell;
    let calls = Cell::new(0usize);
    let (view, _) = simulate(&cfg, "1m", events, |_| {
        calls.set(calls.get() + 1);
        vec![bar(120_000, 1.3)] // would fill if heal were enabled
    });
    assert_eq!(calls.get(), 0, "disabled config must not call REST");
    let timestamps: Vec<i64> = view.iter().map(|b| b.open_time).collect();
    assert_eq!(timestamps, vec![60_000, 300_000]);
}
