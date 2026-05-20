//! Pure-logic gap-heal helpers (no live network).

use std::time::Duration;

use digdigdig3_station::gap_heal::{kline_interval_to_duration, GapHealConfig};

#[test]
fn interval_parsing() {
    assert_eq!(kline_interval_to_duration("1m"), Some(Duration::from_secs(60)));
    assert_eq!(kline_interval_to_duration("5m"), Some(Duration::from_secs(5 * 60)));
    assert_eq!(kline_interval_to_duration("1h"), Some(Duration::from_secs(3600)));
    assert_eq!(kline_interval_to_duration("4h"), Some(Duration::from_secs(4 * 3600)));
    assert_eq!(kline_interval_to_duration("1d"), Some(Duration::from_secs(86400)));
    assert_eq!(kline_interval_to_duration("1w"), Some(Duration::from_secs(7 * 86400)));
    assert_eq!(kline_interval_to_duration("30s"), Some(Duration::from_secs(30)));

    assert_eq!(kline_interval_to_duration("foo"), None);
    assert_eq!(kline_interval_to_duration(""), None);
    assert_eq!(kline_interval_to_duration("xm"), None);
}

#[test]
fn config_builder_chain() {
    let cfg = GapHealConfig::on()
        .trade_gap(Duration::from_secs(30))
        .kline_intervals(5)
        .max_records(200);
    assert!(cfg.enabled);
    assert_eq!(cfg.trade_gap, Duration::from_secs(30));
    assert_eq!(cfg.kline_intervals, 5);
    assert_eq!(cfg.max_records, 200);
}

#[test]
fn default_off() {
    let cfg = GapHealConfig::default();
    assert!(!cfg.enabled);
}
